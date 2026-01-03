//! # ECS World Tests
//!
//! 测试ECS世界系统的基础功能。

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

#[derive(Component)]
struct Health {
    current: u32,
    max: u32,
}

#[test]
fn test_world_creation() {
    let world = World::new();

    assert_eq!(world.entities().len(), 0);
}

#[test]
fn test_world_spawn_entity() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    assert!(world.is_alive(entity.id()));
    assert_eq!(world.entities().len(), 1);
}

#[test]
fn test_world_spawn_with_component() {
    let mut world = World::new();
    let entity = world.spawn(Position { x: 1.0, y: 2.0, z: 3.0 }).id();

    assert!(world.is_alive(entity));

    let position = world.get::<Position>(entity);
    assert!(position.is_some());
    assert_eq!(position.unwrap().x, 1.0);
}

#[test]
fn test_world_spawn_with_multiple_components() {
    let mut world = World::new();

    let entity = world
        .spawn((
            Position { x: 0.0, y: 0.0, z: 0.0 },
            Velocity { x: 1.0, y: 0.0, z: 0.0 },
            Health {
                current: 100,
                max: 100,
            },
        ))
        .id();

    assert!(world.get::<Position>(entity).is_some());
    assert!(world.get::<Velocity>(entity).is_some());
    assert!(world.get::<Health>(entity).is_some());
}

#[test]
fn test_world_despawn_entity() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.despawn(entity);

    assert!(!world.is_alive(entity));
    assert_eq!(world.entities().len(), 0);
}

#[test]
fn test_world_get_component() {
    let mut world = World::new();

    let entity = world
        .spawn(Position { x: 5.0, y: 10.0, z: 15.0 })
        .id();

    let position = world.get::<Position>(entity);

    assert!(position.is_some());
    let pos = position.unwrap();
    assert_eq!(pos.x, 5.0);
    assert_eq!(pos.y, 10.0);
    assert_eq!(pos.z, 15.0);
}

#[test]
fn test_world_get_component_mut() {
    let mut world = World::new();

    let entity = world
        .spawn(Position { x: 1.0, y: 2.0, z: 3.0 })
        .id();

    let mut position = world.get_mut::<Position>(entity).unwrap();
    position.x = 10.0;

    let position = world.get::<Position>(entity).unwrap();
    assert_eq!(position.x, 10.0);
}

#[test]
fn test_world_query_empty() {
    let mut world = World::new();

    let mut query = world.query::<&Position>();
    let results: Vec<&Position> = query.iter(&world).collect();

    assert!(results.is_empty());
}

#[test]
fn test_world_query_single_component() {
    let mut world = World::new();

    world.spawn(Position { x: 1.0, y: 2.0, z: 3.0 });
    world.spawn(Position { x: 4.0, y: 5.0, z: 6.0 });
    world.spawn(Position { x: 7.0, y: 8.0, z: 9.0 });

    let mut query = world.query::<&Position>();
    let results: Vec<&Position> = query.iter(&world).collect();

    assert_eq!(results.len(), 3);
}

#[test]
fn test_world_query_multiple_components() {
    let mut world = World::new();

    world.spawn((
        Position { x: 0.0, y: 0.0, z: 0.0 },
        Velocity { x: 1.0, y: 0.0, z: 0.0 },
    ));
    world.spawn((
        Position { x: 1.0, y: 0.0, z: 0.0 },
        Velocity { x: 0.0, y: 1.0, z: 0.0 },
    ));

    let mut query = world.query::<(&Position, &Velocity)>();
    let results: Vec<(&Position, &Velocity)> = query.iter(&world).collect();

    assert_eq!(results.len(), 2);
}

#[test]
fn test_world_query_filtered() {
    let mut world = World::new();

    world.spawn((
        Position { x: 0.0, y: 0.0, z: 0.0 },
        Velocity { x: 1.0, y: 0.0, z: 0.0 },
    ));
    world.spawn((
        Position { x: 1.0, y: 0.0, z: 0.0 },
        Velocity { x: 0.0, y: 1.0, z: 0.0 },
    ));
    world.spawn(Position { x: 2.0, y: 0.0, z: 0.0 }); // 没有Velocity组件

    let mut query = world.query::<(&Position, &Velocity)>();
    let results: Vec<(&Position, &Velocity)> = query.iter(&world).collect();

    assert_eq!(results.len(), 2); // 只有2个实体同时有Position和Velocity
}

#[test]
fn test_world_remove_component() {
    let mut world = World::new();

    let entity = world
        .spawn((
            Position { x: 1.0, y: 2.0, z: 3.0 },
            Velocity { x: 0.0, y: 0.0, z: 0.0 },
        ))
        .id();

    let entity_mut = world.entity_mut(entity);
    entity_mut.remove::<Velocity>();

    assert!(world.get::<Position>(entity).is_some());
    assert!(!world.get::<Velocity>(entity).is_some());
}

#[test]
fn test_world_clear() {
    let mut world = World::new();

    world.spawn(Position { x: 1.0, y: 2.0, z: 3.0 });
    world.spawn(Position { x: 4.0, y: 5.0, z: 6.0 });

    world.clear();

    assert_eq!(world.entities().len(), 0);
}
