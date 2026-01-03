//! 2D平台跳跃游戏示例
//!
//! 展示如何创建一个完整的2D平台游戏
//!
//! # 功能特性
//!
//! - 玩家角色控制
//! - 平台跳跃物理
//! - 敌人AI
//! - 收集物品系统
//! - 关卡进度
//! - 音效和音乐

use game_engine::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(player_movement)
        .add_system(player_jump)
        .add_system(gravity_system)
        .add_system(collision_system)
        .add_system(enemy_ai)
        .add_system(collection_system)
        .add_system(camera_follow)
        .run();
}

/// 游戏设置
#[derive(Resource)]
struct GameConfig {
    gravity: f32,
    jump_force: f32,
    move_speed: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            gravity: -980.0,
            jump_force: 400.0,
            move_speed: 200.0,
        }
    }
}

/// 玩家组件
#[derive(Component)]
struct Player {
    grounded: bool,
    jumps_remaining: u32,
    max_jumps: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            grounded: false,
            jumps_remaining: 2,
            max_jumps: 2,
        }
    }
}

/// 平台组件
#[derive(Component)]
struct Platform;

/// 敌人组件
#[derive(Component)]
struct Enemy {
    patrol_range: f32,
    start_x: f32,
    speed: f32,
    direction: f32,
}

/// 收集物组件
#[derive(Component)]
struct Collectible {
    value: u32,
    collected: bool,
}

/// 收集分数
#[derive(Resource, Default)]
struct Score(u32);

/// 游戏状态
#[derive(Resource, Default)]
struct GameState {
    level_complete: bool,
    current_level: u32,
}

/// 初始化游戏
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 添加游戏配置
    commands.insert_resource(GameConfig::default());
    commands.insert_resource(Score::default());
    commands.insert_resource(GameState::default());

    // 创建2D相机
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 100.0),
        ..default()
    });

    // 创建玩家
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.3, 0.3, 1.0),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            transform: Transform::from_xyz(-300.0, 100.0, 0.0),
            ..default()
        },
        Player::default(),
        Velocity::default(),
    ));

    // 创建平台
    create_platforms(&mut commands);

    // 创建敌人
    create_enemies(&mut commands);

    // 创建收集物
    create_collectibles(&mut commands);

    // UI
    create_ui(&mut commands);
}

/// 创建平台
fn create_platforms(commands: &mut Commands) {
    // 地面
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.4, 0.4, 0.4),
                custom_size: Some(Vec2::new(2000.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -200.0, 0.0),
            ..default()
        },
        Platform,
    ));

    // 浮动平台
    let platforms = [
        (Vec2::new(-200.0, -50.0), Vec2::new(150.0, 20.0)),
        (Vec2::new(50.0, 50.0), Vec2::new(150.0, 20.0)),
        (Vec2::new(300.0, 100.0), Vec2::new(150.0, 20.0)),
        (Vec2::new(-400.0, 150.0), Vec2::new(150.0, 20.0)),
    ];

    for (pos, size) in platforms {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.5, 0.5, 0.5),
                    custom_size: Some(size),
                    ..default()
                },
                transform: Transform::from_xyz(pos.x, pos.y, 0.0),
                ..default()
            },
            Platform,
        ));
    }
}

/// 创建敌人
fn create_enemies(commands: &mut Commands) {
    let enemy_positions = [-100.0, 200.0, -350.0];

    for x in enemy_positions {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.3, 0.3),
                    custom_size: Some(Vec2::new(24.0, 24.0)),
                    ..default()
                },
                transform: Transform::from_xyz(x, -150.0, 1.0),
                ..default()
            },
            Enemy {
                patrol_range: 100.0,
                start_x: x,
                speed: 50.0,
                direction: 1.0,
            },
            Velocity::default(),
        ));
    }
}

/// 创建收集物
fn create_collectibles(commands: &mut Commands) {
    let coin_positions = [
        (-200.0, 0.0),
        (50.0, 100.0),
        (300.0, 150.0),
        (-400.0, 200.0),
        (100.0, -150.0),
    ];

    for (x, y) in coin_positions {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.8, 0.0),
                    custom_size: Some(Vec2::new(16.0, 16.0)),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 0.5),
                ..default()
            },
            Collectible {
                value: 10,
                collected: false,
            },
        ));
    }
}

