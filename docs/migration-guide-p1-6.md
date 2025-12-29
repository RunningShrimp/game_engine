# P1-6 API变更迁移指南

**版本**: 1.0
**日期**: 2025-12-28
**影响范围**: Event系统, Platform适配器, 错误处理

---

## 概述

P1-6项目替换了269+处unsafe的`unwrap()`和`expect()`调用，改进了错误处理。部分API签名已变更，需要调用者相应更新。

**变更原则**:
- ✅ 更好的错误消息
- ✅ 类型安全的错误传播
- ✅ 优雅的错误恢复
- ⚠️ 部分API现在返回`Result`

---

## 受影响的API

### 1. EventId::now()

**位置**: `core/event_sourcing.rs`

**变更前**:
```rust
let event_id = EventId::now(sequence);
```

**变更后**:
```rust
// 方式1: 使用 ? 操作符（推荐）
let event_id = EventId::now(sequence)?;

// 方式2: 使用 expect
let event_id = EventId::now(sequence)
    .expect("Failed to create event ID");

// 方式3: 显式处理错误
let event_id = match EventId::now(sequence) {
    Ok(id) => id,
    Err(e) => {
        // 处理错误
        return Err(e.into());
    }
};
```

**错误类型**: `EventError::TimeError`

**迁移步骤**:
1. 搜索所有`EventId::now(`调用
2. 在调用处添加`?`或`.expect()`
3. 确保函数返回`Result`

---

### 2. EventBus::subscribe()

**位置**: `core/event_sourcing.rs`

**变更前**:
```rust
bus.subscribe::<TestEvent>(handler);
```

**变更后**:
```rust
// 方式1: 使用 ? 操作符（推荐）
bus.subscribe::<TestEvent>(handler)?;

// 方式2: 使用 expect
bus.subscribe::<TestEvent>(handler)
    .expect("Failed to subscribe to event");

// 方式3: 忽略错误（不推荐）
if let Err(e) = bus.subscribe::<TestEvent>(handler) {
    log::error!("Failed to subscribe: {:?}", e);
}
```

**错误类型**: `EventError::LockError`

---

### 3. EventBus::publish()

**位置**: `core/event_sourcing.rs`

**变更前**:
```rust
bus.publish(event);
```

**变更后**:
```rust
// 方式1: 使用 ? 操作符（推荐）
bus.publish(event)?;

// 方式2: 使用 expect
bus.publish(event)
    .expect("Failed to publish event");

// 方式3: 忽略错误（不推荐）
if let Err(e) = bus.publish(event) {
    log::error!("Failed to publish: {:?}", e);
}
```

**错误类型**: `EventError::LockError`

---

### 4. EventSourcingManager getter方法

**位置**: `core/event_sourcing.rs`

**受影响的方法**:
- `get_event_history()` → `Result<Vec<StoredEvent>, EventError>`
- `get_aggregate_history()` → `Result<Vec<StoredEvent>, EventError>`
- `get_aggregate_snapshots()` → `Result<Vec<Snapshot>, EventError>`

**变更前**:
```rust
let events = manager.get_event_history(&aggregate_id);
```

**变更后**:
```rust
// 方式1: 使用 ? 操作符（推荐）
let events = manager.get_event_history(&aggregate_id)?;

// 方式2: 使用 expect
let events = manager
    .get_event_history(&aggregate_id)
    .expect("Failed to get event history");

// 方式3: 显式处理错误
let events = match manager.get_event_history(&aggregate_id) {
    Ok(events) => events,
    Err(e) => {
        log::error!("Failed to get event history: {:?}", e);
        Vec::new() // 降级处理
    }
};
```

---

### 5. PlatformAdapter::new()

**位置**: `platform/adapter.rs`

**变更前**:
```rust
let adapter = PlatformAdapter::new();
```

**变更后**:
```rust
// 方式1: 使用 Result（推荐）
let adapter = PlatformAdapter::new()?;

// 方式2: 使用新的with_fallbacks方法
let adapter = PlatformAdapter::new_with_fallbacks();

// 方式3: 显式处理错误
let adapter = match PlatformAdapter::new() {
    Ok(adapter) => adapter,
    Err(e) => {
        eprintln!("Platform adapter error: {:?}", e);
        // 使用最小化适配器
        PlatformAdapter::new_with_fallbacks()
    }
};
```

**新增方法**:
- `PlatformAdapter::new_with_fallbacks()` - 自动降级，总是成功

**错误类型**: `PlatformAdapterError`
- `FilesystemError(String)`
- `InputError(String)`

---

## 迁移策略

### 阶段1: 准备（1天）

1. **备份代码**
   ```bash
   git checkout -b backup-before-p1-6-migration
   git push origin backup-before-p1-6-migration
   ```

2. **创建迁移分支**
   ```bash
   git checkout -b feature/p1-6-api-migration
   ```

3. **识别所有调用点**
   ```bash
   # EventId::now
   grep -r "EventId::now(" --include="*.rs" > api-migration-eventid.txt

   # EventBus::subscribe
   grep -r "\.subscribe<" --include="*.rs" src/ > api-migration-subscribe.txt

   # EventBus::publish
   grep -r "\.publish(" --include="*.rs" src/ > api-migration-publish.txt

   # PlatformAdapter::new
   grep -r "PlatformAdapter::new()" --include="*.rs" > api-migration-platform.txt
   ```

### 阶段2: 迁移（2-3天）

#### 优先级P0（关键路径）

1. **EventId::now** - 影响所有事件生成
2. **EventBus::subscribe** - 影响所有事件订阅
3. **EventBus::publish** - 影响所有事件发布

#### 优先级P1（重要）

