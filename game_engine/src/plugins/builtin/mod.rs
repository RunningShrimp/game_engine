//! 内置插件模块
//!
//! 提供引擎核心功能的内置插件实现。

pub mod audio;
pub mod physics;
pub mod render;
pub mod ui;
pub mod scripting;
pub mod xr;
pub mod scene;
pub mod resources;

// 重新导出内置插件
pub use audio::AudioPlugin;
pub use physics::PhysicsPlugin;
pub use render::RenderPlugin;
pub use ui::UiPlugin;
pub use scripting::ScriptingPlugin;
pub use xr::XrPlugin;
pub use scene::ScenePlugin;
pub use resources::ResourcePlugin;
