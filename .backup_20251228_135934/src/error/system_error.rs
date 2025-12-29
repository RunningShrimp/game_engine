//  系统级错误类型
//
//  定义了系统级相关的所有错误类型，包括初始化、配置、权限等。

use crate::error::{ErrorCategory, ErrorSeverity};
use thiserror::Error;

/// 系统级错误
///
/// 涵盖了系统级操作中的所有可能的错误情况，
/// 从系统初始化到资源管理。
#[derive(Error, Debug, Clone)]
pub enum SystemError {
    /// 系统初始化错误
    #[error("System initialization failed: {component} - {message}")]
    Initialization {
        /// 组件名称
        component: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统关闭错误
    #[error("System shutdown failed: {component} - {message}")]
    Shutdown {
        /// 组件名称
        component: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统配置错误
    #[error("System configuration error: {config} - {message}")]
    Configuration {
        /// 配置名称
        config: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统权限错误
    #[error("System permission error: {permission} - {message}")]
    Permission {
        /// 权限名称
        permission: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统资源不足
    #[error("System resource exhausted: {resource} - {message}")]
    ResourceExhausted {
        /// 资源名称
        resource: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统内存不足
    #[error("System out of memory: {message}")]
    OutOfMemory {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统超时
    #[error("System timeout: {operation} - {message}")]
    Timeout {
        /// 操作名称
        operation: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统并发错误
    #[error("System concurrency error: {message}")]
    Concurrency {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统线程错误
    #[error("System thread error: {thread} - {message}")]
    Thread {
        /// 线程名称
        thread: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统同步错误
    #[error("System synchronization error: {message}")]
    Synchronization {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统文件系统错误
    #[error("System filesystem error: {path} - {message}")]
    Filesystem {
        /// 文件路径
        path: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统网络错误
    #[error("System network error: {message}")]
    Network {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统平台错误
    #[error("System platform error: {platform} - {message}")]
    Platform {
        /// 平台名称
        platform: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统API错误
    #[error("System API error: {api} - {message}")]
    Api {
        /// API名称
        api: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统版本不兼容
    #[error("System version incompatibility: {component} - expected: {expected}, found: {found}")]
    VersionIncompatibility {
        /// 组件名称
        component: String,
        /// 期望版本
        expected: String,
        /// 实际版本
        found: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统依赖错误
    #[error("System dependency error: {dependency} - {message}")]
    Dependency {
        /// 依赖名称
        dependency: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统状态错误
    #[error("System state error: {state} - {message}")]
    State {
        /// 状态名称
        state: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统日志错误
    #[error("System logging error: {message}")]
    Logging {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统监控错误
    #[error("System monitoring error: {message}")]
    Monitoring {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统性能错误
    #[error("System performance error: {metric} - {message}")]
    Performance {
        /// 性能指标名称
        metric: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统安全错误
    #[error("System security error: {message}")]
    Security {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统数据库错误
    #[error("System database error: {database} - {message}")]
    Database {
        /// 数据库名称
        database: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统缓存错误
    #[error("System cache error: {cache} - {message}")]
    Cache {
        /// 缓存名称
        cache: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统序列化错误
    #[error("System serialization error: {message}")]
    Serialization {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统反序列化错误
    #[error("System deserialization error: {message}")]
    Deserialization {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统编码错误
    #[error("System encoding error: {encoding} - {message}")]
    Encoding {
        /// 编码名称
        encoding: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统解码错误
    #[error("System decoding error: {encoding} - {message}")]
    Decoding {
        /// 解码名称
        encoding: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统压缩错误
    #[error("System compression error: {message}")]
    Compression {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统解压缩错误
    #[error("System decompression error: {message}")]
    Decompression {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统加密错误
    #[error("System encryption error: {message}")]
    Encryption {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统解密错误
    #[error("System decryption error: {message}")]
    Decryption {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统哈希错误
    #[error("System hash error: {message}")]
    Hash {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统校验和错误
    #[error("System checksum error: {message}")]
    Checksum {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统时间错误
    #[error("System time error: {message}")]
    Time {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 系统随机数生成错误
    #[error("System random number generation error: {message}")]
    Random {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 通用系统错误
    #[error("System error: {message}")]
    General {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },
}

impl SystemError {
    /// 创建初始化错误
    pub fn initialization(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Initialization {
            component: component.into(),
            message: message.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建关闭错误
    pub fn shutdown(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Shutdown {
            component: component.into(),
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建配置错误
    pub fn configuration(config: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Configuration {
            config: config.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建权限错误
    pub fn permission(permission: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Permission {
            permission: permission.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建资源不足错误
    pub fn resource_exhausted(resource: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ResourceExhausted {
            resource: resource.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建内存不足错误
    pub fn out_of_memory(message: impl Into<String>) -> Self {
        Self::OutOfMemory {
            message: message.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建超时错误
    pub fn timeout(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Timeout {
            operation: operation.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建并发错误
    pub fn concurrency(message: impl Into<String>) -> Self {
        Self::Concurrency {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建线程错误
    pub fn thread(thread: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Thread {
            thread: thread.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建同步错误
    pub fn synchronization(message: impl Into<String>) -> Self {
        Self::Synchronization {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建文件系统错误
    pub fn filesystem(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Filesystem {
            path: path.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建网络错误
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建平台错误
    pub fn platform(platform: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Platform {
            platform: platform.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建API错误
    pub fn api(api: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Api {
            api: api.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建版本不兼容错误
    pub fn version_incompatibility(
        component: impl Into<String>,
        expected: impl Into<String>,
        found: impl Into<String>,
    ) -> Self {
        Self::VersionIncompatibility {
            component: component.into(),
            expected: expected.into(),
            found: found.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建依赖错误
    pub fn dependency(dependency: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Dependency {
            dependency: dependency.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建状态错误
    pub fn state(state: impl Into<String>, message: impl Into<String>) -> Self {
        Self::State {
            state: state.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建日志错误
    pub fn logging(message: impl Into<String>) -> Self {
        Self::Logging {
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建监控错误
    pub fn monitoring(message: impl Into<String>) -> Self {
        Self::Monitoring {
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建性能错误
    pub fn performance(metric: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Performance {
            metric: metric.into(),
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建安全错误
    pub fn security(message: impl Into<String>) -> Self {
        Self::Security {
            message: message.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建数据库错误
    pub fn database(database: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Database {
            database: database.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建缓存错误
    pub fn cache(cache: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Cache {
            cache: cache.into(),
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建序列化错误
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建反序列化错误
    pub fn deserialization(message: impl Into<String>) -> Self {
        Self::Deserialization {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建编码错误
    pub fn encoding(encoding: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Encoding {
            encoding: encoding.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建解码错误
    pub fn decoding(encoding: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Decoding {
            encoding: encoding.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建加密错误
    pub fn encryption(message: impl Into<String>) -> Self {
        Self::Encryption {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建解密错误
    pub fn decryption(message: impl Into<String>) -> Self {
        Self::Decryption {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建哈希错误
    pub fn hash(message: impl Into<String>) -> Self {
        Self::Hash {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建校验和错误
    pub fn checksum(message: impl Into<String>) -> Self {
        Self::Checksum {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建时间错误
    pub fn time(message: impl Into<String>) -> Self {
        Self::Time {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建随机数生成错误
    pub fn random(message: impl Into<String>) -> Self {
        Self::Random {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建通用系统错误
    pub fn general(message: impl Into<String>) -> Self {
        Self::General {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建带有严重级别的通用系统错误
    pub fn general_with_severity(message: impl Into<String>, severity: ErrorSeverity) -> Self {
        Self::General {
            message: message.into(),
            severity,
        }
    }

    /// 获取错误的严重级别
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            SystemError::Initialization { severity, .. }
            | SystemError::Shutdown { severity, .. }
            | SystemError::Configuration { severity, .. }
            | SystemError::Permission { severity, .. }
            | SystemError::ResourceExhausted { severity, .. }
            | SystemError::OutOfMemory { severity, .. }
            | SystemError::Timeout { severity, .. }
            | SystemError::Concurrency { severity, .. }
            | SystemError::Thread { severity, .. }
            | SystemError::Synchronization { severity, .. }
            | SystemError::Filesystem { severity, .. }
            | SystemError::Network { severity, .. }
            | SystemError::Platform { severity, .. }
            | SystemError::Api { severity, .. }
            | SystemError::VersionIncompatibility { severity, .. }
            | SystemError::Dependency { severity, .. }
            | SystemError::State { severity, .. }
            | SystemError::Logging { severity, .. }
            | SystemError::Monitoring { severity, .. }
            | SystemError::Performance { severity, .. }
            | SystemError::Security { severity, .. }
            | SystemError::Database { severity, .. }
            | SystemError::Cache { severity, .. }
            | SystemError::Serialization { severity, .. }
            | SystemError::Deserialization { severity, .. }
            | SystemError::Encoding { severity, .. }
            | SystemError::Decoding { severity, .. }
            | SystemError::Compression { severity, .. }
            | SystemError::Decompression { severity, .. }
            | SystemError::Encryption { severity, .. }
            | SystemError::Decryption { severity, .. }
            | SystemError::Hash { severity, .. }
            | SystemError::Checksum { severity, .. }
            | SystemError::Time { severity, .. }
            | SystemError::Random { severity, .. }
            | SystemError::General { severity, .. } => *severity,
        }
    }

    /// 检查错误是否可恢复
    pub fn is_recoverable(&self) -> bool {
        match self {
            // 严重错误通常不可恢复
            SystemError::Initialization { severity, .. }
            | SystemError::OutOfMemory { severity, .. }
            | SystemError::Security { severity, .. } => *severity < ErrorSeverity::Critical,

            // 警告级别错误通常可恢复
            SystemError::Shutdown { severity, .. }
            | SystemError::Logging { severity, .. }
            | SystemError::Monitoring { severity, .. }
            | SystemError::Performance { severity, .. }
            | SystemError::Cache { severity, .. } => *severity <= ErrorSeverity::Warning,

            // 配置和依赖错误通常可恢复
            SystemError::Configuration { .. }
            | SystemError::Dependency { .. }
            | SystemError::VersionIncompatibility { .. } => true,

            // 资源不足可能可恢复（可以等待资源释放）
            SystemError::ResourceExhausted { .. } => true,

            // 超时错误通常可恢复（可以重试）
            SystemError::Timeout { .. } => true,

            // 其他错误需要根据严重级别判断
            _ => self.severity() < ErrorSeverity::Critical,
        }
    }

    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        ErrorCategory::System
    }

    /// 检查是否为初始化相关错误
    pub fn is_initialization_related(&self) -> bool {
        matches!(
            self,
            SystemError::Initialization { .. }
                | SystemError::Configuration { .. }
                | SystemError::Dependency { .. }
                | SystemError::VersionIncompatibility { .. }
        )
    }

    /// 检查是否为资源相关错误
    pub fn is_resource_related(&self) -> bool {
        matches!(
            self,
            SystemError::ResourceExhausted { .. }
                | SystemError::OutOfMemory { .. }
                | SystemError::Cache { .. }
        )
    }

    /// 检查是否为并发相关错误
    pub fn is_concurrency_related(&self) -> bool {
        matches!(
            self,
            SystemError::Concurrency { .. }
                | SystemError::Thread { .. }
                | SystemError::Synchronization { .. }
        )
    }

    /// 检查是否为存储相关错误
    pub fn is_storage_related(&self) -> bool {
        matches!(
            self,
            SystemError::Filesystem { .. }
                | SystemError::Database { .. }
                | SystemError::Serialization { .. }
                | SystemError::Deserialization { .. }
        )
    }

    /// 检查是否为网络相关错误
    pub fn is_network_related(&self) -> bool {
        matches!(self, SystemError::Network { .. })
    }

    /// 检查是否为安全相关错误
    pub fn is_security_related(&self) -> bool {
        matches!(
            self,
            SystemError::Security { .. }
                | SystemError::Permission { .. }
                | SystemError::Encryption { .. }
                | SystemError::Decryption { .. }
                | SystemError::Hash { .. }
                | SystemError::Checksum { .. }
        )
    }
}

// 从IO错误转换
impl From<std::io::Error> for SystemError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => SystemError::filesystem("Unknown", err.to_string()),
            std::io::ErrorKind::PermissionDenied => {
                SystemError::permission("FileAccess", err.to_string())
            }
            std::io::ErrorKind::OutOfMemory => SystemError::out_of_memory(err.to_string()),
            std::io::ErrorKind::TimedOut => SystemError::timeout("IOOperation", err.to_string()),
            _ => SystemError::general(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_error_creation() {
        let err = SystemError::initialization("Renderer", "Failed to initialize GPU");
        assert_eq!(err.severity(), ErrorSeverity::Critical);
        assert!(err.is_initialization_related());
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_system_error_severity() {
        let critical_err = SystemError::out_of_memory("System memory exhausted");
        assert_eq!(critical_err.severity(), ErrorSeverity::Critical);
        assert!(!critical_err.is_recoverable());

        let normal_err = SystemError::general("Temporary system issue");
        assert_eq!(normal_err.severity(), ErrorSeverity::Error);
        assert!(normal_err.is_recoverable());
    }

    #[test]
    fn test_system_error_categories() {
        let init_err = SystemError::configuration("config.json", "Invalid format");
        assert!(init_err.is_initialization_related());

        let resource_err = SystemError::resource_exhausted("FileHandles", "Too many open files");
        assert!(resource_err.is_resource_related());

        let concurrency_err = SystemError::concurrency("Race condition detected");
        assert!(concurrency_err.is_concurrency_related());

        let storage_err = SystemError::filesystem("data/save.dat", "Disk full");
        assert!(storage_err.is_storage_related());

        let network_err = SystemError::network("Connection refused");
        assert!(network_err.is_network_related());

        let security_err = SystemError::security("Unauthorized access attempt");
        assert!(security_err.is_security_related());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let system_err: SystemError = io_err.into();

        assert!(matches!(system_err, SystemError::Filesystem { .. }));
        assert_eq!(system_err.severity(), ErrorSeverity::Error);
    }

    #[test]
    fn test_version_incompatibility_error() {
        let err = SystemError::version_incompatibility("GraphicsDriver", "2.0", "1.0");
        assert_eq!(err.severity(), ErrorSeverity::Error);
        assert!(err.is_initialization_related());
        assert!(err.is_recoverable());
    }
}
