# Nanite虚拟几何体系统使用指南

## 快速开始

### 1. 初始化系统

```rust
use game_engine::render::nanite::*;

// 创建配置
let config = NaniteConfig {
    max_triangles_per_cluster: 128,
    max_lod_depth: 8,
    target_screen_space_error: 1.0,
    enable_occlusion_culling: true,
    enable_compute_acceleration: true,
    ..Default::default()
};

// 创建Nanite系统
let mut nanite_system = NaniteSystem::new(&device, config)?;
```

### 2. 加载网格

```rust
// 准备顶点和索引数据
let vertices: Vec<Vec3> = vec![
    /* 顶点数据 */
];
let indices: Vec<u32> = vec![
    /* 索引数据 */
];

// 注册到Nanite系统
let mesh_id = nanite_system.register_mesh(&device, &vertices, &indices)?;
println!("Mesh registered with ID: {}", mesh_id);
```

### 3. 渲染循环

```rust
// 游戏循环
loop {
    let delta_time = frame_timer.elapsed();
    frame_timer.reset();

    // 更新相机
    update_camera(&mut camera);

    // 更新Nanite系统
    let stats = nanite_system.update(
        &device,
        &queue,
        &camera,
        delta_time.as_secs_f32(),
    )?;

    // 渲染
    render_frame(&device, &queue, &nanite_system)?;

    // 显示统计信息
    if show_stats {
        println!("Visible: {} clusters, {} tris, {:.2}ms",
            stats.visible_clusters,
            stats.visible_triangles,
            stats.frame_time_ms
        );
    }
}
```

## 高级用法

### 自定义聚类配置

```rust
let cluster_config = ClusterConfig {
    max_triangles_per_cluster: 256,  // 更大的Cluster
    max_depth: 12,                    // 更深的层次
    error_threshold: 0.005,           // 更低的误差
    preserve_hard_edges: true,
    hard_edge_angle: std::f32::consts::PI / 6.0,
};

let mut builder = ClusterBuilder::new(cluster_config);
let hierarchy = builder.build_hierarchy(&vertices, &indices)?;
```

### 质量控制

#### 手动设置质量

```rust
let quality_controller = nanite_system.quality_controller();

// 设置目标质量
quality_controller.set_target_quality(1.5); // 150%质量

// 强制特定质量
quality_controller.force_quality(0.8); // 80%质量

// 重置为默认
quality_controller.reset_quality();
```

#### 使用质量预设

```rust
use QualityPreset;

let presets = [
    QualityPreset::Ultra,    // 最高质量，30 FPS目标
    QualityPreset::High,     // 高质量，60 FPS目标
    QualityPreset::Medium,   // 中等质量，60 FPS目标
    QualityPreset::Low,      // 低质量，90 FPS目标
    QualityPreset::Potato,   // 最低质量，120 FPS目标
];

let preset = QualityPreset::High;
let quality = preset.quality_multiplier();
let target_fps = preset.target_fps();
```

#### 监控性能

```rust
let stats = quality_controller.stats();

println!("FPS: {:.1}", stats.fps);
println!("Frame Time: {:.2}ms", stats.frame_time_ms);
println!("Average FPS: {:.1}", stats.average_fps);
println!("1% Low: {:.1}", stats.fps_1_percent_low);
println!("0.1% Low: {:.1}", stats.fps_0_1_percent_low);
```

### LOD选择优化

```rust
let lod_config = LODConfig {
    max_lod_depth: 8,
    target_screen_space_error: 1.0,
    distance_factor: 1.0,
    smooth_transitions: true,    // 平滑LOD过渡
    transition_speed: 0.1,       // 过渡速度
};

let mut lod_manager = LODManager::new(lod_config)?;
```

### 剔除系统配置

```rust
let culling_config = CullingConfig {
    enable_occlusion_culling: true,
    min_cluster_size: 4,                    // 最小剔除尺寸
    hiz_buffer_scale: 0.25,                 // Hi-Z分辨率比例
    occlusion_query_delay: 2,               // 查询延迟帧数
};

let mut culling_system = CullingSystem::new(culling_config)?;
```

### 缓冲管理

