//! 资源管理错误类型
//!
//! 定义了资源管理相关的所有错误类型，包括资源加载、缓存、流式传输等。

use crate::error::{ErrorSeverity, ErrorCategory};
use thiserror::Error;

/// 资源管理错误
///
/// 涵盖了资源管理中的所有可能的错误情况，
/// 从资源发现到资源释放。
#[derive(Error, Debug, Clone)]
pub enum ResourceError {
    /// 资源未找到
    #[error("Resource not found: {path}")]
    NotFound {
        /// 资源路径
        path: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源加载失败
    #[error("Resource loading failed: {path} - {message}")]
    LoadFailed {
        /// 资源路径
        path: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源格式无效
    #[error("Invalid resource format: {path} - expected: {expected}")]
    InvalidFormat {
        /// 资源路径
        path: String,
        /// 期望格式
        expected: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源解析错误
    #[error("Resource parsing error: {path} - {message}")]
    Parsing {
        /// 资源路径
        path: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源依赖缺失
    #[error("Resource dependency missing: {dependency} for {resource}")]
    DependencyMissing {
        /// 资源路径
        resource: String,
        /// 缺失的依赖
        dependency: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源循环依赖
    #[error("Circular resource dependency: {path}")]
    CircularDependency {
        /// 资源路径
        path: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源缓存错误
    #[error("Resource cache error: {message}")]
    Cache {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源内存不足
    #[error("Resource out of memory: {resource} - {message}")]
    OutOfMemory {
        /// 资源名称
        resource: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源上传错误
    #[error("Resource upload error: {resource} - {message}")]
    Upload {
        /// 资源名称
        resource: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源下载错误
    #[error("Resource download error: {url} - {message}")]
    Download {
        /// 资源URL
        url: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源压缩错误
    #[error("Resource compression error: {resource} - {message}")]
    Compression {
        /// 资源名称
        resource: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源解压缩错误
    #[error("Resource decompression error: {resource} - {message}")]
    Decompression {
        /// 资源名称
        resource: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源验证错误
    #[error("Resource validation error: {resource} - {message}")]
    Validation {
        /// 资源名称
        resource: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源版本不匹配
    #[error("Resource version mismatch: {resource} - expected: {expected}, found: {found}")]
    VersionMismatch {
        /// 资源名称
        resource: String,
        /// 期望版本
        expected: String,
        /// 实际版本
        found: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源权限错误
    #[error("Resource permission error: {path} - {message}")]
    Permission {
        /// 资源路径
        path: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源并发访问错误
    #[error("Resource concurrent access error: {resource} - {message}")]
    ConcurrentAccess {
        /// 资源名称
        resource: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源流式传输错误
    #[error("Resource streaming error: {resource} - {message}")]
    Streaming {
        /// 资源名称
        resource: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源热重载错误
    #[error("Resource hot reload error: {resource} - {message}")]
    HotReload {
        /// 资源名称
        resource: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源包错误
    #[error("Resource bundle error: {bundle} - {message}")]
    Bundle {
        /// 资源包名称
        bundle: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源元数据错误
    #[error("Resource metadata error: {resource} - {message}")]
    Metadata {
        /// 资源名称
        resource: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源引用错误
    #[error("Resource reference error: {reference} - {message}")]
    Reference {
        /// 资源引用
        reference: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源池错误
    #[error("Resource pool error: {message}")]
    Pool {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源生命周期错误
    #[error("Resource lifecycle error: {resource} - {message}")]
    Lifecycle {
        /// 资源名称
        resource: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 资源配置错误
    #[error("Resource configuration error: {message}")]
    Configuration {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 通用资源错误
    #[error("Resource error: {message}")]
    General {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },
}

impl ResourceError {
    /// 创建资源未找到错误
    pub fn not_found(path: impl Into<String>) -> Self {
        Self::NotFound {
            path: path.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建资源加载失败错误
    pub fn load_failed(
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::LoadFailed {
            path: path.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建无效格式错误
    pub fn invalid_format(
        path: impl Into<String>,
        expected: impl Into<String>,
    ) -> Self {
        Self::InvalidFormat {
            path: path.into(),
            expected: expected.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建解析错误
    pub fn parsing(
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Parsing {
            path: path.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建依赖缺失错误
    pub fn dependency_missing(
        resource: impl Into<String>,
        dependency: impl Into<String>,
    ) -> Self {
        Self::DependencyMissing {
            resource: resource.into(),
            dependency: dependency.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建循环依赖错误
    pub fn circular_dependency(path: impl Into<String>) -> Self {
        Self::CircularDependency {
            path: path.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建缓存错误
    pub fn cache(message: impl Into<String>) -> Self {
        Self::Cache {
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建内存不足错误
    pub fn out_of_memory(
        resource: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::OutOfMemory {
            resource: resource.into(),
            message: message.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建上传错误
    pub fn upload(
        resource: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Upload {
            resource: resource.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建下载错误
    pub fn download(
        url: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Download {
            url: url.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建压缩错误
    pub fn compression(
        resource: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Compression {
            resource: resource.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建解压缩错误
    pub fn decompression(
        resource: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Decompression {
            resource: resource.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建验证错误
    pub fn validation(
        resource: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Validation {
            resource: resource.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建版本不匹配错误
    pub fn version_mismatch(
        resource: impl Into<String>,
        expected: impl Into<String>,
        found: impl Into<String>,
    ) -> Self {
        Self::VersionMismatch {
            resource: resource.into(),
            expected: expected.into(),
            found: found.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建权限错误
    pub fn permission(
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Permission {
            path: path.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建并发访问错误
    pub fn concurrent_access(
        resource: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::ConcurrentAccess {
            resource: resource.into(),
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建流式传输错误
    pub fn streaming(
        resource: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Streaming {
            resource: resource.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建热重载错误
    pub fn hot_reload(
        resource: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::HotReload {
            resource: resource.into(),
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建资源包错误
    pub fn bundle(
        bundle: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Bundle {
            bundle: bundle.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建元数据错误
    pub fn metadata(
        resource: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Metadata {
            resource: resource.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建引用错误
    pub fn reference(
        reference: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Reference {
            reference: reference.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建池错误
    pub fn pool(message: impl Into<String>) -> Self {
        Self::Pool {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建生命周期错误
    pub fn lifecycle(
        resource: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Lifecycle {
            resource: resource.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建配置错误
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建通用资源错误
    pub fn general(message: impl Into<String>) -> Self {
        Self::General {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建带有严重级别的通用资源错误
    pub fn general_with_severity(
        message: impl Into<String>,
        severity: ErrorSeverity,
    ) -> Self {
        Self::General {
            message: message.into(),
            severity,
        }
    }

    /// 获取错误的严重级别
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            ResourceError::NotFound { severity, .. }
            | ResourceError::LoadFailed { severity, .. }
            | ResourceError::InvalidFormat { severity, .. }
            | ResourceError::Parsing { severity, .. }
            | ResourceError::DependencyMissing { severity, .. }
            | ResourceError::CircularDependency { severity, .. }
            | ResourceError::Cache { severity, .. }
            | ResourceError::OutOfMemory { severity, .. }
            | ResourceError::Upload { severity, .. }
            | ResourceError::Download { severity, .. }
            | ResourceError::Compression { severity, .. }
            | ResourceError::Decompression { severity, .. }
            | ResourceError::Validation { severity, .. }
            | ResourceError::VersionMismatch { severity, .. }
            | ResourceError::Permission { severity, .. }
            | ResourceError::ConcurrentAccess { severity, .. }
            | ResourceError::Streaming { severity, .. }
            | ResourceError::HotReload { severity, .. }
            | ResourceError::Bundle { severity, .. }
            | ResourceError::Metadata { severity, .. }
            | ResourceError::Reference { severity, .. }
            | ResourceError::Pool { severity, .. }
            | ResourceError::Lifecycle { severity, .. }
            | ResourceError::Configuration { severity, .. }
            | ResourceError::General { severity, .. } => *severity,
        }
    }

    /// 检查错误是否可恢复
    pub fn is_recoverable(&self) -> bool {
        match self {
            // 严重错误通常不可恢复
            ResourceError::CircularDependency { severity, .. }
            | ResourceError::OutOfMemory { severity, .. } => *severity < ErrorSeverity::Critical,

            // 权限错误通常不可恢复
            ResourceError::Permission { severity, .. } => *severity < ErrorSeverity::Critical,

            // 缓存和热重载错误通常可恢复
            ResourceError::Cache { .. }
            | ResourceError::HotReload { .. }
            | ResourceError::ConcurrentAccess { .. } => true,

            // 版本不匹配通常可恢复（可以使用兼容模式）
            ResourceError::VersionMismatch { .. } => true,

            // 依赖问题可能可恢复（可以尝试其他依赖）
            ResourceError::DependencyMissing { .. } => true,

            // 其他错误需要根据严重级别判断
            _ => self.severity() < ErrorSeverity::Critical,
        }
    }

    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        ErrorCategory::Resource
    }

    /// 检查是否为文件相关错误
    pub fn is_file_related(&self) -> bool {
        matches!(
            self,
            ResourceError::NotFound { .. }
                | ResourceError::LoadFailed { .. }
                | ResourceError::InvalidFormat { .. }
                | ResourceError::Parsing { .. }
                | ResourceError::Permission { .. }
        )
    }

    /// 检查是否为内存相关错误
    pub fn is_memory_related(&self) -> bool {
        matches!(
            self,
            ResourceError::OutOfMemory { .. } | ResourceError::Cache { .. }
        )
    }

    /// 检查是否为网络相关错误
    pub fn is_network_related(&self) -> bool {
        matches!(
            self,
            ResourceError::Download { .. } | ResourceError::Streaming { .. }
        )
    }

    /// 检查是否为依赖相关错误
    pub fn is_dependency_related(&self) -> bool {
        matches!(
            self,
            ResourceError::DependencyMissing { .. }
                | ResourceError::CircularDependency { .. }
                | ResourceError::Reference { .. }
        )
    }
}

// 从IO错误转换
impl From<std::io::Error> for ResourceError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                ResourceError::not_found(err.to_string())
            }
            std::io::ErrorKind::PermissionDenied => {
                ResourceError::permission(err.to_string(), "Access denied")
            }
            std::io::ErrorKind::OutOfMemory => {
                ResourceError::out_of_memory("System", err.to_string())
            }
            _ => ResourceError::load_failed("Unknown", err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_error_creation() {
        let err = ResourceError::not_found("textures/player.png");
        assert_eq!(err.severity(), ErrorSeverity::Error);
        assert!(err.is_file_related());
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_resource_error_severity() {
        let critical_err = ResourceError::circular_dependency("resource_a");
        assert_eq!(critical_err.severity(), ErrorSeverity::Critical);
        assert!(!critical_err.is_recoverable());

        let normal_err = ResourceError::general("Temporary resource issue");
        assert_eq!(normal_err.severity(), ErrorSeverity::Error);
        assert!(normal_err.is_recoverable());
    }

    #[test]
    fn test_resource_error_categories() {
        let file_err = ResourceError::invalid_format("data.json", "JSON");
        assert!(file_err.is_file_related());

        let memory_err = ResourceError::out_of_memory("texture_large", "GPU memory full");
        assert!(memory_err.is_memory_related());

        let network_err = ResourceError::download("http://example.com/asset", "Network timeout");
        assert!(network_err.is_network_related());

        let dep_err = ResourceError::dependency_missing("model", "texture");
        assert!(dep_err.is_dependency_related());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let resource_err: ResourceError = io_err.into();
        
        assert!(matches!(resource_err, ResourceError::NotFound { .. }));
        assert_eq!(resource_err.severity(), ErrorSeverity::Error);
    }

    #[test]
    fn test_version_mismatch_error() {
        let err = ResourceError::version_mismatch("shader", "2.0", "1.0");
        assert_eq!(err.severity(), ErrorSeverity::Warning);
        assert!(err.is_recoverable());
    }
}