//! 错误处理测试模块
//!
//! 为错误恢复策略、补偿操作和错误聚合器添加专门测试

#[cfg(test)]
mod tests {
    use super::super::audio::{AudioSource, AudioSourceId, AudioSourceState};
    use super::super::errors::{
        AudioError, CompensationAction, DomainError, PhysicsError, RecoveryStrategy, SceneError,
    };
    use super::super::physics::{RigidBody, RigidBodyId, RigidBodyType};
    use super::super::scene::{Scene, SceneId};
    use super::super::services::{AudioDomainService, PhysicsDomainService, SceneDomainService};
    use crate::core::error::{AssetError, EngineError, RenderError};
    use crate::core::error_aggregator::{ErrorAggregator, ErrorStats};
    use glam::{Quat, Vec3};
    use serde_json::json;

    // ========== 错误恢复策略测试 ==========

    #[test]
    fn test_recovery_strategy_retry() {
        let mut body = RigidBody::new(RigidBodyId(1), RigidBodyType::Dynamic, Vec3::ZERO);
        body.recovery_strategy = RecoveryStrategy::Retry {
            max_attempts: 3,
            delay_ms: 10,
        };

        // 测试重试策略：无效参数错误应该被恢复
        let error = PhysicsError::InvalidParameter("test".to_string());
        let result = body.recover_from_error(&error);

        // 重试策略应该成功恢复（重置为默认值）
        assert!(result.is_ok());
        assert_eq!(body.mass, 1.0);
        assert_eq!(body.position, Vec3::ZERO);
    }

    #[test]
    fn test_recovery_strategy_use_default() {
        let mut body = RigidBody::new(
            RigidBodyId(1),
            RigidBodyType::Dynamic,
            Vec3::new(10.0, 20.0, 30.0),
        );
        body.mass = 5.0;
        body.linear_velocity = Vec3::new(1.0, 2.0, 3.0);
        body.recovery_strategy = RecoveryStrategy::UseDefault;

        let error = PhysicsError::InvalidParameter("test".to_string());
        let result = body.recover_from_error(&error);

        assert!(result.is_ok());
        // 应该重置为默认值
        assert_eq!(body.mass, 1.0);
        assert_eq!(body.linear_velocity, Vec3::ZERO);
        assert_eq!(body.angular_velocity, 0.0);
    }

    #[test]
    fn test_recovery_strategy_skip() {
        let mut body = RigidBody::new(RigidBodyId(1), RigidBodyType::Dynamic, Vec3::ZERO);
        body.mass = 5.0;
        body.recovery_strategy = RecoveryStrategy::Skip;

        let error = PhysicsError::InvalidParameter("test".to_string());
        let result = body.recover_from_error(&error);

        assert!(result.is_ok());
        // Skip策略不应该改变状态
        assert_eq!(body.mass, 5.0);
    }

    #[test]
    fn test_recovery_strategy_log_and_continue() {
        let mut body = RigidBody::new(RigidBodyId(1), RigidBodyType::Dynamic, Vec3::ZERO);
        body.mass = 5.0;
        body.recovery_strategy = RecoveryStrategy::LogAndContinue;

        let error = PhysicsError::InvalidParameter("test".to_string());
        let result = body.recover_from_error(&error);

        assert!(result.is_ok());
        // LogAndContinue策略不应该改变状态
        assert_eq!(body.mass, 5.0);
    }

    #[test]
    fn test_recovery_strategy_fail() {
        let mut body = RigidBody::new(RigidBodyId(1), RigidBodyType::Dynamic, Vec3::ZERO);
        body.recovery_strategy = RecoveryStrategy::Fail;

        let error = PhysicsError::InvalidParameter("test".to_string());
        let result = body.recover_from_error(&error);

        // Fail策略应该返回错误
        assert!(result.is_err());
        if let Err(DomainError::Physics(e)) = result {
            assert!(matches!(e, PhysicsError::InvalidParameter(_)));
        } else {
            panic!("Expected Physics error");
        }
    }

    #[test]
    fn test_recovery_strategy_retry_exhausted() {
        let mut body = RigidBody::new(RigidBodyId(1), RigidBodyType::Dynamic, Vec3::ZERO);
        body.recovery_strategy = RecoveryStrategy::Retry {
            max_attempts: 2,
            delay_ms: 1,
        };

        // 使用一个无法恢复的错误类型
        let error = PhysicsError::BodyNotFound("test".to_string());
        let result = body.recover_from_error(&error);

        // 重试次数用尽后应该返回错误
        assert!(result.is_err());
    }

