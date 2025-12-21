# 领域事件系统使用指南

## 概述

领域事件系统提供了类型安全、无`downcast_ref`的事件分发机制，用于实现领域驱动设计（DDD）中的事件驱动架构。

## 核心概念

### DomainEvent Trait

所有领域事件都必须实现`DomainEvent` trait：

```rust
use game_engine::domain::events::DomainEvent;
use bevy_ecs::prelude::*;

pub struct EntityCreatedEvent {
    pub entity_id: String,
    pub position: Vec3,
}

impl DomainEvent for EntityCreatedEvent {
    fn event_type(&self) -> &'static str {
        "EntityCreatedEvent"
    }

    fn apply(&self, world: &mut World) -> Result<(), EventError> {
        // 应用事件到世界状态
        // 例如：创建实体、更新组件等
        Ok(())
    }

    fn revert(&self, world: &mut World) -> Result<(), EventError> {
        // 撤销事件（反向操作）
        // 例如：删除实体、恢复状态等
        Ok(())
    }
}
```

### SafeEventBus

`SafeEventBus`是类型安全的事件总线，支持事件订阅和发布：

```rust
use game_engine::domain::events::SafeEventBus;
use std::sync::Arc;

// 创建事件总线
let bus = Arc::new(SafeEventBus::new());

// 订阅事件
bus.subscribe::<EntityCreatedEvent>(move |event: &EntityCreatedEvent| {
    println!("Entity created: {}", event.entity_id);
});

// 发布事件
let event = EntityCreatedEvent {
    entity_id: "entity_1".to_string(),
    position: Vec3::ZERO,
};
bus.publish(&event);

// 批量发布事件
let events = vec![event1, event2, event3];
bus.publish_batch(&events);
```

## 聚合根集成

### AggregateRoot Trait

聚合根应该实现`AggregateRoot` trait以支持领域事件：

```rust
use game_engine::domain::events::{AggregateRoot, DomainEvent};
use game_engine::domain::events::AggregateEventQueue;

pub struct Scene {
    id: String,
    name: String,
    event_queue: AggregateEventQueue,
    // ... 其他字段
}

impl AggregateRoot for Scene {
    fn aggregate_id(&self) -> String {
        self.id.clone()
    }

    fn uncommitted_event_count(&self) -> usize {
        self.event_queue.uncommitted_count()
    }

    fn take_uncommitted_events(&mut self) -> Vec<Box<dyn DomainEvent>> {
        self.event_queue.take_uncommitted_events()
    }

    fn clear_uncommitted_events(&mut self) {
        self.event_queue.clear_uncommitted_events();
    }
}
```

### 发布领域事件

在聚合根方法中发布领域事件：

```rust
impl Scene {
    pub fn activate(&mut self) -> Result<(), SceneError> {
        // 业务逻辑
        self.state = SceneState::Active;
        
        // 发布领域事件
        self.event_queue.add_event(SceneActivatedEvent {
            scene_id: self.id.clone(),
            timestamp: SystemTime::now(),
        });
        
        Ok(())
    }
}
```

## 最佳实践

### 1. 事件命名

- 使用过去时态：`EntityCreated`、`SceneActivated`、`EntityRemoved`
- 明确表达发生了什么，而不是将要发生什么

### 2. 事件不可变性

- 事件应该是不可变的（immutable）
- 事件一旦发布，就不应该被修改

### 3. 事件粒度

- 事件应该表示一个完整的业务操作
- 避免过于细粒度的事件（如每个字段的变更）
- 避免过于粗粒度的事件（如整个聚合的状态）

### 4. 事件处理

- 事件处理应该是幂等的（idempotent）
- 事件处理应该快速，避免长时间阻塞
- 对于耗时操作，考虑异步处理

### 5. 错误处理

- 事件应用失败应该返回`EventError`
- 事件撤销失败也应该返回`EventError`
- 考虑实现补偿操作（compensation）

## 示例：完整场景

```rust
use game_engine::domain::events::{SafeEventBus, DomainEvent, AggregateRoot};
use game_engine::domain::scene::Scene;
use std::sync::Arc;

// 1. 创建事件总线
let bus = Arc::new(SafeEventBus::new());

// 2. 订阅场景激活事件
bus.subscribe::<SceneActivatedEvent>(move |event: &SceneActivatedEvent| {
    println!("Scene activated: {}", event.scene_id);
    // 执行副作用：更新UI、播放音效等
});

// 3. 创建场景聚合根
let mut scene = Scene::new("MainScene", "scene_1".to_string())
    .expect("Failed to create scene");

// 4. 激活场景（内部会发布SceneActivatedEvent）
scene.activate().unwrap();

// 5. 提交事件到事件总线
let events = scene.take_uncommitted_events();
for event in events {
    // 通过事件类型分发到对应的处理器
    // 实际使用中，应该通过EventSourcingManager来处理
}

// 6. 清除未提交事件
scene.clear_uncommitted_events();
```

## 常见问题

### Q: 如何处理事件之间的依赖关系？

A: 事件应该表示已经发生的事实，不应该有依赖关系。如果需要顺序处理，应该在事件处理器中处理，而不是在事件本身中。

### Q: 事件可以包含其他聚合的引用吗？

A: 可以，但应该使用ID引用，而不是直接引用聚合对象。这样可以避免循环依赖和序列化问题。

### Q: 如何测试事件系统？

A: 可以创建测试事件和测试处理器，验证事件发布和处理是否正确。参考`tests/integration/event_system_e2e_test.rs`。

## 相关文档

- [事件溯源系统使用指南](./event_sourcing_guide.md)
- [聚合根设计指南](./aggregate_root_guide.md)
- [事件类型注册表文档](../event_registry.md)

