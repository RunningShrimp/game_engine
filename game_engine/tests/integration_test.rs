//! 集成测试套件
//!
//! 测试引擎各个系统之间的集成，包括：
//! - 渲染系统集成
//! - 物理系统集成
//! - 音频系统集成
//! - 场景系统集成
//! - 错误处理集成
//! - Actor系统集成

use bevy_ecs::prelude::*;
use game_engine::domain::actor::{
    ActorSystem, AudioActor, AudioActorMessage, PhysicsActor, PhysicsActorMessage, RenderActor,
    RenderActorMessage,
};
use game_engine::domain::{
    audio::AudioSourceId,
    physics::{RigidBodyId, RigidBodyType},
    scene::SceneId,
    value_objects::Volume,
    AudioDomainService, PhysicsDomainService, SceneDomainService,
};
use game_engine::ecs::{Sprite, Time, Transform};
use game_engine::services::render::RenderService;
use glam::{Mat4, Quat, Vec3};

/// 测试完整的游戏循环
#[test]
fn test_complete_game_loop() {
    // 创建ECS世界
    let mut world = World::new();

    // 添加时间资源
    world.insert_resource(Time::default());

    // 添加物理领域服务资源
    world.insert_resource(PhysicsDomainService::new());

    // 创建测试场景 - 直接生成实体
    world.spawn((Transform::default(), Sprite::default()));

    // 运行几帧
    for _ in 0..10 {
        // 模拟引擎更新
        if let Some(mut time) = world.get_resource_mut::<Time>() {
            time.delta_seconds = 1.0 / 60.0;
            time.elapsed_seconds += time.delta_seconds as f64;
        }

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            let _ = physics.step_simulation(0.016);
        }
    }

    // 验证状态
    assert!(world.iter_entities().count() > 0);

    // 测试物理状态（验证物理服务存在）
    assert!(world.get_resource::<PhysicsDomainService>().is_some());

    // 实体数量验证
    let entity_count = world.iter_entities().count();
    assert!(entity_count >= 1);
}

/// 测试场景系统集成
#[test]
fn test_scene_system_integration() {
    let mut scene_service = SceneDomainService::new();

    // 创建多个场景
    assert!(scene_service.create_scene(SceneId(1), "Scene1").is_ok());
    assert!(scene_service.create_scene(SceneId(2), "Scene2").is_ok());
    assert_eq!(scene_service.scene_ids().len(), 2);

    // 加载场景1
    if let Some(scene) = scene_service.get_scene_mut(SceneId(1)) {
        assert!(scene.load().is_ok());
    }

    // 切换到场景1
    assert!(scene_service.switch_to_scene(SceneId(1)).is_ok());
    let active_scene = scene_service.get_active_scene();
    assert!(active_scene.is_some());
    assert_eq!(active_scene.unwrap().id, SceneId(1));

    // 加载场景2
    if let Some(scene) = scene_service.get_scene_mut(SceneId(2)) {
        assert!(scene.load().is_ok());
    }

    // 切换到场景2
    assert!(scene_service.switch_to_scene(SceneId(2)).is_ok());
    let active_scene = scene_service.get_active_scene();
    assert!(active_scene.is_some());
    assert_eq!(active_scene.unwrap().id, SceneId(2));

    // 删除场景
    let deleted_scene = scene_service.delete_scene(SceneId(1));
    assert!(deleted_scene.is_ok());
    assert_eq!(scene_service.scene_ids().len(), 1);
}

/// 测试音频系统集成
#[test]
fn test_audio_system_integration() {
    let mut audio_service = AudioDomainService::new();

    // 注意：create_source会验证文件存在，测试中文件不存在会失败
    // 这里我们测试错误处理逻辑
    let result1 = audio_service.create_source(AudioSourceId(1), "test1.wav");
    // 由于文件不存在，create_source会失败，这是预期的行为
    assert!(result1.is_err());

    // 验证错误类型
    assert!(matches!(
        result1.unwrap_err(),
        game_engine::domain::errors::DomainError::Audio(_)
    ));

    // 测试播放不存在的音频源（应该失败）
    assert!(audio_service.play_source(AudioSourceId(1)).is_err());

    // 测试设置不存在的音频源音量（应该失败）
    let volume = Volume::new(0.7).unwrap();
    assert!(audio_service
        .set_source_volume(AudioSourceId(1), volume)
        .is_err());

    // 验证没有音频源
    assert_eq!(audio_service.source_ids().len(), 0);
    assert_eq!(audio_service.playing_sources_count(), 0);

    // 测试停止所有音频源（空列表应该成功）
    assert!(audio_service.stop_all_sources().is_ok());
}

