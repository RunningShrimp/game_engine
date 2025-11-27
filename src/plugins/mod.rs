//! 插件系统核心
//!
//! 提供模块化的插件架构，允许按需加载功能模块。

use bevy_ecs::prelude::*;

/// 引擎插件 Trait
pub trait EnginePlugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &'static str;
    
    /// 构建阶段 - 注册资源和系统
    fn build(&self, app: &mut App);
    
    /// 启动阶段 - 初始化运行时状态
    fn startup(&self, _world: &mut World) {}
    
    /// 更新阶段 - 每帧调用
    fn update(&self, _world: &mut World) {}
    
    /// 关闭阶段 - 清理资源
    fn shutdown(&self, _world: &mut World) {}
}

// 重新导出注册表
pub mod registry;
pub use registry::PluginRegistry;

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
}

impl App {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            schedule: Schedule::default(),
            startup_schedule: Schedule::default(),
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
}
