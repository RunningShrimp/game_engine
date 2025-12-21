# 性能优化技巧指南

## 概述

本文档提供游戏引擎性能优化的实用技巧和最佳实践。

## 渲染优化

### 批处理渲染

```rust
// 使用批处理渲染器合并draw call
use game_engine_performance::rendering::BatchRenderer;

let mut batch_renderer = BatchRenderer::new(100);
batch_renderer.add_draw_call(batch_key, vertex_offset, index_offset, index_count);
let batches = batch_renderer.get_batches();
```

### GPU实例化

```rust
// 使用实例化渲染减少draw call
let batch = InstanceBatch::new(key, mesh, material);
batch.add_instance(instance1);
batch.add_instance(instance2);
batch.update_buffer(device, queue); // 增量更新
```

### 剔除优化

- **视锥剔除**：只渲染可见对象
- **遮挡剔除**：跳过被遮挡的对象
- **距离剔除**：根据距离剔除远距离对象

## ECS优化

### 系统调度优化

```rust
use game_engine::core::system_scheduler::SystemSchedulerOptimizer;

let mut optimizer = SystemSchedulerOptimizer::new();
optimizer.add_system_dependency(dependency);
optimizer.analyze_dependencies();
let order = optimizer.execution_order(); // 获取并行执行顺序
```

### 组件脏跟踪

```rust
use game_engine::ecs::dirty_tracking::{DirtyFlags, ComponentDirty};

// 标记组件为脏
dirty_tracker.mark_dirty(entity, DirtyFlags::TRANSFORM);

// 只处理脏组件
for (entity, transform, dirty) in query.iter() {
    if dirty.is_dirty(DirtyFlags::TRANSFORM) {
        // 更新变换
    }
}
```

## 内存优化

### 对象池

```rust
use game_engine_performance::memory::PoolManager;

let pool_manager = global_pool_manager();
let mut vec = pool_manager.vec_u8_pool().acquire();
// 使用vec
// 自动归还到池中
```

### 内存分配统计

```rust
use game_engine::performance::monitoring::SystemPerformanceMonitor;

let monitor = SystemPerformanceMonitor::new();
monitor.update_memory_stats(allocations, deallocations);
let metrics = monitor.get_metrics();
println!("内存分配: {}", metrics.memory_allocations);
```

## 物理优化

### 空间分区

```rust
use game_engine::physics::spatial_partition::{SpatialPartitionManager, SpatialPartitionType};

let mut partition = SpatialPartitionManager::new(SpatialPartitionType::Octree);
partition.build(&collider_set);

// 查询碰撞体（使用空间分区加速）
let results = partition.query_aabb(&query_aabb, &collider_set);
```

### 动态调整

```rust
// 根据场景大小动态调整分区
partition.adjust_for_scene(&scene_aabb, object_count);
```

## 异步操作

### 异步资源加载

```rust
use game_engine::resources::coroutine_loader::CoroutineAssetLoader;

let loader = CoroutineAssetLoader::new();
let handle = loader.load_asset("texture.png", AssetType::Image).await?;
```

### 异步任务调度

```rust
use game_engine::core::scheduler::TaskScheduler;

let scheduler = TaskScheduler::new(4);
scheduler.spawn_background(async {
    // 后台任务
}).await;
```

## 性能监控

### 性能指标收集

```rust
use game_engine::performance::monitoring::SystemPerformanceMonitor;

let mut monitor = SystemPerformanceMonitor::new();
monitor.update_frame();
monitor.update_gpu_render_times(render_time, geometry_time, lighting_time, postprocess_time);
monitor.update_ecs_system_time(ecs_time);
let metrics = monitor.get_metrics();
```

### 性能报告

```rust
let report = monitor.get_report();
println!("FPS: {:.1}", report.current_fps);
println!("帧时间: {:.2}ms", report.average_frame_time_ms);
```

## 最佳实践

1. **测量优先**：先测量性能，再优化
2. **批量操作**：尽可能批量处理数据
3. **避免分配**：在热路径中避免内存分配
4. **并行处理**：利用多核CPU并行处理
5. **缓存友好**：保持数据局部性，提高缓存命中率

## 相关文档

- [性能优化指南](performance_optimization_guide.md)
- [ECS脏跟踪文档](../ecs_dirty_tracking.md)

