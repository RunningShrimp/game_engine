# Clippy豁免迁移追踪表

**创建日期**: 2025-12-28
**基线版本**: v0.1.0
**代码库规模**: 680个Rust源文件，190,898行代码

---

## 当前状态

### lib.rs中的豁免清单
```rust
#![allow(
    // TODO: 将在后续迭代中移除这些允许
    unused_variables,        // 需统计
    unused_mut,              // 需统计
    dead_code,               // 需统计
    unreachable_pub,         // 需统计
    non_snake_case,          // 需统计
    non_camel_case_types,    // 需统计
    deprecated,              // 需统计
    while_true,              // 需统计
    non_upper_case_globals,  // 需统计
    // clippy lint将逐步修复
    clippy::unwrap_used,     // 已发现1415个
    clippy::expect_used,     // 包含在unwrap统计中
    clippy::panic,           // 需统计
    clippy::unimplemented,   // 需统计
    clippy::todo,            // 需统计
    clippy::unreachable,     // 需统计
    clippy::indexing_slicing,// 需统计
)]
```

---

## 统计数据

| 豁免类型 | 当前数量 | 目标 | 状态 |
|---------|---------|------|------|
| unwrap/expect | 1415 | <500 | 🔴 未开始 |
| unused_variables | TBD | 0 | ⚪ 待统计 |
| unused_mut | TBD | 0 | ⚪ 待统计 |
| dead_code | TBD | 最小化 | ⚪ 待统计 |
| unreachable_pub | TBD | 0 | ⚪ 待统计 |
| 命名规范 | TBD | 0 | ⚪ 待统计 |
| panic/todo/unimplemented | TBD | 0 | ⚪ 待统计 |

---

## 迁移计划

### 批次1: unused_variables/unused_mut（0.5天）
- [ ] 统计当前数量
- [ ] 修复或使用 `_` 前缀
- [ ] 验证编译通过

### 批次2: dead_code/unreachable_pub（1天）
- [ ] 标记为 `#[allow(dead_code)]` 在具体函数上
- [ ] 或添加 `#[deprecated]` 注释
- [ ] 移除lib.rs级别的豁免

### 批次3: unwrap_used/expect_used（3天）
- [ ] 批次1: 核心模块（470个）
- [ ] 批次2: 渲染和网络（270个）
- [ ] 批次3: 其他模块
- [ ] 使用 `game_engine/src/error/convenience.rs` 中的安全函数

### 批次4: 命名规范（1天）
- [ ] 重命名不符合规范的类型/变量

### 批次5: panic/unimplemented/todo（1.5天）
- [ ] 替换为 `Result<_, EngineError>`

---

## 关键文件

### 高优先级文件（unwrap/expect密集）
- `game_engine/src/error/concurrency_tests.rs` - 31个
- `game_engine/src/domain/tests/services_tests.rs` - 76个
- `game_engine/src/domain/tests/scene_tests.rs` - 92个
- `game_engine/src/profiling/tracy.rs` - 需分析

### 错误处理工具库
- `game_engine/src/error/convenience.rs` - 安全错误处理函数

---

## 进度追踪

| 日期 | 批次 | 完成数 | 剩余 | 状态 |
|------|------|--------|------|------|
| 2025-12-28 | - | - | 1415 | 🟡 初始化 |

---

## 备注

- **工具**: cargo clippy, grep
- **测试**: 每批完成后运行 `cargo test`
- **CI**: 集成到CI pipeline
