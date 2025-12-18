use std::collections::HashMap;
use std::fmt;
/// 性能基准测试框架
///
/// 提供统一的基准测试接口，用于测量关键路径的执行性能
/// 支持多种测试场景和硬件条件
use std::time::{Duration, Instant};

/// 基准测试结果
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
    pub stddev_duration: Duration,
}

impl fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:\n  Iterations: {}\n  Total: {:.3}ms\n  Avg: {:.3}μs\n  Min: {:.3}μs\n  Max: {:.3}μs\n  StdDev: {:.3}μs",
            self.name,
            self.iterations,
            self.total_duration.as_secs_f64() * 1000.0,
            self.avg_duration.as_secs_f64() * 1_000_000.0,
            self.min_duration.as_secs_f64() * 1_000_000.0,
            self.max_duration.as_secs_f64() * 1_000_000.0,
            self.stddev_duration.as_secs_f64() * 1_000_000.0,
        )
    }
}

/// 基准测试器
#[derive(Default)]
pub struct Benchmark {
    results: HashMap<String, BenchmarkResult>,
}

impl Benchmark {
    /// 创建新的基准测试器
    pub fn new() -> Self {
        Self::default()
    }

    /// 运行基准测试
    pub fn run<F>(&mut self, name: &str, iterations: usize, mut f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        let mut durations = Vec::with_capacity(iterations);

        // 预热 (10%)
        let warmup_iterations = (iterations / 10).max(1);
        for _ in 0..warmup_iterations {
            f();
        }

        // 实际测试
        let start = Instant::now();
        for _ in 0..iterations {
            let iter_start = Instant::now();
            f();
            durations.push(iter_start.elapsed());
        }
        let total_duration = start.elapsed();

        // 统计计算
        let min_duration = *durations.iter().min().unwrap_or(&Duration::ZERO);
        let max_duration = *durations.iter().max().unwrap_or(&Duration::ZERO);
        let sum: Duration = durations.iter().sum();
        let avg_duration = Duration::from_nanos(sum.as_nanos() as u64 / iterations.max(1) as u64);

        // 标准差
        let variance: f64 = durations
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - avg_duration.as_nanos() as f64;
                diff * diff
            })
            .sum::<f64>()
            / iterations.max(1) as f64;
        let stddev_duration = Duration::from_nanos(variance.sqrt() as u64);

        let result = BenchmarkResult {
            name: name.to_string(),
            iterations,
            total_duration,
            min_duration,
            max_duration,
            avg_duration,
            stddev_duration,
        };

        self.results.insert(name.to_string(), result.clone());
        result
    }

    /// 打印所有结果
    pub fn print_results(&self) {
        println!("\n=== Benchmark Results ===\n");
        for (_, result) in &self.results {
            println!("{}\n", result);
        }
    }

    /// 获取结果
    pub fn get_result(&self, name: &str) -> Option<&BenchmarkResult> {
        self.results.get(name)
    }

    /// 比较两个结果
    pub fn compare(&self, name1: &str, name2: &str) -> Option<f64> {
        let r1 = self.results.get(name1)?;
        let r2 = self.results.get(name2)?;

        let ratio = r1.avg_duration.as_nanos() as f64 / r2.avg_duration.as_nanos() as f64;
        Some(ratio)
    }

    /// 清空结果
    pub fn clear(&mut self) {
        self.results.clear();
    }
}


/// 性能回归检测
pub struct PerformanceRegression {
    baseline: HashMap<String, Duration>,
    threshold: f64, // 允许的性能下降百分比
}

impl PerformanceRegression {
    /// 创建新的性能回归检测器
    pub fn new(threshold: f64) -> Self {
        Self {
            baseline: HashMap::new(),
            threshold,
        }
    }

    /// 设置基线
    pub fn set_baseline(&mut self, name: &str, duration: Duration) {
        self.baseline.insert(name.to_string(), duration);
    }

