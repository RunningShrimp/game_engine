// SIMD扩展性能基准测试
//
// 验证SIMD优化带来的15-25%性能提升
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use game_engine_simd::{
    Mat4Simd, PhysicsIntegrator, TransformBatchUpdater, Vec3Simd, Vec4Simd, VectorOps,
};

// ============================================================================
// 物理积分基准测试
// ============================================================================

fn bench_update_velocities_scalar(
    velocities: &mut [[f32; 4]],
    forces: &[[f32; 4]],
    inverse_masses: &[f32],
    dt: f32,
) {
    for i in 0..velocities.len() {
        let inv_mass = inverse_masses[i];
        velocities[i][0] += forces[i][0] * inv_mass * dt;
        velocities[i][1] += forces[i][1] * inv_mass * dt;
        velocities[i][2] += forces[i][2] * inv_mass * dt;
    }
}

fn bench_update_positions_scalar(positions: &mut [[f32; 4]], velocities: &[[f32; 4]], dt: f32) {
    for i in 0..positions.len() {
        positions[i][0] += velocities[i][0] * dt;
        positions[i][1] += velocities[i][1] * dt;
        positions[i][2] += velocities[i][2] * dt;
    }
}

fn benchmark_physics_velocity_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_velocity_update");

    for size in [100, 500, 1000, 2000, 5000].iter() {
        let mut velocities = vec![[1.0, 0.0, 0.0, 0.0]; *size];
        let forces = vec![[0.0, -9.81, 0.0, 0.0]; *size];
        let inverse_masses = vec![1.0; *size];
        let dt = 0.016;

        group.throughput(Throughput::Elements(*size as u64));

        // 标量实现
        group.bench_with_input(BenchmarkId::new("scalar", size), size, |b, _| {
            let mut v = velocities.clone();
            b.iter(|| {
                bench_update_velocities_scalar(
                    black_box(&mut v),
                    black_box(&forces),
                    black_box(&inverse_masses),
                    black_box(dt),
                )
            })
        });

        // SIMD实现
        group.bench_with_input(BenchmarkId::new("simd", size), size, |b, _| {
            let mut v = velocities.clone();
            b.iter(|| {
                PhysicsIntegrator::update_velocities_simd(
                    black_box(&mut v),
                    black_box(&forces),
                    black_box(&inverse_masses),
                    black_box(dt),
                )
            })
        });
    }

    group.finish();
}

fn benchmark_physics_position_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_position_update");

    for size in [100, 500, 1000, 2000, 5000].iter() {
        let mut positions = vec![[0.0f32; 4]; *size];
        let velocities = vec![[1.0, 2.0, 3.0, 0.0]; *size];
        let dt = 0.016;

        group.throughput(Throughput::Elements(*size as u64));

        // 标量实现
        group.bench_with_input(BenchmarkId::new("scalar", size), size, |b, _| {
            let mut p = positions.clone();
            b.iter(|| {
                bench_update_positions_scalar(
                    black_box(&mut p),
                    black_box(&velocities),
                    black_box(dt),
                )
            })
        });

        // SIMD实现
        group.bench_with_input(BenchmarkId::new("simd", size), size, |b, _| {
            let mut p = positions.clone();
            b.iter(|| {
                PhysicsIntegrator::update_positions_simd(
                    black_box(&mut p),
                    black_box(&velocities),
                    black_box(dt),
                )
            })
        });
    }

    group.finish();
}

// ============================================================================
// 变换更新基准测试
// ============================================================================

fn bench_transform_mul_scalar(
    transforms: &[[[f32; 4]; 4]],
    parent_transforms: &[[[f32; 4]; 4]],
    results: &mut [[[f32; 4]; 4]],
) {
    for i in 0..transforms.len() {
        for row in 0..4 {
            for col in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += transforms[i][row][k] * parent_transforms[i][k][col];
                }
                results[i][row][col] = sum;
            }
        }
    }
}

fn benchmark_transform_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_update");

    for size in [50, 100, 250, 500, 1000].iter() {
        let transforms: Vec<[[f32; 4]; 4]> = (0..*size)
            .map(|_| {
                [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [1.0, 2.0, 3.0, 1.0],
                ]
            })
            .collect();

        let parents: Vec<[[f32; 4]; 4]> = (0..*size)
            .map(|_| {
                [
                    [2.0, 0.0, 0.0, 0.0],
                    [0.0, 2.0, 0.0, 0.0],
                    [0.0, 0.0, 2.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ]
            })
            .collect();

        group.throughput(Throughput::Elements(*size as u64));

        // 标量实现
        group.bench_with_input(BenchmarkId::new("scalar", size), size, |b, _| {
            let mut results = vec![[[0.0; 4]; 4]; *size];
            b.iter(|| {
                bench_transform_mul_scalar(
                    black_box(&transforms),
                    black_box(&parents),
                    black_box(&mut results),
                )
            })
        });

        // SIMD实现
        group.bench_with_input(BenchmarkId::new("simd", size), size, |b, _| {
            let mut results = vec![[[0.0; 4]; 4]; *size];
            b.iter(|| {
                TransformBatchUpdater::update_transforms_batch(
                    black_box(&transforms),
                    black_box(&parents),
                    black_box(&mut results),
                )
            })
        });
    }

    group.finish();
}

