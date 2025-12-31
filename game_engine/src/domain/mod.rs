//! # Domain Layer
//!
//! This module implements the core concepts of Domain-Driven Design (DDD),
//! providing rich domain objects and domain services.
//!
//! ## Design Philosophy
//!
//! The domain layer is the core of business logic, independent of any infrastructure,
//! application, or user interface layers. All business rules and domain logic are
//! encapsulated in domain objects, forming a rich domain model.
//!
//! ## Core Components
//!
//! ### Entities
//! - [`EntityFactory`](entity::EntityFactory) - Entity factory for creating and managing domain entities
//! - [`GameEntity`](entity::GameEntity) - Domain representation of game entities
//! - [`EntityId`](entity::EntityId) - Unique identifier for entities
//!
//! ### Value Objects
//! - [`Position`](value_objects::Position) - Position value object
//! - [`Velocity`](value_objects::Velocity) - Velocity value object
//! - [`Mass`](value_objects::Mass) - Mass value object
//! - [`DomainTransform`](value_objects::Transform) - Transform value object
//!
//! ### Aggregates
//! - [`RigidBody`](physics::RigidBody) - Rigid body aggregate with physics properties and behaviors
//! - [`Collider`](physics::Collider) - Collider aggregate
//! - [`Scene`](scene::Scene) - Scene aggregate
//!
//! ### Domain Services
//! - [`PhysicsDomainService`](services::PhysicsDomainService) - Physics domain service
//! - [`AudioDomainService`](services::AudioDomainService) - Audio domain service
//! - [`SceneDomainService`](services::SceneDomainService) - Scene domain service
//!
//! ### Domain Events
//! - [`EnhancedEventBus`](event_bus::EnhancedEventBus) - Event bus for domain event communication
//! - [`EventQueue`](event_bus::EventQueue) - Event queue
//!
//! ## Design Patterns
//!
//! ### CQRS (Command Query Responsibility Segregation)
//! - Commands: Modify domain state
//! - Queries: Read domain state
//! - Separate read and write for performance optimization
//!
//! ### Event Sourcing
//! - Record all state change events
//! - Support time travel and auditing
//! - Replay events to rebuild state
//!
//! ### Dependency Injection (DI)
//! - [`DIContainer`](services::DIContainer) - Dependency injection container
//! - Decouple domain service dependencies
//!
//! ## Examples
//!
//! ```rust,no_run
//! use game_engine::domain::*;
//!
//! // Create entity
//! let entity = EntityFactory::create("player")
//!     .with_position(Position::new(0.0, 0.0, 0.0))
//!     .with_velocity(Velocity::new(0.0, 0.0, 0.0))
//!     .with_mass(Mass::from_kilograms(70.0))
//!     .build();
//!
//! // Use domain service
//! let physics_service = PhysicsDomainService::new();
//! physics_service.apply_force(&entity, Velocity::new(10.0, 0.0, 0.0));
//! ```
//!
//! ## Error Handling
//!
//! All domain operations return [`DomainError`] with detailed error information
//! and recovery strategies.

// 模块私有实现说明：
// - 实现领域驱动设计（DDD）模式
// - 提供富领域模型和领域服务
// - 支持CQRS和事件溯源模式
// - 实现依赖注入容器
// - 使用Actor模式处理领域逻辑
pub mod actor;
#[cfg(test)]
mod aggregate_invariants_tests;
pub mod audio;
pub mod cqrs;
pub mod entity;
#[cfg(test)]
mod error_handling_tests;
pub mod errors;
pub mod event_bus;
pub mod event_registry;
pub mod event_sourcing;
pub mod events;
pub mod physics;
#[cfg(test)]
mod property_tests;
pub mod repository;
pub mod scene;
pub mod services;
pub mod soa_storage;
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
pub use repository::{EntityRepository, Repository, RepositoryError, RigidBodyRepository, SceneRepositoryImpl};
pub use scene::{Scene, SceneId, SceneRepository};
pub use services::{
    AudioDomainService, DIContainer, DomainServiceFactory, PhysicsDomainService, SceneDomainService,
};
pub use soa_storage::{ColliderStorage, RigidBodyStorage};
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
