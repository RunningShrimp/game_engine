# 3D平台跳跃游戏示例

**引擎版本**: v0.6.4
**难度**: ⭐⭐ (初级-中级)
**预计时间**: 4-6小时

---

## 项目概述

这是一个简单的3D平台跳跃游戏，展示了游戏引擎的核心功能。

### 游戏特色
- ✅ 3D渲染
- ✅ 玩家移动和跳跃
- ✅ 平台碰撞检测
- ✅ 收集物品
- ✅ 简单UI
- ✅ 音效
- ✅ Python脚本集成

---

## 项目结构

```
platformer_game/
├── src/
│   ├── main.rs              # 游戏入口
│   ├── player.rs            # 玩家控制
│   ├── platform.rs          # 平台系统
│   ├── collectible.rs       # 收集物品
│   └── game_logic.rs        # 游戏逻辑
├── assets/
│   ├── models/              # 3D模型
│   ├── textures/            # 纹理
│   └── sounds/              # 音效
├── scripts/                 # Python脚本
│   └── game_logic.py
└── README.md                # 本文件
```

---

## 快速开始

### 1. 运行游戏

```bash
# 在项目根目录
cd examples/platformer_game
cargo run --release
```

### 2. 控制说明

- **WASD**: 移动
- **Space**: 跳跃
- **Mouse**: 视角控制
- **Esc**: 暂停/退出

---

## 代码示例

### main.rs - 游戏入口

```rust
use game_engine::core::Engine;
use game_engine::render::{PbrMaterial, Mesh3D};
use game_engine::platform::{Input, KeyCode};
use game_engine::scripting::PythonScriptingService;

mod player;
mod platform;
mod collectible;
mod game_logic;

fn main() {
    // 初始化引擎
    let mut engine = Engine::new();

    // 配置窗口
    engine.set_window_title("3D Platformer Game");
    engine.set_window_size(1280, 720);

    // 初始化Python脚本
    let python_service = PythonScriptingService::new(Default::default())
        .expect("Failed to init Python");

    // 创建场景
    setup_scene(&mut engine);

    // 游戏主循环
    engine.run(|state| {
        game_logic::update(state);
    });
}

fn setup_scene(engine: &mut Engine) {
    // 创建玩家
    player::create_player(engine);

    // 创建平台
    platform::create_level(engine);

    // 创建收集物
    collectible::spawn_collectibles(engine);

    // 设置光照
    engine.add_directionalal_light(
        Vec3::new(-0.5, -1.0, -0.5),
        Vec3::new(1.0, 0.9, 0.8),
        1.0
    );
}
```

### player.rs - 玩家控制

```rust
use game_engine::platform::{Input, KeyCode};
use game_engine::physics::{RigidBody, Collider};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub jump_force: f32,
    pub is_grounded: bool,
}

pub fn create_player(engine: &mut Engine) {
    let player_entity = engine.spawn_entity();

    // 添加3D模型
    let mesh = Mesh3D::cube(1.0);
    engine.add_mesh_to_entity(player_entity, mesh);

    // 添加物理
    let body = RigidBody::dynamic()
        .with_mass(1.0)
        .with_position(Vec3::new(0, 2, 0));

    let collider = Collider::box_shape(Vec3::new(1, 2, 1));

    engine.add_component(player_entity, body);
    engine.add_component(player_entity, collider);

    // 添加玩家组件
    let player = Player {
        speed: 5.0,
        jump_force: 8.0,
        is_grounded: false,
    };

    engine.add_component(player_entity, player);
}

pub fn update_player(input: &impl Input, mut query: Query<(&mut Player, &mut Transform)>) {
    for (player, mut transform) in query.iter_mut() {
        // 水平移动
        let mut move_dir = Vec3::ZERO;

        if input.is_key_pressed(KeyCode::A) || input.is_key_pressed(KeyCode::Left) {
            move_dir.x -= 1.0;
        }
        if input.is_key_pressed(KeyCode::D) || input.is_key_pressed(KeyCode::Right) {
            move_dir.x += 1.0;
        }

        if move_dir != Vec3::ZERO {
            move_dir = move_dir.normalize();
            transform.translation += move_dir * player.speed * delta_seconds();
        }

        // 跳跃
        if input.is_key_pressed(KeyCode::Space) && player.is_grounded {
            // 应用跳跃力
            player.is_grounded = false;
        }
    }
}
```

### platform.rs - 平台系统

```rust
use game_engine::physics::Collider;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Platform {
    pub width: f32,
    pub height: f32,
}

pub fn create_level(engine: &mut Engine) {
    // 起始平台
    create_platform(engine, Vec3::new(0, 0, 0), 10.0, 1.0);

    // 第二个平台
    create_platform(engine, Vec3::new(5, 1, -3), 8.0, 1.0);

    // 第三个平台
    create_platform(engine, Vec3::new(12, 2, -6), 6.0, 1.0);

    // 更多平台...
}

fn create_platform(engine: &mut Engine, position: Vec3, width: f32, height: f32) {
    let entity = engine.spawn_entity();

    // 创建平台网格
    let mesh = Mesh3D::box_size(Vec3::new(width, height, 0.5));
    engine.add_mesh_to_entity(entity, mesh);

    // 创建材质
    let material = PbrMaterial {
        base_color: Vec4::new(0.3, 0.6, 0.3, 1.0),
        roughness: 0.8,
        ..Default::default()
    };
    engine.set_material(entity, material);

    // 添加物理
    let body = RigidBody::static().with_position(position);
    let collider = Collider::box_shape(Vec3::new(width, height, 0.5));

    engine.add_component(entity, body);
    engine.add_component(entity, collider);

    // 添加平台组件
    let platform = Platform { width, height };
    engine.add_component(entity, platform);
}
```

