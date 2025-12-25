# 光线追踪集成指南

## 概述

本文档介绍游戏引擎的光线追踪集成，包括硬件加速（RTX/DXR）支持和优化的软件光线追踪实现。

## 功能特性

### 硬件加速支持

- **NVIDIA RTX**: 自动检测RTX系列GPU
- **AMD RDNA2+**: 支持RX 6000系列及以上
- **Intel Arc**: 支持Alchemist及以上
- **自动降级**: 硬件不可用时自动降级到软件光线追踪

### 软件光线追踪

- **计算着色器实现**: 基于WGSL的高性能实现
- **BVH加速**: 可选的BVH加速结构
- **自适应质量**: 根据性能自动调整质量

## 使用方法

### 基本使用

```rust
use game_engine::render::{
    RayTracingConfigEnhanced, 
    RayTracingRendererEnhanced,
    RayTracingAcceleration,
    RayTracingScene,
};

// 创建配置
let config = RayTracingConfigEnhanced {
    enabled: true,
    acceleration: RayTracingAcceleration::Hardware, // 或 Software
    rays_per_pixel: 1,
    max_bounces: 2,
    resolution_scale: 0.5,
    soft_shadows: true,
    global_illumination: false,
    ambient_occlusion: true,
    use_bvh: true,
    adaptive_quality: true,
    target_fps: 60.0,
};

// 创建渲染器
let renderer = RayTracingRendererEnhanced::new(
    &device,
    &adapter,
    config
)?;

// 准备输出纹理
renderer.prepare_output(&device, width, height)?;

// 更新场景
let scene = RayTracingScene {
    spheres: vec![],
    planes: vec![],
    lights: vec![],
    ambient_color: Vec3::new(0.1, 0.1, 0.1),
};
renderer.update_scene(&device, &queue, &scene)?;

// 创建绑定组
let bind_group = renderer.create_bind_group(&device)?;

// 渲染
renderer.render(&mut encoder, &bind_group, &camera)?;
```

### 硬件加速检测

```rust
// 检测硬件加速支持
let hardware_supported = RayTracingRendererEnhanced::detect_hardware_acceleration(&adapter);

if hardware_supported {
    println!("硬件光线追踪可用");
} else {
    println!("使用软件光线追踪");
}
```

### 性能监控

```rust
// 更新性能统计
renderer.update_performance_stats(frame_time_ms, rt_time_ms);

// 获取性能统计
let stats = renderer.performance_stats();
println!("平均帧时间: {:.2}ms", stats.avg_frame_time_ms);
println!("光线追踪时间: {:.2}ms", stats.ray_tracing_time_ms);
println!("当前FPS: {:.2}", stats.current_fps);
```

## 配置选项

### RayTracingConfigEnhanced

- `enabled`: 是否启用光线追踪
- `acceleration`: 加速类型（Hardware/Software/Disabled）
- `rays_per_pixel`: 每个像素的光线数量（1-16）
- `max_bounces`: 最大反射次数（0-8）
- `resolution_scale`: 分辨率缩放（0.1-1.0）
- `soft_shadows`: 是否启用软阴影
- `global_illumination`: 是否启用全局光照
- `ambient_occlusion`: 是否启用环境光遮蔽
- `use_bvh`: 是否使用BVH加速
- `adaptive_quality`: 是否启用自适应质量
- `target_fps`: 目标帧率（用于自适应质量）

## 性能优化建议

1. **分辨率缩放**: 使用0.5-0.75的缩放比例可以显著提升性能
2. **光线数量**: 每个像素1-2条光线通常足够
3. **反射次数**: 2-3次反射通常提供良好的视觉效果
4. **BVH加速**: 在复杂场景中启用BVH可以提升性能
5. **自适应质量**: 启用自适应质量可以自动平衡性能和视觉效果

## 硬件要求

### 硬件加速（推荐）

- **NVIDIA**: RTX 2060或更高
- **AMD**: RX 6000系列或更高
- **Intel**: Arc A7系列或更高

### 软件光线追踪

- 任何支持计算着色器的GPU
- 建议：GTX 1060或更高（6GB VRAM）

## 限制和注意事项

1. **WGPU限制**: WGPU目前不直接暴露RTX/DXR API，硬件检测主要基于GPU名称
2. **性能**: 软件光线追踪性能较低，建议在简单场景中使用
3. **内存**: BVH结构需要额外的GPU内存
4. **兼容性**: 某些旧GPU可能不支持所需的计算着色器特性

## 未来计划

- [ ] 完整的BVH构建算法
- [ ] 更复杂的光线-几何体相交测试
- [ ] 材质系统集成
- [ ] 降噪支持
- [ ] 时间累积（TAA）

## 更多信息

- [渲染API参考](../api_reference.md)
- [性能调优指南](./performance_tuning_guide.md)

