use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// 性能基线类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaselineType {
    Baseline,    // 设置为基准
    Warning,     // 性能下降10-20%
    Critical,    // 性能下降>20%
    Improvement, // 性能提升
}

impl BaselineType {
    pub fn as_str(&self) -> &str {
        match self {
            BaselineType::Baseline => "Baseline",
            BaselineType::Warning => "Warning",
            BaselineType::Critical => "Critical",
            BaselineType::Improvement => "Improvement",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            BaselineType::Baseline => "GREEN",
            BaselineType::Warning => "YELLOW",
            BaselineType::Critical => "RED",
            BaselineType::Improvement => "BLUE",
        }
    }
}

/// 单个性能指标的基线
#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    pub metric_name: String,
    pub baseline_value: f64,
    pub unit: String,
    pub timestamp: SystemTime,
    pub warning_threshold: f64,  // 默认 10% 下降
    pub critical_threshold: f64, // 默认 20% 下降
}

impl PerformanceBaseline {
    pub fn new(
        metric_name: impl Into<String>,
        baseline_value: f64,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            metric_name: metric_name.into(),
            baseline_value,
            unit: unit.into(),
            timestamp: SystemTime::now(),
            warning_threshold: 0.10,  // 10%
            critical_threshold: 0.20, // 20%
        }
    }

    pub fn with_thresholds(mut self, warning: f64, critical: f64) -> Self {
        self.warning_threshold = warning;
        self.critical_threshold = critical;
        self
    }

    /// 根据当前值判断是否回归
    pub fn evaluate(&self, current_value: f64) -> BaselineType {
        if current_value < self.baseline_value {
            // 性能提升
            return BaselineType::Improvement;
        }

        let regression_ratio = (current_value - self.baseline_value) / self.baseline_value;

        if regression_ratio > self.critical_threshold {
            BaselineType::Critical
        } else if regression_ratio > self.warning_threshold {
            BaselineType::Warning
        } else {
            BaselineType::Baseline
        }
    }

    /// 获取性能下降百分比
    pub fn regression_percentage(&self, current_value: f64) -> f64 {
        ((current_value - self.baseline_value) / self.baseline_value) * 100.0
    }

    /// 获取性能改善百分比
    pub fn improvement_percentage(&self, current_value: f64) -> f64 {
        if current_value >= self.baseline_value {
            0.0
        } else {
            ((self.baseline_value - current_value) / self.baseline_value) * 100.0
        }
    }
}

/// 回归测试结果
#[derive(Debug, Clone)]
pub struct RegressionTestResult {
    pub metric_name: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub baseline_type: BaselineType,
    pub regression_percentage: f64,
    pub timestamp: SystemTime,
    pub description: String,
}

impl RegressionTestResult {
    pub fn new(baseline: &PerformanceBaseline, current_value: f64) -> Self {
        let baseline_type = baseline.evaluate(current_value);
        let regression_percentage = baseline.regression_percentage(current_value);

        let description = match baseline_type {
            BaselineType::Baseline => {
                format!(
                    "{} within acceptable range ({:.2}%)",
                    baseline.metric_name, regression_percentage
                )
            }
            BaselineType::Warning => {
                format!(
                    "[WARNING] {} degraded by {:.2}%",
                    baseline.metric_name, regression_percentage
                )
            }
            BaselineType::Critical => {
                format!(
                    "[CRITICAL] {} degraded by {:.2}%",
                    baseline.metric_name, regression_percentage
                )
            }
            BaselineType::Improvement => {
                let improvement = baseline.improvement_percentage(current_value);
                format!(
                    "[IMPROVEMENT] {} improved by {:.2}%",
                    baseline.metric_name, improvement
                )
            }
        };

        Self {
            metric_name: baseline.metric_name.clone(),
            baseline_value: baseline.baseline_value,
            current_value,
            baseline_type,
            regression_percentage,
            timestamp: SystemTime::now(),
            description,
        }
    }

    pub fn passed(&self) -> bool {
        matches!(
            self.baseline_type,
            BaselineType::Baseline | BaselineType::Improvement
        )
    }

    pub fn failed(&self) -> bool {
        matches!(self.baseline_type, BaselineType::Critical)
    }

    pub fn warned(&self) -> bool {
        matches!(self.baseline_type, BaselineType::Warning)
    }
}

/// 回归测试套件
#[derive(Default)]
pub struct RegressionTestSuite {
    baselines: HashMap<String, PerformanceBaseline>,
    results: Vec<RegressionTestResult>,
    max_results: usize,
}

impl RegressionTestSuite {
    pub fn new() -> Self {
        Self {
            max_results: 1000,
            ..Default::default()
        }
    }

