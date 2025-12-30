//! 资源依赖管理系统
//!
//! 提供资源依赖关系的管理和解析功能，支持：
//! - 依赖图构建和遍历
//! - 依赖解析和加载顺序确定
//! - 循环依赖检测
//! - 依赖预加载

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

/// 资源依赖错误
#[derive(Debug, Error, Clone)]
pub enum DependencyError {
    /// 循环依赖
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
    /// 依赖资源不存在
    #[error("Dependency not found: {0}")]
    DependencyNotFound(String),
    /// 依赖解析失败
    #[error("Failed to resolve dependency: {0}")]
    ResolutionFailed(String),
}

/// 资源依赖关系
#[derive(Debug, Clone)]
pub struct ResourceDependency {
    /// 资源路径
    pub path: PathBuf,
    /// 依赖类型（如：texture, shader, model等）
    pub dependency_type: String,
    /// 是否必需（必需依赖必须在资源加载前完成）
    pub required: bool,
}

/// 资源依赖节点
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// 资源路径
    pub path: PathBuf,
    /// 直接依赖列表
    pub dependencies: Vec<ResourceDependency>,
    /// 依赖此资源的资源列表（反向依赖）
    pub dependents: Vec<PathBuf>,
    /// 加载状态
    pub load_state: LoadState,
}

/// 资源加载状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    /// 未加载
    NotLoaded,
    /// 加载中
    Loading,
    /// 已加载
    Loaded,
    /// 加载失败
    Failed,
}

/// 资源依赖图
pub struct DependencyGraph {
    /// 节点映射：路径 -> 节点
    nodes: HashMap<PathBuf, DependencyNode>,
    /// 资源路径解析器（用于解析相对路径）
    path_resolver: Arc<dyn Fn(&Path, &Path) -> PathBuf + Send + Sync>,
}

