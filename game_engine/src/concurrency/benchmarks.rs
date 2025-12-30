//! 性能基准测试模块
//!
//! 本模块提供性能基准测试，用于验证优化效果。

use std::time::Instant;
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::Mutex as ParkingLotMutex;
use dashmap::DashMap;

// ============================================================================
// 基准测试: Arc<Mutex<HashMap>> vs DashMap
// ============================================================================

/// 基准测试结果
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub before_ns: u128,
    pub after_ns: u128,
    pub speedup: f64,
}

impl BenchmarkResult {
    /// 格式化结果
    pub fn display(&self) -> String {
        format!(
            "{}:\n  优化前: {:?}\n  优化后: {:?}\n  性能提升: {:.2}x",
            self.test_name,
            self.before_duration(),
            self.after_duration(),
            self.speedup
        )
    }

    fn before_duration(&self) -> String {
        format!("{:.2}ms", self.before_ns as f64 / 1_000_000.0)
    }

    fn after_duration(&self) -> String {
        format!("{:.2}ms", self.after_ns as f64 / 1_000_000.0)
    }
}

/// 基准测试: 单线程插入性能
pub fn benchmark_single_thread_insert(num_items: usize) -> BenchmarkResult {
    // 测试Arc<Mutex<HashMap>>
    let hashmap_before: ParkingLotMutex<HashMap<u64, Vec<u8>>> =
        ParkingLotMutex::new(HashMap::new());

    let start = Instant::now();
    for i in 0..num_items {
        let mut map = hashmap_before.lock();
        map.insert(i, vec![i as u8; 100]);
    }
    let before_duration = start.elapsed().as_nanos();

    // 测试DashMap
    let dashmap_after: DashMap<u64, Vec<u8>> = DashMap::new();

    let start = Instant::now();
    for i in 0..num_items {
        dashmap_after.insert(i, vec![i as u8; 100]);
    }
    let after_duration = start.elapsed().as_nanos();

    BenchmarkResult {
        test_name: "单线程插入性能".to_string(),
        before_ns: before_duration,
        after_ns: after_duration,
        speedup: before_duration as f64 / after_duration as f64,
    }
}

/// 基准测试: 并发读取性能
pub fn benchmark_concurrent_read(num_threads: usize, num_ops_per_thread: usize) -> BenchmarkResult {
    use std::thread;

    // 测试Arc<Mutex<HashMap>>
    let hashmap_before = Arc::new(ParkingLotMutex::new(
        (0..num_ops_per_thread).map(|i| (i, vec![i as u8; 100])).collect()
    ));
    let mut handles_before = vec![];

    let start = Instant::now();
    for _ in 0..num_threads {
        let map_clone = hashmap_before.clone();
        let handle = thread::spawn(move || {
            let mut sum = 0usize;
            for i in 0..num_ops_per_thread {
                let map = map_clone.lock();
                if let Some(data) = map.get(&i) {
                    sum += data.len();
                }
            }
            sum
        });
        handles_before.push(handle);
    }

    for handle in handles_before {
        handle.join().unwrap();
    }
    let before_duration = start.elapsed().as_nanos();

    // 测试DashMap
    let dashmap_after = Arc::new(DashMap::new());
    for i in 0..num_ops_per_thread {
        dashmap_after.insert(i, vec![i as u8; 100]);
    }
    let mut handles_after = vec![];

    let start = Instant::now();
    for _ in 0..num_threads {
        let map_clone = dashmap_after.clone();
        let handle = thread::spawn(move || {
            let mut sum = 0usize;
            for i in 0..num_ops_per_thread {
                if let Some(data) = map_clone.get(&i) {
                    sum += data.len();
                }
            }
            sum
        });
        handles_after.push(handle);
    }

    for handle in handles_after {
        handle.join().unwrap();
    }
    let after_duration = start.elapsed().as_nanos();

    BenchmarkResult {
        test_name: format!("并发读取 ({}线程)", num_threads),
        before_ns: before_duration,
        after_ns: after_duration,
        speedup: before_duration as f64 / after_duration as f64,
    }
}

