/// 硬件优化系统错误处理
/// 
/// 提供详细的错误类型和错误处理机制

use thiserror::Error;

/// 硬件优化系统错误类型
#[derive(Error, Debug, Clone)]
pub enum HardwareError {
    /// GPU检测失败
    #[error("GPU检测失败: {reason}。尝试的方法: {attempted_methods:?}")]
    GpuDetectionFailed {
        reason: String,
        attempted_methods: Vec<String>,
    },
    
    /// NPU检测失败
    #[error("NPU检测失败: {reason}")]
    NpuDetectionFailed {
        reason: String,
    },
    
    /// SoC检测失败
    #[error("SoC检测失败: {reason}")]
    SocDetectionFailed {
        reason: String,
    },
    
    /// 缓存操作失败
    #[error("缓存操作失败 ({operation}): {reason}")]
    CacheError {
        operation: String,
        reason: String,
    },
    
    /// 配置错误
    #[error("配置错误 (字段: {field}): {reason}")]
    ConfigError {
        field: String,
        reason: String,
    },
    
    /// NPU加速错误
    #[error("NPU加速错误 ({operation}): {reason}")]
    NpuAccelerationError {
        operation: String,
        reason: String,
    },
    
    /// 超分辨率错误
    #[error("超分辨率错误 ({technology}): {reason}")]
    UpscalingError {
        technology: String,
        reason: String,
    },
    
    /// 性能监控错误
    #[error("性能监控错误 (指标: {metric}): {reason}")]
    PerformanceMonitoringError {
        metric: String,
        reason: String,
    },
    
    /// 不支持的平台
    #[error("平台 {platform} 不支持功能: {feature}")]
    UnsupportedPlatform {
        platform: String,
        feature: String,
    },
    
    /// 不支持的硬件
    #[error("硬件 {hardware} 不支持功能: {feature}")]
    UnsupportedHardware {
        hardware: String,
        feature: String,
    },
    
    /// SDK初始化失败
    #[error("SDK初始化失败 ({sdk_name}): {reason}")]
    SdkInitializationFailed {
        sdk_name: String,
        reason: String,
    },
    
    /// 资源不足
    #[error("资源不足 ({resource}): 需要 {required}, 可用 {available}")]
    InsufficientResources {
        resource: String,
        required: String,
        available: String,
    },
    
    /// 超时
    #[error("操作超时 ({operation}): {timeout_ms}ms")]
    Timeout {
        operation: String,
        timeout_ms: u64,
    },
    
    /// 其他错误
    #[error("硬件优化错误: {0}")]
    Other(String),
}

/// 硬件优化结果类型
pub type HardwareResult<T> = Result<T, HardwareError>;

/// 错误上下文扩展
pub trait ErrorContext<T> {
    /// 添加上下文信息
    fn context(self, context: &str) -> HardwareResult<T>;
    
    /// 添加带格式化的上下文信息
    fn with_context<F>(self, f: F) -> HardwareResult<T>
    where
        F: FnOnce() -> String;
}

impl<T, E: std::error::Error + 'static> ErrorContext<T> for Result<T, E> {
    fn context(self, context: &str) -> HardwareResult<T> {
        self.map_err(|e| {
            HardwareError::Other(format!("{}: {}", context, e))
        })
    }

    fn with_context<F>(self, f: F) -> HardwareResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            HardwareError::Other(format!("{}: {}", f(), e))
        })
    }
}



/// 错误恢复策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// 使用默认值
    UseDefault,
    /// 重试
    Retry,
    /// 降级
    Fallback,
    /// 跳过
    Skip,
    /// 失败
    Fail,
}

/// 错误处理器
pub struct ErrorHandler {
    /// 是否记录错误
    log_errors: bool,
    /// 默认恢复策略
    default_strategy: RecoveryStrategy,
}

impl ErrorHandler {
    /// 创建新的错误处理器
    pub fn new() -> Self {
        Self {
            log_errors: true,
            default_strategy: RecoveryStrategy::UseDefault,
        }
    }
    
    /// 设置是否记录错误
    pub fn set_log_errors(&mut self, log: bool) {
        self.log_errors = log;
    }
    
    /// 设置默认恢复策略
    pub fn set_default_strategy(&mut self, strategy: RecoveryStrategy) {
        self.default_strategy = strategy;
    }
    
