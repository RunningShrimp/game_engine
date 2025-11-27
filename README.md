# Game Engine

一个高性能的跨平台2D/3D游戏引擎，使用Rust构建。

## 特性
- 🎮 ECS架构（基于Bevy ECS）
- 🎨 跨平台渲染（wgpu）
- 🔧 物理引擎（Rapier）
- 🎵 音频系统
- 📝 多语言脚本支持
- ⚡ SIMD优化
- 🚀 GPU驱动渲染
- 🧠 NPU加速支持

## 快速开始

### 安装
```bash
git clone https://github.com/username/game_engine
cd game_engine
cargo build --release
```

### 运行示例
```bash
# 硬件优化演示
cargo run --example hardware_optimization

# 配置系统演示
cargo run --example config_system_demo

# 物理演示
cargo run --example physics_demo

# 音频演示
cargo run --example audio_demo
```

## 第一个游戏

```rust
use game_engine::*;

fn main() {
    let mut engine = GameEngine::new().expect("Failed to create engine");
    
    // 创建场景
    let scene = engine.create_scene("main_scene");
    
    // 添加玩家
    let player = scene.spawn_entity();
    player.insert(Transform::position([0.0, 0.0, 0.0]));
    player.insert(Sprite::color([1.0, 0.0, 0.0]));
    
    // 运行游戏
    engine.run();
}
```

## 文档

### 快速开始
- [安装指南](docs/getting-started/installation.md)
- [快速开始](docs/getting-started/quick-start.md)
- [第一个游戏](docs/getting-started/first-game.md)

### 用户指南
- [配置系统](docs/guides/configuration.md)
- [渲染系统](docs/guides/rendering.md)
- [物理系统](docs/guides/physics.md)
- [动画系统](docs/guides/animation.md)

### 架构设计
- [架构概览](docs/architecture/overview.md)
- [ECS设计](docs/architecture/ecs-design.md)
- [渲染管线](docs/architecture/rendering-pipeline.md)
- [性能优化](docs/architecture/performance.md)

### 其他
- [API参考](https://docs.rs/game_engine)
- [实施计划](IMPLEMENTATION_PLAN.md)

## 许可证
MIT OR Apache-2.0

## 贡献
欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

## 性能
- 支持 x86 (SSE2-AVX512) 和 ARM (NEON) SIMD优化
- GPU驱动渲染，支持计算着色器剔除
- NPU加速，支持华为昇腾、苹果神经引擎等
- 自适应硬件配置，最大化性能

## 路线图
查看 [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) 了解详细的开发计划。