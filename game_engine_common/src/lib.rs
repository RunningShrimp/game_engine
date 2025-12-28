//! game_engine_common
//!
//! 游戏引擎的共享代码库，提供跨多个 crate 共用的工具和类型。
//!
//! ## 模块
//!
//! - [`benchmarking`][]: 性能验证和优化基准测试工具
//! - [`sync`][]: 同步原语和线程安全数据结构
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine_common::{PerformanceValidationSuite, OptimizationGoal};
//!
//! let mut suite = PerformanceValidationSuite::new();
//! suite.add_goal(OptimizationGoal::new("fps", 60.0, 120.0, "fps"));
//! suite.record_result(OptimizationGoal::new("fps", 60.0, 120.0, "fps"), 60.0, 100.0);
//! ```

pub mod benchmarking;
pub mod sync;

pub use benchmarking::*;
pub use sync::*;
