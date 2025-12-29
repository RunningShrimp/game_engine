// SoA (Structure of Arrays) Performance Benchmarks
//
// This benchmark suite measures the performance improvements from using SoA layout
// compared to traditional AoS (Array of Structures) layout.
//
// Key Metrics:
// 1. Sequential access time (cache-friendly)
// 2. Batch query performance
// 3. Memory locality (cache hit rate)
// 4. SIMD-friendly operation speed

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use game_engine::domain::physics::{RigidBody, RigidBodyId, RigidBodyType};
use game_engine::domain::soa_storage::RigidBodyStorage;
use glam::{Quat, Vec3};
use bevy_ecs::prelude::Entity;
use std::time::Duration;

// Number of bodies to benchmark
const BODIES_COUNTS: [usize; 5] = [100, 500, 1000, 5000, 10000];

/// Benchmark: Sequential position query (AoS - Array of Structures)
///
/// This simulates the traditional approach where each RigidBody is a struct
/// with all fields interleaved.
fn benchmark_aos_sequential_query(bodies: &[RigidBody]) -> Vec<Vec3> {
    bodies.iter().map(|body| body.position()).collect()
}

/// Benchmark: Sequential position query (SoA - Structure of Arrays)
///
/// This uses the new SoA layout where positions are stored contiguously.
fn benchmark_soa_sequential_query(storage: &RigidBodyStorage, indices: &[usize]) -> Vec<Vec3> {
    storage.get_positions_batch(indices)
}

/// Benchmark: Batch position update (AoS)
///
/// Update positions based on velocities (traditional approach).
fn benchmark_aos_batch_update(bodies: &mut [RigidBody], dt: f32) {
    for body in bodies.iter_mut() {
        if body.body_type() == RigidBodyType::Dynamic {
            let pos = body.position();
            let vel = body.linear_velocity();
            body.set_position(pos + vel * dt);
        }
    }
}

/// Benchmark: Batch position update (SoA)
///
/// Update positions based on velocities (SoA approach).
fn benchmark_soa_batch_update(storage: &mut RigidBodyStorage, dt: f32) {
    storage.update_positions_batch(dt);
}

/// Benchmark: Mass query (AoS)
fn benchmark_aos_mass_query(bodies: &[RigidBody]) -> Vec<f32> {
    bodies.iter().map(|body| body.mass()).collect()
}

/// Benchmark: Mass query (SoA)
fn benchmark_soa_mass_query(storage: &RigidBodyStorage, indices: &[usize]) -> Vec<f32> {
    storage.get_masses_batch(indices)
}

/// Create test bodies (AoS)
fn create_aos_bodies(count: usize) -> Vec<RigidBody> {
    (0..count)
        .map(|i| {
            RigidBody::new(
                RigidBodyId::new(i as u64),
                RigidBodyType::Dynamic,
                Vec3::new(i as f32, 0.0, 0.0),
            )
        })
        .collect()
}

/// Create test bodies (SoA)
fn create_soa_storage(count: usize) -> RigidBodyStorage {
    let mut storage = RigidBodyStorage::with_capacity(count);

    for i in 0..count {
        let entity = Entity::from_raw(i as u32);
        let id = RigidBodyId::new(i as u64);
        let position = Vec3::new(i as f32, 0.0, 0.0);
        let velocity = Vec3::new(1.0, 0.0, 0.0);

        storage.insert(entity, id, position, Quat::IDENTITY, 10.0, RigidBodyType::Dynamic);
        storage.set_velocity(entity, velocity).unwrap();
    }

    storage
}

/// Benchmark sequential position queries
fn bench_sequential_position_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_position_query");

    for &count in &BODIES_COUNTS {
        // Prepare data
        let aos_bodies = create_aos_bodies(count);
        let soa_storage = create_soa_storage(count);
        let indices: Vec<usize> = (0..count).collect();

        // AoS benchmark
        group.bench_with_input(
            BenchmarkId::new("AoS", count),
            &aos_bodies,
            |b, bodies| {
                b.iter(|| black_box(benchmark_aos_sequential_query(black_box(bodies))));
            },
        );

        // SoA benchmark
        group.bench_with_input(
            BenchmarkId::new("SoA", count),
            (&soa_storage, indices),
            |b, (storage, indices)| {
                b.iter(|| black_box(benchmark_soa_sequential_query(black_box(storage), black_box(indices))));
            },
        );
    }

    group.finish();
}

