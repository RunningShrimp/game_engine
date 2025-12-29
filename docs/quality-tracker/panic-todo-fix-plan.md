# P0-1.6: 移除panic/unimplemented/todo豁免

**任务状态**: 🟡 进行中
**预估**: 1.5天
**总实例**: 66个

---

## 统计分析

### 分类统计

| 类型 | 数量 | 优先级 | 策略 |
|------|------|--------|------|
| panic! | 8 | P0 | 替换为Result |
| todo! | 40+ | P1 | 实现或文档化 |
| unimplemented! | 15+ | P1 | 实现或标记 |
| unreachable! | 3 | P2 | 添加注释 |

### 文件分布TOP 10

| 文件 | 数量 | 类型 | 优先级 |
|------|------|------|--------|
| scripting/lua_tests.rs | 12 | todo/panic | P2 (测试) |
| physics/spatial_partition.rs | 8 | todo/unimplemented | P1 |
| core/engine/input_handler.rs | 8 | panic | P1 (测试) |
| domain/tests/services_tests.rs | 7 | todo/panic | P2 (测试) |
| domain/tests/scene_tests.rs | 4 | todo/panic | P2 (测试) |
| domain/error_handling_tests.rs | 1 | todo | P2 (测试) |
| core/event_sourcing/registry.rs | 1 | unimplemented | P0 |
| ecs/component_validator.rs | 1 | unimplemented | P1 |
| xr/openxr_impl.rs | 1 | unimplemented | P1 |
| resources/manager.rs | 1 | unimplemented | P1 |

---

## 处理策略

### Category A: 测试代码中的panic（保留）

**文件**: input_handler.rs, lua_tests.rs, scene_tests.rs等

**策略**: 保留并添加注释

```rust
#[test]
fn test_touch_event() {
    match event {
        TouchEvent::TouchStart { x, y } => {
            assert_eq!(*x, 100.0);
        }
        _ => panic!("Expected TouchStart event"),  // ✅ OK: 测试代码
    }
}
```

**操作**:
- [x] 已确认测试代码中panic可接受
- [ ] 添加注释说明测试意图

### Category B: todo!（实现或文档化）

**策略1: 实现功能（核心模块）**
```rust
// ❌ Before
fn process_event(&mut self, event: Event) {
    todo!("Implement event processing")
}

// ✅ After
fn process_event(&mut self, event: Event) -> Result<(), EngineError> {
    match event {
        Event::Start => self.start(),
        Event::Stop => self.stop(),
        _ => Err(EngineError::UnsupportedEvent(format!("{:?}", event))),
    }
}
```

**策略2: 添加issue追踪（低优先级功能）**
```rust
// ✅ After
#[allow(clippy::todo)]  // TODO: Issue #1234 - 实现高级事件过滤
fn advanced_filter(&self) -> EventStream {
    todo!("Implement advanced filtering")
}
```

### Category C: unimplemented!（实现或标记废弃）

**策略1: 核心路径必须实现**
```rust
// ❌ Before
fn register(&mut self, handler: EventHandler) {
    unimplemented!("Event registration not implemented")
}

// ✅ After
fn register(&mut self, handler: EventHandler) -> Result<(), RegistrationError> {
    self.handlers.push(handler);
    Ok(())
}
```

**策略2: 实验性功能添加警告**
```rust
// ✅ After (实验性功能)
#[deprecated(note = "Experimental API, may change")]
#[allow(clippy::unimplemented)]
fn experimental_feature(&self) -> Result<Data, Error> {
    unimplemented!("Tracking in Issue #5678")
}
```

### Category D: unreachable!（添加上下文）

**策略**: 添加详细注释说明为何unreachable

```rust
// ❌ Before
match value {
    0 => 1,
    1 => 2,
    _ => unreachable!(),
}

// ✅ After
match value {
    0 => 1,
    1 => 2,
    _ => unreachable!("Invalid value: {}, should be 0 or 1", value),
}
```

