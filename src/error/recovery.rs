//! 错误恢复机制
//!
//! 提供统一的错误恢复策略，支持优雅降级、重试和补偿操作。

use crate::error::{EngineError, ErrorSeverity, ErrorCategory};
use std::time::Duration;
use std::collections::HashMap;

/// 错误恢复结果
#[derive(Debug, Clone)]
pub enum RecoveryResult<T> {
    /// 恢复成功，返回结果
    Success(T),
    /// 恢复失败，返回原始错误
    Failed(EngineError),
    /// 恢复部分成功，返回降级结果
    Degraded(T, RecoveryInfo),
    /// 恢复跳过，使用默认值
    Skipped(RecoveryInfo),
    /// 需要重试
    Retry(RetryInfo),
}

/// 恢复信息
#[derive(Debug, Clone)]
pub struct RecoveryInfo {
    /// 恢复策略
    pub strategy: RecoveryStrategy,
    /// 恢复描述
    pub description: String,
    /// 恢复时间
    pub duration: Duration,
    /// 恢复元数据
    pub metadata: HashMap<String, String>,
}

/// 重试信息
#[derive(Debug, Clone)]
pub struct RetryInfo {
    /// 重试次数
    pub attempt: u32,
    /// 最大重试次数
    pub max_attempts: u32,
    /// 下次重试延迟
    pub next_delay: Duration,
    /// 重试原因
    pub reason: String,
}

/// 错误恢复策略
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// 重试操作
    Retry {
        /// 最大重试次数
        max_attempts: u32,
        /// 基础延迟（毫秒）
        base_delay_ms: u64,
        /// 退避倍数
        backoff_multiplier: f64,
        /// 最大延迟（毫秒）
        max_delay_ms: u64,
    },
    /// 使用默认值
    UseDefault {
        /// 默认值描述
        default_description: String,
        /// 是否记录警告
        log_warning: bool,
    },
    /// 跳过操作
    Skip {
        /// 跳过原因
        reason: String,
        /// 是否记录警告
        log_warning: bool,
    },
    /// 记录错误并继续
    LogAndContinue {
        /// 日志级别
        log_level: ErrorSeverity,
        /// 额外上下文
        context: String,
    },
    /// 优雅降级
    GracefulDegradation {
        /// 降级级别
        degradation_level: u32,
        /// 降级描述
        description: String,
        /// 功能替代方案
        fallback: String,
    },
    /// 快速失败
    FailFast {
        /// 失败原因
        reason: String,
        /// 是否记录错误
        log_error: bool,
    },
}

/// 错误恢复上下文
#[derive(Debug, Clone)]
pub struct RecoveryContext {
    /// 操作名称
    pub operation: String,
    /// 错误历史
    pub error_history: Vec<EngineError>,
    /// 恢复尝试次数
    pub recovery_attempts: u32,
    /// 上下文数据
    pub context_data: HashMap<String, String>,
    /// 开始时间
    pub start_time: std::time::Instant,
}

/// 错误恢复器
pub trait ErrorRecovery {
    /// 尝试恢复错误
    fn recover(&self, error: &EngineError, context: &RecoveryContext) -> RecoveryResult<()>;
    
    /// 检查是否可以处理该错误
    fn can_handle(&self, error: &EngineError) -> bool;
    
    /// 获取恢复器名称
    fn name(&self) -> &str;
}

/// 默认错误恢复器
pub struct DefaultErrorRecovery;

impl DefaultErrorRecovery {
    pub fn new() -> Self {
        Self
    }
}

