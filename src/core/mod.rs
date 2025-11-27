//! 核心模块
//!
//! 包含引擎的核心功能：
//! - `engine` - 主引擎入口和运行循环
//! - `systems` - ECS系统定义
//! - `resources` - ECS资源定义
//! - `error` - 错误类型定义
//! - `scheduler` - 任务调度系统

pub mod error;
pub mod engine;
pub mod systems;
pub mod resources;
pub mod scheduler;

#[cfg(test)]
mod tests;

// 重新导出错误类型
pub use error::{
    EngineError, RenderError, AssetError, PhysicsError, AudioError, ScriptError, PlatformError,
    EngineResult, RenderResult, AssetResult, PhysicsResult, AudioResult, PlatformResult,
};

// 重新导出主要类型
pub use engine::Engine;
pub use resources::{Benchmark, RenderStats, AssetMetrics, LogEvents};
pub use systems::{
            rotate_system,
            apply_texture_handles,
    save_previous_transform_system,
    benchmark_system,
            audio_input_system,
};
