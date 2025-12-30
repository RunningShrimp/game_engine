//! 验证错误类型

use std::path::PathBuf;

/// 验证错误类型
///
/// 所有输入验证失败都应返回此错误或其变体。
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    // ========== 数值错误 ==========

    /// 值超出范围
    #[error("Value {value} is out of range [{min}, {max}]")]
    OutOfRange {
        value: String,
        min: String,
        max: String,
    },

    /// 非有限浮点数
    #[error("Non-finite floating point value: {0}")]
    NonFinite(f32),

    /// NaN值
    #[error("NaN value: {0}")]
    NaN(f32),

    /// 负数
    #[error("Negative value where positive required: {0}")]
    Negative(String),

    /// 零值
    #[error("Zero value where non-zero required")]
    Zero,

    // ========== 字符串错误 ==========

    /// 空字符串
    #[error("String cannot be empty")]
    EmptyString,

    /// 字符串太短
    #[error("String too short: minimum {min} characters, actual {actual}")]
    TooShort { min: usize, actual: usize },

    /// 字符串太长
    #[error("String too long: maximum {max} characters, actual {actual}")]
    TooLong { max: usize, actual: usize },

    /// 包含无效字符
    #[error("String contains invalid characters")]
    InvalidCharacters,

    /// 不匹配模式
    #[error("String does not match required pattern: {pattern}")]
    PatternMismatch { pattern: String },

    // ========== 路径错误 ==========

    /// 路径不存在
    #[error("Path does not exist: {0}")]
    PathNotFound(PathBuf),

    /// 路径不可读
    #[error("Path is not readable: {0}")]
    PathNotReadable(PathBuf),

    /// 无效扩展名
    #[error("Invalid extension for path '{path}': found '{found}', allowed: {allowed:?}")]
    InvalidExtension {
        path: PathBuf,
        found: String,
        allowed: Vec<String>,
    },

    /// 缺少扩展名
    #[error("Path missing required extension: {0}")]
    MissingExtension(PathBuf),

    // ========== 集合错误 ==========

    /// 空集合
    #[error("Collection cannot be empty")]
    EmptyCollection,

    /// 集合太大
    #[error("Collection too large: maximum {max} elements, actual {actual}")]
    CollectionTooLarge { max: usize, actual: usize },

    /// 集合太小
    #[error("Collection too small: minimum {min} elements, actual {actual}")]
    CollectionTooSmall { min: usize, actual: usize },

    /// 重复元素
    #[error("Collection contains duplicate elements")]
    DuplicateElements,

    // ========== 自定义错误 ==========

    /// 自定义验证错误
    #[error("Custom validation error: {0}")]
    Custom(String),

    /// 多个验证错误
    #[error("Multiple validation errors: {}", .0.join(", "))]
    Multiple(Vec<String>),
}

impl ValidationError {
    /// 创建OutOfRange错误的便捷方法
    pub fn out_of_range<T>(value: T, min: T, max: T) -> Self
    where
        T: std::fmt::Display,
    {
        ValidationError::OutOfRange {
            value: value.to_string(),
            min: min.to_string(),
            max: max.to_string(),
        }
    }

    /// 创建Custom错误的便捷方法
    pub fn custom<S: Into<String>>(msg: S) -> Self {
        ValidationError::Custom(msg.into())
    }
}

/// 验证结果类型
pub type ValidationResult<T = ()> = Result<T, ValidationError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ValidationError::EmptyString;
        assert_eq!(err.to_string(), "String cannot be empty");

        let err = ValidationError::out_of_range(15, 0, 10);
        assert!(err.to_string().contains("15"));
        assert!(err.to_string().contains("0"));
        assert!(err.to_string().contains("10"));
    }

    #[test]
    fn test_custom_error() {
        let err = ValidationError::custom("test error");
        match err {
            ValidationError::Custom(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Unexpected error type"),
        }
    }
}

