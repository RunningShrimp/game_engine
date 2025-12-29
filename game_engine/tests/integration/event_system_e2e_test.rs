//! 事件系统端到端测试

use bevy_ecs::prelude::*;
use game_engine::domain::events::{SafeEventBus, DomainEvent, AggregateRoot};
use game_engine::domain::scene::{Scene, SceneLoadedEvent, SceneActivatedEvent, EntityAddedEvent};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestCounterEvent {
    value: u32,
}

impl DomainEvent for TestCounterEvent {
    fn event_type(&self) -> &'static str {
        "TestCounterEvent"
    }

    fn apply(&self, _world: &mut World) -> Result<(), game_engine::domain::events::EventError> {
        Ok(())
    }

    fn revert(&self, _world: &mut World) -> Result<(), game_engine::domain::events::EventError> {
        Ok(())
    }
}

/// 测试事件系统的完整流程：订阅 -> 发布 -> 处理
#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_event_system_end_to_end() {
    let bus = Arc::new(SafeEventBus::new());
    let counter = Arc::new(Mutex::new(0u32));
    
    // 1. 订阅事件
    let counter_clone = Arc::clone(&counter);
    bus.subscribe::<TestCounterEvent>(move |event: &TestCounterEvent| {
        let mut guard = counter_clone.lock().unwrap();
        *guard += event.value;
    });
    
    // 2. 发布多个事件
    for i in 1..=10 {
        let event = TestCounterEvent { value: i };
        bus.publish(&event);
    }
    
    // 3. 等待事件处理（简单延迟）
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    // 4. 验证计数器值（1+2+...+10 = 55）
    let guard = counter.lock().unwrap();
    assert_eq!(*guard, 55);
}

/// 测试场景聚合根的事件发布流程
#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_scene_aggregate_event_flow() {
    // 1. 创建场景聚合根
    let mut scene = Scene::new("TestScene", "test_scene_id".to_string())
        .expect("Failed to create scene");
    
    // 2. 验证初始状态（应该有SceneLoadedEvent）
    let events = scene.uncommitted_events();
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| e.event_type() == "SceneLoadedEvent"));
    
    // 3. 激活场景
    let activate_result = scene.activate();
    assert!(activate_result.is_ok());
    
    // 4. 验证激活事件已发布
    let events = scene.uncommitted_events();
    assert!(events.iter().any(|e| e.event_type() == "SceneActivatedEvent"));
    
    // 5. 添加实体
    let entity = game_engine::domain::entity::GameEntity::new(
        "entity1".to_string(),
        Vec3::ZERO,
    );
    let add_result = scene.add_entity(entity);
    assert!(add_result.is_ok());
    
    // 6. 验证实体添加事件已发布
    let events = scene.uncommitted_events();
    assert!(events.iter().any(|e| e.event_type() == "EntityAddedEvent"));
    
    // 7. 清除未提交事件
    scene.clear_uncommitted_events();
    assert!(scene.uncommitted_events().is_empty());
}

/// 测试批量事件发布
#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_batch_event_publishing() {
    let bus = Arc::new(SafeEventBus::new());
    let counter = Arc::new(Mutex::new(0u32));
    
    // 订阅
    let counter_clone = Arc::clone(&counter);
    bus.subscribe::<TestCounterEvent>(move |event: &TestCounterEvent| {
        let mut guard = counter_clone.lock().unwrap();
        *guard += event.value;
    });
    
    // 批量发布
    let events: Vec<TestCounterEvent> = (1..=5)
        .map(|i| TestCounterEvent { value: i })
        .collect();
    
    bus.publish_batch(&events);
    
    // 等待处理
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    // 验证（1+2+3+4+5 = 15）
    let guard = counter.lock().unwrap();
    assert_eq!(*guard, 15);
}





