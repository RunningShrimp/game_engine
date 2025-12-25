# 全局光照系统指南

## 概述

本文档介绍游戏引擎的全局光照系统，包括VXGI（Voxel Global Illumination）实时全局光照和光照烘焙工具。

## VXGI系统

### 功能特性

- **实时全局光照**: 使用体素锥追踪实现实时间接光照
- **动态更新**: 支持动态场景的体素化更新
- **可配置质量**: 可调整体素分辨率和追踪参数
- **性能优化**: 支持静态场景优化

### 使用方法

```rust
use game_engine::render::{VxgiConfig, VxgiRenderer};

// 创建配置
let config = VxgiConfig {
    enabled: true,
    voxel_resolution: 256,
    voxel_size: 0.1,
    max_trace_distance: 10.0,
    cone_trace_steps: 8,
    indirect_intensity: 1.0,
    dynamic_update: false,
    update_frequency: 1,
};

// 创建渲染器
let mut vxgi = VxgiRenderer::new(&device, config)?;

// 体素化场景
let scene_data = serialize_scene_data(&scene);
vxgi.voxelize_scene(&device, &queue, &mut encoder, &scene_data)?;

// 执行锥追踪
vxgi.cone_trace(
    &mut encoder,
    &output_view,
    &gbuffer_position,
    &gbuffer_normal,
    &sampler,
    width,
    height,
)?;
```

### 配置选项

- `voxel_resolution`: 体素分辨率（必须是2的幂，如128、256、512）
- `voxel_size`: 每个体素的世界空间大小（米）
- `max_trace_distance`: 最大追踪距离
- `cone_trace_steps`: 锥追踪步数（影响质量和性能）
- `indirect_intensity`: 间接光照强度
- `dynamic_update`: 是否启用动态更新
- `update_frequency`: 更新频率（每N帧更新一次）

## 光照烘焙工具

### 功能特性

- **静态光照烘焙**: 为静态几何体生成光照贴图
- **环境光遮蔽**: 烘焙环境光遮蔽到光照贴图
- **间接光照**: 烘焙间接光照反弹
- **多种格式**: 支持RGBA8、RGBA16Float、RGBE格式

### 使用方法

```rust
use game_engine::render::{
    LightBaker, LightmapConfig, LightmapFormat, SceneBakingData,
};

// 创建配置
let config = LightmapConfig {
    resolution: 512,
    bake_ao: true,
    bake_indirect: true,
    indirect_bounces: 2,
    sample_count: 64,
    output_format: LightmapFormat::Rgba16Float,
};

// 创建烘焙器
let mut baker = LightBaker::new(config);

// 准备场景数据
let scene_data = SceneBakingData {
    static_meshes: vec![],
    lights: vec![],
    ambient_color: Vec3::new(0.1, 0.1, 0.1),
};

// 烘焙场景
let lightmaps = baker.bake_scene(&scene_data)?;

// 保存光照贴图
for (entity_id, _lightmap) in lightmaps {
    baker.save_lightmap(entity_id, &PathBuf::from("lightmap.png"))?;
}
```

### 烘焙流程

1. **准备场景数据**: 收集所有静态网格和光源
2. **生成UV坐标**: 为每个网格生成光照贴图UV坐标
3. **计算直接光照**: 计算光源的直接光照
4. **烘焙环境光遮蔽**: 如果启用，计算AO
5. **烘焙间接光照**: 如果启用，计算间接光照反弹
6. **保存光照贴图**: 将结果保存为图像文件

## 性能优化建议

### VXGI优化

1. **体素分辨率**: 使用256或512，更高分辨率会显著影响性能
2. **动态更新**: 静态场景禁用动态更新
3. **追踪距离**: 根据场景大小调整追踪距离
4. **追踪步数**: 6-8步通常足够，更多步数提升质量但降低性能

### 光照烘焙优化

1. **分辨率**: 根据网格大小选择合适的分辨率
2. **采样数量**: 64-128个采样通常足够
3. **间接反弹**: 2-3次反弹通常提供良好的效果
4. **批量烘焙**: 批量处理多个网格以提高效率

## 硬件要求

### VXGI

- **推荐**: GTX 1060或更高（6GB VRAM）
- **体素分辨率256**: 需要约256MB VRAM
- **体素分辨率512**: 需要约1GB VRAM

### 光照烘焙

- **CPU**: 多核CPU推荐（可并行处理）
- **内存**: 根据场景大小，可能需要几GB内存
- **存储**: 光照贴图文件可能较大（每张512x512 RGBA16Float约2MB）

## 限制和注意事项

1. **VXGI限制**:
   - 体素分辨率必须是2的幂
   - 动态更新会影响性能
   - 小细节可能丢失（取决于体素大小）

2. **光照烘焙限制**:
   - 仅适用于静态几何体
   - 烘焙时间可能较长
   - 需要额外的存储空间

## 未来计划

- [ ] 完整的体素化算法实现
- [ ] 更精确的锥追踪
- [ ] 光照贴图压缩
- [ ] 实时预览
- [ ] 渐进式烘焙

## 更多信息

- [光线追踪集成](./ray_tracing_integration.md)
- [渲染API参考](../api_reference.md)
- [性能调优指南](./performance_tuning_guide.md)