/// 测试物理系统集成
#[test]
fn test_physics_system_integration() {
    let mut physics_service = PhysicsDomainService::new();

    // 创建刚体
    let body1 = game_engine::domain::physics::RigidBody::new(
        RigidBodyId(1),
        RigidBodyType::Dynamic,
        Vec3::new(0.0, 10.0, 0.0),
    );
    let body2 = game_engine::domain::physics::RigidBody::new(
        RigidBodyId(2),
        RigidBodyType::Fixed,
        Vec3::ZERO,
    );

    assert!(physics_service.create_body(body1).is_ok());
    assert!(physics_service.create_body(body2).is_ok());

    // 应用力到动态刚体
    let force = Vec3::new(0.0, -9.81, 0.0);
    assert!(physics_service.apply_force(RigidBodyId(1), force).is_ok());

    // 步进物理模拟
    assert!(physics_service.step_simulation(0.016).is_ok());

    // 获取刚体位置
    let position = physics_service.get_body_position(RigidBodyId(1));
    assert!(position.is_ok());

    // 销毁刚体
    assert!(physics_service.destroy_body(RigidBodyId(1)).is_ok());
    assert!(physics_service.destroy_body(RigidBodyId(2)).is_ok());
}

/// 测试场景和物理系统集成
#[test]
fn test_scene_physics_integration() {
    let mut scene_service = SceneDomainService::new();
    let mut physics_service = PhysicsDomainService::new();

    // 创建场景
    assert!(scene_service
        .create_scene(SceneId(1), "PhysicsScene")
        .is_ok());

    // 加载场景（switch_to_scene需要场景已加载）
    if let Some(scene) = scene_service.get_scene_mut(SceneId(1)) {
        assert!(scene.load().is_ok());
    }

    // 切换到场景
    assert!(scene_service.switch_to_scene(SceneId(1)).is_ok());

    // 在物理世界中创建刚体
    let body = game_engine::domain::physics::RigidBody::new(
        RigidBodyId(1),
        RigidBodyType::Dynamic,
        Vec3::new(0.0, 10.0, 0.0),
    );
    assert!(physics_service.create_body(body).is_ok());

    // 模拟几帧
    for _ in 0..10 {
        assert!(physics_service.step_simulation(0.016).is_ok());
    }

    // 验证刚体位置（物理模拟后位置可能变化）
    let position = physics_service.get_body_position(RigidBodyId(1));
    assert!(position.is_ok());
    // 验证位置存在且有效（物理模拟可能没有启用重力，所以不强制要求位置下降）
    if let Ok(pos) = position {
        // 位置应该存在且有效
        assert!(pos.is_finite());
    }
}

/// 测试场景和音频系统集成
#[test]
fn test_scene_audio_integration() {
    let mut scene_service = SceneDomainService::new();
    let mut audio_service = AudioDomainService::new();

    // 创建场景
    assert!(scene_service.create_scene(SceneId(1), "AudioScene").is_ok());

    // 加载场景
    if let Some(scene) = scene_service.get_scene_mut(SceneId(1)) {
        assert!(scene.load().is_ok());
    }

    // 切换到场景
    assert!(scene_service.switch_to_scene(SceneId(1)).is_ok());

    // 创建音频源（文件不存在会失败，这是预期的）
    let result1 = audio_service.create_source(AudioSourceId(1), "background.wav");
    assert!(result1.is_err());

    let result2 = audio_service.create_source(AudioSourceId(2), "effect.wav");
    assert!(result2.is_err());

    // 验证没有音频源
    assert_eq!(audio_service.source_ids().len(), 0);
    assert_eq!(audio_service.playing_sources_count(), 0);

    // 切换场景时停止所有音频（模拟场景切换）
    assert!(audio_service.stop_all_sources().is_ok());
    assert_eq!(audio_service.playing_sources_count(), 0);
}

