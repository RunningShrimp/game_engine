// Rendering Pipeline Benchmarks
//
// Measures overall rendering pipeline performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::Duration;

#[derive(Clone, Copy)]
struct RenderPass {
    vertex_count: u32,
    index_count: u32,
    draw_call_count: u32,
}

struct RenderPipeline {
    passes: Vec<RenderPass>,
    total_vertices: u32,
    total_indices: u32,
}

impl RenderPipeline {
    fn new() -> Self {
        Self {
            passes: Vec::new(),
            total_vertices: 0,
            total_indices: 0,
        }
    }

    fn add_pass(&mut self, pass: RenderPass) {
        self.total_vertices += pass.vertex_count;
        self.total_indices += pass.index_count;
        self.passes.push(pass);
    }

    fn execute(&self) -> PipelineStats {
        let mut draw_calls = 0;
        let mut vertices = 0;
        let mut indices = 0;

        for pass in &self.passes {
            draw_calls += pass.draw_call_count;
            vertices += pass.vertex_count;
            indices += pass.index_count;
        }

        PipelineStats {
            draw_calls,
            vertices,
            indices,
            pass_count: self.passes.len() as u32,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PipelineStats {
    draw_calls: u32,
    vertices: u32,
    indices: u32,
    pass_count: u32,
}

// Create different rendering scenarios
fn create_shadow_pass(lights: u32) -> RenderPass {
    RenderPass {
        vertex_count: 100_000 * lights,
        index_count: 200_000 * lights,
        draw_call_count: 50 * lights,
    }
}

fn create_geometry_pass(objects: u32) -> RenderPass {
    RenderPass {
        vertex_count: 5_000 * objects,
        index_count: 10_000 * objects,
        draw_call_count: objects,
    }
}

fn create_transparency_pass(objects: u32) -> RenderPass {
    RenderPass {
        vertex_count: 2_000 * objects,
        index_count: 4_000 * objects,
        draw_call_count: objects,
    }
}

fn create_post_process_pass() -> RenderPass {
    RenderPass {
        vertex_count: 4, // Fullscreen quad
        index_count: 6,
        draw_call_count: 5, // Multiple post-processing effects
    }
}

fn bench_shadow_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("shadow_rendering");
    group.measurement_time(Duration::from_secs(10));

    for light_count in [1, 4, 8, 16].iter() {
        group.throughput(Throughput::Elements(*light_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(light_count),
            light_count,
            |b, &lights| {
                let mut pipeline = RenderPipeline::new();
                pipeline.add_pass(create_shadow_pass(lights));

                b.iter(|| black_box(pipeline.execute()));
            },
        );
    }

    group.finish();
}

fn bench_deferred_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("deferred_rendering");
    group.measurement_time(Duration::from_secs(10));

    for object_count in [1_000, 5_000, 10_000, 50_000].iter() {
        group.throughput(Throughput::Elements(*object_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            object_count,
            |b, &objects| {
                let mut pipeline = RenderPipeline::new();
                pipeline.add_pass(create_geometry_pass(objects));
                pipeline.add_pass(create_shadow_pass(4));
                pipeline.add_pass(create_post_process_pass());

                b.iter(|| black_box(pipeline.execute()));
            },
        );
    }

    group.finish();
}

fn bench_forward_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("forward_rendering");
    group.measurement_time(Duration::from_secs(10));

    for object_count in [1_000, 5_000, 10_000, 50_000].iter() {
        group.throughput(Throughput::Elements(*object_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            object_count,
            |b, &objects| {
                let mut pipeline = RenderPipeline::new();
                pipeline.add_pass(RenderPass {
                    vertex_count: 5_000 * objects,
                    index_count: 10_000 * objects,
                    draw_call_count: objects,
                });
                pipeline.add_pass(create_transparency_pass(objects / 10));

                b.iter(|| black_box(pipeline.execute()));
            },
        );
    }

    group.finish();
}

fn bench_forward_vs_deferred(c: &mut Criterion) {
    let mut group = c.benchmark_group("forward_vs_deferred");
    group.measurement_time(Duration::from_secs(10));

    let object_count = 10_000;

    // Forward rendering
    group.bench_function("forward_10k", |b| {
        let mut pipeline = RenderPipeline::new();
        pipeline.add_pass(RenderPass {
            vertex_count: 5_000 * object_count,
            index_count: 10_000 * object_count,
            draw_call_count: object_count,
        });
        pipeline.add_pass(create_transparency_pass(object_count / 10));

        b.iter(|| black_box(pipeline.execute()));
    });

    // Deferred rendering
    group.bench_function("deferred_10k", |b| {
        let mut pipeline = RenderPipeline::new();
        pipeline.add_pass(create_geometry_pass(object_count));
        pipeline.add_pass(create_shadow_pass(4));
        pipeline.add_pass(create_post_process_pass());

        b.iter(|| black_box(pipeline.execute()));
    });

    group.finish();
}

fn bench_post_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("post_processing");
    group.measurement_time(Duration::from_secs(10));

    let effects = vec![
        ("bloom", 2),
        ("tone_mapping", 1),
        ("taa", 1),
        ("ssao", 3),
        ("motion_blur", 1),
        ("depth_of_field", 2),
    ];

    for (effect_name, pass_count) in effects {
        group.bench_with_input(
            BenchmarkId::from_parameter(effect_name),
            pass_count,
            |b, &passes| {
                let pipeline = RenderPass {
                    vertex_count: 4,
                    index_count: 6,
                    draw_call_count: passes,
                };

                b.iter(|| {
                    let stats = PipelineStats {
                        draw_calls: pipeline.draw_call_count,
                        vertices: pipeline.vertex_count,
                        indices: pipeline.index_count,
                        pass_count: 1,
                    };
                    black_box(stats)
                });
            },
        );
    }

    group.finish();
}

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");
    group.measurement_time(Duration::from_secs(15));

    let scenes = vec![
        ("simple", 1_000, 1, 1),
        ("medium", 10_000, 4, 2),
        ("complex", 50_000, 8, 4),
    ];

    for (name, objects, lights, shadow_cascades) in scenes {
        group.bench_with_input(
            BenchmarkId::new(name, objects),
            &(objects, lights, shadow_cascades),
            |b, &(obj, light, shadows)| {
                b.iter(|| {
                    let mut pipeline = RenderPipeline::new();

                    // Shadow passes
                    for _ in 0..light * shadows {
                        pipeline.add_pass(create_shadow_pass(1));
                    }

                    // Main geometry pass
                    pipeline.add_pass(create_geometry_pass(obj));

                    // Transparency pass
                    pipeline.add_pass(create_transparency_pass(obj / 10));

                    // Post-processing
                    pipeline.add_pass(create_post_process_pass());

                    black_box(pipeline.execute());
                });
            },
        );
    }

    group.finish();
}

fn bench_multi_pass_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_pass_rendering");
    group.measurement_time(Duration::from_secs(10));

    for pass_count in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(pass_count),
            pass_count,
            |b, &passes| {
                let mut pipeline = RenderPipeline::new();
                for _ in 0..passes {
                    pipeline.add_pass(create_geometry_pass(1_000));
                }

                b.iter(|| black_box(pipeline.execute()));
            },
        );
    }

    group.finish();
}

criterion_group!(
    name = rendering_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(15))
        .sample_size(100);
    targets =
        bench_shadow_rendering,
        bench_deferred_rendering,
        bench_forward_rendering,
        bench_forward_vs_deferred,
        bench_post_processing,
        bench_full_pipeline,
        bench_multi_pass_rendering
);

criterion_main!(rendering_benches);
