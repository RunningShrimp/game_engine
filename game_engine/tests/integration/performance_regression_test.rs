//! 性能回归测试
//!
//! 检测关键路径的性能回归，确保优化不会导致性能下降

use std::time::Instant;
use bevy_ecs::prelude::*;
use game_engine::domain::scene::Scene;
use game_engine::domain::event_sourcing::{
    EventSourcingManager, MemoryEventStore, MemorySnapshotStore,
};
use std::sync::Arc;

/// 性能基准：场景创建时间
const SCENE_CREATION_BENCHMARK_MS: u64 = 10;

/// 性能基准：事件提交时间
const EVENT_COMMIT_BENCHMARK_MS: u64 = 5;

/// 性能基准：事件重放时间（100个事件）
const EVENT_REPLAY_BENCHMARK_MS: u64 = 50;

/// 测试场景创建性能
#[test]
fn test_scene_creation_performance() {
    let start = Instant::now();
    
    for _ in 0..100 {
        let _scene = Scene::new("TestScene", format!("test_id_{}", 0))
            .expect("Failed to create scene");
    }
    
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() / 100;
    
    println!("Average scene creation time: {}ms", avg_ms);
    
    // 验证性能没有显著退化（允许20%的误差）
    assert!(
        avg_ms < (SCENE_CREATION_BENCHMARK_MS as u128 * 120 / 100),
        "Scene creation performance regression: {}ms > {}ms",
        avg_ms,
        SCENE_CREATION_BENCHMARK_MS * 120 / 100
    );
}

/// 测试事件提交性能
#[test]
fn test_event_commit_performance() {
    let manager = EventSourcingManager::new(
        Arc::new(std::sync::RwLock::new(Box::new(MemoryEventStore::new()) as Box<dyn game_engine::domain::event_sourcing::EventStore>)),
        Arc::new(std::sync::RwLock::new(Box::new(MemorySnapshotStore::new()) as Box<dyn game_engine::domain::event_sourcing::SnapshotStore>)),
    );
    
    let mut scene = Scene::new("TestScene", "perf_test".to_string())
        .expect("Failed to create scene");
    
    let mut world = World::new();
    
    let start = Instant::now();
    
    for _ in 0..100 {
        manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
    }
    
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() / 100;
    
    println!("Average event commit time: {}ms", avg_ms);
    
    // 验证性能没有显著退化
    assert!(
        avg_ms < (EVENT_COMMIT_BENCHMARK_MS as u128 * 120 / 100),
        "Event commit performance regression: {}ms > {}ms",
        avg_ms,
        EVENT_COMMIT_BENCHMARK_MS * 120 / 100
    );
}

/// 测试事件重放性能
#[test]
fn test_event_replay_performance() {
    let manager = EventSourcingManager::new(
        Arc::new(std::sync::RwLock::new(Box::new(MemoryEventStore::new()) as Box<dyn game_engine::domain::event_sourcing::EventStore>)),
        Arc::new(std::sync::RwLock::new(Box::new(MemorySnapshotStore::new()) as Box<dyn game_engine::domain::event_sourcing::SnapshotStore>)),
    );
    
    // 创建场景并提交100个事件
    let mut scene = Scene::new("TestScene", "replay_perf_test".to_string())
        .expect("Failed to create scene");
    
    let mut world = World::new();
    
    for _ in 0..100 {
        manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
    }
    
    // 测试重放性能
    let start = Instant::now();
    
    let events = manager.replay_aggregate_events("replay_perf_test", None);
    assert!(events.is_ok());
    
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();
    
    println!("Event replay time (100 events): {}ms", elapsed_ms);
    
    // 验证性能没有显著退化
    assert!(
        elapsed_ms < (EVENT_REPLAY_BENCHMARK_MS as u128 * 120 / 100),
        "Event replay performance regression: {}ms > {}ms",
        elapsed_ms,
        EVENT_REPLAY_BENCHMARK_MS * 120 / 100
    );
}

/// 测试批量操作性能
#[test]
fn test_batch_operations_performance() {
    let manager = EventSourcingManager::new(
        Arc::new(std::sync::RwLock::new(Box::new(MemoryEventStore::new()) as Box<dyn game_engine::domain::event_sourcing::EventStore>)),
        Arc::new(std::sync::RwLock::new(Box::new(MemorySnapshotStore::new()) as Box<dyn game_engine::domain::event_sourcing::SnapshotStore>)),
    );
    
    let start = Instant::now();
    
    // 创建并提交多个场景的事件
    for i in 0..10 {
        let mut scene = Scene::new(
            format!("Scene{}", i),
            format!("scene_{}", i),
        )
        .expect("Failed to create scene");
        
        let mut world = World::new();
        
        scene.activate().unwrap();
        manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
        
        // 添加实体
        for j in 0..10 {
            let entity = game_engine::domain::entity::GameEntity::new(
                format!("entity_{}_{}", i, j),
                glam::Vec3::ZERO,
            );
            scene.add_entity(entity).unwrap();
            manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
        }
    }
    
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();
    
    println!("Batch operations time (10 scenes, 10 entities each): {}ms", elapsed_ms);
    
    // 验证批量操作性能合理（允许较大的时间范围）
    assert!(
        elapsed_ms < 1000,
        "Batch operations took too long: {}ms",
        elapsed_ms
    );
}
