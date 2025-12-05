# 创建第一个场景

本教程将教您如何创建场景并添加实体。

## 步骤 1: 初始化引擎

```rust
use game_engine::core::Engine;
use bevy_ecs::prelude::*;

let mut engine = Engine::new();
engine.initialize()?;
```

## 步骤 2: 创建实体

```rust
use game_engine::ecs::Transform;
use glam::{Vec3, Quat};

let world = engine.world_mut();

// 创建一个立方体
let cube = world.spawn((
    Transform {
        pos: Vec3::new(0.0, 0.0, 0.0),
        rot: Quat::IDENTITY,
        scale: Vec3::ONE,
    },
    // 添加渲染组件
    // ... 
)).id();
```

## 步骤 3: 添加相机

```rust
use game_engine::ecs::{Camera, Projection};

world.spawn((
    Transform {
        pos: Vec3::new(0.0, 5.0, 10.0),
        rot: Quat::from_rotation_x(-0.3),
        scale: Vec3::ONE,
    },
    Camera {
        fov: 60.0,
        near: 0.1,
        far: 1000.0,
    },
    Projection::Perspective,
));
```

## 步骤 4: 运行循环

```rust
loop {
    engine.update()?;
}
```

## 下一步

- 添加光照
- 加载模型
- 添加物理

