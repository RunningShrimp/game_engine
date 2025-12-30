//! 统一错误处理模式
//!
//! 本模块提供引擎范围内的统一错误处理模式和指南。

use thiserror::Error;

/// 统一的引擎错误类型
///
/// 提供引擎范围内的统一错误层次结构，包含所有可能的错误情况。
///
/// ## 错误层次
///
/// ```text
/// EngineError (顶层错误)
/// ├── Io (IO相关错误)
/// ├── Serialization (序列化错误)
/// ├── Resource (资源管理错误)
/// ├── Physics (物理系统错误)
/// ├── Network (网络错误)
/// ├── Validation (参数验证错误)
/// └── Config (配置错误)
/// ```
///
/// ## 使用示例
///
/// ```rust
/// use game_engine::error::EngineError;
///
/// pub fn load_resource(path: &str) -> Result<Vec<u8>, EngineError> {
///     let data = std::fs::read(path)?;
///     Ok(data)
/// }
/// ```
///
/// ## 转换规则
///
/// 使用`?`运算符自动转换底层错误：
/// - `std::io::Error` → `EngineError::Io`
/// - `SerializationError` → `EngineError::Serialization`
/// - 其他特定错误 → 相应的变体
#[derive(Error, Debug)]
pub enum EngineError {
    /// IO操作错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 序列化错误
    #[error("Serialization error: {0}")]
    Serialization(#[from] crate::error::SerializationError),

    /// 资源管理错误
    #[error("Resource error: {0}")]
    Resource(String),

    /// 物理系统错误
    #[error("Physics error: {0}")]
    Physics(String),

    /// 网络错误
    #[error("Network error: {0}")]
    Network(String),

    /// 参数验证错误
    #[error("Validation error: {0}")]
    Validation(String),

    /// 配置错误
    #[error("Configuration error: {0}")]
    Config(String),

    /// 未实现的功能
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),

    /// 通用错误
    #[error("General error: {0}")]
    General(String),
}

impl EngineError {
    /// 创建资源错误
    pub fn resource<S: Into<String>>(msg: S) -> Self {
        Self::Resource(msg.into())
    }

    /// 创建物理错误
    pub fn physics<S: Into<String>>(msg: S) -> Self {
        Self::Physics(msg.into())
    }

    /// 创建网络错误
    pub fn network<S: Into<String>>(msg: S) -> Self {
        Self::Network(msg.into())
    }

    /// 创建验证错误
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// 创建配置错误
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    /// 检查是否是IO错误
    pub fn is_io_error(&self) -> bool {
        matches!(self, EngineError::Io(_))
    }

    /// 检查是否是资源错误
    pub fn is_resource_error(&self) -> bool {
        matches!(self, EngineError::Resource(_))
    }
}

/// 服务错误trait
///
/// 为服务实现统一的错误处理。
pub trait ServiceError {
    /// 将服务错误转换为EngineError
    fn into_engine_error(self) -> EngineError;
}

// 为常见错误类型实现ServiceError
impl ServiceError for std::io::Error {
    fn into_engine_error(self) -> EngineError {
        EngineError::Io(self)
    }
}

impl ServiceError for crate::error::SerializationError {
    fn into_engine_error(self) -> EngineError {
        EngineError::Serialization(self)
    }
}

impl ServiceError for String {
    fn into_engine_error(self) -> EngineError {
        EngineError::General(self)
    }
}

/// 结果类型别名
///
/// 简化返回类型定义。
pub type Result<T> = std::result::Result<T, EngineError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = EngineError::resource("File not found");
        assert!(err.is_resource_error());
        assert!(err.to_string().contains("Resource error"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let engine_err: EngineError = io_err.into();
        assert!(engine_err.is_io_error());
    }

    #[test]
    fn test_service_error() {
        let msg = "Test error".to_string();
        let engine_err = msg.into_engine_error();
        assert!(matches!(engine_err, EngineError::General(_)));
    }

    #[test]
    fn test_error_display() {
        let err = EngineError::validation("Invalid parameter");
        assert_eq!(err.to_string(), "Validation error: Invalid parameter");
    }
}