    /// 注册基线
    pub fn register_baseline(&mut self, baseline: PerformanceBaseline) {
        self.baselines
            .insert(baseline.metric_name.clone(), baseline);
    }

    /// 注册多个基线
    pub fn register_baselines(&mut self, baselines: Vec<PerformanceBaseline>) {
        for baseline in baselines {
            self.register_baseline(baseline);
        }
    }

    /// 测试单个指标
    pub fn test_metric(
        &mut self,
        metric_name: &str,
        current_value: f64,
    ) -> Result<RegressionTestResult, &'static str> {
        let baseline = self
            .baselines
            .get(metric_name)
            .ok_or("Metric not found in baseline")?;

        let result = RegressionTestResult::new(baseline, current_value);
        self.results.push(result.clone());

        if self.results.len() > self.max_results {
            self.results.remove(0);
        }

        Ok(result)
    }

    /// 批量测试指标
    pub fn test_metrics(
        &mut self,
        measurements: &HashMap<String, f64>,
    ) -> Vec<RegressionTestResult> {
        let mut results = Vec::new();

        for (metric_name, current_value) in measurements {
            if let Ok(result) = self.test_metric(metric_name, *current_value) {
                results.push(result);
            }
        }

        results
    }

    /// 获取所有失败的测试
    pub fn get_failed_tests(&self) -> Vec<&RegressionTestResult> {
        self.results.iter().filter(|r| r.failed()).collect()
    }

    /// 获取所有警告的测试
    pub fn get_warned_tests(&self) -> Vec<&RegressionTestResult> {
        self.results.iter().filter(|r| r.warned()).collect()
    }

    /// 获取所有改进的测试
    pub fn get_improved_tests(&self) -> Vec<&RegressionTestResult> {
        self.results
            .iter()
            .filter(|r| matches!(r.baseline_type, BaselineType::Improvement))
            .collect()
    }

    /// 获取回归测试摘要
    pub fn get_summary(&self) -> RegressionSummary {
        let total_tests = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed()).count();
        let failed = self.results.iter().filter(|r| r.failed()).count();
        let warned = self.results.iter().filter(|r| r.warned()).count();
        let improved = self
            .results
            .iter()
            .filter(|r| matches!(r.baseline_type, BaselineType::Improvement))
            .count();

        let pass_rate = if total_tests > 0 {
            (passed as f64 / total_tests as f64) * 100.0
        } else {
            100.0
        };

        RegressionSummary {
            total_tests,
            passed,
            failed,
            warned,
            improved,
            pass_rate,
            timestamp: SystemTime::now(),
        }
    }

    /// 生成HTML报告
    pub fn generate_html_report(&self) -> String {
        let summary = self.get_summary();

        let mut html = String::from(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Performance Regression Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .summary { background: #f0f0f0; padding: 10px; border-radius: 5px; }
        .passed { color: green; }
        .warning { color: orange; }
        .critical { color: red; }
        .improvement { color: blue; }
        table { border-collapse: collapse; width: 100%; margin-top: 20px; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #4CAF50; color: white; }
    </style>
</head>
<body>
    <h1>Performance Regression Report</h1>
"#,
        );

        html.push_str(&format!(
            r#"    <div class="summary">
        <p><strong>Total Tests:</strong> {}</p>
        <p><strong>Passed:</strong> <span class="passed">{}</span></p>
        <p><strong>Failed:</strong> <span class="critical">{}</span></p>
        <p><strong>Warnings:</strong> <span class="warning">{}</span></p>
        <p><strong>Improvements:</strong> <span class="improvement">{}</span></p>
        <p><strong>Pass Rate:</strong> {:.1}%</p>
    </div>
"#,
            summary.total_tests,
            summary.passed,
            summary.failed,
            summary.warned,
            summary.improved,
            summary.pass_rate
        ));

        html.push_str(
            r#"    <table>
        <tr>
            <th>Metric</th>
            <th>Baseline</th>
            <th>Current</th>
            <th>Status</th>
            <th>Change</th>
        </tr>
"#,
        );

        for result in &self.results {
            let status_class = match result.baseline_type {
                BaselineType::Baseline => "passed",
                BaselineType::Warning => "warning",
                BaselineType::Critical => "critical",
                BaselineType::Improvement => "improvement",
            };

            html.push_str(&format!(
                r#"        <tr>
            <td>{}</td>
            <td>{:.2}</td>
            <td>{:.2}</td>
            <td><span class="{}">{}</span></td>
            <td>{:.2}%</td>
        </tr>
"#,
                result.metric_name,
                result.baseline_value,
                result.current_value,
                status_class,
                result.baseline_type.as_str(),
                result.regression_percentage
            ));
        }

        html.push_str(
            r#"    </table>
</body>
</html>"#,
        );

        html
    }

    /// 清空结果但保留基线
    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    /// 获取基线数量
    pub fn baseline_count(&self) -> usize {
        self.baselines.len()
    }

    /// 获取测试结果数量
    pub fn result_count(&self) -> usize {
        self.results.len()
    }
}


/// 回归测试摘要
#[derive(Debug, Clone)]
pub struct RegressionSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub warned: usize,
    pub improved: usize,
    pub pass_rate: f64,
    pub timestamp: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_baseline() {
        let baseline = PerformanceBaseline::new("fps", 60.0, "frames/sec");
        assert_eq!(baseline.metric_name, "fps");
        assert_eq!(baseline.baseline_value, 60.0);
    }

    #[test]
    fn test_baseline_evaluate() {
        let baseline = PerformanceBaseline::new("fps", 60.0, "fps");

        // 无回归
        assert_eq!(baseline.evaluate(60.5), BaselineType::Baseline);

        // 警告：12% 增长
        let warning_value = 60.0 * 1.12; // 67.2
        assert_eq!(baseline.evaluate(warning_value), BaselineType::Warning);

        // 严重：25% 增长
        let critical_value = 60.0 * 1.25; // 75
        assert_eq!(baseline.evaluate(critical_value), BaselineType::Critical);

        // 改进
        assert_eq!(baseline.evaluate(59.0), BaselineType::Improvement);
    }

    #[test]
    fn test_regression_result() {
        let baseline = PerformanceBaseline::new("latency", 16.0, "ms");
        let result = RegressionTestResult::new(&baseline, 18.0);

        assert_eq!(result.metric_name, "latency");
        assert_eq!(result.baseline_value, 16.0);
        assert_eq!(result.current_value, 18.0);
        // 18-16 / 16 = 12.5%, which is > 10% but < 20%, so Warning
        assert_eq!(result.baseline_type, BaselineType::Warning);
        assert!(result.warned());
        assert!(!result.failed());
        assert!(!result.passed());
    }

    #[test]
    fn test_regression_suite() {
        let mut suite = RegressionTestSuite::new();

        let baseline1 = PerformanceBaseline::new("fps", 60.0, "fps");
        let baseline2 = PerformanceBaseline::new("memory", 256.0, "MB");

        suite.register_baselines(vec![baseline1, baseline2]);

        assert_eq!(suite.baseline_count(), 2);

        // 测试 fps
        let result1 = suite.test_metric("fps", 59.0).unwrap();
        assert!(result1.passed());

        // 测试 memory 警告 (280 - 256) / 256 = 9.4%, which is < 10%, so Baseline
        // Let's use a value that triggers warning: 256 + 28 = 284 (10.9% increase)
        let result2 = suite.test_metric("memory", 284.0).unwrap();
        assert!(result2.warned());

        assert_eq!(suite.result_count(), 2);
    }

    #[test]
    fn test_regression_summary() {
        let mut suite = RegressionTestSuite::new();

        suite.register_baseline(PerformanceBaseline::new("fps", 60.0, "fps"));
        suite.register_baseline(PerformanceBaseline::new("latency", 16.0, "ms"));

        suite.test_metric("fps", 61.0).unwrap();
        suite.test_metric("latency", 14.0).unwrap();

        let summary = suite.get_summary();
        assert_eq!(summary.total_tests, 2);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 0);
        assert!(summary.pass_rate > 99.0);
    }

    #[test]
    fn test_batch_test_metrics() {
        let mut suite = RegressionTestSuite::new();
        suite.register_baseline(PerformanceBaseline::new("fps", 60.0, "fps"));
        suite.register_baseline(PerformanceBaseline::new("memory", 256.0, "MB"));

        let mut measurements = HashMap::new();
        measurements.insert("fps".to_string(), 61.0);
        measurements.insert("memory".to_string(), 300.0);

        let results = suite.test_metrics(&measurements);
        assert_eq!(results.len(), 2);
        assert_eq!(suite.get_warned_tests().len(), 1);
    }

    #[test]
    fn test_html_report_generation() {
        let mut suite = RegressionTestSuite::new();
        suite.register_baseline(PerformanceBaseline::new("fps", 60.0, "fps"));

        suite.test_metric("fps", 59.0).unwrap();

        let html = suite.generate_html_report();
        assert!(html.contains("Performance Regression Report"));
        assert!(html.contains("Total Tests"));
        assert!(html.contains("fps"));
    }
}
