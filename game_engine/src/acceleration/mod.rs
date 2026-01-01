//! # 硬件加速模块
//!
//! 提供各种硬件加速功能，包括NPU加速和其他优化。

pub mod npus;

// 重新导出NPU相关类型
pub use npus::*;
