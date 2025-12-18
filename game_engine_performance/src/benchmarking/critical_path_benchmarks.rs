use super::benchmark::Benchmark;
/// 关键路径性能基准测试
///
/// 测试引擎核心系统的性能：
/// - 数学运算 (向量/矩阵操作)
/// - ECS系统 (创建实体、添加组件、系统执行)
/// - 渲染系统 (视锥剔除、LOD计算、批渲染)
/// - 物理系统 (刚体更新、碰撞检测)
/// - 内存分配 (Arena分配器、对象池)
use glam::{Mat4, Quat, Vec3, Vec4};

/// 运行所有关键路径基准测试
pub fn run_all_benchmarks() {
    tracing::info!(target: "benchmark", "\n=== Game Engine Performance Benchmarks ===\n");

    let mut bench = Benchmark::new();

    // 数学运算基准
    tracing::info!(target: "benchmark", "--- Math Operations ---");
    benchmark_vector_operations(&mut bench);
    benchmark_matrix_operations(&mut bench);
    benchmark_quaternion_operations(&mut bench);

    // 内存操作基准
    // NOTE: Arena和ObjectPool基准测试依赖引擎核心的memory模块，已跳过
    // 如果需要运行这些基准测试，请在引擎中运行
    // tracing::info!(target: "benchmark", "\n--- Memory Operations ---");
    // benchmark_arena_allocation(&mut bench);
    // benchmark_object_pooling(&mut bench);

    // 几何计算基准
    // NOTE: Frustum和LOD基准测试依赖引擎核心的render模块，已跳过
    // 如果需要运行这些基准测试，请在引擎中运行
    // tracing::info!(target: "benchmark", "\n--- Geometry Operations ---");
    // benchmark_frustum_calculations(&mut bench);
    // benchmark_lod_calculations(&mut bench);

    // 数据结构基准
    tracing::info!(target: "benchmark", "\n--- Data Structures ---");
    benchmark_hashmap_operations(&mut bench);
    benchmark_vector_operations_collection(&mut bench);

    bench.print_results();
}

/// 向量操作基准测试
fn benchmark_vector_operations(bench: &mut Benchmark) {
    // Vec3 加法
    let v1 = Vec3::new(1.0, 2.0, 3.0);
    let v2 = Vec3::new(4.0, 5.0, 6.0);
    let result = bench.run("Vec3::add", 100_000, || {
        let _ = v1 + v2;
    });
    tracing::info!(target: "benchmark", "{}", result);

    // Vec3 点积
    let result = bench.run("Vec3::dot", 100_000, || {
        let _ = v1.dot(v2);
    });
    tracing::info!(target: "benchmark", "{}", result);

    // Vec3 叉积
    let result = bench.run("Vec3::cross", 100_000, || {
        let _ = v1.cross(v2);
    });
    tracing::info!(target: "benchmark", "{}", result);

    // Vec3 归一化
    let result = bench.run("Vec3::normalize", 100_000, || {
        let _ = v1.normalize();
    });
    tracing::info!(target: "benchmark", "{}", result);

    // Vec4 操作
    let v3 = Vec4::new(1.0, 2.0, 3.0, 4.0);
    let v4 = Vec4::new(5.0, 6.0, 7.0, 8.0);
    let result = bench.run("Vec4::add", 100_000, || {
        let _ = v3 + v4;
    });
    tracing::info!(target: "benchmark", "{}", result);
}

/// 矩阵操作基准测试
fn benchmark_matrix_operations(bench: &mut Benchmark) {
    let m1 = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let m2 = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));

    // 矩阵乘法
    let result = bench.run("Mat4::mul_mat4", 100_000, || {
        let _ = m1 * m2;
    });
    tracing::info!(target: "benchmark", "{}", result);

    // 矩阵-向量乘法
    let v = Vec3::new(1.0, 2.0, 3.0);
    let result = bench.run("Mat4::mul_vec3", 100_000, || {
        let _ = m1.transform_point3(v);
    });
    tracing::info!(target: "benchmark", "{}", result);

    // 矩阵转置
    let result = bench.run("Mat4::transpose", 100_000, || {
        let _ = m1.transpose();
    });
    tracing::info!(target: "benchmark", "{}", result);

    // 矩阵逆
    let result = bench.run("Mat4::inverse", 50_000, || {
        let _ = m1.inverse();
    });
    tracing::info!(target: "benchmark", "{}", result);
}

