# 快速开始指南

本指南将帮助你快速上手游戏引擎，包括基础使用和最新功能的示例。

## 目录

- [安装和设置](#安装和设置)
- [基础示例](#基础示例)
- [后处理效果](#后处理效果)
- [资源管理](#资源管理)
- [WebAssembly部署](#webassembly部署)
- [性能监控](#性能监控)
- [完整游戏示例](#完整游戏示例)

---

## 安装和设置

### 1. 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
game_engine = { path = "../game_engine", features = ["gltf"] }
```

### 2. 基础设置

```rust
use game_engine::core::Engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建引擎
    let mut engine = Engine::new();
    
    // 初始化
    engine.initialize()?;
    
    // 运行游戏循环
    engine.run()?;
    
    Ok(())
}
```

---

## 基础示例

### 创建实体和组件

```rust
use game_engine::ecs::{World, Component};
use game_engine::core::Transform;

#[derive(Component)]
struct Sprite {
    color: [f32; 4],
    size: [f32; 2],
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

fn setup_scene(world: &mut World) {
    // 创建实体
    let entity = world.spawn((
        Transform::default(),
        Sprite {
            color: [1.0, 0.0, 0.0, 1.0],
            size: [50.0, 50.0],
        },
        Velocity { x: 1.0, y: 0.0 },
    ));
    
    println!("Created entity: {:?}", entity);
}
```

### 系统更新

```rust
use game_engine::ecs::{System, Query};

fn movement_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.position.x += velocity.x;
        transform.position.y += velocity.y;
    }
}

// 在引擎中注册系统
engine.add_system(movement_system);
```

---

## 后处理效果

### 使用 PostProcessPipeline（简单场景）

```rust
use game_engine::render::postprocess::{
    PostProcessPipeline, AntialiasingMode, TonemapOperator
};

// 创建后处理管线
let mut postprocess = PostProcessPipeline::new(device, &surface_config);

// 配置Bloom效果
postprocess.set_bloom_enabled(true);
postprocess.set_bloom_intensity(0.8);
postprocess.set_bloom_threshold(1.0);
postprocess.set_bloom_radius(5.0);

// 配置SSAO
postprocess.set_ssao_enabled(true);
postprocess.set_ssao_params(0.5, 1.0, 0.025);

// 配置色调映射
postprocess.set_tonemap_operator(TonemapOperator::ACES);
postprocess.set_exposure(1.0);
postprocess.set_gamma(2.2);

// 在渲染循环中使用
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

### 使用 PostProcessEffectManager（动态场景）

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

// 设置质量模式
manager.set_quality_mode(QualityMode::High);

// 启用自适应质量
manager.set_adaptive_quality(true);
manager.set_target_frame_time(16.67); // 60 FPS

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
```

### 效果预设示例

```rust
// 电影风格预设
fn setup_cinematic_preset(manager: &mut PostProcessEffectManager) {
    manager.clear_effects();
    
    manager.add_effect(PostProcessEffect::Bloom {
        intensity: 1.0,
        threshold: 0.8,
        radius: 8.0,
    });
    
    manager.add_effect(PostProcessEffect::SSAO {
        radius: 0.6,
        intensity: 1.2,
        bias: 0.02,
    });
    
    manager.add_effect(PostProcessEffect::DepthOfField {
        focus_distance: 10.0,
        aperture: 0.4,
        near_blur: 1.5,
        far_blur: 3.0,
        max_blur_radius: 15.0,
    });
    
    manager.add_effect(PostProcessEffect::ColorCorrection {
        brightness: 0.1,
        contrast: 1.1,
        saturation: 1.2,
        hue_shift: 0.0,
        chromatic_aberration: 0.1,
        vignette_intensity: 0.3,
        vignette_roundness: 0.6,
    });
    
    manager.add_effect(PostProcessEffect::Tonemap {
        operator: TonemapOperator::ACES,
        exposure: 1.2,
        gamma: 2.2,
    });
    
    manager.set_quality_mode(QualityMode::Ultra);
    manager.save_preset("cinematic".to_string());
}

// 性能优先预设
fn setup_performance_preset(manager: &mut PostProcessEffectManager) {
    manager.clear_effects();
    
    manager.add_effect(PostProcessEffect::Bloom {
        intensity: 0.5,
        threshold: 1.0,
        radius: 3.0,
    });
    
    manager.add_effect(PostProcessEffect::Tonemap {
        operator: TonemapOperator::Reinhard,
        exposure: 1.0,
        gamma: 2.2,
    });
    
    manager.set_quality_mode(QualityMode::Low);
    manager.save_preset("performance".to_string());
}
```

---

## 资源管理

### 统一资源管理器

```rust
use game_engine::resources::{
    UnifiedResourceManager, Resource, ResourceLoader
};

// 创建资源管理器
let mut manager = UnifiedResourceManager::new(device, queue);

// 加载纹理
let texture_handle = manager.load_texture("assets/texture.png").await?;

// 加载模型（需要gltf特性）
#[cfg(feature = "gltf")]
let model_handle = manager.load_model("assets/model.gltf").await?;

// 加载音频
let audio_handle = manager.load_audio("assets/sound.mp3").await?;

// 获取资源
let texture = manager.get_texture(&texture_handle)?;
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

// 获取加载顺序
let load_order = graph.get_load_order()?;
for resource_path in load_order {
    println!("Loading: {}", resource_path);
    // 按顺序加载资源
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

// 处理热重载事件（在游戏循环中）
loop {
    if let Ok(event) = event_rx.try_recv() {
        match event {
            HotReloadEvent::ResourceModified(path) => {
                println!("Resource modified: {}", path);
                // 重新加载资源
                let targets = hot_reload.get_reload_targets(&path);
                for target in targets {
                    println!("Reloading: {}", target);
                    // 重新加载资源
                }
            }
            _ => {}
        }
    }
    
    // 游戏循环...
}
```

---

## WebAssembly部署

### 构建WASM版本

```bash
# 使用构建脚本
./scripts/build_wasm.sh --release

# 或手动构建
cd game_engine
wasm-pack build --target web --release --out-dir ../dist
```

### WASM示例代码

```rust
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    log::info!("=== Game Engine WASM Example ===");

    let mut engine = Engine::new();
    if let Err(e) = engine.initialize() {
        log::error!("Failed to initialize engine: {}", e);
        return;
    }

    // 创建实体
    let world = engine.world_mut();
    world.spawn((
        Transform::default(),
        Sprite {
            color: [0.0, 0.0, 1.0, 1.0],
            size: [50.0, 50.0],
        },
    ));

    // 游戏循环（在浏览器中使用requestAnimationFrame）
}
```

### HTML集成

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Game Engine WASM</title>
</head>
<body>
    <canvas id="game-canvas"></canvas>
    <script type="module">
        import init from './game_engine.js';

        async function run() {
            await init();
            console.log('WASM module initialized');
        }

        run();
    </script>
</body>
</html>
```

### WASM性能优化

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

// 检查SIMD支持
let simd_support = WasmSimdSupport::detect();
if simd_support.is_available() {
    println!("SIMD is available!");
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

// 在游戏循环中记录指标
loop {
    let frame_start = Instant::now();
    
    // 游戏更新和渲染...
    
    let frame_time = frame_start.elapsed().as_secs_f64() * 1000.0;
    dashboard.record_frame_time(frame_time);
    dashboard.record_gpu_time(gpu_time);
    dashboard.record_cpu_time(cpu_time);
    dashboard.record_memory_usage(memory_usage);
}

// 访问 http://127.0.0.1:8080 查看仪表盘
```

### 性能分析器

```rust
use game_engine::profiling::profiler::Profiler;

let mut profiler = Profiler::new();

// 在游戏循环中
loop {
    profiler.start_frame();
    
    profiler.begin_scope("update");
    // 更新逻辑
    profiler.end_scope("update");
    
    profiler.begin_scope("render");
    // 渲染逻辑
    profiler.end_scope("render");
    
    profiler.end_frame();
    
    // 获取报告
    let report = profiler.get_report();
    println!("Frame time: {:.2}ms", report.total_frame_time);
}
```

---

## 完整游戏示例

### 基础游戏结构

```rust
use game_engine::core::Engine;
use game_engine::ecs::{World, Component, System, Query};

#[derive(Component)]
struct Player {
    health: f32,
    speed: f32,
}

#[derive(Component)]
struct Enemy {
    health: f32,
}

fn player_movement_system(
    mut query: Query<(&mut Transform, &Player)>,
    input: Res<Input>,
) {
    for (mut transform, player) in query.iter_mut() {
        let mut velocity = Vec2::ZERO;
        
        if input.is_key_pressed(KeyCode::W) {
            velocity.y += player.speed;
        }
        if input.is_key_pressed(KeyCode::S) {
            velocity.y -= player.speed;
        }
        if input.is_key_pressed(KeyCode::A) {
            velocity.x -= player.speed;
        }
        if input.is_key_pressed(KeyCode::D) {
            velocity.x += player.speed;
        }
        
        transform.position += velocity;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::new();
    engine.initialize()?;
    
    // 设置场景
    let world = engine.world_mut();
    world.spawn((
        Transform::default(),
        Player { health: 100.0, speed: 5.0 },
        Sprite { color: [0.0, 1.0, 0.0, 1.0], size: [32.0, 32.0] },
    ));
    
    // 注册系统
    engine.add_system(player_movement_system);
    
    // 运行游戏
    engine.run()?;
    
    Ok(())
}
```

### 集成后处理效果

```rust
use game_engine::render::postprocess::{
    PostProcessEffectManager, PostProcessEffect, QualityMode
};

fn setup_postprocess(device: &Device, config: &SurfaceConfiguration) -> PostProcessEffectManager {
    let mut manager = PostProcessEffectManager::new(device, config);
    
    // 根据平台设置质量
    #[cfg(target_arch = "wasm32")]
    manager.set_quality_mode(QualityMode::Low);
    
    #[cfg(not(target_arch = "wasm32"))]
    manager.set_quality_mode(QualityMode::High);
    
    // 添加效果
    manager.add_effect(PostProcessEffect::Bloom {
        intensity: 0.8,
        threshold: 1.0,
        radius: 5.0,
    });
    
    manager.add_effect(PostProcessEffect::Tonemap {
        operator: TonemapOperator::ACES,
        exposure: 1.0,
        gamma: 2.2,
    });
    
    // 启用自适应质量
    manager.set_adaptive_quality(true);
    manager.set_target_frame_time(16.67);
    
    manager
}
```

### 集成资源管理

```rust
use game_engine::resources::UnifiedResourceManager;

async fn load_game_assets(manager: &mut UnifiedResourceManager) -> Result<(), Box<dyn std::error::Error>> {
    // 加载纹理
    let player_texture = manager.load_texture("assets/player.png").await?;
    let enemy_texture = manager.load_texture("assets/enemy.png").await?;
    
    // 加载音频
    let bgm = manager.load_audio("assets/bgm.mp3").await?;
    let hit_sound = manager.load_audio("assets/hit.wav").await?;
    
    // 使用资源
    let player_tex = manager.get_texture(&player_texture)?;
    // ...
    
    Ok(())
}
```

---

## 下一步

- 查看 [API参考](../api_reference.md) 了解详细API
- 阅读 [后处理效果指南](postprocess_api_guide.md) 了解后处理系统
- 阅读 [WASM构建指南](wasm_build_guide.md) 了解Web部署
- 查看 [架构文档](../architecture.md) 了解系统设计
- 查看 [示例代码](../../examples/) 获取更多示例

---

## 常见问题

### Q: 如何选择使用 PostProcessPipeline 还是 PostProcessEffectManager？

**A**: 
- 如果效果配置固定，使用 `PostProcessPipeline`（性能更好）
- 如果需要运行时动态管理效果，使用 `PostProcessEffectManager`（更灵活）

### Q: 资源热重载会影响性能吗？

**A**: 热重载只在开发时使用，生产环境应禁用。文件系统监控有轻微开销，但通常可以忽略。

### Q: WASM版本性能如何？

**A**: WASM版本性能接近原生，但建议：
- 使用发布构建
- 启用SIMD（如果支持）
- 使用内存池减少分配开销
- 优化资源大小

### Q: 如何调试性能问题？

**A**: 
1. 使用性能仪表盘查看实时指标
2. 使用性能分析器定位瓶颈
3. 检查GPU和CPU时间分布
4. 查看内存使用情况

---

## 相关文档

- [API参考](../api_reference.md)
- [后处理效果指南](postprocess_api_guide.md)
- [WASM构建指南](wasm_build_guide.md)
- [资源管理指南](../architecture.md#资源管理)
- [性能调优指南](../performance_tuning_guide.md)

