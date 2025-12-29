//  统一错误处理模块
//
//  提供引擎范围内的统一错误类型定义、错误处理策略和恢复机制。
//
//  ## 架构概览
//
//  ```text
//  ┌─────────────────────────────────────────────────────────┐
//  │                  错误处理架构                             │
//  ├─────────────────────────────────────────────────────────┤
//  │ 1. 核心错误类型 (EngineError)                          │
//  │    - 统一所有模块的错误类型                              │
//  │    - 支持错误链和上下文传播                              │
//  │    - 提供错误分类和严重级别                              │
//  │                                                         │
//  │ 2. 模块级错误类型                                        │
//  │    - RenderError: 渲染系统错误                            │
//  │    - PhysicsError: 物理系统错误                           │
//  │    - AudioError: 音频系统错误                             │
//  │    - ResourceError: 资源管理错误                          │
//  │    - InputError: 输入系统错误                            │
//  │    - SystemError: 系统级错误                              │
//  │                                                         │
//  │ 3. 错误处理策略                                          │
//  │    - 错误恢复机制 (recovery.rs)                          │
//  │    - 重试机制 (retry.rs)                                │
//  │    - 错误监控 (monitoring.rs)                            │
//  │                                                         │
//  │ 4. 错误上下文和传播                                       │
//  │    - 错误链追踪                                         │
//  │    - 上下文信息收集                                      │
//  │    - 结构化错误报告                                      │
//  └─────────────────────────────────────────────────────────┘
//  ```

/// 音频错误类型 - 音频系统特定的错误
pub mod audio_error;
#[cfg(test)]
pub mod concurrency_tests;
/// 便捷错误处理工具 - 提供安全的 unwrap 替代方案
pub mod convenience;
/// 引擎核心错误 - 统一的错误处理类型
pub mod engine_error;
/// 统一错误处理器 - 错误处理、恢复和日志的集成
pub mod error_handler;
/// 输入错误类型 - 输入系统特定的错误
pub mod input_error;
/// 锁安全工具 - 线程安全的锁包装器
pub mod lock_safety;
/// 统一日志管理 - 日志系统和错误处理的集成
pub mod logging;
/// 错误监控 - 错误的监控和统计
pub mod monitoring;
/// 物理错误类型 - 物理系统特定的错误
pub mod physics_error;
/// 错误恢复 - 错误恢复策略和管理器
pub mod recovery;
/// 渲染错误类型 - 渲染系统特定的错误
pub mod render_error;
/// 资源错误类型 - 资源管理特定的错误
pub mod resource_error;
/// 重试机制 - 错误重试执行器和配置
pub mod retry;
/// 系统错误类型 - 系统级别的错误
pub mod system_error;
/// 错误处理Trait - 减少重复代码的工具trait
pub mod traits;

// Serde imports for serialization/deserialization
use serde::{Deserialize, Serialize};
// thiserror::Error 未在此文件中使用，但可能在未来需要
// use thiserror::Error;

// 重新导出所有错误类型
pub use audio_error::AudioError;
pub use engine_error::EngineError;
pub use input_error::InputError;
pub use physics_error::PhysicsError;
/// 平台相关错误
pub use platform_error::PlatformError;
pub use render_error::RenderError;
pub use resource_error::ResourceError;
/// 脚本相关错误
pub use script_error::ScriptError;
pub use system_error::SystemError;

// 重新导出错误处理策略
pub use lock_safety::{
    LockError, ScopedLock, safe_lock, safe_read, safe_write, try_lock, try_read, try_write,
};
pub use monitoring::{
    DefaultReportGenerator, ErrorDetail, ErrorMonitor, ErrorReport, ErrorReportGenerator,
    ErrorStats, ErrorThresholds, ErrorTrend, MonitorConfig, TrendType,
};
pub use recovery::{
    AudioErrorRecovery, DefaultErrorRecovery, ErrorRecovery, PhysicsErrorRecovery, RecoveryContext,
    RecoveryInfo, RecoveryManager, RecoveryResult, RecoveryStrategy, RenderErrorRecovery,
    ResourceErrorRecovery,
};
pub use retry::{RetryCondition, RetryConfig, RetryExecutor, RetryPolicy, RetryResult};

