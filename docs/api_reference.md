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
- [物理系统](#物理系统)
- [AI系统](#ai系统)
- [动画系统](#动画系统)
- [音频系统](#音频系统)
- [脚本系统](#脚本系统)
- [场景管理](#场景管理)
- [兼容性和特性管理](#兼容性和特性管理)
- [平台检测](#平台检测)
- [编辑器](#编辑器)

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

使用协程进行批量处理和并发重载。

```rust
use game_engine::resources::hot_reload::{HotReloadManager, HotReloadEvent};
use game_engine::resources::dependency_manager::DependencyGraph;
use std::sync::{Arc, RwLock};
use std::time::Duration;

let graph = Arc::new(RwLock::new(DependencyGraph::new()));
let mut hot_reload = HotReloadManager::new("assets", graph)?;

// 监视资源文件
hot_reload.watch_resource("assets/texture.png".into());
hot_reload.watch_resource("assets/material.json".into());

// 批量处理热重载事件（协程版本）
let events = hot_reload.process_events_batch(
    100,  // 最大批处理大小
    Duration::from_millis(100)  // 超时时间
).await;

for event in events {
    match event {
        HotReloadEvent::ResourceModified(path) => {
            println!("Resource modified: {}", path.display());
            // 获取需要重新加载的资源列表（考虑依赖关系）
            let targets = hot_reload.get_reload_targets(&path);
            for target in targets {
                println!("Reloading: {}", target.display());
            }
        }
        HotReloadEvent::ResourceCreated(path) => {
            println!("Resource created: {}", path.display());
        }
        HotReloadEvent::ResourceDeleted(path) => {
            println!("Resource deleted: {}", path.display());
        }
    }
}

// 并发重载多个资源（协程版本）
let paths = vec![
    "assets/texture1.png".into(),
    "assets/texture2.png".into(),
    "assets/texture3.png".into(),
];

let results = hot_reload.reload_resources_concurrent(
    paths,
    |path| async move {
        // 重载逻辑
        println!("Reloading: {}", path.display());
        Ok(())
    }
).await;

for result in results {
    if let Err(e) = result {
        eprintln!("Reload error: {}", e);
    }
}
```

**性能优势**:
- 批量处理：支持批量处理多个热重载事件
- 并发重载：使用 Tokio 协程并发重载多个资源
- 防抖优化：合并相同路径的连续事件，避免重复处理

**代码位置**: `game_engine/src/resources/hot_reload.rs`

### GLTF模型加载

```rust
use game_engine::resources::gltf_loader::{GltfLoader, GltfLoadError};

let loader = GltfLoader::new();

// 异步加载GLTF场景
let scene = loader.load_scene("model.gltf").await?;

// 访问场景数据
println!("Scene name: {}", scene.name);
println!("Entity count: {}", scene.entities.len());

for entity in scene.entities {
    println!("Entity: {:?}", entity);
}
```

**代码位置**: `game_engine/src/resources/gltf_loader.rs`

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

### 并行消息处理

使用协程批量处理网络消息，提升性能。

```rust
use game_engine::network::parallel::{ParallelMessageProcessor, NetworkState, NetworkCompressor};
use game_engine::network::NetworkMessage;
use std::sync::Arc;

let processor = ParallelMessageProcessor::new(32); // 批处理大小
let state = Arc::new(NetworkState::default());
let compressor = Arc::new(NetworkCompressor::new());

// 同步处理（使用Rayon）
let results = processor.process_messages_parallel(
    messages,
    &state,
    Some(&compressor)
);

// 异步处理（使用Tokio协程，推荐）
let results = processor.process_messages_async(
    messages,
    state,
    Some(compressor)
).await;
```

**性能优势**:
- 批量处理：将消息分批处理，减少上下文切换
- 并发执行：使用 `tokio::task::spawn_blocking` 并发处理批次
- 非阻塞：网络消息处理不阻塞主线程

**代码位置**: `game_engine/src/network/parallel.rs`

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

### 网络同步

```rust
use game_engine::network::synchronization::{NetworkSyncManager, SyncConfig};

let config = SyncConfig {
    sync_rate: 60,  // 同步频率（Hz）
    interpolation_enabled: true,
    prediction_enabled: true,
    ..Default::default()
};

let mut sync_manager = NetworkSyncManager::new(config);

// 同步实体状态
sync_manager.sync_entity(entity_id, transform, velocity)?;

// 获取插值后的状态
if let Some(interpolated) = sync_manager.get_interpolated_state(entity_id) {
    // 使用插值后的状态
}
```

**代码位置**: `game_engine/src/network/synchronization.rs`

---

## 性能监控

### 系统性能监控

```rust
use game_engine_performance::monitoring::SystemPerformanceMonitor;

let mut monitor = SystemPerformanceMonitor::new();

// 开始监控
monitor.start()?;

// 获取性能指标
let metrics = monitor.get_metrics();
println!("Frame time: {:.2}ms", metrics.frame_time);
println!("CPU usage: {:.2}%", metrics.cpu_usage);
println!("Memory usage: {}MB", metrics.memory_usage_mb);
println!("GPU usage: {:.2}%", metrics.gpu_usage);

// 获取性能报告
let report = monitor.generate_report();
println!("Average FPS: {:.2}", report.avg_fps);
println!("Min frame time: {:.2}ms", report.min_frame_time);
println!("Max frame time: {:.2}ms", report.max_frame_time);
```

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

// 记录协程任务指标
dashboard.record_coroutine_tasks(10, 5, 0); // active, completed, failed

// 记录SIMD指标
dashboard.record_simd_backend("AVX2");
dashboard.record_simd_width(8);

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

### 连续性能分析器

```rust
use game_engine::profiling::continuous_profiler::ContinuousProfiler;

let mut profiler = ContinuousProfiler::new(300); // 保留300帧

// 在每帧开始时记录
profiler.start_frame();

// 执行游戏逻辑...

// 在每帧结束时记录
profiler.end_frame();

// 分析性能趋势
let analysis = profiler.analyze();
if analysis.avg_frame_time > 16.67 {
    println!("Performance warning: Average frame time exceeds target");
}

// 检测性能瓶颈
let bottlenecks = profiler.detect_bottlenecks();
for bottleneck in bottlenecks {
    println!("Bottleneck: {} (impact: {:.2}%)", 
             bottleneck.name, bottleneck.impact_percent);
}
```

**代码位置**: `game_engine/src/profiling/`, `game_engine_performance/src/monitoring/`

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

## 物理系统

### PhysicsWorld

物理世界，负责物理模拟和碰撞检测。

```rust
use game_engine::domain::physics::{PhysicsWorld, RigidBody, RigidBodyType};

let mut physics_world = PhysicsWorld::new();

// 创建动态刚体
let body = RigidBody::new(
    RigidBodyId::new(1),
    RigidBodyType::Dynamic,
    Vec3::new(0.0, 10.0, 0.0),
    Quat::IDENTITY,
);
let handle = physics_world.add_body(body)?;

// 步进模拟（同步版本）
physics_world.step(0.016)?; // 60 FPS

// 步进模拟（异步版本）
physics_world.step_async(0.016).await?;

// 射线投射
if let Some((body_id, distance, hit_point)) = physics_world.raycast(
    Vec3::new(0.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    100.0,
) {
    println!("Hit body {} at distance {}: {:?}", body_id.as_u64(), distance, hit_point);
}
```

### 并行物理世界

使用独立线程进行物理模拟，避免阻塞主线程。

```rust
use game_engine::physics::parallel::ParallelPhysicsWorld;

let mut parallel_physics = ParallelPhysicsWorld::new();

// 创建刚体
parallel_physics.create_rigid_body(1, 0, 0.0, 10.0); // id, type, x, y
parallel_physics.create_collider(1, Some(1), 0, [0.5, 0.5], 0.0);

// 执行物理步进（非阻塞）
parallel_physics.step(0.016);

// 读取状态
let snapshot = parallel_physics.read_state();
if let Some(pos) = snapshot.positions.get(&1) {
    println!("Body position: {:?}", pos);
}
```

**代码位置**: `game_engine/src/domain/physics.rs`, `game_engine/src/physics/parallel.rs`

---

## AI系统

### 异步寻路服务

基于协程的异步寻路服务，提供高性能的路径查找。

```rust
use game_engine::ai::pathfinding::AsyncPathfindingService;
use game_engine::ai::pathfinding::NavigationMesh;

// 创建导航网格
let nav_mesh = NavigationMesh::new(/* ... */);

// 创建异步寻路服务
let service = AsyncPathfindingService::new(nav_mesh, 10); // max_concurrent = 10

// 异步寻路
let path = service.find_path(
    Vec3::new(0.0, 0.0, 0.0),
    Vec3::new(10.0, 0.0, 10.0)
).await?;

if let Some(path) = path {
    println!("Path found with {} waypoints", path.len());
    for waypoint in path {
        println!("  Waypoint: {:?}", waypoint);
    }
}
```

**性能数据**:
- 延迟降低 12.5%（相比线程池版本）
- 内存使用减少 97%+（1000并发时）
- 上下文切换开销降低 5-10倍

**代码位置**: `game_engine/src/ai/pathfinding.rs`

---

## 动画系统

### 关键帧动画

```rust
use game_engine::animation::{AnimationClip, Keyframe, AnimationPlayer};

// 创建动画片段
let mut clip = AnimationClip::new("walk".to_string(), 1.0); // 1秒动画

// 添加关键帧
clip.add_keyframe(Keyframe {
    time: 0.0,
    value: Transform {
        position: Vec3::new(0.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    },
});

clip.add_keyframe(Keyframe {
    time: 1.0,
    value: Transform {
        position: Vec3::new(1.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    },
});

// 创建动画播放器
let mut player = AnimationPlayer::new();
player.add_clip(clip);

// 播放动画
player.play("walk", true); // 循环播放

// 更新动画
player.update(0.016); // 更新16ms

// 获取当前变换
if let Some(transform) = player.get_current_transform() {
    // 应用变换到实体
}
```

**代码位置**: `game_engine/src/animation/`

---

## 音频系统

### 音频流式加载

异步音频流式加载，支持多个音频流并发加载。

```rust
use game_engine::audio::streaming::{AudioStreamLoader, StreamConfig};

let mut loader = AudioStreamLoader::new();

// 同步加载（向后兼容）
let stream_id = loader.start_streaming(
    "background_music.ogg",
    StreamConfig::default()
)?;

// 异步加载（协程版本，推荐）
let stream_id = loader.start_streaming_async(
    "background_music.ogg",
    StreamConfig {
        buffer_size: 44100,
        preload_buffers: 2,
        looped: true,
        sample_rate: Some(44100),
        channels: Some(2),
    }
).await?;

// 获取音频流
if let Some(stream) = loader.get_stream(stream_id) {
    let mut stream = stream.lock().unwrap();
    
    // 播放流
    stream.play()?;
    
    // 获取样本数据
    let samples = stream.get_samples(1024)?;
    // 发送到音频设备...
}

// 并发更新所有流
loader.update_all_async().await?;
```

### 音频效果

```rust
use game_engine::audio::effects::{EffectChain, ReverbConfig, EqualizerConfig};

let mut effect_chain = EffectChain::new();

// 添加混响效果
effect_chain.add_reverb(ReverbConfig {
    room_size: 0.5,
    damping: 0.5,
    wet_level: 0.3,
    dry_level: 0.7,
});

// 添加均衡器效果
effect_chain.add_equalizer(EqualizerConfig {
    low_gain: 1.0,
    mid_gain: 0.8,
    high_gain: 1.2,
});

// 处理音频样本
let processed_samples = effect_chain.process(&input_samples)?;
```

**代码位置**: `game_engine/src/audio/streaming.rs`, `game_engine/src/audio/effects.rs`

---

## 脚本系统

### 脚本组件

使用脚本组件为实体添加脚本逻辑。

```rust
use game_engine::scripting::engine::ScriptComponent;

// 创建脚本组件
let script = ScriptComponent::new(
    r#"
    function update(delta_time) {
        // 脚本逻辑
        entity.position.x += 1.0 * delta_time;
    }
    "#.to_string(),
    "lua".to_string(), // 或 "wasm"
);

// 添加到实体
world.entity(entity).insert(script);
```

### WASM脚本支持

```rust
use game_engine::scripting::wasm_support::WasmRuntime;

let mut runtime = WasmRuntime::new()?;

// 加载WASM模块
runtime.load_module("script.wasm").await?;

// 调用导出函数
let result = runtime.call_function("update", &[delta_time.into()])?;
```

**代码位置**: `game_engine/src/scripting/engine.rs`, `game_engine/src/scripting/wasm_support.rs`

---

## 场景管理

### 场景序列化

```rust
use game_engine::scene::{SceneManager, SerializedScene};

let mut scene_manager = SceneManager::new();

// 保存场景
let scene = scene_manager.serialize_scene(&world)?;
let json = serde_json::to_string(&scene)?;
std::fs::write("scene.json", json)?;

// 加载场景
let json = std::fs::read_to_string("scene.json")?;
let scene: SerializedScene = serde_json::from_str(&json)?;
scene_manager.deserialize_scene(&mut world, &scene)?;
```

**代码位置**: `game_engine/src/scene/serialization.rs`

---

## 兼容性和特性管理

### 特性管理

统一管理引擎特性，支持运行时特性检查。

```rust
use game_engine::compat::features::{FeatureSet, Feature};

let mut features = FeatureSet::new();

// 启用特性
features.enable(Feature::Wasm);
features.enable(Feature::Gltf);
features.enable(Feature::Physics);

// 检查特性
if features.is_enabled(&Feature::Wasm) {
    // WASM相关代码
}

// 获取所有启用的特性
let enabled = features.get_enabled_features();
for feature in enabled {
    println!("Enabled: {:?}", feature);
}
```

**代码位置**: `game_engine/src/compat/features.rs`

---

## 平台检测

### 平台检测函数

提供统一的平台检测功能。

```rust
use game_engine::platform::detection::{
    is_wasm, is_mobile, is_desktop,
    is_windows, is_macos, is_linux,
    current_platform_name
};

// 检查平台类型
if is_wasm() {
    println!("Running on WebAssembly");
} else if is_mobile() {
    println!("Running on mobile device");
} else if is_desktop() {
    println!("Running on desktop");
}

// 检查特定平台
if is_windows() {
    // Windows特定代码
} else if is_macos() {
    // macOS特定代码
} else if is_linux() {
    // Linux特定代码
}

// 获取平台名称
let platform = current_platform_name();
println!("Current platform: {}", platform);
```

**代码位置**: `game_engine/src/platform/detection.rs`

---

## 编辑器

### 场景编辑器

```rust
use game_engine::editor::scene_editor::SceneEditor;

let mut editor = SceneEditor::new();

// 选择实体
editor.select_entity(entity);

// 移动实体
editor.move_entity(entity, Vec3::new(1.0, 0.0, 0.0));

// 撤销/重做
editor.undo();
editor.redo();
```

### 性能监控面板

```rust
use game_engine::editor::performance_panel::PerformancePanel;

let mut panel = PerformancePanel::new();

// 显示性能指标
panel.show_fps();
panel.show_memory_usage();
panel.show_cpu_usage();
panel.show_gpu_usage();
```

**代码位置**: `game_engine/src/editor/`

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

## 协程任务管理

### CoroutineTaskManager

协程任务管理器，用于在游戏循环中提交和管理异步任务。

```rust
use game_engine::core::engine::game_loop_coroutine::CoroutineTaskManager;
use game_engine::core::engine::game_loop_coroutine::TaskPriority;

// 获取任务管理器（从ECS资源）
let task_manager = world.get_resource::<CoroutineTaskManager>().unwrap();

// 提交异步任务
let task_id = task_manager.spawn_task(
    "ai_update".to_string(),
    TaskPriority::Normal,
    || async move {
        // 异步任务逻辑
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }
).await;

// 取消任务
task_manager.cancel_task(task_id).await?;

// 获取任务统计
let stats = task_manager.stats().await;
println!("Active tasks: {}", stats.active_tasks);
println!("Completed tasks: {}", stats.completed_tasks);
```

**代码位置**: `game_engine/src/core/engine/game_loop_coroutine.rs`

---

## GPU计算着色器

### GPU计算上下文

使用GPU计算着色器加速大规模并行计算。

```rust
use game_engine::performance::gpu::gpu_compute::GpuComputeContext;

let context = GpuComputeContext::new(device, queue)?;

// 粒子系统配置
let config = ParticleSystemConfig {
    max_particles: 100000,
    enable_wind: true,
    enable_color_gradient: true,
    enable_size_animation: true,
    enable_rotation: true,
};

// 创建GPU粒子系统
let mut particle_system = GpuParticleSystem::new(context, config)?;

// 更新粒子（在GPU上执行）
particle_system.update(delta_time)?;

// 获取粒子数据
let particles = particle_system.get_particles()?;
```

**性能数据**:
- 粒子系统（10万粒子）：CPU ~16ms → GPU ~1ms（**16x提升**）
- 批量寻路（1000个）：CPU ~400ms → GPU ~20ms（**20x提升**）

**代码位置**: `game_engine/src/performance/gpu/gpu_compute.rs`

---

## 相关文档

- [用户指南](user_guide/index.html)
- [架构文档](architecture.md)
- [最佳实践](BEST_PRACTICES.md)
- [性能调优指南](performance_tuning_guide.md)
- [协程游戏循环评估](coroutine_game_loop_evaluation.md)
- [条件编译指南](CONDITIONAL_COMPILATION_GUIDE.md)

---

## API索引

### 按模块分类

| 模块 | 主要API | 文档位置 |
|------|---------|---------|
| 核心引擎 | `Engine`, `CoroutineTaskManager` | [核心引擎](#核心引擎) |
| 渲染 | `GpuDrivenRenderer`, `LodSelector`, `PostProcessEffectManager` | [渲染系统](#渲染系统) |
| 物理 | `PhysicsWorld`, `ParallelPhysicsWorld` | [物理系统](#物理系统) |
| AI | `AsyncPathfindingService` | [AI系统](#ai系统) |
| 动画 | `AnimationClip`, `AnimationPlayer` | [动画系统](#动画系统) |
| 音频 | `AudioStreamLoader`, `EffectChain` | [音频系统](#音频系统) |
| 网络 | `ParallelMessageProcessor`, `WebRtcConnection` | [网络系统](#网络系统) |
| 资源 | `UnifiedResourceManager`, `HotReloadManager`, `GltfLoader` | [资源管理](#资源管理) |
| 脚本 | `ScriptComponent`, `WasmRuntime` | [脚本系统](#脚本系统) |
| 场景 | `SceneManager`, `SerializedScene` | [场景管理](#场景管理) |
| 兼容性 | `FeatureSet`, `Feature` | [兼容性和特性管理](#兼容性和特性管理) |
| 平台 | `is_wasm()`, `is_mobile()`, `current_platform_name()` | [平台检测](#平台检测) |
| 编辑器 | `SceneEditor`, `PerformancePanel` | [编辑器](#编辑器) |
| 性能监控 | `SystemPerformanceMonitor`, `PerformanceDashboard` | [性能监控](#性能监控) |
| GPU计算 | `GpuComputeContext`, `GpuParticleSystem` | [GPU计算着色器](#gpu计算着色器) |

---

**文档版本**: 2.0  
**最后更新**: 2025-12-23  
**维护者**: Game Engine Team

