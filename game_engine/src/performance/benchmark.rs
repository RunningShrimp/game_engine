//  性能基准测试和回归检测模块
// 
//  提供自动化性能基准测试功能，支持性能回归检测和阈值配置。
// 
//  ## 功能特性
// 
//  - 自动化性能基准测试
//  - 性能回归检测（基于阈值）
//  - 历史性能数据对比
//  - 多种性能指标支持
// 
//  ## 使用示例
// 
//  ```ignore
//  let mut benchmark = PerformanceBenchmark::new();
//  benchmark.set_threshold("physics_step", 16.0); // ms
//  benchmark.run("physics_step", || {
//      physics_step_simulation(0.016);
//  });
//  
//  let result = benchmark.get_result("physics_step");
//  assert!(!result.has_regression());
//  ```

use std::collections::HashMap;
use std::time::Instant;

// ============================================================================
// 性能指标类型
// ============================================================================

/// 性能指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    /// 执行时间（毫秒）
    Duration,
    /// 内存使用（字节）
    Memory,
    /// 帧率（FPS）
    FrameRate,
    /// CPU使用率（百分比）
    CpuUsage,
    /// GPU使用率（百分比）
    GpuUsage,
}

/// 性能阈值类型
#[derive(Debug, Clone)]
pub struct Threshold {
    /// 阈值类型
    pub threshold_type: ThresholdType,
    /// 基准值
    pub baseline: f64,
    /// 允许的偏差百分比
    pub tolerance_percent: f64,
}

/// 阈值类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdType {
    /// 绝对阈值（不能超过此值）
    Absolute,
    /// 相对阈值（相对于基准值的百分比）
    Relative,
    /// 相对于历史平均值的百分比
    RelativeToAverage,
}

impl Threshold {
    pub fn new_absolute(baseline: f64) -> Self {
        Self {
            threshold_type: ThresholdType::Absolute,
            baseline,
            tolerance_percent: 0.0,
        }
    }

    pub fn new_relative(baseline: f64, tolerance_percent: f64) -> Self {
        Self {
            threshold_type: ThresholdType::Relative,
            baseline,
            tolerance_percent,
        }
    }

    pub fn is_exceeded(&self, value: f64, history_average: Option<f64>) -> bool {
        match self.threshold_type {
            ThresholdType::Absolute => value > self.baseline,
            ThresholdType::Relative => value > self.baseline * (1.0 + self.tolerance_percent / 100.0),
            ThresholdType::RelativeToAverage => {
                if let Some(avg) = history_average {
                    value > avg * (1.0 + self.tolerance_percent / 100.0)
                } else {
                    false
                }
            }
        }
    }
}

// ============================================================================
// 性能测试结果
// ============================================================================

/// 性能测试结果
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// 测试名称
    pub name: String,
    /// 当前值
    pub current_value: f64,
    /// 基准值
    pub baseline: f64,
    /// 是否检测到回归
    pub has_regression: bool,
    /// 回归程度（百分比）
    pub regression_percent: f64,
    /// 单位
    pub unit: String,
}

impl BenchmarkResult {
    pub fn has_improvement(&self) -> bool {
        self.current_value < self.baseline * 0.95
    }

    pub fn is_within_threshold(&self, threshold: &Threshold) -> bool {
        !threshold.is_exceeded(self.current_value, None)
    }
}

// ============================================================================
// 性能基准测试器
// ============================================================================

/// 性能基准测试器
#[derive(Debug, Clone)]
pub struct PerformanceBenchmark {
    /// 阈值配置
    thresholds: HashMap<String, Threshold>,
    /// 测试结果
    results: HashMap<String, BenchmarkResult>,
    /// 历史数据（用于平均值计算）
    history: HashMap<String, Vec<f64>>,
    /// 最大历史记录数
    max_history_size: usize,
}

impl Default for PerformanceBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceBenchmark {
    pub fn new() -> Self {
        Self {
            thresholds: HashMap::new(),
            results: HashMap::new(),
            history: HashMap::new(),
            max_history_size: 100,
        }
    }

    /// 设置性能阈值
    pub fn set_threshold(&mut self, name: &str, threshold: Threshold) {
        self.thresholds.insert(name.to_string(), threshold);
    }