impl ErrorRecovery for DefaultErrorRecovery {
    fn recover(&self, error: &EngineError, context: &RecoveryContext) -> RecoveryResult<()> {
        match error.severity() {
            ErrorSeverity::Info => {
                // 信息级别错误，直接跳过
                RecoveryResult::Skipped(RecoveryInfo {
                    strategy: RecoveryStrategy::Skip {
                        reason: "Info level error, skipping".to_string(),
                        log_warning: false,
                    },
                    description: "Skipping info level error".to_string(),
                    duration: context.start_time.elapsed(),
                    metadata: HashMap::new(),
                })
            }
            ErrorSeverity::Warning => {
                // 警告级别错误，记录并继续
                RecoveryResult::LogAndContinue(RecoveryInfo {
                    strategy: RecoveryStrategy::LogAndContinue {
                        log_level: ErrorSeverity::Warning,
                        context: format!("Warning in operation: {}", context.operation),
                    },
                    description: "Logging warning and continuing".to_string(),
                    duration: context.start_time.elapsed(),
                    metadata: HashMap::new(),
                })
            }
            ErrorSeverity::Error => {
                // 错误级别，尝试重试
                if context.recovery_attempts < 3 {
                    RecoveryResult::Retry(RetryInfo {
                        attempt: context.recovery_attempts,
                        max_attempts: 3,
                        next_delay: Duration::from_millis(100 * (2_u64.pow(context.recovery_attempts))),
                        reason: "Error occurred, retrying".to_string(),
                    })
                } else {
                    // 超过重试次数，降级处理
                    RecoveryResult::Degraded((), RecoveryInfo {
                        strategy: RecoveryStrategy::GracefulDegradation {
                            degradation_level: 1,
                            description: "Failed after retries, using fallback".to_string(),
                            fallback: "Default behavior".to_string(),
                        },
                        description: "Graceful degradation after failed retries".to_string(),
                        duration: context.start_time.elapsed(),
                        metadata: HashMap::new(),
                    })
                }
            }
            ErrorSeverity::Critical => {
                // 严重错误，记录并快速失败
                RecoveryResult::Failed(error.clone())
            }
            ErrorSeverity::Fatal => {
                // 致命错误，直接失败
                RecoveryResult::Failed(error.clone())
            }
        }
    }

    fn can_handle(&self, _error: &EngineError) -> bool {
        true // 默认恢复器处理所有错误
    }

    fn name(&self) -> &str {
        "DefaultErrorRecovery"
    }
}

/// 渲染错误恢复器
pub struct RenderErrorRecovery;

impl RenderErrorRecovery {
    pub fn new() -> Self {
        Self
    }
}

impl ErrorRecovery for RenderErrorRecovery {
    fn recover(&self, error: &EngineError, context: &RecoveryContext) -> RecoveryResult<()> {
        if let Some(render_err) = error.downcast_ref::<crate::error::RenderError>() {
            match render_err {
                crate::error::RenderError::OutOfMemory { .. } => {
                    // GPU内存不足，尝试降级渲染质量
                    RecoveryResult::Degraded((), RecoveryInfo {
                        strategy: RecoveryStrategy::GracefulDegradation {
                            degradation_level: 1,
                            description: "GPU memory low, reducing render quality".to_string(),
                            fallback: "Low quality rendering".to_string(),
                        },
                        description: "Render quality degradation due to memory constraints".to_string(),
                        duration: context.start_time.elapsed(),
                        metadata: HashMap::new(),
                    })
                }
                crate::error::RenderError::DeviceCreation { .. } => {
                    // 设备创建失败，尝试软件渲染
                    RecoveryResult::Degraded((), RecoveryInfo {
                        strategy: RecoveryStrategy::GracefulDegradation {
                            degradation_level: 2,
                            description: "Hardware rendering failed, falling back to software".to_string(),
                            fallback: "Software rendering".to_string(),
                        },
                        description: "Software rendering fallback".to_string(),
                        duration: context.start_time.elapsed(),
                        metadata: HashMap::new(),
                    })
                }
                crate::error::RenderError::ShaderCompilation { .. } => {
                    // 着色器编译失败，使用默认着色器
                    RecoveryResult::Degraded((), RecoveryInfo {
                        strategy: RecoveryStrategy::UseDefault {
                            default_description: "Using default shader".to_string(),
                            log_warning: true,
                        },
                        description: "Shader compilation failed, using default".to_string(),
                        duration: context.start_time.elapsed(),
                        metadata: HashMap::new(),
                    })
                }
                _ => {
                    // 其他渲染错误，使用默认恢复策略
                    let default_recovery = DefaultErrorRecovery::new();
                    default_recovery.recover(error, context)
                }
            }
        } else {
            // 非渲染错误，使用默认恢复策略
            let default_recovery = DefaultErrorRecovery::new();
            default_recovery.recover(error, context)
        }
    }

    fn can_handle(&self, error: &EngineError) -> bool {
        error.category() == ErrorCategory::Render
    }

    fn name(&self) -> &str {
        "RenderErrorRecovery"
    }
}

/// 音频错误恢复器
pub struct AudioErrorRecovery;

impl AudioErrorRecovery {
    pub fn new() -> Self {
        Self
    }
}