---

## 执行计划

### 阶段1: 核心模块（0.5天）

**文件**:
- core/event_sourcing/registry.rs: 1个unimplemented
- xr/openxr_impl.rs: 1个unimplemented
- ecs/component_validator.rs: 1个unimplemented

**操作**:
1. 实现unimplemented函数或返回Error
2. 文档化todo功能并创建issue
3. 添加测试覆盖

### 阶段2: 物理模块（0.5天）

**文件**: physics/spatial_partition.rs: 8个

**操作**:
1. 实现todo功能
2. 或添加issue追踪

### 阶段3: 其他模块（0.5天）

**文件**: resources/, scripting/, domain/tests/

**操作**:
1. 测试代码添加注释
2. 其他功能实现或文档化

### 阶段4: 移除全局豁免（0.1小时）

从lib.rs移除：
```rust
#![allow(
    clippy::panic,           // ← 移除
    clippy::unimplemented,   // ← 移除
    clippy::todo,            // ← 移除
    clippy::unreachable,     // ← 移除
    // ...
)]
```

---

## 实施示例

### 示例1: core/event_sourcing/registry.rs

```rust
// Before:
fn register_aggregate(&mut self, aggregate: Aggregate) {
    unimplemented!("Aggregate registration")
}

// After:
fn register_aggregate(&mut self, aggregate: Aggregate) -> Result<(), RegistryError> {
    if self.aggregates.contains_key(&aggregate.name()) {
        return Err(RegistryError::AlreadyRegistered(aggregate.name()));
    }
    self.aggregates.insert(aggregate.name(), aggregate);
    Ok(())
}
```

### 示例2: physics/spatial_partition.rs

```rust
// Before:
fn optimize_partitions(&mut self) {
    todo!("Optimize spatial partitioning")
}

// After (Option 1: 实现功能):
fn optimize_partitions(&mut self) -> Result<(), OptimizationError> {
    // 实现优化逻辑
    self.rebalance_partitions()?;
    Ok(())
}

// After (Option 2: 添加issue追踪):
#[allow(clippy::todo)]  // TODO: Issue #2345 - 实现空间分区优化算法
fn optimize_partitions(&mut self) {
    todo!("Optimize spatial partitioning - see Issue #2345")
}
```

---

## 验收标准

- [ ] 核心路径（core/, ecs/, physics/）无panic/todo/unimplemented
- [ ] 所有todo都有issue追踪或实现计划
- [ ] 测试代码panic有注释说明
- [ ] 从lib.rs移除4个豁免
- [ ] `cargo clippy`无相关警告
- [ ] 所有测试通过

---

## 追踪表

| 文件 | 数量 | 类型 | 状态 | Issue |
|------|------|------|------|-------|
| core/event_sourcing/registry.rs | 1 | unimplemented | ⚪ 待处理 | |
| xr/openxr_impl.rs | 1 | unimplemented | ⚪ 待处理 | |
| ecs/component_validator.rs | 1 | unimplemented | ⚪ 待处理 | |
| physics/spatial_partition.rs | 8 | todo | ⚪ 待处理 | |
| scripting/lua_tests.rs | 12 | panic/todo | 🟢 保留(测试) | |
| core/engine/input_handler.rs | 8 | panic | 🟢 保留(测试) | |
| domain/tests/*.rs | 15+ | panic/todo | 🟢 保留(测试) | |
| ... | ... | ... | ... | |

---

## 风险与缓解

### 风险1: 功能未实现可能破坏现有功能
**缓解**: 添加返回Error的stub实现，保持API兼容

### 风险2: 测试代码panic移除可能降低测试质量
**缓解**: 仅移除生产代码panic，测试代码保留并添加注释

### 风险3: todo功能实现可能超出时间预算
**缓解**: 优先实现核心路径，低优先级功能添加issue追踪

---

**开始时间**: 待P0-1.5完成后
**预计完成**: 1.5天
**状态**: 准备就绪