    /// 设置绝对阈值
    pub fn set_absolute_threshold(&mut self, name: &str, baseline: f64) {
        self.thresholds.insert(name.to_string(), Threshold::new_absolute(baseline));
    }

    /// 设置相对阈值
    pub fn set_relative_threshold(&mut self, name: &str, baseline: f64, tolerance_percent: f64) {
        self.thresholds.insert(
            name.to_string(),
            Threshold::new_relative(baseline, tolerance_percent),
        );
    }

    /// 运行性能测试（单次）
    pub fn run<F>(&mut self, name: &str, f: F, unit: &str)
    where
        F: FnOnce(),
    {
        let start = Instant::now();
        f();
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

        self.record_result(name, duration_ms, unit);
    }

    /// 运行性能测试（多次取平均）
    pub fn run_multiple<F>(&mut self, name: &str, iterations: usize, mut f: F, unit: &str)
    where
        F: FnMut(),
    {
        let mut durations = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();
            f();
            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            durations.push(duration_ms);
        }

        let avg_duration = durations.iter().sum::<f64>() / durations.len() as f64;
        self.record_result(name, avg_duration, unit);
    }

    /// 记录测试结果
    fn record_result(&mut self, name: &str, value: f64, unit: &str) {
        // 更新历史数据
        let history = self.history.entry(name.to_string()).or_insert_with(Vec::new);
        history.push(value);
        if history.len() > self.max_history_size {
            history.remove(0);
        }

        // 计算基准值（使用阈值配置或历史平均值）
        let baseline = self.thresholds
            .get(name)
            .map(|t| t.baseline)
            .unwrap_or_else(|| {
                history.iter().sum::<f64>() / history.len() as f64
            });

        // 检测回归
        let threshold = self.thresholds.get(name);
        let has_regression = threshold
            .map(|t| t.is_exceeded(value, None))
            .unwrap_or(false);

        let regression_percent = if baseline > 0.0 {
            ((value - baseline) / baseline) * 100.0
        } else {
            0.0
        };

        self.results.insert(
            name.to_string(),
            BenchmarkResult {
                name: name.to_string(),
                current_value: value,
                baseline,
                has_regression,
                regression_percent,
                unit: unit.to_string(),
            },
        );
    }

    /// 获取测试结果
    pub fn get_result(&self, name: &str) -> Option<&BenchmarkResult> {
        self.results.get(name)
    }

    /// 获取所有结果
    pub fn get_all_results(&self) -> Vec<&BenchmarkResult> {
        self.results.values().collect()
    }

    /// 检查是否有任何回归
    pub fn has_any_regression(&self) -> bool {
        self.results.values().any(|r| r.has_regression)
    }

    /// 获取回归数量
    pub fn regression_count(&self) -> usize {
        self.results.values().filter(|r| r.has_regression).count()
    }

    /// 生成报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== 性能基准测试报告 ===\n\n");

        let mut results: Vec<_> = self.results.values().collect();
        results.sort_by(|a, b| a.name.cmp(&b.name));

        for result in results {
            let status = if result.has_regression {
                "❌ 回退"
            } else if result.has_improvement() {
                "✅ 改进"
            } else {
                "✓ 正常"
            };

            report.push_str(&format!(
                "{}: {} {} (基准: {} {}, 变化: {:.2}%)\n",
                result.name,
                result.current_value,
                result.unit,
                result.baseline,
                result.unit,
                result.regression_percent
            ));

            report.push_str(&format!("  状态: {}\n", status));
        }

        report.push_str(&format!("\n总测试数: {}\n", self.results.len()));
        report.push_str(&format!("回归数: {}\n", self.regression_count()));
        report.push_str(&format!("改进数: {}\n", self.results.values().filter(|r| r.has_improvement()).count()));

        report
    }

    /// 清除所有结果
    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    /// 清除历史数据
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

// ============================================================================
// 默认阈值配置
// ============================================================================

/// 默认性能阈值配置
pub struct DefaultThresholds;