impl ErrorRecovery for AudioErrorRecovery {
    fn recover(&self, error: &EngineError, context: &RecoveryContext) -> RecoveryResult<()> {
        if let Some(audio_err) = error.downcast_ref::<crate::error::AudioError>() {
            match audio_err {
                crate::error::AudioError::DeviceInitialization { .. } => {
                    // 音频设备初始化失败，静音处理
                    RecoveryResult::Degraded((), RecoveryInfo {
                        strategy: RecoveryStrategy::GracefulDegradation {
                            degradation_level: 1,
                            description: "Audio device failed, muting audio".to_string(),
                            fallback: "Silent audio".to_string(),
                        },
                        description: "Audio muted due to device failure".to_string(),
                        duration: context.start_time.elapsed(),
                        metadata: HashMap::new(),
                    })
                }
                crate::error::AudioError::Playback { .. } => {
                    // 播放失败，跳过当前音频
                    RecoveryResult::Skipped(RecoveryInfo {
                        strategy: RecoveryStrategy::Skip {
                            reason: "Audio playback failed, skipping".to_string(),
                            log_warning: true,
                        },
                        description: "Skipping audio playback".to_string(),
                        duration: context.start_time.elapsed(),
                        metadata: HashMap::new(),
                    })
                }
                crate::error::AudioError::InvalidVolume { .. } => {
                    // 无效音量，使用默认音量
                    RecoveryResult::Degraded((), RecoveryInfo {
                        strategy: RecoveryStrategy::UseDefault {
                            default_description: "Using default volume".to_string(),
                            log_warning: true,
                        },
                        description: "Volume clamped to valid range".to_string(),
                        duration: context.start_time.elapsed(),
                        metadata: HashMap::new(),
                    })
                }
                _ => {
                    // 其他音频错误，使用默认恢复策略
                    let default_recovery = DefaultErrorRecovery::new();
                    default_recovery.recover(error, context)
                }
            }
        } else {
            // 非音频错误，使用默认恢复策略
            let default_recovery = DefaultErrorRecovery::new();
            default_recovery.recover(error, context)
        }
    }

    fn can_handle(&self, error: &EngineError) -> bool {
        error.category() == ErrorCategory::Audio
    }

    fn name(&self) -> &str {
        "AudioErrorRecovery"
    }
}

/// 物理错误恢复器
pub struct PhysicsErrorRecovery;

impl PhysicsErrorRecovery {
    pub fn new() -> Self {
        Self
    }
}

impl ErrorRecovery for PhysicsErrorRecovery {
    fn recover(&self, error: &EngineError, context: &RecoveryContext) -> RecoveryResult<()> {
        if let Some(physics_err) = error.downcast_ref::<crate::error::PhysicsError>() {
            match physics_err {
                crate::error::PhysicsError::WorldNotInitialized { .. } => {
                    // 物理世界未初始化，跳过物理模拟
                    RecoveryResult::Skipped(RecoveryInfo {
                        strategy: RecoveryStrategy::Skip {
                            reason: "Physics world not initialized".to_string(),
                            log_warning: true,
                        },
                        description: "Skipping physics simulation".to_string(),
                        duration: context.start_time.elapsed(),
                        metadata: HashMap::new(),
                    })
                }
                crate::error::PhysicsError::Simulation { .. } => {
                    // 物理模拟错误，简化物理模拟
                    RecoveryResult::Degraded((), RecoveryInfo {
                        strategy: RecoveryStrategy::GracefulDegradation {
                            degradation_level: 1,
                            description: "Physics simulation failed, using simplified physics".to_string(),
                            fallback: "Simplified physics".to_string(),
                        },
                        description: "Simplified physics due to simulation error".to_string(),
                        duration: context.start_time.elapsed(),
                        metadata: HashMap::new(),
                    })
                }
                _ => {
                    // 其他物理错误，使用默认恢复策略
                    let default_recovery = DefaultErrorRecovery::new();
                    default_recovery.recover(error, context)
                }
            }
        } else {
            // 非物理错误，使用默认恢复策略
            let default_recovery = DefaultErrorRecovery::new();
            default_recovery.recover(error, context)
        }
    }

    fn can_handle(&self, error: &EngineError) -> bool {
        error.category() == ErrorCategory::Physics
    }

    fn name(&self) -> &str {
        "PhysicsErrorRecovery"
    }
}

/// 资源错误恢复器
pub struct ResourceErrorRecovery;

impl ResourceErrorRecovery {
    pub fn new() -> Self {
        Self
    }
}

