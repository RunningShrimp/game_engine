//! # Render System Benchmarks
//!
//! 渲染系统性能基准测试。

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use game_engine::render::mesh::{Mesh, MeshVertex, PrimitiveTopology};
use glam::{Quat, Vec3};

fn bench_mesh_creation(c: &mut Criterion) {
    c.bench_function("mesh creation", |b| {
        b.iter(|| {
            let mesh = Mesh::new(PrimitiveTopology::TriangleList);
            black_box(mesh)
        })
    });
}

fn bench_mesh_add_vertices(c: &mut Criterion) {
    let mut group = c.benchmark_group("mesh_add_vertices");

    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            let vertices: Vec<MeshVertex> = (0..size)
                .map(|_| MeshVertex {
                    position: Vec3::ZERO,
                    normal: Vec3::Y,
                    uv: [0.0, 0.0],
                })
                .collect();

            b.iter(|| {
                mesh.set_vertices(vertices.clone());
                black_box(&mesh)
            })
        });
    }

    group.finish();
}

fn bench_mesh_compute_bounds(c: &mut Criterion) {
    let mut group = c.benchmark_group("mesh_compute_bounds");

    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            let vertices: Vec<MeshVertex> = (0..size)
                .map(|i| {
                    let x = (i as f32) * 0.1;
                    MeshVertex {
                        position: Vec3::new(x, x, x),
                        normal: Vec3::Y,
                        uv: [0.0, 0.0],
                    }
                })
                .collect();

            mesh.set_vertices(vertices);

            b.iter(|| black_box(mesh.compute_bounds()))
        });
    }

    group.finish();
}

fn bench_transform_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_calculation");

    group.bench_function("trs_from_components", |b| {
        b.iter(|| {
            let translation = Vec3::new(1.0, 2.0, 3.0);
            let rotation = Quat::from_axis_angle(Vec3::Y, 0.5);
            let scale = Vec3::splat(2.0);
            let transform = glam::Mat4::from_scale_rotation_translation(scale, rotation, translation);
            black_box(transform)
        })
    });

    group.bench_function("transform_point", |b| {
        let transform = glam::Mat4::from_scale_rotation_translation(
            Vec3::splat(2.0),
            Quat::IDENTITY,
            Vec3::new(1.0, 2.0, 3.0),
        );
        let point = Vec3::new(5.0, 6.0, 7.0);

        b.iter(|| black_box(transform.transform_point3(point)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_mesh_creation,
    bench_mesh_add_vertices,
    bench_mesh_compute_bounds,
    bench_transform_calculation
);
criterion_main!(benches);
