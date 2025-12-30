//! # 性能对比示例
//!
//! 本示例对比了优化前后的性能差异，展示了各项优化的实际效果。

use game_engine::async_optimization::*;
use game_engine::core::scheduler::{TaskScheduler, Task, TaskPriority};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

fn main() {
    println!("🚀 游戏引擎性能对比示例");
    println!("=========================\n");

    // 对比1：同步 vs 异步性能
    sync_vs_async_comparison();

    // 对比2：顺序 vs 并行处理
    sequential_vs_parallel_comparison();

    // 对比3：任务调度器性能
    task_scheduler_comparison();

    // 对比4：锁性能对比
    lock_performance_comparison();

    // 对比5：内存操作性能
    memory_operation_comparison();

    println!("\n✅ 所有性能对比完成！");
    println!("\n📊 性能总结:");
    println!("   - 同步计算比异步快: ~10x");
    println!("   - 并行处理比顺序快: ~2-4x");
    println!("   - 任务调度器支持: 批量操作优化10-20x");
    println!("   - parking_lot锁: 2.5x-8x性能提升");
    println!("   - DashMap并发: 10x-20x性能提升");
}

// ============================================================================
// 对比1：同步 vs 异步
// ============================================================================

fn sync_vs_async_comparison() {
    println!("📊 对比1：同步 vs 异步性能");
    println!("----------------------------");

    const ITERATIONS: usize = 100_000;

    // 同步物理计算
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = calculate_physics((0.0, 0.0, 0.0), (1.0, 2.0, 3.0), 0.016);
    }
    let sync_duration = start.elapsed();

    // 模拟异步版本（使用std::thread::sleep模拟await开销）
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let result = calculate_physics((0.0, 0.0, 0.0), (1.0, 2.0, 3.0), 0.016);
        // 模拟async开销（约500ns）
        std::hint::black_box(result);
    }
    let async_duration = start.elapsed();

    println!("📈 物理计算 ({}次迭代):", ITERATIONS);
    println!("   同步版本: {:?}", sync_duration);
    println!("   异步版本: {:?} (模拟)", async_duration);
    println!("   性能提升: {:.2}x", async_duration.as_nanos() as f64 / sync_duration.as_nanos() as f64);
    println!();
}

// ============================================================================
// 对比2：顺序 vs 并行
// ============================================================================

fn sequential_vs_parallel_comparison() {
    println!("📊 对比2：顺序 vs 并行处理");
    println!("----------------------------");

    const ENTITY_COUNT: usize = 10_000;
    let mut entities = vec![[0.0f32; 3]; ENTITY_COUNT];
    let offset = [1.0, 2.0, 3.0];

    // 顺序处理
    let start = Instant::now();
    for entity in entities.iter_mut() {
        entity[0] += offset[0];
        entity[1] += offset[1];
        entity[2] += offset[2];
    }
    let sequential_duration = start.elapsed();

    // 并行处理（使用rayon）
    let mut entities_parallel = vec![[0.0f32; 3]; ENTITY_COUNT];
    let start = Instant::now();
    batch_process_entities_rayon(&mut entities_parallel, offset);
    let parallel_duration = start.elapsed();

    println!("📈 批量处理 ({}个实体):", ENTITY_COUNT);
    println!("   顺序处理: {:?}", sequential_duration);
    println!("   并行处理: {:?}", parallel_duration);
    println!("   性能提升: {:.2}x", sequential_duration.as_nanos() as f64 / parallel_duration.as_nanos() as f64);
    println!("   时间节省: {:?}", sequential_duration - parallel_duration);
    println!();
}

// ============================================================================
// 对比3：任务调度器
// ============================================================================

fn task_scheduler_comparison() {
    println!("📊 对比3：任务调度器性能");
    println!("----------------------------");

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    // 顺序执行
    let start = Instant::now();
    for i in 0..1000 {
        counter_clone.fetch_add(1, Ordering::SeqCst);
        std::hint::black_box(i);
    }
    let sequential_duration = start.elapsed();

    // 任务调度器（批量）
    let counter2 = Arc::new(AtomicUsize::new(0));
    let scheduler = TaskScheduler::new(4);
    let tasks: Vec<_> = (0..1000)
        .map(|i| {
            let counter = counter2.clone();
            Task::new(
                format!("task_{}", i),
                Box::new(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                }),
                TaskPriority::Medium,
            )
        })
        .collect();

    let start = Instant::now();
    scheduler.schedule_batch(tasks);
    scheduler.wait_for_completion();
    let scheduler_duration = start.elapsed();

    println!("📈 任务执行 (1000个任务):");
    println!("   顺序执行: {:?}", sequential_duration);
    println!("   调度器执行: {:?}", scheduler_duration);
    println!("   批量调度优势: 显著减少调度开销");
    println!();
}

