# 游戏引擎故障排除指南

## 概述

本指南提供了游戏引擎常见问题的诊断和解决方案。

---

## 快速诊断流程

### 第1步：检查环境

```bash
# 检查Rust版本
rustc --version

# 检查Cargo版本
cargo --version

# 检查系统
uname -a
```

**要求**:
- Rust >= 1.85.0
- Cargo >= 1.85.0
- 支持的操作系统: Linux, macOS, Windows

---

### 第2步：清理构建

```bash
# 清理所有构建产物
cargo clean

# 清理并重新构建
cargo clean && cargo build --release

# 更新依赖
cargo update
```

---

### 第3步：运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定测试
cargo test --package game_engine <test_name>

# 显示测试输出
cargo test --workspace -- --nocapture
```

---

## 常见问题

### 编译问题

#### 问题1: 编译错误 - 找不到模块

**错误信息**:
```
error[E0433]: failed to resolve: use of undeclared crate or module `xxx`
```

**原因**: 模块路径错误或模块未启用

**解决方案**:
1. 检查模块导入
   ```rust
   // 确保路径正确
   use game_engine::core::scheduler::{TaskScheduler, Task, TaskPriority};
   ```

2. 检查feature标志
   ```toml
   # Cargo.toml
   [features]
   default = ["wasm", "parking_lot"]
   ```

3. 检查模块声明
   ```rust
   // mod.rs 或 lib.rs
   pub mod scheduler;
   ```

---

#### 问题2: 类型不匹配

**错误信息**:
```
error[E0308]: mismatched types
```

**原因**: API变更，特别是async到sync的迁移

**解决方案**:
```rust
// 旧代码（async）
let result = calculate_physics(...).await?;

// 新代码（sync）
let result = calculate_physics(...);
```

---

#### 问题3: 特征未实现

**错误信息**:
```
error[E0599]: the method `xxx` exists but the following trait bounds were not satisfied
```

**原因**: 缺少trait导入或类型约束

**解决方案**:
1. 导入所需的trait
   ```rust
   use std::fmt::Debug;
   use std::clone::Clone;
   ```

2. 添加类型约束
   ```rust
   fn func<T: Debug + Clone>(value: T) {
       // ...
   }
   ```

---

### 运行时问题

#### 问题4: 任务调度器死锁

**症状**: 程序挂起，无响应

**诊断**:
```bash
# 使用gdb或lldb
cargo build && gdb target/debug/my_game
# 或
lldb target/debug/my_game

# 检查线程
thread apply all bt
```

**解决方案**:
1. 避免在任务中等待其他任务
2. 使用正确的优先级
3. 检查循环依赖

```rust
// ❌ 错误：任务A等待任务B，任务B等待任务A
let task_a = Task::new("a", Box::new(|| {
    // 等待任务B
}), TaskPriority::High);

// ✅ 正确：任务独立执行
let task_a = Task::new("a", Box::new(|| {
    // 独立逻辑
}), TaskPriority::High);
```

---

#### 问题5: 性能下降

**症状**: 帧率低，卡顿

**诊断**:
```bash
# 运行性能分析
cargo bench --workspace

# 使用flamegraph
cargo install flamegraph
cargo flamegraph --example my_example
```

**常见原因和解决方案**:

**原因1**: 过度使用async
```rust
// ❌ 错误：纯计算使用async
pub async fn calculate_physics(...) -> Vec3 { /* ... */ }

// ✅ 正确：使用同步函数
pub fn calculate_physics(...) -> Vec3 { /* ... */ }
```

**原因2**: 锁竞争
```rust
// ❌ 错误：频繁锁同一个数据
for _ in 0..10000 {
    let mut data = mutex.lock().unwrap();
    data.value += 1;
}

// ✅ 正确：批量处理
{
    let mut data = mutex.lock().unwrap();
    for _ in 0..10000 {
        data.value += 1;
    }
}
```

**原因3**: 内存分配
```rust
// ❌ 错误：频繁分配
for i in 0..10000 {
    let vec = vec![0; 1000];
    process(vec);
}

// ✅ 正确：重用缓冲区
let mut vec = vec![0; 1000];
for i in 0..10000 {
    vec.clear();
    process(&mut vec);
}
```

---

#### 问题6: 内存泄漏

**症状**: 内存使用持续增长

**诊断**:
```bash
# 使用valgrind
cargo build && valgrind --leak-check=full target/debug/my_game

# 使用heaptrack
cargo install heaptrack
heaptrack target/release/my_game
```

**解决方案**:
1. 检查循环引用
   ```rust
   // ❌ 可能导致循环引用
   struct Node {
       next: Option<Rc<RefCell<Node>>>,
   }
   
   // ✅ 使用弱引用
   struct Node {
       next: Option<Rc<RefCell<Node>>>,
       prev: Option<Weak<RefCell<Node>>>,
   }
   ```

2. 正确清理资源
   ```rust
   impl Drop for MyResource {
       fn drop(&mut self) {
           // 清理逻辑
       }
   }
   ```

---

### WASM问题

#### 问题7: WASM模块加载失败

**错误信息**:
```
Failed to compile WASM module: ...
```

**诊断**:
```bash
# 检查WASM字节码
wasm-objdump -x module.wasm

