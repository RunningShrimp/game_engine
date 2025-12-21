# 事件类型注册表

## 概述

事件类型注册表提供事件类型的注册、序列化/反序列化和验证功能，与事件溯源系统集成，确保事件能够正确存储和重放。

## 设计目标

1. **类型安全**：确保事件类型正确注册和验证
2. **序列化支持**：支持完整的事件序列化/反序列化
3. **版本兼容**：支持事件类型的版本管理
4. **性能优化**：使用HashMap快速查找

## 核心组件

### EventRegistry

事件类型注册表，管理所有已注册的事件类型。

```rust
use game_engine::domain::{EventRegistry, DomainEvent};
use serde::{Serialize, Deserialize};

// 创建注册表
let registry = EventRegistry::new();

// 注册事件类型
registry.register::<SceneLoadedEvent>("SceneLoaded", 1)?;

// 序列化事件
let event = SceneLoadedEvent { scene_id: 1, scene_name: "Test".to_string() };
let serialized = registry.serialize(&event)?;

// 反序列化事件
let deserialized: Box<dyn DomainEvent> = registry.deserialize("SceneLoaded", &serialized)?;
```

### 全局注册表

使用全局单例注册表，方便在整个应用中使用：

```rust
use game_engine::domain::{register_event_type, deserialize_event};

// 注册到全局注册表
register_event_type::<SceneLoadedEvent>("SceneLoaded", 1)?;

// 从全局注册表反序列化
let deserialized = deserialize_event("SceneLoaded", &serialized)?;
```

## 使用示例

### 基本使用

```rust
use game_engine::domain::{EventRegistry, DomainEvent};
use bevy_ecs::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyEvent {
    value: u32,
}

impl DomainEvent for MyEvent {
    fn event_type(&self) -> &'static str {
        "MyEvent"
    }

    fn apply(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }

    fn revert(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }
}

// 注册事件类型
let registry = EventRegistry::new();
registry.register::<MyEvent>("MyEvent", 1)?;

// 序列化和反序列化
let event = MyEvent { value: 42 };
let serialized = registry.serialize(&event)?;
let deserialized = registry.deserialize("MyEvent", &serialized)?;
```

### 与事件溯源系统集成

```rust
use game_engine::domain::{EventSourcingManager, EventRegistry, register_event_type};
use game_engine::domain::events::{DomainEvent, AggregateRoot};

// 注册所有事件类型
register_event_type::<SceneLoadedEvent>("SceneLoaded", 1)?;
register_event_type::<SceneActivatedEvent>("SceneActivated", 1)?;
register_event_type::<EntityAddedEvent>("EntityAdded", 1)?;

// 创建事件溯源管理器（使用全局注册表）
let registry = global_registry();
let manager = EventSourcingManager::with_registry(
    event_store,
    snapshot_store,
    registry,
);

// 提交事件（会自动使用注册表序列化）
let mut scene = Scene::new(SceneId(1), "Test Scene");
scene.load()?;
manager.commit_aggregate_events(&mut scene, &mut world)?;

// 重放并反序列化事件
let events = manager.replay_and_deserialize_events("Scene_1", None)?;
for event in events {
    event.apply(&mut world)?;
}
```

### 事件类型验证

```rust
use game_engine::domain::EventRegistry;

let registry = EventRegistry::new();
registry.register::<MyEvent>("MyEvent", 1)?;

// 检查事件类型是否已注册
assert!(registry.is_registered("MyEvent"));

// 获取事件类型信息
if let Some(info) = registry.get_type_info("MyEvent") {
    println!("Event type: {}, version: {}", info.name, info.version);
}

// 验证事件类型
registry.validate_event_type::<MyEvent>("MyEvent")?;
```

## 版本管理

事件类型注册表支持版本管理，可以跟踪事件类型的版本号：

```rust
// 注册版本1
registry.register::<MyEvent>("MyEvent", 1)?;

// 获取版本
let version = registry.get_version("MyEvent");
assert_eq!(version, Some(1));

// 升级到版本2（需要实现版本迁移逻辑）
registry.unregister("MyEvent")?;
registry.register::<MyEventV2>("MyEvent", 2)?;
```

## 与事件溯源系统的集成

### 自动序列化

当使用`EventSourcingManager::save_event`时，事件会自动通过注册表序列化：

```rust
let manager = EventSourcingManager::with_registry(
    event_store,
    snapshot_store,
    event_registry,
);

// 保存事件（自动序列化）
manager.save_event(
    &SceneLoadedEvent { scene_id: 1, scene_name: "Test".to_string() },
    Some("Scene_1"),
    1,
    &world,
)?;
```

### 自动反序列化

使用`replay_and_deserialize_events`可以自动反序列化事件：

```rust
// 重放并反序列化事件
let events = manager.replay_and_deserialize_events("Scene_1", None)?;

// 应用事件
for event in events {
    event.apply(&mut world)?;
}
```

## 最佳实践

### 1. 在应用启动时注册所有事件类型

```rust
fn setup_event_registry() -> Result<(), EventError> {
    let registry = global_registry();
    let mut registry_guard = registry.write()?;
    
    // 注册所有领域事件
    registry_guard.register::<SceneLoadedEvent>("SceneLoaded", 1)?;
    registry_guard.register::<SceneActivatedEvent>("SceneActivated", 1)?;
    registry_guard.register::<EntityAddedEvent>("EntityAdded", 1)?;
    registry_guard.register::<EntityRemovedEvent>("EntityRemoved", 1)?;
    
    Ok(())
}
```

### 2. 使用全局注册表

对于大多数应用，使用全局注册表更方便：

```rust
use game_engine::domain::{register_event_type, deserialize_event, global_registry};

// 注册
register_event_type::<MyEvent>("MyEvent", 1)?;

// 使用
let registry = global_registry();
let registry_guard = registry.read()?;
let deserialized = registry_guard.deserialize("MyEvent", &data)?;
```

### 3. 事件类型命名规范

- 使用PascalCase命名（如`SceneLoadedEvent`）
- 事件类型名称必须与`event_type()`返回的值一致
- 使用版本号管理事件类型变更

### 4. 错误处理

```rust
match registry.serialize(&event) {
    Ok(data) => {
        // 序列化成功
    }
    Err(EventError::UnknownEventType(name)) => {
        // 事件类型未注册
        eprintln!("Event type '{}' is not registered", name);
    }
    Err(EventError::SerializationError(e)) => {
        // 序列化失败
        eprintln!("Serialization error: {}", e);
    }
    Err(e) => {
        // 其他错误
        eprintln!("Error: {}", e);
    }
}
```

## 性能考虑

- **查找性能**：使用HashMap，O(1)查找时间
- **序列化开销**：使用`bincode`，高效的二进制序列化
- **内存开销**：每个事件类型约100-200字节（反序列化器对象）

## 限制和注意事项

1. **类型安全**：事件类型必须在编译时已知，不支持动态类型
2. **版本兼容**：版本管理需要手动实现迁移逻辑
3. **序列化格式**：使用`bincode`，不支持跨语言序列化
4. **线程安全**：使用`RwLock`，读多写少场景优化

## 未来改进

- [ ] 支持事件类型迁移（版本升级）
- [ ] 支持事件类型别名
- [ ] 支持事件类型继承
- [ ] 支持事件类型验证规则
- [ ] 支持事件类型元数据（描述、标签等）