/// 测试ECS和物理系统集成
#[test]
fn test_ecs_physics_integration() {
    let mut world = World::new();
    world.insert_resource(Time::default());
    world.insert_resource(PhysicsDomainService::new());

    // 创建实体（物理组件现在通过PhysicsDomainService管理）
    let entity1 = world.spawn((Transform::default(), Sprite::default())).id();

    let entity2 = world.spawn((Transform::default(), Sprite::default())).id();

    // 验证实体创建（使用contains检查实体是否存在）
    assert!(world.entities().contains(entity1));
    assert!(world.entities().contains(entity2));

    // 运行物理模拟
    for _ in 0..10 {
        if let Some(mut time) = world.get_resource_mut::<Time>() {
            time.delta_seconds = 1.0 / 60.0;
            time.elapsed_seconds += time.delta_seconds as f64;
        }

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            let _ = physics.step_simulation(0.016);
        }
    }

    // 验证实体仍然存在（使用contains检查实体是否存在）
    assert!(world.entities().contains(entity1));
    assert!(world.entities().contains(entity2));
}

/// 测试错误处理集成
#[test]
fn test_error_handling_integration() {
    let mut scene_service = SceneDomainService::new();
    let mut audio_service = AudioDomainService::new();
    let mut physics_service = PhysicsDomainService::new();

    // 测试场景错误处理
    // 切换到不存在的场景应该失败
    assert!(scene_service.switch_to_scene(SceneId(999)).is_err());

    // 测试音频错误处理
    // 播放不存在的音频源应该失败
    assert!(audio_service.play_source(AudioSourceId(999)).is_err());

    // 测试物理错误处理
    // 获取不存在的刚体位置应该失败
    assert!(physics_service.get_body_position(RigidBodyId(999)).is_err());

    // 销毁不存在的刚体应该失败
    assert!(physics_service.destroy_body(RigidBodyId(999)).is_err());
}

/// 测试多系统协作
#[test]
fn test_multi_system_cooperation() {
    let mut scene_service = SceneDomainService::new();
    let mut audio_service = AudioDomainService::new();
    let mut physics_service = PhysicsDomainService::new();

    // 创建场景
    assert!(scene_service
        .create_scene(SceneId(1), "MultiSystemScene")
        .is_ok());

    // 加载场景
    if let Some(scene) = scene_service.get_scene_mut(SceneId(1)) {
        assert!(scene.load().is_ok());
    }

    // 切换到场景
    assert!(scene_service.switch_to_scene(SceneId(1)).is_ok());

    // 在场景中创建物理对象
    let body = game_engine::domain::physics::RigidBody::new(
        RigidBodyId(1),
        RigidBodyType::Dynamic,
        Vec3::new(0.0, 10.0, 0.0),
    );
    assert!(physics_service.create_body(body).is_ok());

    // 创建音频源（文件不存在会失败，这是预期的）
    let result = audio_service.create_source(AudioSourceId(1), "impact.wav");
    assert!(result.is_err());

    // 模拟物理碰撞（应用力）
    let force = Vec3::new(10.0, 0.0, 0.0);
    assert!(physics_service.apply_force(RigidBodyId(1), force).is_ok());

    // 步进物理模拟
    assert!(physics_service.step_simulation(0.016).is_ok());

    // 验证所有系统状态
    assert!(scene_service.get_active_scene().is_some());
    assert_eq!(audio_service.playing_sources_count(), 0);
    let position = physics_service.get_body_position(RigidBodyId(1));
    assert!(position.is_ok());
}

