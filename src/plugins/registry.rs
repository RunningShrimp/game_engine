//! 插件注册表
//!
//! 管理所有已注册的插件。

use super::{EnginePlugin, App};

pub struct PluginRegistry {
    plugins: Vec<Box<dyn EnginePlugin>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// 添加插件
    pub fn add<P: EnginePlugin + 'static>(&mut self, plugin: P) -> &mut Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    /// 构建所有插件
    pub fn build_all(&self, app: &mut App) {
        for plugin in &self.plugins {
            plugin.build(app);
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
        for plugin in &self.plugins {
            plugin.shutdown(world);
        }
    }
}
