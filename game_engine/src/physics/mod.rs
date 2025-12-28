//! # 物理系统（Physics System）
//!
//! 本模块提供完整的物理模拟功能，基于Rapier物理引擎和ECS架构。
//!
//! ## 核心组件
//!
//! ### 刚体物理（Rigid Body Physics）
//! - [`RigidBodyComp`]: 刚体ECS组件
//! - [`RigidBodyDesc`]: 刚体描述符，用于创建刚体
//! - [`ColliderComp`]: 碰撞体ECS组件
//! - [`ColliderDesc`]: 碰撞体描述符
//!
//! ### 软体物理（Soft Body Physics）
//! - [`SoftBodyComp`]: 软体组件
//! - [`ClothSimulation`]: 布料模拟
//! - [`FluidSimulation`]: 流体模拟
//!
//! ### 空间分区（Spatial Partitioning）
//! - [`SpatialPartition`]: 空间分区trait
//! - [`SpatialHash`]: 空间哈希实现
//! - [`BVH`]: Bounding Volume Hierarchy
//!
//! ### 批处理和并行（Batching & Parallel）
//! - [`BatchSync`]: 批量同步系统
//! - [`ParallelPhysics`]: 并行物理计算
//! - [`GPUPhysics`]: GPU加速物理
//!
//! ## 使用方式
//!
//! ### 方式1：使用ECS组件（推荐）
//!
//! ```rust,no_run
//! use game_engine::physics::{RigidBodyDesc, ColliderDesc};
//! use game_engine::ecs::Transform;
//! use bevy_ecs::prelude::*;
//! use glam::Vec3;
//!
//! fn spawn_physics_entity(mut commands: Commands) {
//!     commands.spawn((
//!         RigidBodyDesc {
//!             body_type: game_engine::domain::physics::RigidBodyType::Dynamic,
//!             position: Vec3::new(0.0, 10.0, 0.0),
//!             rotation: glam::Quat::IDENTITY,
//!         },
//!         ColliderDesc::ball(1.0),
//!         Transform::default(),
//!     ));
//! }
//! ```
//!
//! ### 方式2：使用领域服务
//!
//! ```rust,no_run
//! use game_engine::domain::services::PhysicsDomainService;
//! use game_engine::domain::physics::{RigidBody, RigidBodyId, RigidBodyType};
//! use glam::Vec3;
//!
//! fn create_physics_body() {
//!     let mut physics_service = PhysicsDomainService::new();
//!
//!     let body = RigidBody::new(
//!         RigidBodyId::new(1),
//!         RigidBodyType::Dynamic,
//!         Vec3::new(0.0, 10.0, 0.0),
//!     );
//!
//!     physics_service.create_body(body).unwrap();
//! }
//! ```
//!
//! ## 空间分区优化
//!
//! 物理系统提供多种空间分区数据结构，用于加速碰撞检测：
//!
//! - **空间哈希（Spatial Hashing）**: 适合均匀分布的物体
//! - **BVH**: 适合大小不一的物体
//! - **网格划分（Grid）**: 简单高效，适合2D
//! - **四叉树/八叉树**: 适合层次化场景
//!
//! ## GPU加速
//!
//! 部分物理计算可以卸载到GPU：
//!
//! - **粒子物理**: 大规模粒子系统
//! - **流体模拟**: SPH（平滑粒子流体动力学）
//! - **碰撞检测**: 宽阶段检测
//!
//! ## 性能优化
//!
//! - **批量同步**: 减少同步开销
//! - **空间分区**: 减少碰撞检测对数
//! - **休眠（Sleeping）**: 静止物体不计算
//! - **固定物体优化**: 静态物体不移动
//!
//! ## 相关模块
//!
//! - [`crate::domain::physics`]: 物理领域对象
//! - [`crate::domain::services::PhysicsDomainService`]: 物理领域服务
//! - [`crate::render`]: 物理可视化
//!

use crate::impl_default;

pub mod batch_sync;
pub mod collision_performance;
pub mod dirty_tracker;
pub mod gpu_acceleration;
pub mod gpu_particle_physics;
pub mod gpu_fluid_simulation;
pub mod joints;
pub mod multithreaded;
pub mod parallel;
pub mod physics3d;
pub mod soft_body;
pub mod spatial_partition;

