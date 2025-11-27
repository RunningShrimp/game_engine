# 快速开始

## 安装

```bash
git clone https://github.com/username/game_engine
cd game_engine
cargo build --release
```

## 运行示例

```bash
# 硬件优化演示
cargo run --example hardware_optimization

# 配置系统演示
cargo run --example config_system_demo

# 物理演示
cargo run --example physics_demo
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