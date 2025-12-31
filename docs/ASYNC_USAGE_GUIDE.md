# Async/Tokio 使用指南

## 概述

本文档提供游戏引擎项目中async/await使用的最佳实践，帮助开发者正确选择异步 vs 同步 API。

---

## 核心原则

### 何时使用 Async

**✅ 适合使用 async 的场景**:

1. **I/O 密集型操作**
```rust
pub async fn read_file(path: &Path) -> Result<Vec<u8>> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).await?;
    Ok(buf)
}
```

2. **网络操作**
```rust
pub async fn send_message(&self, msg: Message) -> Result<()> {
    self.socket.write_all(msg.bytes()).await?;
    Ok(())
}
```

3. **需要等待其他异步操作**
```rust
pub async fn process_request(&self, req: Request) -> Result<Response> {
    let config = self.config.read().await;
    self.remote_service.call(&config, req).await?;
    Ok(Response::ok())
}
```

### 何时避免 Async

**❌ 不适合使用 async 的场景**:

1. **纯查询操作** (热路径)
```rust
// ❌ 反模式
pub async fn subscriber_count(&self) -> usize {
    self.subscribers.read().await.len()  // 80-350µs 开销
}

// ✅ 优化后
pub fn subscriber_count(&self) -> usize {
    self.subscribers.blocking_read().len()  // <1µs
}
```

2. **简单计算**
```rust
// ❌ 不必要
pub async fn calculate(&self, x: i32) -> i32 {
    x * 2  // 异步开销 > 计算时间
}

// ✅ 同步
pub fn calculate(&self, x: i32) -> i32 {
    x * 2
}
```

---

## Tokio vs Rayon 选择

### Tokio (异步)

**适用场景**:
- I/O 密集型任务
- 需要同时处理数千个并发操作
- 涉及网络、文件系统等异步 API

```rust
// ✅ 网络服务器
async fn handle_client(socket: TcpStream) {
    let mut reader = BufReader::new(&socket);
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        process_line(&line);
    }
}
```

### Rayon (并行)

**适用场景**:
- CPU 密集型任务
- 批量数据处理
- 需要充分利用多核 CPU

```rust
// ✅ 批量AI寻路
use rayon::prelude::*;

pub fn find_paths_batch_parallel(&self, paths: Vec<(Vec3, Vec3)>) -> Vec<Option<Vec<Vec3>>> {
    paths.par_iter()
        .map(|(start, end)| self.find_path(start, end))
        .collect()
}
```

### 决策树

```
任务类型？
├─ I/O 密集型 → Tokio async/await
├─ CPU 密集型 → Rayon par_iter
└─ 混合型 → 
    ├─ CPU 部分: spawn_blocking + Rayon
    └─ I/O 部分: async/await
```

---

## blocking_read() 使用指南

### 何时使用 blocking_read()

**适用条件**:
1. ✅ 纯查询操作（无 I/O）
2. ✅ 热路径上的频繁调用
3. ✅ 操作时间 < 异步开销（<80µs）

**示例**:

```rust
// ✅ 适合: 简单查询
pub fn client_count(&self) -> usize {
    self.clients.blocking_read().len()
}

// ✅ 适合: 纯内存操作
pub fn get_config_value(&self, key: &str) -> Option<String> {
    self.config.blocking_read().get(key).cloned()
}

// ❌ 不适合: 涉及 I/O
pub fn load_texture(&self, path: &Path) -> Result<Texture> {
    // 不要用 blocking_read，这会阻塞异步运行时
    tokio::task::spawn_blocking(move || {
        // CPU 密集型加载操作
        load_from_disk(path)
    }).await?
}
```

### 性能对比

| 操作 | Async | blocking_read | 加速比 |
|------|-------|---------------|--------|
| HashMap len() 查询 | 350µs | <1µs | ~350x |
| 简单 get() | 250µs | <1µs | ~250x |
| 复杂遍历 | 500µs | 200µs | 2.5x |

---

## spawn_blocking 使用

### 何时使用 spawn_blocking

```rust
// ✅ CPU 密集型任务在异步上下文中
pub async fn process_audio(&self, samples: Vec<f32>) -> Result<Vec<f32>> {
    tokio::task::spawn_blocking(move || {
        // CPU 密集型音频处理
        apply_effects(samples)
    }).await?
}
```

**注意**:
- 用于将 CPU 密集型任务移到专用线程池
- 避免阻塞异步运行时
- 适合短暂的计算任务（<1秒）

---

## 错误处理

### Async 错误传播

```rust
pub async fn process_request(&self, req: Request) -> Result<Response, Error> {
    // 使用 ? 操作符传播错误
    let config = self.load_config().await?;
    let result = self.compute(&config, req).await?;
    Ok(result)
}
```

### Blocking 错误处理

```rust
pub fn get_value(&self, key: &str) -> Result<String, Error> {
    let guard = self.map.blocking_read().map_err(|_| Error::LockPoisoned)?;
    guard.get(key)
        .cloned()
        .ok_or(Error::NotFound(key.to_string()))
}
```

---

## 性能优化清单

- [ ] **识别热路径**: 频繁调用的纯查询操作
  - [ ] 考虑 `blocking_read()` 替代 `.await`
  
- [ ] **避免过度异步**: 简单计算不需要异步
  - [ ] 测量操作时间
  - [ ] 如果 <80µs，考虑同步
  
- [ ] **使用 Rayon**: CPU 密集型批量操作
  - [ ] 设置合理阈值（>10个任务）
  
- [ ] **基准测试**: 优化前后对比
  - [ ] 使用 criterion 测试
  - [ ] 验证性能提升

---

## 相关资源

- **条件编译**: `CONDITIONAL_COMPILATION_BEST_PRACTICES.md`
- **性能最佳实践**: `PERFORMANCE_BEST_PRACTICES.md`
- **性能分析报告**: `PERFORMANCE_ANALYSIS_REPORT.md`

---
**版本**: v0.1.0
**更新**: 2025-12-31
