//! # ECS System Tests
//!
//! 测试ECS系统的基础功能。

use bevy_ecs::system::IntoSystem;
use bevy_ecs::world::World;
use glam::Vec3;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component, Clone, Copy)]
struct DeltaTime(f32);

/// 简单的运动系统
fn movement_system(mut query: Query<(&mut Position, &Velocity)>, dt: Res<DeltaTime>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.x * dt.0;
        pos.y += vel.y * dt.0;
        pos.z += vel.z * dt.0;
    }
}

/// 重力系统
fn gravity_system(mut query: Query<&mut Velocity>, dt: Res<DeltaTime>) {
    for mut vel in query.iter_mut() {
        vel.y -= 9.81 * dt.0;
    }
}

#[test]
fn test_system_creation() {
    let system = movement_system.system();
    assert!(true); // 系统创建成功
}

#[test]
fn test_movement_system() {
    let mut world = World::new();

    world.insert_resource(DeltaTime(1.0));

    world.spawn((
        Position { x: 0.0, y: 0.0, z: 0.0 },
        Velocity { x: 1.0, y: 0.0, z: 0.0 },
    ));

    let mut system = movement_system.system();
    system.run(&mut world);

    let mut query = world.query::<&Position>();
    let pos = query.iter(&world).next().unwrap();

    assert_eq!(pos.x, 1.0);
}

#[test]
fn test_gravity_system() {
    let mut world = World::new();

    world.insert_resource(DeltaTime(1.0));

    world.spawn((
        Position { x: 0.0, y: 10.0, z: 0.0 },
        Velocity { x: 0.0, y: 0.0, z: 0.0 },
    ));

    let mut system = gravity_system.system();
    system.run(&mut world);

    let mut query = world.query::<&Velocity>();
    let vel = query.iter(&world).next().unwrap();

    assert_eq!(vel.y, -9.81);
}

#[test]
fn test_system_chain() {
    let mut world = World::new();

    world.insert_resource(DeltaTime(1.0));

    world.spawn((
        Position { x: 0.0, y: 10.0, z: 0.0 },
        Velocity { x: 1.0, y: 0.0, z: 0.0 },
    ));

    // 先应用重力
    let mut gravity = gravity_system.system();
    gravity.run(&mut world);

    // 然后应用运动
    let mut movement = movement_system.system();
    movement.run(&mut world);

    let mut query = world.query::<(&Position, &Velocity)>();
    let (pos, vel) = query.iter(&world).next().unwrap();

    assert_eq!(vel.y, -9.81);
    assert_eq!(pos.x, 1.0); // x速度为1，移动了1单位
    assert_eq!(pos.y, 10.0 - 9.81); // y受重力影响
}

#[test]
fn test_system_multiple_entities() {
    let mut world = World::new();

    world.insert_resource(DeltaTime(1.0));

    world.spawn((
        Position { x: 0.0, y: 0.0, z: 0.0 },
        Velocity { x: 1.0, y: 0.0, z: 0.0 },
    ));

    world.spawn((
        Position { x: 10.0, y: 10.0, z: 10.0 },
        Velocity { x: 0.0, y: -1.0, z: 0.0 },
    ));

    world.spawn((
        Position { x: -5.0, y: 5.0, z: -5.0 },
        Velocity { x: 2.0, y: 2.0, z: 2.0 },
    ));

    let mut system = movement_system.system();
    system.run(&mut world);

    let mut query = world.query::<&Position>();
    let positions: Vec<&Position> = query.iter(&world).collect();

    assert_eq!(positions.len(), 3);
    assert_eq!(positions[0].x, 1.0);
    assert_eq!(positions[1].y, 9.0);
    assert_eq!(positions[2].x, -3.0);
}

#[test]
fn test_system_filtered_entities() {
    let mut world = World::new();

    world.insert_resource(DeltaTime(1.0));

    // 有Velocity的实体会被移动
    world.spawn((
        Position { x: 0.0, y: 0.0, z: 0.0 },
        Velocity { x: 1.0, y: 0.0, z: 0.0 },
    ));

    // 没有Velocity的实体不会移动
    world.spawn(Position { x: 10.0, y: 10.0, z: 10.0 });

    let mut system = movement_system.system();
    system.run(&mut world);

    let mut query = world.query::<&Position>();
    let positions: Vec<&Position> = query.iter(&world).collect();

    assert_eq!(positions.len(), 2);

    // 第一个实体应该移动了
    assert_eq!(positions[0].x, 1.0);

    // 第二个实体不应该移动
    assert_eq!(positions[1].x, 10.0);
}

#[test]
fn test_system_variable_dt() {
    let mut world = World::new();

    world.spawn((
        Position { x: 0.0, y: 0.0, z: 0.0 },
        Velocity { x: 10.0, y: 0.0, z: 0.0 },
    ));

    // 不同的时间步长
    for dt in [0.5, 1.0, 2.0] {
        world.insert_resource(DeltaTime(dt));

        let mut system = movement_system.system();
        system.run(&mut world);
    }

    let mut query = world.query::<&Position>();
    let pos = query.iter(&world).next().unwrap();

    // 总位移 = 10 * (0.5 + 1.0 + 2.0) = 35.0
    assert_eq!(pos.x, 35.0);
}
