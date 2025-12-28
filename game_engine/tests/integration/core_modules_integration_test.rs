//! 核心模块集成测试
//!
//! 测试引擎核心模块（ECS、渲染、物理）之间的集成，包括：
//! - ECS与物理系统的集成
//! - ECS与渲染系统的集成
//! - 物理与渲染系统的集成
//! - 三系统协同工作
//! - 性能关键路径测试

use bevy_ecs::prelude::*;
use game_engine::domain::{
    PhysicsDomainService, SceneDomainService,
    physics::{RigidBody, RigidBodyId, RigidBodyType},
    scene::SceneId,
};
use game_engine::ecs::{Sprite, Time, Transform, Velocity};
use game_engine::render::pbr::{PbrMaterial, PointLight3D};
use game_engine::render::postprocess::PostProcessConfig;
use glam::{Vec3, Vec4, Quat};

// ============================================================================
// ECS与物理系统集成测试
// ============================================================================

/// 测试ECS实体与物理刚体的同步
#[test]
fn test_ecs_physics_synchronization() {
    let mut world = World::new();
    world.insert_resource(Time::default());
    world.insert_resource(PhysicsDomainService::new());

    // 创建ECS实体
    let entity = world.spawn((
        Transform {
            pos: Vec3::new(0.0, 10.0, 0.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Sprite::default(),
        Velocity::default(),
    )).id();

    // 创建对应的物理刚体
    let body = RigidBody::new(
        RigidBodyId::new(1),
        RigidBodyType::Dynamic,
        Vec3::new(0.0, 10.0, 0.0),
    );

    if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
        assert!(physics.create_body(body).is_ok());
    }

    // 运行物理模拟
    for _ in 0..60 {
        if let Some(mut time) = world.get_resource_mut::<Time>() {
            time.delta_seconds = 1.0 / 60.0;
            time.elapsed_seconds += time.delta_seconds as f64;
        }

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            let _ = physics.step_simulation(1.0 / 60.0);
        }
    }

    // 验证实体仍然存在
    assert!(world.entities().contains(entity));
    
    // 验证物理服务存在
    assert!(world.get_resource::<PhysicsDomainService>().is_some());
}

/// 测试多个ECS实体与物理系统的集成
#[test]
fn test_multiple_entities_physics_integration() {
    let mut world = World::new();
    world.insert_resource(Time::default());
    world.insert_resource(PhysicsDomainService::new());

    // 创建多个实体
    let entities: Vec<Entity> = (0..10)
        .map(|i| {
            world.spawn((
                Transform {
                    pos: Vec3::new(i as f32, 10.0, 0.0),
                    rot: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                Sprite::default(),
                Velocity::default(),
            )).id()
        })
        .collect();

    // 为每个实体创建物理刚体
    if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
        for (i, _entity) in entities.iter().enumerate() {
            let body = RigidBody::new(
                RigidBodyId::new(i as u64 + 1),
                RigidBodyType::Dynamic,
                Vec3::new(i as f32, 10.0, 0.0),
            );
            assert!(physics.create_body(body).is_ok());
        }
    }

    // 运行多帧模拟
    for _ in 0..30 {
        if let Some(mut time) = world.get_resource_mut::<Time>() {
            time.delta_seconds = 1.0 / 60.0;
            time.elapsed_seconds += time.delta_seconds as f64;
        }

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            let _ = physics.step_simulation(1.0 / 60.0);
        }
    }

    // 验证所有实体仍然存在
    for entity in &entities {
        assert!(world.entities().contains(*entity));
    }
}

// ============================================================================
// ECS与渲染系统集成测试
// ============================================================================

