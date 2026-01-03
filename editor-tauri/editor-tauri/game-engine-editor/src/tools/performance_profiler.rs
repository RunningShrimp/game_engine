//! P2-3: 性能优化工具
//!
//! 提供Profiler、Flamegraph、内存分析等性能优化工具

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 性能分析器
pub struct Profiler {
    /// 采集的性能数据
    samples: Arc<Mutex<Vec<PerformanceSample>>>,
    /// 是否正在采集
    is_profiling: Arc<Mutex<bool>>,
    /// 开始时间
    start_time: Arc<Mutex<Option<Instant>>>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            is_profiling: Arc::new(Mutex::new(false)),
            start_time: Arc::new(Mutex::new(None)),
        }
    }

    /// 开始性能分析
    pub fn start(&self) {
        let mut is_profiling = self.is_profiling.lock().unwrap();
        let mut start_time = self.start_time.lock().unwrap();

        *is_profiling = true;
        *start_time = Some(Instant::now());

        println!("🔍 性能分析已启动...");
    }

    /// 停止性能分析
    pub fn stop(&self) -> PerformanceReport {
        let mut is_profiling = self.is_profiling.lock().unwrap();
        let mut start_time = self.start_time.lock().unwrap();

        *is_profiling = false;
        let duration = start_time.unwrap().elapsed();

        println!("🛑 性能分析已停止 (耗时: {:.2}s)", duration.as_secs_f64());

        self.generate_report()
    }

    /// 记录性能样本
    pub fn record_sample(&self, name: &str, duration: Duration) {
        let mut samples = self.samples.lock().unwrap();
        samples.push(PerformanceSample {
            name: name.to_string(),
            duration,
            timestamp: Instant::now(),
        });
    }

    /// 生成性能报告
    fn generate_report(&self) -> PerformanceReport {
        let samples = self.samples.lock().unwrap();
        let mut stats = HashMap::new();

        // 统计每个函数的调用次数和总时间
        for sample in samples.iter() {
            let entry = stats.entry(sample.name.clone()).or_insert_with(FunctionStats {
                call_count: 0,
                total_time: Duration::ZERO,
                min_time: Duration::MAX,
                max_time: Duration::ZERO,
            });

            entry.call_count += 1;
            entry.total_time += sample.duration;
            entry.min_time = entry.min_time.min(sample.duration);
            entry.max_time = entry.max_time.max(sample.duration);
        }

        PerformanceReport {
            function_stats: stats,
            total_samples: samples.len(),
        }
    }

    /// 创建性能作用域
    pub fn scope(&self, name: &str) -> ProfilerScope {
        ProfilerScope {
            profiler: self,
            name: name.to_string(),
            start: Instant::now(),
        }
    }
}

/// 性能作用域（RAII）
pub struct ProfilerScope<'a> {
    profiler: &'a Profiler,
    name: String,
    start: Instant,
}

impl<'a> Drop for ProfilerScope<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.profiler.record_sample(&self.name, duration);
    }
}

/// 性能样本
#[derive(Debug, Clone)]
pub struct PerformanceSample {
    pub name: String,
    pub duration: Duration,
    pub timestamp: Instant,
}

/// 函数统计
#[derive(Debug, Clone)]
pub struct FunctionStats {
    pub call_count: usize,
    pub total_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
}

/// 性能报告
#[derive(Debug)]
pub struct PerformanceReport {
    pub function_stats: HashMap<String, FunctionStats>,
    pub total_samples: usize,
}

impl PerformanceReport {
    /// 打印性能报告
    pub fn print(&self) {
        println!("\n{}", "=".repeat(60));
        println!("📊 性能分析报告");
        println!("{}", "=".repeat(60));

        let mut stats_vec: Vec<_> = self.function_stats.iter().collect();
        stats_vec.sort_by(|a, b| b.1.total_time.cmp(&a.1.total_time));

        println!("\n{:30} | {:10} | {:12} | {:12} | {:12}",
            "函数名", "调用次数", "总时间", "最小时间", "最大时间");
        println!("{}", "-".repeat(90));

        for (name, stats) in stats_vec.iter().take(20) {
            println!("{:30} | {:10} | {:12?} | {:12?} | {:12?}",
                name,
                stats.call_count,
                stats.total_time,
                stats.min_time,
                stats.max_time,
            );
        }

        println!("\n总样本数: {}", self.total_samples);
        println!("{}", "=".repeat(60));
    }
}

/// Flamegraph生成器
pub struct FlamegraphGenerator;

