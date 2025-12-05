# API 快速参考

本文档提供常用 API 的快速参考。

## 引擎初始化

```rust
use game_engine::core::Engine;

// 运行引擎（阻塞）
Engine::run()?;
```

## ECS 系统

### 创建实体

```rust
use game_engine::ecs::Transform;
use bevy_ecs::prelude::*;

let entity = world.spawn((
    Transform {
        pos: glam::Vec3::new(0.0, 0.0, 0.0),
        rot: glam::Quat::IDENTITY,
        scale: glam::Vec3::ONE,
    },
)).id();
```

### 查询组件

```rust
use game_engine::ecs::Transform;
use bevy_ecs::prelude::*;

fn update_system(mut query: Query<&mut Transform>) {
    for mut transform in query.iter_mut() {
        transform.pos += glam::Vec3::new(1.0, 0.0, 0.0);
    }
}
```

## 渲染

### 创建材质

```rust
use game_engine::render::pbr::PbrMaterial;

let material = PbrMaterial {
    base_color: glam::Vec4::new(1.0, 0.0, 0.0, 1.0),
    metallic: 0.5,
    roughness: 0.3,
    ..Default::default()
};
```

### 加载纹理

```rust
use game_engine::render::TextureManager;

let texture = texture_manager.load_texture("texture.png").await?;
```

## 物理

### 创建刚体

```rust
use game_engine::physics::{RigidBodyDesc, ColliderDesc, ShapeType};
use rapier3d::prelude::RigidBodyType;

let rigid_body = RigidBodyDesc {
    body_type: RigidBodyType::Dynamic,
    mass: 1.0,
    ..Default::default()
};

let collider = ColliderDesc {
    shape: ShapeType::Sphere { radius: 0.5 },
    ..Default::default()
};
```

## 动画

### 创建动画剪辑

```rust
use game_engine::animation::{AnimationClip, KeyframeTrack, InterpolationMode};
use glam::Vec3;

let mut clip = AnimationClip::new("walk".to_string(), 5.0);
let mut track = KeyframeTrack::new(InterpolationMode::Linear);
track.add_keyframe(0.0, Vec3::ZERO);
track.add_keyframe(5.0, Vec3::new(10.0, 0.0, 0.0));
clip.add_track("position".to_string(), track);
```

## 网络

### 客户端预测

```rust
use game_engine::network::{NetworkState, NetworkService};

let network_state = NetworkState::default();
let network_service = NetworkService::new();

// 提交输入
network_service.submit_input(input_command)?;

// 应用预测
network_service.apply_prediction(&mut world)?;
```

## XR

### 初始化 XR

```rust
use game_engine::xr::{XrConfig, OpenXrBackend};

let config = XrConfig::default();
let mut xr_backend = OpenXrBackend::new(config)?;

// 开始帧
let frame_state = xr_backend.begin_frame()?;

// 获取视图
let views = xr_backend.locate_views(frame_state.predicted_display_time)?;

// 结束帧
xr_backend.end_frame(&layers)?;
```

### XR 输入

```rust
use game_engine::xr::{XrInputManager, Hand};

let mut input_manager = XrInputManager::new();

// 检查按钮
if input_manager.is_button_pressed(Hand::Right, ControllerButton::TriggerClick) {
    // 处理触发器点击
}

// 获取触发器值
let trigger_value = input_manager.get_trigger_value(Hand::Right);

// 触发震动
input_manager.vibrate(Hand::Right, 0.5, 100_000_000); // 50% 强度，100ms
```

## 性能优化

### 使用 SIMD

```rust
use game_engine::performance::{Vec3Simd, Mat4Simd};

let v1 = Vec3Simd::new(1.0, 2.0, 3.0);
let v2 = Vec3Simd::new(4.0, 5.0, 6.0);
let result = v1 + v2; // SIMD 加速的向量加法
```

### 性能分析

```rust
use game_engine::performance::Profiler;

let mut profiler = Profiler::new();
profiler.start("update_system");
// ... 执行代码 ...
profiler.end("update_system");
profiler.print_stats();
```

## 资源加载

### 异步加载

```rust
use game_engine::resources::coroutine_loader::CoroutineAssetLoader;

let mut loader = CoroutineAssetLoader::new(config);
loader.load_texture("texture.png", priority)?;

// 更新加载器
loader.update(delta_time);

// 检查加载状态
if loader.is_loaded("texture.png") {
    let texture = loader.get_texture("texture.png")?;
}
```

## 编辑器

### 使用快捷键

```rust
use game_engine::editor::{ShortcutManager, ShortcutAction, Modifiers};

let mut shortcuts = ShortcutManager::new();
shortcuts.register_action(ShortcutAction::Undo, Box::new(|| {
    // 执行撤销操作
}));

// 处理输入
if shortcuts.handle_input(Modifiers::ctrl(), "Z") {
    // 快捷键已处理
}
```