### collectible.rs - 收集物品

```rust
use game_engine::physics::{Collider, Sensor};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Collectible {
    pub collected: bool,
    pub value: u32,
}

pub fn spawn_collectibles(engine: &mut Engine) {
    let positions = vec![
        Vec3::new(5, 1.5, 0),
        Vec3::new(12, 2.5, 0),
        Vec3::new(8, 3.5, -6),
    ];

    for pos in positions {
        create_coin(engine, pos);
    }
}

fn create_coin(engine: &mut Engine, position: Vec3) {
    let entity = engine.spawn_entity();

    // 金币模型
    let mesh = Mesh3D::cylinder(0.5, 0.1);
    engine.add_mesh_to_entity(entity, mesh);

    // 金色材质
    let material = PbrMaterial {
        base_color: Vec4::new(1.0, 0.84, 0.0, 1.0),
        metallic: 1.0,
        roughness: 0.3,
        ..Default::default()
    };
    engine.set_material(entity, material);

    // 旋转动画
    engine.add_rotation_animation(entity, 180.0);

    // 碰撞检测（传感器）
    let collider = Sensor::sphere(0.5);
    collider.set_position(position);
    engine.add_component(entity, collider);

    // 收集组件
    let collectible = Collectible {
        collected: false,
        value: 10,
    };
    engine.add_component(entity, collectible);
}

pub fn check_collisions(mut query: Query<(&mut Collectible, &Transform)>) {
    for (mut collectible, transform) in query.iter_mut() {
        if !collectible.collected {
            // 检查是否与玩家碰撞
            if let Some(player_pos) = get_player_position() {
                let distance = transform.translation.distance(player_pos);
                if distance < 1.0 {
                    collectible.collected = true;
                    add_score(collectible.value);
                    play_sound("coin_collect.ogg");
                }
            }
        }
    }
}
```

### game_logic.rs - 游戏逻辑

```rust
use bevy_ecs::prelude::*;

pub struct GameState {
    pub score: u32,
    pub level: u32,
    pub game_over: bool,
}

pub fn update(state: &mut GameState) {
    if state.game_over {
        show_game_over_screen();
        return;
    }

    // 更新游戏
    update_player::update_player(input, &mut query);
    platform::check_falling(&mut query);
    collectible::check_collisions(&mut query);

    // 检查胜利条件
    if all_collectibles_collected() {
        next_level();
    }
}

fn next_level() {
    // 加载下一关
    println!("Level Complete!");
}

fn add_score(points: u32) {
    println!("Score +{}", points);
}
```

---

## Python脚本集成

### scripts/game_logic.py

```python
import game_engine as ge

class GameLogic:
    def __init__(self):
        self.score = 0
        self.level = 1

    def on_coin_collected(self, value):
        self.score += value
        print(f"Coin collected! Score: {self.score}")

        # 检查是否获胜
        if self.score >= 100:
            self.win_game()

    def on_player_fall(self):
        print("Player fell! Game Over.")
        ge.Engine.quit()

    def win_game(self):
        print(f"You win! Final score: {self.score}")
        ge.Time.set_time_scale(0.5)  # 慢动作

# 创建游戏逻辑实例
game = GameLogic()
```

---

## 资源需求

### 3D模型
- 玩家模型: `assets/models/player.glb`
- 平台纹理: `assets/textures/platform.png`
- 金币模型: `assets/models/coin.glb`

### 音效
- 跳跃: `assets/sounds/jump.ogg`
- 收集金币: `assets/sounds/coin_collect.ogg`
- 游戏结束: `assets/sounds/game_over.ogg`

---

## 扩展建议

### 进阶功能
1. ✅ 添加更多关卡
2. ✅ 实现敌人AI
3. ✅ 添加道具系统
4. ✅ 实现存档系统
5. ✅ 添加多人模式

### 性能优化
1. ✅ 使用LOD系统
2. ✅ 实现对象池
3. ✅ 优化碰撞检测
4. ✅ 减少Draw Calls

---

## 故障排除

### 问题: 游戏运行缓慢
**解决方案**:
- 检查是否使用release模式: `cargo run --release`
- 降低阴影质量
- 减少同时渲染的对象数量

### 问题: Python脚本不工作
**解决方案**:
- 确保启用了python feature: `cargo run --features python`
- 检查Python脚本路径是否正确

---

## 学习资源

- **教程索引**: [TUTORIALS_INDEX.md](../../TUTORIALS_INDEX.md)
- **ECS文档**: [TUTORIAL_02_ECS.md](../../TUTORIAL_02_ECS.md)
- **物理文档**: [TUTORIAL_06_PHYSICS.md](../../TUTORIAL_06_PHYSICS.md)

---

## 贡献

欢迎改进这个示例项目！请提交Pull Request。

---

**示例版本**: v1.0
**最后更新**: 2025-12-31
**维护者**: 游戏引擎团队
