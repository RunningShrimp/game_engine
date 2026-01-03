//! # Nanite Performance Benchmarks
//!
//! Benchmark suite for Nanite virtual geometry system performance testing.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use game_engine::render::nanite::*;

fn bench_clustering(c: &mut Criterion) {
    let mut group = c.benchmark_group("clustering");

    for triangle_count in [100, 1000, 10000, 100000].iter() {
        let (vertices, indices) = generate_test_mesh(*triangle_count);

        group.bench_with_input(
            BenchmarkId::new("cluster_hierarchy", triangle_count),
            triangle_count,
            |b, _| {
                b.iter(|| {
                    let mut builder = ClusterBuilder::new(ClusterConfig::default());
                    builder.build_hierarchy(black_box(&vertices), black_box(&indices))
                });
            },
        );
    }

    group.finish();
}

fn bench_lod_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("lod_selection");

    let device = create_test_device();
    let mut lod_manager = LODManager::new(LODConfig::default()).unwrap();

    // Create test hierarchies
    let hierarchies = create_test_hierarchies(10, 1000);
    let camera = create_test_camera();
    let culling_results = create_test_culling_results(&hierarchies);
    let quality_metrics = QualityMetrics::default();

    group.bench_function("select_lods_10_meshes", |b| {
        b.iter(|| {
            lod_manager.select_lods(
                black_box(&hierarchies),
                black_box(&camera),
                black_box(&culling_results),
                black_box(&quality_metrics),
            )
        });
    });

    group.finish();
}

fn bench_culling(c: &mut Criterion) {
    let mut group = c.benchmark_group("culling");

    let device = create_test_device();
    let mut culling_system = CullingSystem::new(CullingConfig::default()).unwrap();

    let hierarchies = create_test_hierarchies(10, 1000);
    let camera = create_test_camera();
    let quality_metrics = QualityMetrics::default();

    group.bench_function("cull_all_10_meshes", |b| {
        b.iter(|| {
            culling_system.cull_all(
                black_box(&device),
                black_box(&create_test_queue()),
                black_box(&hierarchies),
                black_box(&camera),
                black_box(&quality_metrics),
            )
        });
    });

    group.finish();
}

fn bench_buffer_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_management");

    let device = create_test_device();
    let config = BufferConfig::default();
    let mut buffer_manager = BufferManager::new(&device, config).unwrap();

    // Create test hierarchy
    let (vertices, indices) = generate_test_mesh(10000);
    let mut builder = ClusterBuilder::new(ClusterConfig::default());
    let hierarchy = builder.build_hierarchy(&vertices, &indices).unwrap();

    group.bench_function("upload_mesh_instances", |b| {
        b.iter(|| {
            let mut bm = buffer_manager.clone();
            bm.upload_mesh_instances(black_box(&device), black_box(&hierarchy))
        });
    });

    group.bench_function("update_instances", |b| {
        let lod_selections = create_test_lod_selections(&hierarchy, 100);

        b.iter(|| {
            buffer_manager.update_instances(
                black_box(&device),
                black_box(&create_test_queue()),
                black_box(&lod_selections),
            )
        });
    });

    group.finish();
}