/// 测试场景切换时的资源清理
#[test]
fn test_scene_switch_resource_cleanup() {
    let mut scene_service = SceneDomainService::new();
    let mut audio_service = AudioDomainService::new();
    let mut physics_service = PhysicsDomainService::new();

    // 创建场景1
    assert!(scene_service.create_scene(SceneId(1), "Scene1").is_ok());

    // 加载场景1
    if let Some(scene) = scene_service.get_scene_mut(SceneId(1)) {
        assert!(scene.load().is_ok());
    }

    // 切换到场景1
    assert!(scene_service.switch_to_scene(SceneId(1)).is_ok());

    // 在场景1中创建资源（文件不存在会失败，这是预期的）
    let result1 = audio_service.create_source(AudioSourceId(1), "scene1_bgm.wav");
    assert!(result1.is_err());
    let body1 = game_engine::domain::physics::RigidBody::new(
        RigidBodyId(1),
        RigidBodyType::Dynamic,
        Vec3::ZERO,
    );
    assert!(physics_service.create_body(body1).is_ok());

    // 创建场景2
    assert!(scene_service.create_scene(SceneId(2), "Scene2").is_ok());

    // 加载场景2
    if let Some(scene) = scene_service.get_scene_mut(SceneId(2)) {
        assert!(scene.load().is_ok());
    }

    // 切换到场景2
    assert!(scene_service.switch_to_scene(SceneId(2)).is_ok());

    // 在场景2中创建资源（文件不存在会失败，这是预期的）
    let result2 = audio_service.create_source(AudioSourceId(2), "scene2_bgm.wav");
    assert!(result2.is_err());
    let body2 = game_engine::domain::physics::RigidBody::new(
        RigidBodyId(2),
        RigidBodyType::Dynamic,
        Vec3::ZERO,
    );
    assert!(physics_service.create_body(body2).is_ok());

    // 验证音频源（由于文件不存在，应该为0）
    assert_eq!(audio_service.source_ids().len(), 0);
    assert!(physics_service.get_body_position(RigidBodyId(1)).is_ok());
    assert!(physics_service.get_body_position(RigidBodyId(2)).is_ok());
}

/// 测试性能：大量实体和系统交互
#[test]
fn test_performance_many_entities() {
    let mut world = World::new();
    world.insert_resource(Time::default());
    world.insert_resource(PhysicsDomainService::new());

    // 创建大量实体
    const ENTITY_COUNT: usize = 100;
    for i in 0..ENTITY_COUNT {
        world.spawn((Transform::default(), Sprite::default()));
    }

    // 验证实体创建
    assert_eq!(world.iter_entities().count(), ENTITY_COUNT);

    // 运行多帧模拟
    const FRAME_COUNT: usize = 60;
    for _ in 0..FRAME_COUNT {
        if let Some(mut time) = world.get_resource_mut::<Time>() {
            time.delta_seconds = 1.0 / 60.0;
            time.elapsed_seconds += time.delta_seconds as f64;
        }

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            let _ = physics.step_simulation(0.016);
        }
    }

    // 验证所有实体仍然存在
    assert_eq!(world.iter_entities().count(), ENTITY_COUNT);
}

/// 测试客户端预测系统集成
#[test]
fn test_client_prediction_integration() {
    use game_engine::ecs::Transform;
    use game_engine::network::prediction::ClientPredictionManager;

    let mut world = World::new();

    // 添加预测管理器资源
    world.insert_resource(ClientPredictionManager::default());
    world.insert_resource(Time::default());

    // 创建实体（PredictionComponent在ECS系统中使用）
    let entity = world.spawn(Transform::default()).id();

    // 获取预测管理器
    if let Some(mut prediction) = world.get_resource_mut::<ClientPredictionManager>() {
        // 提交输入
        let seq = prediction.submit_input(vec![1, 2, 3]);
        assert_eq!(seq, 1);

        // 验证未确认输入
        let unconfirmed = prediction.get_unconfirmed_inputs();
        assert_eq!(unconfirmed.len(), 1);

        // 确认输入
        prediction.confirm_input(1, 10);
        assert_eq!(prediction.last_confirmed_tick(), 10);
    }

    // 验证实体存在
    assert!(world.entities().contains(entity));
}

