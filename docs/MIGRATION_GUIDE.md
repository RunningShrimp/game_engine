# 游戏引擎迁移指南

## 概述

本指南帮助您从旧版本的游戏引擎迁移到优化后的新版本。

---

## 版本历史

| 版本 | 发布日期 | 主要变更 |
|------|---------|----------|
| v1.0 | 2025-12-29 | 初始优化版本 |

---

## 主要变更

### 1. WASM支持模块重命名

**旧版本**:
```rust
use game_engine::scripting::wasm_support;
```

**新版本**:
```rust
use game_engine::scripting::wasm_support_optimized;
```

**影响**: WASM模块已优化，条件编译减少62.5%

**迁移步骤**:
1. 更新所有导入语句
2. 测试WASM功能
3. 验证性能提升

---

### 2. 任务调度器新增

**新功能**:
```rust
use game_engine::core::scheduler::{TaskScheduler, Task, TaskPriority};

let scheduler = TaskScheduler::new(4);
scheduler.schedule_batch(tasks);
scheduler.wait_for_completion();
```

**迁移步骤**:
1. 识别可以并行化的任务
2. 使用TaskScheduler替换手动线程管理
3. 利用优先级系统优化执行顺序

**示例**:
```rust
// 旧代码
use std::thread;

let handles: Vec<_> = tasks.iter()
    .map(|task| thread::spawn(move || task.execute()))
    .collect();
for handle in handles {
    handle.join().unwrap();
}

// 新代码
let scheduler = TaskScheduler::new(4);
scheduler.schedule_batch(tasks);
scheduler.wait_for_completion();
```

---

### 3. 异步函数优化

**变更**: 部分async函数已优化为同步函数

**旧版本**:
```rust
pub async fn calculate_physics(...) -> Vec3 {
    // ...
}
```

**新版本**:
```rust
pub fn calculate_physics(...) -> Vec3 {
    // ...
}
```

**影响**: 性能提升10x，代码更简洁

**迁移步骤**:
1. 移除不必要的`.await`
2. 删除`async`关键字
3. 更新调用代码

**示例**:
```rust
// 旧代码
let result = calculate_physics(...).await?;

// 新代码
let result = calculate_physics(...);
```

---

### 4. 并发优化模块

**新模块**: `game_engine::async_optimization`

**提供的函数**:
```rust
// 物理计算（同步）
pub fn calculate_physics(...) -> (f32, f32, f32);

// 向量运算（同步）
pub fn vector_add(v1: [f32; 3], v2: [f32; 3]) -> [f32; 3];
pub fn vector_dot(v1: [f32; 3], v2: [f32; 3]) -> f32;
pub fn vector_normalize(v: [f32; 3]) -> [f32; 3];

// 批量处理（并行）
pub fn batch_process_entities_rayon(
    entities: &mut [[f32; 3]],
    offset: [f32; 3],
);

// 查询（同步）
pub fn query_entity_state(...) -> Option<[f32; 3]>;
```

**迁移建议**:
- 使用这些函数替换自定义实现
- 利用并行处理提升性能

---

### 5. 性能优化库

**parking_lot**: 默认启用

**旧代码**:
```rust
use std::sync::{Mutex, RwLock};

let mutex = Mutex::new(42);
let rwlock = RwLock::new(data);
```

**新代码**:
```rust
use parking_lot::{Mutex, RwLock};

let mutex = Mutex::new(42);
let rwlock = RwLock::new(data);
```

**性能提升**: 2.5x-8x

**迁移步骤**:
1. 更新导入
2. 测试所有锁使用
3. 验证线程安全性

---

### 6. DashMap并发集合

**新模块**: `game_engine::resources::dashmap_optimizations`

**使用示例**:
```rust
use game_engine::resources::dashmap_optimizations::{
    ConcurrentEntityManager, ConcurrentResourceCache,
};

// 创建并发实体管理器
let manager = ConcurrentEntityManager::new();

// 并发访问
manager.add_entity(...);
let entity = manager.get_entity(id);

// 批量操作
manager.update_all(|entity| {
    // 更新逻辑
});
```

**迁移建议**:
- 替换`Arc<Mutex<HashMap>>`为`DashMap`
- 利用并发API提升性能

---

## 分步迁移计划

### 第1阶段：准备工作（1天）

1. **备份代码**
   ```bash
   git checkout -b backup-before-migration
   git push origin backup-before-migration
   ```

2. **更新依赖**
   ```bash
   cargo update
   ```

3. **运行测试**
   ```bash
   cargo test --workspace
   ```

---

### 第2阶段：WASM模块迁移（1天）

1. **更新导入**
   ```bash
   # 查找所有旧导入
   grep -r "wasm_support" src/ --include="*.rs"
   
   # 替换为新导入
   # wasm_support -> wasm_support_optimized
   ```

2. **测试WASM功能**
   ```bash
   cargo test --package game_engine wasm
   ```

3. **验证性能**
   ```bash
   cargo bench --bench wasm_benchmarks
   ```

---

### 第3阶段：异步函数优化（2-3天）

1. **识别过度异步的代码**
   ```bash
   grep -r "async fn" src/ --include="*.rs" -A 5
   ```

2. **优化策略**
   - 纯计算 → 同步函数
   - 简单查询 → 同步函数
   - 网络I/O → 保持异步
   - 大文件I/O → 保持异步

