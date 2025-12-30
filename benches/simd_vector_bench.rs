// SIMD向量运算性能基准测试
//
// 验证SIMD优化相比标量实现的性能提升
//
// 运行: cargo bench --bench simd_vector_bench --features simd

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use game_engine_simd::{Vec3Simd, Vec4Simd, VectorOps};
use glam::Vec3;

// ============================================================================
// Vec3 运算基准测试
// ============================================================================

/// Vec3 加法基准测试
fn bench_vec3_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec3_add");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        // 生成测试数据
        let vec_a: Vec<Vec3> = (0..*size)
            .map(|i| Vec3::new(i as f32, (i + 1) as f32, (i + 2) as f32))
            .collect();
        let vec_b: Vec<Vec3> = (0..*size)
            .map(|i| Vec3::new((i + 3) as f32, (i + 4) as f32, (i + 5) as f32))
            .collect();

        // SIMD版本
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("simd", size), size, |bencher, size| {
            bencher.iter(|| {
                let mut results = vec![Vec3::ZERO; *size];
                for i in 0..*size {
                    let a = Vec3Simd {
                        data: vec_a[i].to_array(),
                    };
                    let b = Vec3Simd {
                        data: vec_b[i].to_array(),
                    };
                    let result = a.add(&b);
                    results[i] = Vec3::from_array(result.data);
                }
                black_box(results);
            });
        });

        // 标量版本 (glam)
        group.bench_with_input(
            BenchmarkId::new("scalar_glam", size),
            size,
            |bencher, size| {
                bencher.iter(|| {
                    let mut results = vec![Vec3::ZERO; *size];
                    for i in 0..*size {
                        results[i] = vec_a[i] + vec_b[i];
                    }
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

/// Vec3 点积基准测试
fn bench_vec3_dot(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec3_dot");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        let vec_a: Vec<Vec3> = (0..*size)
            .map(|i| Vec3::new(i as f32, (i + 1) as f32, (i + 2) as f32))
            .collect();
        let vec_b: Vec<Vec3> = (0..*size)
            .map(|i| Vec3::new((i + 3) as f32, (i + 4) as f32, (i + 5) as f32))
            .collect();

        group.throughput(Throughput::Elements(*size as u64));

        // SIMD版本
        group.bench_with_input(BenchmarkId::new("simd", size), size, |bencher, size| {
            bencher.iter(|| {
                let mut results = vec![0.0f32; *size];
                for i in 0..*size {
                    let a = Vec3Simd {
                        data: vec_a[i].to_array(),
                    };
                    let b = Vec3Simd {
                        data: vec_b[i].to_array(),
                    };
                    results[i] = a.dot(&b);
                }
                black_box(results);
            });
        });

        // 标量版本 (glam)
        group.bench_with_input(
            BenchmarkId::new("scalar_glam", size),
            size,
            |bencher, size| {
                bencher.iter(|| {
                    let mut results = vec![0.0f32; *size];
                    for i in 0..*size {
                        results[i] = vec_a[i].dot(vec_b[i]);
                    }
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

/// Vec3 归一化基准测试
fn bench_vec3_normalize(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec3_normalize");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        let vec_a: Vec<Vec3> = (0..*size)
            .map(|i| Vec3::new(i as f32, (i + 1) as f32, (i + 2) as f32))
            .collect();

        group.throughput(Throughput::Elements(*size as u64));

        // SIMD版本
        group.bench_with_input(BenchmarkId::new("simd", size), size, |bencher, size| {
            bencher.iter(|| {
                let mut results = vec![Vec3::ZERO; *size];
                for i in 0..*size {
                    let a = Vec3Simd {
                        data: vec_a[i].to_array(),
                    };
                    let normalized = a.normalize();
                    results[i] = Vec3::from_array(normalized.data);
                }
                black_box(results);
            });
        });

        // 标量版本 (glam)
        group.bench_with_input(
            BenchmarkId::new("scalar_glam", size),
            size,
            |bencher, size| {
                bencher.iter(|| {
                    let mut results = vec![Vec3::ZERO; *size];
                    for i in 0..*size {
                        results[i] = vec_a[i].normalize();
                    }
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

/// Vec3 距离计算基准测试
fn bench_vec3_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec3_distance");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        let vec_a: Vec<Vec3> = (0..*size)
            .map(|_| {
                Vec3::new(
                    rand::random::<f32>() * 100.0,
                    rand::random::<f32>() * 100.0,
                    rand::random::<f32>() * 100.0,
                )
            })
            .collect();
        let target = Vec3::new(50.0, 50.0, 50.0);

        group.throughput(Throughput::Elements(*size as u64));

        // SIMD版本
        group.bench_with_input(BenchmarkId::new("simd", size), size, |bencher, size| {
            bencher.iter(|| {
                let mut results = vec![0.0f32; *size];
                let target_simd = Vec3Simd {
                    data: target.to_array(),
                };
                for i in 0..*size {
                    let a = Vec3Simd {
                        data: vec_a[i].to_array(),
                    };
                    let diff = a.sub(&target_simd);
                    results[i] = diff.length();
                }
                black_box(results);
            });
        });

        // 标量版本 (glam)
        group.bench_with_input(
            BenchmarkId::new("scalar_glam", size),
            size,
            |bencher, size| {
                bencher.iter(|| {
                    let mut results = vec![0.0f32; *size];
                    for i in 0..*size {
                        results[i] = vec_a[i].distance(target);
                    }
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Vec4 运算基准测试
// ============================================================================

/// Vec4 点积基准测试
fn bench_vec4_dot(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec4_dot");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        let vec_a: Vec<[f32; 4]> = (0..*size)
            .map(|i| [i as f32, (i + 1) as f32, (i + 2) as f32, (i + 3) as f32])
            .collect();
        let vec_b: Vec<[f32; 4]> = (0..*size)
            .map(|i| {
                [
                    (i + 4) as f32,
                    (i + 5) as f32,
                    (i + 6) as f32,
                    (i + 7) as f32,
                ]
            })
            .collect();

        group.throughput(Throughput::Elements(*size as u64));

        // SIMD版本
        group.bench_with_input(BenchmarkId::new("simd", size), size, |bencher, size| {
            bencher.iter(|| {
                let mut results = vec![0.0f32; *size];
                for i in 0..*size {
                    let a = Vec4Simd { data: vec_a[i] };
                    let b = Vec4Simd { data: vec_b[i] };
                    results[i] = a.dot(&b);
                }
                black_box(results);
            });
        });

        // 标量版本
        group.bench_with_input(BenchmarkId::new("scalar", size), size, |bencher, size| {
            bencher.iter(|| {
                let mut results = vec![0.0f32; *size];
                for i in 0..*size {
                    results[i] = vec_a[i][0] * vec_b[i][0]
                        + vec_a[i][1] * vec_b[i][1]
                        + vec_a[i][2] * vec_b[i][2]
                        + vec_a[i][3] * vec_b[i][3];
                }
                black_box(results);
            });
        });
    }

    group.finish();
}

// ============================================================================
// 批量操作基准测试 (更接近实际使用场景)
// ============================================================================

/// 批量Vec3加法 - 模拟物理更新场景
fn bench_batch_vec3_add_physics(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_vec3_add_physics");

    for entity_count in [100, 1_000, 10_000].iter() {
        // 模拟物理数据
        let positions: Vec<Vec3> =
            (0..*entity_count).map(|i| Vec3::new(i as f32, i as f32, i as f32)).collect();
        let velocities: Vec<Vec3> = (0..*entity_count).map(|_| Vec3::new(1.0, 2.0, 3.0)).collect();
        let dt = 0.016_f32;

        group.throughput(Throughput::Elements(*entity_count as u64));

        // SIMD版本 (手动展开)
        group.bench_with_input(
            BenchmarkId::new("simd", entity_count),
            entity_count,
            |bencher, count| {
                bencher.iter(|| {
                    let mut new_positions = vec![Vec3::ZERO; *count];
                    for i in 0..*count {
                        let pos_simd = Vec3Simd {
                            data: positions[i].to_array(),
                        };
                        let vel_simd = Vec3Simd {
                            data: velocities[i].to_array(),
                        };
                        let scaled_vel = vel_simd.mul(dt);
                        let result = pos_simd.add(&scaled_vel);
                        new_positions[i] = Vec3::from_array(result.data);
                    }
                    black_box(new_positions);
                });
            },
        );

        // 标量版本 (glam)
        group.bench_with_input(
            BenchmarkId::new("scalar_glam", entity_count),
            entity_count,
            |bencher, count| {
                bencher.iter(|| {
                    let mut new_positions = vec![Vec3::ZERO; *count];
                    for i in 0..*count {
                        new_positions[i] = positions[i] + velocities[i] * dt;
                    }
                    black_box(new_positions);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    name = simd_vector_benches;
    config = Criterion::default().sample_size(100);
    targets =
        bench_vec3_add,
        bench_vec3_dot,
        bench_vec3_normalize,
        bench_vec3_distance,
        bench_vec4_dot,
        bench_batch_vec3_add_physics
);

criterion_main!(simd_vector_benches);
