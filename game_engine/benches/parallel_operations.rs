// 性能基准测试: 并行操作 vs 串行操作
//
// 用途: 演示使用Rayon进行并行化的性能提升
// 预期: 4-8x 性能提升 (取决于CPU核心数)

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use glam::Vec3;

/// 串行版本: 更新位置
fn update_positions_serial(positions: &mut [Vec3], velocities: &[Vec3], dt: f32) {
    for (i, pos) in positions.iter_mut().enumerate() {
        *pos += velocities[i] * dt;
    }
}

/// 并行版本: 更新位置 (Rayon)
fn update_positions_parallel(positions: &mut [Vec3], velocities: &[Vec3], dt: f32) {
    use rayon::prelude::*;
    positions.par_iter_mut().enumerate().for_each(|(i, pos)| {
        *pos += velocities[i] * dt;
    });
}

/// 串行版本: 向量加法
fn vector_add_serial(a: &[Vec3], b: &[Vec3], result: &mut [Vec3]) {
    for i in 0..a.len() {
        result[i] = a[i] + b[i];
    }
}

/// 并行版本: 向量加法 (Rayon)
fn vector_add_parallel(a: &[Vec3], b: &[Vec3], result: &mut [Vec3]) {
    use rayon::prelude::*;
    a.par_iter()
        .zip(b.par_iter())
        .zip(result.par_iter_mut())
        .for_each(|((ai, bi), res)| {
            *res = *ai + *bi;
        });
}

fn bench_position_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("position_updates");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        let positions = vec![Vec3::ZERO; *size];
        let velocities = vec![Vec3::new(1.0, 2.0, 3.0); *size];

        // 串行版本
        group.bench_with_input(BenchmarkId::new("serial", size), size, |bencher, &_size| {
            bencher.iter(|| {
                let mut pos = positions.clone();
                update_positions_serial(&mut pos, &velocities, 0.016);
                black_box(pos);
            });
        });

        // 并行版本
        group.bench_with_input(
            BenchmarkId::new("parallel", size),
            size,
            |bencher, &_size| {
                bencher.iter(|| {
                    let mut pos = positions.clone();
                    update_positions_parallel(&mut pos, &velocities, 0.016);
                    black_box(pos);
                });
            },
        );
    }

    group.finish();
}

fn bench_vector_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_operations");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        let vec_a = vec![Vec3::X; *size];
        let vec_b = vec![Vec3::Y; *size];

        // 串行版本
        group.bench_with_input(BenchmarkId::new("serial", size), size, |bencher, &_size| {
            bencher.iter(|| {
                let mut result = vec![Vec3::ZERO; *size];
                vector_add_serial(&vec_a, &vec_b, &mut result);
                black_box(result);
            });
        });

        // 并行版本
        group.bench_with_input(
            BenchmarkId::new("parallel", size),
            size,
            |bencher, &_size| {
                bencher.iter(|| {
                    let mut result = vec![Vec3::ZERO; *size];
                    vector_add_parallel(&vec_a, &vec_b, &mut result);
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

// 演示更复杂的计算: 距离计算
fn calculate_distances_serial(positions: &[Vec3], target: Vec3, result: &mut [f32]) {
    for (i, pos) in positions.iter().enumerate() {
        result[i] = pos.distance(target);
    }
}

fn calculate_distances_parallel(positions: &[Vec3], target: Vec3) -> Vec<f32> {
    use rayon::prelude::*;
    positions.par_iter().map(|pos| pos.distance(target)).collect()
}

fn bench_distance_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("distance_calculations");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        let positions = vec![Vec3::new(1.0, 2.0, 3.0); *size];
        let target = Vec3::new(10.0, 20.0, 30.0);

        // 串行版本
        group.bench_with_input(BenchmarkId::new("serial", size), size, |bencher, &_size| {
            bencher.iter(|| {
                let mut result = vec![0.0; *size];
                calculate_distances_serial(&positions, target, &mut result);
                black_box(result);
            });
        });

        // 并行版本
        group.bench_with_input(
            BenchmarkId::new("parallel", size),
            size,
            |bencher, &_size| {
                bencher.iter(|| {
                    let result = calculate_distances_parallel(&positions, target);
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_position_updates,
    bench_vector_operations,
    bench_distance_calculations
);
criterion_main!(benches);
