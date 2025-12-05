use super::super::event_sourcing::{self, DomainEvent, EventBus, EventError, EventSourcingManager, MemoryEventStore, MemorySnapshotStore};
use bevy_ecs::prelude::*;

// 测试领域事件
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerCreatedEvent {
    pub player_id: u32,
    pub player_name: String,
}

impl DomainEvent for PlayerCreatedEvent {
    fn event_type(&self) -> &'static str {
        "PlayerCreated"
    }

    fn apply(&self, _world: &mut World) -> Result<(), EventError> {
        println!("Applied PlayerCreatedEvent for player: {}", self.player_name);
        Ok(())
    }

    fn revert(&self, _world: &mut World) -> Result<(), EventError> {
        println!("Reverted PlayerCreatedEvent for player: {}", self.player_name);
        Ok(())
    }
}

// 测试事件总线功能
#[test]
fn test_event_bus_pub_sub() {
    // 创建事件总线
    let event_bus = EventBus::new();

    // 创建订阅者
    let mut received_events = Vec::new();
    
    event_bus.subscribe(move |event: &PlayerCreatedEvent| {
        received_events.push(event.clone());
        println!("Subscriber received PlayerCreatedEvent: {:?}", event);
    });

    // 创建并发布事件
    let player_event = PlayerCreatedEvent {
        player_id: 123,
        player_name: "TestPlayer".to_string(),
    };
    
    event_bus.publish(&player_event);

    // 验证事件被正确接收
    assert_eq!(received_events.len(), 1);
    assert_eq!(received_events[0].player_id, 123);
    assert_eq!(received_events[0].player_name, "TestPlayer".to_string());
    
    println!("✓ Event bus pub/sub test passed!");
}

// 测试事件溯源管理器功能
#[test]
fn test_event_sourcing_manager() {
    // 创建内存事件存储和快照存储
    let event_store = event_sourcing::MemoryEventStore::new();
    let snapshot_store = event_sourcing::MemorySnapshotStore::new();

    // 创建事件溯源管理器
    let manager = EventSourcingManager::new(
        std::sync::Arc::new(std::sync::Mutex::new(event_store)),
        std::sync::Arc::new(std::sync::Mutex::new(snapshot_store)),
    );

    // 创建世界
    let mut world = World::new();

    // 创建玩家创建事件
    let player_event = PlayerCreatedEvent {
        player_id: 123,
        player_name: "TestPlayer".to_string(),
    };

    // 记录事件
    let event_id = manager.record_event(player_event.clone(), Some(123)).unwrap();
    println!("✓ Event recorded with ID: {:?}", event_id);

    // 验证事件被存储
    let events = manager.get_event_history();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "PlayerCreated".to_string());
    
    println!("✓ Event sourcing manager test passed!");
}

// 测试事件应用
#[test]
fn test_event_application() {
    // 创建玩家创建事件
    let player_event = PlayerCreatedEvent {
        player_id: 123,
        player_name: "TestPlayer".to_string(),
    };

    // 创建世界
    let mut world = World::new();

    // 应用事件
    let result = player_event.apply(&mut world);
    assert!(result.is_ok());
    
    println!("✓ Event application test passed!");
}

// 测试事件撤销
#[test]
fn test_event_revert() {
    // 创建玩家创建事件
    let player_event = PlayerCreatedEvent {
        player_id: 123,
        player_name: "TestPlayer".to_string(),
    };

    // 创建世界
    let mut world = World::new();

    // 撤销事件
    let result = player_event.revert(&mut world);
    assert!(result.is_ok());
    
    println!("✓ Event revert test passed!");
}