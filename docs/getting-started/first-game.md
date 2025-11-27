# 第一个游戏

现在让我们创建一个简单的游戏来熟悉引擎的基本API。本教程将创建一个经典的弹跳球游戏。

## 项目设置

首先创建一个新项目：

```bash
cargo new my_first_game
cd my_first_game
```

在`Cargo.toml`中添加依赖：

```toml
[package]
name = "my_first_game"
version = "0.1.0"
edition = "2021"

[dependencies]
game_engine = { path = "../game_engine" }
glam = "0.24"
rand = "0.8"
```

## 基本结构

创建`src/main.rs`：

```rust
use game_engine::*;
use glam::Vec3;
use rand::Rng;

fn main() {
    // 创建应用
    let mut app = App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(ball_movement)
        .add_system(paddle_movement)
        .add_system(collision_detection);

    // 运行游戏
    app.run();
}
```

## 创建游戏实体

### 游戏设置

```rust
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 创建相机
    commands.spawn(Camera2dBundle::default());

    // 创建球拍
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(120.0, 20.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, -200.0, 0.0)),
            ..default()
        },
        Paddle,
    ));

    // 创建球
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        Ball {
            velocity: Vec2::new(200.0, 200.0),
        },
    ));

    // 创建砖块
    for row in 0..bricks rows {
        for col in 0..bricks per row {
            let brick_pos = Vec3::new(
                col as f32 * brick_width - (bricks_per_row as f32 * brick_width) / 2.0,
                row as f32 * brick_height + 100.0,
                0.0
            );

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(
                            0.5 + 0.5 * rand::thread_rng().gen::<f32>(),
                            0.5 + 0.5 * rand::thread_rng().gen::<f32>(),
                            0.5 + 0.5 * rand::thread_rng().gen::<f32>(),
                        ),
                        custom_size: Some(Vec2::new(brick_width, brick_height)),
                        ..default()
                    },
                    transform: Transform::from_translation(brick_pos),
                    ..default()
                },
                Brick,
            ));
        }
    }
}
```

## 组件定义

```rust
#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball {
    velocity: Vec2,
}

#[derive(Component)]
struct Brick;

const BRICK_WIDTH: f32 = 80.0;
const BRICK_HEIGHT: f32 = 30.0;
const BRICKS_PER_ROW: usize = 10;
const BRICK_ROWS: usize = 5;
```

## 游戏逻辑系统

### 球运动系统

```rust
fn ball_movement(
    time: Res<Time>,
    mut ball_query: Query<(&mut Transform, &mut Ball, &Sprite)>,
    window_query: Query<&Window>,
) {
    let window = window_query.single();
    let window_width = window.width();
    let window_height = window.height();

    for (mut transform, mut ball, sprite) in ball_query.iter_mut() {
        // 更新位置
        transform.translation.x += ball.velocity.x * time.delta_seconds();
        transform.translation.y += ball.velocity.y * time.delta_seconds();

        // 边界碰撞检测
        let ball_size = sprite.custom_size.unwrap_or(Vec2::new(20.0, 20.0));
        let half_width = ball_size.x / 2.0;
        let half_height = ball_size.y / 2.0;

        // 左右边界
        if transform.translation.x - half_width <= -window_width / 2.0
            || transform.translation.x + half_width >= window_width / 2.0 {
            ball.velocity.x = -ball.velocity.x;
        }

        // 顶部边界
        if transform.translation.y + half_height >= window_height / 2.0 {
            ball.velocity.y = -ball.velocity.y;
        }

        // 底部（游戏结束条件）
        if transform.translation.y - half_height <= -window_height / 2.0 {
            // 游戏结束逻辑
            println!("Game Over!");
        }
    }
}
```

### 球拍控制

