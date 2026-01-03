//! # ECS System Benchmarks
//!
//! ECS系统性能基准测试。

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use bevy_ecs::world::World;
use bevy_ecs::component::Component;

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

fn bench_entity_spawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_spawn");

    for count in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut world = World::new();
                for _ in 0..count {
                    world.spawn(Position {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    });
                }
                black_box(&world)
            })
        });
    }

    group.finish();
}

fn bench_entity_despawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_despawn");

    for count in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut world = World::new();
                let entities: Vec<_> = (0..count)
                    .map(|_| world.spawn(Position { x: 0.0, y: 0.0, z: 0.0 }).id())
                    .collect();

                for entity in entities {
                    world.despawn(entity);
                }
                black_box(&world)
            })
        });
    }

    group.finish();
}

fn bench_query_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_iteration");

    for count in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut world = World::new();

            for _ in 0..count {
                world.spawn((
                    Position {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    Velocity {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                ));
            }

            b.iter(|| {
                let mut query = world.query::<(&Position, &Velocity)>();
                let count = query.iter(&world).count();
                black_box(count)
            })
        });
    }

    group.finish();
}

fn bench_component_access(c: &mut Criterion) {
    let mut world = World::new();
    let entity = world.spawn(Position { x: 1.0, y: 2.0, z: 3.0 }).id();

    c.bench_function("component_get", |b| {
        b.iter(|| black_box(world.get::<Position>(entity)))
    });
}

fn bench_component_mutate(c: &mut Criterion) {
    let mut world = World::new();
    let entity = world.spawn(Position { x: 1.0, y: 2.0, z: 3.0 }).id();

    c.bench_function("component_mut", |b| {
        b.iter(|| {
            let mut pos = world.get_mut::<Position>(entity).unwrap();
            pos.x += 1.0;
            black_box(&pos)
        })
    });
}

fn bench_add_component(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_component");

    group.bench_function("single_component", |b| {
        b.iter(|| {
            let mut world = World::new();
            let entity = world.spawn_empty().id();

            let mut entity_mut = world.entity_mut(entity);
            entity_mut.insert(Position { x: 0.0, y: 0.0, z: 0.0 });

            black_box(&world)
        })
    });

    group.bench_function("multiple_components", |b| {
        b.iter(|| {
            let mut world = World::new();
            let entity = world.spawn_empty().id();

            let mut entity_mut = world.entity_mut(entity);
            entity_mut.insert((
                Position { x: 0.0, y: 0.0, z: 0.0 },
                Velocity { x: 0.0, y: 0.0, z: 0.0 },
                Health {
                    current: 100,
                    max: 100,
                },
            ));

            black_box(&world)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_entity_spawn,
    bench_entity_despawn,
    bench_query_iteration,
    bench_component_access,
    bench_component_mutate,
    bench_add_component
);
criterion_main!(benches);
