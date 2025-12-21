# 性能优化最佳实践指南

## 概述

本文档介绍游戏引擎中的性能优化最佳实践，涵盖ECS系统、渲染管线、资源管理、物理引擎等关键领域。

## ECS系统优化

### 1. 组件脏跟踪

使用ECS组件脏跟踪系统，只更新修改过的组件：

```rust
use game_engine::ecs::dirty_tracking::{ComponentDirty, DirtyFlags};

// 标记组件为脏
let mut dirty = world.get_component_mut::<ComponentDirty>(entity).unwrap();
dirty.flags.set_dirty(DirtyFlags::TRANSFORM);

// 系统只处理脏组件
fn update_transform_system(
    mut query: Query<&mut Transform, (With<ComponentDirty>, Changed<Transform>)>,
) {
    for mut transform in query.iter_mut() {
        // 只处理修改过的组件
    }
}
```

### 2. 查询优化

- 使用`Changed<T>`过滤器只处理修改过的组件
- 避免在热路径中使用`Option<T>`查询
- 使用并行查询（`par_for_each`）处理大量实体

```rust
use bevy_ecs::prelude::*;

// ✅ 优化：只查询修改过的组件
fn optimized_system(query: Query<&Transform, Changed<Transform>>) {
    for transform in query.iter() {
        // 只处理修改过的
    }
}

// ✅ 优化：并行处理
fn parallel_system(query: Query<&mut Transform>) {
    query.par_for_each_mut(32, |mut transform| {
        // 并行处理
    });
}
```

## 渲染管线优化

### 1. 批处理优化

使用`BatchOptimizer`进行智能批处理：

```rust
use game_engine::render::BatchOptimizer;

let mut optimizer = BatchOptimizer::new(BatchOptimizerConfig {
    max_instances_per_batch: 1000,
    ..Default::default()
});

// 添加绘制命令
optimizer.add_draw_command(draw_command);

// 优化批处理
let batches = optimizer.optimize();
```

### 2. 状态切换优化

- 按状态切换成本排序绘制命令
- 减少不必要的状态切换
- 使用实例化渲染减少Draw Call

```rust
// 批处理键按状态切换成本排序
// 优先级：Pipeline > Blend > Depth > RenderFlags > Mesh > Material
let batch_key = BatchKey {
    pipeline_id: 1,
    blend_mode: BlendMode::Alpha,
    depth_test: true,
    // ...
};
```

## 资源管理优化

### 1. 异步资源加载

使用协程优化的异步资源加载系统：

```rust
use game_engine::resources::coroutine_loader::CoroutineAssetLoader;

let loader = CoroutineAssetLoader::new(CoroutineLoaderConfig::default());

// 加载资源
let handle = loader.load_asset("texture.png".to_string());

// 等待加载完成
let result = loader.wait_for_completed(handle, Duration::from_secs(5)).await;
```

### 2. 对象池

使用对象池减少内存分配：

```rust
use game_engine::performance::memory::pool_manager::GLOBAL_POOL_MANAGER;

// 从对象池获取
let mut vec: Vec<u8> = GLOBAL_POOL_MANAGER.acquire();

// 使用后释放回对象池
GLOBAL_POOL_MANAGER.release(vec);
```

### 3. 资源缓存

- 实现资源缓存避免重复加载
- 使用LRU缓存策略
- 考虑资源生命周期管理

## 物理引擎优化

### 1. 空间分区

使用空间分区（如BVH、四叉树）优化碰撞检测：

```rust
// 未来实现：空间分区
struct SpatialPartition {
    // BVH树或四叉树
}

impl SpatialPartition {
    fn query_collisions(&self, bounds: AABB) -> Vec<ColliderId> {
        // 只查询相关区域
    }
}
```

### 2. 碰撞检测优化

- 使用粗检测（broad phase）和细检测（narrow phase）
- 避免不必要的碰撞检测
- 使用时间步长优化

## 内存优化

### 1. 减少分配

- 使用对象池
- 预分配缓冲区
- 避免在热路径中分配内存

### 2. 数据结构选择

- 使用`Vec`而不是`VecDeque`（如果不需要双端操作）
- 使用`HashMap`而不是`BTreeMap`（如果不需要有序）
- 考虑使用`SmallVec`存储小数组

### 3. 内存对齐

- 注意结构体字段顺序，减少内存占用
- 使用`#[repr(C)]`或`#[repr(align(N))]`控制对齐

## 并发优化

### 1. 并行系统

使用Bevy ECS的并行系统：

```rust
use bevy_ecs::prelude::*;

fn parallel_update_system(query: Query<&mut Transform>) {
    query.par_for_each_mut(32, |mut transform| {
        // 并行更新
    });
}
```

### 2. 锁优化

- 使用`RwLock`而不是`Mutex`（读多写少场景）
- 使用`safe_lock`、`safe_read`、`safe_write`处理锁中毒
- 最小化持锁时间

```rust
use game_engine::error::{safe_read, safe_write};

// ✅ 优化：使用读写锁
let data = safe_read(&shared_data, "shared_data")?;

// ✅ 优化：最小化持锁时间
{
    let mut data = safe_write(&shared_data, "shared_data")?;
    data.update();
} // 锁在这里释放
```

## 性能监控

### 1. 性能指标

使用性能监控系统跟踪关键指标：

```rust
use game_engine::performance::monitoring::PerformanceMonitor;

let monitor = PerformanceMonitor::new();
monitor.record_metric(MetricType::FrameTime, 16.67);
monitor.record_metric(MetricType::DrawCalls, 100);
```

### 2. 性能分析

- 使用`tracing`进行性能分析
- 使用基准测试检测性能回归
- 定期运行性能测试

```rust
use tracing::info_span;

let span = info_span!("update_system");
let _guard = span.enter();
// 系统代码
```

## 最佳实践总结

### 1. 测量优先

- 先测量性能，再优化
- 使用性能分析工具识别瓶颈
- 避免过早优化

### 2. 渐进优化

- 从最大的瓶颈开始优化
- 每次优化后测量效果
- 避免过度优化

### 3. 代码可读性

- 保持代码可读性
- 添加性能相关的注释
- 使用有意义的性能指标名称

### 4. 测试和验证

- 编写性能基准测试
- 使用性能回归检测
- 验证优化效果

## 相关文档

- [ECS脏跟踪文档](../ecs_dirty_tracking.md)
- [渲染批处理优化文档](../render_batch_optimization.md)
- [异步资源优化文档](../async_resource_optimization.md)
- [对象池扩展文档](../object_pool_extension.md)

