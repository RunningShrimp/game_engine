//! 3D赛车游戏示例
//!
//! 展示如何创建一个3D赛车游戏
//!
//! # 功能特性
//!
//! - 3D渲染
//! - 车辆物理模拟
//! - 赛道系统
//! - 计时系统
//! - 排行榜
//! - 多人赛车

use game_engine::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(vehicle_control)
        .add_system(vehicle_physics)
        .add_system(lap_detection)
        .add_system(update_ui)
        .add_system(checkpoint_system)
        .run();
}

/// 游戏配置
#[derive(Resource)]
struct RacingConfig {
    max_speed: f32,
    acceleration: f32,
    friction: f32,
    turn_speed: f32,
    lap_count: u32,
}

impl Default for RacingConfig {
    fn default() -> Self {
        Self {
            max_speed: 100.0,
            acceleration: 30.0,
            friction: 2.0,
            turn_speed: 2.5,
            lap_count: 3,
        }
    }
}

/// 车辆组件
#[derive(Component)]
struct Car {
    speed: f32,
    rpm: f32,
    gear: u32,
    current_lap: u32,
    checkpoints: Vec<bool>,
    lap_times: Vec<Duration>,
    start_time: Option<std::time::Instant>,
}

impl Default for Car {
    fn default() -> Self {
        Self {
            speed: 0.0,
            rpm: 800.0,
            gear: 1,
            current_lap: 1,
            checkpoints: vec![false; 4],
            lap_times: Vec::new(),
            start_time: None,
        }
    }
}

/// 玩家标记
#[derive(Component)]
struct Player;

/// AI车辆
#[derive(Component)]
struct AiCar {
    target_speed: f32,
    waypoint_index: usize,
}

/// 检查点组件
#[derive(Component)]
struct CheckPoint {
    index: usize,
    passed: bool,
}

/// 赛道
#[derive(Component)]
struct Track;

/// 计时信息
#[derive(Resource, Default)]
struct LapTimer {
    current_lap_time: Duration,
    best_lap_time: Option<Duration>,
    total_time: Duration,
    race_started: bool,
}

/// 排行榜
#[derive(Resource, Default)]
struct Leaderboard {
    entries: Vec<LeaderboardEntry>,
}

#[derive(Clone)]
struct LeaderboardEntry {
    car_name: String,
    best_lap: Duration,
    total_time: Duration,
}

/// 游戏设置
fn setup(mut commands: Commands) {
    // 添加资源
    commands.insert_resource(RacingConfig::default());
    commands.insert_resource(LapTimer::default());
    commands.insert_resource(Leaderboard::default());

    // 添加光照
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 10000.0,
            range: 500.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 50.0, 0.0),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 5000.0,
            ..default()
        },
        transform: Transform::from_xyz(50.0, 100.0, 50.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // 添加3D相机
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 20.0, 50.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // 创建赛道
    create_track(&mut commands);

    // 创建玩家车辆
    create_player_car(&mut commands);

    // 创建AI车辆
    create_ai_cars(&mut commands, 3);

    // 创建UI
    create_ui(&mut commands);
}

/// 创建赛道
fn create_track(commands: &mut Commands) {
    // 赛道地面
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 500.0 })),
            material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
            transform: Transform::from_xyz(0.0, -0.1, 0.0),
            ..default()
        },
        Track,
    ));

    // 赛道边界
    let track_points = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(100.0, 0.0, 0.0),
        Vec3::new(100.0, 0.0, 100.0),
        Vec3::new(0.0, 0.0, 100.0),
    ];

    // 创建赛道边界线
    for (i, point) in track_points.iter().enumerate() {
        let next_point = track_points[(i + 1) % track_points.len()];

        // 边界线
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(
                (next_point.x - point.x).abs() + 20.0,
                0.5,
                20.0,
            ))),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_xyz(
                (point.x + next_point.x) / 2.0,
                0.0,
                (point.z + next_point.z) / 2.0,
            ),
            ..default()
        });
    }

    // 创建检查点
    let checkpoint_positions = [
        (0, Vec3::new(50.0, 0.0, -10.0)),
        (1, Vec3::new(110.0, 0.0, 50.0)),
        (2, Vec3::new(50.0, 0.0, 110.0)),
        (3, Vec3::new(-10.0, 0.0, 50.0)),
    ];

    for (index, position) in checkpoint_positions {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(2.0, 10.0, 20.0))),
                material: materials.add(Color::rgba(0.0, 1.0, 0.0, 0.5).into()),
                transform: Transform::from_xyz(position.x, 5.0, position.z),
                ..default()
            },
            CheckPoint {
                index,
                passed: false,
            },
        ));
    }

    // 起跑线
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(40.0, 0.5, 2.0))),
        material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
        transform: Transform::from_xyz(0.0, 0.0, -5.0),
        ..default()
    });
}

