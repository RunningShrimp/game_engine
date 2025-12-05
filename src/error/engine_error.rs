//! 引擎核心错误类型
//!
//! 提供统一的错误处理机制，支持错误链、上下文传播和错误分类。

use crate::error::{ErrorCategory, ErrorSeverity};
use std::fmt;
use std::backtrace::Backtrace;
use thiserror::Error;

/// 引擎核心错误类型
///
/// 这是所有引擎错误的统一入口点，包含了所有子系统的错误类型。
/// 支持错误链、上下文信息和错误严重级别分类。
#[derive(Error, Debug, Clone)]
pub enum EngineError {
    /// 渲染系统错误
    #[error("Render error: {0}")]
    Render(#[from] RenderError),

    /// 物理系统错误
    #[error("Physics error: {0}")]
    Physics(#[from] PhysicsError),

    /// 音频系统错误
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    /// 资源管理错误
    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),

    /// 输入系统错误
    #[error("Input error: {0}")]
    Input(#[from] InputError),

    /// 系统级错误
    #[error("System error: {0}")]
    System(#[from] SystemError),

    /// 通用错误 - 用于不属于特定子系统的错误
    #[error("General error: {message}")]
    General {
        /// 错误消息
        message: String,
        /// 错误上下文
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        /// 错误严重级别
        severity: ErrorSeverity,
        /// 错误发生位置
        location: Option<String>,
        /// 错误回溯
        backtrace: Option<Backtrace>,
    },

    /// 多个错误聚合 - 用于批量操作中的多个错误
    #[error("Multiple errors occurred: {count} errors")]
    Multiple {
        /// 错误数量
        count: usize,
        /// 错误列表
        errors: Vec<EngineError>,
        /// 主要错误（第一个错误）
        #[source]
        primary: Option<Box<EngineError>>,
    },

    /// 错误链 - 用于包装其他错误并添加上下文
    #[error("Error chain: {context}")]
    Chain {
        /// 上下文信息
        context: String,
        /// 原始错误
        #[source]
        source: Box<EngineError>,
        /// 额外的上下文数据
        metadata: std::collections::HashMap<String, String>,
    },
}

impl EngineError {
    /// 创建通用错误
    pub fn general(message: impl Into<String>) -> Self {
        Self::General {
            message: message.into(),
            source: None,
            severity: ErrorSeverity::Error,
            location: None,
            backtrace: Some(Backtrace::capture()),
        }
    }

    /// 创建带有严重级别的通用错误
    pub fn general_with_severity(
        message: impl Into<String>,
        severity: ErrorSeverity,
    ) -> Self {
        Self::General {
            message: message.into(),
            source: None,
            severity,
            location: None,
            backtrace: Some(Backtrace::capture()),
        }
    }

    /// 创建带有源错误的通用错误
    pub fn general_with_source(
        message: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::General {
            message: message.into(),
            source: Some(source.into()),
            severity: ErrorSeverity::Error,
            location: None,
            backtrace: Some(Backtrace::capture()),
        }
    }

    /// 添加上下文信息到错误
    pub fn with_context(self, context: impl Into<String>) -> Self {
        Self::Chain {
            context: context.into(),
            source: Box::new(self),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 添加元数据到错误上下文
    pub fn with_metadata<K, V>(self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        match self {
            Self::Chain { context, source, mut metadata } => {
                metadata.insert(key.into(), value.into());
                Self::Chain { context, source, metadata }
            }
            other => Self::Chain {
                context: String::new(),
                source: Box::new(other),
                metadata: {
                    let mut map = std::collections::HashMap::new();
                    map.insert(key.into(), value.into());
                    map
                },
            },
        }
    }

    /// 获取错误的严重级别
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Render(err) => err.severity(),
            Self::Physics(err) => err.severity(),
            Self::Audio(err) => err.severity(),
            Self::Resource(err) => err.severity(),
            Self::Input(err) => err.severity(),
            Self::System(err) => err.severity(),
            Self::General { severity, .. } => *severity,
            Self::Multiple { errors, .. } => {
                // 返回最严重的错误级别
                errors
                    .iter()
                    .map(|e| e.severity())
                    .max()
                    .unwrap_or(ErrorSeverity::Error)
            }
            Self::Chain { source, .. } => source.severity(),
        }
    }

    /// 获取错误的分类
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Render(_) => ErrorCategory::Render,
            Self::Physics(_) => ErrorCategory::Physics,
            Self::Audio(_) => ErrorCategory::Audio,
            Self::Resource(_) => ErrorCategory::Resource,
            Self::Input(_) => ErrorCategory::Input,
            Self::System(_) => ErrorCategory::System,
            Self::General { .. } => ErrorCategory::Unknown,
            Self::Multiple { .. } => ErrorCategory::Unknown,
            Self::Chain { source, .. } => source.category(),
        }
    }

    /// 检查错误是否可恢复
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Render(err) => err.is_recoverable(),
            Self::Physics(err) => err.is_recoverable(),
            Self::Audio(err) => err.is_recoverable(),
            Self::Resource(err) => err.is_recoverable(),
            Self::Input(err) => err.is_recoverable(),
            Self::System(err) => err.is_recoverable(),
            Self::General { severity, .. } => *severity < ErrorSeverity::Critical,
            Self::Multiple { errors, .. } => {
                // 如果所有错误都可恢复，则整体可恢复
                errors.iter().all(|e| e.is_recoverable())
            }
            Self::Chain { source, .. } => source.is_recoverable(),
        }
    }

    /// 获取错误的根本原因
    pub fn root_cause(&self) -> &EngineError {
        match self {
            Self::Chain { source, .. } => source.root_cause(),
            _ => self,
        }
    }

    /// 收集所有错误链中的错误
    pub fn collect_chain(&self) -> Vec<&EngineError> {
        let mut chain = Vec::new();
        let mut current = self;
        chain.push(current);
        
        while let Self::Chain { source, .. } = current {
            chain.push(source);
            current = source;
        }
        
        chain
    }

    /// 创建多错误聚合
    pub fn multiple(errors: Vec<EngineError>) -> Self {
        let count = errors.len();
        if count == 0 {
            return Self::general("No errors provided for multiple error aggregation");
        }
        if count == 1 {
            return errors.into_iter().next().unwrap();
        }

        let primary = errors.first().map(|e| Box::new(e.clone()));
        Self::Multiple { count, errors, primary }
    }

    /// 检查是否为特定类型的错误
    pub fn is<T>(&self) -> bool
    where
        T: std::any::Any,
    {
        match self {
            Self::Render(_) => std::any::TypeId::of::<T>() == std::any::TypeId::of::<RenderError>(),
            Self::Physics(_) => std::any::TypeId::of::<T>() == std::any::TypeId::of::<PhysicsError>(),
            Self::Audio(_) => std::any::TypeId::of::<T>() == std::any::TypeId::of::<AudioError>(),
            Self::Resource(_) => std::any::TypeId::of::<T>() == std::any::TypeId::of::<ResourceError>(),
            Self::Input(_) => std::any::TypeId::of::<T>() == std::any::TypeId::of::<InputError>(),
            Self::System(_) => std::any::TypeId::of::<T>() == std::any::TypeId::of::<SystemError>(),
            _ => false,
        }
    }

    /// 尝试向下转型为特定错误类型
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: std::any::Any,
    {
        match self {
            Self::Render(err) if std::any::TypeId::of::<T>() == std::any::TypeId::of::<RenderError>() => {
                Some(unsafe { &*(err as *const _ as *const T) })
            }
            Self::Physics(err) if std::any::TypeId::of::<T>() == std::any::TypeId::of::<PhysicsError>() => {
                Some(unsafe { &*(err as *const _ as *const T) })
            }
            Self::Audio(err) if std::any::TypeId::of::<T>() == std::any::TypeId::of::<AudioError>() => {
                Some(unsafe { &*(err as *const _ as *const T) })
            }
            Self::Resource(err) if std::any::TypeId::of::<T>() == std::any::TypeId::of::<ResourceError>() => {
                Some(unsafe { &*(err as *const _ as *const T) })
            }
            Self::Input(err) if std::any::TypeId::of::<T>() == std::any::TypeId::of::<InputError>() => {
                Some(unsafe { &*(err as *const _ as *const T) })
            }
            Self::System(err) if std::any::TypeId::of::<T>() == std::any::TypeId::of::<SystemError>() => {
                Some(unsafe { &*(err as *const _ as *const T) })
            }
            _ => None,
        }
    }
}

