//! 统一错误处理器
//!
//! 提供统一的错误处理接口，集成错误恢复、日志记录和错误监控。

use crate::error::recovery::{ErrorRecovery, RecoveryContext};
use crate::error::{
    EngineError, ErrorMonitor, ErrorSeverity, Logger, RecoveryResult, RecoveryStrategy,
};
use std::sync::Arc;

/// 错误处理配置
#[derive(Debug, Clone)]
pub struct ErrorHandlerConfig {
    /// 是否自动记录错误
    pub auto_log: bool,
    /// 是否自动尝试恢复
    pub auto_recover: bool,
    /// 默认恢复策略
    pub default_recovery_strategy: RecoveryStrategy,
    /// 是否启用错误监控
    pub enable_monitoring: bool,
}

impl Default for ErrorHandlerConfig {
    fn default() -> Self {
        Self {
            auto_log: true,
            auto_recover: false,
            default_recovery_strategy: RecoveryStrategy::LogAndContinue {
                log_level: ErrorSeverity::Warning,
                context: String::new(),
            },
            enable_monitoring: true,
        }
    }
}

/// 统一错误处理器
pub struct ErrorHandler {
    /// 配置
    config: ErrorHandlerConfig,
    /// 日志管理器
    logger: Option<Arc<Logger>>,
    /// 错误恢复器
    recovery: Option<Arc<dyn ErrorRecovery>>,
    /// 错误监控器
    monitor: Option<Arc<ErrorMonitor>>,
}

impl ErrorHandler {
    /// 创建新的错误处理器
    pub fn new(config: ErrorHandlerConfig) -> Self {
        Self {
            config,
            logger: None,
            recovery: None,
            monitor: None,
        }
    }

    /// 设置日志管理器
    pub fn with_logger(mut self, logger: Arc<Logger>) -> Self {
        self.logger = Some(logger);
        self
    }

    /// 设置错误恢复器
    pub fn with_recovery(mut self, recovery: Arc<dyn ErrorRecovery>) -> Self {
        self.recovery = Some(recovery);
        self
    }

    /// 设置错误监控器
    pub fn with_monitor(mut self, monitor: Arc<ErrorMonitor>) -> Self {
        self.monitor = Some(monitor);
        self
    }

    /// 处理错误
    ///
    /// # 参数
    ///
    /// * `error` - 要处理的错误
    ///
    /// # 返回
    ///
    /// 恢复结果（如果启用了自动恢复）
    pub fn handle(&self, error: &EngineError) -> Option<RecoveryResult<()>> {
        // 记录错误
        if self.config.auto_log {
            self.log_error(error);
        }

        // 监控错误
        if self.config.enable_monitoring
            && let Some(ref monitor) = self.monitor
        {
            monitor.record_error(error.clone());
        }

        // 尝试恢复
        if self.config.auto_recover
            && let Some(ref recovery) = self.recovery
        {
            let context = RecoveryContext {
                operation: "error_handling".to_string(),
                error_history: vec![error.clone()],
                recovery_attempts: 0,
                context_data: std::collections::HashMap::new(),
                start_time: std::time::Instant::now(),
            };
            return Some(recovery.recover(error, &context));
        }

        None
    }

    /// 记录错误
    fn log_error(&self, error: &EngineError) {
        if let Some(ref logger) = self.logger {
            logger.log_error(error, None);
        } else {
            // 使用全局日志函数
            crate::error::log_error(error, None);
        }
    }

    /// 处理错误并返回恢复结果
    ///
    /// 这是一个便利方法，结合了错误处理和恢复。
    pub fn handle_and_recover(&self, error: &EngineError) -> RecoveryResult<()> {
        // 先处理错误
        self.handle(error);

        // 尝试恢复
        if let Some(ref recovery) = self.recovery {
            let context = RecoveryContext {
                operation: "error_handling".to_string(),
                error_history: vec![error.clone()],
                recovery_attempts: 0,
                context_data: std::collections::HashMap::new(),
                start_time: std::time::Instant::now(),
            };
            recovery.recover(error, &context)
        } else {
            RecoveryResult::Failed(error.clone())
        }
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new(ErrorHandlerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // ErrorSeverity 未在此文件中使用，但可能在未来需要
    // use crate::error::ErrorSeverity;

    #[test]
    fn test_error_handler_config_default() {
        let config = ErrorHandlerConfig::default();
        assert!(config.auto_log);
        assert!(!config.auto_recover);
    }

    #[test]
    fn test_error_handler_new() {
        let handler = ErrorHandler::new(ErrorHandlerConfig::default());
        assert!(handler.logger.is_none());
        assert!(handler.recovery.is_none());
    }
}
