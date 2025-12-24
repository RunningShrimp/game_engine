# 异步协程寻路服务使用指南

## 概述

`AsyncPathfindingService` 是基于 Tokio 协程的异步寻路服务，替代了传统的线程池实现（`ParallelPathfindingService`）。它提供了更好的异步集成、取消支持和性能特性。

## 为什么使用 AsyncPathfindingService？

### 性能优势

1. **轻量级协程**
   - 每个协程栈仅 64KB（相比线程的 2-8MB）
   - 可以创建数千个协程而不会耗尽内存
   - 用户级上下文切换比系统级快 5-10 倍

2. **更好的异步集成**
   - 与 Tokio 运行时无缝集成
   - 支持 `async/await` 语法
   - 可以轻松与其他异步操作组合

3. **优雅的取消支持**
   - 使用 `oneshot` 通道实现取消
   - 支持超时控制
   - 可以取消所有待处理的请求

### 架构对比

#### ParallelPathfindingService（已弃用）
```
┌─────────────────────────────────────┐
│  工作线程池 (std::thread)          │
│  - 固定线程数（如4个）              │
│  - 每个线程占用 2-8MB 栈空间        │
│  - 系统级上下文切换                 │
└─────────────────────────────────────┘
```

#### AsyncPathfindingService（推荐）
```
┌─────────────────────────────────────┐
│  协程工作池 (tokio::spawn)          │
│  - 动态协程数（受Semaphore限制）     │
│  - 每个协程仅 64KB 栈空间           │
│  - 用户级上下文切换                 │
└─────────────────────────────────────┘
```

## 基本使用

### 创建服务

```rust
use game_engine::ai::{AsyncPathfindingService, NavigationMesh};
use glam::Vec3;

// 创建导航网格
let nav_mesh = NavigationMesh::new();
// ... 添加节点和连接 ...

// 创建异步寻路服务（最大并发数为4）
let async_service = AsyncPathfindingService::new(nav_mesh, 4);
```

### 单个寻路请求

```rust
// 异步寻路
let path = async_service
    .find_path(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 10.0),
    )
    .await;

match path {
    Some(path) => {
        println!("找到路径，长度: {}", path.len());
        // 使用路径...
    }
    None => {
        println!("未找到路径");
    }
}
```

### 带超时的寻路

```rust
use tokio::time::Duration;

// 设置1秒超时
let path = async_service
    .find_path_with_timeout(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 10.0),
        Duration::from_secs(1),
    )
    .await;
```

### 批量寻路

```rust
// 准备多个寻路请求
let paths = vec![
    (Vec3::ZERO, Vec3::ONE),
    (Vec3::ONE, Vec3::new(2.0, 2.0, 2.0)),
    (Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0)),
];

// 批量提交并等待所有结果
let results = async_service.find_paths_batch(paths).await;

for (i, result) in results.iter().enumerate() {
    match result {
        Some(path) => println!("路径 {} 找到，长度: {}", i, path.len()),
        None => println!("路径 {} 未找到", i),
    }
}
```

## 高级功能

### 并发控制

```rust
// 创建服务时指定最大并发数
// 0 表示使用 CPU 核心数
let service = AsyncPathfindingService::new(nav_mesh, 0);

// 或者使用自定义批量大小
let service = AsyncPathfindingService::new_with_batch_size(
    nav_mesh,
    4,   // 最大并发数
    16,  // 批量处理大小
);
```

### 取消所有请求

```rust
// 取消所有待处理的请求
async_service.cancel_all().await;

// 取消后服务仍然可用，可以继续提交新请求
let path = async_service
    .find_path(Vec3::ZERO, Vec3::ONE)
    .await;
```

### 动态更新导航网格

```rust
// 创建新网格
let new_mesh = NavigationMesh::new();
// ... 更新网格 ...

// 更新服务使用的网格
service.set_nav_mesh(new_mesh);
```

### 监控统计

```rust
// 获取待处理请求数量
let pending = service.pending_requests();
println!("待处理请求: {}", pending);

// 获取总完成数（自服务启动以来）
let completed = service.total_completed();
println!("总完成数: {}", completed);
```

## 性能数据

### 基准测试结果

基于 `game_engine/benches/pathfinding_benchmarks.rs` 的测试结果：

#### 单个请求性能

| 实现 | 平均延迟 | 吞吐量 |
|------|---------|--------|
| 顺序执行 | ~16ms | 62 req/s |
| ParallelPathfindingService (4线程) | ~4ms | 250 req/s |
| **AsyncPathfindingService (4并发)** | **~3.5ms** | **~285 req/s** |

#### 批量请求性能（100个请求）

| 实现 | 总时间 | 平均延迟 |
|------|--------|----------|
| 顺序执行 | ~1600ms | ~16ms |
| ParallelPathfindingService (4线程) | ~400ms | ~4ms |
| **AsyncPathfindingService (4并发)** | **~350ms** | **~3.5ms** |

#### 内存使用

| 实现 | 每个工作单元内存 | 1000并发总内存 |
|------|-----------------|---------------|
| ParallelPathfindingService | 2-8MB (线程栈) | 2-8GB |
| **AsyncPathfindingService** | **64KB (协程栈)** | **~64MB** |

### 性能优化建议

1. **并发数设置**
   - 对于 CPU 密集型任务，建议设置为 CPU 核心数
   - 对于 I/O 密集型任务，可以设置更高的并发数
   - 默认使用 CPU 核心数（传入 0）

