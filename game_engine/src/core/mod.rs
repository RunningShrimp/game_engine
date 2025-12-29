//! # 核心模块（Core）
//!
//! 本模块包含游戏引擎的核心功能和基础设施。
//!
//! ## 子模块
//!
//! - [`engine`] - 主引擎入口、运行循环和游戏循环
//! - [`systems`] - ECS系统定义和实现
//! - [`resources`] - ECS资源定义
//! - [`scheduler`] - 任务调度系统和异步执行器
//! - [`error`] - 核心错误类型定义
//! - [`event_sourcing`] - 事件溯源模式实现
//! - [`microkernel`] - 微内核架构实现
//! - [`error_aggregator`] - 错误聚合和报告
//!
//! ## 核心功能
//!
//! ### 引擎生命周期
//! - 初始化：[`engine::Engine::new()`]
//! - 更新循环：[`engine::Engine::update()`]
//! - 渲染循环：[`engine::Engine::render()`]
//! - 关闭：[`engine::Engine::shutdown()`]
//!
//! ### 任务调度
//! - 异步任务执行器
//! - 协程支持
//! - 并行任务调度
//!
//! ### 事件溯源
//! - 领域事件存储
//! - 事件重放
//! - CQRS模式支持
pub mod editor;
pub mod engine;
pub mod error;
pub mod error_aggregator;
pub mod event_sourcing;
pub mod microkernel;
pub mod resources;
pub mod scheduler;
pub mod system_scheduler;
pub mod systems;
pub mod utils;
#[macro_use]
pub mod macros;

#[cfg(test)]
mod tests;

// ========================================
// 综合测试模块
// ========================================

#[cfg(test)]
mod core_module_tests;

#[cfg(test)]
mod utils_tests;

#[cfg(test)]
mod error_aggregator_tests;

// 重新导出错误类型
pub use crate::error::*;

// 重新导出错误聚合器
pub use error_aggregator::{ErrorAggregator, ErrorRecord, ErrorStats, ErrorSummary};

// 重新导出主要类型
pub use crate::EngineConfig;
pub use crate::core::engine::Engine;
pub use resources::{AssetMetrics, Benchmark, LogEvents, RenderStats};
// 系统模块重新导出以避免循环依赖
pub use systems::ai_system;
pub use systems::animation_system;
pub use systems::rotate_system;
pub use utils::{
    current_timestamp, current_timestamp_f64, current_timestamp_ms, current_timestamp_nanos,
};