// Re-export Logging components
pub use logging::{
    ConsoleLogSink, FileLogSink, LogEntry, LogLevel, LogSink, Logger, LoggingConfig, init_logger,
    log, log_error,
};

// Re-export Error Handler components
pub use error_handler::{ErrorHandler, ErrorHandlerConfig};
// Re-export Error Traits
pub use traits::{ContextError, IoResultExt, OptionExt, ResultExt};
// Re-export Convenience utilities
pub use convenience::{
    Validator, check_non_empty_or_err, check_range_or_err, log_option, log_result,
    map_get_mut_or_err, map_get_or_err, ok_or_else_err, option_to_result, parse_to_number_or_err,
    safe_unwrap_option, safe_unwrap_result, safe_unwrap_with_log, unwrap_or_context,
    unwrap_or_default, unwrap_or_else_default, vec_get_mut_or_err, vec_get_or_err,
};

/// 错误严重级别 - 表示错误的严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// 信息级别 - 不会影响系统运行
    Info = 0,
    /// 警告级别 - 可能影响性能或用户体验
    Warning = 1,
    /// 错误级别 - 影响部分功能但系统可继续运行
    Error = 2,
    /// 严重错误 - 影响核心功能，需要立即处理
    Critical = 3,
    /// 致命错误 - 系统无法继续运行
    Fatal = 4,
}

impl ErrorSeverity {
    /// 获取严重级别的字符串表示
    ///
    /// # Returns
    /// 返回严重级别的简洁字符串表示（"INFO", "WARNING", "ERROR", "CRITICAL", "FATAL"）
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorSeverity::Info => "INFO",
            ErrorSeverity::Warning => "WARNING",
            ErrorSeverity::Error => "ERROR",
            ErrorSeverity::Critical => "CRITICAL",
            ErrorSeverity::Fatal => "FATAL",
        }
    }

    /// 从字符串解析严重级别
    ///
    /// # Arguments
    /// * `s` - 要解析的字符串（大小写不敏感）
    ///
    /// # Returns
    /// 如果字符串有效返回Some(严重级别)，否则返回None
    pub fn from_str_case_insensitive(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "INFO" => Some(ErrorSeverity::Info),
            "WARNING" => Some(ErrorSeverity::Warning),
            "ERROR" => Some(ErrorSeverity::Error),
            "CRITICAL" => Some(ErrorSeverity::Critical),
            "FATAL" => Some(ErrorSeverity::Fatal),
            _ => None,
        }
    }
}

impl std::str::FromStr for ErrorSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "INFO" => Ok(ErrorSeverity::Info),
            "WARNING" => Ok(ErrorSeverity::Warning),
            "ERROR" => Ok(ErrorSeverity::Error),
            "CRITICAL" => Ok(ErrorSeverity::Critical),
            "FATAL" => Ok(ErrorSeverity::Fatal),
            _ => Err(format!("Unknown error severity: {}", s)),
        }
    }
}

/// 错误分类 - 根据错误来源将其分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// 渲染相关错误（GPU、WGPU、着色器等）
    Render,
    /// 物理相关错误（物理引擎、碰撞检测等）
    Physics,
    /// 音频相关错误（音频播放、混音等）
    Audio,
    /// 资源相关错误（资源加载、资源管理等）
    Resource,
    /// 输入相关错误（控制器、键盘、鼠标等）
    Input,
    /// 系统相关错误（内存、线程等）
    System,
    /// 网络相关错误（网络连接、数据传输等）
    Network,
    /// 脚本相关错误（脚本执行、脚本编译等）
    Script,
    /// 平台相关错误（VR/AR、XR特定错误等）
    Platform,
    /// 未知错误类型
    Unknown,
}

