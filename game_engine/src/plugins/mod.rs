//! 插件系统核心
//!
//! 提供模块化的插件架构，允许按需加载功能模块。

use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// 插件版本信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl PluginVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
}

/// 插件依赖信息
#[derive(Debug, Clone)]
pub struct PluginDependency {
    pub name: String,
    pub version_requirement: String,
}

/// 插件元数据
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: PluginVersion,
    pub description: String,
    pub dependencies: Vec<PluginDependency>,
}

/// 引擎插件 Trait
pub trait EnginePlugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &'static str;

    /// 插件版本
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }

    /// 插件描述
    fn description(&self) -> &'static str {
        ""
    }

    /// 插件依赖
    fn dependencies(&self) -> Vec<PluginDependency> {
        Vec::new()
    }

    /// 构建阶段 - 注册资源和系统
    fn build(&self, app: &mut App);

    /// 启动阶段 - 初始化运行时状态
    fn startup(&self, _world: &mut World) {}

    /// 更新阶段 - 每帧调用
    fn update(&self, _world: &mut World) {}

    /// 关闭阶段 - 清理资源
    fn shutdown(&self, _world: &mut World) {}

    /// 获取插件元数据
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: self.version(),
            description: self.description().to_string(),
            dependencies: self.dependencies(),
        }
    }
}

// 重新导出注册表
pub mod registry;
pub use registry::PluginRegistry;

// 热加载支持
pub mod hot_reload;
pub use hot_reload::{HotReloadManager, HotReloadError};

// 配置系统
pub mod config;
pub use config::{PluginConfigManager, PluginConfig};

// 内置插件
pub mod builtin;

// 模拟 App 结构（如果 bevy_app 不可用）
// 在实际项目中，这通常是 bevy_app::App
// 这里我们假设用户使用的是 bevy_ecs，可能没有完整的 App 结构，
// 或者我们定义一个简单的 App 包装器。
// 为了兼容性，我们这里定义一个简单的 App 结构，或者引用 crate::core::App 如果存在。
// 假设 crate::core::engine::Engine 是 App 的对应物，或者我们需要定义 App。
// 让我们查看 src/core/engine.rs

pub struct App {
    pub world: World,
    pub schedule: Schedule,
    pub startup_schedule: Schedule,
    pub plugin_registry: PluginRegistry,
}

impl App {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            schedule: Schedule::default(),
            startup_schedule: Schedule::default(),
            plugin_registry: PluginRegistry::new(),
        }
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    pub fn add_system<M>(&mut self, system: impl IntoSystemConfigs<M>) -> &mut Self {
        self.schedule.add_systems(system);
        self
    }

    pub fn add_startup_system<M>(&mut self, system: impl IntoSystemConfigs<M>) -> &mut Self {
        self.startup_schedule.add_systems(system);
        self
    }

    /// 添加插件
    pub fn add_plugin<P: EnginePlugin + 'static>(&mut self, plugin: P) -> &mut Self {
        self.plugin_registry.add(plugin);
        self
    }

    /// 构建所有插件
    pub fn build_plugins(&mut self) -> &mut Self {
        self.plugin_registry.build_all(self);
        self
    }

    /// 运行启动系统
    pub fn run_startup(&mut self) {
        self.startup_schedule.run(&mut self.world);
        self.plugin_registry.startup_all(&mut self.world);
    }

    /// 运行主循环更新
    pub fn update(&mut self) {
        self.schedule.run(&mut self.world);
        self.plugin_registry.update_all(&mut self.world);
    }

    /// 关闭应用
    pub fn shutdown(&mut self) {
        self.plugin_registry.shutdown_all(&mut self.world);
    }
}
