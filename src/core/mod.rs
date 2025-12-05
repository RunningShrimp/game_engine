//! 核心模块
//!
//! 包含引擎的核心功能：
//! - `engine` - 主引擎入口和运行循环
//! - `systems` - ECS系统定义
//! - `resources` - ECS资源定义
//! - `error` - 错误类型定义
//! - `scheduler` - 任务调度系统

pub mod engine;
pub mod error;
pub mod error_aggregator;
pub mod event_sourcing;
pub mod resources;
pub mod scheduler;
pub mod systems;
pub mod utils;
#[macro_use]
pub mod macros;

#[cfg(test)]
mod tests;

// 重新导出错误类型
pub use error::{
    AssetError, AssetResult, AudioError, AudioResult, EngineError, EngineResult, PhysicsError,
    PhysicsResult, PlatformError, PlatformResult, RenderError, RenderResult, ScriptError,
};

// 重新导出错误聚合器
pub use error_aggregator::{ErrorAggregator, ErrorRecord, ErrorStats, ErrorSummary};

// 重新导出主要类型
pub use engine::Engine;
pub use resources::{AssetMetrics, Benchmark, LogEvents, RenderStats};
pub use systems::{
    apply_texture_handles, audio_input_system, benchmark_system, rotate_system,
    save_previous_transform_system,
};
pub use utils::{
    current_timestamp, current_timestamp_f64, current_timestamp_ms, current_timestamp_nanos,
};
