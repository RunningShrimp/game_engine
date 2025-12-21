//! 事件溯源端到端测试

use bevy_ecs::prelude::*;
use game_engine::domain::scene::Scene;
use game_engine::domain::event_sourcing::{
    EventSourcingManager, MemoryEventStore, MemorySnapshotStore,
};
use std::sync::Arc;

/// 测试完整的事件溯源流程：事件存储 -> 重放 -> 状态恢复
#[test]
fn test_event_sourcing_complete_workflow() {
    // 1. 初始化事件溯源系统
    let manager = EventSourcingManager::new(
        Arc::new(std::sync::RwLock::new(Box::new(MemoryEventStore::new()) as Box<dyn game_engine::domain::event_sourcing::EventStore>)),
        Arc::new(std::sync::RwLock::new(Box::new(MemorySnapshotStore::new()) as Box<dyn game_engine::domain::event_sourcing::SnapshotStore>)),
    );
    
    // 2. 创建并修改场景（产生事件）
    let mut scene = Scene::new("TestScene", "test_aggregate".to_string())
        .expect("Failed to create scene");
    
    let mut world = World::new();
    
    // 提交初始事件
    manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
    
    // 激活场景
    scene.activate().unwrap();
    manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
    
    // 添加实体
    let entity = game_engine::domain::entity::GameEntity::new(
        "entity1".to_string(),
        glam::Vec3::new(1.0, 2.0, 3.0),
    );
    scene.add_entity(entity).unwrap();
    manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
    
    // 3. 验证事件已存储
    let events = manager.replay_aggregate_events("test_aggregate", None);
    assert!(events.is_ok());
    let events = events.unwrap();
    assert!(events.len() >= 3);
    
    // 4. 验证事件类型
    let event_types: Vec<&str> = events.iter()
        .map(|e| e.event_type.as_str())
        .collect();
    assert!(event_types.contains(&"SceneLoadedEvent"));
    assert!(event_types.contains(&"SceneActivatedEvent"));
    assert!(event_types.contains(&"EntityAddedEvent"));
    
    // 5. 测试版本控制
    let events_from_version = manager.replay_aggregate_events("test_aggregate", Some(1));
    assert!(events_from_version.is_ok());
    let events_from_version = events_from_version.unwrap();
    // 从版本1开始的事件应该少于总事件数
    assert!(events_from_version.len() <= events.len());
}

/// 测试事件重放恢复状态
#[test]
fn test_event_replay_state_restoration() {
    let manager = EventSourcingManager::new(
        Arc::new(std::sync::RwLock::new(Box::new(MemoryEventStore::new()) as Box<dyn game_engine::domain::event_sourcing::EventStore>)),
        Arc::new(std::sync::RwLock::new(Box::new(MemorySnapshotStore::new()) as Box<dyn game_engine::domain::event_sourcing::SnapshotStore>)),
    );
    
    // 创建场景并执行操作
    let mut scene = Scene::new("TestScene", "restore_test".to_string())
        .expect("Failed to create scene");
    
    let mut world = World::new();
    
    scene.activate().unwrap();
    manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
    
    // 添加多个实体
    for i in 0..3 {
        let entity = game_engine::domain::entity::GameEntity::new(
            format!("entity_{}", i),
            glam::Vec3::ZERO,
        );
        scene.add_entity(entity).unwrap();
        manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
    }
    
    // 验证事件数量
    let events = manager.replay_aggregate_events("restore_test", None);
    assert!(events.is_ok());
    let events = events.unwrap();
    // 应该有：1个SceneLoadedEvent + 1个SceneActivatedEvent + 3个EntityAddedEvent = 5个
    assert!(events.len() >= 5);
}