```rust
fn paddle_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut paddle_query: Query<&mut Transform, With<Paddle>>,
    window_query: Query<&Window>,
) {
    let window = window_query.single();
    let window_width = window.width();

    for mut transform in paddle_query.iter_mut() {
        let mut direction = 0.0;

        if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
            direction -= 1.0;
        }
        if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
            direction += 1.0;
        }

        let paddle_speed = 400.0;
        let new_x = transform.translation.x + direction * paddle_speed * time.delta_seconds();
        let paddle_width = 120.0; // 从之前的代码中获取

        // 保持球拍在屏幕内
        let clamped_x = new_x.clamp(
            -window_width / 2.0 + paddle_width / 2.0,
            window_width / 2.0 - paddle_width / 2.0,
        );

        transform.translation.x = clamped_x;
    }
}
```

### 碰撞检测

```rust
fn collision_detection(
    mut commands: Commands,
    mut ball_query: Query<(&Transform, &Sprite, &mut Ball)>,
    paddle_query: Query<(&Transform, &Sprite), With<Paddle>>,
    brick_query: Query<(Entity, &Transform, &Sprite), With<Brick>>,
) {
    for (ball_transform, ball_sprite, mut ball) in ball_query.iter_mut() {
        let ball_pos = ball_transform.translation.truncate();
        let ball_size = ball_sprite.custom_size.unwrap();

        // 球拍碰撞
        for (paddle_transform, paddle_sprite) in paddle_query.iter() {
            let paddle_pos = paddle_transform.translation.truncate();
            let paddle_size = paddle_sprite.custom_size.unwrap();

            if check_collision(ball_pos, ball_size, paddle_pos, paddle_size) {
                ball.velocity.y = ball.velocity.y.abs(); // 向上反弹
                break;
            }
        }

        // 砖块碰撞
        for (brick_entity, brick_transform, brick_sprite) in brick_query.iter() {
            let brick_pos = brick_transform.translation.truncate();
            let brick_size = brick_sprite.custom_size.unwrap();

            if check_collision(ball_pos, ball_size, brick_pos, brick_size) {
                ball.velocity.y = -ball.velocity.y; // 反向
                commands.entity(brick_entity).despawn(); // 销毁砖块
                break;
            }
        }
    }
}

fn check_collision(pos1: Vec2, size1: Vec2, pos2: Vec2, size2: Vec2) -> bool {
    let half1 = size1 / 2.0;
    let half2 = size2 / 2.0;

    pos1.x - half1.x < pos2.x + half2.x
        && pos1.x + half1.x > pos2.x - half2.x
        && pos1.y - half1.y < pos2.y + half2.y
        && pos1.y + half1.y > pos2.y - half2.y
}
```

## 完整代码

将所有部分组合在一起：

```rust
use game_engine::prelude::*;
use glam::{Vec2, Vec3};

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball {
    velocity: Vec2,
}

#[derive(Component)]
struct Brick;

const BRICK_WIDTH: f32 = 80.0;
const BRICK_HEIGHT: f32 = 30.0;
const BRICKS_PER_ROW: usize = 10;
const BRICK_ROWS: usize = 5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(ball_movement)
        .add_system(paddle_movement)
        .add_system(collision_detection)
        .run();
}

fn setup(mut commands: Commands) {
    // 设置相机、球拍、球和砖块
    // (setup代码见上文)
}

fn ball_movement(/* 参数 */) {
    // 球运动逻辑
    // (ball_movement代码见上文)
}

fn paddle_movement(/* 参数 */) {
    // 球拍控制逻辑
    // (paddle_movement代码见上文)
}

fn collision_detection(/* 参数 */) {
    // 碰撞检测逻辑
    // (collision_detection代码见上文)
}

fn check_collision(/* 参数 */) -> bool {
    // 碰撞检测辅助函数
    // (check_collision代码见上文)
}
```

## 运行游戏

```bash
cargo run
```

使用左/右箭头键或A/D键控制球拍，防止球落到屏幕底部，同时击破所有砖块！

## 下一步

- 添加音效和音乐
- 实现得分系统
- 添加不同的关卡
- 创建菜单系统

有关更多高级功能，请查看[用户指南](guides/)部分。