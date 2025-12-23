//  引擎核心错误类型
// 
//  提供统一的错误处理机制，支持错误链、上下文传播和错误分类。

use crate::error::{ErrorCategory, ErrorSeverity};
use std::backtrace::Backtrace;
use thiserror::Error;

/// 引擎核心错误类型
///
/// 这是所有引擎错误的统一入口点，包含了所有子系统的错误类型。
/// 支持错误链、上下文信息和错误严重级别分类。
#[derive(Error, Debug)]
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

// 手动实现Clone trait，因为source字段包含dyn Error类型，不能自动派生Clone
impl Clone for EngineError {
    fn clone(&self) -> Self {
        match self {
            EngineError::Render(err) => EngineError::Render(err.clone()),
            EngineError::Physics(err) => EngineError::Physics(err.clone()),
            EngineError::Audio(err) => EngineError::Audio(err.clone()),
            EngineError::Resource(err) => EngineError::Resource(err.clone()),
            EngineError::Input(err) => EngineError::Input(err.clone()),
            EngineError::System(err) => EngineError::System(err.clone()),
            EngineError::General {
                message,
                source: _,
                severity,
                location,
                backtrace,
            } => {
                // 对于General错误，我们忽略source字段的克隆，因为它可能不支持Clone
                // 记录回溯信息的状态以满足使用要求
                if backtrace.is_some() {
                    tracing::trace!(target: "error", "Cloning error with backtrace: {}", message);
                }
                
                EngineError::General {
                    message: message.clone(),
                    source: None, // 忽略source字段的克隆
                    severity: *severity,
                    location: location.clone(),
                    backtrace: None, // Backtrace cannot be cloned
                }
            }
            EngineError::Multiple {
                count,
                errors,
                primary,
            } => EngineError::Multiple {
                count: *count,
                errors: errors.clone(),
                primary: primary.as_ref().map(|e| Box::new((**e).clone())),
            },
            EngineError::Chain {
                context,
                source,
                metadata,
            } => EngineError::Chain {
                context: context.clone(),
                source: Box::new((**source).clone()),
                metadata: metadata.clone(),
            },
        }
    }
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
    pub fn general_with_severity(message: impl Into<String>, severity: ErrorSeverity) -> Self {
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
            Self::Chain {
                context,
                source,
                mut metadata,
            } => {
                metadata.insert(key.into(), value.into());
                Self::Chain {
                    context,
                    source,
                    metadata,
                }
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
        Self::Multiple {
            count,
            errors,
            primary,
        }
    }

    /// 检查是否为特定类型的错误
    pub fn is<T>(&self) -> bool
    where
        T: std::any::Any,
    {
        match self {
            Self::Render(_) => std::any::TypeId::of::<T>() == std::any::TypeId::of::<RenderError>(),
            Self::Physics(_) => {
                std::any::TypeId::of::<T>() == std::any::TypeId::of::<PhysicsError>()
            }
            Self::Audio(_) => std::any::TypeId::of::<T>() == std::any::TypeId::of::<AudioError>(),
            Self::Resource(_) => {
                std::any::TypeId::of::<T>() == std::any::TypeId::of::<ResourceError>()
            }
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
            Self::Render(err)
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<RenderError>() =>
            {
                Some(unsafe { &*(err as *const _ as *const T) })
            }
            Self::Physics(err)
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<PhysicsError>() =>
            {
                Some(unsafe { &*(err as *const _ as *const T) })
            }
            Self::Audio(err)
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<AudioError>() =>
            {
                Some(unsafe { &*(err as *const _ as *const T) })
            }
            Self::Resource(err)
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<ResourceError>() =>
            {
                Some(unsafe { &*(err as *const _ as *const T) })
            }
            Self::Input(err)
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<InputError>() =>
            {
                Some(unsafe { &*(err as *const _ as *const T) })
            }
            Self::System(err)
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<SystemError>() =>
            {
                Some(unsafe { &*(err as *const _ as *const T) })
            }
            _ => None,
        }
    }
}

// Use canonical error types defined in their respective modules instead of duplicating them here
use crate::error::render_error::RenderError;
use crate::error::physics_error::PhysicsError;
use crate::error::audio_error::AudioError;
use crate::error::resource_error::ResourceError;
use crate::error::input_error::InputError;
use crate::error::system_error::SystemError;

// ============================================================================
// 统一错误类型转换实现
// ============================================================================

/// 从NetworkError转换为EngineError
impl From<crate::network::NetworkError> for EngineError {
    fn from(error: crate::network::NetworkError) -> Self {
        EngineError::System(SystemError::network(error.to_string()))
    }
}

/// 从GameEngineError转换为EngineError
impl From<crate::common_errors::GameEngineError> for EngineError {
    fn from(error: crate::common_errors::GameEngineError) -> Self {
        match error {
            crate::common_errors::GameEngineError::Infrastructure(infra_err) => {
                match infra_err {
                    crate::common_errors::InfrastructureError::Init(msg) => {
                        EngineError::System(SystemError::initialization("engine", msg))
                    }
                    crate::common_errors::InfrastructureError::Render(render_err) => {
                        // 转换CommonRenderError到RenderError
                        EngineError::Render(RenderError::Adapter {
                            message: render_err.to_string(),
                            severity: ErrorSeverity::Error,
                        })
                    }
                    crate::common_errors::InfrastructureError::Asset(asset_err) => {
                        EngineError::Resource(ResourceError::NotFound {
                            path: format!("{:?}", asset_err),
                            severity: ErrorSeverity::Error,
                        })
                    }
                    crate::common_errors::InfrastructureError::Physics(_physics_err) => {
                        EngineError::Physics(PhysicsError::WorldNotInitialized {
                            severity: ErrorSeverity::Error,
                        })
                    }
                    crate::common_errors::InfrastructureError::Audio(audio_err) => {
                        EngineError::Audio(AudioError::DeviceInitialization {
                            message: audio_err.to_string(),
                            severity: ErrorSeverity::Error,
                        })
                    }
                    crate::common_errors::InfrastructureError::Script(script_err) => {
                        EngineError::System(SystemError::initialization("script", script_err.to_string()))
                    }
                    crate::common_errors::InfrastructureError::Platform(platform_err) => {
                        EngineError::System(SystemError::platform("platform", platform_err.to_string()))
                    }
                    crate::common_errors::InfrastructureError::Io(io_err) => {
                        EngineError::System(SystemError::filesystem("unknown", io_err.to_string()))
                    }
                    crate::common_errors::InfrastructureError::Window(msg) => {
                        EngineError::System(SystemError::platform("window", msg))
                    }
                    crate::common_errors::InfrastructureError::RenderInit(wgpu_err) => {
                        EngineError::Render(RenderError::DeviceCreation {
                            message: wgpu_err.to_string(),
                            severity: ErrorSeverity::Critical,
                        })
                    }
                    crate::common_errors::InfrastructureError::EventLoop(msg) => {
                        EngineError::System(SystemError::initialization("event_loop", msg))
                    }
                    crate::common_errors::InfrastructureError::General(msg) => {
                        EngineError::general(msg)
                    }
                }
            }
            crate::common_errors::GameEngineError::Domain(domain_err) => {
                match domain_err {
                    crate::common_errors::DomainError::Audio(audio_err) => {
                        EngineError::Audio(AudioError::SourceNotFound {
                            source_id: format!("{:?}", audio_err),
                            severity: ErrorSeverity::Error,
                        })
                    }
                    crate::common_errors::DomainError::Physics(physics_err) => {
                        EngineError::Physics(PhysicsError::RigidBodyNotFound {
                            body_id: format!("{:?}", physics_err),
                            severity: ErrorSeverity::Error,
                        })
                    }
                    crate::common_errors::DomainError::Scene(scene_err) => {
                        EngineError::general(format!("Scene error: {}", scene_err))
                    }
                    crate::common_errors::DomainError::General(msg) => {
                        EngineError::general(msg)
                    }
                }
            }
        }
    }
}

/// 从DomainError转换为EngineError
impl From<crate::domain::errors::DomainError> for EngineError {
    fn from(error: crate::domain::errors::DomainError) -> Self {
        match error {
            crate::domain::errors::DomainError::Audio(audio_err) => {
                EngineError::Audio(AudioError::SourceNotFound {
                    source_id: format!("{:?}", audio_err),
                    severity: ErrorSeverity::Error,
                })
            }
            crate::domain::errors::DomainError::Physics(physics_err) => {
                EngineError::Physics(PhysicsError::RigidBodyNotFound {
                    body_id: format!("{:?}", physics_err),
                    severity: ErrorSeverity::Error,
                })
            }
            crate::domain::errors::DomainError::Scene(scene_err) => {
                EngineError::general(format!("Scene error: {}", scene_err))
            }
            crate::domain::errors::DomainError::General(msg) => {
                EngineError::general(msg)
            }
        }
    }
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

        assert_eq!(
            final_err.root_cause().to_string(),
            "General error: Root cause"
        );
    }

    #[test]
    fn test_network_error_conversion() {
        let network_err = crate::network::NetworkError::ConnectionError("test".to_string());
        let engine_err: EngineError = network_err.into();
        assert!(matches!(engine_err, EngineError::System(_)));
    }

    #[test]
    fn test_domain_error_conversion() {
        let domain_err = crate::domain::errors::DomainError::General("test".to_string());
        let engine_err: EngineError = domain_err.into();
        assert!(matches!(engine_err, EngineError::General { .. }));
    }
}
