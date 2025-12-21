//  核心模块
// 
//  包含引擎的核心功能：
//  - `engine` - 主引擎入口和运行循环
//  - `systems` - ECS系统定义
//  - `resources` - ECS资源定义
//  - `error` - 错误类型定义
//  - `scheduler` - 任务调度系统

// 引擎模块现在是一个子模块，通过 engine/mod.rs 导入
// 引擎模块现在是一个子模块，通过 engine/mod.rs 导入
pub mod engine;
pub mod error;
pub mod error_aggregator;
pub mod event_sourcing;
pub mod resources;
pub mod scheduler;
pub mod system_scheduler;
pub mod systems;
pub mod utils;
#[macro_use]
pub mod macros;

#[cfg(test)]
mod tests;

// 重新导出错误类型
pub use crate::error::*;

// 重新导出错误聚合器
pub use error_aggregator::{ErrorAggregator, ErrorRecord, ErrorStats, ErrorSummary};

// 重新导出主要类型
pub use crate::config::EngineConfig;
pub use crate::core::engine::Engine;
pub use resources::{AssetMetrics, Benchmark, LogEvents, RenderStats};
// 系统模块重新导出以避免循环依赖
pub use systems::ai_system;
pub use systems::animation_system;
pub use systems::rotate_system;
pub use utils::{
    current_timestamp, current_timestamp_f64, current_timestamp_ms, current_timestamp_nanos,
};
