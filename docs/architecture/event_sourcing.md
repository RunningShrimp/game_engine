# 事件溯源系统文档

## 概述

事件溯源系统为游戏引擎提供完整的事件记录和重放功能，支持：
- 事件记录和存储
- 事件重放
- 撤销/重做
- 时间旅行调试
- 快照系统（性能优化）

## 核心概念

### 事件 (Event)

事件是不可变的，表示已经发生的事实：

```rust
use game_engine::core::event_sourcing::{DomainEvent, EntityCreatedEvent};

let event = EntityCreatedEvent {
    entity_id: 1,
    entity_type: "Player".to_string(),
};
```

### 命令 (Command)

命令表示要执行的操作：

```rust
use game_engine::core::event_sourcing::{Command, CreateEntityCommand};

let command = CreateEntityCommand {
    entity_type: "Player".to_string(),
    initial_data: vec![],
};
```

### 事件存储 (Event Store)

事件存储负责持久化事件：

```rust
use game_engine::core::event_sourcing::{EventStore, MemoryEventStore};

let mut store = MemoryEventStore::new();
store.save_event(stored_event)?;
```

## 使用示例

### 基本使用

```rust
use game_engine::core::event_sourcing::*;

// 创建事件溯源管理器
let event_store = Arc::new(Mutex::new(MemoryEventStore::new()));
let snapshot_store = Arc::new(Mutex::new(MemorySnapshotStore::new()));
let manager = Arc::new(EventSourcingManager::new(event_store, snapshot_store));

// 记录事件
let event = EntityCreatedEvent {
    entity_id: 1,
    entity_type: "Player".to_string(),
};
let event_id = manager.record_event(event, Some(1))?;

// 重放事件
manager.replay_events(&mut world, None, None)?;
```

### 命令处理

```rust
use game_engine::core::event_sourcing::*;

let handler = CommandHandler::new(manager.clone());

// 执行命令
let command = CreateEntityCommand {
    entity_type: "Enemy".to_string(),
    initial_data: vec![],
};
let event_id = handler.execute_command(command, &mut world, Some(2))?;
```

### 撤销/重做

```rust
// 撤销最后一个事件
if let Some(event_id) = manager.undo_last_event(&mut world)? {
    println!("Undid event: {:?}", event_id);
}

// 重放事件（重做）
manager.replay_events(&mut world, None, Some(event_id))?;
```

### 时间旅行调试

```rust
use game_engine::core::event_sourcing::TimeTravelDebugger;

let debugger = TimeTravelDebugger::new(manager.clone());

// 跳转到特定时间点
debugger.jump_to_time(&mut world, event_id)?;

// 前进一个事件
debugger.step_forward(&mut world)?;

// 后退一个事件
debugger.step_backward(&mut world)?;
```

## 快照系统

快照系统用于优化性能，避免重放大量事件：

```rust
// 设置快照间隔（每100个事件创建一个快照）
manager.set_snapshot_interval(100);

// 从快照恢复
manager.replay_aggregate_events(&mut world, aggregate_id)?;
```

## 集成到ECS

```rust
use game_engine::core::event_sourcing::EventSourcingResource;

// 添加资源
world.insert_resource(EventSourcingResource::new(manager));

// 在系统中使用
fn update_system(
    event_sourcing: Res<EventSourcingResource>,
    mut world: &mut World,
) {
    // 记录事件
    let event = EntityCreatedEvent { ... };
    event_sourcing.manager().record_event(event, Some(1))?;
}
```

## 性能考虑

1. **快照间隔**: 根据事件频率调整快照间隔
2. **历史长度**: 限制事件历史长度，避免内存溢出
3. **异步处理**: 事件存储可以异步处理，避免阻塞主循环

## 最佳实践

1. **只记录重要事件**: 不要记录每一帧的微小变化
2. **使用聚合ID**: 为相关事件使用聚合ID，便于查询
3. **定期清理**: 清理旧事件，保持内存使用合理
4. **快照策略**: 根据事件频率和性能需求调整快照策略

## 限制

1. **序列化**: 当前实现需要手动序列化/反序列化事件
2. **类型注册**: 需要类型注册系统来支持动态事件类型
3. **世界状态序列化**: ECS世界状态的完整序列化需要额外实现

## 未来改进

1. 自动类型注册系统
2. ECS世界状态序列化
3. 持久化事件存储（文件/数据库）
4. 分布式事件溯源支持

