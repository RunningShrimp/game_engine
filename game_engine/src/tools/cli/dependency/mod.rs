//! # 依赖管理系统
//!
//! 提供依赖分析、冲突检测、优化建议等功能。
//!
//! ## 功能
//!
//! - 依赖图构建和可视化
//! - 版本冲突检测
//! - 未使用依赖检测
//! - 依赖替换建议
//!
//! ## 使用示例
//!
//! ```no_run
//! use game_engine::tools::cli::dependency::graph::DependencyGraph;
//!
//! let graph = DependencyGraph::from_project(".").unwrap();
//! println!("{}", graph.display_tree());
//! ```

pub mod conflict;
pub mod graph;
pub mod optimizer;
pub mod unused;

pub use conflict::{ConflictDetector, ConflictReport, VersionConflict};
pub use graph::{DependencyGraph, DependencyStatistics, GraphError};
pub use optimizer::{DependencyOptimizer, OptimizationReport, OptimizationSuggestion};
pub use unused::{UnusedDependency, UnusedDetector, UnusedReason};
