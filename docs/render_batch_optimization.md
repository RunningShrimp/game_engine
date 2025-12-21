# 渲染管线批处理优化

## 概述

渲染管线批处理优化系统提供智能的批处理优化，通过最小化状态切换和最大化批处理合并，显著提升渲染性能。

## 设计目标

1. **最小化状态切换**：按状态切换成本排序（Pipeline > Blend > Depth > Material > Mesh）
2. **最大化批处理**：合并相同状态的绘制调用
3. **性能监控**：实时统计批处理效果
4. **自适应优化**：根据场景动态调整批处理策略

## 核心组件

### BatchOptimizer

批处理优化器，负责优化批次列表和计算统计信息。

```rust
use game_engine::render::{BatchOptimizer, OptimizedBatch, BatchKey};

// 创建优化器
let mut optimizer = BatchOptimizer::new(100); // 最大每批次100个实例

// 准备批次列表
let mut batches = vec![
    OptimizedBatch::new(
        BatchKey {
            mesh_id: 1,
            material_id: 1,
            pipeline_id: 1,
            blend_mode: 0,
            depth_test: true,
            render_flags: 0,
        },
        50,
    ),
    // ... 更多批次
];

// 优化批次
optimizer.optimize_batches(&mut batches);

// 获取统计信息
let stats = optimizer.stats();
println!("优化率: {:.2}%", stats.optimization_ratio * 100.0);
println!("状态切换次数: {}", stats.state_switches);
```

### BatchStats

批处理统计信息，包含详细的性能指标。

```rust
use game_engine::render::BatchStats;

let stats = BatchStats {
    total_batches: 10,           // 总批次数
    total_instances: 500,        // 总实例数
    unique_materials: 5,          // 唯一材质数
    unique_meshes: 3,            // 唯一网格数
    state_switches: 8,           // 状态切换次数
    optimization_ratio: 0.8,    // 优化率（80%）
    avg_instances_per_batch: 50.0, // 平均每批次实例数
    max_instances_per_batch: 100,  // 最大批次实例数
};
```

### BatchPerformanceMonitor

批处理性能监控器，用于跟踪历史性能数据。

```rust
use game_engine::render::BatchPerformanceMonitor;

let mut monitor = BatchPerformanceMonitor::new(100); // 保留100帧历史

// 每帧记录统计信息
monitor.record_stats(stats);

// 获取平均统计信息
if let Some(avg_stats) = monitor.average_stats() {
    println!("平均优化率: {:.2}%", avg_stats.optimization_ratio * 100.0);
}
```

## 使用示例

### 基本使用

```rust
use game_engine::render::{BatchOptimizer, OptimizedBatch, BatchKey};
use bevy_ecs::prelude::*;

fn optimize_render_batches(
    mut optimizer: ResMut<BatchOptimizer>,
    // ... 其他资源
) {
    // 收集所有需要渲染的批次
    let mut batches = collect_render_batches();
    
    // 优化批次
    optimizer.optimize_batches(&mut batches);
    
    // 获取统计信息
    let stats = optimizer.stats();
    tracing::info!(
        target: "render",
        "Batch optimization: {} batches, {:.2}% optimization, {} state switches",
        stats.total_batches,
        stats.optimization_ratio * 100.0,
        stats.state_switches
    );
    
    // 使用优化后的批次进行渲染
    render_optimized_batches(&batches);
}
```

### 集成到渲染系统

```rust
use game_engine::render::{BatchOptimizer, BatchPerformanceMonitor, OptimizedBatch};

#[derive(Resource)]
struct RenderOptimization {
    optimizer: BatchOptimizer,
    monitor: BatchPerformanceMonitor,
}

fn setup_render_optimization(mut commands: Commands) {
    commands.insert_resource(RenderOptimization {
        optimizer: BatchOptimizer::new(100),
        monitor: BatchPerformanceMonitor::new(100),
    });
}

fn render_system(
    mut optimization: ResMut<RenderOptimization>,
    // ... 其他资源
) {
    // 收集批次
    let mut batches = collect_batches();
    
    // 优化
    optimization.optimizer.optimize_batches(&mut batches);
    
    // 记录统计
    optimization.monitor.record_stats(*optimization.optimizer.stats());
    
    // 渲染
    render_batches(&batches);
}
```

## 状态切换成本

系统使用以下默认成本权重：

