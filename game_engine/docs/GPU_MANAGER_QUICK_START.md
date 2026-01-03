# GPU管理器快速开始指南

## 概述

本指南将帮助您快速上手游戏引擎的增强GPU管理器，实现高性能的GPU驱动渲染。

---

## 前置条件

- Rust 1.70+
- 支持WebGPU的硬件和驱动
- 基本的图形编程知识

---

## 1. 基本使用

### 1.1 创建GPU管理器

```rust
use game_engine::render::gpu_unified_manager_v2::{
    EnhancedGpuRenderConfig, EnhancedGpuRenderManager,
};

// 使用默认配置
let mut manager = EnhancedGpuRenderManager::default_config(&device)?;

// 或使用自定义配置
let config = EnhancedGpuRenderConfig {
    enable_frustum_culling: true,
    enable_distance_culling: true,
    max_view_distance: 1000.0,
    ..Default::default()
};

let mut manager = EnhancedGpuRenderManager::new(&device, config)?;
```

### 1.2 准备实例数据

```rust
use game_engine::render::gpu_driven::culling::GpuInstance;

// 创建实例
let mut instances = Vec::new();

for i in 0..1000 {
    let mut instance = GpuInstance::default();

    // 设置变换矩阵
    instance.model[3][0] = x;
    instance.model[3][1] = y;
    instance.model[3][2] = z;

    // 设置AABB（轴对齐包围盒）
    instance.aabb_min = [-0.5, -0.5, -0.5];
    instance.aabb_max = [0.5, 0.5, 0.5];

    // 设置实例ID
    instance.instance_id = i;

    instances.push(instance);
}

// 上传到GPU
manager.update_instances(&device, &queue, &instances);
```

### 1.3 执行渲染

```rust
// 创建命令编码器
let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
    label: Some("Render Encoder"),
});

// 计算视图投影矩阵
let view_proj = compute_view_projection_matrix();

// 获取相机位置
let camera_pos = (camera.position.x, camera.position.y, camera.position.z);

// 执行GPU渲染（包含剔除）
let stats = manager.render(
    &mut encoder,
    &device,
    &queue,
    view_proj,
    camera_pos,
    instances.len() as u32,
)?;

// 提交命令
queue.submit(Some(encoder.finish()));

// 查看统计信息
println!("Visible instances: {}", stats.visible_instances);
println!("Cull rate: {:.1}%", stats.cull_rate * 100.0);
println!("GPU time: {:.2}ms", stats.gpu_time_ms);
```

---

## 2. 高级配置

### 2.1 性能优化配置

```rust
let config = EnhancedGpuRenderConfig {
    // 启用所有剔除策略
    enable_frustum_culling: true,
    enable_occlusion_culling: true,  // 需要深度缓冲
    enable_distance_culling: true,
    max_view_distance: 1000.0,
    distance_culling_threshold: 800.0,

    // 启用批处理和实例化
    enable_batching: true,
    enable_instancing: true,
    batch_size: 200,
    enable_multi_draw: false,  // WebGPU支持有限

    // VRAM管理
    vram_budget: 2 * 1024 * 1024 * 1024,  // 2GB
    vram_warning_threshold: 0.8,
    enable_auto_unload: true,
    resource_unload_delay: 60,  // 60帧后卸载

    ..Default::default()
};
```

### 2.2 兼容性配置

```rust
let config = EnhancedGpuRenderConfig {
    // 仅基础剔除
    enable_frustum_culling: true,
    enable_occlusion_culling: false,  // 不启用遮挡剔除
    enable_distance_culling: false,

    // 基础批处理
    enable_batching: true,
    enable_instancing: true,
    enable_multi_draw: false,

    // 较小的VRAM预算
    vram_budget: 512 * 1024 * 1024,  // 512MB

    ..Default::default()
};
```

---

## 3. 性能监控

### 3.1 实时监控

```rust
// 渲染循环中
loop {
    // 执行渲染
    let stats = manager.render(...)?;

    // 检查VRAM使用
    if manager.should_warn_vram() {
        eprintln!("Warning: High VRAM usage!");
    }

    // 打印统计
    println!("VRAM: {:.1}MB / {:.1}MB ({:.1}%)",
        stats.vram_used as f32 / (1024.0 * 1024.0),
        stats.vram_budget as f32 / (1024.0 * 1024.0),
        stats.vram_usage_ratio * 100.0
    );
}
```

