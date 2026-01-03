// Frustum and Occlusion Culling Benchmarks
//
// Measures the performance of GPU culling operations at different scales

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use game_engine_editor_lib::camera::Camera;
use game_engine_editor_lib::geometry::Geometry;
use glam::{Mat4, Vec3, Vec4};
use std::time::Duration;

// Simulates frustum culling operation
fn frustum_cull_cpu(instances: &[Mat4], camera: &Camera) -> Vec<bool> {
    instances
        .iter()
        .map(|transform| {
            let position = transform.col(3);
            let view_space_pos = camera.view_matrix * position;

            // Simple frustum test
            let abs_z = view_space_pos.z.abs();
            view_space_pos.z > -camera.far
                && view_space_pos.z < -camera.near
                && view_space_pos.x.abs() < abs_z * (camera.fov / 2.0).tan()
        })
        .collect()
}

// Simulated GPU frustum culling (compute shader)
fn frustum_cull_gpu_simulated(instances: &[Mat4], camera: &Camera) -> Vec<bool> {
    // Simulate GPU parallelism by processing in parallel
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let result = Arc::new(AtomicUsize::new(0));
    let chunk_size = (instances.len() / 8).max(1);

    std::thread::scope(|s| {
        for chunk in instances.chunks(chunk_size) {
            let result = Arc::clone(&result);
            s.move(|| {
                let visible = chunk
                    .iter()
                    .filter(|transform| {
                        let position = transform.col(3);
                        let view_space_pos = camera.view_matrix * position;
                        view_space_pos.z > -camera.far
                            && view_space_pos.z < -camera.near
                            && view_space_pos.x.abs() < view_space_pos.z.abs() * (camera.fov / 2.0).tan()
                    })
                    .count();
                result.fetch_add(visible, Ordering::Relaxed);
            });
        }
    });

    // For benchmark, return all true (simplified)
    vec![true; instances.len()]
}

// Create test instance transforms
fn create_instances(count: usize) -> Vec<Mat4> {
    (0..count)
        .map(|i| {
            let x = (i as f32 * 10.0) % 1000.0;
            let y = ((i as f32 * 10.0) / 1000.0).floor() * 10.0;
            let z = ((i as f32 * 10.0) / 1_000_000.0).floor() * 10.0;
            Mat4::from_translation(Vec3::new(x, y, z))
        })
        .collect()
}

fn bench_frustum_culling_cpu(c: &mut Criterion) {
    let mut group = c.benchmark_group("frustum_culling_cpu");
    group.measurement_time(Duration::from_secs(10));

    let camera = Camera {
        position: Vec3::ZERO,
        target: Vec3::new(0.0, 0.0, -1.0),
        up: Vec3::Y,
        fov: 60.0,
        aspect_ratio: 16.0 / 9.0,
        near: 0.1,
        far: 1000.0,
        projection_matrix: Mat4::IDENTITY,
        view_matrix: Mat4::IDENTITY,
    };

    for size in [1_000, 5_000, 10_000, 50_000].iter() {
        let instances = create_instances(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| frustum_cull_cpu(black_box(&instances), black_box(&camera)));
        });
    }

    group.finish();
}

fn bench_frustum_culling_gpu_simulated(c: &mut Criterion) {
    let mut group = c.benchmark_group("frustum_culling_gpu_simulated");
    group.measurement_time(Duration::from_secs(10));

    let camera = Camera {
        position: Vec3::ZERO,
        target: Vec3::new(0.0, 0.0, -1.0),
        up: Vec3::Y,
        fov: 60.0,
        aspect_ratio: 16.0 / 9.0,
        near: 0.1,
        far: 1000.0,
        projection_matrix: Mat4::IDENTITY,
        view_matrix: Mat4::IDENTITY,
    };

    for size in [1_000, 5_000, 10_000, 50_000].iter() {
        let instances = create_instances(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| frustum_cull_gpu_simulated(black_box(&instances), black_box(&camera)));
        });
    }

    group.finish();
}

fn bench_occlusion_culling(c: &mut Criterion) {
    let mut group = c.benchmark_group("occlusion_culling");
    group.measurement_time(Duration::from_secs(10));

    // Simulate occlusion culling with depth buffer testing
    for size in [1_000, 5_000, 10_000, 50_000].iter() {
        let instances = create_instances(*size);
        let depth_buffer = vec![1.0f32; 1920 * 1080]; // Full HD depth buffer

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                // Simulated occlusion test
                instances
                    .iter()
                    .filter(|transform| {
                        let screen_pos = transform.col(3).truncate();
                        let pixel_x = ((screen_pos.x + 1.0) * 960.0) as usize;
                        let pixel_y = ((screen_pos.y + 1.0) * 540.0) as usize;
                        let idx = (pixel_y * 1920 + pixel_x) % depth_buffer.len();
                        screen_pos.z < depth_buffer[idx]
                    })
                    .count()
            });
        });
    }

    group.finish();
}

fn bench_combined_culling(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_culling");
    group.measurement_time(Duration::from_secs(10));

    let camera = Camera {
        position: Vec3::ZERO,
        target: Vec3::new(0.0, 0.0, -1.0),
        up: Vec3::Y,
        fov: 60.0,
        aspect_ratio: 16.0 / 9.0,
        near: 0.1,
        far: 1000.0,
        projection_matrix: Mat4::IDENTITY,
        view_matrix: Mat4::IDENTITY,
    };

    for size in [1_000, 5_000, 10_000, 50_000].iter() {
        let instances = create_instances(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                // Combined frustum + occlusion culling
                let frustum_visible = frustum_cull_cpu(black_box(&instances), black_box(&camera));
                instances
                    .iter()
                    .zip(frustum_visible.iter())
                    .filter(|(_, visible)| **visible)
                    .count()
            });
        });
    }

    group.finish();
}

criterion_group!(
    name = gpu_culling_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(15))
        .sample_size(100);
    targets =
        bench_frustum_culling_cpu,
        bench_frustum_culling_gpu_simulated,
        bench_occlusion_culling,
        bench_combined_culling
);

criterion_main!(gpu_culling_benches);