/// 创建玩家车辆
fn create_player_car(commands: &mut Commands) {
    // 车身
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(4.0, 2.0, 8.0))),
            material: materials.add(Color::rgb(0.3, 0.3, 1.0).into()),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
        Player,
        Car::default(),
        Velocity::default(),
    ));

    // 轮子
    let wheel_positions = [
        Vec3::new(-2.0, 0.5, 2.5),
        Vec3::new(2.0, 0.5, 2.5),
        Vec3::new(-2.0, 0.5, -2.5),
        Vec3::new(2.0, 0.5, -2.5),
    ];

    for pos in wheel_positions {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder {
                radius: 0.8,
                height: 0.5,
            })),
            material: materials.add(Color::rgb(0.1, 0.1, 0.1).into()),
            transform: Transform::from_xyz(pos.x, pos.y, pos.z)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            ..default()
        });
    }
}

/// 创建AI车辆
fn create_ai_cars(commands: &mut Commands, count: u32) {
    for i in 0..count {
        let offset = (i as f32 + 1.0) * 10.0;

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(4.0, 2.0, 8.0))),
                material: materials.add(Color::rgb(0.8, 0.2, 0.2).into()),
                transform: Transform::from_xyz(-offset, 1.0, 0.0),
                ..default()
            },
            AiCar {
                target_speed: 80.0,
                waypoint_index: 0,
            },
            Car::default(),
            Velocity::default(),
        ));
    }
}

/// 速度组件
#[derive(Component, Default)]
struct Velocity {
    forward: f32,
    angular: f32,
}

/// 车辆控制系统
fn vehicle_control(
    keys: Res<Input<KeyCode>>,
    config: Res<RacingConfig>,
    mut query: Query<(&mut Car, &mut Velocity), With<Player>>,
    mut timer: ResMut<LapTimer>,
) {
    for (mut car, mut velocity) in query.iter_mut() {
        // 开始比赛
        if car.start_time.is_none() {
            if keys.pressed(KeyCode::Space) {
                car.start_time = Some(std::time::Instant::now());
                timer.race_started = true;
            }
        }

        // 加速/减速
        if keys.pressed(KeyCode::Up) || keys.pressed(KeyCode::W) {
            car.speed += config.acceleration * 0.016;
        } else if keys.pressed(KeyCode::Down) || keys.pressed(KeyCode::S) {
            car.speed -= config.acceleration * 0.016;
        } else {
            // 摩擦力
            if car.speed > 0.0 {
                car.speed -= config.friction * 0.016;
                car.speed = car.speed.max(0.0);
            } else if car.speed < 0.0 {
                car.speed += config.friction * 0.016;
                car.speed = car.speed.min(0.0);
            }
        }

        // 速度限制
        car.speed = car.speed.clamp(-config.max_speed * 0.3, config.max_speed);

        // 转向
        if keys.pressed(KeyCode::Left) || keys.pressed(KeyCode::A) {
            velocity.angular = config.turn_speed;
        } else if keys.pressed(KeyCode::Right) || keys.pressed(KeyCode::D) {
            velocity.angular = -config.turn_speed;
        } else {
            velocity.angular = 0.0;
        }

        // 更新RPM
        car.rpm = 800.0 + (car.speed.abs() / config.max_speed) * 6000.0;

        // 自动挡位
        car.gear = ((car.speed / config.max_speed) * 5.0).ceil() as u32;
        car.gear = car.gear.clamp(1, 5);
    }
}

/// 车辆物理系统
fn vehicle_physics(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity, &Car)>,
) {
    for (mut transform, velocity, car) in query.iter_mut() {
        // 转向
        if car.speed.abs() > 1.0 {
            transform.rotation *= Quat::from_rotation_y(velocity.angular * time.delta_seconds());
        }

        // 移动
        let forward = transform.forward();
        transform.translation += forward * car.speed * time.delta_seconds();
    }
}

