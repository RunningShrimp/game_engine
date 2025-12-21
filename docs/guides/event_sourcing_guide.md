# 事件溯源系统使用指南

## 概述

事件溯源系统提供了完整的事件存储、重放、快照和版本控制功能，与类型安全的事件系统集成。

## 核心概念

### EventSourcingManager

`EventSourcingManager`是事件溯源系统的核心管理器：

```rust
use game_engine::domain::event_sourcing::{
    EventSourcingManager, MemoryEventStore, MemorySnapshotStore,
};
use game_engine::domain::event_registry::GLOBAL_EVENT_REGISTRY;
use std::sync::Arc;

// 创建事件存储和快照存储
let event_store = Arc::new(std::sync::RwLock::new(
    Box::new(MemoryEventStore::new()) as Box<dyn EventStore>
));
let snapshot_store = Arc::new(std::sync::RwLock::new(
    Box::new(MemorySnapshotStore::new()) as Box<dyn SnapshotStore>
));

// 创建事件溯源管理器（使用全局事件注册表）
let manager = EventSourcingManager::with_registry(
    event_store,
    snapshot_store,
    Arc::clone(&GLOBAL_EVENT_REGISTRY),
);
```

### 事件注册

在使用事件溯源之前，需要注册事件类型：

```rust
use game_engine::domain::event_registry::GLOBAL_EVENT_REGISTRY;
use game_engine::domain::events::DomainEvent;

// 注册事件类型（通常在应用启动时进行）
{
    let registry = GLOBAL_EVENT_REGISTRY.write().unwrap();
    registry.register::<SceneLoadedEvent>(1).unwrap();
    registry.register::<SceneActivatedEvent>(1).unwrap();
    registry.register::<EntityAddedEvent>(1).unwrap();
}
```

## 基本使用

### 1. 提交聚合事件

```rust
use game_engine::domain::scene::Scene;
use bevy_ecs::prelude::*;

// 创建场景聚合根
let mut scene = Scene::new("MainScene", "scene_1".to_string())
    .expect("Failed to create scene");

// 执行操作（会产生领域事件）
scene.activate().unwrap();

// 提交事件到事件存储
let mut world = World::new();
let event_id = manager.commit_aggregate_events(&mut scene, &mut world)
    .expect("Failed to commit events");
```

### 2. 重放事件

```rust
// 重放聚合的所有事件
let events = manager.replay_aggregate_events("scene_1", None)
    .expect("Failed to replay events");

// 重放从指定版本开始的事件
let events_from_version = manager.replay_aggregate_events("scene_1", Some(5))
    .expect("Failed to replay events");
```

### 3. 创建快照

```rust
use serde::{Serialize, Deserialize};

// 注意：聚合必须实现Serialize trait才能创建快照
// Scene可能不完全可序列化，需要特殊处理
let snapshot_id = manager.create_snapshot(&scene, "scene_1", 10)
    .expect("Failed to create snapshot");
```

### 4. 从快照恢复

```rust
// 从快照恢复聚合状态
let snapshot = manager.restore_from_snapshot("scene_1")
    .expect("Failed to restore snapshot");

// 然后重放快照之后的事件
let events_after_snapshot = manager.replay_aggregate_events("scene_1", Some(snapshot.version));
```

## 事件存储

### MemoryEventStore

内存事件存储，用于测试和开发：

```rust
use game_engine::domain::event_sourcing::MemoryEventStore;

let mut store = MemoryEventStore::new();

// 保存事件
let stored_event = StoredEvent {
    id: EventId::now(1),
    event_type: "SceneLoadedEvent".to_string(),
    data: vec![],
    aggregate_id: Some("scene_1".to_string()),
    aggregate_version: 1,
};
store.save_event(stored_event).unwrap();

// 获取聚合的所有事件
let events = store.get_aggregate_events("scene_1");
```

### 自定义事件存储

可以实现`EventStore` trait来创建自定义事件存储（如数据库存储）：

```rust
use game_engine::domain::event_sourcing::EventStore;

pub struct DatabaseEventStore {
    // 数据库连接等
}

impl EventStore for DatabaseEventStore {
    fn save_event(&mut self, event: StoredEvent) -> Result<(), EventError> {
        // 保存到数据库
        Ok(())
    }
    
    // ... 实现其他方法
}
```

## 版本控制

事件溯源系统支持聚合版本控制：

```rust
// 获取聚合的当前版本
let version = manager.get_aggregate_version("scene_1")
    .expect("Failed to get version");

// 重放从指定版本开始的事件
let events = manager.replay_aggregate_events("scene_1", Some(version));
```

## 最佳实践

### 1. 事件序列化

- 确保所有事件类型都实现了`Serialize`和`Deserialize`
- 使用版本号管理事件结构的变更
- 避免在事件中包含不可序列化的类型

### 2. 快照策略

- 定期创建快照（如每100个事件）
- 快照应该包含完整的聚合状态
- 从快照恢复后，只重放快照之后的事件

### 3. 事件清理

- 定期清理旧事件（如保留最近10000个事件）
- 清理前确保已创建快照
- 考虑事件归档策略

### 4. 性能优化

- 使用批量操作处理多个事件
- 异步处理事件重放
- 使用快照减少重放时间

## 示例：完整场景

```rust
use game_engine::domain::event_sourcing::{
    EventSourcingManager, MemoryEventStore, MemorySnapshotStore,
};
use game_engine::domain::scene::Scene;
use game_engine::domain::event_registry::GLOBAL_EVENT_REGISTRY;
use bevy_ecs::prelude::*;
use std::sync::Arc;

// 1. 注册事件类型
{
    let registry = GLOBAL_EVENT_REGISTRY.write().unwrap();
    registry.register::<SceneLoadedEvent>(1).unwrap();
    registry.register::<SceneActivatedEvent>(1).unwrap();
}

// 2. 创建事件溯源管理器
let manager = EventSourcingManager::with_registry(
    Arc::new(std::sync::RwLock::new(
        Box::new(MemoryEventStore::new()) as Box<dyn EventStore>
    )),
    Arc::new(std::sync::RwLock::new(
        Box::new(MemorySnapshotStore::new()) as Box<dyn SnapshotStore>
    )),
    Arc::clone(&GLOBAL_EVENT_REGISTRY),
);

// 3. 创建并修改场景
let mut scene = Scene::new("MainScene", "scene_1".to_string())
    .expect("Failed to create scene");

let mut world = World::new();

// 4. 提交初始事件
manager.commit_aggregate_events(&mut scene, &mut world).unwrap();

// 5. 执行操作并提交事件
scene.activate().unwrap();
manager.commit_aggregate_events(&mut scene, &mut world).unwrap();

// 6. 重放事件验证
let events = manager.replay_aggregate_events("scene_1", None).unwrap();
assert_eq!(events.len(), 2);
```

## 常见问题

### Q: 如何处理事件结构的变更？

A: 使用版本号管理事件结构变更。在事件注册时指定版本号，反序列化时根据版本号选择对应的反序列化逻辑。

### Q: 快照创建失败怎么办？

A: 如果聚合不完全可序列化，可以：
1. 实现自定义序列化逻辑
2. 只序列化可序列化的部分
3. 使用其他快照机制（如状态转储）

### Q: 如何实现事件重放的性能优化？

A: 
1. 使用快照减少重放的事件数量
2. 批量处理事件
3. 异步重放事件
4. 使用索引加速事件查询

## 相关文档

- [领域事件系统使用指南](./domain_events_guide.md)
- [聚合根设计指南](./aggregate_root_guide.md)
- [事件类型注册表文档](../event_registry.md)

