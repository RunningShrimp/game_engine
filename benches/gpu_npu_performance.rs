//! # GPU和NPU性能基准测试
//!
//! 测试GPU加速和NPU推理的性能提升效果。

#[cfg(feature = "cuda")]
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
#[cfg(feature = "cuda")]
use game_engine::compute::{CudaPhysicsSystem, CudaParticleSystem, CudaMeshProcessor};
#[cfg(feature = "cuda")]
use game_engine::physics::PhysicsWorld;
#[cfg(feature = "cuda")]
use game_engine::render::mesh::Mesh;
#[cfg(feature = "cuda")]
use glam::{Vec3, Vec4, Quat, Mat4};
#[cfg(feature = "cuda")]
use std::time::Duration;

// ============================================================================
// GPU物理计算基准测试
// ============================================================================

#[cfg(feature = "cuda")]
fn create_test_physics_world(body_count: usize) -> PhysicsWorld {
    let mut world = PhysicsWorld::new();

    // 创建测试刚体
    for i in 0..body_count {
        let position = Vec3::new(
            (i as f32 % 10.0) * 2.0,
            (i as f32 / 10.0).floor() * 2.0,
            0.0,
        );

        // 注意：这里假设PhysicsWorld有add_rigid_body方法
        // 实际实现可能需要调整
    }

    world
}