```rust
let buffer_config = BufferConfig {
    instance_buffer_size_mb: 512,           // 512MB实例缓冲
    enable_compute_acceleration: true,
    buffer_alignment: 256,
    enable_defragmentation: true,           // 启用碎片整理
};

let buffer_manager = BufferManager::new(&device, buffer_config)?;

// 检查内存使用
println!("GPU Memory: {:.2} MB", buffer_manager.memory_usage_mb());
println!("Buffer Usage: {:.1}%", buffer_manager.instance_buffer_usage());

// 碎片整理
buffer_manager.defragment(&device, &queue)?;
```

## 实际应用场景

### 场景1：大世界渲染

```rust
// 创建多个地形块
for chunk_x in 0..10 {
    for chunk_z in 0..10 {
        let (vertices, indices) = generate_terrain_chunk(chunk_x, chunk_z);
        let mesh_id = nanite_system.register_mesh(&device, &vertices, &indices)?;
        terrain_chunks.push((chunk_x, chunk_z, mesh_id));
    }
}

// 只渲染可见块
for &(chunk_x, chunk_z, mesh_id) in &terrain_chunks {
    if is_chunk_visible(chunk_x, chunk_z, &camera) {
        let hierarchy = nanite_system.hierarchy(mesh_id)?;
        // 渲染...
    }
}
```

### 场景2：高质量角色

```rust
// 加载高精度角色模型（1M+ 三角形）
let (hero_vertices, hero_indices) = load_high_poly_hero();
let hero_mesh_id = nanite_system.register_mesh(&device, &hero_vertices, &hero_indices)?;

// 根据距离自动LOD
// 近距离：全细节
// 中距离：自动简化
// 远距离：极低LOD
```

### 场景3：粒子系统

```rust
// 为每个粒子实例注册一次网格
let particle_mesh_id = nanite_system.register_mesh(&device, &particle_verts, &particle_inds)?;

// 实例化渲染数千个粒子
let particle_instances: Vec<InstanceData> = particles.iter()
    .map(|p| InstanceData {
        model_matrix_0: [p.transform[0][0], p.transform[0][1], p.transform[0][2], p.transform[0][3]],
        // ... 填充矩阵
        ..Default::default()
    })
    .collect();

// 批量更新实例
buffer_manager.update_instances(&device, &queue, &particle_instances)?;
```

## 性能调优

### 1. 聚类参数调优

```rust
// 高质量模式
let config = NaniteConfig {
    max_triangles_per_cluster: 64,   // 更小的Cluster = 更精细的LOD
    max_lod_depth: 12,                // 更深的LOD层次
    target_screen_space_error: 0.5,   // 更严格的SSE
    ..Default::default()
};

// 性能模式
let config = NaniteConfig {
    max_triangles_per_cluster: 256,  // 更大的Cluster = 更少draw call
    max_lod_depth: 6,                 // 更浅的LOD层次
    target_screen_space_error: 2.0,   // 更宽松的SSE
    ..Default::default()
};
```

### 2. 内存优化

```rust
// 限制缓冲区大小
let config = NaniteConfig {
    instance_buffer_size_mb: 128,     // 减少内存使用
    ..Default::default()
};

// 定期碎片整理
if frame_count % 300 == 0 {  // 每5秒
    nanite_system.buffer_manager().defragment(&device, &queue)?;
}
```

### 3. CPU/GPU平衡

```rust
// CPU受限：启用GPU加速
let config = NaniteConfig {
    enable_compute_acceleration: true,  // 使用Compute Shader
    ..Default::default()
};

// GPU受限：减少精度
let config = NaniteConfig {
    target_screen_space_error: 2.0,     // 提高误差阈值
    enable_occlusion_culling: false,    // 禁用遮挡剔除
    ..Default::default()
};
```

## 调试和诊断

### 启用调试模式

```rust
// 获取详细统计信息
let stats = nanite_system.update(/* ... */)?;

println!("=== Nanite Statistics ===");
println!("Visible Clusters: {}", stats.visible_clusters);
println!("Visible Triangles: {}", stats.visible_triangles);
println!("Culled Clusters: {}", stats.culled_clusters);
println!("Average LOD: {:.2}", stats.average_lod);
println!("Frame Time: {:.2}ms", stats.frame_time_ms);
println!("GPU Memory: {:.2}MB", stats.gpu_memory_mb);
```

