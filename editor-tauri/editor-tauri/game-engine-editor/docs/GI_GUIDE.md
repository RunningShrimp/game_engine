# 全局光照系统快速指南

## 快速开始

### 1. 创建GI系统

```rust
use game_engine::render::gi::{GISystem, GIConfig, GITechnique, GIQuality};

let config = GIConfig {
    enabled_techniques: GITechnique {
        ssr: true,        // 屏幕空间反射
        ssgi: true,       // 屏幕空间GI
        ssdo: true,       // 屏幕空间AO
        light_probes: true,
        hybrid: true,
        ..Default::default()
    },
    quality: GIQuality::High,
    target_fps: 60.0,
    ..Default::default()
};

let mut gi_system = GISystem::new(device, queue, config)?;
```

### 2. 每帧更新

```rust
// 游戏循环
loop {
    gi_system.update(delta_time);

    gi_system.render(
        &output_view,
        &depth_view,
        &normal_view,
        view_matrix,
        proj_matrix,
    )?;
}
```

### 3. 性能监控

```rust
let stats = gi_system.get_stats();

if stats.hybrid_stats.current_fps < 30.0 {
    gi_system.adjust_quality(GIQuality::Medium);
}
```

## 技术选择

### 屏幕空间技术 (推荐入门)

**适合**: 移动设备、中低端PC

```rust
let config = GIConfig {
    enabled_techniques: GITechnique {
        ssr: true,
        ssgi: true,
        ssdo: true,
        ..Default::default()
    },
    quality: GIQuality::Medium,
    ..Default::default()
};
```

**性能**: >60 FPS
**质量**: 中等

### 混合渲染 (推荐)

**适合**: PC、主机

```rust
let config = GIConfig {
    enabled_techniques: GITechnique {
        hybrid: true,
        light_probes: true,
        ..Default::default()
    },
    quality: GIQuality::High,
    ..Default::default()
};
```

**性能**: >45 FPS
**质量**: 高

### 全光线追踪 (高端)

**适合**: RTX GPU、高端PC

```rust
let config = GIConfig {
    enabled_techniques: GITechnique {
        ray_traced_reflection: true,
        ray_traced_gi: true,
        ray_traced_ao: true,
        ray_traced_shadows: true,
        ..Default::default()
    },
    quality: GIQuality::Ultra,
    ..Default::default()
};
```

**性能**: >30 FPS
**质量**: 超高

## 质量设置

| 质量 | 光线追踪样本 | 屏幕空间迭代 | 探针分辨率 | 适用场景 |
|------|------------|------------|----------|---------|
| Low  | 1          | 8          | 3        | 移动设备 |
| Medium | 2        | 16         | 6        | 中端PC |
| High   | 4        | 32         | 9        | 高端PC |
| Ultra  | 8        | 64         | 12       | RTX GPU |

## 调整质量

```rust
// 手动调整
gi_system.adjust_quality(GIQuality::Medium);

// 自适应调整（推荐）
config.hybrid.adaptive_quality = true;
```

## 光照探针

### 放置探针

```rust
let bounds = BoundingBox {
    min: Vec3::new(-10.0, 0.0, -10.0),
    max: Vec3::new(10.0, 5.0, 10.0),
};

gi_system.rebuild_probes(bounds)?;
```

### 烘焙光照

```rust
let scene = Scene::new();
gi_system.bake_lighting(&scene)?;
```

## 性能优化

### 启用优化

```rust
config.screen_space.optimization = OptimizationConfig {
    use_depth_pyramid: true,      // 深度金字塔
    use_early_exit: true,          // 早期退出
    use_spatial_reuse: false,      // 空间复用
    pyramid_levels: 5,
};
```

### 缓存配置

```rust
let cache = GICache::new(device, 512 * 1024 * 1024)?; // 512MB
```

## 故障排除

### FPS太低

```rust
// 降低质量
gi_system.adjust_quality(GIQuality::Low);

// 或减少光线追踪比例
config.hybrid.ray_tracing_ratio = 0.3;
```

### 噪点问题

```rust
// 启用去噪
config.ray_tracing.denoising.enabled = true;

// 增加样本
config.ray_tracing.samples_per_pixel = 4;

// 启用时域累积
config.ray_tracing.denoising.temporal_accumulation = true;
```

### 内存问题

```rust
// 减少缓存
let cache = GICache::new(device, 256 * 1024 * 1024)?;

// 或减少探针
config.light_probes.grid_resolution = 6;
```

## 示例代码

完整示例见 `game_engine/examples/gi_example.rs`。

## API参考

详细API文档见 `docs/ADVANCED_GI_IMPLEMENTATION.md`。