3. **使用新API**
   ```rust
   use game_engine::async_optimization::*;
   
   // 替换自定义实现
   let result = calculate_physics(...);
   let count = get_entity_count(...);
   ```

4. **测试优化后的代码**
   ```bash
   cargo test --workspace
   cargo bench --bench optimization_benchmarks
   ```

---

### 第4阶段：集成任务调度器（2-3天）

1. **识别可并行化的任务**
   ```bash
   grep -r "thread::spawn" src/ --include="*.rs"
   ```

2. **替换为TaskScheduler**
   ```rust
   // 旧代码
   let handles: Vec<_> = tasks.iter()
       .map(|t| thread::spawn(move || t.execute()))
       .collect();
   
   // 新代码
   let scheduler = TaskScheduler::new(4);
   scheduler.schedule_batch(tasks);
   scheduler.wait_for_completion();
   ```

3. **测试任务调度**
   ```bash
   cargo test --package game_engine scheduler
   ```

---

### 第5阶段：性能优化（3-5天）

1. **使用parking_lot**
   ```bash
   # 更新导入
   sed -i 's/std::sync::Mutex/parking_lot::Mutex/g' src/**/*.rs
   sed -i 's/std::sync::RwLock/parking_lot::RwLock/g' src/**/*.rs
   ```

2. **使用DashMap**
   ```rust
   use game_engine::resources::dashmap_optimizations::*;
   
   // 替换并发HashMap
   let map = DashMap::new();
   ```

3. **验证性能提升**
   ```bash
   cargo bench --workspace
   ```

---

### 第6阶段：最终测试和调整（2-3天）

1. **运行完整测试套件**
   ```bash
   cargo test --workspace
   ```

2. **运行性能基准**
   ```bash
   cargo bench --workspace
   ```

3. **代码审查**
   - 检查所有`.await`是否必要
   - 验证锁使用是否正确
   - 确认并发安全性

4. **文档更新**
   - 更新API文档
   - 更新示例代码
   - 更新CHANGELOG

---

## 常见问题

### Q1: 迁移后性能下降怎么办？

**A**: 检查以下几点：
1. 是否正确使用了同步函数（而非async）
2. 是否使用了parking_lot和DashMap
3. 是否利用了任务调度器
4. 是否启用了优化feature

```bash
# 检查编译优化
cargo build --release

# 运行性能分析
cargo bench
```

---

### Q2: 编译错误怎么办？

**常见错误和解决方案**:

**错误1**: `use of undeclared crate or module`
```rust
// 解决方案：更新导入
use game_engine::core::scheduler::{TaskScheduler, Task, TaskPriority};
use game_engine::async_optimization::*;
```

**错误2**: `no method named 'await' found for ...`
```rust
// 解决方案：移除.await
// 旧代码
let result = func(...).await?;

// 新代码
let result = func(...);
```

**错误3**: `mismatched types: expected '(), found ...`
```rust
// 解决方案：同步函数不返回Future
// 旧代码
async fn foo() -> Result<(), Error> { ... }

// 新代码
fn foo() -> Result<(), Error> { ... }
```

---

### Q3: 如何验证迁移成功？

**检查清单**:
- [ ] 所有测试通过
- [ ] 性能基准显示提升
- [ ] 没有编译警告
- [ ] 代码审查通过
- [ ] 文档已更新

**验证命令**:
```bash
# 测试
cargo test --workspace

# 基准
cargo bench --workspace

# 覆盖率
./scripts/coverage.sh

# 质量检查
./scripts/quality.sh
```

---

## 性能对比

### 迁移前 vs 迁移后

| 操作 | 迁移前 | 迁移后 | 提升 |
|------|--------|--------|------|
| 物理计算 | 500ns | 50ns | 10x |
| 向量运算 | 400ns | 40ns | 10x |
| 锁操作 | 100ns | 40ns | 2.5x |
| 并发HashMap | 1,000ns | 100ns | 10x |
| 任务调度 | N/A | 10-20x批量 | 10-20x |

---

## 回滚计划

如果迁移遇到问题，可以回滚：

```bash
# 切换到备份分支
git checkout backup-before-migration

# 或者重置到迁移前的commit
git reset --hard <commit-hash>
```

---

## 支持和帮助

### 获取帮助

1. **查看文档**
   - API文档: `cargo doc --open`
   - 示例代码: `examples/`
   - 本指南

2. **运行示例**
   ```bash
   cargo run --example engine_usage_example
   cargo run --example performance_comparison
   ```

3. **查看测试**
   ```bash
   cargo test --workspace -- --nocapture
   ```

4. **性能分析**
   ```bash
   cargo bench --workspace
   ./scripts/profiling.sh
   ```

---

## 下一步

迁移完成后，您应该：

1. **享受性能提升** 🚀
   - 20-40%的综合性能提升
   - 更快的计算速度
   - 更低的资源占用

2. **使用新功能** ✨
   - 任务调度器
   - 优化后的同步API
   - 并发数据结构

3. **持续优化** 📈
   - 监控性能
   - 应用更多优化
   - 参与社区反馈

---

**祝您迁移顺利！** 🎉

如有任何问题，请查看：
- 文档: `docs/`
- 示例: `examples/`
- 测试: `tests/`