impl DefaultThresholds {
    pub fn get_default_thresholds() -> Vec<(&'static str, Threshold)> {
        vec![
            // 物理系统阈值
            (
                "physics_step",
                Threshold::new_relative(0.1, 20.0), // 0.1ms基准，允许20%偏差
            ),
            (
                "physics_collision_detection",
                Threshold::new_relative(0.5, 20.0),
            ),
            // 渲染系统阈值
            (
                "render_frame",
                Threshold::new_relative(16.6, 20.0), // 60FPS基准
            ),
            (
                "render_scene_build",
                Threshold::new_relative(0.5, 20.0),
            ),
            // 音频系统阈值
            (
                "audio_source_creation",
                Threshold::new_relative(0.01, 30.0),
            ),
            // 场景系统阈值
            (
                "scene_load",
                Threshold::new_relative(100.0, 30.0),
            ),
            (
                "scene_switch",
                Threshold::new_relative(50.0, 30.0),
            ),
        ]
    }
}

// ============================================================================
// 性能基准测试套件
// ============================================================================

/// 性能基准测试套件
pub struct BenchmarkSuite {
    benchmark: PerformanceBenchmark,
}

impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchmarkSuite {
    pub fn new() -> Self {
        let mut benchmark = PerformanceBenchmark::new();

        // 应用默认阈值
        for (name, threshold) in DefaultThresholds::get_default_thresholds() {
            benchmark.set_threshold(name, threshold);
        }

        Self { benchmark }
    }

    pub fn run_benchmark<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce(),
    {
        let unit = if name.contains("time") || name.contains("step") || name.contains("render") {
            "ms"
        } else if name.contains("memory") {
            "bytes"
        } else if name.contains("fps") {
            "fps"
        } else {
            "ms"
        };

        self.benchmark.run(name, f, unit);
    }

    pub fn get_benchmark(&self) -> &PerformanceBenchmark {
        &self.benchmark
    }

    pub fn get_benchmark_mut(&mut self) -> &mut PerformanceBenchmark {
        &mut self.benchmark
    }

    pub fn has_any_regression(&self) -> bool {
        self.benchmark.has_any_regression()
    }

    pub fn generate_report(&self) -> String {
        self.benchmark.generate_report()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_threshold_absolute() {
        let threshold = Threshold::new_absolute(10.0);
        assert!(!threshold.is_exceeded(5.0, None));
        assert!(!threshold.is_exceeded(10.0, None));
        assert!(threshold.is_exceeded(11.0, None));
    }

    #[test]
    fn test_threshold_relative() {
        let threshold = Threshold::new_relative(100.0, 20.0);
        assert!(!threshold.is_exceeded(100.0, None));
        assert!(!threshold.is_exceeded(119.0, None));
        assert!(threshold.is_exceeded(121.0, None));
    }

    #[test]
    fn test_benchmark_run() {
        let mut benchmark = PerformanceBenchmark::new();
        benchmark.run("test", || {
            std::thread::sleep(Duration::from_millis(1));
        }, "ms");

        let result = benchmark.get_result("test");
        assert!(result.is_some());
        assert!(!result.unwrap().has_regression);
    }

    #[test]
    fn test_benchmark_run_multiple() {
        let mut benchmark = PerformanceBenchmark::new();
        benchmark.run_multiple("test", 5, || {
            std::thread::sleep(Duration::from_millis(1));
        }, "ms");

        let result = benchmark.get_result("test");
        assert!(result.is_some());
        assert!(result.unwrap().current_value > 0.0);
    }

    #[test]
    fn test_benchmark_regression_detection() {
        let mut benchmark = PerformanceBenchmark::new();
        benchmark.set_relative_threshold("test", 1.0, 10.0);

        benchmark.run("test", || {
            std::thread::sleep(Duration::from_millis(1));
        }, "ms");

        let result = benchmark.get_result("test").unwrap();
        assert!(!result.has_regression);
    }

    #[test]
    fn test_benchmark_suite() {
        let mut suite = BenchmarkSuite::new();
        suite.run_benchmark("physics_step", || {
            std::thread::sleep(Duration::from_micros(100));
        });

        assert!(!suite.has_any_regression());
    }

    #[test]
    fn test_default_thresholds() {
        let thresholds = DefaultThresholds::get_default_thresholds();
        assert!(!thresholds.is_empty());
        assert!(thresholds.iter().any(|(name, _)| *name == "physics_step"));
    }
}
