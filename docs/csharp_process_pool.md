# C# 进程池实现文档

## 概述

C# 进程池是一个性能优化特性，通过持久化 .NET 进程避免每次执行脚本时启动新进程的开销。

## 性能提升

### 无进程池（传统方式）
- **每次执行**: ~50ms (进程启动 ~45ms + 执行 ~5ms)
- **100次执行**: ~5000ms

### 有进程池（优化方式）
- **首次执行**: ~50ms (启动进程池)
- **后续执行**: <5ms (仅执行)
- **100次执行**: ~550ms (50ms + 100 × 5ms)

**性能提升**: **10x**

## 架构

### DotNetProcess
单个 .NET 进程包装器。

```rust
pub struct DotNetProcess {
    pub id: usize,
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<BufReader<ChildStdout>>,
    state: ProcessState,
    // ...
}
```

**功能:**
- 进程生命周期管理
- 标准输入/输出通信
- 健康检查
- 执行脚本代码

### DotNetProcessPool
进程池管理器。

```rust
pub struct DotNetProcessPool {
    processes: VecDeque<DotNetProcess>,
    config: ProcessPoolConfig,
    // ...
}
```

**功能:**
- 进程分配和复用
- 自动进程恢复
- 健康检查和清理
- 统计信息收集

## 配置

### ProcessPoolConfig

```rust
pub struct ProcessPoolConfig {
    /// 最大进程数（默认: 4）
    pub max_processes: usize,

    /// 最小空闲进程数（默认: 1）
    pub min_idle_processes: usize,

    /// 进程空闲超时（秒，默认: 60）
    pub idle_timeout_secs: u64,

    /// 健康检查间隔（秒，默认: 10）
    pub health_check_interval_secs: u64,

    /// 执行超时（秒，默认: 5）
    pub execution_timeout_secs: u64,
}
```

### 环境变量

通过环境变量控制进程池启用状态：

```bash
# 启用进程池（默认）
export CSHARP_ENABLE_PROCESS_POOL=true

# 禁用进程池
export CSHARP_ENABLE_PROCESS_POOL=false
```

## 使用方法

### 自动启用

进程池在 `DotNetCliHost` 初始化时自动启用（如果 .NET SDK 可用）：

```rust
let host = DotNetCliHost::initialize()?;
// 进程池自动启用
```

### 手动配置

```rust
use game_engine::scripting::csharp_dotnet::{DotNetCliHost, DotNetProcessPool, ProcessPoolConfig};

// 自定义配置
let config = ProcessPoolConfig {
    max_processes: 8,
    min_idle_processes: 2,
    idle_timeout_secs: 120,
    ..Default::default()
};

let work_dir = PathBuf::from("./temp/pool");
let pool = DotNetProcessPool::new(config, work_dir)?;
```

### 性能统计

```rust
let stats = host.get_process_pool_stats();

if let Some(stats) = stats {
    println!("Total executions: {}", stats["total_executions"]);
    println!("Pool hits: {}", stats["pool_hits"]);
    println!("Hit rate: {}%", stats["hit_rate_percent"]);
    println!("Active processes: {}", stats["active_processes"]);
}
```

## 统计信息

### PoolStats

```rust
pub struct PoolStats {
    /// 总执行次数
    pub total_executions: usize,

    /// 进程池命中次数
    pub pool_hits: usize,

    /// 进程创建次数
    pub process_creations: usize,

    /// 进程失败次数
    pub process_failures: usize,

    /// 进程重启次数
    pub process_restarts: usize,
}
```

### ProcessStats

```rust
pub struct ProcessStats {
    pub id: usize,
    pub state: ProcessState,
    pub execution_count: usize,
    pub uptime_secs: u64,
    pub idle_secs: u64,
}
```

## 维护任务

### 健康检查

定期执行健康检查，自动恢复失败的进程：

```rust
host.health_check_process_pool();
```

### 清理空闲进程

定期清理空闲超时的进程：

```rust
host.cleanup_idle_processes();
```

### 优雅关闭

进程池在 `Drop` 时自动关闭所有进程：

```rust
{
    let host = DotNetCliHost::initialize()?;
    // 使用 host...
} // 自动清理
```

## 性能调优

### 调整进程池大小

根据并发需求调整 `max_processes`:

```rust
// 低并发（<10 并发执行）
max_processes: 2

// 中并发（10-50 并发执行）
max_processes: 4  // 默认

// 高并发（>50 并发执行）
max_processes: 8
```

### 调整空闲超时

根据内存和性能需求调整 `idle_timeout_secs`:

```rust
// 内存受限
idle_timeout_secs: 30  // 更快清理

// 性能优先
idle_timeout_secs: 120  // 更长保持
```

## 故障恢复

进程池包含自动故障恢复机制：

1. **进程崩溃**: 自动检测并重启
2. **执行超时**: 终止并重启进程
3. **健康检查失败**: 标记为失败并重启

## 最佳实践

1. **启动时预热**: 预启动最小进程数
2. **定期健康检查**: 每10秒检查一次
3. **定期清理空闲进程**: 每分钟清理一次
4. **监控统计信息**: 追踪命中率和性能
5. **优雅关闭**: 让进程池自然清理

## 限制和注意事项

1. **进程数限制**: 受系统资源限制
2. **内存开销**: 每个进程约10-20MB
3. **进程启动**: 首次启动需要额外时间
4. **平台兼容性**: 需要 .NET SDK 8+

## 相关文档

- [C# 编译缓存](./csharp_compile_cache.md)
- [C# 热重载](./csharp_hot_reload.md)
- [C# 运行时评估](./csharp_runtime_evaluation.md)