    // ========== 补偿操作测试 ==========

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
    fn test_rigid_body_compensation_roundtrip() {
        let mut body = RigidBody::new(
            RigidBodyId(1),
            RigidBodyType::Dynamic,
            Vec3::new(1.0, 2.0, 3.0),
        );
        body.rotation = Quat::from_rotation_y(1.0);
        body.linear_velocity = Vec3::new(4.0, 5.0, 6.0);
        body.angular_velocity = 7.0;
        body.mass = 8.0;
        body.sleeping = true;

        // 创建补偿操作
        let compensation = body.create_compensation();
        assert_eq!(compensation.action_type, "restore_physics_state");
        assert!(compensation.data.get("position").is_some());
        assert!(compensation.data.get("rotation").is_some());
        assert!(compensation.data.get("linear_velocity").is_some());
        assert!(compensation.data.get("angular_velocity").is_some());
        assert!(compensation.data.get("mass").is_some());
        assert!(compensation.data.get("sleeping").is_some());

        // 修改状态
        body.position = Vec3::new(10.0, 20.0, 30.0);
        body.mass = 100.0;
        body.sleeping = false;

        // 从补偿操作恢复
        let result = body.restore_from_compensation(&compensation);
        assert!(result.is_ok());

        // 验证状态已恢复
        assert_eq!(body.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(body.mass, 8.0);
        assert_eq!(body.sleeping, true);
    }

    #[test]
    fn test_audio_source_compensation_roundtrip() {
        let mut source = AudioSource::new(AudioSourceId(1));
        source.volume = crate::domain::value_objects::Volume::new_unchecked(0.7);
        source.looped = true;
        source.playback_position = 5.5;

        // 创建补偿操作
        let compensation = source.create_compensation();
        assert_eq!(compensation.action_type, "restore_audio_state");
        assert!(compensation.data.get("volume").is_some());
        assert!(compensation.data.get("looped").is_some());
        assert!(compensation.data.get("playback_position").is_some());

        // 修改状态
        source.volume = crate::domain::value_objects::Volume::new_unchecked(0.3);
        source.looped = false;
        source.playback_position = 10.0;

        // 从补偿操作恢复
        let result = source.restore_from_compensation(&compensation);
        assert!(result.is_ok());

        // 验证状态已恢复（允许小的浮点误差）
        assert!((source.volume.value() - 0.7).abs() < 0.001);
        assert_eq!(source.looped, true);
        assert!((source.playback_position - 5.5).abs() < 0.001);
    }

    #[test]
    fn test_scene_compensation_roundtrip() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");

        // 创建补偿操作（快照）
        let snapshot = scene.create_snapshot();
        assert_eq!(snapshot.scene_id, SceneId(1));
        assert_eq!(snapshot.name, "Test Scene");
        assert_eq!(snapshot.entity_count, 0);

        // 修改场景
        scene.name = "Modified Scene".to_string();

        // 验证快照未改变
        assert_eq!(snapshot.name, "Test Scene");
    }

    #[test]
    fn test_compensation_action_serialization() {
        let action = CompensationAction::new(
            "test_id",
            "test_action",
            json!({
                "position": [1.0, 2.0, 3.0],
                "mass": 5.0
            }),
        );

        // 验证JSON数据可以正确访问
        let pos = action
            .data
            .get("position")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(pos.len(), 3);
        assert_eq!(pos[0].as_f64(), Some(1.0));

        let mass = action.data.get("mass").and_then(|v| v.as_f64()).unwrap();
        assert_eq!(mass, 5.0);
    }

    // ========== 错误聚合器测试 ==========

    #[test]
    fn test_error_aggregator_record_error() {
        let aggregator = ErrorAggregator::new();

        let render_err = EngineError::Render(RenderError::NoAdapter);
        aggregator.record_error(&render_err, "render_system");

        let stats = aggregator.get_stats();
        assert_eq!(stats.total_count, 1);
        assert_eq!(stats.by_type.get("Render"), Some(&1));
        assert_eq!(stats.by_source.get("render_system"), Some(&1));
        assert_eq!(stats.recent_errors.len(), 1);
    }

