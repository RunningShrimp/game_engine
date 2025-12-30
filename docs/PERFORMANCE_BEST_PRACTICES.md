# 性能最佳实践

**版本**: v1.0
**最后更新**: 2025-12-30
**状态**: 活跃维护

## 概述

本文档提供游戏引擎的性能优化最佳实践，涵盖渲染、物理、内存管理、并发编程等关键领域。基于实际优化经验和基准测试数据，提供可操作的性能提升策略。

## 目录

1. [性能优化原则](#性能优化原则)
2. [渲染性能](#渲染性能)
3. [物理性能](#物理性能)
4. [内存管理](#内存管理)
5. [并发编程](#并发编程)
6. [协程与异步](#协程与异步)
7. [SIMD优化](#simd优化)
8. [GPU加速](#gpu加速)
9. [性能测试](#性能测试)
10. [常见性能陷阱](#常见性能陷阱)

---

## 性能优化原则

### 1. 测量优先，优化其次

**原则**: 不要猜测瓶颈，使用性能分析工具找到真正的热点。

```bash
# CPU性能分析
cargo flamegraph --bin game_engine

# 内存分析
valgrind --tool=massif ./target/release/game_engine

# 性能基准测试
cargo bench --bench optimization_benchmarks
```

### 2. 避免过早优化

**原则**: 先实现正确功能，性能问题出现时再优化。

```rust
// ✅ 正确：先实现清晰、正确的代码
fn process_entities(entities: &[Entity]) -> Vec<Entity> {
    entities.iter()
        .filter(|e| e.active)
        .map(|e| transform(e))
        .collect()
}

// 性能分析显示这是瓶颈后再优化
fn process_entities_optimized(entities: &[Entity]) -> Vec<Entity> {
    // 使用SIMD、并行等优化技术
}
```

### 3. 渐进式优化

**原则**: 每次只做一个优化，验证效果后再继续。

```bash
# 优化流程
1. 运行基准测试 -> 记录基线
2. 应用单一优化
3. 再次运行基准测试
4. 对比性能提升
5. 重复步骤2-4
```

### 4. 平衡质量和性能

**原则**: 根据目标平台调整质量设置。

```rust
pub struct QualitySettings {
    pub shadows: ShadowQuality,
    pub ssao: bool,
    pub reflections: bool,
    pub lod_level: usize,
}

impl QualitySettings {
    pub fn for_target_fps(fps: u32) -> Self {
        match fps {
            120 => Self::ultra(),
            60 => Self::high(),
            30 => Self::medium(),
            _ => Self::low(),
        }
    }
}
```

---

## 渲染性能

### 1. GPU驱动渲染

**启用GPU驱动剔除**

```rust
use game_engine::render::gpu_driven::{GpuDrivenRenderer, GpuDrivenConfig};

let config = GpuDrivenConfig {
    frustum_culling: true,    // 视锥剔除
    occlusion_culling: false,   // 按需启用遮挡剔除
    lod_enabled: true,         // LOD
    max_instances: 65536,
    workgroup_size: 64,
};

let renderer = GpuDrivenRenderer::new(device, config)?;
```

**预期性能提升**: 30-50%（取决于场景复杂度）

**适用场景**:
- 大量动态物体
- 复杂场景层次
- 开放世界

### 2. LOD系统

**配置合适的LOD级别**

```rust
use game_engine::render::lod::{LodConfig, LodLevel};

let config = LodConfig {
    lod_levels: vec![
        LodLevel::new(0, 0.0, 50.0, "high"),    // 高质量网格
        LodLevel::new(1, 50.0, 100.0, "medium"), // 中等网格
        LodLevel::new(2, 100.0, 200.0, "low"),   // 低质量网格
    ],
    quality: LodQuality::Balanced,
    transition_distance: 10.0,  // 平滑过渡
};
```

**优化建议**:
- 远距离物体使用简化的网格
- 远距离光照使用烘焙
- 减少远距离阴影精度

**性能提升**: 减少40-60%三角形数量

### 3. 批处理优化

**合并相似绘制调用**

```rust
use game_engine::render::batch_optimizer::BatchOptimizer;

let optimizer = BatchOptimizer::new(config);

// 收集绘制调用
for draw_call in frame_draw_calls {
    optimizer.collect_draw_call(draw_call);
}

// 优化并获取批次
let optimized_batches = optimizer.optimize();

// 渲染批次
for batch in optimized_batches {
    renderer.render_batch(batch);
}
```

**优化策略**:
- 按材质排序绘制调用
- 使用实例化渲染
- 合并小批次

**性能提升**: 减少50-70%绘制调用

### 4. 实例批处理

**使用GPU实例化**

```rust
use game_engine::render::instance_batch::BatchManager;

let mut batch_manager = BatchManager::new(config);

// 添加实例数据
for instance in instances {
    batch_manager.add_instance(mesh_id, instance_data);
}

// 渲染批次
for batch in batch_manager.get_visible_batches() {
    renderer.render_batch_instanced(batch);
}
```

**适用场景**:
- 大量相同物体（树木、草、石头）
- 粒子系统
- UI元素

**性能提升**: 10-100倍（取决于实例数量）

### 5. 后处理优化

**仅启用需要的后处理效果**

```rust
pub struct PostProcessingConfig {
    pub ssao: bool,      // 屏幕空间环境光遮蔽
    pub bloom: bool,     // 泛光
    pub taa: bool,       // 时间抗锯齿
    pub motion_blur: bool, // 运动模糊
}

impl PostProcessingConfig {
    pub fn for_target_fps(fps: u32) -> Self {
        match fps {
            120 => Self {
                ssao: true,
                bloom: true,
                taa: true,
                motion_blur: false,
            },
            60 => Self {
                ssao: true,
                bloom: true,
                taa: false,
                motion_blur: false,
            },
            _ => Self {
                ssao: false,
                bloom: false,
                taa: false,
                motion_blur: false,
            },
        }
    }
}
```

---

## 物理性能

### 1. 空间分区

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

**性能提升**: 10-100倍（取决于物体数量）

**选择指南**:
- BVH: 静态或很少移动的物体
- 空间哈希: 高度动态的物体
- 混合: 静态物体用BVH，动态用空间哈希

### 2. 脏追踪同步

**避免不必要的同步**

```rust
use game_engine::physics::PhysicsSyncConfig;

let config = PhysicsSyncConfig {
    sync_threshold: 0.001,    // 仅在移动超过阈值时同步
    batch_size: 100,           // 批量同步大小
    enable_dirty_tracking: true,
};
```

**性能提升**: 减少80-90%同步操作

### 3. 并行物理计算

**使用rayon并行处理物理**

```rust
use game_engine::physics::parallel::parallel_physics_step;
use rayon::prelude::*;

fn update_physics_parallel(world: &mut PhysicsWorld) {
    // 并行更新所有刚体
    world.bodies.par_iter_mut().for_each(|body| {
        body.integrate_velocity();
        body.apply_forces();
    });

    // 并行碰撞检测
    let pairs = find_collision_pairs_parallel(&world.bodies);
    pairs.par_iter().for_each(|pair| {
        resolve_collision(pair);
    });
}
```

**性能提升**: 2-4倍（取决于CPU核心数）

**注意事项**:
- 确保线程池大小与CPU核心数匹配
- 监控并行开销
- 仅对大量独立刚体使用

### 4. 固定时间步长

**使用固定时间步长保证确定性**

```rust
const PHYSICS_DT: f32 = 1.0 / 60.0;  // 60 FPS

struct PhysicsLoop {
    accumulator: f32,
}

impl PhysicsLoop {
    pub fn update(&mut self, mut delta_time: f32, world: &mut PhysicsWorld) {
        self.accumulator += delta_time;

        // 固定时间步长更新
        while self.accumulator >= PHYSICS_DT {
            world.step(PHYSICS_DT);
            self.accumulator -= PHYSICS_DT;
        }
    }
}
```

---

## 内存管理

### 1. Arena分配器

**处理临时数据**

```rust
use game_engine::performance::memory::Arena;

let mut arena = Arena::new();

{
    // 临时分配
    let temp_data = arena.alloc_vec::<u8>(1024);
    let temp_buffer = arena.alloc_array::<f32>(512);

    // 使用临时数据...
}

// arena.drop()时自动释放所有内存
arena.drop();
```

**适用场景**:
- 帧内临时数据
- 按生命周期分组的分配
- 需要批量释放的场景

**性能提升**: 减少90%+堆分配

### 2. 对象池

**重用频繁创建销毁的对象**

```rust
use game_engine::performance::memory::ObjectPool;

let mut pool = ObjectPool::new(100, || Particle::new());

// 从池中获取
let mut particle = pool.acquire();

// 使用粒子
particle.position = (10.0, 20.0, 30.0);
particle.velocity = (1.0, 2.0, 3.0);
particle.update(dt);

// 返回到池中
pool.release(particle);
```

**适用场景**:
- 粒子系统
- 投射物
- AI实体
- 事件对象

**性能提升**: 减少95%+堆分配

### 3. 环形缓冲池

**固定大小的缓冲区管理**

```rust
use game_engine::resources::ring_buffer_pool::RingBufferPool;

let pool = RingBufferPool::new(10, 1024);

// 获取缓冲区
let buffer = pool.acquire();

// 使用缓冲区
buffer.write(data);

// 释放缓冲区
pool.release(buffer);
```

**适用场景**:
- 网络数据包缓冲
- 音频缓冲
- 渲染命令缓冲

### 4. 内存预分配

**减少运行时分配**

```rust
// ❌ 错误：动态增长
let mut entities = Vec::new();
for i in 0..1000 {
    entities.push(Entity::new());  // 可能多次重新分配
}

// ✅ 正确：预分配
let mut entities = Vec::with_capacity(1000);
for i in 0..1000 {
    entities.push(Entity::new());  // 无重新分配
}
```

**HashMap预分配**:

```rust
use std::collections::HashMap;

// ✅ 正确：预分配
let mut map = HashMap::with_capacity(1000);
for i in 0..1000 {
    map.insert(i, value);
}
```

---

## 并发编程

### 1. 使用parking_lot

**更快的锁实现**

```rust
// 使用parking_lot替代std::sync
use parking_lot::{Mutex, RwLock};

// RwLock性能提升
let data = RwLock::new(MyData::new());

// 读锁 - 2.5x faster
let r = data.read();

// 写锁 - 4x faster
let w = data.write();
```

**性能对比**:

| 操作 | std::sync::RwLock | parking_lot::RwLock | 提升 |
|------|-------------------|--------------------|------|
| 读锁 | 100ns | 40ns | 2.5x |
| 写锁 | 200ns | 50ns | 4x |
| 争用读 | 500ns | 100ns | 5x |
| 争用写 | 1000ns | 125ns | 8x |

### 2. 使用DashMap

**并发HashMap**

```rust
use dashmap::DashMap;

let map = DashMap::new();

// 插入 - 无锁或细粒度锁
map.insert(key, value);

// 获取 - 几乎无锁
if let Some(value) = map.get(&key) {
    // 使用value
}

// 更新 - 细粒度锁
map.entry(key).and_modify(|v| *v += 1);
```

**性能提升**: 10-20倍（并发场景）

### 3. 减少锁粒度

**细粒度锁优于粗粒度锁**

```rust
// ❌ 错误：粗粒度锁
struct World {
    data: RwLock<WorldData>,
}

struct WorldData {
    entities: Vec<Entity>,
    resources: Vec<Resource>,
    physics: PhysicsState,
    rendering: RenderState,
}

// ✅ 正确：细粒度锁
struct World {
    entities: RwLock<Vec<Entity>>,
    resources: RwLock<Vec<Resource>>,
    physics: RwLock<PhysicsState>,
    rendering: RwLock<RenderState>,
}
```

### 4. 无锁设计

**使用原子操作和通道**

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;

// 原子计数器
struct Counter {
    value: AtomicUsize,
}

impl Counter {
    fn increment(&self) -> usize {
        self.value.fetch_add(1, Ordering::Relaxed)
    }
}

// 通道通信
let (tx, rx) = mpsc::channel();

// 生产者
tx.send(data)?;

// 消费者
while let Ok(data) = rx.recv() {
    process(data);
}
```

---

## 协程与异步

### 1. 避免过度异步

**纯计算应同步**

```rust
// ❌ 错误：纯计算不应该异步
pub async fn calculate_physics(&self, position: Vec3, velocity: Vec3, dt: f32) -> Vec3 {
    position + velocity * dt
}

// ✅ 正确：使用同步函数
pub fn calculate_physics(&self, position: Vec3, velocity: Vec3, dt: f32) -> Vec3 {
    position + velocity * dt
}
```

**性能对比**:

| 操作 | 异步版本 | 同步版本 | 提升 |
|------|---------|---------|------|
| calculate_physics | 500ns | 50ns | 10x |
| vector_add | 400ns | 40ns | 10x |
| get_entity_count | 300ns | 20ns | 15x |

### 2. 使用rayon并行

**CPU密集型任务**

```rust
use rayon::prelude::*;

// 并行迭代
entities.par_iter_mut().for_each(|entity| {
    entity.update();
});

// 并行映射
let results: Vec<_> = items.par_iter()
    .map(|item| process(item))
    .collect();

// 并行归约
let sum: i32 = numbers.par_iter()
    .sum();
```

**性能提升**: 2-8倍（取决于CPU核心数）

### 3. 异步I/O优化

**大文件I/O保持异步**

```rust
// ✅ 正确：大文件异步加载
pub async fn_load_large_asset(&self, path: &Path) -> Result<Vec<u8>, Error> {
    tokio::fs::read(path).await
}

// ✅ 正确：小文件同步加载
pub fn load_small_asset(&self, path: &Path) -> Result<Vec<u8>, Error> {
    std::fs::read(path)
}
```

**阈值建议**:
- <1KB: 同步更快
- 1KB-100KB: 取决于场景
- >100KB: 异步更好

### 4. 批量处理

**减少协程开销**

```rust
use futures::future::join_all;

// ❌ 错误：逐个处理
for item in items {
    process_item(item).await;
}

// ✅ 正确：批量处理
let tasks: Vec<_> = items.into_iter()
    .map(|item| process_item(item))
    .collect();

let results = join_all(tasks).await;
```

---

## SIMD优化

### 1. 向量运算

**使用SIMD加速**

```rust
use game_engine::math::simd::{Vec3Simd, Vec4Simd};

// 批量处理向量
let positions: Vec<Vec3> = /* ... */;
let simd_positions = Vec3Simd::from_slice(&positions);

// SIMD批量计算
let result = simd_positions * 2.0;
```

**性能提升**: 2-4倍（取决于SIMD宽度）

### 2. 数据对齐

**确保16字节对齐**

```rust
#[repr(align(16))]
struct AlignedData {
    positions: [Vec3; 4],
}
```

### 3. 批量处理

**充分利用SIMD**

```rust
// ✅ 正确：批量处理
for chunk in positions.chunks(4) {
    let simd = Vec3Simd::from_slice(chunk);
    process_simd(simd);
}

// ❌ 错误：逐个处理
for pos in positions {
    process(pos);  // 无法利用SIMD
}
```

### 4. 回退机制

**提供非SIMD实现**

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[target_feature(enable = "sse2")]
unsafe fn process_vector_sse2(data: &[f32]) -> Vec<f32> {
    // SSE2实现
}

#[cfg(not(target_arch = "x86_64"))]
fn process_vector_sse2(data: &[f32]) -> Vec<f32> {
    // 标量回退实现
}
```

---

## GPU加速

### 1. 计算着色器

**大规模并行计算**

```rust
use game_engine::performance::gpu::gpu_compute::GpuComputeContext;

let context = GpuComputeContext::new(device, queue)?;

// 配置计算着色器
let config = ComputeShaderConfig {
    workgroup_size: 64,
    max_workgroups: 1024,
    enable_shared_memory: true,
};

// 执行GPU计算
context.compute(&shader, &buffers, config)?;
```

**适用场景**:
- 粒子系统（10万+粒子）
- 批量寻路（1000+智能体）
- 大规模物理模拟

**性能提升**: 10-100倍

### 2. 减少数据传输

**最小化CPU-GPU传输**

```rust
// ❌ 错误：频繁小数据传输
for item in items {
    buffer.write(&item);  // 每次传输都有开销
}

// ✅ 正确：批量传输
buffer.write_all(&items);
```

### 3. 共享内存

**使用共享内存优化**

```rust
let config = ComputeShaderConfig {
    enable_shared_memory: true,
    shared_memory_size: 16384,  // 16KB
    // ...
};
```

### 4. CPU回退

**提供CPU实现**

```rust
#[cfg(feature = "gpu_compute")]
fn compute_on_gpu() -> Result<Data, Error> {
    // GPU实现
}

#[cfg(not(feature = "gpu_compute"))]
fn compute_on_gpu() -> Result<Data, Error> {
    // CPU回退实现
}
```

---

## 性能测试

### 1. 基准测试

**使用Criterion**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_calculate_physics(c: &mut Criterion) {
    c.bench_function("calculate_physics", |b| {
        b.iter(|| {
            calculate_physics(
                black_box(Vec3::new(0.0, 0.0, 0.0)),
                black_box(Vec3::new(1.0, 2.0, 3.0)),
                black_box(0.016),
            )
        })
    });
}

criterion_group!(benches, benchmark_calculate_physics);
criterion_main!(benches);
```

### 2. 性能监控

**实时性能指标**

```rust
use game_engine::profiling::SystemPerformanceMonitor;

let monitor = SystemPerformanceMonitor::new();

loop {
    monitor.start_frame();

    // 游戏逻辑...

    monitor.end_frame();

    let metrics = monitor.get_metrics();
    if metrics.avg_frame_time > 16.67 {
        tracing::warn!("Frame time exceeded: {:?}", metrics.avg_frame_time);
    }
}
```

### 3. 内存分析

**追踪内存分配**

```rust
use game_engine::performance::memory::MemoryTracker;

let tracker = MemoryTracker::new();

{
    let _guard = tracker.track_scope();
    // 分配内存...
}

let stats = tracker.stats();
println!("Allocations: {}", stats.allocations);
println!("Deallocations: {}", stats.deallocations);
println!("Peak memory: {} bytes", stats.peak_memory);
```

### 4. 性能回归检测

**CI集成**

```yaml
# .github/workflows/benchmarks.yml
name: Benchmarks

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run benchmarks
        run: cargo bench --bench optimization_benchmarks
      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
```

---

## 常见性能陷阱

### 1. 过早优化

```rust
// ❌ 错误：过早优化
struct OptimizedVector {
    // 复杂的优化结构
}

// ✅ 正确：先实现简单版本
struct Vector {
    x: f32,
    y: f32,
    z: f32,
}
```

### 2. 忽略缓存友好性

```rust
// ❌ 错误：缓存不友好
struct AoS {
    positions: Vec<[f32; 3]>,
    velocities: Vec<[f32; 3]>,
}

// ✅ 正确：SoA布局
struct SoA {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
}
```

### 3. 忘记预分配

```rust
// ❌ 错误：动态增长
let mut vec = Vec::new();
for i in 0..1000 {
    vec.push(i);
}

// ✅ 正确：预分配
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 {
    vec.push(i);
}
```

### 4. 过度抽象

```rust
// ❌ 错误：过度抽象导致虚函数调用
trait Renderer {
    fn render(&self);
}

struct OpenGLRenderer;
struct VulkanRenderer;

// ✅ 正确：使用泛型内联
fn render<R: Renderer>(renderer: &R) {
    renderer.render();
}
```

### 5. 忽略分支预测

```rust
// ❌ 错误：随机分支
if entity.active {
    update(entity);
}

// ✅ 正确：排序后处理
entities.sort_by_key(|e| !e.active);  // 活跃实体在前
for entity in entities {
    if entity.active {
        update(entity);
    }
}
```

---

## 相关文档

- [OPTIMIZATION_GUIDE.md](./OPTIMIZATION_GUIDE.md) - 优化指南
- [OPTIMIZATION_STATUS.md](./OPTIMIZATION_STATUS.md) - 优化状态跟踪
- [performance_tuning_guide.md](./performance_tuning_guide.md) - 性能调优详细指南
- [benchmarking_guide.md](./benchmarking_guide.md) - 基准测试指南

---

**文档维护**: 本文档随引擎性能优化持续更新
**反馈**: 如有问题或建议，请提交Issue
**最后审核**: 2025-12-30