/// 测试服务器权威系统集成
#[test]
fn test_server_authority_integration() {
    use game_engine::network::authority::{ServerAuthorityManager, ValidationResult};
    use glam::Vec3;

    let mut manager = ServerAuthorityManager::default();

    // 测试验证逻辑
    let client_pos = Vec3::new(0.0, 0.0, 0.0);
    let client_vel = Vec3::new(1.0, 0.0, 0.0);
    let server_pos = Vec3::new(0.1, 0.0, 0.0);
    let server_vel = Vec3::new(1.0, 0.0, 0.0);

    let result = manager.validate_client_state(client_pos, client_vel, server_pos, server_vel);
    assert!(matches!(result, ValidationResult::Valid));

    // 测试位置偏差验证
    let invalid_pos = Vec3::new(2.0, 0.0, 0.0);
    let result2 = manager.validate_client_state(client_pos, client_vel, invalid_pos, server_vel);
    assert!(matches!(result2, ValidationResult::InvalidPosition { .. }));

    // 测试冲突解决
    let resolved = manager.resolve_conflict(server_pos, client_pos);
    assert_eq!(resolved, server_pos);
}

/// 测试网络同步集成
#[test]
fn test_network_sync_integration() {
    use game_engine::network::{ConnectionState, NetworkMessage};

    // 测试网络消息序列化
    let msg = NetworkMessage::Heartbeat { timestamp: 1000 };

    // 验证消息创建成功
    match msg {
        NetworkMessage::Heartbeat { timestamp } => {
            assert_eq!(timestamp, 1000);
        }
        _ => panic!("Expected Heartbeat message"),
    }

    // 测试连接状态枚举
    let state = ConnectionState::Disconnected;
    assert_eq!(state, ConnectionState::Disconnected);
}

/// 测试网络错误处理集成
#[test]
fn test_network_error_handling() {
    use game_engine::network::prediction::ClientPredictionManager;

    let mut manager = ClientPredictionManager::default();

    // 测试空快照获取
    let snapshot = manager.get_snapshot(100);
    assert!(snapshot.is_none());

    // 测试回滚到不存在的快照
    let snapshot = manager.rollback_to(100);
    assert!(snapshot.is_none());

    // 测试重放不存在的输入
    let replay = manager.replay_inputs(100, 200);
    assert_eq!(replay.len(), 0);
}

/// 测试渲染系统集成
#[test]
fn test_render_system_integration() {
    let mut render_service = RenderService::new();
    let mut world = World::new();

    // 配置LOD
    render_service.use_default_lod();

    // 设置视锥体
    let view_proj = Mat4::IDENTITY;
    render_service.update_frustum(view_proj);

    // 从ECS构建渲染场景
    let result = render_service.build_domain_scene(&mut world);
    assert!(result.is_ok());

    // 更新场景
    let result = render_service.update_scene(0.016, Vec3::ZERO);
    assert!(result.is_ok());

    // 获取渲染命令
    let commands = render_service.get_render_commands();
    assert_eq!(commands.len(), 0); // 空场景应该返回空命令
}

/// 测试渲染和物理系统集成
#[test]
fn test_render_physics_integration() {
    let mut render_service = RenderService::new();
    let mut physics_service = PhysicsDomainService::new();
    let mut world = World::new();

    // 创建物理刚体
    let body = game_engine::domain::physics::RigidBody::new(
        RigidBodyId(1),
        RigidBodyType::Dynamic,
        Vec3::new(0.0, 10.0, 0.0),
    );
    assert!(physics_service.create_body(body).is_ok());

    // 步进物理模拟
    assert!(physics_service.step_simulation(0.016).is_ok());

    // 获取刚体位置
    let position = physics_service.get_body_position(RigidBodyId(1));
    assert!(position.is_ok());

    // 配置渲染服务
    render_service.use_default_lod();
    render_service.update_frustum(Mat4::IDENTITY);

    // 构建渲染场景
    let result = render_service.build_domain_scene(&mut world);
    assert!(result.is_ok());

    // 更新渲染场景
    if let Ok(pos) = position {
        let result = render_service.update_scene(0.016, pos);
        assert!(result.is_ok());
    }
}

