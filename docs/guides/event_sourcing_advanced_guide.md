# 事件溯源高级用法指南

## 概述

事件溯源（Event Sourcing）是一种持久化模式，将应用状态的变化记录为一系列事件，而不是直接修改状态。本引擎提供了完整的事件溯源系统。

## 核心概念

### 事件溯源的优势

1. **完整历史记录**：所有状态变化都被记录，可以回放历史
2. **时间旅行调试**：可以回退到任意历史状态
3. **审计追踪**：自动记录所有操作，便于审计和合规
4. **事件重放**：可以重放事件来重建状态或迁移数据

### 架构组件

- **EventStore**：存储事件序列
- **SnapshotStore**：存储状态快照，加速恢复
- **EventSourcingManager**：管理事件和快照
- **Aggregate Roots**：领域聚合根，维护一致性边界

## 高级用法

### 事件版本控制

```rust
use game_engine::domain::event_sourcing::{EventSourcingManager, MemoryEventStore};

let manager = EventSourcingManager::new(
    Arc::new(RwLock::new(Box::new(MemoryEventStore::new()))),
    // ...
);

// 注册事件类型和版本
manager.register_event_type("SceneLoadedEvent", 1, serialize_fn, deserialize_fn)?;

// 升级事件版本
manager.upgrade_event("SceneLoadedEvent", 1, 2, upgrade_fn)?;
```

### 快照策略

```rust
// 配置快照策略
let snapshot_config = SnapshotConfig {
    // 每N个事件创建一个快照
    snapshot_interval: 100,
    // 保留最近N个快照
    max_snapshots: 10,
    // 自动清理旧快照
    auto_cleanup: true,
};

manager.set_snapshot_config(snapshot_config);
```

### 事件重放

```rust
// 重放所有事件重建状态
manager.replay_events(aggregate_id, |event| {
    // 应用事件到状态
    apply_event_to_state(event);
})?;

// 从快照恢复并重放后续事件
manager.restore_from_snapshot(aggregate_id)?;
manager.replay_events_since(aggregate_id, snapshot_version)?;
```

### 事件查询

```rust
// 查询特定时间范围的事件
let events = manager.query_events(
    aggregate_id,
    start_time,
    end_time,
)?;

// 查询特定类型的事件
let scene_events = manager.query_events_by_type(
    aggregate_id,
    "SceneLoadedEvent",
)?;
```

## 性能优化

1. **批量提交**：使用批量提交减少I/O操作
2. **异步持久化**：使用异步写入提升性能
3. **快照优化**：合理设置快照间隔，平衡恢复速度和存储空间
4. **事件压缩**：定期压缩旧事件，减少存储空间

## 最佳实践

- 保持事件小而专注，避免大型事件
- 使用事件版本控制，支持向后兼容
- 定期创建快照，加速状态恢复
- 监控事件存储大小，及时清理旧事件

## 相关文档

- [事件溯源基础指南](event_sourcing_guide.md)
- [聚合根指南](aggregate_root_guide.md)

