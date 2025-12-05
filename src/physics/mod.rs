//! 物理系统模块
//!
//! 提供物理模拟功能，基于富领域对象架构。
//! 使用 `crate::domain::physics` 模块中的富领域对象。

use crate::impl_default;

pub mod dirty_tracker;
pub mod joints;
pub mod parallel;
pub mod physics3d;

pub use dirty_tracker::{
    BatchSyncData, CachedPhysicsState, PhysicsDirty, PhysicsSyncConfig, PhysicsSyncStats,
};

#[cfg(feature = "physics_2d")]
pub use dirty_tracker::{optimized_physics_sync_system, transform_to_physics_sync_system};

// 重新导出富领域对象（推荐使用）
pub use crate::domain::physics::{
    Collider, ColliderId, RigidBody, RigidBodyId, RigidBodyType as RichRigidBodyType,
    ShapeType as RichShapeType,
};

pub use crate::domain::services::PhysicsDomainService;

use crate::ecs::Transform;
use bevy_ecs::prelude::*;

// ============================================================================
// ECS 组件定义
// ============================================================================

/// 刚体组件 - 关联实体与物理刚体（使用富领域对象ID）
#[derive(Component, Clone, Copy, Debug)]
pub struct RigidBodyComp {
    /// 刚体ID（富领域对象）
    pub body_id: RigidBodyId,
}

/// 碰撞体组件 - 关联实体与物理碰撞体（使用富领域对象ID）
#[derive(Component, Clone, Copy, Debug)]
pub struct ColliderComp {
    /// 碰撞体ID（富领域对象）
    pub collider_id: ColliderId,
}

// ============================================================================
// ECS 系统函数（使用富领域对象）
// ============================================================================

/// 物理步进系统 - 使用富领域对象
pub fn physics_step_system(
    mut physics_service: ResMut<PhysicsDomainService>,
    time: Res<crate::ecs::Time>,
) {
    if let Err(e) = physics_service.step_simulation(time.delta_seconds) {
        tracing::error!(target: "physics", "Physics step failed: {:?}", e);
    }
}

/// 同步物理到 Transform 系统 - 使用富领域对象
pub fn sync_physics_to_transform_system(
    physics_service: Res<PhysicsDomainService>,
    mut query: Query<(&RigidBodyComp, &mut Transform)>,
) {
    let world = physics_service.get_world();
    for (rb_comp, mut transform) in query.iter_mut() {
        // 获取刚体位置
        if let Ok(pos) = physics_service.get_body_position(rb_comp.body_id) {
            transform.pos = pos;
        }

        // 获取刚体旋转（从PhysicsWorld内部获取）
        if let Some(body_state) = world.get_body_state(rb_comp.body_id) {
            transform.rot = body_state.rotation;
        }
    }
}

/// 物理步进系统（别名，向后兼容）
pub use physics_step_system as physics_step_system_v2;

/// 同步物理到 Transform 系统（别名，向后兼容）
pub use sync_physics_to_transform_system as sync_physics_to_transform_system_v2;

// ============================================================================
// 构建器组件（用于声明式创建）
// ============================================================================

/// 刚体描述组件 - 用于声明式创建刚体
#[derive(Component, Clone)]
pub struct RigidBodyDesc {
    /// 刚体类型
    pub body_type: crate::domain::physics::RigidBodyType,
    /// 初始位置
    pub position: glam::Vec3,
    /// 初始旋转
    pub rotation: glam::Quat,
}

impl_default!(RigidBodyDesc {
    body_type: crate::domain::physics::RigidBodyType::Dynamic,
    position: glam::Vec3::ZERO,
    rotation: glam::Quat::IDENTITY,
});

/// 碰撞体描述组件 - 用于声明式创建碰撞体
#[derive(Component, Clone)]
pub struct ColliderDesc {
    /// 形状类型
    pub shape_type: crate::domain::physics::ShapeType,
    /// 立方体半尺寸
    pub half_extents: glam::Vec3,
    /// 球体半径
    pub radius: f32,
}

impl_default!(ColliderDesc {
    shape_type: crate::domain::physics::ShapeType::Cuboid,
    half_extents: glam::Vec3::ONE * 0.5,
    radius: 0.5,
});

/// 初始化物理刚体系统 - 使用富领域对象
pub fn init_physics_bodies(
    mut commands: Commands,
    mut physics_service: ResMut<PhysicsDomainService>,
    query: Query<
        (Entity, &RigidBodyDesc, Option<&ColliderDesc>),
        (Without<RigidBodyComp>, Without<ColliderComp>),
    >,
) {
    use crate::domain::physics::{Collider, RigidBody};

    for (entity, rb_desc, col_desc) in query.iter() {
        // 创建刚体ID（使用实体索引作为ID）
        let body_id = RigidBodyId::new(entity.index() as u64);

        // 创建富领域对象刚体
        let mut body = RigidBody::new(body_id, rb_desc.body_type, rb_desc.position);
        body.rotation = rb_desc.rotation;

        // 添加到物理世界
        if let Err(e) = physics_service.create_body(body) {
            tracing::error!(target: "physics", "Failed to create rigid body for entity {:?}: {:?}", entity, e);
            continue;
        }

        // 创建碰撞体（如果有）
        if let Some(cd) = col_desc {
            let collider_id = ColliderId::new(entity.index() as u64 + 1000000); // 偏移以避免冲突

            let collider = match cd.shape_type {
                crate::domain::physics::ShapeType::Cuboid => {
                    Collider::cuboid(collider_id, cd.half_extents)
                }
                crate::domain::physics::ShapeType::Ball => Collider::ball(collider_id, cd.radius),
            };

            if let Err(e) = physics_service.create_collider(collider, body_id) {
                tracing::error!(target: "physics", "Failed to create collider for entity {:?}: {:?}", entity, e);
            } else {
                commands.entity(entity).insert(ColliderComp { collider_id });
            }
        }

        commands.entity(entity).insert(RigidBodyComp { body_id });
    }
}

/// 初始化物理刚体系统（别名，向后兼容）
pub use init_physics_bodies as init_physics_bodies_v2;

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{PhysicsDomainService, RigidBody, RigidBodyId, RigidBodyType};
    use glam::Vec3;

    #[test]
    fn test_physics_domain_service() {
        let mut service = PhysicsDomainService::new();

        // 创建刚体
        let body = RigidBody::new(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 10.0, 0.0),
        );
        assert!(service.create_body(body).is_ok());

        // 步进模拟
        assert!(service.step_simulation(0.016).is_ok());

        // 获取位置
        let position = service.get_body_position(RigidBodyId::new(1));
        assert!(position.is_ok());
    }
}
