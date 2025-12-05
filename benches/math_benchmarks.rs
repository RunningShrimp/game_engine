//! 数学运算性能基准测试
//!
//! 测试向量、矩阵、四元数等数学运算的性能

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use glam::{Mat4, Quat, Vec3, Vec4};
// SIMD数学模块已分离到game_engine_simd crate
// use game_engine::performance::simd_math;

fn bench_vec3_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec3_operations");

    let v1 = Vec3::new(1.0, 2.0, 3.0);
    let v2 = Vec3::new(4.0, 5.0, 6.0);

    group.bench_function("add", |b| {
        b.iter(|| black_box(v1 + v2));
    });

    group.bench_function("dot", |b| {
        b.iter(|| black_box(v1.dot(v2)));
    });

    group.bench_function("cross", |b| {
        b.iter(|| black_box(v1.cross(v2)));
    });

    group.bench_function("normalize", |b| {
        b.iter(|| black_box(v1.normalize()));
    });

    group.bench_function("distance", |b| {
        b.iter(|| black_box(v1.distance(v2)));
    });

    group.finish();
}

fn bench_matrix_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_operations");

    let m1 = Mat4::from_scale_rotation_translation(Vec3::ONE, Quat::IDENTITY, Vec3::ZERO);
    let m2 = Mat4::from_scale_rotation_translation(
        Vec3::new(2.0, 2.0, 2.0),
        Quat::from_rotation_z(0.785398), // PI / 4.0
        Vec3::new(1.0, 2.0, 3.0),
    );
    let _v = Vec4::new(1.0, 2.0, 3.0, 1.0);

    group.bench_function("multiply", |b| {
        b.iter(|| black_box(m1 * m2));
    });

    group.bench_function("transform_point3", |b| {
        b.iter(|| black_box(m1.transform_point3(Vec3::new(1.0, 2.0, 3.0))));
    });

    group.bench_function("transform_vector3", |b| {
        b.iter(|| black_box(m1.transform_vector3(Vec3::new(1.0, 2.0, 3.0))));
    });

    group.bench_function("inverse", |b| {
        b.iter(|| black_box(m1.inverse()));
    });

    group.finish();
}

fn bench_quaternion_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("quaternion_operations");

    let q1 = Quat::from_euler(glam::EulerRot::YXZ, 1.0, 2.0, 3.0);
    let q2 = Quat::from_euler(glam::EulerRot::YXZ, 4.0, 5.0, 6.0);
    let v = Vec3::new(1.0, 2.0, 3.0);

    group.bench_function("multiply", |b| {
        b.iter(|| black_box(q1 * q2));
    });

    group.bench_function("rotate_vector3", |b| {
        b.iter(|| black_box(q1 * v));
    });

    group.bench_function("slerp", |b| {
        b.iter(|| black_box(q1.slerp(q2, 0.5)));
    });

    group.bench_function("to_euler", |b| {
        b.iter(|| black_box(q1.to_euler(glam::EulerRot::XYZ)));
    });

    group.finish();
}

fn bench_simd_math(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_math");

    let count = 1000;
    let vectors: Vec<Vec3> = (0..count)
        .map(|i| Vec3::new(i as f32, i as f32 * 2.0, i as f32 * 3.0))
        .collect();

    group.bench_with_input(
        BenchmarkId::new("batch_normalize", count),
        &vectors,
        |b, vecs| {
            b.iter(|| {
                let mut result = Vec::with_capacity(vecs.len());
                for v in vecs {
                    result.push(black_box(v.normalize()));
                }
                result
            });
        },
    );

    group.finish();
}

criterion_group!(
    benches,
    bench_vec3_operations,
    bench_matrix_operations,
    bench_quaternion_operations,
    bench_simd_math
);
criterion_main!(benches);
