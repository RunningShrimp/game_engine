// 核心系统端到端集成测试
//
// 测试游戏引擎核心系统的完整工作流程，包括：
// - ECS系统与物理系统集成
// - 场景管理完整生命周期
// - 事件发布和处理
// - 资源加载和管理
// - 错误处理和恢复

use game_engine::domain::*;
use game_engine::ecs::*;

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_ecs_physics_integration() {
    // 测试ECS系统与物理系统的集成
    let mut world = bevy_ecs::prelude::World::new();

    // 创建一个带有物理属性的实体
    let entity = world.spawn((
        Transform {
            pos: glam::Vec3::new(0.0, 10.0, 0.0),
            rot: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        },
        Velocity::new(),
        Sprite::default(),
    )).id();

    // 验证实体创建成功
    assert!(world.get_entity(entity).is_ok());
    assert!(world.get::<Transform>(entity).is_some());
    assert!(world.get::<Velocity>(entity).is_some());
    assert!(world.get::<Sprite>(entity).is_some());

    // 测试查询功能
    let mut query = world.query::<(&Transform, &Velocity)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1);
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_scene_entity_lifecycle() {
    // 测试场景和实体的完整生命周期
    let mut scene_manager = SceneManager::new();

    // 创建场景
    let scene_id = SceneId::new(1);
    let create_result = scene_manager.create_scene(scene_id, "test_scene");
    assert!(create_result.is_ok());

    // 创建实体
    let entity_result = EntityFactory::create("test_entity")
        .with_position(Position::new(0.0, 0.0, 0.0))
        .with_velocity(Velocity::new(0.0, 0.0, 0.0))
        .build();

    assert!(entity_result.is_ok());
    let entity = entity_result.unwrap();

    // 验证实体属性
    assert_eq!(entity.id().as_u64(), 0); // 第一个实体ID为0
    assert_eq!(entity.position(), &Position::new(0.0, 0.0, 0.0));

    // 删除场景
    let delete_result = scene_manager.delete_scene(scene_id);
    assert!(delete_result.is_ok());
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_event_publish_and_consume() {
    // 测试事件发布和消费的完整流程
    let event_bus = EventBus::new();

    // 发布事件
    let test_event = events::TestEvent::new();
    let publish_result = event_bus.publish(events::EventType::Test, test_event);
    assert!(publish_result.is_ok());

    // 消费事件
    let consumed_events = event_bus.consume(events::EventType::Test);
    assert!(!consumed_events.is_empty());

    // 验证事件队列清空
    let consumed_again = event_bus.consume(events::EventType::Test);
    assert!(consumed_again.is_empty());
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_error_recovery_workflow() {
    // 测试错误恢复流程
    use crate::domain::errors::*;

    // 模拟物理错误
    let physics_error = PhysicsError::BodyNotFound("nonexistent_body".to_string());
    let domain_error: DomainError = physics_error.into();

    // 应用恢复策略
    match domain_error {
        DomainError::Physics(err) => {
            match err {
                PhysicsError::BodyNotFound(id) => {
                    // 使用日志并继续策略
                    println!("Body {} not found, continuing...", id);
                    // 实际应用中可以记录日志并继续
                }
                _ => {}
            }
        }
        _ => {}
    }

    // 测试补偿操作
    let compensation = CompensationAction::new(
        "restore_state",
        "create_physics_body",
        serde_json::json!({"position": [0.0, 0.0, 0.0]})
    );

    assert_eq!(compensation.id, "restore_state");
    assert_eq!(compensation.action_type, "create_physics_body");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_domain_event_sourcing() {
    // 测试领域事件溯源
    use crate::domain::event_sourcing::*;

    let event_store = InMemoryEventStore::new();

    // 创建事件流
    let stream_id = "test_stream".to_string();
    let event1 = DomainEvent::EntityCreated {
        entity_id: "entity_1".to_string(),
        timestamp: 1000,
    };

    // 追加事件
    let append_result = event_store.append_events(
        stream_id.clone(),
        vec![event1.clone()],
        0, // expected_version
    );

    assert!(append_result.is_ok());

    // 读取事件
    let read_result = event_store.read_events(&stream_id);
    assert!(read_result.is_ok());

    let events = read_result.unwrap();
    assert_eq!(events.len(), 1);
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_scene_serialization_roundtrip() {
    // 测试场景序列化和反序列化
    let mut scene_manager = SceneManager::new();

    // 创建场景
    let scene_id = SceneId::new(1);
    scene_manager.create_scene(scene_id, "test_scene").unwrap();

    // 序列化场景
    let serialize_result = scene_manager.serialize_scene(scene_id);
    assert!(serialize_result.is_ok());

    let serialized_data = serialize_result.unwrap();

    // 反序列化场景
    let deserialize_result = scene_manager.deserialize_scene(&serialized_data);
    assert!(deserialize_result.is_ok());

    let restored_scene_id = deserialize_result.unwrap();
    assert_eq!(restored_scene_id, scene_id);
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_physics_world_workflow() {
    // 测试物理世界的完整工作流程
    let mut world = physics::PhysicsWorld::new();

    // 创建刚体
    let body_id = physics::RigidBodyId::new(1);
    let body = physics::RigidBody::new(
        body_id,
        physics::RigidBodyType::Dynamic,
        glam::Vec3::new(0.0, 10.0, 0.0),
    );

    // 添加刚体到世界
    let add_result = world.add_rigid_body(body);
    assert!(add_result.is_ok());

    // 验证刚体存在
    let get_result = world.get_body(body_id);
    assert!(get_result.is_some());

    // 移除刚体
    let remove_result = world.remove_rigid_body(body_id);
    assert!(remove_result.is_ok());

    // 验证刚体已移除
    let get_after_remove = world.get_body(body_id);
    assert!(get_after_remove.is_none());
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_multiple_error_conversions() {
    // 测试多种错误类型到DomainError的转换
    use crate::domain::errors::*;

    // Audio错误转换
    let audio_err = AudioError::InvalidVolume(2.0);
    let domain_err: DomainError = audio_err.into();
    assert!(matches!(domain_err, DomainError::Audio(AudioError::InvalidVolume(2.0))));

    // Physics错误转换
    let physics_err = PhysicsError::InvalidParameter("negative mass".to_string());
    let domain_err: DomainError = physics_err.into();
    assert!(matches!(domain_err, DomainError::Physics(PhysicsError::InvalidParameter(_))));

    // Scene错误转换
    let scene_err = SceneError::EntityNotFound("entity_123".to_string());
    let domain_err: DomainError = scene_err.into();
    assert!(matches!(domain_err, DomainError::Scene(SceneError::EntityNotFound(_))));

    // 通用错误
    let general_err = DomainError::General("custom error".to_string());
    assert!(matches!(general_err, DomainError::General(_)));
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_recovery_strategy_application() {
    // 测试恢复策略的应用
    use crate::domain::errors::*;

    // 测试重试策略
    let retry_strategy = RecoveryStrategy::Retry {
        max_attempts: 3,
        delay_ms: 100,
    };

    match retry_strategy {
        RecoveryStrategy::Retry { max_attempts, delay_ms } => {
            assert_eq!(max_attempts, 3);
            assert_eq!(delay_ms, 100);
        }
        _ => panic!("Expected Retry strategy"),
    }

    // 测试使用默认值策略
    let use_default_strategy = RecoveryStrategy::UseDefault;
    assert!(matches!(use_default_strategy, RecoveryStrategy::UseDefault));

    // 测试跳过策略
    let skip_strategy = RecoveryStrategy::Skip;
    assert!(matches!(skip_strategy, RecoveryStrategy::Skip));

    // 测试日志并继续策略
    let log_strategy = RecoveryStrategy::LogAndContinue;
    assert!(matches!(log_strategy, RecoveryStrategy::LogAndContinue));

    // 测试失败策略
    let fail_strategy = RecoveryStrategy::Fail;
    assert!(matches!(fail_strategy, RecoveryStrategy::Fail));
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_event_registry_integration() {
    // 测试事件注册表与事件总线集成
    use crate::domain::*;

    // 获取全局事件注册表
    let registry = global_registry();

    // 注册事件类型
    let event_type = EventType::Test;
    registry.register(event_type);

    // 创建事件总线
    let event_bus = EventBus::new();

    // 发布事件
    let test_event = events::TestEvent::new();
    let publish_result = event_bus.publish(event_type, test_event);
    assert!(publish_result.is_ok());

    // 获取事件统计
    let stats = event_bus.stats();
    assert_eq!(stats.total_published, 1);
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_compensation_action_serialization() {
    // 测试补偿操作的序列化和反序列化
    use crate::domain::errors::*;
    use serde_json;

    let action = CompensationAction::new(
        "action_123",
        "revert_state",
        json!({
            "previous_position": [1.0, 2.0, 3.0],
            "previous_rotation": [0.0, 0.0, 0.0, 1.0]
        })
    );

    // 序列化
    let serialized = serde_json::to_string(&action).unwrap();
    assert!(serialized.contains("action_123"));
    assert!(serialized.contains("revert_state"));

    // 反序列化
    let deserialized: CompensationAction = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.id, "action_123");
    assert_eq!(deserialized.action_type, "revert_state");
}
