//! 事件溯源示例
//!
//! 展示事件溯源系统的使用，包括：
//! - 事件存储和检索
//! - 事件重放
//! - 时间旅行调试
//! - 事件查询和统计

use game_engine::domain::event_sourcing::{
    EventSourcingManager, MemoryEventStore, MemorySnapshotStore,
};
use game_engine::domain::event_sourcing_enhanced::{
    EnhancedEventSourcingManager, EventQuery, EventStreamProcessor,
};
use game_engine::domain::events::{DomainEvent, EventError};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// 示例领域事件
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlayerMovedEvent {
    player_id: u32,
    from: [f32; 3],
    to: [f32; 3],
}

impl DomainEvent for PlayerMovedEvent {
    fn event_type(&self) -> &'static str {
        "PlayerMoved"
    }

    fn apply(&self, _world: &mut World) -> Result<(), EventError> {
        println!("Applying PlayerMoved: player {} moved from {:?} to {:?}", 
                 self.player_id, self.from, self.to);
        Ok(())
    }

    fn revert(&self, _world: &mut World) -> Result<(), EventError> {
        println!("Reverting PlayerMoved: player {} moved back from {:?} to {:?}", 
                 self.player_id, self.to, self.from);
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn main() -> Result<(), EventError> {
    tracing_subscriber::fmt::init();

    println!("=== Event Sourcing Example ===\n");

    // 1. 创建事件溯源管理器
    println!("1. Creating event sourcing manager...");
    let event_store: Arc<RwLock<Box<dyn game_engine::domain::event_sourcing::EventStore>>> =
        Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
    let snapshot_store: Arc<RwLock<Box<dyn game_engine::domain::event_sourcing::SnapshotStore>>> =
        Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));

    let manager = EventSourcingManager::new(event_store.clone(), snapshot_store.clone());
    let enhanced_manager = EnhancedEventSourcingManager::new(event_store, snapshot_store);

    // 2. 保存一些事件
    println!("2. Saving events...");
    let mut world = World::new();

    for i in 0..5 {
        let event = PlayerMovedEvent {
            player_id: 1,
            from: [i as f32, 0.0, 0.0],
            to: [(i + 1) as f32, 0.0, 0.0],
        };

        let event_id = manager.save_event(
            &event,
            Some("Player_1"),
            i + 1,
            &world,
        )?;

        println!("  Saved event {}: PlayerMoved", event_id.sequence);
    }

    // 3. 查询事件
    println!("\n3. Querying events...");
    let query = EventQuery::by_aggregate("Player_1")
        .with_limit(10)
        .with_offset(0);
    
    let events = enhanced_manager.query_events(query)?;
    println!("  Found {} events for Player_1", events.len());

    // 4. 获取事件统计
    println!("\n4. Getting event statistics...");
    let stats = enhanced_manager.get_event_stats(Some("Player_1"))?;
    println!("  Total events: {}", stats.total_events);
    println!("  Events by type: {:?}", stats.events_by_type);
    println!("  Events by aggregate: {:?}", stats.events_by_aggregate);

    // 5. 时间旅行：重放到指定版本
    println!("\n5. Time travel: replaying to version 3...");
    enhanced_manager.replay_to_version(&mut world, "Player_1", 3)?;
    println!("  Replayed events up to version 3");

    // 6. 事件流处理
    println!("\n6. Event stream processing...");
    let mut processor = EventStreamProcessor::new();
    
    // 添加过滤器：只保留PlayerMoved事件
    processor.add_filter(|e| e.event_type == "PlayerMoved");
    
    let all_events = manager.base().replay_aggregate_events("Player_1", None)?;
    let filtered = processor.process(all_events);
    println!("  Filtered {} events", filtered.len());

    // 7. 重放所有事件
    println!("\n7. Replaying all events...");
    let deserialized_events = manager.replay_and_deserialize_events("Player_1", None)?;
    println!("  Replaying {} events", deserialized_events.len());
    
    for event in deserialized_events {
        event.apply(&mut world)?;
    }

    println!("\n=== Example completed successfully! ===");
    Ok(())
}