fn bench_quality_controller(c: &mut Criterion) {
    let mut group = c.benchmark_group("quality_controller");

    let config = MetricsConfig::default();

    group.bench_function("update_quality", |b| {
        let mut controller = QualityController::new(config.clone()).unwrap();

        b.iter(|| {
            controller.update(black_box(0.016))
        });
    });

    group.bench_function("adjust_quality", |b| {
        let mut controller = QualityController::new(config.clone()).unwrap();

        // Pre-warm with some frame times
        for _ in 0..30 {
            controller.update(0.020).unwrap();
        }

        b.iter(|| {
            controller.update(black_box(0.025))
        });
    });

    group.finish();
}

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    let device = create_test_device();
    let config = NaniteConfig::default();
    let mut nanite_system = NaniteSystem::new(&device, config).unwrap();

    // Register test meshes
    let (vertices, indices) = generate_test_mesh(10000);
    let mesh_id = nanite_system.register_mesh(&device, &vertices, &indices).unwrap();

    let camera = create_test_camera();

    group.bench_function("nanite_update", |b| {
        b.iter(|| {
            nanite_system.update(
                black_box(&device),
                black_box(&create_test_queue()),
                black_box(&camera),
                black_box(0.016),
            )
        });
    });

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    let device = create_test_device();

    for triangle_count in [1000, 10000, 100000].iter() {
        let (vertices, indices) = generate_test_mesh(*triangle_count);

        group.bench_with_input(
            BenchmarkId::new("cluster_hierarchy_memory", triangle_count),
            triangle_count,
            |b, _| {
                b.iter(|| {
                    let mut builder = ClusterBuilder::new(ClusterConfig::default());
                    let hierarchy = builder.build_hierarchy(&vertices, &indices).unwrap();
                    hierarchy.nodes.len()
                });
            },
        );
    }

    group.finish();
}

// === Helper Functions ===

fn generate_test_mesh(triangle_count: usize) -> (Vec<Vec3>, Vec<u32>) {
    let vertex_count = triangle_count * 3;
    let mut vertices = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(triangle_count * 3);

    for i in 0..triangle_count {
        let base = (i * 3) as u32;
        indices.extend_from_slice(&[base, base + 1, base + 2]);

        // Generate random-ish vertices
        let f = i as f32;
        vertices.push([f.sin(), f.cos(), 0.0]);
        vertices.push([f.cos(), 0.0, f.sin()]);
        vertices.push([0.0, f.sin(), f.cos()]);
    }

    (vertices, indices)
}

fn create_test_device() -> wgpu::Device {
    // In real benchmarks, this would create an actual device
    // For now, this is a placeholder
    panic!("Test device not implemented - benchmarks require actual wgpu setup");
}

fn create_test_queue() -> wgpu::Queue {
    panic!("Test queue not implemented - benchmarks require actual wgpu setup");
}

fn create_test_hierarchies(count: usize, triangles_per_mesh: usize) -> Vec<ClusterHierarchy> {
    let mut hierarchies = Vec::new();

    for _ in 0..count {
        let (vertices, indices) = generate_test_mesh(triangles_per_mesh);
        let mut builder = ClusterBuilder::new(ClusterConfig::default());
        let hierarchy = builder.build_hierarchy(&vertices, &indices).unwrap();
        hierarchies.push(hierarchy);
    }

    hierarchies
}

fn create_test_camera() -> Camera {
    Camera {
        position: [0.0, 0.0, 10.0],
        view_matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        projection_matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        fov_y: std::f32::consts::PI / 4.0,
        aspect_ratio: 16.0 / 9.0,
        near_plane: 0.1,
        far_plane: 1000.0,
    }
}

fn create_test_culling_results(hierarchies: &[ClusterHierarchy]) -> CullingResults {
    let mut results = CullingResults::new();

    for hierarchy in hierarchies {
        for node in &hierarchy.nodes {
            results.add(ClusterCullingResult {
                cluster_id: node.cluster.id,
                visibility: ClusterVisibility::Visible,
                distance: 10.0,
                screen_bounds: None,
            });
        }
    }

    results
}

fn create_test_lod_selections(hierarchy: &ClusterHierarchy, count: usize) -> Vec<LODSelection> {
    hierarchy.nodes.iter()
        .take(count)
        .map(|node| LODSelection {
            cluster_id: node.cluster.id,
            lod_level: 0,
            screen_space_error: 0.5,
            distance: 10.0,
            visible: true,
        })
        .collect()
}

criterion_group!(
    benches,
    bench_clustering,
    bench_lod_selection,
    bench_culling,
    bench_buffer_management,
    bench_quality_controller,
    bench_full_pipeline,
    bench_memory_usage
);

criterion_main!(benches);