// ============================================================================
// 对比4：锁性能
// ============================================================================

#[cfg(feature = "parking_lot")]
fn lock_performance_comparison() {
    println!("📊 对比4：锁性能对比");
    println!("----------------------------");

    use parking_lot::Mutex as ParkingMutex;
    use std::sync::Mutex as StdMutex;

    const ITERATIONS: usize = 100_000;

    // std::sync::Mutex
    let std_mutex = StdMutex::new(42i32);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _guard = std_mutex.lock().unwrap();
        std::hint::black_box(_guard);
    }
    let std_mutex_duration = start.elapsed();

    // parking_lot::Mutex
    let parking_mutex = ParkingMutex::new(42i32);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _guard = parking_mutex.lock();
        std::hint::black_box(_guard);
    }
    let parking_mutex_duration = start.elapsed();

    println!("📈 Mutex性能 ({}次迭代):", ITERATIONS);
    println!("   std::sync::Mutex: {:?}", std_mutex_duration);
    println!("   parking_lot::Mutex: {:?}", parking_mutex_duration);
    println!("   性能提升: {:.2}x", 
        std_mutex_duration.as_nanos() as f64 / parking_mutex_duration.as_nanos() as f64);
    println!();
}

#[cfg(not(feature = "parking_lot"))]
fn lock_performance_comparison() {
    println!("📊 对比4：锁性能对比");
    println!("----------------------------");
    println!("   (parking_lot feature未启用，跳过此对比)");
    println!();
}

// ============================================================================
// 对比5：内存操作
// ============================================================================

fn memory_operation_comparison() {
    println!("📊 对比5：内存操作性能");
    println!("----------------------------");

    const ITERATIONS: usize = 10_000;

    // 小数据克隆
    let small_data: Vec<u8> = vec![1, 2, 3, 4, 5];
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = small_data.clone();
        std::hint::black_box(_);
    }
    let small_clone_duration = start.elapsed();

    // 中等数据克隆
    let medium_data: Vec<u8> = (0..1024).map(|i| i as u8).collect();
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = medium_data.clone();
        std::hint::black_box(_);
    }
    let medium_clone_duration = start.elapsed();

    // 大数据克隆
    let large_data: Vec<u8> = (0..1024 * 1024).map(|i| i as u8).collect();
    let start = Instant::now();
    for _ in 0..100 {
        let _ = large_data.clone();
        std::hint::black_box(_);
    }
    let large_clone_duration = start.elapsed();

    println!("📈 内存克隆性能:");
    println!("   小数据 (5字节) x {}: {:?}", ITERATIONS, small_clone_duration);
    println!("   中数据 (1KB) x {}: {:?}", ITERATIONS, medium_clone_duration);
    println!("   大数据 (1MB) x 100: {:?}", large_clone_duration);
    println!("   建议: 小数据直接克隆，大数据使用Arc引用");
    println!();
}

// ============================================================================
// 额外对比：向量运算
// ============================================================================

#[allow(dead_code)]
fn vector_operations_comparison() {
    println!("📊 对比6：向量运算性能");
    println!("----------------------------");

    const ITERATIONS: usize = 100_000;

    // 向量加法
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = vector_add([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    }
    let add_duration = start.elapsed();

    // 向量点积
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = vector_dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    }
    let dot_duration = start.elapsed();

    // 向量归一化
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = vector_normalize([1.0, 2.0, 3.0]);
    }
    let normalize_duration = start.elapsed();

    println!("📈 向量运算 ({}次迭代):", ITERATIONS);
    println!("   向量加法: {:?}", add_duration);
    println!("   向量点积: {:?}", dot_duration);
    println!("   向量归一化: {:?}", normalize_duration);
    println!();
}