/// Benchmark batch position updates
fn bench_batch_position_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_position_update");

    for &count in &BODIES_COUNTS {
        // Prepare data
        let mut aos_bodies = create_aos_bodies(count);
        let mut soa_storage = create_soa_storage(count);

        // AoS benchmark
        group.bench_with_input(
            BenchmarkId::new("AoS", count),
            &mut aos_bodies,
            |b, bodies| {
                b.iter(|| {
                    let mut bodies_clone = bodies.clone();
                    benchmark_aos_batch_update(&mut bodies_clone, black_box(0.016))
                });
            },
        );

        // SoA benchmark
        group.bench_with_input(
            BenchmarkId::new("SoA", count),
            &mut soa_storage,
            |b, storage| {
                b.iter(|| {
                    let mut storage_clone = storage.clone();
                    benchmark_soa_batch_update(&mut storage_clone, black_box(0.016))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark mass queries
fn bench_mass_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("mass_query");

    for &count in &BODIES_COUNTS {
        // Prepare data
        let aos_bodies = create_aos_bodies(count);
        let soa_storage = create_soa_storage(count);
        let indices: Vec<usize> = (0..count).collect();

        // AoS benchmark
        group.bench_with_input(
            BenchmarkId::new("AoS", count),
            &aos_bodies,
            |b, bodies| {
                b.iter(|| black_box(benchmark_aos_mass_query(black_box(bodies))));
            },
        );

        // SoA benchmark
        group.bench_with_input(
            BenchmarkId::new("SoA", count),
            (&soa_storage, indices),
            |b, (storage, indices)| {
                b.iter(|| black_box(benchmark_soa_mass_query(black_box(storage), black_box(indices))));
            },
        );
    }

    group.finish();
}

/// Benchmark: Random access (simulating worst-case scenario for SoA)
fn bench_random_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_access");

    for &count in &[1000, 5000, 10000] {
        // Prepare data
        let aos_bodies = create_aos_bodies(count);
        let soa_storage = create_soa_storage(count);
        let entities: Vec<Entity> = (0..count).map(|i| Entity::from_raw(i as u32)).collect();
        let indices: Vec<usize> = (0..count).collect();

        // AoS benchmark - random access
        group.bench_with_input(
            BenchmarkId::new("AoS", count),
            &aos_bodies,
            |b, bodies| {
                b.iter(|| {
                    // Random access pattern
                    for i in 0..count {
                        let idx = (i * 7) % count; // Pseudo-random
                        black_box(bodies[idx].position());
                    }
                });
            },
        );

        // SoA benchmark - random access
        group.bench_with_input(
            BenchmarkId::new("SoA", count),
            &soa_storage,
            |b, storage| {
                b.iter(|| {
                    // Random access pattern
                    for i in 0..count {
                        let idx = (i * 7) % count;
                        black_box(storage.get_position(entities[idx]));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Memory allocation pattern
fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    group.throughput(Throughput::Elements(1000));

    // AoS allocation
    group.bench_function("AoS_1000_bodies", |b| {
        b.iter(|| {
            let bodies: Vec<RigidBody> = (0..1000)
                .map(|i| {
                    RigidBody::new(
                        RigidBodyId::new(i),
                        RigidBodyType::Dynamic,
                        Vec3::ZERO,
                    )
                })
                .collect();
            black_box(bodies)
        });
    });

    // SoA allocation
    group.bench_function("SoA_1000_bodies", |b| {
        b.iter(|| {
            let mut storage = RigidBodyStorage::with_capacity(1000);
            for i in 0..1000 {
                let entity = Entity::from_raw(i);
                let id = RigidBodyId::new(i);
                storage.insert(entity, id, Vec3::ZERO, Quat::IDENTITY, 10.0, RigidBodyType::Dynamic);
            }
            black_box(storage)
        });
    });

    group.finish();
}

/// Custom benchmark: Cache hit rate simulation
///
/// This benchmark simulates cache behavior by accessing memory in different patterns.
/// Real cache hit rates would need hardware performance counters, but we can
/// estimate based on access patterns.
fn bench_cache_behavior(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_behavior");

    let count = 10000;
    let iterations = 100;

    // Sequential access (cache-friendly)
    group.bench_function("sequential_access", |b| {
        let storage = create_soa_storage(count);
        let indices: Vec<usize> = (0..count).collect();

        b.iter(|| {
            for _ in 0..iterations {
                black_box(storage.get_positions_batch(&indices));
            }
        });
    });

    // Strided access (cache-unfriendly)
    group.bench_function("strided_access", |b| {
        let storage = create_soa_storage(count);
        let indices: Vec<usize> = (0..count).filter(|&i| i % 8 == 0).collect();

        b.iter(|| {
            for _ in 0..iterations {
                for &idx in &indices {
                    black_box(storage.get_position(Entity::from_raw(idx as u32)));
                }
            }
        });
    });

    // Random access (very cache-unfriendly)
    group.bench_function("random_access", |b| {
        let storage = create_soa_storage(count);
        let entities: Vec<Entity> = (0..count).map(|i| Entity::from_raw(i as u32)).collect();

        b.iter(|| {
            for _ in 0..iterations {
                for i in 0..count {
                    let idx = (i * 7) % count;
                    black_box(storage.get_position(entities[idx]));
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_sequential_position_query,
    bench_batch_position_update,
    bench_mass_query,
    bench_random_access,
    bench_memory_allocation,
    bench_cache_behavior
);

criterion_main!(benches);