// 为了让EngineError可以使用，我们需要前向声明这些类型
// 这些类型的实际实现在各自的模块中
#[derive(Error, Debug, Clone)]
pub enum RenderError {
    #[error("Unknown render error")]
    Unknown,
}

#[derive(Error, Debug, Clone)]
pub enum PhysicsError {
    #[error("Unknown physics error")]
    Unknown,
}

#[derive(Error, Debug, Clone)]
pub enum AudioError {
    #[error("Unknown audio error")]
    Unknown,
}

#[derive(Error, Debug, Clone)]
pub enum ResourceError {
    #[error("Unknown resource error")]
    Unknown,
}

#[derive(Error, Debug, Clone)]
pub enum InputError {
    #[error("Unknown input error")]
    Unknown,
}

#[derive(Error, Debug, Clone)]
pub enum SystemError {
    #[error("Unknown system error")]
    Unknown,
}

// 为这些类型实现基本方法
impl RenderError {
    pub fn severity(&self) -> ErrorSeverity { ErrorSeverity::Error }
    pub fn is_recoverable(&self) -> bool { true }
}

impl PhysicsError {
    pub fn severity(&self) -> ErrorSeverity { ErrorSeverity::Error }
    pub fn is_recoverable(&self) -> bool { true }
}

impl AudioError {
    pub fn severity(&self) -> ErrorSeverity { ErrorSeverity::Error }
    pub fn is_recoverable(&self) -> bool { true }
}

