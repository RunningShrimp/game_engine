//  场景管理系统
//
//  提供场景的加载、保存、切换和管理功能。

pub mod manager;
pub mod serialization;

pub use manager::{Scene, SceneId, SceneTransition, SceneTransitionManager};
pub use manager::{scene_cleanup_system, scene_load_system, scene_update_system};
pub use serialization::{SerializedComponent, SerializedEntity, SerializedScene};
