//! 场景生命周期端到端测试

use bevy_ecs::prelude::*;
use game_engine::domain::scene::Scene;
use game_engine::domain::event_sourcing::{
    EventSourcingManager, MemoryEventStore, MemorySnapshotStore,
};
use std::sync::Arc;

/// 测试场景完整生命周期：创建 -> 加载 -> 激活 -> 添加实体 -> 保存 -> 卸载
#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_scene_complete_lifecycle() {
    // 1. 创建事件溯源管理器
    let manager = EventSourcingManager::new(
        Arc::new(std::sync::RwLock::new(Box::new(MemoryEventStore::new()) as Box<dyn game_engine::domain::event_sourcing::EventStore>)),
        Arc::new(std::sync::RwLock::new(Box::new(MemorySnapshotStore::new()) as Box<dyn game_engine::domain::event_sourcing::SnapshotStore>)),
    );
    
    // 2. 创建场景
    let mut scene = Scene::new("MainScene", "main_scene_id".to_string())
        .expect("Failed to create scene");
    
    // 3. 提交创建事件
    let mut world = World::new();
    manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
    
    // 4. 加载场景
    let load_result = scene.load();
    assert!(load_result.is_ok());
    manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
    
    // 5. 激活场景
    let activate_result = scene.activate();
    assert!(activate_result.is_ok());
    manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
    
    // 6. 添加实体
    let entity = game_engine::domain::entity::GameEntity::new(
        "player".to_string(),
        glam::Vec3::ZERO,
    );
    let add_result = scene.add_entity(entity);
    assert!(add_result.is_ok());
    manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
    
    // 7. 验证场景状态
    assert_eq!(scene.entities().len(), 1);
    assert!(scene.is_active());
    
    // 8. 验证所有事件都已存储
    let events = manager.replay_aggregate_events("main_scene_id", None);
    assert!(events.is_ok());
    let events = events.unwrap();
    assert!(events.len() >= 3); // SceneLoadedEvent, SceneActivatedEvent, EntityAddedEvent
    
    // 9. 停用场景
    scene.deactivate();
    assert!(!scene.is_active());
}

/// 测试场景切换完整流程
#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_scene_switching_flow() {
    // 创建两个场景
    let mut scene1 = Scene::new("Scene1", "scene1_id".to_string())
        .expect("Failed to create scene1");
    let mut scene2 = Scene::new("Scene2", "scene2_id".to_string())
        .expect("Failed to create scene2");
    
    // 激活第一个场景
    scene1.activate().unwrap();
    assert!(scene1.is_active());
    assert!(!scene2.is_active());
    
    // 切换到第二个场景
    scene1.deactivate();
    scene2.activate().unwrap();
    
    assert!(!scene1.is_active());
    assert!(scene2.is_active());
}
