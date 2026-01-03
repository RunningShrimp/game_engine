//! 导入器错误类型

use std::io;

pub type ImportResult<T> = Result<T, ImportError>;

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to parse file: {0}")]
    ParseError(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Missing required data: {0}")]
    MissingData(String),

    #[error("Invalid file version: {0}")]
    InvalidVersion(String),

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Feature not supported: {0}")]
    NotSupported(String),
}
