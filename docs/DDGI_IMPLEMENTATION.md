# DDGI (Dynamic Diffuse Global Illumination) 实现文档

## 概述

DDGI (Dynamic Diffuse Global Illumination) 是一种基于探针的实时全局光照技术，通过在场景中放置探针网格来捕获和传播光照信息。

### 核心特性

- **动态更新**: 支持动态场景的全局光照更新
- **高质量光照**: 提供逼真的漫反射全局光照
- **性能优化**: 可配置更新率和探针数量
- **调试支持**: 丰富的可视化调试工具

## 架构设计

### 模块结构

```
src/render/gi/
├── mod.rs              # 模块定义
├── ddgi.rs            # DDGI核心实现
├── probe.rs           # 探针管理
├── volume.rs          # 体积配置
├── irradiance.rs      # 辐照度纹理
├── debug.rs           # 调试可视化
└── tests.rs           # 单元测试
```

### 核心组件

#### 1. DDGIVolume

DDGI体积是核心数据结构，管理一个探针网格：

```rust
pub struct DDGIVolume {
    probes: Vec<DDGIProbe>,           // 探针列表
    probe_spacing: f32,                // 探针间距
    probe_counts: UVec3,               // 探针数量
    irradiance_texture: IrradianceTexture,  // 辐照度纹理
    depth_texture: Texture,            // 深度纹理
    offset_texture: Texture,           // 偏移纹理
    config: DDGIConfig,                // 配置
}
```

#### 2. DDGIProbe

单个探针存储位置和光照信息：

```rust
pub struct DDGIProbe {
    pub position: Vec3,     // 世界空间位置
    pub irradiance: Vec3,   // 辐照度（RGB）
    pub depth: f32,         // 深度值
    pub offset: Vec2,       // 采样偏移
}
```

#### 3. ProbeManager

管理多个DDGI体积：

```rust
pub struct ProbeManager {
    volumes: Vec<DDGIVolume>,
    active_volume: Option<usize>,
}
```

## 算法说明

### 1. 探针网格生成

在场景中创建规则3D网格的探针：

```
探针布局（俯视图）：
+---+---+---+
| P | P | P |
+---+---+---+
| P | P | P |
+---+---+---+
| P | P | P |
+---+---+---+
```

每个探针的位置：
```rust
let pos = Vec3::new(
    x as f32 * probe_spacing,
    y as f32 * probe_spacing,
    z as f32 * probe_spacing,
);
```

### 2. 探针渲染

每个探针渲染6个方向的深度和法线：

```
      +Y
       |
  -X---+---+X
       |
      -Y

(每个探针类似立方体贴图)
```

### 3. 辐照度更新

从深度纹理计算辐照度：

1. 读取6个方向的深度和法线
2. 使用球谐函数存储辐照度
3. 应用时序滤波减少闪烁

### 4. 光照传播

探针间光照传播：

1. 邻居探针查找
2. 加权平均
3. 迭代扩散

### 5. 光照采样

三线性插值采样：

```
对于任意点P：
1. 找到8个邻居探针
2. 计算插值权重
3. 加权采样辐照度
```

着色器实现：
```wgsl
fn sample_irradiance_trilinear(world_pos: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let grid_index = get_probe_index(world_pos);
    let local_pos = (world_pos - ddgi.volume_origin) / ddgi.probe_spacing;
    let fract_pos = fract(local_pos);

    let mut result = vec3<f32>(0.0);
    var weight_sum = 0.0;

    // 8个邻居探针
    for (var dz: u32 = 0u; dz < 2u; dz++) {
        for (var dy: u32 = 0u; dy < 2u; dy++) {
            for (var dx: u32 = 0u; dx < 2u; dx++) {
                // ... 采样和加权
            }
        }
    }

    return result / weight_sum;
}
```

## 性能特性

### 内存占用

内存计算公式：
```
总内存 = 探针数量 × 6面 × 分辨率² × 纹理格式大小

例如（中等质量）：
- 探针数量：1000 (10×10×10)
- 辐照度分辨率：16×16
- 格式：RGBA32Float (16字节/像素)
- 内存：1000 × 6 × 256 × 16 = 24.5 MB
```

### 性能优化

1. **可配置更新率**
   - 低质量：每6帧更新一次
   - 中等质量：每3帧更新一次
   - 高质量：每帧更新

