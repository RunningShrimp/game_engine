//! 领域特定错误类型

use thiserror::Error;

/// 领域层错误枚举
#[derive(Error, Debug, Clone)]
pub enum DomainError {
    /// 音频领域错误
    #[error("Audio domain error: {0}")]
    Audio(#[from] AudioError),
    /// 物理领域错误
    #[error("Physics domain error: {0}")]
    Physics(#[from] PhysicsError),
    /// 场景领域错误
    #[error("Scene domain error: {0}")]
    Scene(#[from] SceneError),
    /// 通用领域错误
    #[error("Domain error: {0}")]
    General(String),
}

/// 音频领域错误
#[derive(Error, Debug, Clone)]
pub enum AudioError {
    /// 音频源未找到
    #[error("Audio source not found: {0}")]
    SourceNotFound(String),
    /// 音频播放失败
    #[error("Audio playback failed: {0}")]
    PlaybackFailed(String),
    /// 无效音频格式
    #[error("Invalid audio format: {0}")]
    InvalidFormat(String),
    /// 音频设备错误
    #[error("Audio device error: {0}")]
    DeviceError(String),
    /// 音量超出范围
    #[error("Invalid volume: {0}")]
    InvalidVolume(f32),
}

/// 物理领域错误
#[derive(Error, Debug, Clone)]
pub enum PhysicsError {
    /// 刚体未找到
    #[error("Physics body not found: {0}")]
    BodyNotFound(String),
    /// 碰撞体未找到
    #[error("Collider not found: {0}")]
    ColliderNotFound(String),
    /// 无效物理参数
    #[error("Invalid physics parameter: {0}")]
    InvalidParameter(String),
    /// 物理世界未初始化
    #[error("Physics world not initialized")]
    WorldNotInitialized,
    /// 关节创建失败
    #[error("Joint creation failed: {0}")]
    JointCreationFailed(String),
}

/// 场景领域错误
#[derive(Error, Debug, Clone)]
pub enum SceneError {
    /// 实体未找到
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
    /// 场景未找到
    #[error("Scene not found: {0}")]
    SceneNotFound(String),
    /// 组件未找到
    #[error("Component not found: {0}")]
    ComponentNotFound(String),
    /// 序列化失败
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    /// 反序列化失败
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
}

/// 错误恢复策略
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// 重试操作
    Retry { max_attempts: u32, delay_ms: u64 },
    /// 使用默认值
    UseDefault,
    /// 跳过操作
    Skip,
    /// 记录错误并继续
    LogAndContinue,
    /// 抛出错误
    Fail,
}

/// 补偿操作
#[derive(Debug, Clone)]
pub struct CompensationAction {
    /// 操作ID
    pub id: String,
    /// 操作类型
    pub action_type: String,
    /// 补偿数据
    pub data: serde_json::Value,
}

impl CompensationAction {
    pub fn new(
        id: impl Into<String>,
        action_type: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            action_type: action_type.into(),
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_domain_error_from_audio_error() {
        let audio_error = AudioError::InvalidVolume(1.5);
        let domain_error: DomainError = audio_error.into();
        assert!(matches!(domain_error, DomainError::Audio(AudioError::InvalidVolume(_))));
    }

    #[test]
    fn test_domain_error_from_physics_error() {
        let physics_error = PhysicsError::InvalidParameter("test".to_string());
        let domain_error: DomainError = physics_error.into();
        assert!(matches!(domain_error, DomainError::Physics(PhysicsError::InvalidParameter(_))));
    }

    #[test]
    fn test_domain_error_from_scene_error() {
        let scene_error = SceneError::SceneNotFound("test".to_string());
        let domain_error: DomainError = scene_error.into();
        assert!(matches!(domain_error, DomainError::Scene(SceneError::SceneNotFound(_))));
    }

    #[test]
    fn test_domain_error_general() {
        let error = DomainError::General("test error".to_string());
        assert!(matches!(error, DomainError::General(_)));
    }

    #[test]
    fn test_audio_error_variants() {
        assert!(matches!(AudioError::SourceNotFound("test".to_string()), AudioError::SourceNotFound(_)));
        assert!(matches!(AudioError::PlaybackFailed("test".to_string()), AudioError::PlaybackFailed(_)));
        assert!(matches!(AudioError::InvalidFormat("test".to_string()), AudioError::InvalidFormat(_)));
        assert!(matches!(AudioError::DeviceError("test".to_string()), AudioError::DeviceError(_)));
        assert!(matches!(AudioError::InvalidVolume(1.5), AudioError::InvalidVolume(_)));
    }

    #[test]
    fn test_physics_error_variants() {
        assert!(matches!(PhysicsError::BodyNotFound("test".to_string()), PhysicsError::BodyNotFound(_)));
        assert!(matches!(PhysicsError::ColliderNotFound("test".to_string()), PhysicsError::ColliderNotFound(_)));
        assert!(matches!(PhysicsError::InvalidParameter("test".to_string()), PhysicsError::InvalidParameter(_)));
        assert!(matches!(PhysicsError::WorldNotInitialized, PhysicsError::WorldNotInitialized));
        assert!(matches!(PhysicsError::JointCreationFailed("test".to_string()), PhysicsError::JointCreationFailed(_)));
    }

    #[test]
    fn test_scene_error_variants() {
        assert!(matches!(SceneError::EntityNotFound("test".to_string()), SceneError::EntityNotFound(_)));
        assert!(matches!(SceneError::SceneNotFound("test".to_string()), SceneError::SceneNotFound(_)));
        assert!(matches!(SceneError::ComponentNotFound("test".to_string()), SceneError::ComponentNotFound(_)));
        assert!(matches!(SceneError::SerializationFailed("test".to_string()), SceneError::SerializationFailed(_)));
        assert!(matches!(SceneError::DeserializationFailed("test".to_string()), SceneError::DeserializationFailed(_)));
    }

    #[test]
    fn test_recovery_strategy_variants() {
        assert!(matches!(RecoveryStrategy::Retry { max_attempts: 3, delay_ms: 100 }, RecoveryStrategy::Retry { .. }));
        assert!(matches!(RecoveryStrategy::UseDefault, RecoveryStrategy::UseDefault));
        assert!(matches!(RecoveryStrategy::Skip, RecoveryStrategy::Skip));
        assert!(matches!(RecoveryStrategy::LogAndContinue, RecoveryStrategy::LogAndContinue));
        assert!(matches!(RecoveryStrategy::Fail, RecoveryStrategy::Fail));
    }

    #[test]
    fn test_compensation_action_creation() {
        let action = CompensationAction::new("test_id", "test_action", json!({"key": "value"}));
        assert_eq!(action.id, "test_id");
        assert_eq!(action.action_type, "test_action");
        assert_eq!(action.data.get("key").and_then(|v| v.as_str()), Some("value"));
    }

    #[test]
    fn test_compensation_action_with_string_conversions() {
        let id_string = "test_id".to_string();
        let action_type_string = "test_action".to_string();
        let action = CompensationAction::new(id_string, action_type_string, json!({}));
        assert_eq!(action.id, "test_id");
        assert_eq!(action.action_type, "test_action");
    }
}