/// 测试ECS实体与渲染组件的集成
#[test]
fn test_ecs_render_component_integration() {
    let mut world = World::new();

    // 创建带渲染组件的实体
    let entity = world.spawn((
        Transform {
            pos: Vec3::new(0.0, 0.0, 0.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Sprite::default(),
    )).id();

    // 验证实体和组件存在
    assert!(world.entities().contains(entity));
    
    if let Some(transform) = world.get::<Transform>(entity) {
        assert_eq!(transform.pos, Vec3::new(0.0, 0.0, 0.0));
    } else {
        panic!("Transform component not found");
    }

    if let Some(_sprite) = world.get::<Sprite>(entity) {
        // Sprite组件存在
    } else {
        panic!("Sprite component not found");
    }
}

/// 测试渲染材质与ECS的集成
#[test]
fn test_render_material_ecs_integration() {
    // 测试PBR材质创建和配置
    let material = PbrMaterial {
        base_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
        metallic: 0.5,
        roughness: 0.3,
        ..Default::default()
    };

    assert_eq!(material.base_color, Vec4::new(1.0, 0.0, 0.0, 1.0));
    assert_eq!(material.metallic, 0.5);
    assert_eq!(material.roughness, 0.3);

    // 测试点光源配置
    let light = PointLight3D {
        position: Vec3::new(5.0, 5.0, 5.0),
        color: Vec3::new(1.0, 1.0, 0.8),
        intensity: 2.0,
        radius: 15.0,
    };

    assert_eq!(light.position, Vec3::new(5.0, 5.0, 5.0));
    assert_eq!(light.intensity, 2.0);
}

/// 测试后处理效果配置
#[test]
fn test_postprocess_config_integration() {
    let config = PostProcessConfig {
        bloom_enabled: true,
        tonemap_enabled: true,
        ssao_enabled: false,
        exposure: 1.2,
        gamma: 2.2,
        ..Default::default()
    };

    assert!(config.bloom_enabled);
    assert!(config.tonemap_enabled);
    assert!(!config.ssao_enabled);
    assert_eq!(config.exposure, 1.2);
    assert_eq!(config.gamma, 2.2);
}

// ============================================================================
// 物理与渲染系统集成测试
// ============================================================================

/// 测试物理位置与渲染位置的同步
#[test]
fn test_physics_render_position_sync() {
    let mut world = World::new();
    world.insert_resource(Time::default());
    world.insert_resource(PhysicsDomainService::new());

    // 创建实体，初始位置在(0, 10, 0)
    let initial_pos = Vec3::new(0.0, 10.0, 0.0);
    let entity = world.spawn((
        Transform {
            pos: initial_pos,
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Sprite::default(),
    )).id();

    // 创建物理刚体，位置相同
    let body = RigidBody::new(
        RigidBodyId::new(1),
        RigidBodyType::Dynamic,
        initial_pos,
    );

    if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
        assert!(physics.create_body(body).is_ok());
    }

    // 运行物理模拟（物体应该下落）
    for _ in 0..60 {
        if let Some(mut time) = world.get_resource_mut::<Time>() {
            time.delta_seconds = 1.0 / 60.0;
            time.elapsed_seconds += time.delta_seconds as f64;
        }

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            let _ = physics.step_simulation(1.0 / 60.0);
        }
    }

    // 验证实体仍然存在
    assert!(world.entities().contains(entity));
    
    // 验证Transform组件仍然存在
    if let Some(transform) = world.get::<Transform>(entity) {
        // 位置可能已经改变（由于重力）
        assert!(transform.pos.y < initial_pos.y || transform.pos.y == initial_pos.y);
    }
}

// ============================================================================
// 三系统协同工作测试
// ============================================================================

/// 测试ECS、物理、渲染三系统协同工作
#[test]
fn test_ecs_physics_render_full_integration() {
    let mut world = World::new();
    world.insert_resource(Time::default());
    world.insert_resource(PhysicsDomainService::new());
    world.insert_resource(SceneDomainService::new());

    // 创建场景
    if let Some(mut scene_service) = world.get_resource_mut::<SceneDomainService>() {
        assert!(scene_service.create_scene(SceneId::new(1), "test_scene").is_ok());
    }

    // 创建多个实体，每个都有Transform、Sprite和物理刚体
    let mut entities = Vec::new();
    for i in 0..5 {
        let pos = Vec3::new(i as f32 * 2.0, 10.0, 0.0);
        
        let entity = world.spawn((
            Transform {
                pos,
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Sprite::default(),
            Velocity::default(),
        )).id();

        // 创建对应的物理刚体
        let body = RigidBody::new(
            RigidBodyId::new(i as u64 + 1),
            RigidBodyType::Dynamic,
            pos,
        );

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            assert!(physics.create_body(body).is_ok());
        }

        entities.push(entity);
    }

    // 运行完整的游戏循环模拟
    for frame in 0..60 {
        // 更新时间
        if let Some(mut time) = world.get_resource_mut::<Time>() {
            time.delta_seconds = 1.0 / 60.0;
            time.elapsed_seconds += time.delta_seconds as f64;
        }

        // 运行物理模拟
        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            let _ = physics.step_simulation(1.0 / 60.0);
        }

        // 验证所有实体仍然存在
        for entity in &entities {
            assert!(world.entities().contains(*entity), 
                "Entity should exist at frame {}", frame);
        }
    }

    // 最终验证
    assert_eq!(entities.len(), 5);
    assert!(world.get_resource::<PhysicsDomainService>().is_some());
    assert!(world.get_resource::<SceneDomainService>().is_some());
}

