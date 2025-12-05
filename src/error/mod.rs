//! 统一错误处理模块
//!
//! 提供引擎范围内的统一错误类型定义、错误处理策略和恢复机制。
//!
//! ## 架构概览
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                  错误处理架构                             │
//! ├─────────────────────────────────────────────────────────┤
//! │ 1. 核心错误类型 (EngineError)                          │
//! │    - 统一所有模块的错误类型                              │
//! │    - 支持错误链和上下文传播                              │
//! │    - 提供错误分类和严重级别                              │
//! │                                                         │
//! │ 2. 模块级错误类型                                        │
//! │    - RenderError: 渲染系统错误                            │
//! │    - PhysicsError: 物理系统错误                           │
//! │    - AudioError: 音频系统错误                             │
//! │    - ResourceError: 资源管理错误                          │
//! │    - InputError: 输入系统错误                            │
//! │    - SystemError: 系统级错误                              │
//! │                                                         │
//! │ 3. 错误处理策略                                          │
//! │    - 错误恢复机制 (recovery.rs)                          │
//! │    - 重试机制 (retry.rs)                                │
//! │    - 错误监控 (monitoring.rs)                            │
//! │                                                         │
//! │ 4. 错误上下文和传播                                       │
//! │    - 错误链追踪                                         │
//! │    - 上下文信息收集                                      │
//! │    - 结构化错误报告                                      │
//! └─────────────────────────────────────────────────────────┘
//! ```

pub mod engine_error;
pub mod render_error;
pub mod physics_error;
pub mod audio_error;
pub mod resource_error;
pub mod input_error;
pub mod system_error;
pub mod recovery;
pub mod retry;
pub mod monitoring;
pub mod lock_safety;

// Serde imports for serialization/deserialization
use serde::{Serialize, Deserialize};

// 重新导出所有错误类型
pub use engine_error::EngineError;
pub use render_error::RenderError;
pub use physics_error::PhysicsError;
pub use audio_error::AudioError;
pub use resource_error::ResourceError;
pub use input_error::InputError;
pub use system_error::SystemError;

// 重新导出错误处理策略
pub use recovery::{
    RecoveryResult, RecoveryInfo, RecoveryStrategy, RecoveryContext,
    ErrorRecovery, DefaultErrorRecovery, RenderErrorRecovery, AudioErrorRecovery,
    PhysicsErrorRecovery, ResourceErrorRecovery, RecoveryManager
};
pub use retry::{
    RetryConfig, RetryCondition, RetryResult, RetryPolicy, RetryExecutor,
};
pub use monitoring::{
    ErrorStats, ErrorReport, ErrorDetail, ErrorTrend, TrendType,
    ErrorMonitor, MonitorConfig, ErrorThresholds, ErrorReportGenerator,
    DefaultReportGenerator
};
pub use lock_safety::{
    LockError, safe_lock, try_lock, safe_read, safe_write, try_read, try_write,
    ScopedLock
};

/// 错误严重级别
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
    pub fn from_str(s: &str) -> Option<Self> {
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

/// 错误分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// 渲染相关错误
    Render,
    /// 物理相关错误
    Physics,
    /// 音频相关错误
    Audio,
    /// 资源相关错误
    Resource,
    /// 输入相关错误
    Input,
    /// 系统相关错误
    System,
    /// 网络相关错误
    Network,
    /// 脚本相关错误
    Script,
    /// 平台相关错误
    Platform,
    /// 未知错误类型
    Unknown,
}

impl ErrorCategory {
    /// 获取分类的字符串表示
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

/// 统一的结果类型
pub type EngineResult<T> = Result<T, EngineError>;
pub type RenderResult<T> = Result<T, RenderError>;
pub type PhysicsResult<T> = Result<T, PhysicsError>;
pub type AudioResult<T> = Result<T, AudioError>;
pub type ResourceResult<T> = Result<T, ResourceError>;
pub type InputResult<T> = Result<T, InputError>;
pub type SystemResult<T> = Result<T, SystemError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        assert_eq!(ErrorSeverity::Info.as_str(), "INFO");
        assert_eq!(ErrorSeverity::Fatal.as_str(), "FATAL");
        assert!(ErrorSeverity::Critical > ErrorSeverity::Error);
        
        assert_eq!(ErrorSeverity::from_str("warning"), Some(ErrorSeverity::Warning));
        assert_eq!(ErrorSeverity::from_str("invalid"), None);
    }

    #[test]
    fn test_error_category() {
        assert_eq!(ErrorCategory::Render.as_str(), "RENDER");
        assert_eq!(ErrorCategory::Physics.as_str(), "PHYSICS");
        
        let category = ErrorCategory::Render;
        assert_eq!(category, ErrorCategory::Render);
    }
}