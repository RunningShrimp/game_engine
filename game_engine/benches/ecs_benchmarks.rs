//  ECS系统性能基准测试
//
//  测试实体创建、组件添加、系统执行等ECS操作的性能

use bevy_ecs::prelude::*;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use game_engine::ecs::{Sprite, Transform, Velocity};
use glam::{Quat, Vec3};

#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Rotation(Quat);

#[derive(Component)]
struct Health(f32);

impl Health {
    // 添加方法来访问字段值，形成逻辑闭环
    pub fn value(&self) -> f32 {
        self.0
    }
}

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

// 新增系统：使用 Position 组件
fn system_update_position(mut query: Query<&mut Position>) {
    for mut pos in query.iter_mut() {
        pos.0 += Vec3::new(0.01, 0.0, 0.0);
    }
}

// 新增系统：使用 Rotation 组件
fn system_update_rotation(mut query: Query<&mut Rotation>) {
    for mut rot in query.iter_mut() {
        rot.0 = rot.0 * Quat::from_rotation_z(0.01);
    }
}

// 新增系统：使用 Health 组件
fn system_check_health(query: Query<&Health>) {
    // 检查实体健康状况，形成逻辑闭环
    for health in query.iter() {
        let _ = health.value(); // 使用字段值
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
                std::hint::black_box(world)
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
            let entities: Vec<Entity> =
                (0..count).map(|_| world.spawn(Transform::default()).id()).collect();

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
                let _ = system.run((), &mut world);
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
                let _ = system.run((), &mut world);
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
            schedule.add_systems((
                system_rotate_entities,
                system_update_velocity,
                system_check_health,
            ));

            b.iter(|| {
                let _ = schedule.run(&mut world);
            });
        });
    }

    group.finish();
}

// 新增基准测试：使用 Position 组件
fn bench_custom_components(c: &mut Criterion) {
    let mut group = c.benchmark_group("custom_components");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut world = World::new();
            // 使用 Position 和 Rotation 组件
            for _ in 0..count {
                world.spawn((Position(Vec3::new(0.0, 0.0, 0.0)), Rotation(Quat::IDENTITY)));
            }

            let mut system = IntoSystem::into_system(system_update_position);
            let mut rotation_system = IntoSystem::into_system(system_update_rotation);
            system.initialize(&mut world);
            rotation_system.initialize(&mut world);

            b.iter(|| {
                let _ = system.run((), &mut world);
                let _ = rotation_system.run((), &mut world);
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
    bench_schedule_execution,
    bench_custom_components
);
criterion_main!(benches);