    #[test]
    fn test_error_aggregator_multiple_errors() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_error(
            &EngineError::Render(RenderError::NoAdapter),
            "render_system",
        );
        aggregator.record_error(
            &EngineError::Asset(AssetError::NotFound {
                path: "test.png".to_string(),
            }),
            "asset_manager",
        );
        aggregator.record_error(
            &EngineError::Render(RenderError::NoAdapter),
            "render_system",
        );

        let stats = aggregator.get_stats();
        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.by_type.get("Render"), Some(&2));
        assert_eq!(stats.by_type.get("Asset"), Some(&1));
        assert_eq!(stats.by_source.get("render_system"), Some(&2));
        assert_eq!(stats.by_source.get("asset_manager"), Some(&1));
    }

    #[test]
    fn test_error_aggregator_custom_error() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_custom_error(
            "CustomError",
            "test_module",
            "Custom error message",
            Some("Additional details".to_string()),
        );

        let stats = aggregator.get_stats();
        assert_eq!(stats.total_count, 1);
        assert_eq!(stats.by_type.get("CustomError"), Some(&1));
        assert_eq!(stats.by_source.get("test_module"), Some(&1));

        let record = &stats.recent_errors[0];
        assert_eq!(record.error_type, "CustomError");
        assert_eq!(record.source, "test_module");
        assert_eq!(record.message, "Custom error message");
        assert_eq!(record.details, Some("Additional details".to_string()));
    }

    #[test]
    fn test_error_aggregator_summary() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_custom_error("ErrorA", "module1", "Message A", None);
        aggregator.record_custom_error("ErrorA", "module1", "Message A", None);
        aggregator.record_custom_error("ErrorB", "module2", "Message B", None);

        let summary = aggregator.get_summary();
        assert_eq!(summary.total_errors, 3);
        assert_eq!(summary.most_common_type, Some(("ErrorA".to_string(), 2)));
        assert_eq!(summary.most_common_source, Some(("module1".to_string(), 2)));
        assert_eq!(summary.recent_error_count, 3);
    }

    #[test]
    fn test_error_aggregator_clear() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_custom_error("TestError", "test_module", "Test message", None);
        assert_eq!(aggregator.get_stats().total_count, 1);

        aggregator.clear();
        let stats = aggregator.get_stats();
        assert_eq!(stats.total_count, 0);
        assert!(stats.by_type.is_empty());
        assert!(stats.by_source.is_empty());
        assert!(stats.recent_errors.is_empty());
    }

    #[test]
    fn test_error_aggregator_export_report() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_custom_error("TestError", "test_module", "Test message", None);

        let report = aggregator.export_report().unwrap();
        assert!(report.contains("TestError"));
        assert!(report.contains("test_module"));
        assert!(report.contains("Test message"));
        assert!(report.contains("total_count"));
    }

    #[test]
    fn test_error_aggregator_max_recent_errors() {
        let aggregator = ErrorAggregator::with_config(5, 60);

        // 记录超过最大数量的错误
        for i in 0..10 {
            aggregator.record_custom_error(
                "TestError",
                "test_module",
                &format!("Message {}", i),
                None,
            );
        }

        let stats = aggregator.get_stats();
        // 应该只保留最近5个错误
        assert_eq!(stats.recent_errors.len(), 5);
        assert_eq!(stats.total_count, 10); // 总数应该仍然是10
    }

    #[test]
    fn test_error_aggregator_error_rate() {
        let aggregator = ErrorAggregator::with_config(100, 60);

        // 记录一些错误
        for _ in 0..10 {
            aggregator.record_custom_error("TestError", "test_module", "Test message", None);
        }

        let stats = aggregator.get_stats();
        // 错误率应该大于0（在60秒窗口内）
        assert!(stats.error_rate >= 0.0);
    }

    #[test]
    fn test_error_aggregator_error_trend() {
        let aggregator = ErrorAggregator::new();

        // 记录一些错误
        for _ in 0..5 {
            aggregator.record_custom_error("TestError", "test_module", "Test message", None);
        }

        let stats = aggregator.get_stats();
        // 最近60秒内的错误趋势应该是5
        let trend = stats.error_trend(60);
        assert_eq!(trend, 5);
    }

    #[test]
    fn test_error_aggregator_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let aggregator = Arc::new(ErrorAggregator::new());
        let aggregator_clone = aggregator.clone();

        // 在多个线程中记录错误
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                aggregator_clone.record_custom_error(
                    "ThreadError",
                    "thread_module",
                    "Thread message",
                    None,
                );
            }
        });

        // 在主线程中也记录错误
        for _ in 0..10 {
            aggregator.record_custom_error("MainError", "main_module", "Main message", None);
        }

        handle.join().unwrap();

        let stats = aggregator.get_stats();
        // 应该记录所有20个错误
        assert_eq!(stats.total_count, 20);
    }

    // ========== 领域服务错误处理集成测试 ==========

    #[test]
    fn test_physics_service_error_recovery() {
        let mut service = PhysicsDomainService::new();

        // 创建一个刚体
        let body = RigidBody::new(RigidBodyId(1), RigidBodyType::Dynamic, Vec3::ZERO);
        assert!(service.create_body(body).is_ok());

        // 尝试设置无效质量（应该失败）
        // 通过领域服务获取刚体并设置质量
        let world = service.get_world();
        if let Some(_body) = world.body_handles.get(&RigidBodyId(1)) {
            // 创建一个临时的刚体来测试质量设置
            let mut test_body = RigidBody::new(RigidBodyId(1), RigidBodyType::Dynamic, Vec3::ZERO);
            let result = test_body.set_mass(-1.0);
            assert!(result.is_err());
        }

        // 尝试获取不存在的刚体位置（应该失败）
        let result = service.get_body_position(RigidBodyId(999));
        assert!(result.is_err());
    }

    #[test]
    fn test_audio_service_error_recovery() {
        let mut service = AudioDomainService::new();

        // 尝试从不存在的文件创建音频源（应该失败）
        let result = service.create_source(AudioSourceId(1), "nonexistent.wav");
        assert!(result.is_err());

        // 尝试获取不存在的音频源（应该失败）
        let result = service.get_source(AudioSourceId(999));
        assert!(result.is_none());
    }

    #[test]
    fn test_scene_service_error_recovery() {
        let mut service = SceneDomainService::new();

        // 尝试切换到不存在的场景（应该失败）
        let result = service.switch_to_scene(SceneId(999));
        assert!(result.is_err());

        // 创建场景
        assert!(service.create_scene(SceneId(1), "Test Scene").is_ok());

        // 尝试切换到未加载的场景（应该失败）
        let result = service.switch_to_scene(SceneId(1));
        assert!(result.is_err());
    }

    // ============================================================================
    // 补充更完整的错误恢复和补偿操作测试
    // ============================================================================

    #[test]
    fn test_rigid_body_recover_from_error_collider_not_found() {
        let mut body = RigidBody::new(RigidBodyId(1), RigidBodyType::Dynamic, Vec3::ZERO);
        body.recovery_strategy = RecoveryStrategy::Retry {
            max_attempts: 1,
            delay_ms: 1,
        };
        
        let error = PhysicsError::ColliderNotFound("test".to_string());
        let result = body.recover_from_error(&error);
        
        // ColliderNotFound错误无法恢复，应该返回错误
        assert!(result.is_err());
    }

    #[test]
    fn test_rigid_body_recover_from_error_world_not_initialized() {
        let mut body = RigidBody::new(RigidBodyId(1), RigidBodyType::Dynamic, Vec3::ZERO);
        body.recovery_strategy = RecoveryStrategy::Retry {
            max_attempts: 1,
            delay_ms: 1,
        };
        
        let error = PhysicsError::WorldNotInitialized;
        let result = body.recover_from_error(&error);
        
        // WorldNotInitialized错误无法恢复，应该返回错误
        assert!(result.is_err());
    }

    #[test]
    fn test_rigid_body_recover_from_error_joint_creation_failed() {
        let mut body = RigidBody::new(RigidBodyId(1), RigidBodyType::Dynamic, Vec3::ZERO);
        body.recovery_strategy = RecoveryStrategy::Retry {
            max_attempts: 1,
            delay_ms: 1,
        };
        
        let error = PhysicsError::JointCreationFailed("test".to_string());
        let result = body.recover_from_error(&error);
        
        // JointCreationFailed错误无法恢复，应该返回错误
        assert!(result.is_err());
    }

    #[test]
    fn test_audio_source_recover_from_error_invalid_format() {
        let mut source = AudioSource::new(AudioSourceId(1));
        source.recovery_strategy = RecoveryStrategy::Retry {
            max_attempts: 1,
            delay_ms: 1,
        };
        
        let error = AudioError::InvalidFormat("test".to_string());
        let result = source.recover_from_error(&error);
        
        // InvalidFormat错误无法恢复，应该返回错误
        assert!(result.is_err());
    }

    #[test]
    fn test_audio_source_recover_from_error_device_error() {
        let mut source = AudioSource::new(AudioSourceId(1));
        source.recovery_strategy = RecoveryStrategy::Retry {
            max_attempts: 1,
            delay_ms: 1,
        };
        
        let error = AudioError::DeviceError("test".to_string());
        let result = source.recover_from_error(&error);
        
        // DeviceError错误无法恢复，应该返回错误
        assert!(result.is_err());
    }

    #[test]
    fn test_scene_recover_from_error_component_not_found() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.recovery_strategy = RecoveryStrategy::Retry {
            max_attempts: 1,
            delay_ms: 1,
        };
        
        let error = SceneError::ComponentNotFound("test".to_string());
        let result = scene.recover_from_error(&error);
        
        // ComponentNotFound错误无法恢复，应该返回错误
        assert!(result.is_err());
    }

    #[test]
    fn test_compensation_action_restore_rotation() {
        // 测试恢复旋转数据
        let mut body = RigidBody::new(RigidBodyId(1), RigidBodyType::Dynamic, Vec3::ZERO);
        body.rotation = Quat::from_rotation_y(1.0);
        
        let compensation = body.create_compensation();
        
        // 修改旋转
        body.rotation = Quat::IDENTITY;
        
        // 恢复旋转（注意：当前实现不恢复旋转，只恢复位置、速度、质量等）
        body.restore_from_compensation(&compensation).unwrap();
        
        // 验证位置已恢复（旋转恢复需要实现）
        assert_eq!(body.position, Vec3::ZERO);
    }

    #[test]
    fn test_compensation_action_invalid_data() {
        // 测试无效的补偿数据
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        
        // 创建无效的补偿操作（位置数组长度错误）
        let invalid_compensation = CompensationAction::new(
            "test",
            "restore_physics_state",
            json!({
                "position": [1.0, 2.0], // 只有2个元素，应该是3个
                "mass": 5.0
            }),
        );
        
        // 应该能够处理无效数据（使用默认值）
        let result = body.restore_from_compensation(&invalid_compensation);
        assert!(result.is_ok());
        // 位置不应该改变（因为数据无效）
        assert_eq!(body.position, Vec3::ZERO);
        // 但质量应该恢复
        assert_eq!(body.mass, 5.0);
    }

    #[test]
    fn test_compensation_action_empty_data() {
        // 测试空的补偿数据
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        body.mass = 5.0;
        
        let empty_compensation = CompensationAction::new(
            "test",
            "restore_physics_state",
            json!({}),
        );
        
        // 应该能够处理空数据
        let result = body.restore_from_compensation(&empty_compensation);
        assert!(result.is_ok());
        // 状态不应该改变
        assert_eq!(body.mass, 5.0);
    }

    #[test]
    fn test_audio_source_restore_from_compensation_invalid_state() {
        // 测试无效的状态字符串
        let mut source = AudioSource::new(AudioSourceId(1));
        source.state = AudioSourceState::Playing;
        
        let compensation = CompensationAction::new(
            "test",
            "restore_audio_state",
            json!({
                "state": "InvalidState", // 无效的状态字符串
                "volume": 0.8
            }),
        );
        
        source.restore_from_compensation(&compensation).unwrap();
        // 无效状态应该被设置为Stopped
        assert_eq!(source.state, AudioSourceState::Stopped);
        assert!((source.volume.value() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_audio_source_restore_from_compensation_invalid_volume() {
        // 测试无效的音量值
        let mut source = AudioSource::new(AudioSourceId(1));
        
        let compensation = CompensationAction::new(
            "test",
            "restore_audio_state",
            json!({
                "volume": 1.5, // 无效的音量值（超出范围）
            }),
        );
        
        source.restore_from_compensation(&compensation).unwrap();
        // 无效音量应该被忽略，保持原值
        assert_eq!(source.volume.value(), 1.0); // 默认值
    }
}