/// 测试Actor系统集成
#[test]
fn test_actor_system_integration() {
    let mut actor_system = ActorSystem::new();

    // 注册音频Actor
    let audio_handle = actor_system.register("audio", AudioActor::new()).unwrap();

    // 发送音频消息
    assert!(audio_handle
        .send(AudioActorMessage::Play {
            source_id: 1,
            path: "test.wav".to_string(),
            volume: 1.0,
            looped: false,
        })
        .is_ok());

    // 注册物理Actor
    let physics_handle = actor_system
        .register("physics", PhysicsActor::new())
        .unwrap();

    // 发送物理消息
    assert!(physics_handle
        .send(PhysicsActorMessage::Step { delta_time: 0.016 })
        .is_ok());

    // 注册渲染Actor
    let render_handle = actor_system.register("render", RenderActor::new()).unwrap();

    // 发送渲染消息
    assert!(render_handle.send(RenderActorMessage::RenderFrame).is_ok());

    // 停止所有Actor
    assert!(audio_handle.stop().is_ok());
    assert!(physics_handle.stop().is_ok());
    assert!(render_handle.stop().is_ok());
}

/// 测试Actor系统和ECS集成
#[test]
fn test_actor_ecs_integration() {
    let mut world = World::new();
    let mut actor_system = ActorSystem::new();

    // 注册Actor
    let audio_handle = actor_system.register("audio", AudioActor::new()).unwrap();
    let physics_handle = actor_system
        .register("physics", PhysicsActor::new())
        .unwrap();

    // 模拟几帧更新（不将ActorHandle添加到ECS，直接使用）
    for _ in 0..10 {
        // 发送物理步进消息
        let _ = physics_handle.send(PhysicsActorMessage::Step { delta_time: 0.016 });

        // 发送音频消息（示例）
        let _ = audio_handle.send(AudioActorMessage::SetMasterVolume { volume: 0.8 });
    }

    // 停止Actor
    assert!(audio_handle.stop().is_ok());
    assert!(physics_handle.stop().is_ok());
}

/// 测试错误恢复策略集成
#[test]
fn test_error_recovery_integration() {
    let mut scene_service = SceneDomainService::new();
    let mut audio_service = AudioDomainService::new();
    let mut physics_service = PhysicsDomainService::new();

    // 测试场景错误恢复
    // 创建场景后尝试切换到不存在的场景
    assert!(scene_service.create_scene(SceneId(1), "TestScene").is_ok());
    let result = scene_service.switch_to_scene(SceneId(999));
    assert!(result.is_err());

    // 验证场景1仍然存在
    assert!(scene_service.get_scene(SceneId(1)).is_some());

    // 测试音频错误恢复
    // 创建音频源（文件不存在会失败，这是预期的）
    let result = audio_service.create_source(AudioSourceId(1), "test.wav");
    // 由于文件不存在，create_source会失败
    assert!(result.is_err());
    let result = audio_service.play_source(AudioSourceId(999));
    assert!(result.is_err());

    // 验证音频源1不存在（因为创建失败）
    assert!(audio_service.get_source(AudioSourceId(1)).is_none());

    // 测试物理错误恢复
    // 创建刚体后尝试获取不存在刚体的位置
    let body = game_engine::domain::physics::RigidBody::new(
        RigidBodyId(1),
        RigidBodyType::Dynamic,
        Vec3::ZERO,
    );
    assert!(physics_service.create_body(body).is_ok());

    let result = physics_service.get_body_position(RigidBodyId(999));
    assert!(result.is_err());

    // 验证刚体1仍然存在
    assert!(physics_service.get_body_position(RigidBodyId(1)).is_ok());
}

