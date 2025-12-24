# API Reference

本文档提供了游戏引擎主要公共API的详细说明和使用示例。

## 目录

- [核心引擎](#核心引擎)
- [渲染系统](#渲染系统)
- [后处理效果](#后处理效果)
- [资源管理](#资源管理)
- [ECS系统](#ecs系统)
- [网络系统](#网络系统)
- [性能监控](#性能监控)

---

## 核心引擎

### Engine

主引擎入口点，负责初始化和运行游戏循环。

```rust
use game_engine::core::Engine;

// 创建并初始化引擎
let mut engine = Engine::new();
engine.initialize()?;

// 运行游戏循环
engine.run()?;
```

#### 主要方法

- `new()` - 创建新的引擎实例
- `initialize()` - 初始化引擎（GPU、音频、输入等）
- `update()` - 更新一帧
- `world()` / `world_mut()` - 获取ECS世界（只读/可变）

---

## 渲染系统

### 渲染器初始化

```rust
use game_engine::render::backend::RenderBackend;
use wgpu::Instance;

let instance = Instance::new(wgpu::InstanceDescriptor::default());
let backend = RenderBackend::new(&instance, window)?;
```

### GPU驱动渲染

```rust
use game_engine::render::{GpuDrivenRenderer, GpuDrivenConfig, GpuInstance};

// 创建GPU驱动渲染器
let config = GpuDrivenConfig {
    max_instances: 10000,
    indirect_draw: true,
    ..Default::default()
};
let mut renderer = GpuDrivenRenderer::new(device, &config)?;

// 添加实例
let instance = GpuInstance {
    transform: [1.0, 0.0, 0.0, 0.0,
                 0.0, 1.0, 0.0, 0.0,
                 0.0, 0.0, 1.0, 0.0,
                 0.0, 0.0, 0.0, 1.0],
    color: [1.0, 1.0, 1.0, 1.0],
    ..Default::default()
};
renderer.add_instance(instance);

// 渲染
renderer.render(encoder, device, queue, camera)?;
```

### 实例批处理

```rust
use game_engine::render::{BatchManager, InstanceBatch, BatchKey};

let mut batch_manager = BatchManager::new(device);

// 创建批次
let batch_key = BatchKey {
    mesh_id: 1,
    material_id: 2,
    ..Default::default()
};

let batch = InstanceBatch::new(device, &batch_key, 100)?;
batch_manager.add_batch(batch_key, batch);

// 添加实例到批次
batch_manager.add_instance(batch_key, transform, color)?;

// 渲染所有批次
batch_manager.render(encoder, device, queue)?;
```

### LOD系统

```rust
use game_engine::render::lod::{LodSelector, LodConfig, LodQuality};

let config = LodConfig {
    distances: vec![10.0, 50.0, 100.0],
    quality_levels: vec![LodQuality::High, LodQuality::Medium, LodQuality::Low],
    ..Default::default()
};

let mut selector = LodSelector::new(config);
let lod_level = selector.select_lod(camera_position, object_position);
```

### 级联阴影贴图 (CSM)

```rust
use game_engine::render::csm::{CsmRenderer, CsmConfig, ShadowQuality};

let config = CsmConfig {
    cascade_count: 4,
    shadow_resolution: 2048,
    quality: ShadowQuality::High,
    ..Default::default()
};

let mut csm_renderer = CsmRenderer::new(device, &config)?;
csm_renderer.update_cascades(light_direction, camera)?;
csm_renderer.render_shadows(encoder, device, queue, scene)?;
```

---

## 后处理效果

### PostProcessPipeline

基础后处理管线，提供固定的效果链。

```rust
use game_engine::render::postprocess::{
    PostProcessPipeline, AntialiasingMode, TonemapOperator
};
use wgpu::SurfaceConfiguration;

// 创建后处理管线
let mut postprocess = PostProcessPipeline::new(device, &surface_config);

// 配置效果
postprocess.set_bloom_enabled(true);
postprocess.set_bloom_intensity(0.8);
postprocess.set_bloom_threshold(1.0);
postprocess.set_ssao_enabled(true);
postprocess.set_ssao_params(0.5, 1.0, 0.025);
postprocess.set_tonemap_operator(TonemapOperator::ACES);
postprocess.set_exposure(1.0);

// 渲染
postprocess.render(
    &mut encoder,
    device,
    queue,
    &scene_view,
    Some(&depth_view),
    Some(&motion_vector_view),
    &output_view,
);
```

### PostProcessEffectManager

动态后处理效果管理器，支持运行时添加/移除效果和自适应质量调整。

```rust
use game_engine::render::postprocess::{
    PostProcessEffectManager, PostProcessEffect, QualityMode
};

// 创建效果管理器
let mut manager = PostProcessEffectManager::new(device, &surface_config);

// 添加效果
manager.add_effect(PostProcessEffect::Bloom {
    intensity: 0.8,
    threshold: 1.0,
    radius: 5.0,
});

manager.add_effect(PostProcessEffect::SSAO {
    radius: 0.5,
    intensity: 1.0,
    bias: 0.025,
});

manager.add_effect(PostProcessEffect::MotionBlur {
    intensity: 0.3,
    max_samples: 16,
});

manager.add_effect(PostProcessEffect::DepthOfField {
    focus_distance: 10.0,
    aperture: 0.5,
    near_blur: 1.0,
    far_blur: 2.0,
    max_blur_radius: 10.0,
});

manager.add_effect(PostProcessEffect::ColorCorrection {
    brightness: 0.0,
    contrast: 1.0,
    saturation: 1.0,
    hue_shift: 0.0,
    chromatic_aberration: 0.0,
    vignette_intensity: 0.0,
    vignette_roundness: 0.5,
});

manager.add_effect(PostProcessEffect::Tonemap {
    operator: TonemapOperator::ACES,
    exposure: 1.0,
    gamma: 2.2,
});

// 设置质量模式
manager.set_quality_mode(QualityMode::High);

// 启用自适应质量（根据性能自动调整）
manager.set_adaptive_quality(true);
manager.set_target_frame_time(16.67); // 60 FPS

// 优化效果链（合并兼容效果）
manager.optimize_chain();

// 保存预设
manager.save_preset("cinematic".to_string());

// 渲染
manager.render(
    &mut encoder,
    device,
    queue,
    &scene_view,
    Some(&depth_view),
    Some(&motion_vector_view),
    &output_view,
);

// 查看性能统计
let stats = manager.performance_stats();
for (effect_name, stat) in stats {
    println!("{}: avg={:.2}ms, max={:.2}ms, calls={}",
             effect_name, stat.avg_gpu_time, stat.max_gpu_time, stat.call_count);
}
```

### 效果预设

```rust
// 加载预设
manager.load_preset("cinematic");

// 创建自定义预设
manager.add_effect(PostProcessEffect::Bloom { intensity: 1.2, threshold: 0.8, radius: 8.0 });
manager.add_effect(PostProcessEffect::SSAO { radius: 0.7, intensity: 1.5, bias: 0.02 });
manager.set_quality_mode(QualityMode::Ultra);
manager.save_preset("ultra_quality".to_string());
```

---

## 资源管理

### UnifiedResourceManager

统一资源管理器，支持多种资源类型的加载和管理。

```rust
use game_engine::resources::{
    UnifiedResourceManager, Resource, ResourceLoader, ResourceMetadata
};

let mut manager = UnifiedResourceManager::new(device, queue);

// 加载纹理
let texture_handle = manager.load_texture("assets/texture.png").await?;

// 加载模型
let model_handle = manager.load_model("assets/model.gltf").await?;

// 加载音频
let audio_handle = manager.load_audio("assets/sound.mp3").await?;

// 获取资源
let texture = manager.get_texture(&texture_handle)?;
let model = manager.get_model(&model_handle)?;
let audio = manager.get_audio(&audio_handle)?;

// 查看缓存统计
let stats = manager.cache_stats();
println!("Cache hits: {}, misses: {}, size: {}MB",
         stats.hits, stats.misses, stats.total_size_mb);
```

### 资源依赖管理

```rust
use game_engine::resources::dependency_manager::DependencyGraph;
use game_engine::resources::ResourceDependency;

let mut graph = DependencyGraph::new();

// 添加资源
graph.add_resource("texture.png".into());
graph.add_resource("material.json".into());
graph.add_resource("model.gltf".into());

// 添加依赖关系
graph.add_dependency(
    "model.gltf".into(),
    ResourceDependency {
        path: "material.json".into(),
        dependency_type: "material".to_string(),
        required: true,
    },
)?;

graph.add_dependency(
    "material.json".into(),
    ResourceDependency {
        path: "texture.png".into(),
        dependency_type: "texture".to_string(),
        required: true,
    },
)?;

// 获取加载顺序（拓扑排序）
let load_order = graph.get_load_order()?;
for resource_path in load_order {
    println!("Loading: {}", resource_path);
}
```

### 热重载

```rust
use game_engine::resources::hot_reload::HotReloadManager;
use std::sync::mpsc;
use std::time::Duration;

let (event_tx, event_rx) = mpsc::channel(100);
let graph = Arc::new(RwLock::new(DependencyGraph::new()));

let mut hot_reload = HotReloadManager::new(
    graph.clone(),
    event_tx,
    Duration::from_millis(100),
)?;

// 监视资源文件
hot_reload.watch_resource("assets/texture.png".into())?;
hot_reload.watch_resource("assets/material.json".into())?;

// 处理热重载事件
loop {
    if let Ok(event) = event_rx.try_recv() {
        match event {
            HotReloadEvent::ResourceModified(path) => {
                println!("Resource modified: {}", path);
                // 重新加载资源
                let targets = hot_reload.get_reload_targets(&path);
                for target in targets {
                    println!("Reloading: {}", target);
                }
            }
            _ => {}
        }
    }
}
```

---

## ECS系统

### 世界和实体

```rust
use game_engine::ecs::{World, Entity, Component};

// 创建世界
let mut world = World::new();

// 创建实体
let entity = world.spawn((
    Transform::default(),
    Sprite { color: [1.0, 0.0, 0.0, 1.0], size: [50.0, 50.0] },
    Velocity { x: 1.0, y: 0.0 },
));

// 查询实体
let mut query = world.query::<(&Transform, &mut Velocity)>();
for (transform, velocity) in query.iter() {
    velocity.x += 0.1;
}

// 移除实体
world.despawn(entity);
```

### 系统

```rust
use game_engine::ecs::System;

// 定义系统
fn movement_system(query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter() {
        transform.position.x += velocity.x;
        transform.position.y += velocity.y;
    }
}

// 注册系统
world.add_system(movement_system);
```

---

## 网络系统

### WebRTC连接

```rust
use game_engine::network::webrtc::{WebRtcConnection, WebRtcConfig};

let config = WebRtcConfig {
    ice_servers: vec!["stun:stun.l.google.com:19302".to_string()],
    ..Default::default()
};

let mut connection = WebRtcConnection::new(config)?;

// 连接到对等方
connection.connect(peer_id).await?;

// 发送消息
connection.send_message(b"Hello, World!").await?;

// 接收消息
if let Some(message) = connection.receive_message().await? {
    println!("Received: {:?}", message);
}
```

---

## 性能监控

### 性能仪表盘

```rust
use game_engine::profiling::dashboard::PerformanceDashboard;

let mut dashboard = PerformanceDashboard::new("127.0.0.1:8080")?;

// 启动Web服务器
dashboard.start()?;

// 记录指标
dashboard.record_frame_time(16.67);
dashboard.record_gpu_time(5.0);
dashboard.record_cpu_time(11.67);
dashboard.record_memory_usage(1024 * 1024 * 100); // 100MB

// 访问 http://127.0.0.1:8080 查看仪表盘
```

### 性能分析器

```rust
use game_engine::profiling::profiler::Profiler;

let mut profiler = Profiler::new();

// 开始分析
profiler.start_frame();

// 标记阶段
profiler.begin_scope("update");
// ... 更新逻辑
profiler.end_scope("update");

profiler.begin_scope("render");
// ... 渲染逻辑
profiler.end_scope("render");

// 结束帧
profiler.end_frame();

// 获取报告
let report = profiler.get_report();
println!("Frame time: {:.2}ms", report.total_frame_time);
for (scope, time) in report.scope_times {
    println!("  {}: {:.2}ms", scope, time);
}
```

---

## 平台特定API

### WebAssembly

```rust
#[cfg(target_arch = "wasm32")]
use game_engine::platform::wasm_performance::{
    WasmMemoryPool, WasmMemoryPoolConfig, WasmSimdSupport
};

// 创建内存池
let config = WasmMemoryPoolConfig {
    initial_size: 1024 * 1024, // 1MB
    max_size: 10 * 1024 * 1024, // 10MB
    ..Default::default()
};
let mut memory_pool = WasmMemoryPool::new(config);

// 分配内存
let ptr = memory_pool.allocate(1024)?;

// 检查SIMD支持
let simd_support = WasmSimdSupport::detect();
if simd_support.is_available() {
    println!("SIMD is available!");
}
```

### WebGL适配器

```rust
#[cfg(target_arch = "wasm32")]
use game_engine::render::webgl_adapter::{
    WebGLAdapter, WebGLCapabilities, WGSLToGLSLConverter
};

// 检测WebGL能力
let capabilities = WebGLCapabilities::detect();
println!("WebGL Version: {:?}", capabilities.webgl_version);
println!("Max Texture Size: {}", capabilities.max_texture_size);

// 转换WGSL到GLSL
let converter = WGSLToGLSLConverter::new();
let glsl_code = converter.convert_wgsl_to_glsl(wgsl_code)?;

// 获取性能优化建议
let optimizer = WebGLAdapter::new();
let recommendations = optimizer.get_performance_recommendations(&capabilities);
for rec in recommendations {
    println!("Recommendation: {}", rec);
}
```

---

## 错误处理

所有API都返回 `Result<T, E>` 类型，使用 `?` 操作符进行错误传播：

```rust
use game_engine::error::EngineError;

fn load_game() -> Result<(), EngineError> {
    let mut engine = Engine::new();
    engine.initialize()?; // 自动传播错误
    
    let texture = manager.load_texture("texture.png").await?;
    
    Ok(())
}
```

---

## 更多示例

查看 `examples/` 目录获取更多完整示例：

- `hello_world` - 基础引擎使用
- `rendering` - 渲染示例
- `postprocess` - 后处理效果示例
- `wasm_example` - WebAssembly示例
- `multiplayer` - 多人游戏示例

---

## 相关文档

- [用户指南](user_guide/index.html)
- [架构文档](architecture.md)
- [最佳实践](BEST_PRACTICES.md)
- [性能调优指南](performance_tuning_guide.md)

