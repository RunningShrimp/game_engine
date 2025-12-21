//  SIMD性能测试和功能验证
// 
//  测试SIMD优化模块的性能提升和功能正确性

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use game_engine_simd::{
    BoundingVolumeOps, GeometryOps, Mat4Simd, MatrixBatchOps, QuatSimd, SimdBackend, Vec3Simd,
    Vec4Simd, VectorBatchOps, detect_cpu_features,
};
use glam::{Mat4, Quat, Vec3};

/// 测试SIMD向量操作性能
fn bench_simd_vector_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_vector_operations");

    let vectors: Vec<Vec3> = (0..1000)
        .map(|i| Vec3::new(i as f32, i as f32 * 2.0, i as f32 * 3.0))
        .collect();

    // 标准实现基准
    group.bench_with_input(
        BenchmarkId::new("standard_dot_product", vectors.len()),
        &vectors,
        |b, vecs| {
            b.iter(|| {
                let mut sum = 0.0f32;
                for i in 0..vecs.len() - 1 {
                    sum += black_box(vecs[i].dot(vecs[i + 1]));
                }
                black_box(sum)
            });
        },
    );

    // SIMD实现基准
    group.bench_with_input(
        BenchmarkId::new("simd_dot_product", vectors.len()),
        &vectors,
        |b, vecs| {
            b.iter(|| {
                let mut sum = 0.0f32;
                for i in 0..vecs.len() - 1 {
                    let a = Vec3Simd::new(vecs[i].x, vecs[i].y, vecs[i].z);
                    let b = Vec3Simd::new(vecs[i + 1].x, vecs[i + 1].y, vecs[i + 1].z);
                    sum += black_box(a.dot(&b));
                }
                black_box(sum)
            });
        },
    );

    // 批量SIMD操作
    group.bench_with_input(
        BenchmarkId::new("batch_dot_product", vectors.len()),
        &vectors,
        |b, vecs| {
            b.iter(|| {
                let mut v1 = Vec::new();
                let mut v2 = Vec::new();
                for i in 0..vecs.len() - 1 {
                    v1.push(vecs[i]);
                    v2.push(vecs[i + 1]);
                }
                black_box(VectorBatchOps::batch_dot_simd(&v1, &v2))
            });
        },
    );

    group.finish();
}

/// 测试SIMD矩阵操作性能
fn bench_simd_matrix_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_matrix_operations");

    let matrix = Mat4::from_scale_rotation_translation(
        Vec3::new(2.0, 3.0, 4.0),
        Quat::from_rotation_y(0.785398),
        Vec3::new(1.0, 2.0, 3.0),
    );

    let vectors: Vec<Vec3> = (0..1000)
        .map(|i| Vec3::new(i as f32, i as f32 * 0.1, i as f32 * 0.2))
        .collect();

    // 标准矩阵变换
    group.bench_with_input(
        BenchmarkId::new("standard_transform", vectors.len()),
        &vectors,
        |b, vecs| {
            b.iter(|| {
                let mut results = Vec::with_capacity(vecs.len());
                for &v in vecs.iter() {
                    results.push(black_box(matrix.transform_point3(v)));
                }
                black_box(results)
            });
        },
    );

    // SIMD批量变换
    group.bench_with_input(
        BenchmarkId::new("simd_batch_transform", vectors.len()),
        &vectors,
        |b, vecs| {
            b.iter(|| {
                black_box(MatrixBatchOps::batch_transform_vec3_optimized(
                    &matrix, vecs,
                ))
            });
        },
    );

    // 优化的批量变换
    group.bench_with_input(
        BenchmarkId::new("optimized_batch_transform", vectors.len()),
        &vectors,
        |b, vecs| {
            b.iter(|| {
                black_box(MatrixBatchOps::batch_transform_vec3_optimized(
                    &matrix, vecs,
                ))
            });
        },
    );

    group.finish();
}

/// 测试SIMD几何操作性能
fn bench_simd_geometry_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_geometry_operations");

    let points: Vec<Vec3> = (0..1000)
        .map(|i| {
            Vec3::new(
                (i as f32 * 0.1).sin() * 100.0,
                (i as f32 * 0.1).cos() * 100.0,
                (i as f32 * 0.1).tan() * 10.0,
            )
        })
        .collect();

    let plane_normal = Vec3::new(0.0, 1.0, 0.0);
    let plane_d = 0.0;

    // 标准点到平面距离
    group.bench_with_input(
        BenchmarkId::new("standard_point_plane_distance", points.len()),
        &points,
        |b, points| {
            b.iter(|| {
                let mut distances = Vec::with_capacity(points.len());
                for &p in points.iter() {
                    distances.push(black_box(plane_normal.dot(p) - plane_d));
                }
                black_box(distances)
            });
        },
    );

    // SIMD批量点到平面距离
    group.bench_with_input(
        BenchmarkId::new("simd_point_plane_distance", points.len()),
        &points,
        |b, points| {
            b.iter(|| {
                black_box(GeometryOps::batch_point_plane_distance_optimized(
                    points,
                    plane_normal,
                    plane_d,
                ))
            });
        },
    );

    group.finish();
}

