//! # Physics System
//!
//! This module provides complete physics simulation functionality based on
//! the Rapier physics engine and ECS architecture.
//!
//! ## Core Components
//!
//! ### Rigid Body Physics
//! - [`RigidBodyComp`][]: Rigid body ECS component
//! - [`RigidBodyDesc`][]: Rigid body descriptor for creating rigid bodies
//! - [`ColliderComp`][]: Collider ECS component
//! - [`ColliderDesc`][]: Collider descriptor
//!
//! ### Soft Body Physics
//! - [`SoftBodyComp`][]: Soft body component
//! - [`ClothSimulation`][]: Cloth simulation
//! - [`FluidSimulation`][]: Fluid simulation
//!
//! ### Spatial Partitioning
//! - [`SpatialPartition`][]: Spatial partition trait
//! - [`SpatialHash`][]: Spatial hash implementation
//! - [`BVH`][]: Bounding Volume Hierarchy
//!
//! ### Batching & Parallel
//! - [`BatchSync`][]: Batch synchronization system
//! - [`ParallelPhysics`][]: Parallel physics computation
//! - [`GPUPhysics`][]: GPU-accelerated physics
//!
//! ## Usage
//!
//! ### Method 1: Using ECS Components (Recommended)
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
//! ### Method 2: Using Domain Services
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
//!     physics_service.create_body(body).expect("Test: operation should succeed");
//! }
//! ```
//!
//! ## Spatial Partitioning Optimization
//!
//! The physics system provides various spatial partitioning data structures
//! to accelerate collision detection:
//!
//! - **Spatial Hashing**: Suitable for uniformly distributed objects
//! - **BVH**: Suitable for objects of varying sizes
//! - **Grid**: Simple and efficient, suitable for 2D
//! - **Quadtree/Octree**: Suitable for hierarchical scenes
//!
//! ## GPU Acceleration
//!
//! Some physics computations can be offloaded to GPU:
//!
//! - **Particle Physics**: Large-scale particle systems
//! - **Fluid Simulation**: SPH (Smoothed Particle Hydrodynamics)
//! - **Collision Detection**: Broad phase detection
//!
//! ## Performance Optimization
//!
//! - **Batch Synchronization**: Reduce synchronization overhead
//! - **Spatial Partitioning**: Reduce collision detection pairs
//! - **Sleeping**: Stationary objects are not calculated
//! - **Fixed Body Optimization**: Static objects don't move
//!
//! ## Related Modules
//!
//! - [`crate::domain::physics`][]: Physics domain objects
//! - [`crate::domain::services::PhysicsDomainService`][]: Physics domain services
//! - [`crate::render`][]: Physics visualization

// 模块私有实现说明：
// - 基于Rapier物理引擎（2D和3D）
// - 支持刚体和软体物理模拟
// - 提供空间分区数据结构优化碰撞检测
// - 支持GPU加速的物理计算
// - 集成ECS架构用于组件化管理

use crate::impl_default;

pub mod batch_sync;
pub mod collision_performance;
pub mod cqrs;
pub mod cqrs_performance_tests;
pub mod dirty_tracker;
pub mod gpu_acceleration;
pub mod gpu_fluid_simulation;
pub mod gpu_particle_physics;
pub mod joints;
pub mod multithreaded;
pub mod parallel;
pub mod physics3d;
pub mod simd_integration;
pub mod soft_body;
pub mod spatial_partition;
pub mod test_helpers;

pub use batch_sync::{
    BatchSyncBuffer, BatchSyncManager, BatchSyncResource, batch_collect_physics_state_system,
    batch_physics_to_transform_system, position_changed_simd, rotation_changed_simd,
};
pub use simd_integration::{
    ParentTransform, SimdBackendType, SimdPerformanceMonitor, SimdPerformanceStats,
    SimdPhysicsState, simd_performance_monitor_system, simd_physics_integrate_system,
    simd_transform_update_system, PhysicsIntegrateBatch, TransformUpdateBatch,
};
pub use collision_performance::{
    CollisionPerformanceMonitor, CollisionPerformanceStats, CollisionProfiler,
};
pub use dirty_tracker::{
    BatchSyncData, CachedPhysicsState, PhysicsDirty, PhysicsSyncConfig, PhysicsSyncStats,
    optimized_physics_sync_system, transform_to_physics_sync_system,
};
pub use multithreaded::{
    MultithreadedPhysicsConfig, MultithreadedPhysicsWorld, PhysicsPerformanceStats,
    multithreaded_physics_step_system, sync_multithreaded_physics_to_transform_system,
};
pub use soft_body::{
    ClothConfig, ClothSoftBody, FluidSoftBody, Particle, SoftBodyComponent, SoftBodyPhysicsWorld,
    SoftBodyType, SphParameters, soft_body_physics_system,
};
pub use spatial_partition::{
    BVHTree, SpatialHash, SpatialPartitionEnhancedConfig, SpatialPartitionManager,
    SpatialPartitionType,
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
pub use gpu_fluid_simulation::{
    GpuFluidParticle, GpuFluidSimulationConfig, GpuFluidSimulationError, GpuFluidSimulator,
};
pub use gpu_particle_physics::{
    GpuParticle, GpuParticlePhysicsAccelerator, GpuParticlePhysicsConfig, GpuParticlePhysicsError,
};

// CQRS exports
pub use cqrs::{
    ApplyImpulseCommand, ApplyImpulseHandler, CreateRigidBodyCommand, CreateRigidBodyHandler,
    GetBodiesInRadiusHandler, GetBodiesInRadiusQuery, GetBodyPositionHandler,
    GetBodyPositionQuery, GetDynamicBodiesHandler, GetDynamicBodiesQuery, PhysicsApplicationService,
    PhysicsQueryModel, RigidBodySnapshot, RemoveRigidBodyCommand, SetVelocityCommand,
    UpdatePositionCommand, UpdatePositionHandler,
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

// ========================================
// 综合测试模块
// ========================================

#[cfg(test)]
mod physics_core_tests;

#[cfg(test)]
mod spatial_partition_tests;

#[cfg(test)]
mod gpu_parallel_tests;

#[cfg(test)]
mod extended_tests;
