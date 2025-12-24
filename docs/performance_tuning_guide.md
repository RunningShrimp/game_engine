# 性能调优指南

## 概述

本指南提供游戏引擎的性能调优策略和最佳实践，涵盖渲染、物理、内存管理等关键领域。

## 目录

1. [渲染优化](#渲染优化)
2. [物理优化](#物理优化)
3. [内存优化](#内存优化)
4. [批处理优化](#批处理优化)
5. [异步I/O优化](#异步io优化)

---

## 渲染优化

### GPU驱动渲染

**启用GPU驱动剔除**（推荐）

```rust
use game_engine::render::gpu_driven::{GpuDrivenRenderer, GpuDrivenConfig};

let config = GpuDrivenConfig {
    frustum_culling: true,    // 启用视锥剔除
    occlusion_culling: false,   // 按需启用遮挡剔除
    lod_enabled: true,         // 启用LOD
    max_instances: 65536,     // 根据场景调整
    workgroup_size: 64,
};
```

**预期性能提升**：30-50%（取决于场景复杂度）

### LOD系统

**配置合适的LOD级别**

```rust
use game_engine::render::lod::{LodConfig, LodLevel, LodQuality};

let config = LodConfig {
    lod_levels: vec![
        LodLevel::new(0, 0.0, 50.0, "high"),
        LodLevel::new(1, 50.0, 100.0, "medium"),
        LodLevel::new(2, 100.0, 200.0, "low"),
    ],
    quality: LodQuality::Balanced,
    transition_distance: 10.0,
};
```

**建议**：
- 远距离物体使用简化的网格
- 远距离光照使用烘焙
- 减少远距离阴影精度

### 视锥剔除

**自动启用**，无需手动配置。确保场景使用正确的包围盒（AABB）。

### 遮挡剔除

**适用于密集场景**（如室内、城市）

```rust
let config = GpuDrivenConfig {
    occlusion_culling: true,  // 启用遮挡剔除
    // ...
};
```

**注意事项**：
- 遮挡剔除增加GPU计算开销
- 仅在有明显遮挡时启用
- 确保深度预通过正确

### 批处理优化

**启用实例批处理**

```rust
use game_engine::render::instance_batch::{BatchManager, DynamicBatchConfig};

let config = DynamicBatchConfig {
    max_instances_per_batch: 1024,
    max_batches: 64,
    enable_frustum_culling: true,
};

let mut batch_manager = BatchManager::new(config);
```

**优化建议**：
- 合并相同材质的绘制调用
- 使用实例化渲染大量相似物体
- 按材质排序绘制调用

### 后处理优化

**仅启用需要的后处理效果**

```rust
// 根据性能预算选择效果
let enable_ssao = target_fps >= 60;  // 仅在高性能目标启用
let enable_bloom = target_fps >= 30;
let enable_taa = enable_ssao;  // TAA与SSAO通常一起使用
```

---

## 物理优化

### 空间分区

**使用BVH或空间哈希加速碰撞检测**

```rust
use game_engine::physics::{SpatialPartitionType, SpatialPartitionManager};

// BVH树 - 适合静态物体
let bvh_manager = SpatialPartitionManager::new(SpatialPartitionType::BVH {
    max_depth: 16,
    max_objects_per_leaf: 4,
});

// 空间哈希 - 适合动态物体
let hash_manager = SpatialPartitionManager::new(SpatialPartitionType::SpatialHash {
    cell_size: 10.0,  // 根据物体大小调整
});
```

### 脏追踪同步

**使用脏追踪避免不必要的同步**

```rust
use game_engine::physics::PhysicsSyncConfig;

let config = PhysicsSyncConfig {
    sync_threshold: 0.001,    // 仅在移动超过阈值时同步
    batch_size: 100,           // 批量同步大小
    enable_dirty_tracking: true,
};
```

### 并行物理计算

**启用并行物理步进**

```rust
use game_engine::physics::parallel::parallel_physics_step;

// 使用rayon并行处理物理
parallel_physics_step(&mut physics_world, &mut transform_query);
```

**建议**：
- 仅对大量独立刚体使用并行
- 确保线程池大小与CPU核心数匹配
- 监控并行开销

---

## 内存优化

### Arena分配器

**使用Arena分配器处理临时数据**

```rust
use game_engine::performance::memory::Arena;

let mut arena = Arena::new();

{
    // 临时分配
    let temp_data = arena.alloc_vec::<u8>(1024);
    // 使用temp_data...
}

// arena.drop()时自动释放所有内存
```

**适用场景**：
- 帧内临时数据
- 按生命周期分组的分配
- 需要批量释放的场景

### 对象池

**重用频繁创建销毁的对象**

```rust
use game_engine::performance::memory::ObjectPool;

let mut pool = ObjectPool::new(100, || MyObject::new());

// 从池中获取
let obj = pool.acquire();

// 使用对象...

// 返回到池中
pool.release(obj);
```

**适用场景**：
- 粒子系统
- 投射物
- AI实体

### 环形缓冲池

**用于固定大小的缓冲区**

```rust
use game_engine::resources::ring_buffer_pool::RingBufferPool;

let pool = RingBufferPool::new(10, 1024);

// 获取缓冲区
let buffer = pool.acquire();

// 使用缓冲区...

pool.release(buffer);
```

---

## 批处理优化

### 动态批处理

**合并相似绘制调用**

```rust
use game_engine::render::batch_optimizer::BatchOptimizer;

let optimizer = BatchOptimizer::new(config);

// 收集绘制调用
optimizer.collect_draw_call(draw_call);

// 优化并获取批次
let optimized_batches = optimizer.optimize();
```

### 实例批处理

**使用GPU实例化**

```rust
use game_engine::render::instance_batch::BatchManager;

// 添加实例数据
batch_manager.add_instance(mesh_id, instance_data);

// 渲染批次
for batch in batch_manager.get_visible_batches() {
    renderer.render_batch(batch);
}
```

---

## 异步I/O优化

### 资源异步加载

**使用协程加载器**

```rust
use game_engine::resources::CoroutineLoader;

let loader = CoroutineLoader::new();

// 高优先级加载关键资源
loader.load_critical("player_mesh.gltf");

// 后台预加载
loader.preload("level2.gltf", Priority::Low);
```

### 预加载管理器

**提前加载下一场景资源**

```rust
use game_engine::resources::PreloadManager;

let mut preload_manager = PreloadManager::new();

// 添加预加载任务
preload_manager.add_task("next_level_meshes", vec![
    "mesh1.gltf",
    "mesh2.gltf",
]);

// 在场景切换时检查完成状态
if preload_manager.is_complete("next_level_meshes") {
    // 切换场景
}
```

---

## 性能分析

### 使用Metrics存储

```rust
use game_engine::performance::tracing_metrics::TracingMetricsManager;

let manager = TracingMetricsManager::new();

// 记录性能指标
manager.record_metric("frame_time", 16.67);
manager.record_metric("draw_calls", 1500);

// 查询聚合统计
let agg = manager.query_metric_aggregate("frame_time", Some(Duration::from_secs(5)));
```

### 使用性能分析器

```rust
use game_engine::profiling::ContinuousProfiler;

let profiler = ContinuousProfiler::new(300);

// 在帧开始时记录
profiler.start_frame();

// 执行渲染...
// 执行物理...

// 在帧结束时记录
profiler.end_frame();

// 分析性能
let analysis = profiler.analyze();
if analysis.avg_frame_time > 16.67 {
    // 性能问题，需要优化
}
```

---

## 通用优化建议

### 1. 避免过早优化

- 首先测量性能瓶颈
- 使用profiler找到真正的瓶颈
- 优先优化最影响性能的部分

### 2. 监控性能

- 在开发过程中持续监控FPS
- 使用性能分析工具
- 定期检查内存使用情况

### 3. 渐进式优化

- 每次只做一个优化
- 验证优化效果
- 避免回归

### 4. 平衡质量和性能

- 根据目标平台调整质量设置
- 提供可配置的图形选项
- 使用动态质量调整

---

## 性能目标参考

| 平台 | 目标FPS | 绘制调用数 | 三角形数量 |
|--------|----------|-------------|-------------|
| 高端PC | 120+ | 10000+ | 10M+ |
| 中端PC | 60+ | 5000+ | 5M+ |
| 低端PC | 30+ | 2000+ | 2M+ |
| 移动设备 | 30-60 | 1000+ | 1M+ |

---

## 工具和资源

### 内置工具

- **Performance Dashboard**：实时性能监控
- **Metrics Storage**：性能数据存储和查询
- **Continuous Profiler**：持续性能分析
- **Bottleneck Detector**：性能瓶颈检测

### 推荐工具

- **RenderDoc**：图形调试
- **Tracy Profiler**：性能分析
- **Valgrind**：内存分析
- **Intel VTune**：CPU性能分析