/// 四元数操作基准测试
fn benchmark_quaternion_operations(bench: &mut Benchmark) {
    let q1 = Quat::from_axis_angle(
        Vec3::new(1.0, 0.0, 0.0).normalize(),
        std::f32::consts::PI / 4.0,
    );
    let q2 = Quat::from_axis_angle(
        Vec3::new(0.0, 1.0, 0.0).normalize(),
        std::f32::consts::PI / 6.0,
    );

    // 四元数乘法
    let result = bench.run("Quat::mul", 100_000, || {
        let _ = q1 * q2;
    });
    tracing::info!(target: "benchmark", "{}", result);

    // 四元数归一化
    let result = bench.run("Quat::normalize", 100_000, || {
        let _ = q1.normalize();
    });
    tracing::info!(target: "benchmark", "{}", result);

    // 四元数旋转向量
    let v = Vec3::new(1.0, 0.0, 0.0);
    let result = bench.run("Quat::mul_vec3", 100_000, || {
        let _ = q1 * v;
    });
    tracing::info!(target: "benchmark", "{}", result);

    // 四元数球面插值
    let result = bench.run("Quat::slerp", 100_000, || {
        let _ = q1.slerp(q2, 0.5);
    });
    tracing::info!(target: "benchmark", "{}", result);
}

/// Arena分配器基准测试
/// 
/// 注意：此基准测试依赖引擎核心的memory模块，已注释掉。
/// 如果需要运行此基准测试，请在引擎中运行。
#[allow(dead_code)]
fn benchmark_arena_allocation(_bench: &mut Benchmark) {
    // NOTE: TypedArena is in game_engine::performance::memory, not in profiling crate
    // This benchmark should be run in the engine, not in the profiling crate
    // use game_engine::performance::memory::TypedArena;
    
    tracing::warn!(target: "benchmark", "benchmark_arena_allocation skipped - requires engine memory module");
}

/// 对象池基准测试
/// 
/// 注意：此基准测试依赖引擎核心的memory模块，已注释掉。
/// 如果需要运行此基准测试，请在引擎中运行。
#[allow(dead_code)]
fn benchmark_object_pooling(_bench: &mut Benchmark) {
    // NOTE: ObjectPool is in game_engine::performance::memory, not in profiling crate
    // This benchmark should be run in the engine, not in the profiling crate
    // use game_engine::performance::memory::ObjectPool;

    tracing::warn!(target: "benchmark", "benchmark_object_pooling skipped - requires engine memory module");
}

/// 视锥剔除计算基准测试
/// 
/// 注意：此基准测试依赖引擎核心的render模块，已注释掉。
/// 如果需要运行此基准测试，请在引擎中运行。
#[allow(dead_code)]
fn benchmark_frustum_calculations(_bench: &mut Benchmark) {
    // NOTE: Frustum is in game_engine::render::frustum, not in profiling crate
    // This benchmark should be run in the engine, not in the profiling crate
    // use game_engine::render::frustum::Frustum;
    
    tracing::warn!(target: "benchmark", "benchmark_frustum_calculations skipped - requires engine render module");
}

/// LOD计算基准测试
/// 
/// 注意：此基准测试依赖引擎核心的render模块，已注释掉。
/// 如果需要运行此基准测试，请在引擎中运行。
#[allow(dead_code)]
fn benchmark_lod_calculations(_bench: &mut Benchmark) {
    // NOTE: LodConfig and LodSelector are in game_engine::render::lod, not in profiling crate
    // This benchmark should be run in the engine, not in the profiling crate
    // use game_engine::render::lod::{LodConfig, LodSelector};
    
    tracing::warn!(target: "benchmark", "benchmark_lod_calculations skipped - requires engine render module");
}

/// HashMap操作基准测试
fn benchmark_hashmap_operations(bench: &mut Benchmark) {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    for i in 0..1000 {
        map.insert(i, i as f32);
    }

    let result = bench.run("HashMap::insert", 10_000, || {
        let mut m = map.clone();
        m.insert(999, 999.0);
    });
    tracing::info!(target: "benchmark", "{}", result);

    let result = bench.run("HashMap::get", 100_000, || {
        let _ = map.get(&500);
    });
    tracing::info!(target: "benchmark", "{}", result);
}

/// 向量集合操作基准测试
fn benchmark_vector_operations_collection(bench: &mut Benchmark) {
    let mut vec = Vec::with_capacity(1000);
    for i in 0..1000 {
        vec.push(i as f32);
    }

    let result = bench.run("Vec::push", 10_000, || {
        let mut v = vec.clone();
        v.push(1001.0);
    });
    tracing::info!(target: "benchmark", "{}", result);

    let result = bench.run("Vec::iteration", 10_000, || {
        let mut sum = 0.0;
        for v in &vec {
            sum += v;
        }
        let _ = sum;
    });
    tracing::info!(target: "benchmark", "{}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_benchmarks() {
        run_all_benchmarks();
    }
}
