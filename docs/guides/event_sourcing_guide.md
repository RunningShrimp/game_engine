# 事件溯源指南

本指南详细介绍事件溯源系统的使用方法和最佳实践。

## 概述

事件溯源（Event Sourcing）是一种架构模式，将应用程序状态的变化记录为一系列事件。游戏引擎提供了完整的事件溯源实现，支持：

- **事件存储和检索**
- **事件重放**
- **快照管理**
- **版本控制**
- **时间旅行调试**
- **事件查询和过滤**
- **事件投影**

## 核心概念

### 领域事件

领域事件表示业务中发生的重要事情：

```rust
use game_engine::domain::events::{DomainEvent, EventError};
use bevy_ecs::prelude::*;

pub struct PlayerMovedEvent {
    pub player_id: u32,
    pub from: [f32; 3],
    pub to: [f32; 3],
}

impl DomainEvent for PlayerMovedEvent {
    fn event_type(&self) -> &'static str {
        "PlayerMoved"
    }

    fn apply(&self, world: &mut World) -> Result<(), EventError> {
        // 应用事件到世界状态
        // 例如：更新玩家位置
        Ok(())
    }

    fn revert(&self, world: &mut World) -> Result<(), EventError> {
        // 撤销事件（反向操作）
        // 例如：恢复玩家位置
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
```

### 聚合根

聚合根是领域对象的根实体，负责管理其内部状态和事件：

```rust
use game_engine::domain::events::{AggregateRoot, DomainEvent};

pub struct Player {
    id: u32,
    position: [f32; 3],
    uncommitted_events: Vec<Box<dyn DomainEvent>>,
}

impl AggregateRoot for Player {
    fn aggregate_id(&self) -> String {
        format!("Player_{}", self.id)
    }

    fn take_uncommitted_events(&mut self) -> Vec<Box<dyn DomainEvent>> {
        std::mem::take(&mut self.uncommitted_events)
    }

    fn mark_events_committed(&mut self) {
        // 事件已提交，可以清除
    }

    fn uncommitted_event_count(&self) -> usize {
        self.uncommitted_events.len()
    }
}

impl Player {
    pub fn move_to(&mut self, new_position: [f32; 3]) {
        let event = PlayerMovedEvent {
            player_id: self.id,
            from: self.position,
            to: new_position,
        };
        
        // 应用事件到聚合
        self.position = new_position;
        
        // 记录事件
        self.uncommitted_events.push(Box::new(event));
    }
}
```

## 基础使用

### 创建事件溯源管理器

```rust
use game_engine::domain::event_sourcing::{
    EventSourcingManager, MemoryEventStore, MemorySnapshotStore
};
use std::sync::{Arc, RwLock};

// 创建存储
let event_store: Arc<RwLock<Box<dyn EventStore>>> =
    Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
let snapshot_store: Arc<RwLock<Box<dyn SnapshotStore>>> =
    Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));

// 创建管理器
let manager = EventSourcingManager::new(event_store, snapshot_store);
```

### 保存事件

```rust
use game_engine::domain::event_sourcing::EventSourcingManager;

let event = PlayerMovedEvent {
    player_id: 1,
    from: [0.0, 0.0, 0.0],
    to: [10.0, 0.0, 0.0],
};

let event_id = manager.save_event(
    &event,
    Some("Player_1"),
    1, // 聚合版本
    &world,
)?;
```

### 提交聚合事件

```rust
let mut player = Player::new(1);
player.move_to([10.0, 0.0, 0.0]);

// 提交聚合的未提交事件
let event_id = manager.commit_aggregate_events(&mut player, &mut world)?;
```

### 重放事件

```rust
// 重放聚合的所有事件
let events = manager.replay_aggregate_events("Player_1", None)?;

// 重放并反序列化事件
let deserialized_events = manager.replay_and_deserialize_events("Player_1", None)?;

// 应用到世界
for event in deserialized_events {
    event.apply(&mut world)?;
}
```

## 高级功能

### 事件查询

使用增强的事件溯源管理器进行高级查询：

```rust
use game_engine::domain::event_sourcing_enhanced::{
    EnhancedEventSourcingManager, EventQuery
};

let enhanced_manager = EnhancedEventSourcingManager::new(event_store, snapshot_store);

// 查询特定聚合的所有事件
let events = enhanced_manager.query_events(
    EventQuery::by_aggregate("Player_1")
)?;

// 查询特定类型的事件
let events = enhanced_manager.query_events(
    EventQuery::by_event_type("PlayerMoved")
)?;

// 查询时间范围内的事件
let events = enhanced_manager.query_events(
    EventQuery::by_time_range(start_time, end_time)
)?;

// 组合查询
let events = enhanced_manager.query_events(
    EventQuery::by_aggregate("Player_1")
        .with_limit(100)
        .with_offset(0)
)?;
```

