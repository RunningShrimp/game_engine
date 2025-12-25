# GPU物理加速扩展指南

## 概述

本文档介绍游戏引擎的GPU物理加速扩展功能，包括粒子系统GPU物理模拟和流体模拟（SPH）。

## GPU粒子物理

### 功能特性

- **大规模粒子支持**: 支持65536+粒子的实时物理模拟
- **GPU碰撞检测**: 粒子间碰撞检测完全在GPU上执行
- **力场系统**: 支持引力、斥力、涡流等多种力场
- **粒子间相互作用**: 支持粒子间的相互作用力计算

### 使用方法

```rust
use game_engine::physics::{
    GpuParticlePhysicsAccelerator, GpuParticlePhysicsConfig, GpuParticle,
};
use std::sync::Arc;

// 创建配置
let config = GpuParticlePhysicsConfig {
    enabled: true,
    max_particles: 65536,
    workgroup_size: 64,
    collision_radius: 0.1,
    interaction_radius: 0.5,
    enable_collision: true,
    enable_force_fields: true,
};

// 创建加速器
let accelerator = GpuParticlePhysicsAccelerator::new(
    device.clone(),
    queue.clone(),
    config,
);

// 准备粒子数据
let particles: Vec<GpuParticle> = vec![
    GpuParticle {
        position: [0.0, 0.0, 0.0],
        velocity: [1.0, 0.0, 0.0],
        force: [0.0, 0.0, 0.0],
        mass: 1.0,
        radius: 0.1,
        _padding: [0.0; 2],
    },
    // ... 更多粒子
];

// 更新粒子物理
let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
    label: Some("Particle Physics Update"),
});

accelerator.update_particles(&mut encoder, &particles, delta_time)?;

// 提交命令
queue.submit(Some(encoder.finish()));
```

### 配置选项

- `max_particles`: 最大粒子数量（默认65536）
- `workgroup_size`: GPU工作组大小（默认64）
- `collision_radius`: 碰撞检测半径
- `interaction_radius`: 粒子间相互作用范围
- `enable_collision`: 是否启用碰撞检测
- `enable_force_fields`: 是否启用力场计算

## GPU流体模拟（SPH）

### 功能特性

- **SPH算法**: 使用平滑粒子流体动力学（SPH）算法
- **密度计算**: 实时计算粒子密度
- **压力计算**: 基于密度的压力计算
- **粘性力**: 支持粘性力计算
- **表面张力**: 支持表面张力效果

### 使用方法

```rust
use game_engine::physics::{
    GpuFluidSimulator, GpuFluidSimulationConfig, GpuFluidParticle,
};
use std::sync::Arc;

// 创建配置
let config = GpuFluidSimulationConfig {
    enabled: true,
    max_particles: 16384,
    workgroup_size: 64,
    smoothing_radius: 0.2,
    rest_density: 1000.0,
    pressure_constant: 2000.0,
    viscosity: 0.018,
    surface_tension: 0.0728,
    time_step: 0.001,
};

// 创建模拟器
let simulator = GpuFluidSimulator::new(
    device.clone(),
    queue.clone(),
    config,
);

// 准备流体粒子数据
let particles: Vec<GpuFluidParticle> = vec![
    GpuFluidParticle {
        position: [0.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0],
        density: 1000.0,
        pressure: 0.0,
        mass: 0.02,
        _padding: [0.0; 2],
    },
    // ... 更多粒子
];

// 模拟流体
let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
    label: Some("Fluid Simulation"),
});

simulator.simulate(&mut encoder, &particles, delta_time)?;

// 提交命令
queue.submit(Some(encoder.finish()));
```

### 配置选项

- `max_particles`: 最大粒子数量（默认16384）
- `smoothing_radius`: SPH平滑半径
- `rest_density`: 静止密度（水的密度约为1000.0）
- `pressure_constant`: 压力常数（控制流体压缩性）
- `viscosity`: 粘性系数
- `surface_tension`: 表面张力系数
- `time_step`: 时间步长（建议0.001或更小）

### SPH算法说明

SPH（Smoothed Particle Hydrodynamics）是一种基于粒子的流体模拟方法：

1. **密度计算**: 使用平滑核函数计算每个粒子的密度
2. **压力计算**: 基于密度差异计算压力
3. **力计算**: 计算压力力、粘性力等
4. **位置更新**: 根据力更新粒子位置和速度

## 性能优化建议

### 粒子物理优化

1. **粒子数量**: 根据GPU性能调整最大粒子数量
2. **工作组大小**: 使用64或128的工作组大小
3. **碰撞检测**: 对于不需要碰撞的场景，禁用碰撞检测
4. **力场**: 限制力场数量以提高性能

### 流体模拟优化

1. **粒子数量**: 16384个粒子通常足够大多数场景
2. **平滑半径**: 较小的平滑半径提高性能但降低质量
3. **时间步长**: 使用自适应时间步长以提高稳定性
4. **空间分区**: 使用空间分区优化邻居搜索（未来功能）

## 硬件要求

### 粒子物理

- **推荐**: GTX 1060或更高（6GB VRAM）
- **粒子数量65536**: 需要约2MB VRAM
- **工作组大小64**: 适合大多数GPU

### 流体模拟

- **推荐**: GTX 1070或更高（8GB VRAM）
- **粒子数量16384**: 需要约1MB VRAM
- **计算密集**: 需要较强的GPU计算能力

## 限制和注意事项

1. **粒子物理限制**:
   - 碰撞检测使用简化的球-球碰撞
   - 力场数量有限
   - 粒子间相互作用计算复杂度为O(n²)

2. **流体模拟限制**:
   - SPH算法计算复杂度为O(n²)
   - 需要稳定的时间步长
   - 边界处理较简单

## 未来计划

- [ ] 空间分区优化（降低复杂度到O(n log n)）
- [ ] 自适应时间步长
- [ ] 更精确的碰撞检测
- [ ] 表面重建和渲染
- [ ] 多相流体支持

## 更多信息

- [GPU物理加速](./gpu_acceleration.md)
- [物理系统API参考](../api_reference.md)
- [性能调优指南](./performance_tuning_guide.md)

