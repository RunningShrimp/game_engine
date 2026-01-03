//! # 硬件加速模块
//!
//! 提供各种硬件加速功能，包括NPU加速、LLM推理等。

pub mod llm;
pub mod npus;

// 重新导出NPU相关类型
pub use npus::*;

// 重新导出LLM相关类型
pub use llm::*;
