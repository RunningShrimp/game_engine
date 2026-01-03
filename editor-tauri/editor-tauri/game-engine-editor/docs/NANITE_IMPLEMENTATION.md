# Nanite虚拟几何体系统实现文档

## 概述

本文档描述了游戏引擎中Nanite式虚拟几何体系统的实现。Nanite是Unreal Engine 5引入的革命性渲染技术，能够实时渲染具有数百万三角形的高质量模型。

## 核心概念

### 1. 虚拟几何体 (Virtual Geometry)

传统渲染方法需要将整个网格加载到GPU内存中，而Nanite使用虚拟化方法：

- **分层聚类**：将网格分解为多层次的小三角形集群(Cluster)
- **按需渲染**：只渲染当前帧所需的LOD级别
- **GPU驱动**：使用Compute Shader进行LOD选择和剔除

### 2. Cluster（集群）

Cluster是Nanite的基本渲染单元：

- 包含64-256个三角形
- 具有包围球和AABB包围盒
- 存储多个LOD级别的三角形数据
- 支持父子层次关系

## 架构设计

### 模块结构

```
game_engine/src/render/nanite/
├── mod.rs              # 主模块导出
├── clustering.rs       # 网格聚类算法
├── lod_manager.rs      # LOD管理
├── culling.rs          # 剔除系统
├── renderer.rs         # 渲染器
├── buffer.rs           # 缓冲管理
└── metrics.rs          # 质量指标
```

### 核心组件

#### 1. ClusterHierarchy（聚类层次）

```rust
pub struct ClusterHierarchy {
    pub nodes: Vec<ClusterNode>,
    pub root_id: u32,
    pub total_triangles: usize,
    pub max_depth: u8,
    pub mesh_bounds: (Vec3, Vec3),
}
```

**功能**：
- 存储完整的聚类树结构
- 支持快速遍历和查询
- 管理网格的全局边界

**构建过程**：
1. 将输入网格转换为三角形列表
2. 递归地划分三角形到Cluster
3. 计算每个Cluster的包围体
4. 生成LOD级别（简化三角形）

#### 2. LODManager（LOD管理器）

```rust
pub struct LODManager {
    config: LODConfig,
    lod_cache: HashMap<u32, Vec<LODLevel>>,
    previous_selections: HashMap<u32, LODSelection>,
}
```

**LOD选择策略**：

基于屏幕空间误差(SSE)选择LOD：

```
SSE = (geometric_error × projection_scale) / distance
```

- SSE < 阈值：使用最高质量LOD
- SSE > 阈值：使用较低质量LOD
- 应用平滑过渡避免突变

**自适应LOD**：

```rust
fn select_lod_level(
    cluster: &Cluster,
    distance: f32,
    screen_space_error: f32,
    quality_metrics: &QualityMetrics,
) -> u8
```

#### 3. CullingSystem（剔除系统）

```rust
pub struct CullingSystem {
    config: CullingConfig,
    occlusion_culling: Option<OcclusionCulling>,
    stats: CullingStats,
}
```

**剔除流程**：

1. **视锥剔除** (Frustum Culling)
   - 提取6个视锥平面
   - 测试Cluster包围球
   - 快速拒绝视野外对象

2. **遮挡剔除** (Occlusion Culling)
   - 使用Hi-Z深度缓冲
   - 异步遮挡查询
   - 保守估计避免过度剔除

**视锥平面提取**：

```rust
fn extract_frustum_planes(mvp: &[[f32; 4]; 4]) -> Vec<[f32; 4]>
```

#### 4. NaniteRenderer（渲染器）

```rust
pub struct NaniteRenderer {
    config: RenderConfig,
    render_pipeline: Option<RenderPipeline>,
    compute_pipeline: Option<ComputePipeline>,
    bind_group_layouts: Vec<BindGroupLayout>,
    pipeline_layout: Option<PipelineLayout>,
    uniform_buffers: HashMap<String, Buffer>,
}
```

**渲染管线**：

1. **Compute Pass** (可选)
   - GPU驱动的LOD选择
   - 并行剔除
   - 生成实例数据

2. **Render Pass**
   - 绘制可见Cluster
   - 使用实例化渲染
   - 应用材质和光照

**着色器结构**：

```wgsl
// Vertex Shader
@vertex
fn vertex_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput

// Fragment Shader
@fragment
fn fragment_main(world_position: vec3<f32>, normal: vec3<f32>) -> vec4<f32>

// Compute Shader (可选)
@compute @workgroup_size(64)
fn cull_main(@builtin(global_invocation_id) global_id: vec3<u32>)
```

#### 5. BufferManager（缓冲管理器）

```rust
pub struct BufferManager {
    config: BufferConfig,
    instance_buffers: Vec<GPUBuffer>,
    cluster_buffers: Vec<GPUBuffer>,
    instance_cache: HashMap<u32, InstanceData>,
    total_allocated: u64,
}
```

**内存管理策略**：

- **池分配**：预分配大块内存
- **子分配**：在池中管理小块
- **碎片整理**：定期合并空闲区域
- **动态扩展**：按需创建新缓冲区

**实例数据结构**：

```rust
#[repr(C)]
pub struct InstanceData {
    pub model_matrix_0: [f32; 4],
    pub model_matrix_1: [f32; 4],
    pub model_matrix_2: [f32; 4],
    pub model_matrix_3: [f32; 4],
    pub lod_level: u32,
    pub cluster_id: u32,
    pub padding: [u32; 2],
}
```