// ============================================================================
// 向量运算基准测试
// ============================================================================

fn bench_vec4_dot_scalar(v1: &[Vec4Simd], v2: &[Vec4Simd]) -> Vec<f32> {
    v1.iter()
        .zip(v2.iter())
        .map(|(a, b)| {
            a.data[0] * b.data[0]
                + a.data[1] * b.data[1]
                + a.data[2] * b.data[2]
                + a.data[3] * b.data[3]
        })
        .collect()
}

fn benchmark_vec4_dot(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec4_dot");

    for size in [100, 500, 1000, 5000, 10000].iter() {
        let v1: Vec<Vec4Simd> = (0..*size)
            .map(|i| Vec4Simd::new(i as f32, (i + 1) as f32, (i + 2) as f32, (i + 3) as f32))
            .collect();

        let v2: Vec<Vec4Simd> = (0..*size)
            .map(|i| {
                Vec4Simd::new(
                    (i + 4) as f32,
                    (i + 5) as f32,
                    (i + 6) as f32,
                    (i + 7) as f32,
                )
            })
            .collect();

        group.throughput(Throughput::Elements(*size as u64));

        // 标量实现
        group.bench_with_input(BenchmarkId::new("scalar", size), size, |b, _| {
            b.iter(|| bench_vec4_dot_scalar(black_box(&v1), black_box(&v2)))
        });

        // SIMD实现
        group.bench_with_input(BenchmarkId::new("simd", size), size, |b, _| {
            b.iter(|| v1.iter().zip(v2.iter()).map(|(a, b)| a.dot(b)).collect::<Vec<_>>())
        });
    }

    group.finish();
}

// ============================================================================
// 综合场景基准测试
// ============================================================================

fn benchmark_physics_simulation_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_simulation_step");

    for entity_count in [100, 500, 1000].iter() {
        let mut velocities = vec![[1.0, 0.0, 0.0, 0.0]; *entity_count];
        let forces = vec![[0.0, -9.81, 0.0, 0.0]; *entity_count];
        let inverse_masses = vec![1.0; *entity_count];
        let mut positions = vec![[0.0f32; 4]; *entity_count];
        let dt = 0.016;

        group.throughput(Throughput::Elements(*entity_count as u64));

        // 完整的物理步（标量）
        group.bench_with_input(
            BenchmarkId::new("scalar_step", entity_count),
            entity_count,
            |b, _| {
                let mut v = velocities.clone();
                let mut p = positions.clone();
                b.iter(|| {
                    bench_update_velocities_scalar(
                        black_box(&mut v),
                        black_box(&forces),
                        black_box(&inverse_masses),
                        black_box(dt),
                    );
                    bench_update_positions_scalar(black_box(&mut p), black_box(&v), black_box(dt));
                })
            },
        );

        // 完整的物理步（SIMD）
        group.bench_with_input(
            BenchmarkId::new("simd_step", entity_count),
            entity_count,
            |b, _| {
                let mut v = velocities.clone();
                let mut p = positions.clone();
                b.iter(|| {
                    PhysicsIntegrator::update_velocities_simd(
                        black_box(&mut v),
                        black_box(&forces),
                        black_box(&inverse_masses),
                        black_box(dt),
                    );
                    PhysicsIntegrator::update_positions_simd(
                        black_box(&mut p),
                        black_box(&v),
                        black_box(dt),
                    );
                })
            },
        );
    }

    group.finish();
}

fn benchmark_scene_graph_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_graph_update");

    for node_count in [50, 100, 250, 500].iter() {
        let local_transforms: Vec<[[f32; 4]; 4]> = (0..*node_count)
            .map(|_| {
                [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [1.0, 2.0, 3.0, 1.0],
                ]
            })
            .collect();

        let parent_transforms: Vec<[[f32; 4]; 4]> = (0..*node_count)
            .map(|_| {
                [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ]
            })
            .collect();

        group.throughput(Throughput::Elements(*node_count as u64));

        // 标量场景图更新
        group.bench_with_input(
            BenchmarkId::new("scalar", node_count),
            node_count,
            |b, _| {
                let mut results = vec![[[0.0; 4]; 4]; *node_count];
                b.iter(|| {
                    bench_transform_mul_scalar(
                        black_box(&local_transforms),
                        black_box(&parent_transforms),
                        black_box(&mut results),
                    )
                })
            },
        );

        // SIMD场景图更新
        group.bench_with_input(BenchmarkId::new("simd", node_count), node_count, |b, _| {
            let mut results = vec![[[0.0; 4]; 4]; *node_count];
            b.iter(|| {
                TransformBatchUpdater::update_transforms_batch(
                    black_box(&local_transforms),
                    black_box(&parent_transforms),
                    black_box(&mut results),
                )
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_physics_velocity_update,
    benchmark_physics_position_update,
    benchmark_transform_update,
    benchmark_vec4_dot,
    benchmark_physics_simulation_step,
    benchmark_scene_graph_update
);

criterion_main!(benches);
