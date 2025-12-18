//! ECS系统性能基准测试
//!
//! 测试实体创建、组件添加、系统执行等ECS操作的性能

use bevy_ecs::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use game_engine::ecs::{Sprite, Transform, Velocity};
use glam::{Quat, Vec3};

#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Rotation(Quat);

#[derive(Component)]
struct Health(f32);

fn system_rotate_entities(mut query: Query<&mut Transform>) {
    for mut transform in query.iter_mut() {
        transform.rot = transform.rot * Quat::from_rotation_z(0.01);
    }
}

fn system_update_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.pos += velocity.lin;
    }
}

fn bench_spawn_entities(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_entities");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut world = World::new();
                for _ in 0..count {
                    world.spawn((Transform::default(), Velocity::default(), Sprite::default()));
                }
                black_box(world)
            });
        });
    }

    group.finish();
}

fn bench_add_components(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_components");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut world = World::new();
            let entities: Vec<Entity> = (0..count)
                .map(|_| world.spawn(Transform::default()).id())
                .collect();

            b.iter(|| {
                for entity in &entities {
                    world.entity_mut(*entity).insert((
                        Velocity::default(),
                        Sprite::default(),
                        Health(100.0),
                    ));
                }
            });
        });
    }

    group.finish();
}

fn bench_query_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_iteration");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut world = World::new();
            for _ in 0..count {
                world.spawn((Transform::default(), Velocity::default(), Sprite::default()));
            }

            let mut system = IntoSystem::into_system(system_rotate_entities);
            system.initialize(&mut world);

            b.iter(|| {
                system.run((), &mut world);
            });
        });
    }

    group.finish();
}

fn bench_query_with_multiple_components(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_multiple_components");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut world = World::new();
            for _ in 0..count {
                world.spawn((
                    Transform::default(),
                    Velocity::default(),
                    Sprite::default(),
                    Health(100.0),
                ));
            }

            let mut system = IntoSystem::into_system(system_update_velocity);
            system.initialize(&mut world);

            b.iter(|| {
                system.run((), &mut world);
            });
        });
    }

    group.finish();
}

fn bench_schedule_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("schedule_execution");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut world = World::new();
            for _ in 0..count {
                world.spawn((Transform::default(), Velocity::default(), Sprite::default()));
            }

            let mut schedule = Schedule::default();
            schedule.add_systems((system_rotate_entities, system_update_velocity));

            b.iter(|| {
                schedule.run(&mut world);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_spawn_entities,
    bench_add_components,
    bench_query_iteration,
    bench_query_with_multiple_components,
    bench_schedule_execution
);
criterion_main!(benches);