    /// 处理错误
    pub fn handle(&self, error: &HardwareError) -> RecoveryStrategy {
        if self.log_errors {
            tracing::error!(target: "hardware", "[硬件优化错误] {}", error);
        }
        
        // 根据错误类型选择恢复策略
        match error {
            HardwareError::GpuDetectionFailed { .. } => RecoveryStrategy::UseDefault,
            HardwareError::NpuDetectionFailed { .. } => RecoveryStrategy::Skip,
            HardwareError::SocDetectionFailed { .. } => RecoveryStrategy::Skip,
            HardwareError::CacheError { .. } => RecoveryStrategy::Fallback,
            HardwareError::ConfigError { .. } => RecoveryStrategy::UseDefault,
            HardwareError::NpuAccelerationError { .. } => RecoveryStrategy::Fallback,
            HardwareError::UpscalingError { .. } => RecoveryStrategy::Fallback,
            HardwareError::PerformanceMonitoringError { .. } => RecoveryStrategy::Skip,
            HardwareError::UnsupportedPlatform { .. } => RecoveryStrategy::Skip,
            HardwareError::UnsupportedHardware { .. } => RecoveryStrategy::Fallback,
            HardwareError::SdkInitializationFailed { .. } => RecoveryStrategy::Fallback,
            HardwareError::InsufficientResources { .. } => RecoveryStrategy::Fallback,
            HardwareError::Timeout { .. } => RecoveryStrategy::Retry,
            HardwareError::Other(_) => self.default_strategy,
        }
    }
    
    /// 处理错误并返回建议
    pub fn handle_with_suggestion(&self, error: &HardwareError) -> (RecoveryStrategy, String) {
        let strategy = self.handle(error);
        
        let suggestion = match error {
            HardwareError::GpuDetectionFailed { .. } => {
                "将使用默认GPU配置。请检查显卡驱动是否正确安装。".to_string()
            }
            HardwareError::NpuDetectionFailed { .. } => {
                "NPU功能将被禁用。这不会影响游戏的基本功能。".to_string()
            }
            HardwareError::SocDetectionFailed { .. } => {
                "SoC优化将被禁用。游戏将使用通用配置。".to_string()
            }
            HardwareError::CacheError { operation, .. } => {
                format!("缓存操作({})失败，将直接检测硬件。", operation)
            }
            HardwareError::NpuAccelerationError { .. } => {
                "NPU加速失败，将使用CPU进行计算。".to_string()
            }
            HardwareError::UpscalingError { technology, .. } => {
                format!("{}超分辨率失败，将使用原生分辨率或其他技术。", technology)
            }
            HardwareError::UnsupportedPlatform { platform, feature } => {
                format!("平台{}不支持{}，将使用替代方案。", platform, feature)
            }
            HardwareError::UnsupportedHardware { hardware, feature } => {
                format!("硬件{}不支持{}，将使用替代方案。", hardware, feature)
            }
            HardwareError::InsufficientResources { resource, .. } => {
                format!("{}不足，将降低画质或性能要求。", resource)
            }
            HardwareError::Timeout { operation, .. } => {
                format!("操作{}超时，将重试或使用默认值。", operation)
            }
            _ => "将尝试使用默认配置继续运行。".to_string(),
        };
        
        (strategy, suggestion)
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = HardwareError::GpuDetectionFailed {
            reason: "wgpu初始化失败".to_string(),
            attempted_methods: vec!["wgpu".to_string(), "系统API".to_string()],
        };
        
            tracing::info!(target: "hardware", "{}", error);
        assert!(error.to_string().contains("GPU检测失败"));
    }
    
    #[test]
    fn test_error_handler() {
        let handler = ErrorHandler::new();
        
        let error = HardwareError::NpuDetectionFailed {
            reason: "未找到NPU设备".to_string(),
        };
        
        let (strategy, suggestion) = handler.handle_with_suggestion(&error);
        
        tracing::info!(target: "hardware", "恢复策略: {:?}", strategy);
        tracing::info!(target: "hardware", "建议: {}", suggestion);
        
        assert_eq!(strategy, RecoveryStrategy::Skip);
    }
    
    #[test]
    fn test_error_context() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "文件不存在",
        ));
        
        let hardware_result = result.context("读取配置文件");
        
        assert!(hardware_result.is_err());
        if let Err(e) = hardware_result {
            tracing::info!(target: "hardware", "错误: {}", e);
            assert!(e.to_string().contains("读取配置文件"));
        }
    }
}