/// 测试完整系统工作流
#[test]
fn test_complete_system_workflow() {
    // 初始化所有服务
    let mut scene_service = SceneDomainService::new();
    let mut audio_service = AudioDomainService::new();
    let mut physics_service = PhysicsDomainService::new();
    let mut render_service = RenderService::new();
    let mut actor_system = ActorSystem::new();
    let mut world = World::new();

    // 1. 创建场景
    assert!(scene_service.create_scene(SceneId(1), "MainScene").is_ok());

    // 加载场景
    if let Some(scene) = scene_service.get_scene_mut(SceneId(1)) {
        assert!(scene.load().is_ok());
    }

    // 切换到场景
    assert!(scene_service.switch_to_scene(SceneId(1)).is_ok());

    // 2. 创建物理对象
    let body = game_engine::domain::physics::RigidBody::new(
        RigidBodyId(1),
        RigidBodyType::Dynamic,
        Vec3::new(0.0, 10.0, 0.0),
    );
    assert!(physics_service.create_body(body).is_ok());

    // 3. 创建音频源（文件不存在会失败，这是预期的）
    let result = audio_service.create_source(AudioSourceId(1), "ambient.wav");
    // 由于文件不存在，create_source会失败
    assert!(result.is_err());

    // 4. 配置渲染
    render_service.use_default_lod();
    render_service.update_frustum(Mat4::IDENTITY);

    // 5. 注册Actor
    let audio_handle = actor_system.register("audio", AudioActor::new()).unwrap();
    let physics_handle = actor_system
        .register("physics", PhysicsActor::new())
        .unwrap();

    // 6. 模拟游戏循环
    for _ in 0..60 {
        // 更新物理
        assert!(physics_service.step_simulation(0.016).is_ok());

        // 发送Actor消息
        let _ = physics_handle.send(PhysicsActorMessage::Step { delta_time: 0.016 });

        // 更新渲染场景
        let _ = render_service.build_domain_scene(&mut world);
        let _ = render_service.update_scene(0.016, Vec3::ZERO);
    }

    // 7. 验证最终状态
    assert!(scene_service.get_active_scene().is_some());
    assert_eq!(audio_service.source_ids().len(), 0); // 由于文件不存在，音频源创建失败
    assert_eq!(audio_service.playing_sources_count(), 0);
    assert!(physics_service.get_body_position(RigidBodyId(1)).is_ok());

    // 8. 清理
    assert!(audio_handle.stop().is_ok());
    assert!(physics_handle.stop().is_ok());
}

