# 渲染系统教程

本教程介绍游戏引擎的渲染系统，帮助您理解并使用WebGPU创建高性能的2D/3D图形渲染。

## 目录

1. [渲染系统概述](#渲染系统概述)
2. [设置渲染管线](#设置渲染管线)
3. [创建着色器](#创建着色器)
4. [加载纹理和模型](#加载纹理和模型)
5. [光照系统](#光照系统)
6. [后处理效果](#后处理效果)
7. [性能优化](#性能优化)
8. [实战案例](#实战案例)

## 渲染系统概述

### 渲染架构

```
┌────────────────────────────────────────────────────┐
│                 渲染管线                            │
├────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐        │
│  │ 资源加载  │ -> │ 着色器   │ -> │ 渲染目标 │        │
│  │ Assets  │    │ Shaders │    │ Targets │        │
│  └─────────┘    └─────────┘    └─────────┘        │
│                                                      │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐        │
│  │ 场景管理  │ -> │ 视锥剔除 │ -> │ 渲染提交 │        │
│  │ Scene   │    │ Culling │    │ Submit  │        │
│  └─────────┘    └─────────┘    └─────────┘        │
│                                                      │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐        │
│  │ PBR渲染  │ -> │ 后处理   │ -> │ 呈现     │        │
│  │   PBR   │    │ Post-FX │    │ Present │        │
│  └─────────┘    └─────────┘    └─────────┘        │
│                                                      │
└────────────────────────────────────────────────────┘
```

### 核心特性

- **WebGPU 后端**: 现代跨平台图形 API
- **PBR 渲染**: 基于物理的渲染
- **延迟渲染**: 支持多光源场景
- **视锥剔除**: 自动剔除不可见对象
- **实例化渲染**: 高效渲染大量相同对象
- **后处理**: Bloom、色调映射等效果

## 设置渲染管线

### 初始化渲染器

```rust
use game_engine::render::wgpu_utils::WgpuRenderer;
use game_engine::platform::winit::WinitWindow;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建窗口
    let window = WinitWindow::new("My Game", 1280, 720);

    // 初始化渲染器
    let mut renderer = WgpuRenderer::new(&window).await?;

    // 游戏循环
    loop {
        renderer.begin_frame();
        // 渲染场景
        renderer.end_frame(&window)?;
    }
}
```

### 配置渲染选项

```rust
use game_engine::render::wgpu_utils::RenderConfig;

let config = RenderConfig {
    vsync: true,
    samples: 4,  // 4x MSAA
    shadows: true,
    shadow_quality: 2048,
    bloom: true,
    bloom_threshold: 0.8,
    // 更多选项...
};

renderer.configure(config);
```

### 渲染管线布局

```rust
// 创建渲染管线
let pipeline = renderer.create_pipeline(&PipelineDescriptor {
    label: Some("My Pipeline"),
    layout: Some(&pipeline_layout),
    vertex: vertex_shader,
    fragment: fragment_shader,
    primitive: PrimitiveState {
        topology: PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: FrontFace::Ccw,
        cull_mode: Some(Face::Back),
        polygon_mode: PolygonMode::Fill,
        ..Default::default()
    },
    depth_stencil: Some(depth_state),
    multisample: MultisampleState {
        count: 4,
        mask: !0,
        alpha_to_coverage_enabled: false,
    },
    ..Default::default()
});
```

## 创建着色器

### WGSL 着色器语言

引擎使用 WGSL (WebGPU Shading Language) 作为着色器语言。

### 顶点着色器

创建 `shaders/base.vert.wgsl`:

```wgsl
// 顶点输入
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

// 实例化输入
struct InstanceInput {
    @location(3) model_matrix_0: vec4<f32>,
    @location(4) model_matrix_1: vec4<f32>,
    @location(5) model_matrix_2: vec4<f32>,
    @location(6) model_matrix_3: vec4<f32>,
    @location(7) color: vec4<f32>,
}

// Uniform 缓冲
struct CameraUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

// 顶点输出
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) color: vec4<f32>,
}

@vertex
fn vs_main(
    input: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var output: VertexOutput;

    // 构建模型矩阵
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    // 计算世界位置
    let world_pos = model_matrix * vec4<f32>(input.position, 1.0);

    // 输出裁剪空间位置
    output.clip_position = camera.view_proj * world_pos;
    output.world_pos = world_pos.xyz;
    output.uv = input.uv;
    output.normal = normalize((model_matrix * vec4<f32>(input.normal, 0.0)).xyz);
    output.color = instance.color;

    return output;
}
```

### 片段着色器

创建 `shaders/base.frag.wgsl`:

```wgsl
// 光照数据
struct PointLight {
    position: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
    radius: f32,
};

struct LightUniforms {
    lights: array<PointLight, 16>,
    light_count: u32,
};

@group(0) @binding(1)
var<uniform> lights: LightUniforms;

// 纹理
@group(0) @binding(2)
var texture_diffuse: texture_2d<f32>;

@group(0) @binding(3)
var sampler_diffuse: sampler;

// PBR 材质参数
struct MaterialUniforms {
    albedo: vec4<f32>,
    metallic: f32,
    roughness: f32,
    ao: f32,
};

@group(0) @binding(4)
var<uniform> material: MaterialUniforms;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // 采样纹理
    let tex_color = textureSample(texture_diffuse, sampler_diffuse, input.uv);

    // 基础颜色
    let albedo = tex_color.rgb * material.albedo.rgb * input.color.rgb;

    // 法线（假设已归一化）
    let N = normalize(input.normal);

    // 视线方向
    let V = normalize(camera.camera_pos - input.world_pos);

    // 直接光照
    var Lo = vec3<f32>(0.0);

    for (var i: u32 = 0u; i < lights.light_count; i = i + 1u) {
        let light = lights.lights[i];

        // 光照方向
        let L = normalize(light.position - input.world_pos);
        let H = normalize(V + L);

        // 距离衰减
        let distance = length(light.position - input.world_pos);
        let attenuation = 1.0 / (distance * distance);

        // 漫反射
        let NdotL = max(dot(N, L), 0.0);
        let diffuse = albedo / 3.14159265359;

        // 镜面反射（简化）
        let specular = pow(max(dot(N, H), 0.0), 16.0);

        // 合并
        let radiance = light.color * light.intensity * attenuation;
        Lo = Lo + (diffuse + specular) * radiance * NdotL;
    }

    // 环境光
    let ambient = albedo * 0.1;

    // 最终颜色
    let color = ambient + Lo;

    // HDR 色调映射
    color = color / (color + vec3<f32>(1.0));

    // Gamma 校正
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, 1.0);
}
```

### 加载着色器

```rust
use game_engine::render::shader::ShaderLoader;

// 加载着色器
let vertex_shader = ShaderLoader::load_wgsl("shaders/base.vert.wgsl")?;
let fragment_shader = ShaderLoader::load_wgsl("shaders/base.frag.wgsl")?;

// 创建管线
let pipeline = renderer.create_render_pipeline(
    &vertex_shader,
    &fragment_shader,
    &pipeline_layout,
);
```

## 加载纹理和模型

### 加载纹理

```rust
use game_engine::resources::manager::ResourceManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut resource_manager = ResourceManager::new();

    // 加载 2D 纹理
    let texture_handle = resource_manager
        .load_texture("assets/textures/character.png")
        .await?;

    // 加载法线贴图
    let normal_handle = resource_manager
        .load_texture("assets/textures/character_normal.png")
        .await?;

    // 设置纹理参数
    resource_manager.set_texture_wrap(
        texture_handle,
        WrapMode::Repeat,
    );

    resource_manager.set_texture_filter(
        texture_handle,
        FilterMode::Linear,
    );

    Ok(())
}
```

### 纹理格式支持

引擎支持多种纹理格式：

| 格式 | 扩展名 | 说明 |
|------|--------|------|
| PNG | `.png` | 无损压缩，推荐用于 UI |
| JPEG | `.jpg`, `.jpeg` | 有损压缩，文件小 |
| DDS | `.dds` | DirectX 纹理，支持压缩 |
| KTX/KTX2 | `.ktx`, `.ktx2` | Khronos 纹理，GPU 优化 |
| BASIS | `.basis` | 通用纹理压缩 |

### 加载模型

```rust
use game_engine::resources::model_loader;

// 加载 glTF 模型
let gltf_model = model_loader::load_gltf(
    "assets/models/character.gltf",
    &resource_manager,
).await?;

// 加载 FBX 模型（需要 feature）
#[cfg(feature = "fbx")]
let fbx_model = model_loader::load_fbx(
    "assets/models/character.fbx",
).await?;

// 加载 OBJ 模型
let obj_model = model_loader::load_obj(
    "assets/models/scene.obj",
)?;
```

### 模型组件

```rust
use bevy_ecs::prelude::*;
use game_engine::ecs::{Transform, ModelHandle};

fn spawn_model(mut commands: Commands, model_handle: ModelHandle) {
    commands.spawn((
        Transform::default(),
        ModelHandle(model_handle),
    ));
}
```

## 光照系统

### 光源类型

引擎支持多种光源类型：

```rust
use game_engine::ecs::{PointLight, DirectionalLightComp};

// 点光源
commands.spawn((
    Transform {
        pos: Vec3::new(0.0, 5.0, 0.0),
        ..Default::default()
    },
    PointLight {
        color: [1.0, 1.0, 1.0],  // 白光
        intensity: 100.0,
        radius: 20.0,
        falloff: 2.0,
    },
));

// 平行光（方向光）
commands.spawn((
    DirectionalLightComp {
        direction: Vec3::new(-1.0, -1.0, -1.0).normalize(),
        color: [1.0, 0.9, 0.8],  // 暖色
        intensity: 0.5,
    },
));
```

### 光照配置

```rust
use game_engine::render::lighting::LightingConfig;

let config = LightingConfig {
    max_lights: 16,           // 最大光源数
    ambient_color: [0.1, 0.1, 0.15],
    ambient_intensity: 0.1,
    shadow_cascades: 4,       // 级联阴影
    shadow_distance: 100.0,
};

renderer.set_lighting_config(config);
```

### 阴影

```rust
// 启用阴影
let shadow_config = ShadowConfig {
    resolution: 2048,
    soft_shadows: true,
    bias: 0.005,
    normal_bias: 0.01,
};

renderer.enable_shadows(shadow_config);
```

## 后处理效果

### Bloom（辉光）

```rust
use game_engine::render::post_processing::BloomConfig;

let bloom = BloomConfig {
    threshold: 0.8,
    intensity: 0.5,
    radius: 0.1,
};

renderer.enable_bloom(bloom);
```

### 色调映射

```rust
use game_engine::render::post_processing::ToneMapping;

// ACES 色调映射
renderer.set_tone_mapping(ToneMapping::Aces);

// Reinhard 色调映射
renderer.set_tone_mapping(ToneMapping::Reinhard);

// AgX 色调映射
renderer.set_tone_mapping(ToneMapping::AgX);
```

### 其他效果

```rust
// 景深
let dof = DepthOfFieldConfig {
    focus_distance: 10.0,
    aperture: 0.1,
    max_blur: 0.5,
};
renderer.enable_depth_of_field(dof);

// 运动模糊
let motion_blur = MotionBlurConfig {
    strength: 0.5,
    samples: 8,
};
renderer.enable_motion_blur(motion_blur);

// 抗锯齿
renderer.set_msaa(4);  // 4x MSAA

// 屏幕空间环境光遮蔽
let ssao = SSAOConfig {
    radius: 0.5,
    bias: 0.025,
    intensity: 1.0,
};
renderer.enable_ssao(ssao);
```

## 性能优化

### 批处理

```rust
// 实例化渲染 - 一次绘制多个相同对象
use game_engine::render::instancing::InstancedRender;

let instanced = InstancedRender::new();

// 添加实例
for i in 0..1000 {
    instanced.add_instance(InstanceData {
        model_matrix: model_matrix,
        color: [1.0, 1.0, 1.0, 1.0],
    });
}

// 一次性绘制所有实例
instanced.draw(&renderer);
```

### 视锥剔除

```rust
use game_engine::render::culling::FrustumCulling;

let mut culling = FrustumCulling::new(camera);

// 自动剔除不可见对象
let visible = culling.cull_scene(&scene);

// 只渲染可见对象
renderer.render_visible(visible);
```

### LOD（细节层次）

```rust
use game_engine::render::lod::LODSystem;

// 设置 LOD 级别
let mut lod_system = LODSystem::new();

lod_system.add_lod(
    model_handle,
    LODLevel {
        distance: 0.0,
        model: high_poly_model,
    },
);

lod_system.add_lod(
    model_handle,
    LODLevel {
        distance: 50.0,
        model: medium_poly_model,
    },
);

lod_system.add_lod(
    model_handle,
    LODLevel {
        distance: 100.0,
        model: low_poly_model,
    },
);

// 自动选择 LOD
lod_system.update(camera.position);
```

### 纹理图集

```rust
use game_engine::render::atlas::TextureAtlas;

// 创建纹理图集
let mut atlas = TextureAtlas::new(2048, 2048);

// 添加纹理
let rect1 = atlas.add_texture("sprite1.png")?;
let rect2 = atlas.add_texture("sprite2.png")?;

// 构建图集
atlas.build(&renderer)?;

// 使用图集渲染
renderer.draw_atlas_sprite(atlas, rect1);
```

### GPU 实例化统计

```rust
// 查看渲染统计
let stats = renderer.get_stats();

println!("Draw calls: {}", stats.draw_calls);
println!("Vertices: {}", stats.vertices);
println!("Triangles: {}", stats.triangles);
println!("FPS: {}", stats.fps);
println!("Frame time: {:.2}ms", stats.frame_time);
```

## 实战案例

### 案例 1: 2D 精灵渲染

```rust
use bevy_ecs::prelude::*;
use game_engine::ecs::{Transform, Sprite};

fn spawn_sprite(mut commands: Commands) {
    commands.spawn((
        Transform {
            pos: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
        Sprite {
            tex_index: 0,
            uv_off: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            layer: 0.0,
        },
    ));
}
```

### 案例 2: 3D 模型渲染

```rust
fn spawn_3d_model(
    mut commands: Commands,
    model: Res<ModelHandle>,
) {
    commands.spawn((
        Transform {
            pos: Vec3::new(0.0, 0.0, -5.0),
            rot: Quat::from_euler(glam::EulerRot::YXZ, 0.0, 0.0, 0.0),
            scale: Vec3::ONE,
        },
        ModelHandle(model.clone()),
        Material {
            albedo: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.5,
            roughness: 0.5,
            ..Default::default()
        },
    ));
}
```

### 案例 3: 粒子系统

```rust
#[derive(Component)]
struct Particle {
    lifetime: f32,
    velocity: Vec3,
}

fn spawn_particles(mut commands: Commands) {
    for _ in 0..100 {
        commands.spawn((
            Transform::default(),
            Particle {
                lifetime: 2.0,
                velocity: Vec3::new(
                    rand::random::<f32>() - 0.5,
                    rand::random::<f32>(),
                    rand::random::<f32>() - 0.5,
                ).normalize() * 5.0,
            },
        ));
    }
}

fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Particle)>
) {
    for (entity, mut transform, mut particle) in query.iter_mut() {
        particle.lifetime -= time.delta;
        transform.pos += particle.velocity * time.delta;

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

### 案例 4: 天空盒

```rust
use game_engine::render::skybox::Skybox;

async fn setup_skybox(
    renderer: &mut WgpuRenderer,
    resource_manager: &mut ResourceManager,
) -> Result<(), Box<dyn std::error::Error>> {
    // 加载天空盒纹理
    let skybox_texture = resource_manager.load_cubemap([
        "assets/skybox/right.jpg",
        "assets/skybox/left.jpg",
        "assets/skybox/top.jpg",
        "assets/skybox/bottom.jpg",
        "assets/skybox/front.jpg",
        "assets/skybox/back.jpg",
    ]).await?;

    // 创建天空盒
    let skybox = Skybox::new(renderer, skybox_texture)?;

    // 设置为当前天空盒
    renderer.set_skybox(skybox);

    Ok(())
}
```

## 调试和性能分析

### 渲染调试模式

```rust
// 启用调试可视化
renderer.set_debug_mode(DebugRenderMode {
    show_wireframe: false,
    show_normals: false,
    show_depth: false,
    show_albedo: false,
    show_lighting: true,
});
```

### GPU 性能分析

```rust
use game_engine::performance::profiling::GpuProfiler;

let mut profiler = GpuProfiler::new(&renderer);

// 开始分析
profiler.begin_scope("Frame");

// 渲染场景
profiler.begin_scope("Shadow Pass");
renderer.render_shadows();
profiler.end_scope();

profiler.begin_scope("Main Pass");
renderer.render_scene();
profiler.end_scope();

// 结束分析
profiler.end_scope();

// 获取结果
let results = profiler.get_results();
println!("Shadow pass: {:.2}ms", results.get("Shadow Pass"));
println!("Main pass: {:.2}ms", results.get("Main Pass"));
```

## 常见问题

### Q: 如何处理透明对象？

A: 透明对象需要按深度排序后渲染：

```rust
fn sort_transparent(
    camera_pos: Vec3,
    transparent: &mut Vec<(Entity, Transform)>,
) {
    transparent.sort_by(|a, b| {
        let dist_a = a.1.pos.distance(camera_pos);
        let dist_b = b.1.pos.distance(camera_pos);
        dist_b.partial_cmp(&dist_a).unwrap()
    });
}
```

### Q: 如何优化大量光源？

A: 使用延迟渲染或 tiled 渲染：

```rust
let config = RenderingConfig {
    mode: RenderMode::Deferred,  // 延迟渲染
    light_culling: true,          // 光照剔除
    tile_size: 16,                // Tiled 大小
};
```

### Q: 纹理显示错误怎么办？

A: 检查以下内容：

1. 纹理尺寸是否为 2 的幂
2. 纹理格式是否支持
3. UV 坐标是否正确
4. 着色器是否正确采样

## 参考资源

- [WebGPU 规范](https://gpuweb.github.io/gpuweb/)
- [WGSL 语言指南](https://www.w3.org/TR/WGSL/)
- [PBR 理论](https://google.github.io/filament/Filament.md.html)
- [渲染技术文档](../RENDERING_TECHNICAL.md)

## 总结

渲染系统是游戏引擎的核心组件，通过本教程您应该已经掌握：

- 设置和管理渲染管线
- 编写 WGSL 着色器
- 加载和使用纹理与模型
- 实现光照和阴影
- 添加后处理效果
- 优化渲染性能

继续探索和实践，创建令人惊叹的视觉效果！

---

**相关教程**:
- [快速入门指南](./getting_started.md)
- [ECS 系统深入指南](./ecs_guide.md)