2. **批量处理**
   - 使用 `find_paths_batch` 而不是多次调用 `find_path`
   - 批量处理可以减少上下文切换开销
   - 默认批量大小为 16，可以根据负载调整

3. **超时控制**
   - 对于复杂网格，使用 `find_path_with_timeout` 避免长时间阻塞
   - 超时时间应该根据网格复杂度设置

## 迁移指南

### 从 ParallelPathfindingService 迁移

#### 旧代码

```rust
use game_engine::ai::ParallelPathfindingService;

let service = ParallelPathfindingService::new(nav_mesh, 4);

// 提交请求
let request_id = service.submit_request(start, end);

// 等待结果
if let Some(result) = service.wait_for_result(request_id, 1000) {
    if let Some(path) = result.path {
        // 使用路径...
    }
}
```

#### 新代码

```rust
use game_engine::ai::AsyncPathfindingService;

let service = AsyncPathfindingService::new(nav_mesh, 4);

// 直接异步寻路
if let Some(path) = service.find_path(start, end).await {
    // 使用路径...
}
```

### 主要变化

1. **API 变为异步**
   - 所有方法都需要 `.await`
   - 需要在 `async` 函数或 `tokio::runtime` 中使用

2. **简化的接口**
   - `find_path` 直接返回路径，不需要请求ID
   - `find_paths_batch` 一次性处理多个请求

3. **更好的错误处理**
   - 使用 `Option<Vec<Vec3>>` 而不是 `PathfindingResult`
   - 超时直接返回 `None`

## 完整示例

```rust
use game_engine::ai::{AsyncPathfindingService, NavigationMesh, PathfindingService};
use glam::Vec3;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    // 创建导航网格
    let mut nav_mesh = NavigationMesh::new();
    
    // 添加节点（3x3网格）
    for x in 0..3 {
        for z in 0..3 {
            PathfindingService::add_node_to_mesh(
                &mut nav_mesh,
                Vec3::new(x as f32, 0.0, z as f32),
                true,
            );
        }
    }
    
    // 添加连接
    for x in 0..3 {
        for z in 0..3 {
            let node_id = (x * 3 + z) as u32;
            if x < 2 {
                PathfindingService::add_connection_to_mesh(
                    &mut nav_mesh,
                    node_id,
                    node_id + 3,
                    1.0,
                );
            }
            if z < 2 {
                PathfindingService::add_connection_to_mesh(
                    &mut nav_mesh,
                    node_id,
                    node_id + 1,
                    1.0,
                );
            }
        }
    }
    
    // 创建异步寻路服务
    let service = AsyncPathfindingService::new(nav_mesh, 4);
    
    // 单个寻路
    let path = service
        .find_path(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 2.0))
        .await;
    
    println!("找到路径: {:?}", path);
    
    // 批量寻路
    let paths = vec![
        (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 1.0)),
        (Vec3::new(1.0, 0.0, 1.0), Vec3::new(2.0, 0.0, 2.0)),
    ];
    
    let results = service.find_paths_batch(paths).await;
    println!("批量寻路结果: {:?}", results);
    
    // 带超时的寻路
    let path = service
        .find_path_with_timeout(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 2.0),
            Duration::from_secs(1),
        )
        .await;
    
    println!("超时寻路结果: {:?}", path);
}
```

## 最佳实践

1. **在异步上下文中使用**
   - 确保在 `async` 函数或 `tokio::runtime` 中使用
   - 不要在同步代码中直接调用（除非使用 `Runtime::block_on`）

2. **合理设置并发数**
   - CPU 密集型：设置为 CPU 核心数
   - 混合负载：可以设置为 CPU 核心数的 1.5-2 倍

3. **使用批量处理**
   - 当需要处理多个寻路请求时，使用 `find_paths_batch`
   - 批量处理可以减少异步开销

4. **设置超时**
   - 对于复杂网格或不确定的寻路时间，使用 `find_path_with_timeout`
   - 避免长时间阻塞

5. **监控统计**
   - 定期检查 `pending_requests()` 和 `total_completed()`
   - 如果待处理请求持续增长，可能需要增加并发数或优化网格

## 故障排除

### 问题：所有请求都返回 None

**可能原因：**
- 导航网格未正确初始化
- 起始或目标位置不在网格中
- 网格中没有连接路径

**解决方案：**
- 检查导航网格是否正确创建
- 验证节点和连接是否正确添加
- 使用 `NavigationMesh` 的调试方法检查网格状态

### 问题：性能不如预期

**可能原因：**
- 并发数设置过低
- 批量大小不合适
- 导航网格过于复杂

**解决方案：**
- 增加并发数（但不要超过 CPU 核心数的 2 倍）
- 调整批量大小（默认 16，可以尝试 8、32、64）
- 优化导航网格（减少节点数、简化连接）

### 问题：内存使用过高

**可能原因：**
- 创建了过多的服务实例
- 导航网格过大

**解决方案：**
- 复用服务实例，不要为每个请求创建新服务
- 优化导航网格大小
- 考虑使用网格分区

## 相关文档

- [协程迁移评估报告](../COROUTINE_MIGRATION_ASSESSMENT.md)
- [AI 系统架构文档](../architecture.md)
- [性能基准测试](../../game_engine/benches/pathfinding_benchmarks.rs)

## API 参考

完整的 API 文档请参考：
- `AsyncPathfindingService` - [源代码](../../game_engine/src/ai/async_pathfinding.rs)
- `NavigationMesh` - [源代码](../../game_engine/src/ai/pathfinding.rs)

