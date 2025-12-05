//! 插件注册表
//!
//! 管理所有已注册的插件。

use super::{EnginePlugin, App, PluginMetadata, PluginDependency};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Duplicate plugin: {0}")]
    DuplicatePlugin(String),
    #[error("Missing dependency: {0} requires {1}")]
    MissingDependency(String, String),
    #[error("Version conflict: {0} conflicts with {1}")]
    VersionConflict(String, String),
}

pub type PluginResult<T> = Result<T, PluginError>;

pub struct PluginRegistry {
    plugins: Vec<Box<dyn EnginePlugin>>,
    metadata: HashMap<String, PluginMetadata>,
    dependency_graph: HashMap<String, Vec<String>>,
}

#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn EnginePlugin>>,
    metadata: HashMap<String, PluginMetadata>,
    dependency_graph: HashMap<String, Vec<String>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加插件
    pub fn add<P: EnginePlugin + 'static>(&mut self, plugin: P) -> PluginResult<&mut Self> {
        let metadata = plugin.metadata();
        let name = metadata.name.clone();

        // 检查重复插件
        if self.metadata.contains_key(&name) {
            return Err(PluginError::DuplicatePlugin(name));
        }

        // 检查依赖
        self.check_dependencies(&metadata)?;

        // 添加到注册表
        self.metadata.insert(name.clone(), metadata);
        self.plugins.push(Box::new(plugin));

        // 构建依赖图
        self.build_dependency_graph();

        Ok(self)
    }

    /// 检查插件依赖
    fn check_dependencies(&self, metadata: &PluginMetadata) -> PluginResult<()> {
        for dep in &metadata.dependencies {
            if !self.metadata.contains_key(&dep.name) {
                return Err(PluginError::MissingDependency(
                    metadata.name.clone(),
                    dep.name.clone(),
                ));
            }
            // 这里可以添加版本检查逻辑
        }
        Ok(())
    }

    /// 构建依赖图
    fn build_dependency_graph(&mut self) {
        self.dependency_graph.clear();
        for (name, metadata) in &self.metadata {
            let deps: Vec<String> = metadata.dependencies.iter()
                .map(|d| d.name.clone())
                .collect();
            self.dependency_graph.insert(name.clone(), deps);
        }
    }

    /// 获取拓扑排序的插件列表
    fn get_topological_order(&self) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        let mut order = Vec::new();

        fn visit(
            node: &str,
            dependency_graph: &HashMap<String, Vec<String>>,
            visited: &mut HashSet<String>,
            temp_visited: &mut HashSet<String>,
            order: &mut Vec<String>,
        ) {
            if visited.contains(node) {
                return;
            }
            if temp_visited.contains(node) {
                // 循环依赖，这里简化处理
                return;
            }

            temp_visited.insert(node.to_string());

            if let Some(deps) = dependency_graph.get(node) {
                for dep in deps {
                    visit(dep, dependency_graph, visited, temp_visited, order);
                }
            }

            temp_visited.remove(node);
            visited.insert(node.to_string());
            order.push(node.to_string());
        }

        for node in self.dependency_graph.keys() {
            if !visited.contains(node) {
                visit(node, &self.dependency_graph, &mut visited, &mut temp_visited, &mut order);
            }
        }

        order
    }

    /// 构建所有插件（按依赖顺序）
    pub fn build_all(&self, app: &mut App) {
        let order = self.get_topological_order();
        for name in order {
            if let Some(plugin) = self.plugins.iter().find(|p| p.name() == name) {
                plugin.build(app);
            }
        }
    }

    /// 启动所有插件
    pub fn startup_all(&self, world: &mut bevy_ecs::world::World) {
        for plugin in &self.plugins {
            plugin.startup(world);
        }
    }

    /// 更新所有插件
    pub fn update_all(&self, world: &mut bevy_ecs::world::World) {
        for plugin in &self.plugins {
            plugin.update(world);
        }
    }

    /// 关闭所有插件
    pub fn shutdown_all(&self, world: &mut bevy_ecs::world::World) {
        // 反向顺序关闭
        for plugin in self.plugins.iter().rev() {
            plugin.shutdown(world);
        }
    }

    /// 获取插件列表
    pub fn list_plugins(&self) -> Vec<&PluginMetadata> {
        self.metadata.values().collect()
    }

    /// 检查插件是否已注册
    pub fn has_plugin(&self, name: &str) -> bool {
        self.metadata.contains_key(name)
    }
    
    /// 移除插件（用于热重载）
    pub fn remove_plugin(&mut self, name: &str) -> PluginResult<()> {
        if !self.metadata.contains_key(name) {
            return Err(PluginError::MissingDependency("".to_string(), name.to_string()));
        }
        
        // 移除元数据
        self.metadata.remove(name);
        
        // 移除插件实例
        self.plugins.retain(|p| p.name() != name);
        
        // 重建依赖图
        self.build_dependency_graph();
        
        Ok(())
    }
    
    /// 获取插件数量
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}
