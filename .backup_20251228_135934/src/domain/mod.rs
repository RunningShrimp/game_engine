//! # 领域层（Domain Layer）
//!
//! 本模块实现领域驱动设计（DDD）的核心理念，提供富领域对象和领域服务。
//!
//! ## 设计理念
//!
//! 领域层是业务逻辑的核心，不依赖任何基础设施、应用或用户界面层。
//! 所有业务规则和领域逻辑都封装在领域对象中，形成富领域模型。
//!
//! ## 核心组件
//!
//! ### 实体（Entities）
//! - [`EntityFactory`](entity::EntityFactory) - 实体工厂，创建和管理领域实体
//! - [`GameEntity`](entity::GameEntity) - 游戏实体的领域表示
//! - [`EntityId`](entity::EntityId) - 实体的唯一标识符
//!
//! ### 值对象（Value Objects）
//! - [`Position`](value_objects::Position) - 位置值对象
//! - [`Velocity`](value_objects::Velocity) - 速度值对象
//! - [`Mass`](value_objects::Mass) - 质量值对象
//! - [`DomainTransform`](value_objects::Transform) - 变换值对象
//!
//! ### 聚合（Aggregates）
//! - [`RigidBody`](physics::RigidBody) - 刚体聚合，包含物理属性和行为
//! - [`Collider`](physics::Collider) - 碰撞体聚合
//! - [`Scene`](scene::Scene) - 场景聚合
//!
//! ### 领域服务（Domain Services）
//! - [`PhysicsDomainService`](services::PhysicsDomainService) - 物理领域服务
//! - [`AudioDomainService`](services::AudioDomainService) - 音频领域服务
//! - [`SceneDomainService`](services::SceneDomainService) - 场景领域服务
//!
//! ### 领域事件（Domain Events）
//! - [`EnhancedEventBus`](event_bus::EnhancedEventBus) - 事件总线，用于领域事件通信
//! - [`EventQueue`](event_bus::EventQueue) - 事件队列
//!
//! ## 设计模式
//!
//! ### CQRS（命令查询职责分离）
//! - 命令（Commands）: 修改领域状态
//! - 查询（Queries）: 读取领域状态
//! - 读写分离优化性能
//!
//! ### 事件溯源（Event Sourcing）
//! - 记录所有状态变更事件
//! - 支持时间旅行和审计
//! - 事件重放重建状态
//!
//! ### 依赖注入（DI）
//! - [`DIContainer`](services::DIContainer) - 依赖注入容器
//! - 解耦领域服务依赖
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use game_engine::domain::*;
//!
//! // 创建实体
//! let entity = EntityFactory::create("player")
//!     .with_position(Position::new(0.0, 0.0, 0.0))
//!     .with_velocity(Velocity::new(0.0, 0.0, 0.0))
//!     .with_mass(Mass::from_kilograms(70.0))
//!     .build();
//!
//! // 使用领域服务
//! let physics_service = PhysicsDomainService::new();
//! physics_service.apply_force(&entity, Velocity::new(10.0, 0.0, 0.0));
//! ```
//!
//! ## 错误处理
//!
//! 所有领域操作都返回 [`DomainError`]，包含详细的错误信息和恢复策略。
//!

pub mod actor;
pub mod audio;
pub mod entity;
#[cfg(test)]
mod error_handling_tests;
pub mod errors;
pub mod event_bus;
pub mod event_registry;
pub mod event_sourcing;
pub mod cqrs;
pub mod events;

#[cfg(test)]
mod aggregate_invariants_tests;
pub mod physics;
#[cfg(test)]
mod property_tests;
pub mod scene;
pub mod services;
pub mod value_objects;

// 重新导出主要类型
pub use actor::{
    ActorSystem, AudioActor, AudioActorMessage, PhysicsActor, PhysicsActorMessage, RenderActor,
    RenderActorMessage,
};
pub use audio::{AudioListener, AudioSource, AudioSourceId, SpatialAudioSource};
pub use entity::{EntityFactory, EntityId, GameEntity};
// 注意：AudioError和PhysicsError现在是crate::error::AudioError和crate::error::PhysicsError的别名
// 不在这里重新导出，避免与lib.rs中的pub use error::*冲突
pub use errors::{DomainError, SceneError};
pub use physics::{Collider, ColliderId, RigidBody, RigidBodyId, RigidBodyType};
pub use scene::{Scene, SceneId, SceneRepository};
pub use services::{
    AudioDomainService, DIContainer, DomainServiceFactory, PhysicsDomainService, SceneDomainService,
};
pub use value_objects::{
    Duration, Mass, Position, Rotation, Scale, Transform as DomainTransform, Velocity, Volume,
};

// 事件总线导出
pub use event_bus::{
    EnhancedEventBus, EventBusResource, EventBusStats, EventPriority, EventQueue, EventSystemSet,
    event_publish_system, publish_event,
};

// Re-export event registry
pub use event_registry::{EventRegistry, deserialize_event, global_registry, register_event_type};