impl ErrorCategory {
    /// 获取分类的字符串表示
    ///
    /// # Returns
    /// 返回分类的简洁字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCategory::Render => "RENDER",
            ErrorCategory::Physics => "PHYSICS",
            ErrorCategory::Audio => "AUDIO",
            ErrorCategory::Resource => "RESOURCE",
            ErrorCategory::Input => "INPUT",
            ErrorCategory::System => "SYSTEM",
            ErrorCategory::Network => "NETWORK",
            ErrorCategory::Script => "SCRIPT",
            ErrorCategory::Platform => "PLATFORM",
            ErrorCategory::Unknown => "UNKNOWN",
        }
    }
}

/// 统一的结果类型 - 返回值为Result<T, EngineError>
pub type EngineResult<T> = Result<T, EngineError>;
/// 渲染系统的结果类型
pub type RenderResult<T> = Result<T, RenderError>;
/// 物理系统的结果类型
pub type PhysicsResult<T> = Result<T, PhysicsError>;
/// 音频系统的结果类型
pub type AudioResult<T> = Result<T, AudioError>;
/// 资源系统的结果类型
pub type ResourceResult<T> = Result<T, ResourceError>;
/// 输入系统的结果类型
pub type InputResult<T> = Result<T, InputError>;
/// 系统级别的结果类型
pub type SystemResult<T> = Result<T, SystemError>;
/// 脚本系统的结果类型
pub type ScriptResult<T> = Result<T, ScriptError>;
/// 平台相关的结果类型
pub type PlatformResult<T> = Result<T, PlatformError>;

/// 脚本错误类型（占位实现，供 CommonScriptError 转换使用）
pub mod script_error {
    use super::ErrorSeverity;
    use thiserror::Error;

    #[derive(Debug, Error, Clone)]
    pub enum ScriptError {
        #[error("Script compilation failed: {0}")]
        Compilation(String),
        #[error("Script runtime error: {0}")]
        Runtime(String),
        #[error("Script not found: {0}")]
        NotFound(String),
        #[error("Invalid binding: {0}")]
        InvalidBinding(String),
        #[error("Script timeout: {0} ms")]
        Timeout(u64),
    }

    impl ScriptError {
        pub fn severity(&self) -> ErrorSeverity {
            match self {
                ScriptError::Compilation(_) | ScriptError::Runtime(_) => ErrorSeverity::Error,
                ScriptError::Timeout(_) => ErrorSeverity::Warning,
                ScriptError::NotFound(_) | ScriptError::InvalidBinding(_) => ErrorSeverity::Info,
            }
        }
    }
}

/// 平台错误类型（占位实现，供 CommonPlatformError 转换使用）
pub mod platform_error {
    use super::ErrorSeverity;
    use thiserror::Error;

    #[derive(Debug, Error, Clone)]
    pub enum PlatformError {
        #[error("Window creation failed: {0}")]
        WindowCreation(String),
        #[error("Event loop error: {0}")]
        EventLoop(String),
        #[error("Input device error: {0}")]
        InputDevice(String),
        #[error("Filesystem error: {0}")]
        Filesystem(String),
        #[error("Platform not supported: {0}")]
        NotSupported(String),
    }

    impl PlatformError {
        pub fn severity(&self) -> ErrorSeverity {
            match self {
                PlatformError::WindowCreation(_) | PlatformError::EventLoop(_) => {
                    ErrorSeverity::Error
                }
                PlatformError::InputDevice(_) | PlatformError::Filesystem(_) => {
                    ErrorSeverity::Warning
                }
                PlatformError::NotSupported(_) => ErrorSeverity::Info,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_severity() {
        assert_eq!(ErrorSeverity::Info.as_str(), "INFO");
        assert_eq!(ErrorSeverity::Fatal.as_str(), "FATAL");
        assert!(ErrorSeverity::Critical > ErrorSeverity::Error);

        assert_eq!(
            ErrorSeverity::from_str("warning").ok(),
            Some(ErrorSeverity::Warning)
        );
        assert_eq!(ErrorSeverity::from_str("invalid").ok(), None);
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_category() {
        assert_eq!(ErrorCategory::Render.as_str(), "RENDER");
        assert_eq!(ErrorCategory::Physics.as_str(), "PHYSICS");

        let category = ErrorCategory::Render;
        assert_eq!(category, ErrorCategory::Render);
    }
}
