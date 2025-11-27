/// 游戏引擎综合性能基准测试
/// 
/// 测试SIMD优化、硬件检测、NPU加速等关键性能模块

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use game_engine::performance::simd::{detect_cpu_features};
use game_engine::performance::simd::math::{Vec4Simd, Mat4Simd, QuatSimd, VectorOps, MatrixOps};
use game_engine::performance::hardware::{
    detect_gpu, detect_npu, detect_soc,
    npu_sdk::NpuSdkManager,
    AdaptivePerformance, AutoConfig, PowerManager, HardwareCapability,
};

// ============================================================================
// SIMD性能测试
// ============================================================================

fn bench_simd_dot_product(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_dot_product");
    
    let a = Vec4Simd::new(1.0, 2.0, 3.0, 4.0);
    let b = Vec4Simd::new(5.0, 6.0, 7.0, 8.0);
    
    group.bench_function("runtime_dispatch", |bencher| {
        bencher.iter(|| {
            black_box(a.dot(&b))
        });
    });
    
    // 标量版本对比
    group.bench_function("scalar", |bencher| {
        bencher.iter(|| {
            let a_arr = [1.0f32, 2.0, 3.0, 4.0];
            let b_arr = [5.0f32, 6.0, 7.0, 8.0];
            let result = a_arr[0] * b_arr[0]
                + a_arr[1] * b_arr[1]
                + a_arr[2] * b_arr[2]
                + a_arr[3] * b_arr[3];
            black_box(result)
        });
    });
    
    group.finish();
}

fn bench_simd_matrix_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_matrix_mul");
    
    let a = Mat4Simd::identity();
    let b = Mat4Simd::identity();
    
    group.bench_function("runtime_dispatch", |bencher| {
        bencher.iter(|| {
            let result = a.mul(&b);
            black_box(result)
        });
    });
    
    // 标量版本对比
    group.bench_function("scalar", |bencher| {
        bencher.iter(|| {
            let a_arr = a.data;
            let b_arr = b.data;
            let mut result = [[0.0f32; 4]; 4];
            for i in 0..4 {
                for j in 0..4 {
                    for k in 0..4 {
                        result[i][j] += a_arr[i][k] * b_arr[k][j];
                    }
                }
            }
            black_box(())
        });
    });
    
    group.finish();
}

fn bench_simd_quat_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_quat_mul");
    
    let a = QuatSimd::new(0.707, 0.0, 0.707, 0.0);
    let b = QuatSimd::new(0.707, 0.707, 0.0, 0.0);
    
    group.bench_function("runtime_dispatch", |bencher| {
        bencher.iter(|| {
            let out = a.mul(&b);
            black_box(out)
        });
    });
    
    group.finish();
}

// ============================================================================
// 批量处理性能测试
// ============================================================================

