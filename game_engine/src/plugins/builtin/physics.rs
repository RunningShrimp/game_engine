//  物理插件
//
//  提供物理模拟功能，基于 Rapier 物理引擎。

use crate::impl_default;
use crate::physics::{
    PhysicsDomainService, physics_step_system_v2, sync_physics_to_transform_system_v2,
};
use crate::plugins::{App, EnginePlugin, PluginDependency, PluginVersion};

/// 物理插件配置
#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    /// 重力向量
    pub gravity: [f32; 2],
    /// 时间步长
    pub timestep: f32,
}

impl_default!(PhysicsConfig {
    gravity: [0.0, -9.81],
    timestep: 1.0 / 60.0,
});

/// 物理插件
pub struct PhysicsPlugin {
    config: PhysicsConfig,
}

impl PhysicsPlugin {
    /// 创建物理插件
    pub fn new() -> Self {
        Self {
            config: PhysicsConfig::default(),
        }
    }

    /// 使用自定义配置创建物理插件
    pub fn with_config(config: PhysicsConfig) -> Self {
        Self { config }
    }
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl EnginePlugin for PhysicsPlugin {
    fn name(&self) -> &'static str {
        "PhysicsPlugin"
    }

    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Provides 2D physics simulation using Rapier physics engine"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![
            // 物理插件依赖于核心ECS系统
        ]
    }

    fn build(&self, app: &mut App) {
        // 插入物理领域服务资源
        let physics_service = PhysicsDomainService::new();
        // 注意:当前版本的PhysicsDomainService不提供设置重力的公共API
        // 重力配置被记录但暂时无法应用,需要扩展PhysicsDomainService

        app.insert_resource(physics_service);

        // 添加物理系统 - 使用包装闭包来适配系统参数
        app.add_systems(|schedule| {
            schedule.add_systems(physics_step_system_v2);
        });
        app.add_systems(|schedule| {
            schedule.add_systems(sync_physics_to_transform_system_v2);
        });
    }

    fn startup(&self, _world: &mut bevy_ecs::world::World) {
        // 初始化物理世界
        println!(
            "Physics plugin started with gravity: {:?}",
            self.config.gravity
        );
    }

    fn update(&self, _world: &mut bevy_ecs::world::World) {
        // 物理更新逻辑已在系统函数中处理
    }

    fn shutdown(&self, _world: &mut bevy_ecs::world::World) {
        println!("Physics plugin shutting down");
    }
}