### 可视化调试

```rust
// 绘制Cluster边界（调试模式）
if debug_mode {
    for node in &hierarchy.nodes {
        draw_bounding_sphere(&node.cluster.sphere_center, node.cluster.sphere_radius);
    }
}

// 可视化LOD级别
if show_lod {
    for selection in &lod_selections {
        let color = lod_color(selection.lod_level);
        draw_cluster_with_color(selection.cluster_id, color);
    }
}
```

### 性能分析

```rust
use std::time::Instant;

let start = Instant::now();

// 执行操作
nanite_system.update(/* ... */)?;

let duration = start.elapsed();
println!("Update took: {:?}", duration);

// 分段计时
let clustering_start = Instant::now();
// ... 聚类操作
println!("Clustering: {:.2}ms", clustering_start.elapsed().as_micros() as f32 / 1000.0);

let culling_start = Instant::now();
// ... 剔除操作
println!("Culling: {:.2}ms", culling_start.elapsed().as_micros() as f32 / 1000.0);
```

## 最佳实践

### DO（推荐）

1. **批量加载**：在启动时预加载所有静态网格
2. **使用质量预设**：根据目标平台选择合适的预设
3. **监控性能**：定期检查stats和GPU内存使用
4. **合理设置SSE阈值**：根据分辨率和视野调整
5. **启用剔除**：始终启用视锥剔除，考虑遮挡剔除

### DON'T（不推荐）

1. **每帧创建新Cluster**：避免频繁重新聚类
2. **过高的LOD深度**：超过12层的LOD很少需要
3. **忽略内存限制**：注意GPU内存使用，特别是移动平台
4. **禁用所有优化**：为了调试禁用可以，但要记得启用
5. **盲目追求质量**：根据实际需求平衡质量和性能

## 故障排除

### 问题1：帧率下降

```rust
// 检查性能统计
let stats = quality_controller.stats();

if stats.fps < 30.0 {
    // 降低质量
    quality_controller.set_target_quality(0.7);
}

// 检查LOD级别
if stats.average_lod > 4.0 {
    // Cluster太多或太小，增加max_triangles_per_cluster
}
```

### 问题2：内存不足

```rust
let buffer_manager = nanite_system.buffer_manager();

if buffer_manager.memory_usage_mb() > 2048.0 {
    // 整理碎片
    buffer_manager.defragment(&device, &queue)?;

    // 或减少缓冲区大小
    let new_config = BufferConfig {
        instance_buffer_size_mb: 128,
        ..Default::default()
    };
}
```

### 问题3：视觉质量差

```rust
// 提高SSE精度
let config = nanite_system.config();
config.target_screen_space_error = 0.5; // 从1.0降到0.5

// 禁用质量自适应
quality_controller.force_quality(2.0); // 最高质量

// 检查LOD是否正确选择
println!("Average LOD: {:.2}", stats.average_lod);
```

## 示例项目

完整示例请参考：
- `game_engine/examples/nanite_example.rs` - 基础使用示例
- `benches/nanite_bench.rs` - 性能基准测试

## API参考

详细API文档请查看各模块的文档注释：
- `clustering.rs` - 聚类算法
- `lod_manager.rs` - LOD管理
- `culling.rs` - 剔除系统
- `renderer.rs` - 渲染器
- `buffer.rs` - 缓冲管理
- `metrics.rs` - 质量指标

## 常见问题

**Q: Nanite适合所有场景吗？**

A: 不是。对于非常简单的网格（<1K三角形），传统渲染可能更快。Nanite最适合高多边形模型（>100K三角形）。

**Q: 可以用于动画网格吗？**

A: 当前版本不支持蒙皮网格的Nanite渲染。这是未来的改进方向。

**Q: 移动平台支持如何？**

A: 理论上支持WebGPU的平台都支持Nanite，但移动GPU可能需要降低质量设置。

**Q: 与传统渲染器混合使用？**

A: 可以。部分网格使用Nanite，部分使用传统渲染器。

---

*文档版本：1.0*
*最后更新：2025-01-02*
