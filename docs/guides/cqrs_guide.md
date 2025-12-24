# CQRS模式指南

本指南详细介绍CQRS（Command Query Responsibility Segregation）模式的使用方法和最佳实践。

## 概述

CQRS是一种架构模式，将命令（写操作）和查询（读操作）分离。游戏引擎提供了完整的CQRS实现，支持：

- **命令处理**: 修改系统状态的操作
- **查询处理**: 只读操作
- **命令总线**: 路由命令到相应的处理器
- **查询总线**: 路由查询到相应的处理器
- **与事件溯源集成**: 命令可以产生领域事件

## 核心概念

### 命令（Command）

命令表示修改系统状态的意图，不返回值（或返回事件ID）：

```rust
use game_engine::domain::cqrs::{Command, CommandResult};

#[derive(Debug, Clone)]
pub struct MovePlayerCommand {
    pub player_id: u32,
    pub position: [f32; 3],
}

impl Command for MovePlayerCommand {
    fn command_type(&self) -> &'static str {
        "MovePlayer"
    }
}
```

### 命令处理器（Command Handler）

命令处理器处理特定类型的命令，可能产生领域事件：

```rust
use game_engine::domain::cqrs::{CommandHandler, CommandResult};
use game_engine::domain::events::EventError;
use bevy_ecs::prelude::*;

pub struct MovePlayerCommandHandler;

impl CommandHandler<MovePlayerCommand> for MovePlayerCommandHandler {
    fn handle(
        &self,
        command: MovePlayerCommand,
        world: &mut World,
    ) -> Result<CommandResult, EventError> {
        // 验证命令
        // 修改状态
        // 产生领域事件（可选）
        
        Ok(CommandResult::success(None))
    }
}
```

### 查询（Query）

查询表示只读操作，返回数据但不修改状态：

```rust
use game_engine::domain::cqrs::Query;

#[derive(Debug, Clone)]
pub struct GetPlayerPositionQuery {
    pub player_id: u32,
}

impl Query for GetPlayerPositionQuery {
    fn query_type(&self) -> &'static str {
        "GetPlayerPosition"
    }
}
```

### 查询处理器（Query Handler）

查询处理器处理特定类型的查询，返回只读数据：

```rust
use game_engine::domain::cqrs::{QueryHandler, QueryResult, QueryError};
use bevy_ecs::prelude::*;

pub struct GetPlayerPositionQueryHandler;

impl QueryHandler<GetPlayerPositionQuery> for GetPlayerPositionQueryHandler {
    type Result = [f32; 3];

    fn handle(
        &self,
        query: GetPlayerPositionQuery,
        world: &World,
    ) -> QueryResult<Self::Result> {
        // 从世界状态读取数据
        // 返回结果
        
        Ok([0.0, 0.0, 0.0])
    }
}
```

## 基础使用

### 创建CQRS管理器

```rust
use game_engine::domain::cqrs::CqrsManager;
use game_engine::domain::event_sourcing::{EventSourcingManager, MemoryEventStore, MemorySnapshotStore};
use std::sync::{Arc, RwLock};

// 创建事件溯源管理器（可选）
let event_store = Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
let snapshot_store = Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));
let event_sourcing = Arc::new(EventSourcingManager::new(event_store, snapshot_store));

// 创建CQRS管理器
let cqrs = CqrsManager::with_event_sourcing(event_sourcing);
```

### 注册处理器

```rust
use std::sync::Arc;

// 注册命令处理器
let move_handler = Arc::new(MovePlayerCommandHandler);
cqrs.register_command_handler(move_handler)?;

// 注册查询处理器
let position_handler = Arc::new(GetPlayerPositionQueryHandler);
cqrs.register_query_handler(position_handler)?;
```

### 执行命令

```rust
let command = MovePlayerCommand {
    player_id: 1,
    position: [10.0, 0.0, 0.0],
};

let result = cqrs.execute_command(command, &mut world)?;

if result.success {
    println!("Command executed successfully");
    if let Some(event_id) = result.event_id {
        println!("Event ID: {:?}", event_id);
    }
} else {
    eprintln!("Command failed: {:?}", result.error);
}
```

### 执行查询

```rust
let query = GetPlayerPositionQuery { player_id: 1 };
let position: [f32; 3] = cqrs.execute_query(query, &world)?;

println!("Player position: {:?}", position);
```

## 与事件溯源集成

### 命令产生事件

命令处理器可以产生领域事件，这些事件会被记录到事件溯源系统：

