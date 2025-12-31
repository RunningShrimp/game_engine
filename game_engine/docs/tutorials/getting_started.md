# 从零到游戏 - 快速入门指南

本指南将引导您从零开始创建您的第一个游戏项目，介绍游戏引擎的核心概念和基本使用方法。

## 目录

1. [环境设置](#环境设置)
2. [创建第一个项目](#创建第一个项目)
3. [核心概念介绍](#核心概念介绍)
4. [创建游戏循环](#创建游戏循环)
5. [添加游戏实体](#添加游戏实体)
6. [运行您的游戏](#运行您的游戏)

## 环境设置

### 系统要求

- **Rust**: 1.75.0 或更高版本
- **操作系统**: Windows 10+, macOS 10.15+, Linux (主流发行版)
- **GPU**: 支持 Vulkan、Metal 或 DirectX 12

### 安装 Rust

如果您还没有安装 Rust，请访问 [rustup.rs](https://rustup.rs/) 或运行：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 验证安装

```bash
rustc --version
cargo --version
```

### 克隆引擎仓库

```bash
git clone https://github.com/your-org/game_engine.git
cd game_engine
```

### 构建引擎

```bash
# 构建调试版本
cargo build

# 构建发布版本（性能优化）
cargo build --release

# 运行测试以确保一切正常
cargo test --workspace
```

## 创建第一个项目

### 1. 创建新的 Cargo 项目

```bash
cargo new my_first_game --bin
cd my_first_game
```

### 2. 添加引擎依赖

编辑 `Cargo.toml`：

```toml
[package]
name = "my_first_game"
version = "0.1.0"
edition = "2021"

[dependencies]
game_engine = { path = "../game_engine" }
```

### 3. 项目结构

```
my_first_game/
├── Cargo.toml
├── assets/          # 游戏资源（纹理、模型、音频等）
│   ├── images/
│   ├── models/
│   └── audio/
└── src/
    ├── main.rs      # 入口点
    ├── game.rs      # 游戏逻辑
    └── systems.rs   # ECS 系统实现
```

## 核心概念介绍

### 什么是游戏引擎？

游戏引擎是提供游戏开发核心功能的软件框架。我们的引擎提供：

- **渲染系统**: 将3D/2D图形绘制到屏幕
- **物理系统**: 模拟碰撞、重力等物理现象
- **音频系统**: 播放音乐和音效
- **ECS架构**: 管理游戏对象和行为
- **资源管理**: 加载和管理游戏资源

### ECS 架构

ECS（Entity-Component-System）是一种数据导向的架构模式：

```
┌──────────────────────────────────────┐
│           实体 (Entity)              │  ← 游戏对象的唯一ID
├──────────────────────────────────────┤
│         组件 (Component)             │  ← 数据（位置、速度、精灵等）
├──────────────────────────────────────┤
│           系统 (System)              │  ← 逻辑（移动、渲染、碰撞等）
└──────────────────────────────────────┘
```

**实体**: 只是ID，不包含数据
**组件**: 纯数据，如位置、速度、精灵
**系统**: 纯逻辑，操作特定组件组合

**示例**:

```rust
use game_engine::ecs::{Component, Entity};
use bevy_ecs::prelude::*;

// 1. 定义组件（数据）
#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Velocity {
    dx: f32,
    dy: f32,
}

// 2. 创建系统（逻辑）
fn movement_system(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.dx;
        pos.y += vel.dy;
    }
}
```

## 创建游戏循环

### 基础游戏循环结构

创建 `src/main.rs`:

```rust
use game_engine::prelude::*;
use game_engine::core::Engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建引擎实例
    let mut engine = Engine::new();

    // 初始化引擎
    engine.initialize().await?;

    // 游戏主循环
    loop {
        // 处理输入
        engine.handle_input();

        // 更新游戏逻辑
        engine.update().await?;

        // 渲染画面
        engine.render().await?;

        // 控制帧率
        engine.frame_limit().await;
    }
}
```

### 游戏生命周期

```text
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  初始化   │ -> │  游戏循环  │ -> │  清理资源  │ -> │   退出   │
└──────────┘    └──────────┘    └──────────┘    └──────────┘
                      │
                      ▼
              ┌───────────────┐
              │ 每帧执行:     │
              │ 1. 处理输入   │
              │ 2. 更新逻辑   │
              │ 3. 渲染画面   │
              │ 4. 帧率控制   │
              └───────────────┘
```

## 添加游戏实体

### 创建玩家实体

创建 `src/game.rs`:

```rust
use bevy_ecs::prelude::*;
use game_engine::ecs::{Position, Sprite, Transform, Velocity};

pub fn spawn_player(commands: &mut Commands) {
    commands.spawn((
        // 位置组件
        Position { x: 0.0, y: 0.0 },
        // 变换组件（包含旋转和缩放）
        Transform::default(),
        // 速度组件
        Velocity { dx: 0.0, dy: 0.0 },
        // 精灵组件（用于渲染）
        Sprite {
            tex_index: 0,
            uv_off: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
        // 自定义标签，用于识别
        Player,
    ));
}

// 玩家标记组件
#[derive(Component)]
pub struct Player;
```

### 创建移动系统

创建 `src/systems.rs`:

```rust
use bevy_ecs::prelude::*;
use game_engine::ecs::{Position, Velocity};
use game_engine::input::KeyCode;

/// 玩家移动系统
pub fn player_movement_system(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    for mut velocity in query.iter_mut() {
        velocity.dx = 0.0;
        velocity.dy = 0.0;

        if keyboard.pressed(KeyCode::W) || keyboard.pressed(KeyCode::Up) {
            velocity.dy = 5.0;
        }
        if keyboard.pressed(KeyCode::S) || keyboard.pressed(KeyCode::Down) {
            velocity.dy = -5.0;
        }
        if keyboard.pressed(KeyCode::A) || keyboard.pressed(KeyCode::Left) {
            velocity.dx = -5.0;
        }
        if keyboard.pressed(KeyCode::D) || keyboard.pressed(KeyCode::Right) {
            velocity.dx = 5.0;
        }
    }
}

/// 物理更新系统
pub fn physics_update_system(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.dx;
        pos.y += vel.dy;
    }
}
```

### 注册系统

更新 `src/main.rs`:

```rust
use game_engine::prelude::*;
use game_engine::core::Engine;

mod game;
mod systems;

use systems::{player_movement_system, physics_update_system};
use game::spawn_player;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::new();

    engine.initialize().await?;

    // 注册游戏系统
    engine.add_system(player_movement_system);
    engine.add_system(physics_update_system);

    // 生成玩家实体
    spawn_player(engine.get_commands());

    // 游戏主循环
    loop {
        engine.handle_input();
        engine.update().await?;
        engine.render().await?;
        engine.frame_limit().await;
    }
}
```

## 运行您的游戏

### 1. 添加资源

创建 `assets/images/` 目录并添加一个简单的纹理（例如 `player.png`）。

### 2. 加载资源

更新 `src/game.rs`:

```rust
use game_engine::resources::manager::ResourceManager;

pub async fn load_resources(resource_manager: &mut ResourceManager) {
    // 加载玩家纹理
    resource_manager
        .load_texture("assets/images/player.png")
        .await
        .expect("Failed to load player texture");
}
```

### 3. 运行游戏

```bash
cargo run
```

### 4. 预期结果

- 一个窗口应该打开
- 您应该看到您的玩家角色
- 使用 WASD 或方向键控制玩家移动

## 下一步

恭喜！您已经创建了第一个游戏。接下来您可以：

1. **阅读更多教程**:
   - [ECS系统深入指南](./ecs_guide.md) - 深入了解ECS架构
   - [渲染系统教程](./rendering_guide.md) - 学习创建复杂的视觉效果

2. **添加更多功能**:
   - 添加敌人实体
   - 实现碰撞检测
   - 添加音效和音乐
   - 创建UI界面

3. **探索示例代码**:
   - 查看 `examples/` 目录中的示例
   - 学习最佳实践

4. **阅读API文档**:
   ```bash
   cargo doc --open
   ```

## 常见问题

### Q: 如何更改窗口大小？

A: 在引擎初始化时设置窗口参数：

```rust
engine.set_window_size(1280, 720).await?;
```

### Q: 如何启用调试模式？

A: 使用 `debug` 特性：

```toml
[dependencies]
game_engine = { path = "../game_engine", features = ["debug-ui"] }
```

### Q: 游戏运行缓慢怎么办？

A: 尝试以下优化：

1. 使用发布模式：`cargo run --release`
2. 启用并行处理：`features = ["parallel"]`
3. 检查性能分析器输出的热点

### Q: 如何打包游戏？

A: 使用 cargo 构建可执行文件：

```bash
# 构建
cargo build --release

# 可执行文件位置
# Windows: target/release/my_first_game.exe
# macOS: target/release/my_first_game
# Linux: target/release/my_first_game
```

## 参考资料

- [引擎API文档](https://docs.rs/game_engine)
- [Bevy ECS文档](https://bevyengine.org/learn/book/getting-started/ecs/)
- [WebGPU指南](https://gpuweb.github.io/gpuweb/)
- [Rust游戏开发](https://www.rust-lang.org/what/game-dev)

## 获取帮助

- 📖 查看 [故障排除指南](../TROUBLESHOOTING_GUIDE.md)
- 💬 加入我们的 [Discord社区](#)
- 🐛 报告问题 [GitHub Issues](https://github.com/your-org/game_engine/issues)

---

**祝您游戏开发愉快！** 🎮
