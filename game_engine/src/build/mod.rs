//! 构建管理模块
//!
//! 提供增强的构建功能：
//! - 增量构建
//! - 并行构建
//! - 进度显示
//! - 构建缓存

pub mod build_manager;

pub use build_manager::{
    BuildConfig, BuildError, BuildManager, BuildProfile, BuildResult, BuildStats,
};
