//! SIMD性能基准测试
//!
//! 测试SIMD优化与标量实现的性能对比

use criterion::{criterion_group, criterion_main, Criterion};
use game_engine_simd::{
    math::{Vec3Simd, Vec4Simd, VectorOps},
    SimdBackend,
    detect_cpu_features,
};

/// 向量运算基准测试
fn bench_vector_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_operations");

    // 准备测试数据
    let a = Vec4Simd::new(1.0, 2.0, 3.0, 4.0);
    let b = Vec4Simd::new(5.0, 6.0, 7.0, 8.0);

    group.bench_function("vec4_add", |bencher| {
        bencher.iter(|| {
            std::hint::black_box(a.add(&b));
        });
    });

    group.bench_function("vec4_mul_scalar", |bencher| {
        bencher.iter(|| {
            std::hint::black_box(a.mul(2.0));
        });
    });

    group.bench_function("vec4_dot", |bencher| {
        bencher.iter(|| {
            std::hint::black_box(a.dot(&b));
        });
    });

    group.bench_function("vec4_normalize", |bencher| {
        bencher.iter(|| {
            std::hint::black_box(a.normalize());
        });
    });

    // Vec3测试
    let c = Vec3Simd::new(1.0, 2.0, 3.0);
    let d = Vec3Simd::new(4.0, 5.0, 6.0);

    group.bench_function("vec3_add", |bencher| {
        bencher.iter(|| {
            std::hint::black_box(c.add(&d));
        });
    });

    group.bench_function("vec3_dot", |bencher| {
        bencher.iter(|| {
            std::hint::black_box(c.dot(&d));
        });
    });

    group.finish();
}

/// 矩阵运算基准测试
fn bench_matrix_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_operations");

    // 创建测试矩阵
    let a = game_engine_simd::math::Mat4Simd::identity();
    let b = game_engine_simd::math::Mat4Simd::identity();

    group.bench_function("mat4_mul", |bencher| {
        bencher.iter(|| {
            std::hint::black_box(a.mul(&b));
        });
    });

    group.finish();
}

/// CPU特性检测基准测试
fn bench_cpu_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_detection");

    group.bench_function("detect_cpu_features", |bencher| {
        bencher.iter(|| {
            std::hint::black_box(detect_cpu_features());
        });
    });

    group.bench_function("best_available_backend", |bencher| {
        bencher.iter(|| {
            std::hint::black_box(SimdBackend::best_available());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_vector_ops,
    bench_matrix_ops,
    bench_cpu_detection
);
criterion_main!(benches);