### 3.2 性能分析

```rust
use std::time::Instant;

let mut frame_times = Vec::new();

loop {
    let start = Instant::now();

    // 执行渲染
    manager.render(...)?;

    let elapsed = start.elapsed();
    frame_times.push(elapsed.as_millis() as f32);

    // 每60帧打印一次统计
    if frame_times.len() >= 60 {
        let avg: f32 = frame_times.iter().sum::<f32>() / frame_times.len() as f32;
        let fps = 1000.0 / avg;

        println!("Average frame time: {:.2}ms (FPS: {:.1})", avg, fps);
        frame_times.clear();
    }
}
```

---

## 4. 常见用例

### 4.1 大型开放世界

```rust
let config = EnhancedGpuRenderConfig {
    enable_frustum_culling: true,
    enable_occlusion_culling: true,
    enable_distance_culling: true,
    max_view_distance: 2000.0,
    distance_culling_threshold: 1500.0,
    enable_batching: true,
    batch_size: 500,  // 更大的批次
    enable_instancing: true,
    vram_budget: 4 * 1024 * 1024 * 1024,  // 4GB
    ..Default::default()
};
```

### 4.2 室内场景

```rust
let config = EnhancedGpuRenderConfig {
    enable_frustum_culling: true,
    enable_occlusion_culling: true,  // 室内遮挡很重要
    enable_distance_culling: false,  // 室内不需要距离剔除
    enable_batching: true,
    batch_size: 100,
    enable_instancing: true,
    vram_budget: 1 * 1024 * 1024 * 1024,  // 1GB
    ..Default::default()
};
```

### 4.3 移动平台

```rust
let config = EnhancedGpuRenderConfig {
    enable_frustum_culling: true,
    enable_occlusion_culling: false,  // 移动GPU可能不支持
    enable_distance_culling: true,
    max_view_distance: 500.0,
    distance_culling_threshold: 400.0,
    enable_batching: true,
    batch_size: 50,  // 较小的批次
    enable_instancing: true,
    vram_budget: 256 * 1024 * 1024,  // 256MB
    ..Default::default()
};
```

---

## 5. 故障排除

### 5.1 性能问题

**问题**: 帧率低
**解决**:
1. 检查是否启用了剔除：`config.enable_frustum_culling = true`
2. 调整批次大小：`config.batch_size = 100`
3. 检查VRAM使用：`stats.vram_usage_ratio`
4. 减少实例数量

### 5.2 VRAM不足

**问题**: 内存不足警告
**解决**:
1. 降低VRAM预算：`config.vram_budget = 512 * 1024 * 1024`
2. 启用自动卸载：`config.enable_auto_unload = true`
3. 缩短卸载延迟：`config.resource_unload_delay = 30`
4. 减少资源数量

### 5.3 剔除效果差

**问题**: 可见对象少但性能没有提升
**解决**:
1. 检查AABB是否正确设置
2. 确保视锥体计算正确
3. 启用距离剔除
4. 调整视距参数

---

## 6. 示例代码

完整示例请参考：
- `game_engine/src/render/gpu_optimization_example.rs` - 演示和示例
- `game_engine/tests/gpu_manager_bench.rs` - 性能基准测试

运行示例：
```bash
cargo run --example gpu_optimization
```

运行基准测试：
```bash
cargo test --test gpu_manager_bench -- --nocapture
```

---

## 7. 下一步

1. 阅读[详细优化报告](./GPU_MANAGER_OPTIMIZATION_REPORT.md)
2. 查看[API文档](./API_DOCUMENTATION.md)
3. 尝试[示例代码](../examples/)
4. 加入社区讨论

---

## 8. 参考资料

- [GPU驱动渲染原理](https://developer.nvidia.com/gpugems/GPUGems3/gpugems3_part39.html)
- [间接绘制技术](https://www.khronos.org/opengl/wiki/Vertex_Rendering#Indirect_rendering)
- [WebGPU规范](https://www.w3.org/TR/webgpu/)

---

**最后更新**: 2026-01-02
**版本**: v2.0
