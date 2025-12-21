//! 领域事件示例
//!
//! 展示领域事件系统的使用，包括事件发布、订阅和处理。
//!
//! # 功能特性
//!
//! - 类型安全的事件系统
//! - 事件发布和订阅
//! - 批量事件处理
//! - 聚合根事件集成
//!
//! # 运行
//!
//! ```bash
//! cargo run --example domain_events
//! ```

use bevy_ecs::prelude::*;
use game_engine::domain::events::{DomainEvent, EventError, SafeEventBus};
use serde::{Deserialize, Serialize};

/// 示例领域事件：实体创建事件
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EntityCreatedEvent {
    entity_id: u64,
    entity_type: String,
}

impl DomainEvent for EntityCreatedEvent {
    fn event_type(&self) -> &'static str {
        "EntityCreated"
    }

    fn apply(&self, world: &mut World) -> Result<(), EventError> {
        println!("  Applying EntityCreated event: entity_id={}, type={}", 
                 self.entity_id, self.entity_type);
        // 在实际应用中，这里会创建ECS实体
        Ok(())
    }

    fn revert(&self, world: &mut World) -> Result<(), EventError> {
        println!("  Reverting EntityCreated event: entity_id={}", self.entity_id);
        // 在实际应用中，这里会删除ECS实体
        Ok(())
    }
}

/// 示例领域事件：实体删除事件
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EntityRemovedEvent {
    entity_id: u64,
}

impl DomainEvent for EntityRemovedEvent {
    fn event_type(&self) -> &'static str {
        "EntityRemoved"
    }

    fn apply(&self, world: &mut World) -> Result<(), EventError> {
        println!("  Applying EntityRemoved event: entity_id={}", self.entity_id);
        Ok(())
    }

    fn revert(&self, world: &mut World) -> Result<(), EventError> {
        println!("  Reverting EntityRemoved event: entity_id={}", self.entity_id);
        Ok(())
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    println!("=== Domain Events Example ===");
    println!();
    println!("This example demonstrates:");
    println!("- Type-safe event system");
    println!("- Event publishing and subscription");
    println!("- Batch event processing");
    println!("- Aggregate root event integration");
    println!();

    // 创建事件总线
    let event_bus = SafeEventBus::new();
    println!("Created SafeEventBus");
    println!();

    // 创建ECS世界
    let mut world = World::new();
    println!("Created ECS World");
    println!();

    // 订阅EntityCreated事件
    println!("Subscribing to EntityCreated events...");
    {
        let mut handler_state = 0u32;
        event_bus.subscribe::<EntityCreatedEvent>(move |event: &EntityCreatedEvent| {
            handler_state += 1;
            println!("  Handler received EntityCreated (call #{}): entity_id={}, type={}", 
                     handler_state, event.entity_id, event.entity_type);
        });
    }
    println!("  ✓ Subscribed to EntityCreated events");
    println!();

    // 订阅EntityRemoved事件
    println!("Subscribing to EntityRemoved events...");
    {
        let mut handler_state = 0u32;
        event_bus.subscribe::<EntityRemovedEvent>(move |event: &EntityRemovedEvent| {
            handler_state += 1;
            println!("  Handler received EntityRemoved (call #{}): entity_id={}", 
                     handler_state, event.entity_id);
        });
    }
    println!("  ✓ Subscribed to EntityRemoved events");
    println!();

    // 发布单个事件
    println!("Publishing single events...");
    let create_event = EntityCreatedEvent {
        entity_id: 1,
        entity_type: "Player".to_string(),
    };
    event_bus.publish(&create_event);
    println!("  ✓ Published EntityCreated event");
    println!();

    // 应用事件到世界
    println!("Applying events to world...");
    if let Err(e) = create_event.apply(&mut world) {
        eprintln!("  ✗ Error applying event: {}", e);
    } else {
        println!("  ✓ Event applied successfully");
    }
    println!();

    // 发布批量事件
    println!("Publishing batch events...");
    let batch_create_events = vec![
        EntityCreatedEvent {
            entity_id: 2,
            entity_type: "Enemy".to_string(),
        },
        EntityCreatedEvent {
            entity_id: 3,
            entity_type: "Item".to_string(),
        },
    ];
    
    event_bus.publish_batch(&batch_create_events);
    println!("  ✓ Published {} EntityCreated events in batch", batch_create_events.len());
    
    let remove_event = EntityRemovedEvent { entity_id: 1 };
    event_bus.publish(&remove_event);
    println!("  ✓ Published EntityRemoved event");
    println!();

    // 演示事件撤销
    println!("Demonstrating event revert...");
    if let Err(e) = create_event.revert(&mut world) {
        eprintln!("  ✗ Error reverting event: {}", e);
    } else {
        println!("  ✓ Event reverted successfully");
    }
    println!();

    println!("Example completed!");
    println!("Note: This demonstrates the event system API.");
    println!("      In a real application, events would be integrated with");
    println!("      aggregate roots and event sourcing.");
}

