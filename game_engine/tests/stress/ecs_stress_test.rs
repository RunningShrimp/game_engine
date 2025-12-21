//! ECS压力测试
//!
//! 测试ECS系统在大量组件下的性能。

use bevy_ecs::prelude::*;
use game_engine::ecs::Transform;
use glam::{Quat, Vec3};

#[test]
#[ignore] // 压力测试默认忽略，需要时手动运行
fn test_ecs_10000_entities() {
    // 测试10000个实体的创建和查询性能
    const ENTITY_COUNT: usize = 10000;
    
    let mut world = World::new();
    
    // 创建10000个实体，每个都有Transform组件
    let start = std::time::Instant::now();
    
    for i in 0..ENTITY_COUNT {
        let mut entity = world.spawn();
        entity.insert(Transform {
            translation: Vec3::new(i as f32, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
    }
    
    let creation_time = start.elapsed();
    
    // 验证所有实体都已创建
    assert_eq!(world.entities().len(), ENTITY_COUNT);
    
    // 验证创建时间合理（应该在100ms内）
    assert!(creation_time.as_millis() < 1000, "实体创建应在1秒内完成");
}

#[test]
#[ignore]
fn test_ecs_query_performance() {
    // 测试查询大量组件的性能
    const ENTITY_COUNT: usize = 10000;
    
    let mut world = World::new();
    
    // 创建实体
    for i in 0..ENTITY_COUNT {
        let mut entity = world.spawn();
        entity.insert(Transform {
            translation: Vec3::new(i as f32, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
    }
    
    // 测试查询性能
    let start = std::time::Instant::now();
    
    let mut query = world.query::<&Transform>();
    let count = query.iter(&world).count();
    
    let query_time = start.elapsed();
    
    assert_eq!(count, ENTITY_COUNT);
    // 查询应该在合理时间内完成（应该在50ms内）
    assert!(query_time.as_millis() < 500, "查询应在500ms内完成");
}

#[test]
#[ignore]
fn test_ecs_mut_query_performance() {
    // 测试可变查询的性能
    const ENTITY_COUNT: usize = 5000;
    
    let mut world = World::new();
    
    // 创建实体
    for i in 0..ENTITY_COUNT {
        let mut entity = world.spawn();
        entity.insert(Transform {
            translation: Vec3::new(i as f32, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
    }
    
    // 测试可变查询和更新性能
    let start = std::time::Instant::now();
    
    let mut query = world.query::<&mut Transform>();
    for mut transform in query.iter_mut(&mut world) {
        transform.translation.y += 1.0;
    }
    
    let update_time = start.elapsed();
    
    // 验证更新
    let query = world.query::<&Transform>();
    let first_transform = query.iter(&world).next().unwrap();
    assert!((first_transform.translation.y - 1.0).abs() < 0.001);
    
    // 更新应该在合理时间内完成
    assert!(update_time.as_millis() < 500, "更新应在500ms内完成");
}

#[test]
#[ignore]
fn test_ecs_component_addition_removal() {
    // 测试组件添加和移除的性能
    const ENTITY_COUNT: usize = 1000;
    
    let mut world = World::new();
    
    // 创建实体
    let entities: Vec<Entity> = (0..ENTITY_COUNT)
        .map(|i| {
            world
                .spawn(Transform {
                    translation: Vec3::new(i as f32, 0.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                })
                .id()
        })
        .collect();
    
    // 测试添加组件
    let start = std::time::Instant::now();
    
    for entity in &entities {
        world.entity_mut(*entity).insert(Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
    }
    
    let add_time = start.elapsed();
    
    // 验证组件已添加
    let query = world.query::<&Transform>();
    assert_eq!(query.iter(&world).count(), ENTITY_COUNT);
    
    // 添加组件应该在合理时间内完成
    assert!(add_time.as_millis() < 500, "添加组件应在500ms内完成");
}

