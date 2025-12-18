//! 音频系统错误类型
//!
//! 定义了音频系统相关的所有错误类型，包括音频设备、音频源、播放控制等。

use crate::error::{ErrorSeverity, ErrorCategory};
use thiserror::Error;

/// 音频系统错误
///
/// 涵盖了音频处理中的所有可能的错误情况，
/// 从设备初始化到音频播放控制。
#[derive(Error, Debug, Clone)]
pub enum AudioError {
    /// 音频设备初始化错误
    #[error("Audio device initialization failed: {message}")]
    DeviceInitialization {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频设备未找到
    #[error("Audio device not found: {device_name}")]
    DeviceNotFound {
        /// 设备名称
        device_name: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频设备配置错误
    #[error("Audio device configuration error: {message}")]
    DeviceConfiguration {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频设备访问错误
    #[error("Audio device access error: {message}")]
    DeviceAccess {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频源创建错误
    #[error("Audio source creation failed: {message}")]
    SourceCreation {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频源未找到
    #[error("Audio source not found: {source_id}")]
    SourceNotFound {
        /// 音频源ID
        source_id: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频文件加载错误
    #[error("Audio file loading failed: {file} - {message}")]
    FileLoading {
        /// 文件路径
        file: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频解码错误
    #[error("Audio decoding failed: {file} - {message}")]
    Decoding {
        /// 文件路径
        file: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频格式不支持
    #[error("Unsupported audio format: {format} for file {file}")]
    UnsupportedFormat {
        /// 文件路径
        file: String,
        /// 音频格式
        format: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频播放错误
    #[error("Audio playback failed: {message}")]
    Playback {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频暂停错误
    #[error("Audio pause failed: {message}")]
    Pause {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频恢复错误
    #[error("Audio resume failed: {message}")]
    Resume {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频停止错误
    #[error("Audio stop failed: {message}")]
    Stop {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音量设置错误
    #[error("Volume setting failed: {message}")]
    VolumeSetting {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 无效音量值
    #[error("Invalid volume value: {value} (valid range: 0.0 - 1.0)")]
    InvalidVolume {
        /// 音量值
        value: f32,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音调设置错误
    #[error("Pitch setting failed: {message}")]
    PitchSetting {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 无效音调值
    #[error("Invalid pitch value: {value} (valid range: 0.1 - 10.0)")]
    InvalidPitch {
        /// 音调值
        value: f32,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 声道设置错误
    #[error("Channel setting failed: {message}")]
    ChannelSetting {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 3D音频位置设置错误
    #[error("3D audio position setting failed: {message}")]
    PositionSetting {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 3D音频速度设置错误
    #[error("3D audio velocity setting failed: {message}")]
    VelocitySetting {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频流错误
    #[error("Audio streaming error: {message}")]
    Streaming {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频缓冲区错误
    #[error("Audio buffer error: {message}")]
    Buffer {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频效果错误
    #[error("Audio effect error: {effect} - {message}")]
    Effect {
        /// 效果名称
        effect: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频混音器错误
    #[error("Audio mixer error: {message}")]
    Mixer {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频监听器错误
    #[error("Audio listener error: {message}")]
    Listener {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频总线错误
    #[error("Audio bus error: {bus} - {message}")]
    Bus {
        /// 总线名称
        bus: String,
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频录制错误
    #[error("Audio recording error: {message}")]
    Recording {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频压缩错误
    #[error("Audio compression error: {message}")]
    Compression {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 音频权限错误
    #[error("Audio permission error: {message}")]
    Permission {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },

    /// 通用音频错误
    #[error("Audio error: {message}")]
    General {
        /// 错误消息
        message: String,
        /// 错误严重级别

        severity: ErrorSeverity,
    },
}

impl AudioError {
    /// 创建设备初始化错误
    pub fn device_initialization(message: impl Into<String>) -> Self {
        Self::DeviceInitialization {
            message: message.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建设备未找到错误
    pub fn device_not_found(device_name: impl Into<String>) -> Self {
        Self::DeviceNotFound {
            device_name: device_name.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建设备配置错误
    pub fn device_configuration(message: impl Into<String>) -> Self {
        Self::DeviceConfiguration {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建设备访问错误
    pub fn device_access(message: impl Into<String>) -> Self {
        Self::DeviceAccess {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建音频源创建错误
    pub fn source_creation(message: impl Into<String>) -> Self {
        Self::SourceCreation {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建音频源未找到错误
    pub fn source_not_found(source_id: impl Into<String>) -> Self {
        Self::SourceNotFound {
            source_id: source_id.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建文件加载错误
    pub fn file_loading(
        file: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::FileLoading {
            file: file.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建解码错误
    pub fn decoding(
        file: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Decoding {
            file: file.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建不支持的格式错误
    pub fn unsupported_format(
        file: impl Into<String>,
        format: impl Into<String>,
    ) -> Self {
        Self::UnsupportedFormat {
            file: file.into(),
            format: format.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建播放错误
    pub fn playback(message: impl Into<String>) -> Self {
        Self::Playback {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建暂停错误
    pub fn pause(message: impl Into<String>) -> Self {
        Self::Pause {
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建恢复错误
    pub fn resume(message: impl Into<String>) -> Self {
        Self::Resume {
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建停止错误
    pub fn stop(message: impl Into<String>) -> Self {
        Self::Stop {
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建无效音量错误
    pub fn invalid_volume(value: f32) -> Self {
        Self::InvalidVolume {
            value,
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建音量设置错误
    pub fn volume_setting(message: impl Into<String>) -> Self {
        Self::VolumeSetting {
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建无效音调错误
    pub fn invalid_pitch(value: f32) -> Self {
        Self::InvalidPitch {
            value,
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建3D位置设置错误
    pub fn position_setting(message: impl Into<String>) -> Self {
        Self::PositionSetting {
            message: message.into(),
            severity: ErrorSeverity::Warning,
        }
    }

    /// 创建流错误
    pub fn streaming(message: impl Into<String>) -> Self {
        Self::Streaming {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建缓冲区错误
    pub fn buffer(message: impl Into<String>) -> Self {
        Self::Buffer {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建效果错误
    pub fn effect(
        effect: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Effect {
            effect: effect.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建混音器错误
    pub fn mixer(message: impl Into<String>) -> Self {
        Self::Mixer {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建监听器错误
    pub fn listener(message: impl Into<String>) -> Self {
        Self::Listener {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建总线错误
    pub fn bus(
        bus: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Bus {
            bus: bus.into(),
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

    /// 创建权限错误
    pub fn permission(message: impl Into<String>) -> Self {
        Self::Permission {
            message: message.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建通用音频错误
    pub fn general(message: impl Into<String>) -> Self {
        Self::General {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建带有严重级别的通用音频错误
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
            AudioError::DeviceInitialization { severity, .. }
            | AudioError::DeviceNotFound { severity, .. }
            | AudioError::DeviceConfiguration { severity, .. }
            | AudioError::DeviceAccess { severity, .. }
            | AudioError::SourceCreation { severity, .. }
            | AudioError::SourceNotFound { severity, .. }
            | AudioError::FileLoading { severity, .. }
            | AudioError::Decoding { severity, .. }
            | AudioError::UnsupportedFormat { severity, .. }
            | AudioError::Playback { severity, .. }
            | AudioError::Pause { severity, .. }
            | AudioError::Resume { severity, .. }
            | AudioError::Stop { severity, .. }
            | AudioError::VolumeSetting { severity, .. }
            | AudioError::InvalidVolume { severity, .. }
            | AudioError::PitchSetting { severity, .. }
            | AudioError::InvalidPitch { severity, .. }
            | AudioError::ChannelSetting { severity, .. }
            | AudioError::PositionSetting { severity, .. }
            | AudioError::VelocitySetting { severity, .. }
            | AudioError::Streaming { severity, .. }
            | AudioError::Buffer { severity, .. }
            | AudioError::Effect { severity, .. }
            | AudioError::Mixer { severity, .. }
            | AudioError::Listener { severity, .. }
            | AudioError::Bus { severity, .. }
            | AudioError::Recording { severity, .. }
            | AudioError::Compression { severity, .. }
            | AudioError::Permission { severity, .. }
            | AudioError::General { severity, .. } => *severity,
        }
    }

    /// 检查错误是否可恢复
    pub fn is_recoverable(&self) -> bool {
        match self {
            // 设备级错误通常不可恢复
            AudioError::DeviceInitialization { severity, .. }
            | AudioError::Permission { severity, .. } => *severity < ErrorSeverity::Critical,

            // 文件加载错误通常可恢复（可以尝试其他文件）
            AudioError::FileLoading { .. }
            | AudioError::Decoding { .. }
            | AudioError::UnsupportedFormat { .. } => true,

            // 播放控制错误通常可恢复
            AudioError::Playback { .. }
            | AudioError::Pause { .. }
            | AudioError::Resume { .. }
            | AudioError::Stop { .. } => true,

            // 参数错误通常可恢复（可以修正参数）
            AudioError::InvalidVolume { .. }
            | AudioError::InvalidPitch { .. } => true,

            // 其他错误需要根据严重级别判断
            _ => self.severity() < ErrorSeverity::Critical,
        }
    }

    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        ErrorCategory::Audio
    }

    /// 检查是否为设备相关错误
    pub fn is_device_related(&self) -> bool {
        matches!(
            self,
            AudioError::DeviceInitialization { .. }
                | AudioError::DeviceNotFound { .. }
                | AudioError::DeviceConfiguration { .. }
                | AudioError::DeviceAccess { .. }
                | AudioError::Permission { .. }
        )
    }

    /// 检查是否为文件相关错误
    pub fn is_file_related(&self) -> bool {
        matches!(
            self,
            AudioError::FileLoading { .. }
                | AudioError::Decoding { .. }
                | AudioError::UnsupportedFormat { .. }
        )
    }

    /// 检查是否为播放相关错误
    pub fn is_playback_related(&self) -> bool {
        matches!(
            self,
            AudioError::Playback { .. }
                | AudioError::Pause { .. }
                | AudioError::Resume { .. }
                | AudioError::Stop { .. }
                | AudioError::VolumeSetting { .. }
                | AudioError::InvalidVolume { .. }
                | AudioError::PitchSetting { .. }
                | AudioError::InvalidPitch { .. }
        )
    }

    /// 检查是否为3D音频相关错误
    pub fn is_3d_related(&self) -> bool {
        matches!(
            self,
            AudioError::PositionSetting { .. }
                | AudioError::VelocitySetting { .. }
                | AudioError::Listener { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_error_creation() {
        let err = AudioError::invalid_volume(1.5);
        assert_eq!(err.severity(), ErrorSeverity::Warning);
        assert!(err.is_playback_related());
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_audio_error_severity() {
        let critical_err = AudioError::device_initialization("No audio devices available");
        assert_eq!(critical_err.severity(), ErrorSeverity::Critical);
        assert!(critical_err.is_device_related());
        assert!(!critical_err.is_recoverable());

        let normal_err = AudioError::general("Temporary audio issue");
        assert_eq!(normal_err.severity(), ErrorSeverity::Error);
        assert!(normal_err.is_recoverable());
    }

    #[test]
    fn test_audio_error_categories() {
        let file_err = AudioError::file_loading("test.wav", "File not found");
        assert!(file_err.is_file_related());

        let device_err = AudioError::device_not_found("Default Device");
        assert!(device_err.is_device_related());

        let playback_err = AudioError::playback("Buffer underrun");
        assert!(playback_err.is_playback_related());

        let pos_err = AudioError::position_setting("Invalid coordinates");
        assert!(pos_err.is_3d_related());
    }

    #[test]
    fn test_invalid_volume_error() {
        let err = AudioError::invalid_volume(1.5);
        assert!(matches!(err, AudioError::InvalidVolume { value: 1.5, .. }));
        assert_eq!(err.severity(), ErrorSeverity::Warning);
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_unsupported_format_error() {
        let err = AudioError::unsupported_format("test.xyz", "XYZ");
        assert!(matches!(err, AudioError::UnsupportedFormat { file: _, ref format, .. } if format.as_str() == "XYZ"));
        assert!(err.is_file_related());
        assert!(err.is_recoverable());
    }
}