2. **探针数量优化**
   - 小场景：125-1000个探针
   - 中等场景：1000-8000个探针
   - 大场景：8000+个探针（使用多个体积）

3. **重要性采样**（计划中）
   - 优先更新相机附近的探针
   - 基于屏幕空间重要性排序

### 性能基准

（待实测）

| 配置 | 探针数 | 更新率 | 预期帧率 |
|------|--------|--------|----------|
| Low | 125 | 每6帧 | 60+ FPS |
| Medium | 1000 | 每3帧 | 60 FPS |
| High | 8000 | 每帧 | 30-60 FPS |

## 配置选项

### 质量预设

#### 低质量（Low）
```rust
let config = DDGIConfig {
    probe_spacing: 4.0,
    probe_counts: UVec3::new(5, 5, 5),  // 125个探针
    irradiance_resolution: 8,
    depth_resolution: 8,
    update_rate: 6,
    ..Default::default()
};
```

#### 中等质量（Medium）
```rust
let config = DDGIConfig {
    probe_spacing: 2.0,
    probe_counts: UVec3::new(10, 10, 10),  // 1000个探针
    irradiance_resolution: 16,
    depth_resolution: 16,
    update_rate: 3,
    ..Default::default()
};
```

#### 高质量（High）
```rust
let config = DDGIConfig {
    probe_spacing: 1.0,
    probe_counts: UVec3::new(20, 20, 20),  // 8000个探针
    irradiance_resolution: 32,
    depth_resolution: 32,
    update_rate: 1,
    ..Default::default()
};
```

### 参数说明

| 参数 | 说明 | 推荐范围 |
|------|------|----------|
| `probe_spacing` | 探针间距（世界单位） | 0.5 - 4.0 |
| `probe_counts` | 探针数量（X,Y,Z） | 根据场景大小 |
| `irradiance_resolution` | 辐照度纹理分辨率 | 8 - 32 |
| `depth_resolution` | 深度纹理分辨率 | 8 - 32 |
| `max_depth` | 最大深度 | 50 - 100 |
| `normal_bias` | 法线偏移（防止光泄漏） | 0.01 - 0.1 |
| `update_rate` | 更新率（帧数） | 1 - 6 |

## 使用说明

### 基本使用

```rust
use game_engine::render::gi::{DDGIVolume, DDGIConfig, ProbeManager};

// 1. 创建配置
let config = DDGIConfig::medium_quality();

// 2. 创建DDGI体积
let volume = DDGIVolume::new(&device, &config)?;

// 3. 添加到管理器
let mut manager = ProbeManager::new();
manager.add_volume(volume);

// 4. 每帧更新
manager.update(&device, &queue, &mut encoder)?;
```

### 调试可视化

```rust
use game_engine::render::gi::{GIDebugVisualizer, ProbeVisualization};

let mut debug = GIDebugVisualizer::new();
debug.initialize(&device);

// 设置可视化模式
debug.set_probe_visualization(ProbeVisualization::Spheres);
debug.set_show_probes(true);

// 渲染调试信息
debug.render(&mut encoder, &volume);

// 获取统计信息
let stats = debug.get_probe_stats(&volume);
println!("Active probes: {}", stats.active_probes);
println!("Average irradiance: {:?}", stats.avg_irradiance);
```

### 采样全局光照

```rust
// 在着色器中采样
let world_pos = input.world_position;
let normal = input.world_normal;

// 使用DDGI计算全局光照
let gi = sample_ddgi(world_pos, normal);

// 与直接光照混合
let final_color = direct_lighting + gi * albedo;
```

## 调试工具

### 可视化模式

1. **None**: 不显示探针
2. **Spheres**: 显示探针为球体
3. **Lines**: 显示探针连接
4. **Heatmap**: 辐照度热力图（蓝-绿-红）
5. **Irradiance**: 使用辐照度颜色
6. **Depth**: 显示深度信息

### 统计信息

```rust
pub struct ProbeStats {
    pub total_probes: usize,      // 总探针数
    pub active_probes: usize,     // 活跃探针数
    pub avg_irradiance: Vec3,     // 平均辐照度
    pub min_depth: f32,           // 最小深度
    pub max_depth: f32,           // 最大深度
}
```

## 故障排除

### 常见问题

#### 1. 光泄漏（Light Bleeding）

