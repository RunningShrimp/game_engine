//! 场景管理系统
//!
//! 提供场景的加载、保存、切换和管理功能。

pub mod manager;
pub mod serialization;

pub use manager::{Scene, SceneId, SceneManager, SceneTransition};
pub use serialization::{SerializedComponent, SerializedEntity, SerializedScene};