```rust
use game_engine::domain::events::{DomainEvent, EventError};
use game_engine::domain::event_sourcing::EventId;

pub struct MovePlayerCommandHandler {
    event_sourcing: Arc<EventSourcingManager>,
}

impl CommandHandler<MovePlayerCommand> for MovePlayerCommandHandler {
    fn handle(
        &self,
        command: MovePlayerCommand,
        world: &mut World,
    ) -> Result<CommandResult, EventError> {
        // 创建领域事件
        let event = PlayerMovedEvent {
            player_id: command.player_id,
            from: [0.0, 0.0, 0.0], // 从当前状态获取
            to: command.position,
        };

        // 应用事件
        event.apply(world)?;

        // 保存事件到事件溯源系统
        let event_id = self.event_sourcing.save_event(
            &event,
            Some(&format!("Player_{}", command.player_id)),
            1, // 版本号
            world,
        )?;

        Ok(CommandResult::success(Some(event_id)))
    }
}
```

## 高级功能

### 命令验证

在命令处理器中验证命令：

```rust
impl CommandHandler<MovePlayerCommand> for MovePlayerCommandHandler {
    fn handle(
        &self,
        command: MovePlayerCommand,
        world: &mut World,
    ) -> Result<CommandResult, EventError> {
        // 验证玩家存在
        // 验证位置有效
        // 验证权限等
        
        if !self.is_valid_position(&command.position) {
            return Ok(CommandResult::failure(
                "Invalid position".to_string()
            ));
        }

        // 执行命令
        // ...
    }
}
```

### 查询缓存

查询处理器可以实现缓存以提高性能：

```rust
use std::collections::HashMap;
use std::sync::Mutex;

pub struct CachedGetPlayerPositionQueryHandler {
    cache: Arc<Mutex<HashMap<u32, [f32; 3]>>>,
}

impl QueryHandler<GetPlayerPositionQuery> for CachedGetPlayerPositionQueryHandler {
    type Result = [f32; 3];

    fn handle(
        &self,
        query: GetPlayerPositionQuery,
        world: &World,
    ) -> QueryResult<Self::Result> {
        // 检查缓存
        if let Ok(cache) = self.cache.lock() {
            if let Some(position) = cache.get(&query.player_id) {
                return Ok(*position);
            }
        }

        // 从世界读取
        let position = self.read_from_world(query.player_id, world)?;

        // 更新缓存
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(query.player_id, position);
        }

        Ok(position)
    }
}
```

### 命令组合

可以将多个命令组合成一个原子操作：

```rust
#[derive(Debug, Clone)]
pub struct CompositeCommand {
    commands: Vec<Box<dyn Command>>,
}

impl Command for CompositeCommand {
    fn command_type(&self) -> &'static str {
        "CompositeCommand"
    }
}

pub struct CompositeCommandHandler {
    cqrs: Arc<CqrsManager>,
}

impl CommandHandler<CompositeCommand> for CompositeCommandHandler {
    fn handle(
        &self,
        command: CompositeCommand,
        world: &mut World,
    ) -> Result<CommandResult, EventError> {
        // 执行所有命令（事务性）
        for cmd in command.commands {
            // 执行命令
            // 如果任何命令失败，回滚
        }
        
        Ok(CommandResult::success(None))
    }
}
```

## 最佳实践

### 1. 命令设计

- **单一职责**: 每个命令只做一件事
- **不可变**: 命令应该是不可变的
- **验证**: 在命令处理器中验证命令
- **幂等性**: 尽可能使命令幂等

### 2. 查询设计

- **只读**: 查询不应该修改状态
- **快速**: 查询应该快速执行
- **缓存**: 对频繁查询使用缓存
- **投影**: 使用事件投影优化查询性能

### 3. 处理器设计

- **单一职责**: 每个处理器只处理一种命令/查询
- **无状态**: 处理器应该是无状态的
- **错误处理**: 正确处理错误情况
- **日志记录**: 记录重要的操作

### 4. 性能优化

- **异步处理**: 对于耗时操作使用异步处理
- **批量处理**: 批量处理多个命令/查询
- **缓存**: 使用缓存减少重复查询
- **投影**: 使用事件投影优化查询

## 常见问题

### Q: 何时使用CQRS？

**A**: 适合场景：
- 读写操作频率差异大
- 需要独立的读写模型
- 需要事件溯源
- 需要复杂的查询优化

不适合场景：
- 简单的CRUD应用
- 读写操作频率相近
- 不需要事件溯源

### Q: 命令和事件的区别？

**A**: 
- **命令**: 表示意图，可能被拒绝
- **事件**: 表示已发生的事实，不可撤销

### Q: 如何处理命令失败？

**A**: 
- 返回`CommandResult::failure`
- 不产生事件
- 记录错误日志

### Q: 如何实现命令的撤销？

**A**: 
- 使用事件溯源系统
- 重放事件到之前的版本
- 或使用补偿命令

## 相关文档

- [事件溯源指南](./event_sourcing_guide.md)
- [领域驱动设计](../architecture.md#领域驱动设计)
- [ADR-0002: 领域驱动设计](../adr/0002-domain-driven-design.md)

