//  领域层模块
//  实现富领域对象设计模式，将业务逻辑封装到领域对象中

pub mod actor;
pub mod audio;
pub mod entity;
pub mod event_registry;
pub mod event_sourcing;
pub mod events;
#[cfg(test)]
mod error_handling_tests;
pub mod errors;

#[cfg(test)]
mod aggregate_invariants_tests;
pub mod physics;
#[cfg(test)]
mod property_tests;
pub mod render;
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
pub use errors::{AudioError, DomainError, PhysicsError, SceneError};
pub use physics::{Collider, ColliderId, RigidBody, RigidBodyId, RigidBodyType};
pub use render::{
    LightSource, PbrScene, RenderObject, RenderObjectId, RenderScene, RenderStrategy,
};
pub use scene::{Scene, SceneId, SceneManager};
pub use services::{
    AudioDomainService, DIContainer, DomainServiceFactory, PhysicsDomainService, SceneDomainService,
};
pub use value_objects::{
    Duration, Mass, Position, Rotation, Scale, Transform as DomainTransform, Velocity, Volume,
};

// Re-export event registry
pub use event_registry::{EventRegistry, global_registry, register_event_type, deserialize_event};
