# 使用指南目录

本目录包含游戏引擎各个功能模块的使用指南和最佳实践文档。

## 指南列表

### 核心概念

1. **[领域事件系统使用指南](./domain_events_guide.md)**
   - 领域事件的基本概念和使用方法
   - SafeEventBus的使用
   - 聚合根集成
   - 最佳实践和常见问题

2. **[事件溯源系统使用指南](./event_sourcing_guide.md)**
   - 事件溯源的基本概念
   - EventSourcingManager的使用
   - 事件存储和重放
   - 快照和版本控制

3. **[聚合根设计指南](./aggregate_root_guide.md)**
   - 聚合根的设计原则
   - 不变性约束
   - 事务边界
   - 实现模式和最佳实践

### 性能优化

4. **[性能优化最佳实践指南](./performance_optimization_guide.md)**
   - ECS系统优化
   - 渲染管线优化
   - 资源管理优化
   - 内存和并发优化

## 快速开始

### 领域事件系统

```rust
use game_engine::domain::events::SafeEventBus;
use std::sync::Arc;

let bus = Arc::new(SafeEventBus::new());
bus.subscribe::<MyEvent>(|event| {
    println!("Event received: {:?}", event);
});
bus.publish(&MyEvent { data: 42 });
```

### 事件溯源

```rust
use game_engine::domain::event_sourcing::{
    EventSourcingManager, MemoryEventStore, MemorySnapshotStore,
};

let manager = EventSourcingManager::new(
    Arc::new(RwLock::new(Box::new(MemoryEventStore::new()))),
    Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new()))),
);
```

### 聚合根

```rust
use game_engine::domain::scene::Scene;

let mut scene = Scene::new("MainScene", "scene_1".to_string())?;
scene.activate()?;
let events = scene.uncommitted_events();
```

## 相关文档

- [项目状态文档](../PROJECT_STATUS.md)
- [P4阶段计划](../p4_plan.md)
- [事件类型注册表](../event_registry.md)

## 贡献

如果您发现文档中的错误或需要补充的内容，欢迎提交Issue或Pull Request。

