//! 协程系统性能基准测试
//!
//! 测试协程创建、执行和销毁的性能。

use game_engine::coroutine::{
    CoroutineExecutor, CoroutineId, CoroutinePriority, CoroutineType, CoroutineWaiter,
    WaitForSeconds,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

// =============================================================================
// 基准测试辅助函数
// =============================================================================

fn format_throughput(count: u64, duration: Duration) -> String {
    let secs = duration.as_secs_f64();
    let throughput = count as f64 / secs;
    if throughput >= 1_000_000.0 {
        format!("{:.2}M ops/sec", throughput / 1_000_000.0)
    } else if throughput >= 1_000.0 {
        format!("{:.2}K ops/sec", throughput / 1_000.0)
    } else {
        format!("{:.2} ops/sec", throughput)
    }
}

// =============================================================================
// 协程创建基准
// =============================================================================

#[tokio::test]
async fn bench_coroutine_creation() {
    const COUNT: u64 = 10_000;

    let executor = CoroutineExecutor::with_default_config();
    let start = Instant::now();

    for i in 0..COUNT {
        let future = Box::pin(async { Ok(()) });
        executor
            .add_coroutine(
                format!("bench_{}", i),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;
    }

    let duration = start.elapsed();
    let throughput = format_throughput(COUNT, duration);

    println!(
        "Coroutine Creation Benchmark:\n\
         - Count: {}\n\
         - Time: {:?}\n\
         - Throughput: {}\n",
        COUNT, duration, throughput
    );

    // 断言合理的性能
    assert!(
        duration < Duration::from_secs(5),
        "Creation too slow: {:?}",
        duration
    );
}

#[tokio::test]
async fn bench_coroutine_creation_with_computation() {
    const COUNT: u64 = 1_000;

    let executor = CoroutineExecutor::with_default_config();
    let start = Instant::now();

    for i in 0..COUNT {
        let counter = i;
        let future = Box::pin(async move {
            // 简单计算
            let _ = counter * 2;
            Ok(())
        });

        executor
            .add_coroutine(
                format!("compute_{}", i),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;
    }

    let duration = start.elapsed();
    let throughput = format_throughput(COUNT, duration);

    println!(
        "Coroutine Creation with Computation Benchmark:\n\
         - Count: {}\n\
         - Time: {:?}\n\
         - Throughput: {}\n",
        COUNT, duration, throughput
    );
}

// =============================================================================
// 协程执行基准
// =============================================================================

#[tokio::test]
async fn bench_coroutine_execution() {
    const COUNT: usize = 1000;
    const ITERATIONS: usize = 10;

    let executor = CoroutineExecutor::with_default_config();
    let counter = Arc::new(AtomicU32::new(0));

    // 添加协程
    for _ in 0..COUNT {
        let counter_clone = counter.clone();
        let future = Box::pin(async move {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        executor
            .add_coroutine(
                "exec_bench".to_string(),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;
    }

    // 执行并计时
    let start = Instant::now();

    for _ in 0..ITERATIONS {
        executor.update(Duration::from_millis(16)).await;
    }

    let duration = start.elapsed();
    let total_ops = (COUNT * ITERATIONS) as u64;
    let throughput = format_throughput(total_ops, duration);

    println!(
        "Coroutine Execution Benchmark:\n\
         - Coroutines: {}\n\
         - Iterations: {}\n\
         - Total operations: {}\n\
         - Time: {:?}\n\
         - Throughput: {}\n",
        COUNT, ITERATIONS, total_ops, duration, throughput
    );

    // 验证所有协程都已执行
    let executed = counter.load(Ordering::SeqCst);
    assert_eq!(executed as usize, COUNT, "Not all coroutines executed");
}

#[tokio::test]
async fn bench_coroutine_execution_with_yield() {
    const COUNT: usize = 100;

    let executor = CoroutineExecutor::with_default_config();

    // 添加会yield的协程
    for i in 0..COUNT {
        let future = Box::pin(async move {
            // 模拟多次yield
            for _ in 0..10 {
                WaitForSeconds::new(0.001).await.unwrap();
            }
            Ok(())
        });

        executor
            .add_coroutine(
                format!("yield_{}", i),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;
    }

    let start = Instant::now();

    // 执行直到完成
    loop {
        let has_work = executor.update(Duration::from_millis(16)).await;
        if !has_work {
            break;
        }
    }

    let duration = start.elapsed();

    println!(
        "Coroutine Execution with Yield Benchmark:\n\
         - Coroutines: {}\n\
         - Yields per coroutine: 10\n\
         - Total yields: {}\n\
         - Time: {:?}\n\
         - Avg time per yield: {:?}\n",
        COUNT,
        COUNT * 10,
        duration,
        duration / (COUNT * 10) as u32
    );
}

// =============================================================================
// 并发性能基准
// =============================================================================

#[tokio::test]
async fn bench_concurrent_coroutines() {
    const COUNT: usize = 10_000;

    let executor = CoroutineExecutor::with_default_config();

    let start = Instant::now();

    // 创建大量并发协程
    for i in 0..COUNT {
        let future = Box::pin(async move {
            // 模拟短暂工作
            tokio::time::sleep(Duration::from_micros(100)).await;
            Ok(())
        });

        executor
            .add_coroutine(
                format!("concurrent_{}", i),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;
    }

    let creation_time = start.elapsed();

    // 执行所有协程
    let exec_start = Instant::now();

    loop {
        let has_work = executor.update(Duration::from_millis(16)).await;
        if !has_work {
            break;
        }
    }

    let execution_time = exec_start.elapsed();
    let total_time = creation_time + execution_time;
    let throughput = format_throughput(COUNT as u64, total_time);

    println!(
        "Concurrent Coroutines Benchmark:\n\
         - Concurrent coroutines: {}\n\
         - Creation time: {:?}\n\
         - Execution time: {:?}\n\
         - Total time: {:?}\n\
         - Throughput: {}\n",
        COUNT, creation_time, execution_time, total_time, throughput
    );
}

// =============================================================================
// 协程等待基准
// =============================================================================

#[tokio::test]
async fn bench_wait_for_seconds() {
    const COUNT: u64 = 1_000;

    let start = Instant::now();

    for _ in 0..COUNT {
        WaitForSeconds::new(0.001).await.unwrap();
    }

    let duration = start.elapsed();
    let throughput = format_throughput(COUNT, duration);

    println!(
        "WaitForSeconds Benchmark:\n\
         - Count: {}\n\
         - Wait duration: 1ms each\n\
         - Total time: {:?}\n\
         - Throughput: {}\n",
        COUNT, duration, throughput
    );
}

#[tokio::test]
async fn bench_coroutine_waiter() {
    const COUNT: usize = 1000;

    let waiter = CoroutineWaiter::new();
    let start = Instant::now();

    // 创建多个等待
    for i in 0..COUNT {
        let id = CoroutineId::new(i as u64);
        let waiter_clone = waiter.clone();

        tokio::spawn(async move {
            // 模拟等待
            tokio::time::sleep(Duration::from_micros(100)).await;
            waiter_clone.notify(id);
        });
    }

    let spawn_time = start.elapsed();

    // 通知所有
    let notify_start = Instant::now();
    for i in 0..COUNT {
        waiter.notify(CoroutineId::new(i as u64));
    }
    let notify_time = notify_start.elapsed();

    let total_time = spawn_time + notify_time;
    let throughput = format_throughput(COUNT as u64, total_time);

    println!(
        "CoroutineWaiter Benchmark:\n\
         - Waiters: {}\n\
         - Spawn time: {:?}\n\
         - Notify time: {:?}\n\
         - Total time: {:?}\n\
         - Throughput: {}\n",
        COUNT, spawn_time, notify_time, total_time, throughput
    );
}

// =============================================================================
// 内存使用基准
// =============================================================================

#[tokio::test]
async fn bench_memory_usage() {
    const COUNT: usize = 100_000;

    let executor = CoroutineExecutor::with_default_config();

    // 记录初始内存
    let start_memory = get_memory_usage();

    // 创建大量协程
    for i in 0..COUNT {
        let future = Box::pin(async { Ok(()) });
        executor
            .add_coroutine(
                format!("memory_{}", i),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;
    }

    let end_memory = get_memory_usage();
    let memory_per_coroutine = (end_memory - start_memory) as f64 / COUNT as f64;

    println!(
        "Memory Usage Benchmark:\n\
         - Coroutines: {}\n\
         - Memory used: {} bytes\n\
         - Memory per coroutine: {:.2} bytes\n\
         - Estimated per 1K: {:.2} KB\n\
         - Estimated per 1M: {:.2} MB\n",
        COUNT,
        end_memory - start_memory,
        memory_per_coroutine,
        (memory_per_coroutine * 1000.0) / 1024.0,
        (memory_per_coroutine * 1_000_000.0) / (1024.0 * 1024.0)
    );
}

// =============================================================================
// 综合性能基准
// =============================================================================

#[tokio::test]
async fn bench_comprehensive() {
    println!("\n========== COROUTINE SYSTEM COMPREHENSIVE BENCHMARK ==========\n");

    // 运行所有基准测试
    bench_coroutine_creation().await;
    println!();

    bench_coroutine_creation_with_computation().await;
    println!();

    bench_coroutine_execution().await;
    println!();

    bench_coroutine_execution_with_yield().await;
    println!();

    bench_concurrent_coroutines().await;
    println!();

    bench_wait_for_seconds().await;
    println!();

    bench_coroutine_waiter().await;
    println!();

    bench_memory_usage().await;
    println!();

    println!("========== BENCHMARK COMPLETE ==========\n");
}

// =============================================================================
// 辅助函数
// =============================================================================

#[cfg(unix)]
fn get_memory_usage() -> usize {
    use std::fs;
    // 读取进程的status文件获取内存使用
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                // VmRSS: xxx kB
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        return kb * 1024; // 转换为字节
                    }
                }
            }
        }
    }
    0 // 无法获取，返回0
}

#[cfg(not(unix))]
fn get_memory_usage() -> usize {
    // 非Unix系统，返回占位值
    0
}

#[tokio::test]
async fn bench_simple_overhead() {
    const ITERATIONS: u64 = 100_000;

    // 测试协程创建的纯开销
    let executor = CoroutineExecutor::with_default_config();

    let start = Instant::now();
    for i in 0..ITERATIONS {
        let future = Box::pin(async { Ok(()) });
        executor
            .add_coroutine(
                format!("overhead_{}", i),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;
    }
    let duration = start.elapsed();

    let avg_ns = duration.as_nanos() / ITERATIONS as u128;

    println!(
        "Coroutine Overhead Benchmark:\n\
         - Iterations: {}\n\
         - Total time: {:?}\n\
         - Avg per coroutine: {} ns\n\
         - Coroutines per second: {:.0}\n",
        ITERATIONS,
        duration,
        avg_ns,
        1_000_000_000.0 / avg_ns as f64
    );

    // 断言：每次创建应该在合理时间内（例如 < 100微秒）
    assert!(
        avg_ns < 100_000,
        "Creation too slow: {} ns per coroutine",
        avg_ns
    );
}