#### 6. QualityController（质量控制器）

```rust
pub struct QualityController {
    config: MetricsConfig,
    current_quality: f32,
    target_quality: f32,
    frame_times: Vec<f32>,
    stats: PerformanceStats,
}
```

**自适应质量控制**：

```rust
fn adjust_quality(&mut self) -> Result<(), QualityError> {
    let avg_frame_time = /* ... */;

    if avg_frame_time > target * 1.2 {
        // 降低质量
        target_quality *= (1.0 - adjustment_speed);
    } else if avg_frame_time < target * 0.9 {
        // 提高质量
        target_quality *= (1.0 + adjustment_speed);
    }
}
```

**质量预设**：

- **Ultra**: 质量2.0x, 目标30 FPS
- **High**: 质量1.5x, 目标60 FPS
- **Medium**: 质量1.0x, 目标60 FPS
- **Low**: 质量0.75x, 目标90 FPS
- **Potato**: 质量0.5x, 目标120 FPS

## 性能优化

### 1. 并行化

- **聚类构建**：多线程并行处理
- **LOD选择**：SIMD加速距离计算
- **剔除**：GPU Compute Shader并行测试
- **渲染**：实例化批量绘制

### 2. 内存优化

- **紧凑存储**：使用u16/i16代替u32/i32
- **数据对齐**：优化缓存行利用率
- **内存池**：减少分配开销
- **压缩**：LOD三角形简化

### 3. GPU优化

- **间接渲染**：GPU驱动绘制调用
- **Compute Shader**：减少CPU-GPU同步
- **异步查询**：遮挡查询不阻塞渲染
- **Hi-Z缓冲**：快速深度测试

### 4. 剔除优化

- **层次化剔除**：先剔除父节点
- **保守测试**：包围球vs平面
- **早期拒绝**：距离预过滤
- **帧一致性**：缓存上一帧结果

## 使用示例

### 基本使用

```rust
// 创建Nanite系统
let nanite_config = NaniteConfig::default();
let mut nanite_system = NaniteSystem::new(&device, nanite_config)?;

// 注册网格
let mesh_id = nanite_system.register_mesh(&device, &vertices, &indices)?;

// 每帧更新
let stats = nanite_system.update(&device, &queue, &camera, delta_time)?;

// 渲染
renderer.render(&mut ctx, &hierarchies, &lod_selections)?;
```

### 质量控制

```rust
// 设置质量预设
quality_controller.set_target_quality(1.5); // 高质量

// 或者强制质量
quality_controller.force_quality(0.5); // 最低质量

// 获取性能统计
let stats = quality_controller.stats();
println!("FPS: {:.1}, Frame Time: {:.2}ms", stats.fps, stats.frame_time_ms);
```

### 自定义配置

```rust
let config = NaniteConfig {
    max_triangles_per_cluster: 128,
    max_lod_depth: 8,
    target_screen_space_error: 1.0,
    enable_occlusion_culling: true,
    enable_compute_acceleration: true,
    ..Default::default()
};
```

## 性能基准

### 测试场景

- **小网格**：1K三角形
- **中网格**：100K三角形
- **大网格**：1M+三角形

### 性能指标

| 指标 | 目标 | 实测 |
|------|------|------|
| 聚类构建时间 | <100ms (100K tris) | TBD |
| LOD选择时间 | <5ms (1000 clusters) | TBD |
| 剔除时间 | <2ms | TBD |
| 渲染帧率 | >60 FPS | TBD |
| GPU内存 | 合理使用 | TBD |

### 优化空间

- [ ] 实现完整的Compute Shader加速
- [ ] 优化遮挡剔除精度
- [ ] 添加流式加载支持
- [ ] 实现更激进的LOD简化
- [ ] 支持实例化渲染优化

## 限制和已知问题

### 当前限制

1. **简化算法**：使用基础面积排序，可用更先进的算法
2. **遮挡剔除**：Hi-Z缓冲尚未完整实现
3. **Compute Shader**：仅框架代码，需要完整实现
4. **流式加载**：未实现动态加载/卸载

### 已知问题

1. 内存使用可能较高
2. 极高分辨率下SSE计算需要优化
3. 某些边缘情况下LOD可能闪烁

## 未来改进

### 短期（P3）

- [ ] 完善Compute Shader实现
- [ ] 优化内存布局
- [ ] 添加更多单元测试
- [ ] 性能分析和优化

### 中期

- [ ] 流式加载支持
- [ ] 材质系统集成
- [ ] 阴影渲染支持
- [ ] 多线程优化

### 长期

- [ ] 支持动画网格
- [ ] 程序化生成LOD
- [ ] 机器学习LOD选择
- [ ] 跨平台优化

## 参考资料

1. **Unreal Engine 5 Nanite Paper**
   - Brian Karis, Epic Games
   - SIGGRAPH 2021

2. **Virtual Geometry Textures**
   - NVIDIA Research

3. **Screen Space Error Metrics**
   - Real-Time Rendering 4th Edition

## 贡献者

- Claude (AI Assistant)
- 项目团队

## 许可证

遵循项目主许可证。

---

*最后更新：2025-01-02*