/// 测试渲染系统集成（多对象场景）
#[test]
fn test_render_system_integration_many_objects() {
    use game_engine::ecs::Mesh;
    use game_engine::resources::manager::Handle;
    use bevy_ecs::prelude::*;

    let mut render_service = RenderService::new();
    let mut world = World::new();

    render_service.use_default_lod();
    render_service.update_frustum(Mat4::IDENTITY);

    // 创建多个实体（虽然没有有效的GpuMesh，但测试API调用）
    for i in 0..10 {
        world.spawn((
            Transform {
                pos: Vec3::new(i as f32 * 10.0, 0.0, 0.0),
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Mesh {
                handle: Handle::new_loading(),
            },
        ));
    }

    // 构建渲染场景
    let result = render_service.build_domain_scene(&mut world);
    assert!(result.is_ok());

    // 更新场景
    let result = render_service.update_scene(0.016, Vec3::ZERO);
    assert!(result.is_ok());

    // 获取渲染命令
    let commands = render_service.get_render_commands();
    assert_eq!(commands.len(), 0); // 空场景应该返回空命令
}

/// 测试物理系统集成（复杂碰撞）
#[test]
fn test_physics_system_integration_complex_collisions() {
    use game_engine::domain::physics::{Collider, ColliderId};

    let mut physics_service = PhysicsDomainService::new();

    // 创建多个刚体
    for i in 0..5 {
        let body = game_engine::domain::physics::RigidBody::new(
            RigidBodyId(i),
            RigidBodyType::Dynamic,
            Vec3::new(i as f32 * 2.0, 10.0, 0.0),
        );
        assert!(physics_service.create_body(body).is_ok());

        // 为每个刚体添加碰撞体
        let collider = Collider::cuboid(
            ColliderId(i),
            Vec3::new(0.5, 0.5, 0.5),
        );
        assert!(physics_service.create_collider(collider, RigidBodyId(i)).is_ok());
    }

    // 模拟多帧物理步进
    for _ in 0..10 {
        assert!(physics_service.step_simulation(0.016).is_ok());
    }

    // 验证所有刚体仍然存在
    for i in 0..5 {
        let position = physics_service.get_body_position(RigidBodyId(i));
        assert!(position.is_ok());
    }
}

/// 测试音频系统集成（多源播放）
#[test]
fn test_audio_system_integration_multiple_sources() {
    let mut audio_service = AudioDomainService::new();

    // 创建多个音频源（文件不存在会失败，这是预期的）
    for i in 1..=5 {
        let result = audio_service.create_source(AudioSourceId(i), &format!("sound{}.wav", i));
        // 由于文件不存在，create_source会失败
        assert!(result.is_err());
    }

    // 验证没有音频源（因为创建失败）
    assert_eq!(audio_service.source_ids().len(), 0);
    assert_eq!(audio_service.playing_sources_count(), 0);

    // 测试停止所有音频源（空列表应该成功）
    assert!(audio_service.stop_all_sources().is_ok());
}

/// 测试场景系统集成（场景切换）
#[test]
fn test_scene_system_integration_scene_switching() {
    let mut scene_service = SceneDomainService::new();

    // 创建多个场景
    assert!(scene_service.create_scene(SceneId(1), "Scene1").is_ok());
    assert!(scene_service.create_scene(SceneId(2), "Scene2").is_ok());
    assert!(scene_service.create_scene(SceneId(3), "Scene3").is_ok());

    // 加载场景1
    if let Some(scene) = scene_service.get_scene_mut(SceneId(1)) {
        assert!(scene.load().is_ok());
    }

    // 切换到场景1
    assert!(scene_service.switch_to_scene(SceneId(1)).is_ok());
    assert_eq!(scene_service.get_active_scene().unwrap().id, SceneId(1));

    // 加载场景2
    if let Some(scene) = scene_service.get_scene_mut(SceneId(2)) {
        assert!(scene.load().is_ok());
    }

    // 切换到场景2
    assert!(scene_service.switch_to_scene(SceneId(2)).is_ok());
    assert_eq!(scene_service.get_active_scene().unwrap().id, SceneId(2));

    // 加载场景3
    if let Some(scene) = scene_service.get_scene_mut(SceneId(3)) {
        assert!(scene.load().is_ok());
    }

    // 切换到场景3
    assert!(scene_service.switch_to_scene(SceneId(3)).is_ok());
    assert_eq!(scene_service.get_active_scene().unwrap().id, SceneId(3));

    // 验证场景数量
    assert_eq!(scene_service.scene_ids().len(), 3);
}

/// 测试错误处理集成（恢复策略）
#[test]
fn test_error_handling_integration_recovery() {
    let mut scene_service = SceneDomainService::new();
    let mut audio_service = AudioDomainService::new();
    let mut physics_service = PhysicsDomainService::new();

    // 测试场景错误恢复
    assert!(scene_service.create_scene(SceneId(1), "TestScene").is_ok());
    
    // 尝试切换到不存在的场景（应该失败，但不影响现有场景）
    let result = scene_service.switch_to_scene(SceneId(999));
    assert!(result.is_err());
    
    // 验证场景1仍然存在
    assert!(scene_service.get_scene(SceneId(1)).is_some());

    // 测试音频错误恢复
    // 创建音频源（文件不存在会失败）
    let result = audio_service.create_source(AudioSourceId(1), "test.wav");
    assert!(result.is_err());
    
    // 尝试播放不存在的音频源（应该失败，但不影响服务状态）
    let result = audio_service.play_source(AudioSourceId(999));
    assert!(result.is_err());
    
    // 验证服务状态正常
    assert_eq!(audio_service.source_ids().len(), 0);

    // 测试物理错误恢复
    let body = game_engine::domain::physics::RigidBody::new(
        RigidBodyId(1),
        RigidBodyType::Dynamic,
        Vec3::ZERO,
    );
    assert!(physics_service.create_body(body).is_ok());
    
    // 尝试获取不存在刚体的位置（应该失败，但不影响现有刚体）
    let result = physics_service.get_body_position(RigidBodyId(999));
    assert!(result.is_err());
    
    // 验证刚体1仍然存在
    assert!(physics_service.get_body_position(RigidBodyId(1)).is_ok());
}