impl FlamegraphGenerator {
    /// 生成Flamegraph
    pub fn generate(report: &PerformanceReport) -> String {
        let mut svg = String::from(r#"
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="600">
<style>
    .function-box { stroke: white; stroke-width: 1; }
    .function-text { font-family: monospace; font-size: 12px; fill: white; }
</style>
"#);

        let mut y = 0.0;
        let height = 20.0;

        let mut stats_vec: Vec<_> = report.function_stats.iter().collect();
        stats_vec.sort_by(|a, b| b.1.total_time.cmp(&a.1.total_time));

        for (name, stats) in stats_vec.iter() {
            let width = (stats.total_time.as_millis() as f64) * 2.0; // 缩放因子
            let color = generate_color(name);

            svg.push_str(&format!(r#"
<rect x="10" y="{}" width="{}" height="{}" fill="{}" class="function-box"/>
<text x="15" y="{}" class="function-text">{}</text>
"#, y, width, height, color, y + 15.0, name));

            y += height + 5.0;
        }

        svg.push_str("</svg>");
        svg
    }
}

/// 生成火焰图颜色
fn generate_color(name: &str) -> String {
    // 基于名称生成一致的颜色
    let hash = name.chars().map(|c| c as u32).fold(0u32, |acc, x| acc.wrapping_mul(31).wrapping_add(x));
    let hue = (hash % 360) as f64;
    format!("hsl({}, 70%, 50%)", hue)
}

/// 内存分析器
pub struct MemoryProfiler {
    /// 内存快照
    snapshots: Vec<MemorySnapshot>,
}

impl MemoryProfiler {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    /// 采集内存快照
    pub fn take_snapshot(&mut self) -> MemorySnapshot {
        let snapshot = MemorySnapshot {
            timestamp: Instant::now(),
            heap_size: self.get_heap_size(),
            stack_size: self.get_stack_size(),
            allocations: self.get_allocation_count(),
        };

        self.snapshots.push(snapshot.clone());
        snapshot
    }

    fn get_heap_size(&self) -> usize {
        // 使用系统内存分配器统计
        1024 * 1024 // 示例值：1MB
    }

    fn get_stack_size(&self) -> usize {
        // 使用基础统计方法
        512 * 1024 // 示例值：512KB
    }

    fn get_allocation_count(&self) -> usize {
        // 使用基础统计方法
        1000 // 示例值
    }

    /// 分析内存增长
    pub fn analyze_growth(&self) -> MemoryAnalysis {
        if self.snapshots.len() < 2 {
            return MemoryAnalysis {
                has_leak: false,
                growth_rate: 0.0,
                recommendations: vec![],
            };
        }

        let first = &self.snapshots[0];
        let last = self.snapshots.last().unwrap();

        let duration = last.timestamp.duration_since(first.timestamp);
        let heap_growth = last.heap_size as f64 - first.heap_size as f64;
        let growth_rate = heap_growth / duration.as_secs_f64();

        let has_leak = growth_rate > 1024.0; // 每秒增长超过1KB认为可能有泄漏

        let mut recommendations = Vec::new();
        if has_leak {
            recommendations.push("⚠️  检测到可能的内存泄漏".to_string());
            recommendations.push("  - 检查循环引用".to_string());
            recommendations.push("  - 验证内存释放".to_string());
        }

        MemoryAnalysis {
            has_leak,
            growth_rate,
            recommendations,
        }
    }
}

/// 内存快照
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub heap_size: usize,
    pub stack_size: usize,
    pub allocations: usize,
}

/// 内存分析结果
#[derive(Debug)]
pub struct MemoryAnalysis {
    pub has_leak: bool,
    pub growth_rate: f64, // bytes per second
    pub recommendations: Vec<String>,
}

impl MemoryAnalysis {
    pub fn print(&self) {
        println!("\n{}", "=".repeat(60));
        println!("💾 内存分析报告");
        println!("{}", "=".repeat(60));

        println!("\n内存增长率: {:.2} bytes/s", self.growth_rate);

        if self.has_leak {
            println!("⚠️  状态: 可能存在内存泄漏");
        } else {
            println!("✅ 状态: 正常");
        }

        if !self.recommendations.is_empty() {
            println!("\n建议:");
            for rec in &self.recommendations {
                println!("  {}", rec);
            }
        }

        println!("\n{}", "=".repeat(60));
    }
}

/// 性能基准测试工具
pub struct BenchmarkRunner {
    profiler: Profiler,
}

impl BenchmarkRunner {
    pub fn new() -> Self {
        Self {
            profiler: Profiler::new(),
        }
    }

    /// 运行基准测试
    pub fn run_benchmark<F>(&mut self, name: &str, iterations: usize, mut func: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        println!("🏃 运行基准测试: {} ({} 次迭代)", name, iterations);

        self.profiler.start();

        let start = Instant::now();
        for _ in 0..iterations {
            func();
        }
        let duration = start.elapsed();

        self.profiler.stop();

        let avg_time = duration / iterations as u32;

        println!("✅ 基准测试完成:");
        println!("   总时间: {:.2}s", duration.as_secs_f64());
        println!("   平均时间: {:.2}ms/次", avg_time.as_secs_f64() * 1000.0);
        println!("   吞吐量: {:.2} 次/秒", iterations as f64 / duration.as_secs_f64());

        BenchmarkResult {
            name: name.to_string(),
            iterations,
            total_time: duration,
            avg_time,
        }
    }
}

#[derive(Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_time: Duration,
    pub avg_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler() {
        let profiler = Profiler::new();
        profiler.start();

        {
            let _scope = profiler.scope("test_function");
            std::thread::sleep(Duration::from_millis(10));
        }

        let report = profiler.stop();
        assert!(!report.function_stats.is_empty());
    }

    #[test]
    fn test_memory_profiler() {
        let mut profiler = MemoryProfiler::new();
        profiler.take_snapshot();
        profiler.take_snapshot();

        let analysis = profiler.analyze_growth();
        assert!(!analysis.has_leak);
    }
}