pub use batch_sync::{
    BatchSyncBuffer, BatchSyncManager, BatchSyncResource, batch_collect_physics_state_system,
    batch_physics_to_transform_system, position_changed_simd, rotation_changed_simd,
};
pub use collision_performance::{
    CollisionPerformanceMonitor, CollisionPerformanceStats, CollisionProfiler,
};
pub use dirty_tracker::{
    BatchSyncData, CachedPhysicsState, PhysicsDirty, PhysicsSyncConfig, PhysicsSyncStats,
    optimized_physics_sync_system, transform_to_physics_sync_system,
};
pub use soft_body::{
    ClothConfig, ClothSoftBody, FluidSoftBody, Particle, SoftBodyComponent, SoftBodyPhysicsWorld,
    SoftBodyType, SphParameters, soft_body_physics_system,
};
pub use spatial_partition::{
    BVHTree, SpatialHash, SpatialPartitionEnhancedConfig, SpatialPartitionManager,
    SpatialPartitionType,
};
pub use multithreaded::{
    MultithreadedPhysicsConfig, MultithreadedPhysicsWorld, PhysicsPerformanceStats,
    multithreaded_physics_step_system, sync_multithreaded_physics_to_transform_system,
};

// 向后兼容：Enhanced类型现在指向基础版本的增强功能
/// 增强的空间分区配置（向后兼容别名）
/// 
/// 注意：增强功能已整合到`SpatialPartitionManager`中，通过`SpatialPartitionEnhancedConfig`配置。
/// 保留此类型别名以保持向后兼容。
#[deprecated(
    since = "0.1.0",
    note = "Use SpatialPartitionEnhancedConfig instead. This type is kept for backward compatibility only."
)]
pub type EnhancedSpatialPartitionConfig = SpatialPartitionEnhancedConfig;

/// 增强的空间分区管理器（向后兼容别名）
/// 
/// 注意：增强功能已整合到`SpatialPartitionManager`中。
/// 保留此类型别名以保持向后兼容。
#[deprecated(
    since = "0.1.0",
    note = "Use SpatialPartitionManager with enhanced config instead. This type is kept for backward compatibility only."
)]
pub type EnhancedSpatialPartitionManager = SpatialPartitionManager;
pub use gpu_acceleration::{
    CollisionResult, GpuPhysicsAccelerator, GpuPhysicsConfig, GpuPhysicsError,
    RigidSoftCollisionDetector,
};
pub use gpu_particle_physics::{
    GpuParticle, GpuParticlePhysicsAccelerator, GpuParticlePhysicsConfig,
    GpuParticlePhysicsError,
};
pub use gpu_fluid_simulation::{
    GpuFluidParticle, GpuFluidSimulator, GpuFluidSimulationConfig,
    GpuFluidSimulationError,
};

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
    shape_type: crate::domain::physics::ShapeType::Cuboid {
        half_extents: glam::Vec3::ONE * 0.5
    },
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
        let body = RigidBody::with_all(
            body_id,
            rb_desc.body_type,
            rb_desc.position,
            rb_desc.rotation,
            1.0,
        );

        // 添加到物理世界
        if let Err(e) = physics_service.create_body(body) {
            tracing::error!(target: "physics", "Failed to create rigid body for entity {:?}: {:?}", entity, e);
            continue;
        }

        // 创建碰撞体（如果有）
        if let Some(cd) = col_desc {
            let collider_id = ColliderId::new(entity.index() as u64 + 1000000); // 偏移以避免冲突

            let collider = match cd.shape_type {
                crate::domain::physics::ShapeType::Cuboid { half_extents: _ } => {
                    Collider::cuboid(collider_id, cd.half_extents)
                }
                crate::domain::physics::ShapeType::Ball { radius: _ } => {
                    Collider::ball(collider_id, cd.radius)
                }
                crate::domain::physics::ShapeType::Sphere { radius } => {
                    Collider::ball(collider_id, radius)
                }
                crate::domain::physics::ShapeType::Capsule { radius, height } => {
                    // 使用胶囊体的高度和半径创建一个近似的球体碰撞体
                    Collider::ball(collider_id, (radius + height / 2.0) / 2.0)
                }
                crate::domain::physics::ShapeType::Cylinder { radius, height: _ } => {
                    Collider::ball(collider_id, radius)
                }
                crate::domain::physics::ShapeType::Cone { radius, height: _ } => {
                    Collider::ball(collider_id, radius)
                }
                crate::domain::physics::ShapeType::ConvexHull { points: _ } => {
                    // 对于凸多边形，使用包围球近似
                    Collider::ball(collider_id, cd.radius.max(0.5))
                }
                crate::domain::physics::ShapeType::TriMesh {
                    vertices: _,
                    indices: _,
                } => {
                    // 对于三角网格，使用包围球近似
                    Collider::ball(collider_id, cd.radius.max(0.5))
                }
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
