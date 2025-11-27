# 渲染系统

引擎提供基于WebGPU的先进渲染管线，支持PBR材质、阴影映射、光线追踪等现代渲染技术。

## 渲染架构概述

### 核心组件

- **wgpu-rs**: WebGPU API绑定
- **材质系统**: PBR材质和自定义着色器
- **光照系统**: 点光源、方向光和阴影
- **后处理**: HDR、色调映射、抗锯齿
- **优化技术**: 实例化、LOD、视锥剔除

### 渲染流程

```rust
// 典型渲染帧流程
1. 构建场景 (变换、光源、材质)
2. 视锥剔除和LOD选择
3. 深度预传递 (可选)
4. G缓冲区填充
5. 阴影贴图渲染
6. 光照计算 (延迟渲染)
7. 后处理 (HDR, AA, Bloom)
8. 最终合成
```

## 材质和着色器

### PBR材质组件

```rust
#[derive(Component)]
pub struct PbrMaterialComp {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub ambient_occlusion: f32,
    pub emissive: [f32; 3],
    pub emissive_strength: f32,
}
```

### 材质使用示例

```rust
// 创建PBR材质实体
commands.spawn((
    PbrMaterialComp {
        base_color: [0.8, 0.2, 0.2, 1.0],
        metallic: 0.1,
        roughness: 0.3,
        ambient_occlusion: 1.0,
        emissive: [0.0, 0.0, 0.0],
        emissive_strength: 0.0,
    },
    Mesh { handle: mesh_handle },
));
```

### 自定义着色器

支持WGSL着色器语法：

```wgsl
// 顶点着色器
@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 1.0);
}

// 片段着色器
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
```

## 光照系统

### 光源类型

#### 点光源 (PointLight)
```rust
PointLight {
    color: [f32; 3],
    intensity: f32,
    radius: f32,
    falloff: f32,
}
```

#### 方向光 (DirectionalLight)
```rust
DirectionalLightComp {
    direction: [f32; 3],
    color: [f32; 3],
    intensity: f32,
}
```

#### 聚光灯 (SpotLight) - 即将支持

### 阴影映射

引擎使用层级阴影映射（Cascaded Shadow Maps）：

```rust
let cascades = [50.0, 200.0, 800.0]; // 级联距离
let shadow_resolution = 2048;
let pcf_samples = 16; // 百分比接近过滤
```

## 渲染优化技术

### 实例化渲染

自动批处理相同几何体的实例：

```rust
// 实例化渲染配置
let max_instances = 1000;
let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    size: (max_instances * std::mem::size_of::<Instance>()) as u64,
    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
});
```

### 视锥剔除

基于包围盒和视锥体的剔除：

```rust
fn frustum_cull(entity: Entity, transform: &Transform, camera: &Camera) -> bool {
    let aabb = entity.compute_aabb();
    let transformed_aabb = aabb.transform(transform.matrix());
    camera.frustum.intersects_aabb(&transformed_aabb)
}
```

### 细节层次 (LOD)

基于距离的自动LOD切换：

```rust
struct LodLevel {
    distance: f32,
    mesh: Handle<GpuMesh>,
}

lod_system(entity, distance_to_camera, &mut lod_levels);
```

### 后处理效果

支持多种后处理效果：

- **HDR色调映射**: ACES颜色空间转换
- **抗锯齿**: FXAA, MSAA, TAA
- **Bloom**: 泛光效果
- **景深**: 散景模拟
- **屏幕空间环境光遮蔽 (SSAO)**
- **屏幕空间反射 (SSR)**

## 调试和性能分析

### 渲染统计

引擎提供详细的渲染性能指标：

```rust
struct RenderStats {
    draw_calls: u32,
    instances: u32,
    passes: u32,
    culled_objects: u32,
    total_objects: u32,
    // ... 更多统计数据
}
```

### 可视化调试工具

- **过绘制可视化**: 显示像素被绘制的次数
- **阴影调试**: 显示阴影贴图内容
- **法线调试**: 可视化法线贴图
- **深度缓冲**: 显示深度信息

## 高级渲染特性

### 延迟渲染管线

多通渲染技术，将几何和光照分离：

1. **几何通**: 存储位置、法线、材质属性
2. **光照通**: 计算每个像素的光照
3. **组合通**: 应用后处理和混合

### 前向渲染管线

传统单通渲染，适合半透明对象：

```rust
for light in scene.lights {
    for mesh in scene.meshes {
        if mesh_intersects_light(mesh, light) {
            render_mesh_with_light(mesh, light);
        }
    }
}
```

### 计算着色器加速

使用GPU计算进行复杂计算：

- **粒子系统物理**
- **程序化纹理生成**
- **后处理效果**
- **AI辅助上采样**

## 平台兼容性

### 支持的后端
- **Vulkan**: 高性能主流平台
- **Metal**: Apple平台
- **DirectX 12**: Windows平台
- **WebGPU**: Web浏览器

### 纹理格式支持
- **BCn压缩**: DXT1, DXT5 (桌面)
- **ETC2压缩**: ETC2_RGB, ETC2_RGBA (移动)
- **ASTC压缩**: 高质量压缩 (现代设备)

## 最佳实践

### 材质优化
- 重用相似材质属性
- 使用纹理 atlas 减少纹理切换
- 压缩纹理以节省内存

### 光照优化
- 限制阴影投射光源数量
- 使用光照截锥体优化
- 静态光照预计算（未来功能）

### 渲染顺序
1. 不透明几何体（从前到后）
2. 半透明几何体（从后到前）
3. UI元素
4. 后处理效果

## 故障排除

### 常见渲染问题

**闪烁/深度冲突**
- 使用更高精度的深度缓冲
- 调整near/far平面距离
- 实施深度偏移

**阴影瑕疵**
- 调整阴影贴图分辨率
- 优化PCF采样模式
- 使用级联阴影地图

**性能问题**
- 启用frustum culling
- 使用LOD系统
- 优化纹理格式
- 实施批处理渲染

### 调试技巧

1. 使用 `--features debug_rendering` 编译选项
2. 启用渲染统计输出
3. 可视化深度缓冲和G缓冲区
4. 分析GPU时间线