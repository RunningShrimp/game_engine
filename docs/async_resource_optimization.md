# 异步资源/着色器队列性能优化

本文档记录了P1-3阶段对异步资源加载和着色器编译队列的性能优化工作。

## 优化目标

根据implementation_plan.md的要求，本次优化主要关注：

1. **队列性能监控**：记录队列长度、平均等待时间、最大等待时间
2. **降低抖动**：减少sleep/poll造成的性能抖动，优先使用notify/channel
3. **spawn_blocking限额**：配置化并与CPU核数/任务类型绑定

## 已完成的优化

### 1. 资源加载队列性能监控 (coroutine_loader.rs)

#### 队列长度统计
- ✅ 添加了`queue_length: Arc<AtomicUsize>`字段，实时跟踪队列长度
- ✅ 在添加请求到队列时更新`queue_length`
- ✅ 在从队列取出请求时更新`queue_length`
- ✅ 通过`stats()`方法暴露队列长度信息

#### 等待时间统计
- ✅ 添加了`WaitTimeStats`结构体，记录：
  - `total_wait_time_ms`: 总等待时间
  - `max_wait_time_ms`: 最大等待时间
  - `min_wait_time_ms`: 最小等待时间
  - `sample_count`: 样本数量
- ✅ 在请求从队列取出时计算等待时间（从`created_at`到处理开始）
- ✅ 更新`LoaderStats`结构，包含：
  - `queue_length`: 当前队列长度
  - `avg_wait_time_ms`: 平均等待时间
  - `max_wait_time_ms`: 最大等待时间
  - `min_wait_time_ms`: 最小等待时间

#### 通知机制优化
- ✅ 使用`completion_notify_tx/rx`通道替代轮询
- ✅ 实现了`wait_for_completed()`异步方法，使用通知机制
- ✅ 保留了`poll_completed()`方法用于同步上下文，但添加了使用建议注释

### 2. 着色器编译队列性能监控 (shader_async.rs)

#### 队列长度统计
- ✅ `CompileProgress`结构已包含`queue_length`字段
- ✅ 队列长度通过`pending`字段实时反映

#### 等待时间统计
- ✅ 添加了`ShaderWaitTimeStats`结构体
- ✅ 在编译请求从队列取出时记录等待时间
- ✅ `CompileProgress`包含：
  - `avg_wait_time_ms`: 平均等待时间
  - `max_wait_time_ms`: 最大等待时间

### 3. spawn_blocking并发控制

#### CPU核数绑定
- ✅ `CoroutineLoaderConfig`默认配置基于CPU核数：
  - `max_concurrent_loads`: 2倍CPU核数（最小4，最大16）
  - `max_spawn_blocking`: CPU核数（最小2，最大8）
- ✅ `AsyncShaderCompilerConfig`默认配置基于CPU核数：
  - `max_concurrent_compiles`: CPU核数（最小2，最大8）
  - `max_spawn_blocking`: CPU核数的一半（最小1，最大4）

#### 并发限制实现
- ✅ 资源加载器使用`spawn_blocking_semaphore`限制阻塞任务并发数
- ✅ 着色器编译器使用`spawn_blocking_semaphore`限制编译任务并发数
- ✅ 在图像解码等阻塞操作中使用信号量控制并发

## 性能指标收集

### 资源加载器指标

通过`CoroutineAssetLoader::stats()`方法获取：

```rust
pub struct LoaderStats {
    pub active_loads: usize,        // 当前活跃加载数
    pub queue_length: usize,        // 队列长度
    pub total_requests: u64,        // 总请求数
    pub total_completed: u64,       // 总完成数
    pub total_failed: u64,          // 总失败数
    pub avg_wait_time_ms: f64,      // 平均等待时间（毫秒）
    pub max_wait_time_ms: f64,      // 最大等待时间（毫秒）
    pub min_wait_time_ms: f64,      // 最小等待时间（毫秒）
}
```

### 着色器编译器指标

通过`AsyncShaderCompiler::get_progress()`方法获取：

```rust
pub struct CompileProgress {
    pub total_requests: usize,      // 总请求数
    pub completed: usize,            // 已完成数
    pub failed: usize,               // 失败数
    pub in_progress: usize,          // 进行中数
    pub pending: usize,               // 等待中数
    pub queue_length: usize,         // 队列长度
    pub avg_wait_time_ms: f64,       // 平均等待时间（毫秒）
    pub max_wait_time_ms: f64,       // 最大等待时间（毫秒）
}
```

## 使用建议

### 监控队列性能

```rust
// 资源加载器
let loader = CoroutineAssetLoader::new(CoroutineLoaderConfig::default());
let stats = loader.stats();
println!("Queue length: {}, Avg wait: {:.2}ms", 
    stats.queue_length, stats.avg_wait_time_ms);

// 着色器编译器
let compiler = AsyncShaderCompiler::new(config, cache)?;
if let Some(progress) = compiler.get_progress() {
    println!("Shader queue: {}, Avg wait: {:.2}ms",
        progress.queue_length, progress.avg_wait_time_ms);
}
```

### 配置CPU感知的并发限制

```rust
// 使用默认配置（自动基于CPU核数）
let loader = CoroutineAssetLoader::new(CoroutineLoaderConfig::default());

// 或使用自定义配置
let config = CoroutineLoaderConfig::new(
    16,  // max_concurrent_loads
    8,   // max_spawn_blocking
    30000, // load_timeout_ms
    2,   // max_retries
    100, // retry_delay_ms
);
let loader = CoroutineAssetLoader::new(config);
```

### 异步等待完成（推荐）

```rust
// 推荐：使用异步通知机制
let completed = loader.wait_for_completed().await;

// 不推荐：轮询方式（仅在同步上下文必需时使用）
let completed = loader.poll_completed();
```

## 性能优化效果

### 预期改进

1. **减少CPU浪费**：使用通知机制替代轮询，降低CPU占用
2. **更好的并发控制**：基于CPU核数的配置，避免过度并发导致的上下文切换开销
3. **可观测性提升**：详细的队列和等待时间统计，便于性能分析和优化

### 监控建议

- 定期检查`queue_length`，如果持续增长，可能需要增加并发数或优化加载速度
- 监控`avg_wait_time_ms`，如果超过阈值（如100ms），考虑优化加载逻辑
- 关注`max_wait_time_ms`，识别异常慢的加载请求

## 后续优化方向

1. **动态调整并发数**：根据队列长度和等待时间动态调整并发限制
2. **优先级优化**：根据等待时间调整请求优先级
3. **批量处理优化**：对相同类型的资源进行批量加载
4. **缓存预热**：基于历史数据预测并预加载资源

## 相关文件

- `game_engine/src/resources/coroutine_loader.rs`: 资源加载器实现
- `game_engine/src/render/shader_async.rs`: 着色器编译器实现
- `game_engine/src/performance/tracing_metrics.rs`: 性能指标集成