impl ErrorRecovery for ResourceErrorRecovery {
    fn recover(&self, error: &EngineError, context: &RecoveryContext) -> RecoveryResult<()> {
        if let Some(resource_err) = error.downcast_ref::<crate::error::ResourceError>() {
            match resource_err {
                crate::error::ResourceError::NotFound { .. } => {
                    // 资源未找到，使用默认资源
                    RecoveryResult::Degraded((), RecoveryInfo {
                        strategy: RecoveryStrategy::UseDefault {
                            default_description: "Using default resource".to_string(),
                            log_warning: true,
                        },
                        description: "Default resource used due to missing asset".to_string(),
                        duration: context.start_time.elapsed(),
                        metadata: HashMap::new(),
                    })
                }
                crate::error::ResourceError::LoadFailed { .. } => {
                    // 资源加载失败，重试
                    if context.recovery_attempts < 3 {
                        RecoveryResult::Retry(RetryInfo {
                            attempt: context.recovery_attempts,
                            max_attempts: 3,
                            next_delay: Duration::from_millis(500 * (2_u64.pow(context.recovery_attempts))),
                            reason: "Resource load failed, retrying".to_string(),
                        })
                    } else {
                        // 超过重试次数，使用占位符资源
                        RecoveryResult::Degraded((), RecoveryInfo {
                            strategy: RecoveryStrategy::GracefulDegradation {
                                degradation_level: 1,
                                description: "Resource load failed, using placeholder".to_string(),
                                fallback: "Placeholder resource".to_string(),
                            },
                            description: "Placeholder resource due to load failure".to_string(),
                            duration: context.start_time.elapsed(),
                            metadata: HashMap::new(),
                        })
                    }
                }
                _ => {
                    // 其他资源错误，使用默认恢复策略
                    let default_recovery = DefaultErrorRecovery::new();
                    default_recovery.recover(error, context)
                }
            }
        } else {
            // 非资源错误，使用默认恢复策略
            let default_recovery = DefaultErrorRecovery::new();
            default_recovery.recover(error, context)
        }
    }

    fn can_handle(&self, error: &EngineError) -> bool {
        error.category() == ErrorCategory::Resource
    }

    fn name(&self) -> &str {
        "ResourceErrorRecovery"
    }
}

/// 错误恢复管理器
pub struct RecoveryManager {
    /// 恢复器列表
    recoverers: Vec<Box<dyn ErrorRecovery>>,
    /// 恢复历史
    recovery_history: Vec<RecoveryInfo>,
}

impl RecoveryManager {
    /// 创建新的恢复管理器
    pub fn new() -> Self {
        let mut recoverers: Vec<Box<dyn ErrorRecovery>> = Vec::new();
        
        // 添加默认恢复器（最后处理）
        recoverers.push(Box::new(DefaultErrorRecovery::new()));
        
        // 添加特定恢复器（按优先级顺序）
        recoverers.insert(0, Box::new(RenderErrorRecovery::new()));
        recoverers.insert(0, Box::new(AudioErrorRecovery::new()));
        recoverers.insert(0, Box::new(PhysicsErrorRecovery::new()));
        recoverers.insert(0, Box::new(ResourceErrorRecovery::new()));
        
        Self {
            recoverers,
            recovery_history: Vec::new(),
        }
    }

    /// 尝试恢复错误
    pub fn recover(&mut self, error: EngineError, operation: &str) -> RecoveryResult<()> {
        let context = RecoveryContext {
            operation: operation.to_string(),
            error_history: Vec::new(),
            recovery_attempts: 0,
            context_data: HashMap::new(),
            start_time: std::time::Instant::now(),
        };

        self.recover_with_context(error, &context)
    }

    /// 使用上下文恢复错误
    pub fn recover_with_context(&mut self, error: EngineError, context: &RecoveryContext) -> RecoveryResult<()> {
        // 查找合适的恢复器
        for recoverer in &self.recoverers {
            if recoverer.can_handle(&error) {
                let mut updated_context = context.clone();
                updated_context.recovery_attempts += 1;
                
                let result = recoverer.recover(&error, &updated_context);
                
                // 记录恢复历史
                if let RecoveryResult::Degraded(_, ref info) 
                | RecoveryResult::Skipped(ref info) 
                | RecoveryResult::LogAndContinue(ref info) = &result {
                    self.recovery_history.push(info.clone());
                }
                
                return result;
            }
        }

        // 没有找到合适的恢复器，直接失败
        RecoveryResult::Failed(error)
    }

    /// 获取恢复历史
    pub fn recovery_history(&self) -> &[RecoveryInfo] {
        &self.recovery_history
    }

    /// 清除恢复历史
    pub fn clear_history(&mut self) {
        self.recovery_history.clear();
    }

    /// 添加自定义恢复器
    pub fn add_recorder(&mut self, recoverer: Box<dyn ErrorRecovery>) {
        self.recoverers.insert(0, recoverer);
    }