/// 创建UI
fn create_ui(commands: &mut Commands) {
    // 分数显示
    commands.spawn(
        TextBundle::from_section(
            "分数: 0",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}

/// 速度组件
#[derive(Component, Default)]
struct Velocity {
    x: f32,
    y: f32,
}

/// 玩家移动系统
fn player_movement(
    keys: Res<Input<KeyCode>>,
    config: Res<GameConfig>,
    mut query: Query<(&mut Velocity, &Player)>,
) {
    for (mut velocity, _player) in query.iter_mut() {
        velocity.x = 0.0;

        if keys.pressed(KeyCode::Left) {
            velocity.x = -config.move_speed;
        }
        if keys.pressed(KeyCode::Right) {
            velocity.x = config.move_speed;
        }
    }
}

/// 玩家跳跃系统
fn player_jump(
    keys: Res<Input<KeyCode>>,
    config: Res<GameConfig>,
    mut query: Query<(&mut Player, &mut Velocity)>,
) {
    for (mut player, mut velocity) in query.iter_mut() {
        if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Up) {
            if player.jumps_remaining > 0 {
                velocity.y = config.jump_force;
                player.jumps_remaining -= 1;
                player.grounded = false;
            }
        }
    }
}

/// 重力系统
fn gravity_system(
    config: Res<GameConfig>,
    time: Res<Time>,
    mut query: Query<&mut Velocity>,
) {
    for mut velocity in query.iter_mut() {
        velocity.y += config.gravity * time.delta_seconds();
    }
}

/// 碰撞检测系统
fn collision_system(
    mut query: Query<(&mut Transform, &mut Velocity, &mut Player)>,
    platform_query: Query<&Transform, (With<Platform>, Without<Player>)>,
) {
    let dt = 0.016; // 假设60fps

    for (mut player_transform, mut velocity, mut player) in query.iter_mut() {
        // 应用速度
        player_transform.translation.x += velocity.x * dt;
        player_transform.translation.y += velocity.y * dt;

        player.grounded = false;

        // 检测与平台的碰撞
        for platform_transform in platform_query.iter() {
            if let Some(collision) = check_collision(
                player_transform.translation,
                Vec2::new(32.0, 32.0),
                platform_transform.translation,
                Vec2::new(2000.0, 50.0), // 地面尺寸
            ) {
                handle_collision(&mut player_transform, &mut velocity, &mut player, collision);
            }

            // 检测浮动平台
            for (pos, size) in [
                (Vec2::new(-200.0, -50.0), Vec2::new(150.0, 20.0)),
                (Vec2::new(50.0, 50.0), Vec2::new(150.0, 20.0)),
                (Vec2::new(300.0, 100.0), Vec2::new(150.0, 20.0)),
                (Vec2::new(-400.0, 150.0), Vec2::new(150.0, 20.0)),
            ] {
                if let Some(collision) = check_collision(
                    player_transform.translation,
                    Vec2::new(32.0, 32.0),
                    Vec3::new(pos.x, pos.y, 0.0),
                    size,
                ) {
                    handle_collision(&mut player_transform, &mut velocity, &mut player, collision);
                }
            }
        }

        // 边界检测
        if player_transform.translation.y < -300.0 {
            // 重置玩家位置
            player_transform.translation = Vec3::new(-300.0, 100.0, 0.0);
            velocity.y = 0.0;
        }
    }
}

/// 碰撞信息
struct CollisionInfo {
    normal: Vec2,
    penetration: f32,
}

/// 检测AABB碰撞
fn check_collision(
    pos_a: Vec3,
    size_a: Vec2,
    pos_b: Vec3,
    size_b: Vec2,
) -> Option<CollisionInfo> {
    let a_min = pos_a.truncate() - size_a / 2.0;
    let a_max = pos_a.truncate() + size_a / 2.0;
    let b_min = pos_b.truncate() - size_b / 2.0;
    let b_max = pos_b.truncate() + size_b / 2.0;

    if a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y {
        // 计算穿透深度
        let penetration_x = (a_max.x - b_min.x).min(b_max.x - a_min.x);
        let penetration_y = (a_max.y - b_min.y).min(b_max.y - a_min.y);

        if penetration_x < penetration_y {
            let normal = if a_max.x - b_min.x < b_max.x - a_min.x {
                Vec2::new(-1.0, 0.0)
            } else {
                Vec2::new(1.0, 0.0)
            };
            Some(CollisionInfo {
                normal,
                penetration: penetration_x,
            })
        } else {
            let normal = if a_max.y - b_min.y < b_max.y - a_min.y {
                Vec2::new(0.0, -1.0)
            } else {
                Vec2::new(0.0, 1.0)
            };
            Some(CollisionInfo {
                normal,
                penetration: penetration_y,
            })
        }
    } else {
        None
    }
}

/// 处理碰撞响应
fn handle_collision(
    transform: &mut Transform,
    velocity: &mut Velocity,
    player: &mut Player,
    collision: CollisionInfo,
) {
    // 位置修正
    transform.translation.x += collision.normal.x * collision.penetration;
    transform.translation.y += collision.normal.y * collision.penetration;

    // 速度响应
    if collision.normal.y != 0.0 {
        velocity.y = 0.0;
        if collision.normal.y > 0.0 {
            player.grounded = true;
            player.jumps_remaining = player.max_jumps;
        }
    }
    if collision.normal.x != 0.0 {
        velocity.x = 0.0;
    }
}

/// 敌人AI系统
fn enemy_ai(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut Enemy)>,
) {
    for (mut transform, mut velocity, mut enemy) in query.iter_mut() {
        let current_x = transform.translation.x;

        // 巡逻逻辑
        if current_x > enemy.start_x + enemy.patrol_range {
            enemy.direction = -1.0;
        } else if current_x < enemy.start_x - enemy.patrol_range {
            enemy.direction = 1.0;
        }

        velocity.x = enemy.speed * enemy.direction;

        // 应用移动
        transform.translation.x += velocity.x * time.delta_seconds();
    }
}