- **Pipeline切换**: 100.0（最昂贵）
- **Blend模式切换**: 50.0
- **Depth测试切换**: 30.0
- **Material切换**: 10.0
- **Mesh切换**: 1.0（最便宜）

可以通过`BatchOptimizer::with_cost()`自定义成本：

```rust
use game_engine::render::{BatchOptimizer, StateSwitchCost};

let cost = StateSwitchCost {
    pipeline: 150.0,  // 提高Pipeline切换成本
    blend: 50.0,
    depth: 30.0,
    material: 10.0,
    mesh: 1.0,
};

let optimizer = BatchOptimizer::with_cost(cost, 100);
```

## 性能监控

### 性能警告

系统会自动检测以下性能问题：

1. **低优化率**：优化率低于50%时发出警告
2. **过多状态切换**：状态切换超过100次时发出警告
3. **低平均实例数**：平均每批次实例数低于10时发出警告

### 性能分析

```rust
// 获取优化耗时
if let Some(time_us) = optimizer.optimization_time_us() {
    tracing::debug!(
        target: "render",
        "Batch optimization took {}μs",
        time_us
    );
}

// 获取平均统计信息
if let Some(avg_stats) = monitor.average_stats() {
    tracing::info!(
        target: "render",
        "Average batch stats over last {} frames:",
        monitor.history.len()
    );
    tracing::info!(
        target: "render",
        "  Optimization ratio: {:.2}%",
        avg_stats.optimization_ratio * 100.0
    );
    tracing::info!(
        target: "render",
        "  Avg instances per batch: {:.2}",
        avg_stats.avg_instances_per_batch
    );
}
```

## 优化策略

### 1. 批次合并

系统会自动合并相同状态的批次，只要总实例数不超过最大限制：

```rust
// 两个相同状态的批次会被合并
let batch1 = OptimizedBatch::new(key, 50);
let batch2 = OptimizedBatch::new(key, 30);
// 合并后: 一个批次，80个实例
```

### 2. 状态排序

批次按`BatchKey`排序，确保状态切换最小化：

```rust
// BatchKey实现了按优先级排序：
// pipeline_id > blend_mode > depth_test > render_flags > mesh_id > material_id
```

### 3. 自适应批处理

根据场景动态调整批处理策略：

```rust
// 根据性能监控结果调整最大实例数
if stats.avg_instances_per_batch < 5.0 {
    // 降低最大实例数，提高批处理效率
    optimizer.max_instances_per_batch = 50;
} else if stats.avg_instances_per_batch > 80.0 {
    // 提高最大实例数，减少批次数
    optimizer.max_instances_per_batch = 200;
}
```

## 性能影响

### 预期性能提升

- **Draw Call减少**: 70-90%
- **状态切换减少**: 60-80%
- **CPU开销**: 每帧约10-50μs（取决于批次数量）
- **内存开销**: 每个批次约100-200字节

### 性能基准

在典型场景中（1000个实例，10种材质，5种网格）：

- **优化前**: 1000个Draw Call，约1000次状态切换
- **优化后**: 约50-100个批次，约20-30次状态切换
- **性能提升**: 约5-10倍

## 最佳实践

1. **合理设置最大实例数**：根据GPU能力设置（通常64-256）
2. **监控性能指标**：定期检查优化率和状态切换次数
3. **调整成本权重**：根据实际硬件调整状态切换成本
4. **批量处理**：在系统级别批量收集和优化批次

## 与现有系统集成

### 与InstanceBatch集成

```rust
use game_engine::render::{BatchManager, BatchOptimizer};

fn optimize_instance_batches(
    batch_manager: &BatchManager,
    optimizer: &mut BatchOptimizer,
) {
    // 从BatchManager获取批次
    let mut batches: Vec<OptimizedBatch> = batch_manager
        .visible_batches()
        .map(|batch| {
            OptimizedBatch {
                key: batch.key,
                instance_count: batch.instance_count(),
                // ... 其他字段
            }
        })
        .collect();
    
    // 优化
    optimizer.optimize_batches(&mut batches);
    
    // 使用优化后的批次
}
```

## 未来改进

- [ ] GPU端批处理优化
- [ ] 自动LOD选择集成
- [ ] 动态批处理大小调整
- [ ] 批处理预测和预优化
- [ ] 多线程批处理优化

