# 协程使用模式优化指南

本指南介绍如何正确使用异步协程，确保物理同步和系统性能。

## 核心原则

### 1. 物理同步原则

**物理更新必须在主线程同步执行，确保确定性。**

```rust
use game_engine::core::engine::{PhysicsSyncGuard, PhysicsSyncChecker};

// 在物理更新前获取同步锁
let _guard = PhysicsSyncGuard::acquire().await;

// 执行物理更新（同步）
physics_world.step(delta_time)?;

// guard 自动释放，允许异步任务继续
```

### 2. 任务优先级

使用任务优先级确保关键任务优先执行：

- **Critical**: 必须立即执行的任务（如关键资源加载）
- **High**: 高优先级任务（如纹理加载）
- **Normal**: 普通任务（如AI路径查找）
- **Low**: 后台任务（如日志清理）

```rust
use game_engine::core::engine::{AsyncScheduler, TaskPriority};

let scheduler = AsyncScheduler::new(runtime_handle);

// 高优先级任务
scheduler.spawn_task(
    "load_critical_texture",
    TaskPriority::High,
    async move {
        load_texture("critical.png").await
    }
).await;

// 低优先级任务
scheduler.spawn_task(
    "cleanup_logs",
    TaskPriority::Low,
    async move {
        cleanup_old_logs().await
    }
).await;
```

### 3. 避免阻塞物理

**长时间运行的任务应异步执行，但不影响物理更新。**

```rust
// ❌ 错误：在物理更新中阻塞
pub fn physics_step_system(mut physics: ResMut<PhysicsWorld>) {
    // 这会阻塞物理更新！
    let texture = block_on(load_texture("texture.png"));
}

// ✅ 正确：异步加载，不阻塞物理
pub fn physics_step_system(mut physics: ResMut<PhysicsWorld>) {
    // 物理更新保持同步
    physics.step(delta_time)?;
}

// 在另一个系统中异步加载
pub fn resource_loading_system(scheduler: Res<AsyncScheduler>) {
    scheduler.spawn_task(
        "load_texture",
        TaskPriority::Normal,
        async move {
            load_texture("texture.png").await
        }
    );
}
```

## 使用模式

### 模式1: 物理同步保护

在物理更新期间，使用 `PhysicsSyncGuard` 防止异步任务干扰：

```rust
use game_engine::core::engine::PhysicsSyncGuard;

pub async fn game_loop_update(
    mut physics: ResMut<PhysicsWorld>,
    scheduler: Res<AsyncScheduler>,
) {
    // 获取物理同步锁
    let _guard = PhysicsSyncGuard::acquire().await;
    
    // 物理更新（同步，确定性）
    physics.step(0.016)?;
    
    // guard 自动释放
    // 现在异步任务可以继续执行
}
```

### 模式2: 物理同步检查

使用 `PhysicsSyncChecker` 监控物理更新的一致性：

```rust
use game_engine::core::engine::PhysicsSyncChecker;
use std::time::Duration;

let checker = PhysicsSyncChecker::new(
    Duration::from_millis(16), // 期望间隔（60 FPS）
    Duration::from_millis(2),   // 最大允许偏差
);

// 在每次物理更新后记录
checker.record_physics_update().await?;
```

### 模式3: 任务超时控制

使用 `with_timeout` 防止任务卡死：

```rust
use game_engine::core::engine::with_timeout;
use std::time::Duration;

let result = with_timeout(
    async {
        // 可能长时间运行的任务
        process_large_file().await
    },
    Duration::from_secs(5), // 5秒超时
    "process_file",
).await?;
```

### 模式4: 等待高优先级任务

在关键操作前，等待高优先级任务完成：

```rust
// 在渲染前等待关键资源加载完成
scheduler.wait_for_high_priority_tasks().await;

// 现在可以安全地渲染
render_scene();
```

## 最佳实践

### ✅ 推荐做法

