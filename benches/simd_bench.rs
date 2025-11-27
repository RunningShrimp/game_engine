/// SIMD性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use game_engine::performance::simd::math::{Vec4Simd, Mat4Simd, VectorOps};
use game_engine::performance::simd::batch::{
    BatchTransform, BatchInterpolation, BatchSkinning, BatchParticle,
    BatchConfig, BoneInfluence, Particle,
};

fn bench_vec4_dot(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec4_dot");
    
    let a = Vec4Simd::new(1.0, 2.0, 3.0, 4.0);
    let b = Vec4Simd::new(5.0, 6.0, 7.0, 8.0);
    
    group.bench_function("simd", |bencher| {
        bencher.iter(|| {
            black_box(a.dot(&b))
        });
    });
    
    group.finish();
}

fn bench_vec4_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec4_operations");
    
    let a = Vec4Simd::new(1.0, 2.0, 3.0, 4.0);
    let b = Vec4Simd::new(5.0, 6.0, 7.0, 8.0);
    
    group.bench_function("add", |bencher| {
        bencher.iter(|| {
            black_box(a.add(&b))
        });
    });
    
    group.bench_function("sub", |bencher| {
        bencher.iter(|| {
            black_box(a.sub(&b))
        });
    });
    
    group.bench_function("mul", |bencher| {
        bencher.iter(|| {
            black_box(a.mul(2.0))
        });
    });
    
    group.bench_function("normalize", |bencher| {
        bencher.iter(|| {
            black_box(a.normalize())
        });
    });
    
    group.finish();
}

fn bench_mat4_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("mat4_mul");
    
    let m1 = Mat4Simd::identity();
    let m2 = Mat4Simd::identity();
    
    group.bench_function("simd", |bencher| {
        bencher.iter(|| {
            black_box(m1.mul(&m2))
        });
    });
    
    group.finish();
}

fn bench_batch_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_transform");
    
    let config = BatchConfig::default();
    let transformer = BatchTransform::new(config);
    
    let identity = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    
    for size in [100, 1000, 10000].iter() {
        let vertices = vec![[1.0, 2.0, 3.0, 1.0]; *size];
        let mut output = vec![[0.0; 4]; *size];
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bencher, _| {
            bencher.iter(|| {
                transformer.transform_vertices(&identity, &vertices, &mut output)
            });
        });
    }
    
    group.finish();
}

fn bench_batch_skinning(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_skinning");
    
    let config = BatchConfig::default();
    let skinning = BatchSkinning::new(config);
    
    let identity = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let bone_matrices = vec![identity; 64];
    
    for size in [100, 1000, 5000].iter() {
        let vertices = vec![[1.0, 2.0, 3.0]; *size];
        let normals = vec![[0.0, 1.0, 0.0]; *size];
        let influences = vec![BoneInfluence {
            bone_indices: [0, 1, 2, 3],
            bone_weights: [0.4, 0.3, 0.2, 0.1],
        }; *size];
        
        let mut output_vertices = vec![[0.0; 3]; *size];
        let mut output_normals = vec![[0.0; 3]; *size];
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bencher, _| {
            bencher.iter(|| {
                skinning.linear_blend_skinning(
                    &vertices,
                    &normals,
                    &influences,
                    &bone_matrices,
                    &mut output_vertices,
                    &mut output_normals,
                )
            });
        });
    }
    
    group.finish();
}

fn bench_particle_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("particle_update");
    
    let config = BatchConfig::default();
    let processor = BatchParticle::new(config);
    
    for size in [1000, 10000, 50000].iter() {
        let mut particles = vec![Particle {
            position: [0.0, 0.0, 0.0],
            velocity: [1.0, 1.0, 1.0],
            acceleration: [0.0, -9.8, 0.0],
            life: 1.0,
            size: 1.0,
            rotation: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
        }; *size];
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bencher, _| {
            bencher.iter(|| {
                processor.update_particles(&mut particles, 0.016)
            });
        });
    }
    
    group.finish();
}

fn bench_batch_lerp(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_lerp");
    
    let config = BatchConfig::default();
    let interpolator = BatchInterpolation::new(config);
    
    for size in [100, 1000, 10000].iter() {
        let a = vec![[0.0, 0.0, 0.0, 1.0]; *size];
        let b = vec![[1.0, 1.0, 1.0, 1.0]; *size];
        let mut output = vec![[0.0; 4]; *size];
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bencher, _| {
            bencher.iter(|| {
                interpolator.lerp(&a, &b, 0.5, &mut output)
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_vec4_dot,
    bench_vec4_operations,
    bench_mat4_mul,
    bench_batch_transform,
    bench_batch_skinning,
    bench_particle_update,
    bench_batch_lerp,
);
criterion_main!(benches);
