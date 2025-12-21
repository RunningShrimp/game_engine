# 聚合根设计指南

## 概述

聚合根（Aggregate Root）是领域驱动设计（DDD）中的核心概念，用于封装业务逻辑和维护数据一致性。

## 核心概念

### 聚合根的特征

1. **唯一标识**：每个聚合根都有唯一的ID
2. **一致性边界**：聚合根负责维护其内部的一致性
3. **事务边界**：聚合根是事务的基本单位
4. **访问入口**：外部只能通过聚合根访问聚合内的实体

### AggregateRoot Trait

所有聚合根都应该实现`AggregateRoot` trait：

```rust
use game_engine::domain::events::{AggregateRoot, DomainEvent};

pub struct Scene {
    id: String,
    name: String,
    entities: HashMap<String, GameEntity>,
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

## 设计原则

### 1. 聚合边界

- **包含什么**：聚合根应该包含所有需要保持一致性的实体和值对象
- **不包含什么**：不应该包含其他聚合根的引用（只使用ID引用）

```rust
// ✅ 正确：使用ID引用其他聚合
pub struct Scene {
    id: String,
    entities: HashMap<String, GameEntity>, // 聚合内的实体
    // 不直接引用其他Scene，只使用ID
}

// ❌ 错误：直接引用其他聚合根
pub struct Scene {
    parent_scene: Scene, // 不应该这样做
}
```

### 2. 不变性约束

聚合根应该维护业务不变性（invariants）：

```rust
impl Scene {
    pub fn add_entity(&mut self, entity: GameEntity) -> Result<EntityId, SceneError> {
        // 1. 验证不变性约束
        self.validate()?;
        
        // 2. 执行业务逻辑
        let entity_id = entity.id().clone();
        self.entities.insert(entity_id.clone(), entity);
        
        // 3. 发布领域事件
        self.event_queue.add_event(EntityAddedEvent {
            scene_id: self.id.clone(),
            entity_id: entity_id.clone(),
        });
        
        // 4. 再次验证不变性
        self.check_invariants()?;
        
        Ok(entity_id)
    }
    
    fn check_invariants(&self) -> Result<(), SceneError> {
        // 检查业务规则
        // 例如：场景名称不能为空、实体ID必须唯一等
        if self.name.is_empty() {
            return Err(SceneError::InvalidName);
        }
        Ok(())
    }
}
```

### 3. 事务边界

- 一个事务应该只修改一个聚合根
- 如果需要修改多个聚合根，使用领域事件进行协调

```rust
// ✅ 正确：一个事务修改一个聚合
scene.add_entity(entity)?; // 事务1

// ✅ 正确：使用事件协调多个聚合
scene.activate()?; // 发布SceneActivatedEvent
// 其他聚合订阅此事件并响应

// ❌ 错误：一个事务修改多个聚合
scene.add_entity(entity)?; // 聚合1
other_scene.remove_entity(id)?; // 聚合2 - 不应该在同一事务中
```

### 4. 领域事件

聚合根应该通过领域事件与其他聚合通信：

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

## 实现模式

### 1. 基本聚合根结构

```rust
use game_engine::domain::events::{AggregateRoot, DomainEvent, AggregateEventQueue};

pub struct MyAggregate {
    id: String,
    // 聚合状态
    state: AggregateState,
    // 事件队列
    event_queue: AggregateEventQueue,
}

impl AggregateRoot for MyAggregate {
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

### 2. 验证和不变性检查

```rust
impl MyAggregate {
    pub fn validate(&self) -> Result<(), DomainError> {
        // 验证所有业务规则
        if self.id.is_empty() {
            return Err(DomainError::InvalidId);
        }
        // ... 其他验证
        Ok(())
    }
    
    fn check_invariants(&self) -> Result<(), DomainError> {
        // 检查不变性约束
        // 在状态修改后调用
        self.validate()
    }
}
```

### 3. 错误处理和恢复

```rust
use game_engine::domain::errors::{DomainError, RecoveryStrategy};

impl MyAggregate {
    pub fn perform_operation(&mut self) -> Result<(), DomainError> {
        // 1. 验证前置条件
        self.validate()?;
        
        // 2. 执行业务逻辑
        match self.do_operation() {
            Ok(()) => {
                // 3. 发布领域事件
                self.event_queue.add_event(OperationCompletedEvent {
                    aggregate_id: self.id.clone(),
                });
                
                // 4. 验证后置条件
                self.check_invariants()?;
                Ok(())
            }
            Err(e) => {
                // 5. 错误恢复
                self.recover_from_error(&e)?;
                Err(e)
            }
        }
    }
    
    fn recover_from_error(&mut self, error: &DomainError) -> Result<(), DomainError> {
        // 实现恢复策略
        match error.recovery_strategy() {
            RecoveryStrategy::Retry => {
                // 重试逻辑
            }
            RecoveryStrategy::Compensate => {
                // 补偿操作
            }
            RecoveryStrategy::Fail => {
                // 失败处理
            }
        }
        Ok(())
    }
}
```

## 最佳实践

### 1. 聚合大小

- **不要太大**：聚合应该尽可能小，只包含需要保持一致性的实体
- **不要太小**：避免过度拆分，导致不必要的复杂性

### 2. 聚合ID

- 使用有意义的ID（如UUID、业务ID）
- ID应该是不可变的
- 考虑ID的可读性和可调试性

### 3. 领域事件

- 事件应该表示已经发生的事实
- 事件应该是不可变的
- 使用过去时态命名事件

### 4. 性能考虑

- 避免在聚合中包含大量数据
- 考虑使用懒加载（lazy loading）
- 使用值对象减少内存占用

## 示例：Scene聚合根

```rust
use game_engine::domain::scene::Scene;
use game_engine::domain::entity::GameEntity;

// 创建场景聚合根
let mut scene = Scene::new("MainScene", "scene_1".to_string())
    .expect("Failed to create scene");

// 验证场景
assert!(scene.validate().is_ok());

// 添加实体（会发布EntityAddedEvent）
let entity = GameEntity::new("entity_1".to_string(), Vec3::ZERO);
let entity_id = scene.add_entity(entity).unwrap();

// 激活场景（会发布SceneActivatedEvent）
scene.activate().unwrap();

// 获取未提交的事件
let events = scene.uncommitted_events();
assert_eq!(events.len(), 3); // SceneLoadedEvent, EntityAddedEvent, SceneActivatedEvent

// 提交事件后清除
scene.clear_uncommitted_events();
```

## 常见问题

### Q: 如何确定聚合边界？

A: 聚合边界应该基于业务一致性需求。如果两个实体需要保持一致，它们应该在同一个聚合中。

### Q: 如何处理聚合间的引用？

A: 使用ID引用，而不是直接引用。通过领域事件进行聚合间的通信。

### Q: 如何测试聚合根？

A: 
1. 测试业务规则验证
2. 测试不变性约束
3. 测试领域事件发布
4. 测试错误处理和恢复

参考`game_engine/src/domain/aggregate_invariants_tests.rs`。

## 相关文档

- [领域事件系统使用指南](./domain_events_guide.md)
- [事件溯源系统使用指南](./event_sourcing_guide.md)
- [聚合边界文档](../domain/AGGREGATE_BOUNDARIES.md)

