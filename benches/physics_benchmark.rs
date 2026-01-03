//! # Physics System Benchmarks
//!
//! 物理系统性能基准测试。

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use game_engine::physics::{RigidBody, ColliderShape};
use glam::Vec3;

fn bench_rigidbody_creation(c: &mut Criterion) {
    c.bench_function("rigidbody creation", |b| {
        b.iter(|| {
            let body = RigidBody::new();
            black_box(body)
        })
    });
}

fn bench_rigidbody_update(c: &mut Criterion) {
    let mut body = RigidBody::new();
    body.set_mass(1.0);
    body.set_position(Vec3::new(0.0, 10.0, 0.0));

    c.bench_function("rigidbody update", |b| {
        b.iter(|| {
            body.apply_force(Vec3::new(0.0, -9.81, 0.0));
            body.update(0.016);
            black_box(&body)
        })
    });
}

fn bench_collider_bounds(c: &mut Criterion) {
    let mut group = c.benchmark_group("collider_bounds");

    let sphere = ColliderShape::Sphere { radius: 1.0 };
    let box_collider = ColliderShape::Box {
        half_extents: Vec3::new(1.0, 2.0, 3.0),
    };

    group.bench_function("sphere_bounds", |b| {
        b.iter(|| black_box(sphere.bounds(Vec3::ZERO)))
    });

    group.bench_function("box_bounds", |b| {
        b.iter(|| black_box(box_collider.bounds(Vec3::ZERO)))
    });

    group.finish();
}

fn bench_collision_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("collision_detection");

    for count in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut world = game_engine::physics::CollisionWorld::new();

            // 创建多个物体
            for i in 0..count {
                let x = (i as f32) * 2.0;
                world.add_body(Vec3::new(x, 0.0, 0.0), ColliderShape::Sphere { radius: 1.0 });
            }

            b.iter(|| black_box(world.detect_collisions()))
        });
    }

    group.finish();
}

fn bench_force_application(c: &mut Criterion) {
    let mut group = c.benchmark_group("force_application");

    group.bench_function("single_force", |b| {
        let mut body = RigidBody::new();
        body.set_mass(1.0);

        b.iter(|| {
            body.apply_force(Vec3::new(10.0, 0.0, 0.0));
            black_box(&body)
        })
    });

    group.bench_function("impulse", |b| {
        let mut body = RigidBody::new();
        body.set_mass(1.0);

        b.iter(|| {
            body.apply_impulse(Vec3::new(5.0, 0.0, 0.0));
            black_box(&body)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_rigidbody_creation,
    bench_rigidbody_update,
    bench_collider_bounds,
    bench_collision_detection,
    bench_force_application
);
criterion_main!(benches);