/// 测试SIMD包围体计算性能
fn bench_simd_bounding_volume(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_bounding_volume");

    let points: Vec<Vec3> = (0..10000)
        .map(|i| {
            Vec3::new(
                (i as f32 * 0.01).sin() * 50.0,
                (i as f32 * 0.01).cos() * 50.0,
                (i as f32 * 0.01) * 10.0,
            )
        })
        .collect();

    // 标准AABB计算
    group.bench_with_input(
        BenchmarkId::new("standard_aabb", points.len()),
        &points,
        |b, points| {
            b.iter(|| {
                let mut min = points[0];
                let mut max = points[0];
                for &p in points.iter().skip(1) {
                    min.x = min.x.min(p.x);
                    min.y = min.y.min(p.y);
                    min.z = min.z.min(p.z);
                    max.x = max.x.max(p.x);
                    max.y = max.y.max(p.y);
                    max.z = max.z.max(p.z);
                }
                black_box((min, max))
            });
        },
    );

    // SIMD AABB计算
    group.bench_with_input(
        BenchmarkId::new("simd_aabb", points.len()),
        &points,
        |b, points| {
            b.iter(|| black_box(BoundingVolumeOps::batch_compute_aabb(points)));
        },
    );

    group.finish();
}

/// 测试不同SIMD后端的性能差异
fn bench_simd_backends(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_backends");

    let vectors: Vec<Vec3> = (0..1000)
        .map(|i| Vec3::new(i as f32, i as f32 * 2.0, i as f32 * 3.0))
        .collect();

    let backend = SimdBackend::best_available();
    println!("Testing with backend: {:?}", backend);

    // 测试当前最优后端
    group.bench_with_input(
        BenchmarkId::new("optimal_backend", vectors.len()),
        &vectors,
        |b, vecs| {
            b.iter(|| {
                let mut v1 = Vec::new();
                let mut v2 = Vec::new();
                for i in 0..vecs.len() - 1 {
                    v1.push(vecs[i]);
                    v2.push(vecs[i + 1]);
                }
                black_box(VectorBatchOps::batch_dot_simd(&v1, &v2))
            });
        },
    );

    group.finish();
}

/// 功能正确性测试
#[cfg(test)]
mod correctness_tests {
    use super::*;

    #[test]
    fn test_simd_vector_correctness() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);

        let standard_dot = a.dot(b);

        let simd_a = Vec3Simd::new(a.x, a.y, a.z);
        let simd_b = Vec3Simd::new(b.x, b.y, b.z);
        let simd_dot = simd_a.dot(&simd_b);

        assert!((standard_dot - simd_dot).abs() < 1e-6);
    }

    #[test]
    fn test_simd_matrix_correctness() {
        let matrix = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let vector = Vec3::new(4.0, 5.0, 6.0);

        let standard_result = matrix.transform_point3(vector);

        let simd_result = MatrixBatchOps::batch_transform_vec3_optimized(&matrix, &[vector]);

        assert_eq!(simd_result.results.len(), 1);
        let result = simd_result.results[0];

        assert!((standard_result.x - result.x).abs() < 1e-6);
        assert!((standard_result.y - result.y).abs() < 1e-6);
        assert!((standard_result.z - result.z).abs() < 1e-6);
    }

    #[test]
    fn test_simd_aabb_correctness() {
        let points = vec![
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(-0.5, 0.5, -0.5),
        ];

        let (min, max) = BoundingVolumeOps::batch_compute_aabb(&points);

        assert!((min.x - (-1.0)).abs() < 1e-6);
        assert!((min.y - (-1.0)).abs() < 1e-6);
        assert!((min.z - (-1.0)).abs() < 1e-6);

        assert!((max.x - 1.0).abs() < 1e-6);
        assert!((max.y - 1.0).abs() < 1e-6);
        assert!((max.z - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cpu_feature_detection() {
        let features = detect_cpu_features();
        println!("CPU Features: {:#?}", features);

        // 现代CPU应该至少支持基本SIMD
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            assert!(features.sse2 || features.neon);
        }
    }

    #[test]
    fn test_simd_backend_selection() {
        let backend = SimdBackend::best_available();
        println!("Selected SIMD backend: {:?}", backend);

        // 应该选择一个可用的后端
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            assert_ne!(backend, SimdBackend::Scalar);
        }
    }
}

/// 性能回归测试
#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_performance_regression() {
        let vectors: Vec<Vec3> = (0..1000)
            .map(|i| Vec3::new(i as f32, i as f32 * 2.0, i as f32 * 3.0))
            .collect();

        let start = std::time::Instant::now();

        // 使用SIMD批量操作
        let mut v1 = Vec::new();
        let mut v2 = Vec::new();
        for i in 0..vectors.len() - 1 {
            v1.push(vectors[i]);
            v2.push(vectors[i + 1]);
        }
        let _simd_result = VectorBatchOps::batch_dot_simd(&v1, &v2);

        let simd_duration = start.elapsed();

        // 使用标准操作
        let start = std::time::Instant::now();
        let mut sum = 0.0f32;
        for i in 0..vectors.len() - 1 {
            sum += vectors[i].dot(vectors[i + 1]);
        }
        let _standard_result = sum;

        let standard_duration = start.elapsed();

        println!("SIMD duration: {:?}", simd_duration);
        println!("Standard duration: {:?}", standard_duration);

        // SIMD应该更快（在大多数情况下）
        // 注意：在小数据集上，SIMD开销可能使其变慢
        if vectors.len() > 100 {
            assert!(simd_duration < standard_duration);
        }
    }
}

criterion_group!(
    simd_benchmarks,
    bench_simd_vector_operations,
    bench_simd_matrix_operations,
    bench_simd_geometry_operations,
    bench_simd_bounding_volume,
    bench_simd_backends
);

criterion_main!(simd_benchmarks);