impl DependencyGraph {
    /// 创建新的依赖图
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            path_resolver: Arc::new(|base: &Path, relative: &Path| {
                base.parent().unwrap_or_else(|| Path::new(".")).join(relative)
            }),
        }
    }

    /// 使用自定义路径解析器创建
    pub fn with_path_resolver<F>(resolver: F) -> Self
    where
        F: Fn(&Path, &Path) -> PathBuf + Send + Sync + 'static,
    {
        Self {
            nodes: HashMap::new(),
            path_resolver: Arc::new(resolver),
        }
    }

    /// 添加资源节点
    pub fn add_resource(&mut self, path: PathBuf) {
        self.nodes.entry(path.clone()).or_insert_with(|| DependencyNode {
            path: path.clone(),
            dependencies: Vec::new(),
            dependents: Vec::new(),
            load_state: LoadState::NotLoaded,
        });
    }

    /// 添加依赖关系
    ///
    /// # 参数
    ///
    /// * `resource_path` - 资源路径
    /// * `dependency` - 依赖资源
    ///
    /// # 错误
    ///
    /// 如果检测到循环依赖，返回错误。
    pub fn add_dependency(
        &mut self,
        resource_path: PathBuf,
        dependency: ResourceDependency,
    ) -> Result<(), DependencyError> {
        // 确保两个节点都存在
        self.add_resource(resource_path.clone());

        // 解析依赖路径
        let resolved_dep_path = (self.path_resolver)(&resource_path, &dependency.path);
        self.add_resource(resolved_dep_path.clone());

        // 检查循环依赖
        if self.would_create_cycle(&resource_path, &resolved_dep_path) {
            return Err(DependencyError::CircularDependency(format!(
                "{} -> {}",
                resource_path.display(),
                resolved_dep_path.display()
            )));
        }

        // 添加依赖关系
        if let Some(node) = self.nodes.get_mut(&resource_path)
            && !node.dependencies.iter().any(|d| d.path == resolved_dep_path)
        {
            node.dependencies.push(ResourceDependency {
                path: resolved_dep_path.clone(),
                dependency_type: dependency.dependency_type,
                required: dependency.required,
            });
        }

        // 添加反向依赖
        if let Some(dep_node) = self.nodes.get_mut(&resolved_dep_path)
            && !dep_node.dependents.contains(&resource_path)
        {
            dep_node.dependents.push(resource_path);
        }

        Ok(())
    }

    /// 检查是否会创建循环依赖
    fn would_create_cycle(&self, from: &PathBuf, to: &PathBuf) -> bool {
        if from == to {
            return true;
        }

        // 使用DFS检查是否存在从to到from的路径
        let mut visited = HashSet::new();
        self.has_path_to(to, from, &mut visited)
    }

    /// 检查是否存在从start到target的路径
    fn has_path_to(
        &self,
        start: &PathBuf,
        target: &PathBuf,
        visited: &mut HashSet<PathBuf>,
    ) -> bool {
        if start == target {
            return true;
        }

        if visited.contains(start) {
            return false;
        }

        visited.insert(start.clone());

        if let Some(node) = self.nodes.get(start) {
            for dep in &node.dependencies {
                if self.has_path_to(&dep.path, target, visited) {
                    return true;
                }
            }
        }

        false
    }

    /// 获取资源的加载顺序（拓扑排序）
    ///
    /// 返回按依赖顺序排序的资源路径列表，确保依赖资源在依赖它们的资源之前加载。
    ///
    /// # 错误
    ///
    /// 如果存在循环依赖，返回错误。
    pub fn get_load_order(&self) -> Result<Vec<PathBuf>, DependencyError> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_mark = HashSet::new();

        for path in self.nodes.keys() {
            if !visited.contains(path) {
                self.topological_sort(path, &mut visited, &mut temp_mark, &mut result)?;
            }
        }

        result.reverse(); // 反转得到正确的加载顺序
        Ok(result)
    }

    /// 拓扑排序（DFS）
    fn topological_sort(
        &self,
        path: &PathBuf,
        visited: &mut HashSet<PathBuf>,
        temp_mark: &mut HashSet<PathBuf>,
        result: &mut Vec<PathBuf>,
    ) -> Result<(), DependencyError> {
        if temp_mark.contains(path) {
            return Err(DependencyError::CircularDependency(format!(
                "Circular dependency detected involving: {}",
                path.display()
            )));
        }

        if visited.contains(path) {
            return Ok(());
        }

        temp_mark.insert(path.clone());

        if let Some(node) = self.nodes.get(path) {
            for dep in &node.dependencies {
                if dep.required {
                    self.topological_sort(&dep.path, visited, temp_mark, result)?;
                }
            }
        }

        temp_mark.remove(path);
        visited.insert(path.clone());
        result.push(path.clone());

        Ok(())
    }

    /// 获取资源的所有依赖（递归）
    ///
    /// 返回资源及其所有依赖的路径列表。
    pub fn get_all_dependencies(&self, resource_path: &PathBuf) -> Vec<PathBuf> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        self.collect_dependencies(resource_path, &mut visited, &mut result);
        result
    }

    /// 递归收集依赖
    fn collect_dependencies(
        &self,
        path: &PathBuf,
        visited: &mut HashSet<PathBuf>,
        result: &mut Vec<PathBuf>,
    ) {
        if visited.contains(path) {
            return;
        }

        visited.insert(path.clone());

        if let Some(node) = self.nodes.get(path) {
            for dep in &node.dependencies {
                if dep.required {
                    result.push(dep.path.clone());
                    self.collect_dependencies(&dep.path, visited, result);
                }
            }
        }
    }

    /// 获取依赖资源的资源列表（反向依赖）
    pub fn get_dependents(&self, resource_path: &PathBuf) -> Vec<PathBuf> {
        self.nodes
            .get(resource_path)
            .map(|node| node.dependents.clone())
            .unwrap_or_default()
    }

    /// 更新资源加载状态
    pub fn set_load_state(&mut self, path: &PathBuf, state: LoadState) {
        if let Some(node) = self.nodes.get_mut(path) {
            node.load_state = state;
        }
    }

    /// 获取资源加载状态
    pub fn get_load_state(&self, path: &PathBuf) -> Option<LoadState> {
        self.nodes.get(path).map(|node| node.load_state)
    }

    /// 检查资源是否可以加载（所有必需依赖是否已加载）
    pub fn can_load(&self, resource_path: &PathBuf) -> bool {
        if let Some(node) = self.nodes.get(resource_path) {
            for dep in &node.dependencies {
                if dep.required {
                    if let Some(dep_node) = self.nodes.get(&dep.path) {
                        if dep_node.load_state != LoadState::Loaded {
                            return false;
                        }
                    } else {
                        return false; // 依赖节点不存在
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// 清除所有节点
    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new();

        let resource = PathBuf::from("resource.txt");
        let dependency = ResourceDependency {
            path: PathBuf::from("dependency.txt"),
            dependency_type: "texture".to_string(),
            required: true,
        };

        assert!(graph.add_dependency(resource.clone(), dependency).is_ok());
        assert_eq!(graph.node_count(), 2);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = DependencyGraph::new();

        let a = PathBuf::from("a.txt");
        let b = PathBuf::from("b.txt");

        graph
            .add_dependency(
                a.clone(),
                ResourceDependency {
                    path: b.clone(),
                    dependency_type: "texture".to_string(),
                    required: true,
                },
            )
            .expect("Test: operation should succeed");

        // 尝试创建循环依赖
        let result = graph.add_dependency(
            b.clone(),
            ResourceDependency {
                path: a.clone(),
                dependency_type: "texture".to_string(),
                required: true,
            },
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DependencyError::CircularDependency(_)
        ));
    }

    #[test]
    fn test_load_order() {
        let mut graph = DependencyGraph::new();

        // 创建依赖链: a -> b -> c
        let a = PathBuf::from("a.txt");
        let b = PathBuf::from("b.txt");
        let c = PathBuf::from("c.txt");

        graph
            .add_dependency(
                a.clone(),
                ResourceDependency {
                    path: b.clone(),
                    dependency_type: "texture".to_string(),
                    required: true,
                },
            )
            .expect("Test: operation should succeed");

        graph
            .add_dependency(
                b.clone(),
                ResourceDependency {
                    path: c.clone(),
                    dependency_type: "texture".to_string(),
                    required: true,
                },
            )
            .expect("Test: operation should succeed");

        let load_order = graph.get_load_order().expect("Test: operation should succeed");

        // c应该在b之前，b应该在a之前
        let c_idx =
            load_order.iter().position(|p| p == &c).expect("Test: operation should succeed");
        let b_idx =
            load_order.iter().position(|p| p == &b).expect("Test: operation should succeed");
        let a_idx =
            load_order.iter().position(|p| p == &a).expect("Test: operation should succeed");

        assert!(c_idx < b_idx);
        assert!(b_idx < a_idx);
    }

    #[test]
    fn test_get_all_dependencies() {
        let mut graph = DependencyGraph::new();

        let a = PathBuf::from("a.txt");
        let b = PathBuf::from("b.txt");
        let c = PathBuf::from("c.txt");

        graph
            .add_dependency(
                a.clone(),
                ResourceDependency {
                    path: b.clone(),
                    dependency_type: "texture".to_string(),
                    required: true,
                },
            )
            .expect("Test: operation should succeed");

        graph
            .add_dependency(
                b.clone(),
                ResourceDependency {
                    path: c.clone(),
                    dependency_type: "texture".to_string(),
                    required: true,
                },
            )
            .expect("Test: operation should succeed");

        let deps = graph.get_all_dependencies(&a);
        assert!(deps.contains(&b));
        assert!(deps.contains(&c));
    }
}
