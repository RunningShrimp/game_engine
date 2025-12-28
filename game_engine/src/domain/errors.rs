//  领域特定错误类型
//
//  ## 架构改进 (2025-12-27)
//
//  移除了重复的错误定义，统一使用 src/error/ 中的错误类型。
//  这消除了命名冲突，并确保整个引擎使用一致的错误类型。

use thiserror::Error;
use serde::{Serialize, Deserialize};

// 重新导出统一错误类型
pub use crate::error::{AudioError, PhysicsError};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        let audio_error = AudioError::DeviceConfiguration {
            message: "test".to_string(),
            severity: crate::error::ErrorSeverity::Error,
        };
        let domain_error: DomainError = audio_error.into();
        assert!(matches!(
            domain_error,
            DomainError::Audio(AudioError::DeviceConfiguration { .. })
        ));
    }

    #[test]
    fn test_domain_error_from_physics_error() {
        let physics_error = PhysicsError::Configuration {
            message: "test".to_string(),
            severity: crate::error::ErrorSeverity::Error,
        };
        let domain_error: DomainError = physics_error.into();
        assert!(matches!(
            domain_error,
            DomainError::Physics(PhysicsError::Configuration { .. })
        ));
    }

    #[test]
    fn test_domain_error_from_scene_error() {
        let scene_error = SceneError::SceneNotFound("test".to_string());
        let domain_error: DomainError = scene_error.into();
        assert!(matches!(
            domain_error,
            DomainError::Scene(SceneError::SceneNotFound(_))
        ));
    }

    #[test]
    fn test_domain_error_general() {
        let error = DomainError::General("test error".to_string());
        assert!(matches!(error, DomainError::General(_)));
    }

    // 注意：移除了 test_audio_error_variants 和 test_physics_error_variants
    // 因为这些错误类型现在是重新导出的 crate::error::{AudioError, PhysicsError}
    // 它们有自己的测试模块 (src/error/audio_error/tests.rs 等)

    #[test]
    fn test_scene_error_variants() {
        assert!(matches!(
            SceneError::EntityNotFound("test".to_string()),
            SceneError::EntityNotFound(_)
        ));
        assert!(matches!(
            SceneError::SceneNotFound("test".to_string()),
            SceneError::SceneNotFound(_)
        ));
        assert!(matches!(
            SceneError::ComponentNotFound("test".to_string()),
            SceneError::ComponentNotFound(_)
        ));
        assert!(matches!(
            SceneError::SerializationFailed("test".to_string()),
            SceneError::SerializationFailed(_)
        ));
        assert!(matches!(
            SceneError::DeserializationFailed("test".to_string()),
            SceneError::DeserializationFailed(_)
        ));
    }

    #[test]
    fn test_recovery_strategy_variants() {
        assert!(matches!(
            RecoveryStrategy::Retry {
                max_attempts: 3,
                delay_ms: 100
            },
            RecoveryStrategy::Retry { .. }
        ));
        assert!(matches!(
            RecoveryStrategy::UseDefault,
            RecoveryStrategy::UseDefault
        ));
        assert!(matches!(RecoveryStrategy::Skip, RecoveryStrategy::Skip));
        assert!(matches!(
            RecoveryStrategy::LogAndContinue,
            RecoveryStrategy::LogAndContinue
        ));
        assert!(matches!(RecoveryStrategy::Fail, RecoveryStrategy::Fail));
    }

    #[test]
    fn test_compensation_action_creation() {
        let action = CompensationAction::new("test_id", "test_action", json!({"key": "value"}));
        assert_eq!(action.id, "test_id");
        assert_eq!(action.action_type, "test_action");
        assert_eq!(
            action.data.get("key").and_then(|v| v.as_str()),
            Some("value")
        );
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