/// 基准测试: 并发写入性能
pub fn benchmark_concurrent_write(num_threads: usize, num_ops_per_thread: usize) -> BenchmarkResult {
    use std::thread;

    // 测试Arc<Mutex<HashMap>>
    let hashmap_before = Arc::new(ParkingLotMutex::new(HashMap::new()));
    let mut handles_before = vec![];

    let start = Instant::now();
    for thread_id in 0..num_threads {
        let map_clone = hashmap_before.clone();
        let handle = thread::spawn(move || {
            for i in 0..num_ops_per_thread {
                let key = (thread_id * num_ops_per_thread + i) as u64;
                let mut map = map_clone.lock();
                map.insert(key, vec![i as u8; 100]);
            }
        });
        handles_before.push(handle);
    }

    for handle in handles_before {
        handle.join().unwrap();
    }
    let before_duration = start.elapsed().as_nanos();

    // 测试DashMap
    let dashmap_after = Arc::new(DashMap::new());
    let mut handles_after = vec![];

    let start = Instant::now();
    for thread_id in 0..num_threads {
        let map_clone = dashmap_after.clone();
        let handle = thread::spawn(move || {
            for i in 0..num_ops_per_thread {
                let key = (thread_id * num_ops_per_thread + i) as u64;
                map_clone.insert(key, vec![i as u8; 100]);
            }
        });
        handles_after.push(handle);
    }

    for handle in handles_after {
        handle.join().unwrap();
    }
    let after_duration = start.elapsed().as_nanos();

    BenchmarkResult {
        test_name: format!("并发写入 ({}线程)", num_threads),
        before_ns: before_duration,
        after_ns: after_duration,
        speedup: before_duration as f64 / after_duration as f64,
    }
}

/// 运行所有基准测试
pub fn run_all_benchmarks() -> Vec<BenchmarkResult> {
    let mut results = vec![];

    // 单线程测试
    results.push(benchmark_single_thread_insert(10000));

    // 并发读取测试
    results.push(benchmark_concurrent_read(4, 10000));
    results.push(benchmark_concurrent_read(8, 10000));

    // 并发写入测试
    results.push(benchmark_concurrent_write(4, 10000));
    results.push(benchmark_concurrent_write(8, 10000));

    results
}

// ============================================================================
// 性能对比报告
// ============================================================================

/// 生成性能对比报告
pub fn generate_performance_report() -> String {
    let results = run_all_benchmarks();

    let mut report = String::from("性能基准测试报告\n");
    report.push_str("==================\n\n");

    for result in &results {
        report.push_str(&result.display());
        report.push_str("\n\n");
    }

    // 总结
    report.push_str("总结:\n");
    report.push_str("------\n");

    let avg_speedup: f64 = results.iter().map(|r| r.speedup).sum::<f64>() / results.len() as f64;
    report.push_str(&format!("平均性能提升: {:.2}x\n", avg_speedup));

    let max_speedup = results.iter().map(|r| r.speedup).reduce(f64::max).unwrap_or(0.0);
    report.push_str(&format!("最大性能提升: {:.2}x\n", max_speedup));

    report
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;

    #[test]
    #[ignore] // 基准测试耗时较长，默认忽略
    fn test_single_thread_insert() {
        let result = benchmark_single_thread_insert(1000);
        println!("{}", result.display());
        // DashMap在单线程下也应该略快或相当
        assert!(result.speedup >= 0.8); // 至少不慢
    }

    #[test]
    #[ignore]
    fn test_concurrent_read() {
        let result = benchmark_concurrent_read(4, 1000);
        println!("{}", result.display());
        // DashMap在并发读取下应该快很多
        assert!(result.speedup >= 2.0); // 至少2x
    }

    #[test]
    #[ignore]
    fn test_concurrent_write() {
        let result = benchmark_concurrent_write(4, 1000);
        println!("{}", result.display());
        // DashMap在并发写入下应该快很多
        assert!(result.speedup >= 2.0); // 至少2x
    }

    #[test]
    fn test_performance_report() {
        let report = generate_performance_report();
        println!("{}", report);

        // 验证报告包含关键信息
        assert!(report.contains("性能基准测试报告"));
        assert!(report.contains("平均性能提升"));
        assert!(report.contains("最大性能提升"));
    }
}