1. **物理更新使用同步方法**
   ```rust
   // ✅ 正确
   physics_world.step(delta_time)?;
   ```

2. **资源加载使用异步**
   ```rust
   // ✅ 正确
   scheduler.spawn_task("load", TaskPriority::High, async {
       load_resource().await
   }).await;
   ```

3. **使用物理同步保护**
   ```rust
   // ✅ 正确
   let _guard = PhysicsSyncGuard::acquire().await;
   physics.step(dt)?;
   ```

4. **监控物理同步**
   ```rust
   // ✅ 正确
   let checker = PhysicsSyncChecker::new(expected_interval, max_deviation);
   checker.record_physics_update().await?;
   ```

### ❌ 避免的做法

1. **不要在物理更新中阻塞**
   ```rust
   // ❌ 错误
   pub fn physics_step() {
       block_on(async_task()); // 阻塞物理更新
   }
   ```

2. **不要并发访问物理系统**
   ```rust
   // ❌ 错误
   tokio::spawn(async {
       physics_world.step(dt)?; // 并发访问物理系统
   });
   ```

3. **不要忽略任务优先级**
   ```rust
   // ❌ 错误：关键任务使用低优先级
   scheduler.spawn_task("critical", TaskPriority::Low, task).await;
   ```

4. **不要忘记超时控制**
   ```rust
   // ❌ 错误：没有超时，可能卡死
   let result = long_running_task().await;
   
   // ✅ 正确：添加超时
   let result = with_timeout(long_running_task(), timeout, "task").await;
   ```

## 性能优化

### 1. 任务并发控制

调度器使用信号量控制不同优先级任务的并发数：

- **高优先级**: 最多8个并发任务
- **普通优先级**: 最多16个并发任务
- **低优先级**: 最多32个并发任务

### 2. 任务清理

定期清理已完成的任务，释放内存：

```rust
// 在游戏循环中定期调用
scheduler.cleanup_completed_tasks().await;
```

### 3. 统计监控

监控任务执行情况，识别性能瓶颈：

```rust
let stats = scheduler.stats().await;
tracing::info!(
    "Tasks: {} total, {} completed, {} failed, {} active",
    stats.total_tasks,
    stats.completed_tasks,
    stats.failed_tasks,
    stats.active_task_count
);
```

## 故障排除

### 问题1: 物理更新不稳定

**症状**: 物理模拟结果不一致

**原因**: 异步任务干扰了物理更新

**解决方案**:
```rust
// 使用物理同步保护
let _guard = PhysicsSyncGuard::acquire().await;
physics.step(dt)?;
```

### 问题2: 任务执行缓慢

**症状**: 资源加载等任务执行很慢

**原因**: 任务优先级设置不当或并发数不足

**解决方案**:
```rust
// 提高任务优先级
scheduler.spawn_task("load", TaskPriority::High, task).await;

// 或等待高优先级任务完成
scheduler.wait_for_high_priority_tasks().await;
```

### 问题3: 物理更新间隔偏差

**症状**: 物理更新间隔不稳定

**原因**: 异步任务阻塞了主循环

**解决方案**:
```rust
// 使用物理同步检查器监控
let checker = PhysicsSyncChecker::new(expected_interval, max_deviation);
checker.record_physics_update().await?;

// 检查警告计数
if checker.warning_count() > 0 {
    tracing::warn!("Physics sync issues detected");
}
```

## 总结

1. **物理更新必须同步**: 使用 `PhysicsSyncGuard` 保护物理更新
2. **合理使用任务优先级**: 关键任务使用高优先级
3. **避免阻塞**: 长时间任务应异步执行
4. **监控和检查**: 使用 `PhysicsSyncChecker` 监控物理同步
5. **超时控制**: 使用 `with_timeout` 防止任务卡死

遵循这些原则和模式，可以确保物理系统的确定性和稳定性，同时充分利用异步协程的性能优势。