**症状**: 光线穿过墙壁照亮相邻区域

**解决方案**:
- 增加 `normal_bias` 参数
- 减小探针间距
- 提高深度纹理分辨率

#### 2. 闪烁（Flickering）

**症状**: 光照不稳定，帧间闪烁

**解决方案**:
- 启用时序滤波 `enable_temporal_filter = true`
- 增加滤波强度 `temporal_filter_alpha = 0.9`
- 减少更新频率

#### 3. 性能问题

**症状**: 帧率过低

**解决方案**:
- 减少探针数量
- 增加更新率间隔
- 降低纹理分辨率
- 使用低质量预设

#### 4. 伪影（Artifacts）

**症状**: 光照不连续或有黑斑

**解决方案**:
- 检查探针网格是否覆盖场景
- 调整探针间距
- 增加光照传播迭代次数

### 调试技巧

1. **启用探针可视化**
   ```rust
   debug.set_probe_visualization(ProbeVisualization::Spheres);
   debug.set_show_probes(true);
   ```

2. **检查辐照度值**
   ```rust
   for probe in &volume.probes {
       println!("Probe at {:?}: irradiance={:?}",
                probe.position, probe.irradiance);
   }
   ```

3. **使用热力图**
   ```rust
   debug.set_probe_visualization(ProbeVisualization::Heatmap);
   ```

4. **监控性能**
   ```rust
   let stats = debug.get_probe_stats(&volume);
   println!("Memory usage: {} MB", config.memory_usage() / (1024 * 1024));
   ```

## 限制和已知问题

### 当前限制

1. **探针数量受限**
   - 过多探针会影响性能
   - 建议：单个体积不超过8000个探针

2. **动态场景开销大**
   - 每次更新需要渲染所有探针
   - 解决方案：降低更新率

3. **硬表面支持有限**
   - 当前主要支持漫反射
   - 镜面反射需要单独处理

4. **光泄漏问题**
   - 需要仔细调整normal_bias
   - 复杂几何体可能仍有问题

### 已知问题

- [ ] 时序滤波实现简化
- [ ] 光照传播算法未优化
- [ ] 自适应更新未实现
- [ ] 镜面反射探针缺失

## 未来改进计划

### 短期目标（v0.3.0）

- [ ] 完善时序滤波
- [ ] 优化光照传播算法
- [ ] 添加更多调试选项
- [ ] 性能分析和优化

### 中期目标（v0.4.0）

- [ ] 实现自适应探针更新
- [ ] 添加重要性采样
- [ ] 支持多个DDGI体积
- [ ] 级联DDGI

### 长期目标（v0.5.0+）

- [ ] 镜面反射探针
- [ ] 各向异性反射
- [ ] 与VXGI混合
- [ ] AI辅助优化

## 参考资源

### 论文和文章

1. "Dynamic Diffuse Global Illumination with Ray-Traced Irradiance Probes" - Zeltner et al.
2. "Scaling Probe-Based Real-Time Global Illumination" - Kontkanen et al.
3. "Real-Time Global Illumination using Precomputed Light Field Probes" - Xu et al.

### 开源实现

1. NVIDIA's VXGI (Voxel Global Illumination)
2. UE5 Lumen (Software Ray Tracing)
3. Unity HDRP Global Illumination

## 总结

DDGI提供了一种平衡质量和性能的实时全局光照解决方案。通过合理配置和优化，可以在现代GPU上实现高质量的全局光照效果。

### 优势

- 高质量光照
- 实时性能
- 灵活配置
- 易于调试

### 劣势

- 内存占用较高
- 动态场景开销大
- 光泄漏问题
- 配置复杂

### 适用场景

- 室内场景（建筑可视化）
- 中等规模游戏场景
- 需要高质量光照的应用
- 可以接受一定性能开销的项目

## 版本历史

- **v0.2.0** (2025-12-31): 初始实现
  - 基础DDGI框架
  - 探针网格生成
  - 辐照度纹理管理
  - 调试可视化

- **v0.3.0** (计划): 性能优化
  - 时序滤波完善
  - 自适应更新
  - 传播算法优化

- **v0.4.0** (计划): 高级功能
  - 镜面反射探针
  - 级联DDGI
  - 多体积支持

---

**文档版本**: v0.2.0
**最后更新**: 2025-12-31
**维护者**: Game Engine Team