    /// 检查是否发生回归
    pub fn check_regression(&self, name: &str, current: Duration) -> bool {
        if let Some(&baseline) = self.baseline.get(name) {
            let regression_ratio = (current.as_nanos() as f64 - baseline.as_nanos() as f64)
                / baseline.as_nanos() as f64;
            regression_ratio > self.threshold
        } else {
            false
        }
    }

    /// 获取性能差异百分比
    pub fn get_regression_percent(&self, name: &str, current: Duration) -> Option<f64> {
        if let Some(&baseline) = self.baseline.get(name) {
            let percent = (current.as_nanos() as f64 - baseline.as_nanos() as f64)
                / baseline.as_nanos() as f64
                * 100.0;
            Some(percent)
        } else {
            None
        }
    }
}

/// 吞吐量测试
pub struct ThroughputTest {
    name: String,
    items_processed: usize,
    duration: Duration,
}

impl ThroughputTest {
    /// 创建新的吞吐量测试
    pub fn new(name: &str, items_processed: usize, duration: Duration) -> Self {
        Self {
            name: name.to_string(),
            items_processed,
            duration,
        }
    }

    /// 获取每秒吞吐量
    pub fn throughput_per_second(&self) -> f64 {
        self.items_processed as f64 / self.duration.as_secs_f64()
    }

    /// 获取每毫秒吞吐量
    pub fn throughput_per_ms(&self) -> f64 {
        self.items_processed as f64 / self.duration.as_secs_f64() / 1000.0
    }

    /// 打印结果
    pub fn print(&self) {
        println!(
            "{}: {:.2} items/sec ({:.2} items/ms)",
            self.name,
            self.throughput_per_second(),
            self.throughput_per_ms()
        );
    }
}

/// 内存性能测试
pub struct MemoryBenchmark {
    name: String,
    bytes_allocated: usize,
    duration: Duration,
}

impl MemoryBenchmark {
    /// 创建新的内存性能测试
    pub fn new(name: &str, bytes_allocated: usize, duration: Duration) -> Self {
        Self {
            name: name.to_string(),
            bytes_allocated,
            duration,
        }
    }

    /// 获取分配速率 (MB/s)
    pub fn allocation_rate_mbps(&self) -> f64 {
        (self.bytes_allocated as f64 / 1024.0 / 1024.0) / self.duration.as_secs_f64()
    }

    /// 打印结果
    pub fn print(&self) {
        println!("{}: {:.2} MB/s", self.name, self.allocation_rate_mbps());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_basic_benchmark() {
        let mut bench = Benchmark::new();

        let mut counter = 0;
        let result = bench.run("counter_increment", 1000, || {
            counter += 1;
        });

        println!("{}", result);
        assert!(result.avg_duration < Duration::from_millis(1));
    }

    #[test]
    fn test_performance_regression() {
        let mut regression = PerformanceRegression::new(0.2); // 20%阈值

        let baseline = Duration::from_micros(100);
        regression.set_baseline("test_func", baseline);

        // 无回归
        let current_ok = Duration::from_micros(110);
        assert!(!regression.check_regression("test_func", current_ok));

        // 发生回归
        let current_bad = Duration::from_micros(130);
        assert!(regression.check_regression("test_func", current_bad));

        let percent = regression.get_regression_percent("test_func", current_bad);
        assert_eq!(percent, Some(30.0));
    }

    #[test]
    fn test_throughput() {
        let throughput = ThroughputTest::new("render_frames", 1000, Duration::from_secs(1));
        assert_eq!(throughput.throughput_per_second(), 1000.0);

        let throughput2 = ThroughputTest::new("particles", 1_000_000, Duration::from_millis(100));
        assert_eq!(throughput2.throughput_per_second(), 10_000_000.0);
    }

    #[test]
    fn test_memory_benchmark() {
        let bench = MemoryBenchmark::new(
            "allocation_test",
            10 * 1024 * 1024, // 10 MB
            Duration::from_millis(100),
        );
        assert!(bench.allocation_rate_mbps() > 90.0); // ~100 MB/s
    }
}
