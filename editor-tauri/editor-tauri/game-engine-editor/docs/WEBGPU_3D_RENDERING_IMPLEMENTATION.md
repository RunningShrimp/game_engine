# WebGPU 3D渲染实现文档

## 概述

本文档描述了为Tauri编辑器实现的WebGPU 3D渲染功能。该实现使用wgpu 22库，提供了完整的3D渲染管线，包括着色器、相机系统、几何体生成和性能统计。

## 架构

### 核心模块

1. **camera.rs** - 相机系统
   - 透视投影和视图矩阵计算
   - 轨道控制器（Orbit Camera）
   - 鼠标交互（旋转、平移、缩放）

2. **geometry.rs** - 几何体生成器
   - 立方体网格生成
   - 球体网格生成
   - 网格平面生成

3. **webgpu_renderer.rs** - 渲染器核心
   - WebGPU设备和队列管理
   - 渲染管线创建和管理
   - Uniform缓冲区管理
   - 性能统计（FPS、帧时间）

4. **shaders.wgsl** - WGSL着色器
   - 顶点着色器（变换和光照）
   - 片段着色器（光照计算）
   - 网格着色器（透明网格渲染）

## 技术实现

### 1. 相机系统

```rust
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_degrees: f32,
    pub near: f32,
    pub aspect_ratio: f32,
}
```

**功能：**
- 视图矩阵计算：`Mat4::look_at_rh()`
- 投影矩阵计算：`Mat4::perspective_rh_gl()`
- 轨道控制：鼠标右键旋转
- 平移控制：鼠标中键平移
- 缩放控制：鼠标滚轮

### 2. 几何体生成器

#### Vertex结构
```rust
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}
```

#### 可用几何体
- **立方体**：`Mesh::cube(size)`
- **球体**：`Mesh::sphere(radius, segments, rings)`
- **网格**：`Mesh::grid(size, divisions)`

### 3. WebGPU渲染管线

#### 初始化流程
1. 创建WebGPU实例
2. 请求GPU适配器
3. 创建设备和队列
4. 配置Surface
5. 创建渲染管线

#### Uniform缓冲区
```rust
struct UniformBuffer {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    light_direction: [f32; 3],
    light_color: [f32; 3],
    ambient_color: [f32; 3],
    ambient_strength: f32,
}
```

### 4. WGSL着色器

#### 顶点着色器
- 变换顶点位置到裁剪空间
- 传递法线和UV到片段着色器
- 支持模型、视图、投影变换

#### 片段着色器
- 环境光计算
- 方向光漫反射
- 基础颜色输出（可扩展为纹理）

#### 网格着色器
- 透明网格渲染
- 边缘淡出效果
- 基于距离的透明度

### 5. 性能统计

```rust
pub struct FrameStats {
    pub fps: u32,
    pub frame_time_ms: f32,
    pub draw_calls: u32,
    pub triangles: u32,
}
```

**实现：**
- 帧时间：使用`Instant`测量
- FPS计数器：每秒更新
- Draw calls：统计渲染调用次数
- 三角形数：基于索引数量计算

## 使用示例

### 初始化渲染器

```rust
let mut renderer = WebGPURenderer::new();
renderer.initialize().await?;
renderer.setup_surface(&surface, width, height)?;
renderer.create_pipelines()?;
```

### 渲染循环

```rust
loop {
    let stats = renderer.render()?;
    println!("FPS: {}, Frame Time: {:.2}ms", stats.fps, stats.frame_time_ms);
}
```

### 相机控制

```rust
renderer.handle_mouse_down(x, y, button);  // 右键=2, 中键=1
renderer.handle_mouse_move(x, y);
renderer.handle_scroll(delta);
```

## Tauri集成

### 可用命令

1. **initialize_renderer** - 初始化渲染器
2. **render_frame** - 渲染一帧
3. **update_scene** - 更新场景数据
4. **get_frame_stats** - 获取帧统计
5. **create_entity** - 创建实体
6. **update_entity_transform** - 更新实体变换
7. **delete_entity** - 删除实体
8. **set_transform_mode** - 设置变换模式

### 前端调用示例

```typescript
import { invoke } from '@tauri-apps/api/core';

// 初始化
await invoke('initialize_renderer');

// 获取统计
const stats = await invoke('get_frame_stats');
console.log(`FPS: ${stats.fps}`);

// 创建实体
const entity = await invoke('create_entity', { name: 'Cube' });
```

## 文件结构

```
src-tauri/src/
├── camera.rs              # 相机系统和控制器
├── geometry.rs            # 几何体生成器
├── webgpu_renderer.rs     # 主渲染器
├── shaders.wgsl          # WGSL着色器代码
└── lib.rs                # Tauri命令集成
```

## 技术规格

- **WebGPU版本**: wgpu 22
- **着色器语言**: WGSL
- **数学库**: glam 0.29
- **目标帧率**: 60 FPS
- **支持的几何体**: 立方体、球体、网格平面
- **光照模型**: Phong（环境光 + 方向光）
- **相机类型**: 轨道相机（Orbit Camera）

## 性能特性

1. **实时渲染**: 支持60FPS流畅渲染
2. **高效缓冲**: 使用bytemuck进行零拷贝数据转换
3. **批量绘制**: 使用索引缓冲区优化
4. **动态uniform**: 每帧更新相机和光照参数

## 限制和未来扩展

### 当前限制
- 仅支持基础几何体（立方体、球体、网格）
- 光照模型较简单（无阴影、无PBR）
- 无纹理支持
- 无后处理效果

### 未来扩展方向
1. **纹理支持**: 添加纹理采样和UV映射
2. **PBR材质**: 实现基于物理的渲染
3. **阴影映射**: 添加实时阴影
4. **后处理**: 实现泛光、景深等效果
5. **更多几何体**: 支持加载外部模型（glTF等）
6. **多光源**: 支持点光源、聚光灯等
7. **深度缓冲**: 添加深度测试和写入
8. **实例化渲染**: 支持大量相同物体的渲染

## 故障排除

### 编译错误
如果遇到编译错误，确保：
- Rust版本 >= 1.70
- 正确安装wgpu依赖
- 系统支持WebGPU（或使用WebGL回退）

### 运行时错误
- **无可用GPU**: 检查系统是否支持WebGPU
- **Surface配置失败**: 确保Surface在正确的线程上创建
- **着色器编译失败**: 检查WGSL语法

## 性能优化建议

1. **减少Draw Calls**: 合并小物体，使用实例化渲染
2. **优化几何体**: 使用LOD（细节层次）系统
3. **批处理**: 合并相同材质的物体
4. **剔除**: 实现视锥体剔除和遮挡剔除
5. **多线程**: 使用compute shader进行并行计算

## 参考资料

- [WebGPU Specification](https://www.w3.org/TR/webgpu/)
- [wgpu-rs Documentation](https://docs.rs/wgpu/)
- [WGSL Language](https://gpuweb.github.io/gpuweb/wgsl.html)
- [Learn OpenGL](https://learnopengl.com/)

## 贡献者

- 实现日期: 2026-01-02
- 版本: 1.0.0
- 许可: MIT
