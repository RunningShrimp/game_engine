//! 性能验证和优化基准测试工具
//!
//! 提供性能目标设定、优化结果追踪和 CPU/GPU 性能比较的工具。
//!
//! ## 类型
//!
//! - [`OptimizationGoal`]: 定义性能优化目标（基准值、目标值、单位）
//! - [`OptimizationResult`]: 记录优化结果和达成百分比
//! - [`CpuGpuComparison`]: 比较 CPU 和 GPU 操作的性能
//! - [`PerformanceValidationSuite`]: 性能验证套件，管理多个目标和结果
//! - [`ValidationSummary`]: 性能验证摘要
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine_common::benchmarking::{PerformanceValidationSuite, OptimizationGoal};
//! use std::time::Duration;
//!
//! let mut suite = PerformanceValidationSuite::new();
//!
//! // 设定 FPS 目标：从 60 提升到 120
//! suite.add_goal(OptimizationGoal::new("fps", 60.0, 120.0, "fps"));
//!
//! // 记录优化结果
//! suite.record_result(
//!     OptimizationGoal::new("fps", 60.0, 120.0, "fps"),
//!     60.0,
//!     100.0
//! );
//!
//! // 生成报告
//! println!("{}", suite.generate_report());
//! ```

mod optimization_validation;

pub use optimization_validation::*;
