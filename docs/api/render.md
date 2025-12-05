# 渲染系统 API 参考

## WgpuRenderer

WGPU 渲染器，负责所有渲染操作。

### 示例

```rust
use game_engine::render::WgpuRenderer;

let mut renderer = WgpuRenderer::new(device, queue, config)?;
renderer.render_frame(&mut world)?;
```

## 材质系统

### PbrMaterial

PBR（基于物理的渲染）材质。

### 示例

```rust
use game_engine::render::pbr::PbrMaterial;

let material = PbrMaterial {
    base_color: glam::Vec4::new(1.0, 0.0, 0.0, 1.0),
    metallic: 0.5,
    roughness: 0.3,
    ..Default::default()
};
```

## 纹理系统

### TextureManager

纹理管理器，负责纹理加载和管理。

### 示例

```rust
use game_engine::render::TextureManager;

let mut manager = TextureManager::new(device, queue);
let texture = manager.load_texture("texture.png").await?;
```

## 后处理效果

### PostProcessPipeline

后处理管线，支持多种后处理效果。

### 支持的效果

- 色调映射（Tone Mapping）
- 泛光（Bloom）
- 景深（Depth of Field）
- 运动模糊（Motion Blur）
- 抗锯齿（Anti-Aliasing）