    /// 移除恢复器
    pub fn remove_recorder(&mut self, name: &str) -> bool {
        let initial_len = self.recoverers.len();
        self.recoverers.retain(|r| r.name() != name);
        self.recoverers.len() < initial_len
    }
}

/// 便捷的错误恢复函数
pub fn recover_with_default_strategy<T>(
    result: Result<T, EngineError>,
    operation: &str,
) -> RecoveryResult<T> {
    match result {
        Ok(value) => RecoveryResult::Success(value),
        Err(error) => {
            let mut manager = RecoveryManager::new();
            match manager.recover(error, operation) {
                RecoveryResult::Success(()) => RecoveryResult::Failed(error), // 原操作失败
                RecoveryResult::Degraded(_, _) => RecoveryResult::Failed(error), // 降级但仍失败
                RecoveryResult::Skipped(_) => RecoveryResult::Failed(error), // 跳过但仍失败
                RecoveryResult::LogAndContinue(_) => RecoveryResult::Failed(error), // 记录但仍失败
                RecoveryResult::Retry(_) => RecoveryResult::Failed(error), // 需要重试但仍失败
                RecoveryResult::Failed(e) => RecoveryResult::Failed(e), // 恢复失败
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{RenderError, AudioError, PhysicsError, ResourceError};

    #[test]
    fn test_default_recovery() {
        let recovery = DefaultErrorRecovery::new();
        let context = RecoveryContext {
            operation: "test_operation".to_string(),
            error_history: Vec::new(),
            recovery_attempts: 0,
            context_data: HashMap::new(),
            start_time: std::time::Instant::now(),
        };

        let error = EngineError::general("Test error");
        let result = recovery.recover(&error, &context);
        
        assert!(matches!(result, RecoveryResult::Retry(_)));
    }

    #[test]
    fn test_render_recovery() {
        let recovery = RenderErrorRecovery::new();
        let context = RecoveryContext {
            operation: "render_operation".to_string(),
            error_history: Vec::new(),
            recovery_attempts: 0,
            context_data: HashMap::new(),
            start_time: std::time::Instant::now(),
        };

        let error = EngineError::Render(RenderError::out_of_memory("GPU memory full"));
        let result = recovery.recover(&error, &context);
        
        assert!(matches!(result, RecoveryResult::Degraded(_, _)));
    }

    #[test]
    fn test_audio_recovery() {
        let recovery = AudioErrorRecovery::new();
        let context = RecoveryContext {
            operation: "audio_operation".to_string(),
            error_history: Vec::new(),
            recovery_attempts: 0,
            context_data: HashMap::new(),
            start_time: std::time::Instant::now(),
        };

        let error = EngineError::Audio(AudioError::invalid_volume(1.5));
        let result = recovery.recover(&error, &context);
        
        assert!(matches!(result, RecoveryResult::Degraded(_, _)));
    }

    #[test]
    fn test_physics_recovery() {
        let recovery = PhysicsErrorRecovery::new();
        let context = RecoveryContext {
            operation: "physics_operation".to_string(),
            error_history: Vec::new(),
            recovery_attempts: 0,
            context_data: HashMap::new(),
            start_time: std::time::Instant::now(),
        };

        let error = EngineError::Physics(PhysicsError::world_not_initialized());
        let result = recovery.recover(&error, &context);
        
        assert!(matches!(result, RecoveryResult::Skipped(_)));
    }

    #[test]
    fn test_resource_recovery() {
        let recovery = ResourceErrorRecovery::new();
        let context = RecoveryContext {
            operation: "resource_operation".to_string(),
            error_history: Vec::new(),
            recovery_attempts: 0,
            context_data: HashMap::new(),
            start_time: std::time::Instant::now(),
        };

        let error = EngineError::Resource(ResourceError::not_found("texture.png"));
        let result = recovery.recover(&error, &context);
        
        assert!(matches!(result, RecoveryResult::Degraded(_, _)));
    }

    #[test]
    fn test_recovery_manager() {
        let mut manager = RecoveryManager::new();
        
        let error = EngineError::Render(RenderError::out_of_memory("GPU memory full"));
        let result = manager.recover(error, "render_test");
        
        assert!(matches!(result, RecoveryResult::Degraded(_, _)));
        assert_eq!(manager.recovery_history().len(), 1);
    }

    #[test]
    fn test_recover_with_default_strategy() {
        let result: Result<(), EngineError> = Err(EngineError::general("Test error"));
        let recovery_result = recover_with_default_strategy(result, "test_operation");
        
        assert!(matches!(recovery_result, RecoveryResult::Failed(_)));
    }
}