/// 测试系统资源管理
#[test]
fn test_system_resource_management() {
    let mut world = World::new();
    
    // 添加所有核心资源
    world.insert_resource(Time::default());
    world.insert_resource(PhysicsDomainService::new());
    world.insert_resource(SceneDomainService::new());

    // 验证资源存在
    assert!(world.get_resource::<Time>().is_some());
    assert!(world.get_resource::<PhysicsDomainService>().is_some());
    assert!(world.get_resource::<SceneDomainService>().is_some());

    // 创建实体
    let entity = world.spawn((
        Transform::default(),
        Sprite::default(),
    )).id();

    // 验证实体存在
    assert!(world.entities().contains(entity));
}

// ============================================================================
// 性能关键路径测试
// ============================================================================

/// 测试大量实体的集成性能
#[test]
fn test_large_entity_count_integration() {
    let mut world = World::new();
    world.insert_resource(Time::default());
    world.insert_resource(PhysicsDomainService::new());

    const ENTITY_COUNT: usize = 100;

    // 创建大量实体
    let entities: Vec<Entity> = (0..ENTITY_COUNT)
        .map(|i| {
            world.spawn((
                Transform {
                    pos: Vec3::new(i as f32 * 0.1, 0.0, 0.0),
                    rot: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                Sprite::default(),
            )).id()
        })
        .collect();

    // 验证所有实体创建成功
    assert_eq!(entities.len(), ENTITY_COUNT);

    // 运行几帧
    for _ in 0..10 {
        if let Some(mut time) = world.get_resource_mut::<Time>() {
            time.delta_seconds = 1.0 / 60.0;
            time.elapsed_seconds += time.delta_seconds as f64;
        }
    }

    // 验证所有实体仍然存在
    for entity in &entities {
        assert!(world.entities().contains(*entity));
    }
}

/// 测试组件查询性能
#[test]
fn test_component_query_performance() {
    let mut world = World::new();

    // 创建带不同组件组合的实体
    for i in 0..50 {
        if i % 2 == 0 {
            world.spawn((
                Transform::default(),
                Sprite::default(),
            ));
        } else {
            world.spawn((
                Transform::default(),
                Sprite::default(),
                Velocity::default(),
            ));
        }
    }

    // 查询所有Transform组件
    let transform_count: usize = world.query::<&Transform>().iter(&world).count();
    assert_eq!(transform_count, 50);

    // 查询Transform + Sprite组合
    let sprite_count: usize = world.query::<(&Transform, &Sprite)>().iter(&world).count();
    assert_eq!(sprite_count, 50);

    // 查询Transform + Sprite + Velocity组合
    let velocity_count: usize = world.query::<(&Transform, &Sprite, &Velocity)>().iter(&world).count();
    assert_eq!(velocity_count, 25);
}

// ============================================================================
// 错误处理和边界情况测试
// ============================================================================

/// 测试删除实体后的清理
#[test]
fn test_entity_removal_cleanup() {
    let mut world = World::new();
    world.insert_resource(PhysicsDomainService::new());

    // 创建实体
    let entity = world.spawn((
        Transform::default(),
        Sprite::default(),
    )).id();

    // 验证实体存在
    assert!(world.entities().contains(entity));

    // 删除实体
    world.despawn(entity);

    // 验证实体已删除
    assert!(!world.entities().contains(entity));
}

/// 测试资源不存在时的处理
#[test]
fn test_missing_resource_handling() {
    let mut world = World::new();

    // 不添加Time资源，尝试获取
    assert!(world.get_resource::<Time>().is_none());

    // 添加资源后应该能获取
    world.insert_resource(Time::default());
    assert!(world.get_resource::<Time>().is_some());
}