# 验证WASM有效性
wasm-validate module.wasm
```

**解决方案**:
1. 检查WASM feature是否启用
   ```toml
   [features]
   wasm = ["wasmtime"]
   ```

2. 验证WASM模块
   ```rust
   use game_engine::scripting::wasm_support_optimized::*;
   
   let mut runtime = WasmRuntime::new()?;
   runtime.load_module("test", &wasm_bytes)?;
   ```

---

#### 问题8: WASM性能问题

**症状**: WASM执行速度慢

**诊断**:
```rust
use std::time::Instant;

let start = Instant::now();
runtime.call_function("module", "func", args)?;
let duration = start.elapsed();
println!("WASM调用耗时: {:?}", duration);
```

**优化建议**:
1. 批量调用
2. 使用共享内存
3. 预编译模块
4. 缓存结果

---

### 并发问题

#### 问题9: 数据竞争

**错误信息**:
```
thread 'main' panicked at 'RwLock poisoned: ...'
```

**诊断**:
```bash
# 使用loom检查并发
cargo test --features loom

# 使用ThreadSanitizer
RUSTFLAGS="-Z sanitizer=thread" cargo test
```

**解决方案**:
1. 使用正确的同步原语
   ```rust
   // ❌ 错误：多个写锁
   let mut write1 = rwlock.write().unwrap();
   let mut write2 = rwlock.write().unwrap(); // 死锁
   
   // ✅ 正确：读写分离
   let read1 = rwlock.read().unwrap();
   let read2 = rwlock.read().unwrap(); // OK
   ```

2. 使用原子操作
   ```rust
   use std::sync::atomic::{AtomicUsize, Ordering};
   
   let counter = AtomicUsize::new(0);
   counter.fetch_add(1, Ordering::SeqCst);
   ```

---

#### 问题10: 死锁

**症状**: 程序挂起

**诊断**:
```bash
# 使用gdb
gdb target/debug/my_game
(gdb) run
# 按Ctrl+C
(gdb) thread apply all bt
```

**常见模式**:
1. 锁顺序不一致
2. 嵌套锁
3. 任务相互等待

**解决方案**:
```rust
// ❌ 错误：锁顺序不一致
thread1: lock(A) -> lock(B)
thread2: lock(B) -> lock(A)  // 死锁

// ✅ 正确：一致的锁顺序
thread1: lock(A) -> lock(B)
thread2: lock(A) -> lock(B)  // 不会死锁
```

---

## 性能优化

### 诊断工具

```bash
# CPU性能分析
cargo install flamegraph
cargo flamegraph --example my_example

# 内存分析
cargo install heaptrack
heaptrack target/release/my_game

# 性能基准
cargo bench --workspace

# 覆盖率
./scripts/coverage.sh
```

### 优化检查清单

- [ ] 移除不必要的async
- [ ] 使用parking_lot替代std::sync
- [ ] 使用DashMap替代Arc<Mutex<HashMap>>
- [ ] 批量操作而非逐个处理
- [ ] 缓存频繁计算的结果
- [ ] 使用对象池减少分配
- [ ] 优化热点路径
- [ ] 使用SIMD指令

---

## 调试技巧

### 日志记录

```rust
use tracing::{info, debug, warn, error};

#[tracing::instrument]
fn my_function(param: &str) {
    info!("开始执行函数: {}", param);
    debug!("参数详情: {:?}", param);
    // ...
    info!("函数执行完成");
}
```

### 断点调试

```bash
# 使用lldb
cargo build
lldb target/debug/my_game
(lldb) breakpoint set --name my_function
(lldb) run
(lldb) step
(lldb) print variable_name
```

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        let result = my_function(42);
        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic(expected = "specific error")]
    fn test_panic_case() {
        my_function_that_panics();
    }
}
```

---

## 获取帮助

### 社区资源

1. **GitHub Issues**: 报告bug
2. **文档**: `docs/`
3. **示例**: `examples/`
4. **测试**: `tests/`

### 日志收集

提交问题时，请提供：

```bash
# 系统信息
uname -a
rustc --version
cargo --version

# 编译日志
cargo build 2>&1 > build.log

# 测试日志
cargo test --workspace 2>&1 > test.log

# 性能基准
cargo bench --workspace > bench.txt
```

---

## 预防措施

### 代码审查

1. **检查async使用**
   - 纯计算应该是同步的
   - I/O操作可以是异步的

2. **检查锁使用**
   - 避免嵌套锁
   - 保持一致的锁顺序
   - 最小化锁持有时间

3. **检查资源管理**
   - 实现Drop trait
   - 使用RAII模式
   - 避免循环引用

### 测试策略

1. **单元测试**: 测试单个函数
2. **集成测试**: 测试模块交互
3. **压力测试**: 测试极限情况
4. **性能测试**: 验证性能目标

### CI/CD

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - name: Run tests
        run: cargo test --workspace
      - name: Run benchmarks
        run: cargo bench --workspace
```

---

**记住**: 大多数问题都有解决方案！耐心诊断，系统排查。

祝调试顺利！🔧
