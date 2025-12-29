//  输入系统错误类型
//
//  定义了输入系统相关的所有错误类型，包括键盘、鼠标、触摸、游戏手柄等。

use crate::error::{ErrorCategory, ErrorSeverity};
use thiserror::Error;

/// 输入系统错误
///
/// 涵盖了输入处理中的所有可能的错误情况，
/// 从设备初始化到输入事件处理。
#[derive(Error, Debug, Clone)]
pub enum InputError {
    /// 输入设备初始化错误
    #[error("Input device initialization failed: {device_type} - {message}")]
    DeviceInitialization {
        /// 设备类型
        device_type: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入设备未找到
    #[error("Input device not found: {device_id}")]
    DeviceNotFound {
        /// 设备ID
        device_id: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入设备断开连接
    #[error("Input device disconnected: {device_id}")]
    DeviceDisconnected {
        /// 设备ID
        device_id: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入设备访问错误
    #[error("Input device access error: {device_id} - {message}")]
    DeviceAccess {
        /// 设备ID
        device_id: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入设备配置错误
    #[error("Input device configuration error: {device_id} - {message}")]
    DeviceConfiguration {
        /// 设备ID
        device_id: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入映射错误
    #[error("Input mapping error: {mapping} - {message}")]
    Mapping {
        /// 映射名称
        mapping: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入绑定冲突
    #[error("Input binding conflict: {action} already bound to {existing}")]
    BindingConflict {
        /// 动作名称
        action: String,
        /// 现有绑定
        existing: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 无效输入绑定
    #[error("Invalid input binding: {binding} - {message}")]
    InvalidBinding {
        /// 绑定名称
        binding: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入事件处理错误
    #[error("Input event processing error: {message}")]
    EventProcessing {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入队列溢出
    #[error("Input queue overflow: {queue_type}")]
    QueueOverflow {
        /// 队列类型
        queue_type: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入状态错误
    #[error("Input state error: {message}")]
    State {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入权限错误
    #[error("Input permission error: {message}")]
    Permission {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入驱动错误
    #[error("Input driver error: {driver} - {message}")]
    Driver {
        /// 驱动名称
        driver: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入系统初始化错误
    #[error("Input system initialization failed: {message}")]
    SystemInitialization {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入系统关闭错误
    #[error("Input system shutdown failed: {message}")]
    SystemShutdown {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入配置加载错误
    #[error("Input configuration loading failed: {config} - {message}")]
    ConfigurationLoading {
        /// 配置文件路径
        config: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入配置保存错误
    #[error("Input configuration saving failed: {config} - {message}")]
    ConfigurationSaving {
        /// 配置文件路径
        config: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入配置验证错误
    #[error("Input configuration validation failed: {message}")]
    ConfigurationValidation {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 手柄振动错误
    #[error("Gamepad vibration error: {gamepad_id} - {message}")]
    GamepadVibration {
        /// 手柄ID
        gamepad_id: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 手柄LED错误
    #[error("Gamepad LED error: {gamepad_id} - {message}")]
    GamepadLed {
        /// 手柄ID
        gamepad_id: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 触摸输入错误
    #[error("Touch input error: {message}")]
    Touch {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 手势识别错误
    #[error("Gesture recognition error: {gesture} - {message}")]
    GestureRecognition {
        /// 手势名称
        gesture: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入录制错误
    #[error("Input recording error: {message}")]
    Recording {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入回放错误
    #[error("Input playback error: {message}")]
    Playback {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入过滤器错误
    #[error("Input filter error: {filter} - {message}")]
    Filter {
        /// 过滤器名称
        filter: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入模态错误
    #[error("Input modal error: {message}")]
    Modal {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入焦点错误
    #[error("Input focus error: {message}")]
    Focus {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入光标错误
    #[error("Input cursor error: {message}")]
    Cursor {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入文本输入错误
    #[error("Input text input error: {message}")]
    TextInput {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 输入IME错误
    #[error("Input IME error: {message}")]
    Ime {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 通用输入错误
    #[error("Input error: {message}")]
    General {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },
}

impl InputError {
    /// 创建设备初始化错误
    pub fn device_initialization(
        device_type: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::DeviceInitialization {
            device_type: device_type.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建设备未找到错误
    pub fn device_not_found(device_id: impl Into<String>) -> Self {
        Self::DeviceNotFound {
            device_id: device_id.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建设备断开连接错误
    pub fn device_disconnected(device_id: impl Into<String>) -> Self {
        Self::DeviceDisconnected {
            device_id: device_id.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建设备访问错误
    pub fn device_access(device_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::DeviceAccess {
            device_id: device_id.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建设备配置错误
    pub fn device_configuration(device_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::DeviceConfiguration {
            device_id: device_id.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建映射错误
    pub fn mapping(mapping: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Mapping {
            mapping: mapping.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建绑定冲突错误
    pub fn binding_conflict(action: impl Into<String>, existing: impl Into<String>) -> Self {
        Self::BindingConflict {
            action: action.into(),
            existing: existing.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建无效绑定错误
    pub fn invalid_binding(binding: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidBinding {
            binding: binding.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建事件处理错误
    pub fn event_processing(message: impl Into<String>) -> Self {
        Self::EventProcessing {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建队列溢出错误
    pub fn queue_overflow(queue_type: impl Into<String>) -> Self {
        Self::QueueOverflow {
            queue_type: queue_type.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建状态错误
    pub fn state(message: impl Into<String>) -> Self {
        Self::State {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建权限错误
    pub fn permission(message: impl Into<String>) -> Self {
        Self::Permission {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建驱动错误
    pub fn driver(driver: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Driver {
            driver: driver.into(),
            message: message.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建系统初始化错误
    pub fn system_initialization(message: impl Into<String>) -> Self {
        Self::SystemInitialization {
            message: message.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建系统关闭错误
    pub fn system_shutdown(message: impl Into<String>) -> Self {
        Self::SystemShutdown {
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建配置加载错误
    pub fn configuration_loading(config: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ConfigurationLoading {
            config: config.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建配置保存错误
    pub fn configuration_saving(config: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ConfigurationSaving {
            config: config.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建配置验证错误
    pub fn configuration_validation(message: impl Into<String>) -> Self {
        Self::ConfigurationValidation {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建手柄振动错误
    pub fn gamepad_vibration(gamepad_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::GamepadVibration {
            gamepad_id: gamepad_id.into(),
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建触摸输入错误
    pub fn touch(message: impl Into<String>) -> Self {
        Self::Touch {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建手势识别错误
    pub fn gesture_recognition(gesture: impl Into<String>, message: impl Into<String>) -> Self {
        Self::GestureRecognition {
            gesture: gesture.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建录制错误
    pub fn recording(message: impl Into<String>) -> Self {
        Self::Recording {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建回放错误
    pub fn playback(message: impl Into<String>) -> Self {
        Self::Playback {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建过滤器错误
    pub fn filter(filter: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Filter {
            filter: filter.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建焦点错误
    pub fn focus(message: impl Into<String>) -> Self {
        Self::Focus {
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建光标错误
    pub fn cursor(message: impl Into<String>) -> Self {
        Self::Cursor {
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建文本输入错误
    pub fn text_input(message: impl Into<String>) -> Self {
        Self::TextInput {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建IME错误
    pub fn ime(message: impl Into<String>) -> Self {
        Self::Ime {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建通用输入错误
    pub fn general(message: impl Into<String>) -> Self {
        Self::General {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建带有严重级别的通用输入错误
    pub fn general_with_severity(message: impl Into<String>, severity: ErrorSeverity) -> Self {
        Self::General {
            message: message.into(),
            severity,
        }
    }

    /// 获取错误的严重级别
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            InputError::DeviceInitialization { severity, .. }
            | InputError::DeviceNotFound { severity, .. }
            | InputError::DeviceDisconnected { severity, .. }
            | InputError::DeviceAccess { severity, .. }
            | InputError::DeviceConfiguration { severity, .. }
            | InputError::Mapping { severity, .. }
            | InputError::BindingConflict { severity, .. }
            | InputError::InvalidBinding { severity, .. }
            | InputError::EventProcessing { severity, .. }
            | InputError::QueueOverflow { severity, .. }
            | InputError::State { severity, .. }
            | InputError::Permission { severity, .. }
            | InputError::Driver { severity, .. }
            | InputError::SystemInitialization { severity, .. }
            | InputError::SystemShutdown { severity, .. }
            | InputError::ConfigurationLoading { severity, .. }
            | InputError::ConfigurationSaving { severity, .. }
            | InputError::ConfigurationValidation { severity, .. }
            | InputError::GamepadVibration { severity, .. }
            | InputError::GamepadLed { severity, .. }
            | InputError::Touch { severity, .. }
            | InputError::GestureRecognition { severity, .. }
            | InputError::Recording { severity, .. }
            | InputError::Playback { severity, .. }
            | InputError::Filter { severity, .. }
            | InputError::Modal { severity, .. }
            | InputError::Focus { severity, .. }
            | InputError::Cursor { severity, .. }
            | InputError::TextInput { severity, .. }
            | InputError::Ime { severity, .. }
            | InputError::General { severity, .. } => *severity,
        }
    }

    /// 检查错误是否可恢复
    pub fn is_recoverable(&self) -> bool {
        match self {
            // 严重错误通常不可恢复
            InputError::Driver { severity, .. }
            | InputError::SystemInitialization { severity, .. } => {
                *severity < ErrorSeverity::Critical
            }

            // 设备断开连接通常可恢复（可以重新连接）
            InputError::DeviceDisconnected { .. } => true,

            // 配置错误通常可恢复（可以使用默认配置）
            InputError::ConfigurationLoading { .. }
            | InputError::ConfigurationSaving { .. }
            | InputError::ConfigurationValidation { .. } => true,

            // 队列溢出通常可恢复（可以清空队列）
            InputError::QueueOverflow { .. } => true,

            // 绑定冲突通常可恢复（可以重新映射）
            InputError::BindingConflict { .. } => true,

            // 其他错误需要根据严重级别判断
            _ => self.severity() < ErrorSeverity::Critical,
        }
    }

    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        ErrorCategory::Input
    }

    /// 检查是否为设备相关错误
    pub fn is_device_related(&self) -> bool {
        matches!(
            self,
            InputError::DeviceInitialization { .. }
                | InputError::DeviceNotFound { .. }
                | InputError::DeviceDisconnected { .. }
                | InputError::DeviceAccess { .. }
                | InputError::DeviceConfiguration { .. }
                | InputError::Driver { .. }
        )
    }

    /// 检查是否为映射相关错误
    pub fn is_mapping_related(&self) -> bool {
        matches!(
            self,
            InputError::Mapping { .. }
                | InputError::BindingConflict { .. }
                | InputError::InvalidBinding { .. }
        )
    }

    /// 检查是否为配置相关错误
    pub fn is_configuration_related(&self) -> bool {
        matches!(
            self,
            InputError::ConfigurationLoading { .. }
                | InputError::ConfigurationSaving { .. }
                | InputError::ConfigurationValidation { .. }
        )
    }

    /// 检查是否为游戏手柄相关错误
    pub fn is_gamepad_related(&self) -> bool {
        matches!(
            self,
            InputError::GamepadVibration { .. } | InputError::GamepadLed { .. }
        )
    }

    /// 检查是否为触摸相关错误
    pub fn is_touch_related(&self) -> bool {
        matches!(
            self,
            InputError::Touch { .. } | InputError::GestureRecognition { .. }
        )
    }

    /// 检查是否为文本输入相关错误
    pub fn is_text_input_related(&self) -> bool {
        matches!(self, InputError::TextInput { .. } | InputError::Ime { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_error_creation() {
        let err = InputError::device_not_found("gamepad_001");
        assert_eq!(err.severity(), ErrorSeverity::Error);
        assert!(err.is_device_related());
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_input_error_severity() {
        let critical_err = InputError::system_initialization("Failed to initialize input system");
        assert_eq!(critical_err.severity(), ErrorSeverity::Critical);
        assert!(!critical_err.is_recoverable());

        let normal_err = InputError::general("Temporary input issue");
        assert_eq!(normal_err.severity(), ErrorSeverity::Error);
        assert!(normal_err.is_recoverable());
    }

    #[test]
    fn test_input_error_categories() {
        let device_err = InputError::device_access("keyboard_001", "Permission denied");
        assert!(device_err.is_device_related());

        let mapping_err = InputError::binding_conflict("jump", "space");
        assert!(mapping_err.is_mapping_related());

        let config_err = InputError::configuration_loading("input.json", "Invalid format");
        assert!(config_err.is_configuration_related());

        let gamepad_err = InputError::gamepad_vibration("gamepad_001", "Not supported");
        assert!(gamepad_err.is_gamepad_related());

        let touch_err = InputError::touch("Invalid touch coordinates");
        assert!(touch_err.is_touch_related());

        let text_err = InputError::text_input("Invalid character encoding");
        assert!(text_err.is_text_input_related());
    }

    #[test]
    fn test_binding_conflict_error() {
        let err = InputError::binding_conflict("jump", "space");
        assert!(
            matches!(err, InputError::BindingConflict { ref action, ref existing, .. } if action.as_str() == "jump" && existing.as_str() == "space")
        );
        assert_eq!(err.severity(), ErrorSeverity::Error);
        assert!(err.is_mapping_related());
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_device_disconnected_error() {
        let err = InputError::device_disconnected("mouse_001");
        assert_eq!(err.severity(), ErrorSeverity::Warning);
        assert!(err.is_device_related());
        assert!(err.is_recoverable());
    }
}