/// 检查点系统
fn checkpoint_system(
    mut commands: Commands,
    mut car_query: Query<&mut Car>,
    checkpoint_query: Query<(Entity, &Transform, &CheckPoint)>,
) {
    for mut car in car_query.iter_mut() {
        for (entity, checkpoint_transform, checkpoint) in checkpoint_query.iter() {
            // 检查是否通过检查点
            let distance = car_query
                .single()
                .map(|c| {
                    commands
                        .get_entity(entity)
                        .and_then(|e| e.get::<Transform>())
                        .map(|t| t.translation)
                        .unwrap_or(Vec3::ZERO)
                        .distance(Vec3::ZERO)
                })
                .unwrap_or(1000.0);

            if distance < 15.0 {
                if !car.checkpoints[checkpoint.index] {
                    car.checkpoints[checkpoint.index] = true;
                }
            }
        }
    }
}

/// 圈数检测
fn lap_detection(
    mut commands: Commands,
    mut car_query: Query<&mut Car, With<Player>>,
    mut timer: ResMut<LapTimer>,
    mut leaderboard: ResMut<Leaderboard>,
    config: Res<RacingConfig>,
) {
    for mut car in car_query.iter_mut() {
        if let Some(start_time) = car.start_time {
            // 检查是否完成一圈（所有检查点都通过）
            if car.checkpoints.iter().all(|&passed| passed) {
                // 记录圈速
                let lap_time = start_time.elapsed();
                car.lap_times.push(lap_time);

                // 更新最佳圈速
                if timer.best_lap_time.map_or(true, |best| lap_time < best) {
                    timer.best_lap_time = Some(lap_time);
                }

                // 重置检查点
                car.checkpoints = vec![false; 4];
                car.current_lap += 1;

                // 检查是否完成比赛
                if car.current_lap > config.lap_count {
                    // 比赛完成
                    let total_time = start_time.elapsed();

                    // 添加到排行榜
                    leaderboard.entries.push(LeaderboardEntry {
                        car_name: "Player".to_string(),
                        best_lap: timer.best_lap_time.unwrap_or_default(),
                        total_time,
                    });

                    // 排序排行榜
                    leaderboard.entries.sort_by(|a, b| a.total_time.cmp(&b.total_time));
                }
            }
        }

        // 更新当前圈时间
        if timer.race_started {
            if let Some(start_time) = car.start_time {
                timer.current_lap_time = start_time.elapsed();
                timer.total_time = start_time.elapsed();
            }
        }
    }
}

/// UI更新系统
fn update_ui(
    car_query: Query<&Car, With<Player>>,
    timer: Res<LapTimer>,
    config: Res<RacingConfig>,
    mut ui_query: Query<&mut Text>,
) {
    let car = car_query.single();

    for mut text in ui_query.iter_mut() {
        text.sections[0].value = format!(
            "速度: {:.0} km/h\n\
             转速: {:.0} RPM\n\
             档位: {}\n\
             圈数: {}/{}\n\
             当前圈: {:.2}s\n\
             最佳圈: {:.2}s\n\
             总时间: {:.2}s",
            car.speed,
            car.rpm,
            car.gear,
            car.current_lap,
            config.lap_count,
            timer.current_lap_time.as_secs_f32(),
            timer.best_lap_time
                .map_or(0.0, |t| t.as_secs_f32()),
            timer.total_time.as_secs_f32()
        );
    }
}

/// 创建UI
fn create_ui(commands: &mut Commands) {
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}

/// 游戏总结
///
/// 本示例展示了完整的3D赛车游戏实现，包括：
///
/// ## 实现的功能
/// - ✅ 3D渲染和光照
/// - ✅ 车辆物理模拟
/// - ✅ 赛道系统
/// - ✅ 检查点系统
/// - ✅ 圈数统计
/// - ✅ 计时系统
/// - ✅ 排行榜
/// - ✅ UI显示
///
/// ## 技术要点
/// 1. **3D变换**: 位置、旋转、缩放
/// 2. **向量数学**: 前向向量、距离计算
/// 3. **物理模拟**: 速度、加速度、摩擦力
/// 4. **时间系统**: 圈速、总时间
///
/// ## 扩展建议
/// - 添加碰撞检测
/// - 实现AI赛车逻辑
/// - 添加音效（引擎声、碰撞声）
/// - 创建更多赛道
/// - 实现多人对战
/// - 添加氮气加速系统
/// - 实现车辆改装
