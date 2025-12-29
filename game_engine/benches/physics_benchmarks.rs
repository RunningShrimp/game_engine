// 物理系统性能基准测试
//
// 测试物理模拟、碰撞检测、空间查询等核心物理功能

use bevy_ecs::prelude::*;
use criterion::{black_box, BenchmarkId, Criterion, criterion_group, criterion_main};
use game_engine::physics::physics3d::*;
use glam::{Quat, Vec3};
use rapier3d::prelude::*;

fn create_physics_world_with_bodies(count: usize) -> (World, Vec<Entity>) {
    let mut world = World::new();
    let mut physics_world = PhysicsWorld3D::new();
    let mut entities = Vec::with_capacity(count);

    for i in 0..count {
        // Create rigid body
        let rigid_body = RigidBodyBuilder::new(RigidBodyType::Dynamic)
            .translation(vector![i as f32 * 2.0, 10.0, 0.0])
            .build();

        let handle = physics_world.rigid_body_set.insert(rigid_body);

        // Create collider
        let collider = ColliderBuilder::ball(0.5).build();
        physics_world.collider_set.insert_with_parent(
            collider,
            handle,
            &mut physics_world.rigid_body_set,
        );

        // Create entity
        let entity = world.spawn(RigidBody3D { handle }).id();
        entities.push(entity);
    }

    // Add ground
    let ground_body = RigidBodyBuilder::new(RigidBodyType::Fixed)
        .translation(vector![0.0, -1.0, 0.0])
        .build();
    physics_world.rigid_body_set.insert(ground_body);

    let ground_collider = ColliderBuilder::halfspace(vector![0.0, 1.0, 0.0], 0.0).build();
    physics_world.collider_set.insert(ground_collider);

    world.insert_resource(physics_world);
    (world, entities)
}

/// Benchmark physics stepping with different numbers of bodies
fn bench_physics_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_step");

    for body_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(body_count), body_count, |b, &count| {
            let (mut world, _) = create_physics_world_with_bodies(count);

            b.iter(|| {
                let mut physics_world = world.resource_mut::<PhysicsWorld3D>();
                physics_world.step();
                black_box(&mut physics_world);
            });
        });
    }

    group.finish();
}

/// Benchmark collision detection performance
fn bench_collision_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("collision_detection");

    for body_count in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(body_count), body_count, |b, &count| {
            let (mut world, _) = create_physics_world_with_bodies(count);

            // Run a physics step to trigger collision detection
            b.iter(|| {
                let mut physics_world = world.resource_mut::<PhysicsWorld3D>();
                physics_world.step();
                black_box(&physics_world.narrow_phase);
            });
        });
    }

    group.finish();
}

/// Benchmark spatial queries (raycasting)
fn bench_spatial_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_query");

    let (world, _) = create_physics_world_with_bodies(100);

    group.bench_function("raycast_100_bodies", |b| {
        let physics_world = world.resource::<PhysicsWorld3D>();

        b.iter(|| {
            let origin = Vec3::new(0.0, 10.0, 0.0);
            let direction = Vec3::new(0.0, -1.0, 0.0);
            let max_distance = 20.0;

            black_box(physics_world.raycast(origin, direction, max_distance));
        });
    });

    group.finish();
}

/// Benchmark rigid body creation
fn bench_rigid_body_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("rigid_body_creation");

    for count in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut rigid_body_set = RigidBodySet::new();
                let mut collider_set = ColliderSet::new();

                for i in 0..count {
                    let rigid_body = RigidBodyBuilder::new(RigidBodyType::Dynamic)
                        .translation(vector![i as f32 * 2.0, 10.0, 0.0])
                        .build();

                    let handle = rigid_body_set.insert(rigid_body);

                    let collider = ColliderBuilder::ball(0.5).build();
                    collider_set.insert_with_parent(collider, handle, &mut rigid_body_set);
                }

                black_box((rigid_body_set, collider_set));
            });
        });
    }

    group.finish();
}

/// Benchmark physics integration with ECS
fn bench_physics_ecs_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_ecs_integration");

    for entity_count in [100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(entity_count), entity_count, |b, &count| {
            b.iter(|| {
                let mut world = World::new();
                let mut physics_world = PhysicsWorld3D::new();

                for i in 0..count {
                    let rigid_body = RigidBodyBuilder::new(RigidBodyType::Dynamic)
                        .translation(vector![i as f32 % 10.0, 10.0, (i as f32 / 10.0).floor()])
                        .build();

                    let handle = physics_world.rigid_body_set.insert(rigid_body);

                    let collider = ColliderBuilder::ball(0.5).build();
                    physics_world.collider_set.insert_with_parent(
                        collider,
                        handle,
                        &mut physics_world.rigid_body_set,
                    );

                    world.spawn((
                        RigidBody3D { handle },
                        Transform {
                            pos: Vec3::new(i as f32, 10.0, 0.0),
                            rot: Quat::IDENTITY,
                            scale: Vec3::ONE,
                        },
                    ));
                }

                world.insert_resource(physics_world);
                black_box(world);
            });
        });
    }

    group.finish();
}

/// Benchmark continuous collision detection (CCD)
fn bench_continuous_collision_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("ccd");

    let body_count = 100;

    group.bench_function("ccd_100_bodies", |b| {
        let mut world = World::new();
        let mut physics_world = PhysicsWorld3D::new();

        for i in 0..body_count {
            let rigid_body = RigidBodyBuilder::new(RigidBodyType::Dynamic)
                .translation(vector![0.0, 10.0 + i as f32, 0.0])
                .ccd_enabled(true)
                .build();

            let handle = physics_world.rigid_body_set.insert(rigid_body);

            let collider = ColliderBuilder::ball(0.5).build();
            physics_world.collider_set.insert_with_parent(
                collider,
                handle,
                &mut physics_world.rigid_body_set,
            );

            world.spawn(RigidBody3D { handle });
        }

        let ground_body = RigidBodyBuilder::new(RigidBodyType::Fixed)
            .translation(vector![0.0, -1.0, 0.0])
            .build();
        physics_world.rigid_body_set.insert(ground_body);

        let ground_collider = ColliderBuilder::halfspace(vector![0.0, 1.0, 0.0], 0.0).build();
        physics_world.collider_set.insert(ground_collider);

        world.insert_resource(physics_world);

        b.iter(|| {
            let mut physics_world = world.resource_mut::<PhysicsWorld3D>();
            physics_world.step();
            black_box(&mut physics_world);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_physics_step,
    bench_collision_detection,
    bench_spatial_query,
    bench_rigid_body_creation,
    bench_physics_ecs_integration,
    bench_continuous_collision_detection
);
criterion_main!(benches);