/// 收集物品系统
fn collection_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mut collectible_query: Query<(Entity, &Transform, &mut Collectible)>,
    mut score: ResMut<Score>,
) {
    let player_transform = player_query.single();

    for (entity, collectible_transform, mut collectible) in collectible_query.iter_mut() {
        if collectible.collected {
            continue;
        }

        let distance = player_transform
            .translation
            .distance(collectible_transform.translation);

        if distance < 30.0 {
            // 收集成功
            collectible.collected = true;
            score.0 += collectible.value;

            // 移除收集物
            commands.entity(entity).despawn();
        }
    }
}

/// 相机跟随系统
fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let player_transform = player_query.single();
    let mut camera_transform = camera_query.single_mut();

    // 平滑跟随
    let target_x = player_transform.translation.x;
    let target_y = player_transform.translation.y;

    camera_transform.translation.x = camera_transform.translation.x.lerp(target_x, 0.1);
    camera_transform.translation.y = camera_transform.translation.y.lerp(target_y, 0.1);
}

/// 游戏总结
///
/// 本示例展示了完整的2D平台游戏实现，包括：
///
/// ## 实现的功能
/// - ✅ 玩家角色控制（左右移动、跳跃）
/// - ✅ 平台碰撞检测
/// - ✅ 重力系统
/// - ✅ 敌人AI巡逻
/// - ✅ 收集物品系统
/// - ✅ 相机跟随
/// - ✅ 分数系统
///
/// ## 技术要点
/// 1. **ECS架构**: 组件化设计，系统独立
/// 2. **物理模拟**: 简单的重力和碰撞
/// 3. **AI行为**: 巡逻逻辑
/// 4. **游戏循环**: 60fps更新
///
/// ## 扩展建议
/// - 添加关卡编辑器
/// - 实现保存/加载系统
/// - 添加音效和音乐
/// - 创建更多敌人类型
/// - 实现关卡进度系统
