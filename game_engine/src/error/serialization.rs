//! 序列化错误类型
//!
//! 提供统一的序列化和反序列化错误处理。

use thiserror::Error;

/// 序列化错误
///
/// 当序列化或反序列化操作失败时返回此错误。
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SerializationError {
    /// 编码错误
    #[error("Serialization encode error: {0}")]
    Encode(String),

    /// 解码错误
    #[error("Serialization decode error: {0}")]
    Decode(String),

    /// 不支持的版本
    #[error("Unsupported serialization version: {0}")]
    UnsupportedVersion(u32),

    /// 数据损坏
    #[error("Corrupted data: {0}")]
    CorruptedData(String),

    /// 数据过大
    #[error("Data too large: {0} bytes exceeds maximum of {1} bytes")]
    DataTooLarge(usize, usize),
}

impl SerializationError {
    /// 创建编码错误
    pub fn encode<S: Into<String>>(msg: S) -> Self {
        Self::Encode(msg.into())
    }

    /// 创建解码错误
    pub fn decode<S: Into<String>>(msg: S) -> Self {
        Self::Decode(msg.into())
    }

    /// 检查是否是编码错误
    pub fn is_encode_error(&self) -> bool {
        matches!(self, Self::Encode(_))
    }

    /// 检查是否是解码错误
    pub fn is_decode_error(&self) -> bool {
        matches!(self, Self::Decode(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization_error_display() {
        let err = SerializationError::encode("test error");
        assert_eq!(err.to_string(), "Serialization encode error: test error");

        let err = SerializationError::decode("test error");
        assert_eq!(err.to_string(), "Serialization decode error: test error");
    }

    #[test]
    fn test_serialization_error_helpers() {
        let err = SerializationError::encode("test");
        assert!(err.is_encode_error());
        assert!(!err.is_decode_error());

        let err = SerializationError::decode("test");
        assert!(err.is_decode_error());
        assert!(!err.is_encode_error());
    }

    #[test]
    fn test_serialization_error_equality() {
        let err1 = SerializationError::Encode("test".to_string());
        let err2 = SerializationError::Encode("test".to_string());
        assert_eq!(err1, err2);

        let err3 = SerializationError::Decode("test".to_string());
        assert_ne!(err1, err3);
    }
}