4. **EventSourcingManager getters** - 影响查询
5. **PlatformAdapter::new** - 影响平台初始化

### 阶段3: 测试（1-2天）

1. **单元测试**
   ```bash
   cargo test -p game_engine --lib
   ```

2. **集成测试**
   ```bash
   cargo test --test '*'
   ```

3. **手动测试**
   - 启动引擎
   - 加载场景
   - 运行游戏循环

### 阶段4: 提交（1天）

1. **代码审查**
2. **文档更新**
3. **合并到主分支**

---

## 常见问题

### Q1: 为什么不保持API向后兼容？

**A**: 我们优先考虑错误处理的质量，而不是向后兼容性。`unwrap()`会隐藏错误，导致难以调试的panic。

### Q2: 我可以继续使用`unwrap()`吗？

**A**: 在测试代码中可以。在生产代码中强烈建议使用`?`或`expect()`。

### Q3: `?`操作符在哪里可用？

**A**: 只在返回`Result`的函数中可用。如果函数不返回`Result`，需要使用`.expect()`或显式错误处理。

### Q4: 如何处理平台初始化失败？

**A**: 使用`new_with_fallbacks()`方法，它会提供最小化功能的适配器。

### Q5: 迁移后性能会受影响吗？

**A**: 不会。`Result`是零成本抽象，错误路径不优化（cold path），对性能影响可忽略。

---

## 示例：完整迁移案例

### 迁移前

```rust
use crate::core::event_sourcing::{EventId, EventBus, EventSourcingManager};

pub struct GameSystem {
    bus: EventBus,
    manager: EventSourcingManager,
}

impl GameSystem {
    pub fn new() -> Self {
        let bus = EventBus::new();
        let manager = EventSourcingManager::new();

        // 订阅事件
        bus.subscribe::<GameEvent>(Self::handle_game_event);

        Self { bus, manager }
    }

    pub fn start(&mut self) {
        // 创建事件ID
        let event_id = EventId::now(0);

        // 发布事件
        let event = GameEvent::Started { id: event_id };
        self.bus.publish(event);

        // 获取历史
        let history = self.manager.get_event_history("game-1");
        for event in history {
            println!("{:?}", event);
        }
    }

    fn handle_game_event(event: GameEvent) {
        println!("Game event: {:?}", event);
    }
}
```

### 迁移后

```rust
use crate::core::event_sourcing::{EventId, EventBus, EventSourcingManager, EventError};

pub struct GameSystem {
    bus: EventBus,
    manager: EventSourcingManager,
}

impl GameSystem {
    pub fn new() -> Result<Self, EventError> {
        let bus = EventBus::new();
        let manager = EventSourcingManager::new();

        // 订阅事件 - 添加 ?
        bus.subscribe::<GameEvent>(Self::handle_game_event)?;

        Ok(Self { bus, manager })
    }

    pub fn start(&mut self) -> Result<(), EventError> {
        // 创建事件ID - 添加 ?
        let event_id = EventId::now(0)?;

        // 发布事件 - 添加 ?
        let event = GameEvent::Started { id: event_id };
        self.bus.publish(event)?;

        // 获取历史 - 添加 ?
        let history = self.manager.get_event_history("game-1")?;
        for event in history {
            println!("{:?}", event);
        }

        Ok(())
    }

    fn handle_game_event(event: GameEvent) {
        println!("Game event: {:?}", event);
    }
}
```

**关键变更**:
1. `new()` 返回 `Result<Self, EventError>`
2. `start()` 返回 `Result<(), EventError>`
3. 所有`EventId::now()`, `subscribe()`, `publish()`, `get_*()`调用添加`?`

---

## 工具和脚本

### 自动化迁移脚本

```bash
#!/bin/bash
# migrate-p1-6.sh

# 查找所有需要迁移的文件
find src/ -name "*.rs" -exec grep -l "EventId::now\|\.subscribe<\|\.publish(" {} \;

# 提示用户手动审查
echo "请审查上述文件并添加适当的错误处理"
```

### Git命令

```bash
# 查看变更
git diff HEAD

# 查看特定文件的变更
git diff src/core/event_sourcing.rs

# 暂存变更
git add src/

# 提交变更
git commit -m "Migrate to P1-6 API changes"
```

---

## 检查清单

迁移完成后，请确认：

- [ ] 所有`EventId::now(`调用已添加错误处理
- [ ] 所有`.subscribe<`调用已添加错误处理
- [ ] 所有`.publish(`调用已添加错误处理
- [ ] 所有`get_*(`调用已添加错误处理
- [ ] 所有`PlatformAdapter::new()`调用已更新
- [ ] 所有函数签名已更新为返回`Result`
- [ ] 所有测试通过
- [ ] 无编译警告
- [ ] 代码已审查

---

## 支持

如果遇到问题：

1. 查看综合报告: `docs/code-quality/p1-6-batch3-final-comprehensive-report.md`
2. 查看技术模式: 报告中的"技术模式总结"章节
3. 查看示例代码: 本文档的"示例"章节
4. 联系技术负责人

---

## 附录：错误类型参考

### EventError

```rust
pub enum EventError {
    LockError(String),
    TimeError(String),
    // ... 其他错误类型
}
```

### PlatformAdapterError

```rust
pub enum PlatformAdapterError {
    #[cfg(target_arch = "wasm32")]
    FilesystemError(String),
    #[cfg(target_arch = "wasm32")]
    InputError(String),
}
```

---

**文档版本**: 1.0
**最后更新**: 2025-12-28
**作者**: Claude Code (P1-6项目)

💡 **提示**: 迁移过程中遇到问题时，请优先使用`?`操作符，它是最简洁和安全的错误处理方式。