impl ResourceError {
    pub fn severity(&self) -> ErrorSeverity { ErrorSeverity::Error }
    pub fn is_recoverable(&self) -> bool { true }
}

impl InputError {
    pub fn severity(&self) -> ErrorSeverity { ErrorSeverity::Error }
    pub fn is_recoverable(&self) -> bool { true }
}

impl SystemError {
    pub fn severity(&self) -> ErrorSeverity { ErrorSeverity::Error }
    pub fn is_recoverable(&self) -> bool { true }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_error_creation() {
        let err = EngineError::general("Test error");
        assert_eq!(err.severity(), ErrorSeverity::Error);
        assert_eq!(err.category(), ErrorCategory::Unknown);
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_engine_error_with_context() {
        let err = EngineError::general("Base error")
            .with_context("Additional context")
            .with_metadata("key", "value");

        assert!(matches!(err, EngineError::Chain { .. }));
        let chain = err.collect_chain();
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_engine_error_multiple() {
        let errors = vec![
            EngineError::general("Error 1"),
            EngineError::general("Error 2"),
        ];
        let multi_err = EngineError::multiple(errors);
        
        assert!(matches!(multi_err, EngineError::Multiple { count: 2, .. }));
    }

    #[test]
    fn test_error_severity() {
        let critical_err = EngineError::general_with_severity("Critical", ErrorSeverity::Critical);
        assert_eq!(critical_err.severity(), ErrorSeverity::Critical);
        assert!(!critical_err.is_recoverable());
    }

    #[test]
    fn test_root_cause() {
        let root = EngineError::general("Root cause");
        let chained = root.with_context("Intermediate context");
        let final_err = chained.with_context("Final context");

        assert_eq!(final_err.root_cause().to_string(), "General error: Root cause");
    }
}