fn bench_batch_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_transform");
    
    for size in [10, 100, 1000, 10000].iter() {
        let matrices = vec![Mat4Simd::identity(); *size];
        let vectors = vec![Vec4Simd::new(1.0, 2.0, 3.0, 1.0); *size];
        let mut results = vec![Vec4Simd::zero(); *size];
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bencher, _| {
            bencher.iter(|| {
                for i in 0..*size {
                    results[i] = matrices[i].transform(&vectors[i]);
                }
                black_box(())
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// 硬件检测性能测试
// ============================================================================

fn bench_hardware_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("hardware_detection");
    
    group.bench_function("cpu_features", |bencher| {
        bencher.iter(|| {
            black_box(detect_cpu_features())
        });
    });
    
    group.bench_function("gpu_detection", |bencher| {
        bencher.iter(|| {
            black_box(detect_gpu())
        });
    });
    
    group.bench_function("npu_detection", |bencher| {
        bencher.iter(|| {
            black_box(detect_npu())
        });
    });
    
    group.bench_function("soc_detection", |bencher| {
        bencher.iter(|| {
            black_box(detect_soc())
        });
    });
    
    group.finish();
}

// ============================================================================
// NPU SDK性能测试
// ============================================================================

fn bench_npu_sdk_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("npu_sdk");
    
    let npu_info = detect_npu();
    
    group.bench_function("manager_creation", |bencher| {
        bencher.iter(|| {
            black_box(NpuSdkManager::new(npu_info.clone()))
        });
    });
    
    group.finish();
}

// ============================================================================
// 自适应性能系统测试
// ============================================================================

fn bench_adaptive_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_performance");
    
    let gpu = detect_gpu();
    let npu = detect_npu();
    let soc = detect_soc();
    let capability = HardwareCapability::evaluate(&gpu, &npu, &soc);
    let config = AutoConfig::from_capability(&capability);
    let power = PowerManager::new(soc);
    let mut system = AdaptivePerformance::new(config, power);
    
    group.bench_function("update", |bencher| {
        bencher.iter(|| {
            system.update(black_box(16.67));
            black_box(system.stats())
        });
    });
    
    group.bench_function("should_adjust", |bencher| {
        bencher.iter(|| {
            black_box(system.stats())
        });
    });
    
    group.finish();
}

// ============================================================================
// 内存分配性能测试
// ============================================================================

fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bencher, &size| {
            bencher.iter(|| {
                let vec: Vec<f32> = vec![0.0; size];
                black_box(vec)
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// 数据结构性能测试
// ============================================================================

fn bench_ring_buffer(c: &mut Criterion) {
    use game_engine::performance::hardware::ring_buffer::RingBuffer;
    
    let mut group = c.benchmark_group("ring_buffer");
    
    let mut buffer = RingBuffer::<f32>::new(100);
    
    group.bench_function("push", |bencher| {
        bencher.iter(|| {
            buffer.push(black_box(42.0));
        });
    });
    
    group.bench_function("average", |bencher| {
        bencher.iter(|| {
            black_box(buffer.average())
        });
    });
    
    group.finish();
}

// ============================================================================
// 综合场景测试
// ============================================================================

fn bench_game_frame_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_frame");
    
    group.bench_function("full_frame", |bencher| {
        let gpu = detect_gpu();
        let npu = detect_npu();
        let soc = detect_soc();
        let capability = HardwareCapability::evaluate(&gpu, &npu, &soc);
        let config = AutoConfig::from_capability(&capability);
        let power = PowerManager::new(soc);
        let mut adaptive_system = AdaptivePerformance::new(config, power);
        let matrices = vec![Mat4Simd::identity(); 1000];
        
        bencher.iter(|| {
            // 模拟一帧的处理
            
            // 1. 更新自适应性能系统
            adaptive_system.update(16.67);
            
            // 2. 批量矩阵运算（模拟物体变换）
            for i in 0..100 {
                let a = matrices[i];
                let b = matrices[i + 1];
                let out = a.mul(&b);
                black_box(out);
            }
            
            // 3. 向量运算（模拟光照计算）
            for _ in 0..50 {
                let a = Vec4Simd::new(1.0, 2.0, 3.0, 4.0);
                let b = Vec4Simd::new(5.0, 6.0, 7.0, 8.0);
                let result = a.dot(&b);
                black_box(result);
            }
            
            black_box(adaptive_system.stats())
        });
    });
    
    group.finish();
}

// ============================================================================
// Criterion配置
// ============================================================================

criterion_group!(
    simd_benches,
    bench_simd_dot_product,
    bench_simd_matrix_mul,
    bench_simd_quat_mul,
    bench_batch_transform
);

criterion_group!(
    hardware_benches,
    bench_hardware_detection,
    bench_npu_sdk_creation
);

criterion_group!(
    system_benches,
    bench_adaptive_performance,
    bench_memory_allocation,
    bench_ring_buffer
);

criterion_group!(
    integration_benches,
    bench_game_frame_simulation
);

criterion_main!(
    simd_benches,
    hardware_benches,
    system_benches,
    integration_benches
);