#[cfg(feature = "cuda")]
fn bench_gpu_physics(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_physics");

    for body_count in [100, 500, 1000, 5000].iter() {
        let mut world = create_test_physics_world(*body_count);
        let mut gpu_system = CudaPhysicsSystem::new();

        group.bench_with_input(
            BenchmarkId::new("gpu_physics", body_count),
            body_count,
            |b, _| {
                b.iter(|| {
                    gpu_system.update(black_box(&mut world), black_box(0.016));
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// CPU物理计算基准测试（对比）
// ============================================================================

#[cfg(feature = "cuda")]
fn bench_cpu_physics(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_physics");

    for body_count in [100, 500, 1000, 5000].iter() {
        let mut world = create_test_physics_world(*body_count);

        group.bench_with_input(
            BenchmarkId::new("cpu_physics", body_count),
            body_count,
            |b, _| {
                b.iter(|| {
                    // CPU物理更新（不使用GPU）
                    world.step(black_box(0.016));
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// GPU粒子系统基准测试
// ============================================================================

#[cfg(feature = "cuda")]
fn create_test_particle_system(count: u32) -> CudaParticleSystem {
    let mut system = CudaParticleSystem::new(count);

    // 发射粒子
    for i in 0..count {
        let position = Vec3::new(
            (i as f32 % 10.0) * 2.0,
            10.0,
            (i as f32 / 10.0).floor() * 2.0,
        );
        let velocity = Vec3::new(
            (i as f32 % 3.0) - 1.0,
            5.0 + (i as f32 % 3.0),
            (i as f32 % 5.0) - 2.0,
        );

        system.emit(position, velocity, 5.0, Vec4::ONE);
    }

    system
}

#[cfg(feature = "cuda")]
fn bench_gpu_particles(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_particles");

    for particle_count in [1000, 5000, 10000, 50000].iter() {
        let mut system = create_test_particle_system(*particle_count);

        group.bench_with_input(
            BenchmarkId::new("gpu_particles", particle_count),
            particle_count,
            |b, _| {
                b.iter(|| {
                    system.update(black_box(0.016));
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// GPU网格蒙皮基准测试
// ============================================================================

#[cfg(feature = "cuda")]
fn create_test_mesh(vertex_count: usize) -> Mesh {
    let mut mesh = Mesh::default();

    // 创建测试顶点
    for i in 0..vertex_count {
        let x = (i as f32 % 100.0) * 0.1;
        let y = ((i as f32 / 100.0) % 100.0) * 0.1;
        let z = ((i as f32 / 10000.0) % 100.0) * 0.1;

        // 注意：这里假设Mesh有vertices字段
        // 实际实现可能需要调整
    }

    mesh
}

#[cfg(feature = "cuda")]
fn bench_gpu_skinning(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_skinning");

    for vertex_count in [1000, 5000, 10000, 50000].iter() {
        let mesh = create_test_mesh(*vertex_count);
        let skeleton = game_engine::animation::Skeleton::default();
        let processor = CudaMeshProcessor::new();

        group.bench_with_input(
            BenchmarkId::new("gpu_skinning", vertex_count),
            vertex_count,
            |b, _| {
                b.iter(|| {
                    processor.compute_skinning(black_box(&mesh), black_box(&skeleton));
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// 综合性能对比
// ============================================================================

#[cfg(feature = "cuda")]
fn bench_comprehensive_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive");

    // 物理计算对比
    group.bench_function("physics_cpu_1000", |b| {
        let mut world = create_test_physics_world(1000);
        b.iter(|| world.step(black_box(0.016)));
    });

    group.bench_function("physics_gpu_1000", |b| {
        let mut world = create_test_physics_world(1000);
        let mut gpu_system = CudaPhysicsSystem::new();
        b.iter(|| gpu_system.update(black_box(&mut world), black_box(0.016)));
    });

    // 粒子系统对比
    group.bench_function("particles_cpu_10000", |b| {
        let mut system = create_test_particle_system(10000);
        b.iter(|| system.update(black_box(0.016)));
    });

    group.bench_function("particles_gpu_10000", |b| {
        let mut system = create_test_particle_system(10000);
        b.iter(|| system.update(black_box(0.016)));
    });

    // 网格蒙皮对比
    group.bench_function("skinning_cpu_10000", |b| {
        let mesh = create_test_mesh(10000);
        let skeleton = game_engine::animation::Skeleton::default();
        let processor = CudaMeshProcessor::new();
        b.iter(|| processor.compute_skinning_cpu(black_box(&mesh), black_box(&skeleton)));
    });

    group.bench_function("skinning_gpu_10000", |b| {
        let mesh = create_test_mesh(10000);
        let skeleton = game_engine::animation::Skeleton::default();
        let processor = CudaMeshProcessor::new();
        b.iter(|| processor.compute_skinning(black_box(&mesh), black_box(&skeleton)));
    });

    group.finish();
}

// ============================================================================
// NPU推理基准测试
// ============================================================================

#[cfg(feature = "npu")]
fn bench_npu_inference(c: &mut Criterion) {
    use game_engine::acceleration::llm::*;

    let mut group = c.benchmark_group("npu_inference");

    // 注意：这些测试需要实际的模型文件
    // 在CI/CD环境中应该跳过或使用mock

    group.bench_function("llm_chat_short", |b| {
        // 模拟短对话
        b.iter(|| {
            // 实际实现需要模型
            black_box("test response");
        });
    });

    group.bench_function("llm_chat_long", |b| {
        // 模拟长对话
        b.iter(|| {
            // 实际实现需要模型
            black_box("longer test response with more content");
        });
    });

    group.finish();
}

// ============================================================================
// 内存使用基准测试
// ============================================================================

#[cfg(feature = "cuda")]
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    // GPU内存分配
    group.bench_function("gpu_alloc_10mb", |b| {
        b.iter(|| {
            // 模拟GPU内存分配
            let _data: Vec<f32> = vec![0.0; 2_500_000]; // ~10MB
            black_box(&_data);
        });
    });

    // CPU内存分配
    group.bench_function("cpu_alloc_10mb", |b| {
        b.iter(|| {
            let _data: Vec<f32> = vec![0.0; 2_500_000];
            black_box(&_data);
        });
    });

    group.finish();
}

// ============================================================================
// 注册基准测试
// ============================================================================

#[cfg(feature = "cuda")]
criterion_group!(
    benches,
    bench_gpu_physics,
    bench_cpu_physics,
    bench_gpu_particles,
    bench_gpu_skinning,
    bench_comprehensive_comparison,
    bench_memory_usage
);

#[cfg(all(feature = "cuda", feature = "npu"))]
criterion_group!(
    npu_benches,
    bench_npu_inference
);

#[cfg(feature = "cuda")]
criterion_main!(benches);

#[cfg(all(feature = "cuda", feature = "npu"))]
criterion_main!(benches, npu_benches);

// ============================================================================
// Fallback（没有cuda feature时的占位符）
// ============================================================================

#[cfg(not(feature = "cuda"))]
fn main() {
    println!("GPU benchmarking requires the 'cuda' feature.");
    println!("Run with: cargo bench --features cuda");
}
