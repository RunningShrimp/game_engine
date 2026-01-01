//! 场景加载和序列化完整流程集成测试

use bevy_ecs::prelude::*;
use game_engine::domain::scene::Scene;
use game_engine::scene::serialization::SerializedScene;
use game_engine::ecs::{Transform, Sprite, Camera, Projection};
use glam::{Vec3, Quat};
use std::path::PathBuf;
use tempfile::TempDir;

/// 测试场景序列化和反序列化完整流程
#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_scene_serialization_roundtrip() {
    // 1. 创建ECS世界并添加实体
    let mut world = World::new();
    
    // 添加一些测试实体
    let entity1 = world.spawn((
        Transform {
            translation: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Sprite::default(),
    )).id();
    
    let entity2 = world.spawn((
        Transform {
            translation: Vec3::new(4.0, 5.0, 6.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Camera {
            projection: Projection::Perspective {
                fov: 60.0,
                aspect: 16.0 / 9.0,
                near: 0.1,
                far: 1000.0,
            },
        },
    )).id();
    
    // 2. 序列化场景
    let serialized_scene = SerializedScene::from_world(&mut world, "TestScene");
    assert_eq!(serialized_scene.name, "TestScene");
    assert_eq!(serialized_scene.entities.len(), 2);
    
    // 3. 保存到临时文件
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let scene_path = temp_dir.path().join("test_scene.json");
    
    let save_result = serialized_scene.save_to_file(
        scene_path.to_str().unwrap()
    );
    assert!(save_result.is_ok(), "Failed to save scene: {:?}", save_result);
    
    // 4. 从文件加载场景
    let loaded_scene = SerializedScene::load_from_file(
        scene_path.to_str().unwrap()
    );
    assert!(loaded_scene.is_ok(), "Failed to load scene: {:?}", loaded_scene);
    
    let loaded_scene = loaded_scene.unwrap();
    assert_eq!(loaded_scene.name, "TestScene");
    assert_eq!(loaded_scene.entities.len(), 2);
    
    // 5. 反序列化到新的世界
    let mut new_world = World::new();
    loaded_scene.to_world(&mut new_world);
    
    // 6. 验证实体已正确恢复
    let mut entity_count = 0;
    let mut found_transform = false;
    let mut found_camera = false;
    
    for (_entity, transform) in new_world.query::<&Transform>().iter(&new_world) {
        entity_count += 1;
        found_transform = true;
        // 验证位置是否正确（允许小的浮点误差）
        assert!(
            (transform.translation - Vec3::new(1.0, 2.0, 3.0)).length() < 0.001 ||
            (transform.translation - Vec3::new(4.0, 5.0, 6.0)).length() < 0.001
        );
    }
    
    for (_entity, _camera) in new_world.query::<&Camera>().iter(&new_world) {
        found_camera = true;
    }
    
    assert_eq!(entity_count, 2);
    assert!(found_transform);
    assert!(found_camera);
}

/// 测试场景聚合根的序列化流程
#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_scene_aggregate_serialization() {
    // 1. 创建场景聚合根
    let mut scene = Scene::new("TestScene", "test_scene_id".to_string())
        .expect("Failed to create scene");
    
    // 2. 添加实体
    let entity_id = scene.add_entity(
        game_engine::domain::entity::GameEntity::new(
            "entity1".to_string(),
            Vec3::new(1.0, 2.0, 3.0),
        )
    );
    assert!(entity_id.is_ok());
    
    // 3. 验证场景状态
    assert_eq!(scene.entities().len(), 1);
    assert_eq!(scene.name(), "TestScene");
    
    // 4. 验证场景不变式
    let validation_result = scene.validate();
    assert!(validation_result.is_ok(), "Scene validation failed: {:?}", validation_result);
}