### 时间旅行调试

重放到指定时间点或版本：

```rust
// 重放到指定时间点
let target_time = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos() as i64 - 3600_000_000_000; // 1小时前

enhanced_manager.replay_to_time(&mut world, target_time)?;

// 重放到指定版本
enhanced_manager.replay_to_version(&mut world, "Player_1", 10)?;
```

### 事件统计

获取事件统计信息：

```rust
// 获取特定聚合的统计
let stats = enhanced_manager.get_event_stats(Some("Player_1"))?;

// 获取全局统计（需要提供aggregate_id）
let stats = enhanced_manager.get_event_stats(None)?;

println!("总事件数: {}", stats.total_events);
println!("按类型统计: {:?}", stats.events_by_type);
println!("按聚合统计: {:?}", stats.events_by_aggregate);
```

### 事件流处理

使用事件流处理器进行过滤和转换：

```rust
use game_engine::domain::event_sourcing_enhanced::EventStreamProcessor;

let mut processor = EventStreamProcessor::new();

// 添加过滤器：只保留特定类型的事件
processor.add_filter(|e| e.event_type == "PlayerMoved");

// 添加转换器：修改事件数据
processor.add_transformer(|mut e| {
    // 转换事件
    e
});

// 处理事件流
let filtered_events = processor.process(events);
```

### 快照管理

使用快照加速聚合恢复：

```rust
// 创建快照
let snapshot_id = manager.create_snapshot(
    &player,
    "Player_1",
    current_version,
)?;

// 从快照恢复
let (restored_player, version) = manager.restore_from_snapshot::<Player>("Player_1")?;

// 从快照版本开始重放事件
let events = manager.replay_aggregate_events("Player_1", Some(version))?;
```

## 事件投影

事件投影用于从事件流构建只读视图：

```rust
use game_engine::domain::event_sourcing_enhanced::{EventProjection, StoredEvent};

pub struct PlayerPositionProjection {
    positions: HashMap<u32, [f32; 3]>,
}

impl EventProjection for PlayerPositionProjection {
    fn name(&self) -> &str {
        "PlayerPosition"
    }

    fn handle_event(&mut self, event: &StoredEvent) -> Result<(), EventError> {
        if event.event_type == "PlayerMoved" {
            // 解析事件并更新投影状态
            // ...
        }
        Ok(())
    }

    fn get_state(&self) -> Result<Vec<u8>, EventError> {
        // 序列化投影状态
        Ok(bincode::serialize(&self.positions)?)
    }

    fn restore_from_state(&mut self, state: Vec<u8>) -> Result<(), EventError> {
        // 反序列化投影状态
        self.positions = bincode::deserialize(&state)?;
        Ok(())
    }
}

// 注册投影
let projection = Box::new(PlayerPositionProjection::new());
enhanced_manager.register_projection(projection)?;
```

## 最佳实践

### 1. 事件设计

- **不可变**: 事件应该是不可变的
- **自包含**: 事件应包含所有必要信息
- **语义清晰**: 事件名称应清晰表达业务含义
- **版本化**: 事件结构变化时使用版本号

### 2. 聚合设计

- **单一职责**: 每个聚合只负责一个业务概念
- **边界清晰**: 明确聚合的边界
- **事件一致性**: 确保事件与聚合状态一致

### 3. 性能优化

- **使用快照**: 定期创建快照以减少重放时间
- **事件清理**: 定期清理旧事件
- **批量处理**: 批量提交事件以减少I/O

### 4. 错误处理

- **幂等性**: 确保事件应用是幂等的
- **回滚支持**: 实现事件撤销功能
- **错误恢复**: 处理事件应用失败的情况

## 常见问题

### Q: 何时使用事件溯源？

**A**: 适合场景：
- 需要完整审计日志
- 需要时间旅行调试
- 需要事件重放
- 需要事件投影

不适合场景：
- 简单的CRUD应用
- 性能要求极高的场景
- 事件历史不重要

### Q: 如何处理事件版本迁移？

**A**: 
1. 使用事件注册表管理不同版本
2. 实现版本迁移逻辑
3. 在反序列化时进行版本转换

### Q: 快照何时创建？

**A**: 
- 定期创建（每N个事件）
- 重要状态变更时
- 性能优化需要时

### Q: 如何清理旧事件？

**A**: 
- 设置最大历史长度
- 定期清理旧事件
- 保留重要事件（如快照相关）

## 相关文档

- [领域驱动设计](../architecture.md#领域驱动设计)
- [事件系统](../domain/events.rs)
- [ADR-0002: 领域驱动设计](../adr/0002-domain-driven-design.md)

