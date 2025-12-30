//! 输入验证框架 (Input Validation Framework)
//!
//! 本模块提供统一的输入验证系统，用于验证公共API的输入参数。
//!
//! ## 核心特性
//!
//! - **零panic**: 所有验证失败返回`Result::Err`
//! - **明确错误**: 清晰的错误消息
//! - **性能友好**: 最小化验证开销
//! - **可组合**: 验证器易于组合和复用
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::core::validation::{Validate, ValidationError};
//!
//! // 使用内置验证器
//! use game_engine::core::validation::validators;
//!
//! validators::validate_range(5, 0, 10)?;
//! validators::validate_non_empty("test")?;
//!
//! // 自定义验证
//! #[derive(Validate)]
//! struct EntityConfig {
//!     #[validate(non_empty)]
//!     name: String,
//!
//!     #[validate(range(min = 0, max = 10000))]
//!     id: u64,
//! }
//!
//! let config = EntityConfig { name: "Test".to_string(), id: 42 };
//! config.validate()?;
//! ```
//!
//! ## 模块结构
//!
//! - [`error`]: 验证错误类型
//! - [`validators`]: 内置验证器
//! - [`trait`]: `Validate` trait定义

pub mod error;
pub mod numeric;
pub mod string;
pub mod path;
pub mod trait_def;

// 重新导出常用类型
pub use error::{ValidationError, ValidationResult};
pub use trait_def::Validate;

// 内置验证器
pub mod validators {
    pub use super::numeric::*;
    pub use super::string::*;
    pub use super::path::*;
}
