use std::time::{Duration, SystemTime};

/// 性能优化目标
#[derive(Debug, Clone)]
pub struct OptimizationGoal {
    pub metric_name: String,
    pub baseline_value: f64,
    pub target_value: f64, // 目标性能
    pub unit: String,
}

impl OptimizationGoal {
    pub fn new(
        metric_name: impl Into<String>,
        baseline_value: f64,
        target_value: f64,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            metric_name: metric_name.into(),
            baseline_value,
            target_value,
            unit: unit.into(),
        }
    }

    /// 计算达成百分比
    pub fn achievement_percentage(&self, current_value: f64) -> f64 {
        if self.baseline_value == self.target_value {
            100.0
        } else if self.baseline_value > self.target_value {
            // 降低目标（如延迟）
            let improvement = self.baseline_value - current_value;
            let possible_improvement = self.baseline_value - self.target_value;
            if possible_improvement <= 0.0 {
                100.0
            } else {
                (improvement / possible_improvement).min(1.0) * 100.0
            }
        } else {
            // 提升目标（如FPS）
            let improvement = current_value - self.baseline_value;
            let possible_improvement = self.target_value - self.baseline_value;
            if possible_improvement <= 0.0 {
                100.0
            } else {
                (improvement / possible_improvement).min(1.0) * 100.0
            }
        }
    }

    pub fn is_achieved(&self, current_value: f64) -> bool {
        if self.baseline_value > self.target_value {
            current_value <= self.target_value
        } else {
            current_value >= self.target_value
        }
    }
}

/// CPU与GPU性能比较结果
#[derive(Debug, Clone)]
pub struct CpuGpuComparison {
    pub operation: String,
    pub data_size: usize,
    pub cpu_duration: Duration,
    pub gpu_duration: Duration,
    pub gpu_overhead: Duration, // 数据传输开销
    pub speedup: f64,
}

impl CpuGpuComparison {
    pub fn new(
        operation: impl Into<String>,
        data_size: usize,
        cpu_duration: Duration,
        gpu_duration: Duration,
        gpu_overhead: Duration,
    ) -> Self {
        let cpu_micros = cpu_duration.as_micros() as f64;
        let gpu_total = (gpu_duration + gpu_overhead).as_micros() as f64;
        let speedup = if gpu_total > 0.0 {
            cpu_micros / gpu_total
        } else {
            1.0
        };

        Self {
            operation: operation.into(),
            data_size,
            cpu_duration,
            gpu_duration,
            gpu_overhead,
            speedup,
        }
    }

    pub fn is_gpu_beneficial(&self) -> bool {
        self.speedup > 1.5 // 需要至少 1.5x 加速才值得
    }

    pub fn description(&self) -> String {
        if self.is_gpu_beneficial() {
            format!(
                "{} ({}): GPU {:.1}x faster",
                self.operation, self.data_size, self.speedup
            )
        } else {
            format!(
                "{} ({}): GPU not beneficial ({:.1}x speedup)",
                self.operation, self.data_size, self.speedup
            )
        }
    }
}

/// 优化验证结果
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub goal: OptimizationGoal,
    pub initial_value: f64,
    pub final_value: f64,
    pub achievement: f64, // 百分比
    pub achieved: bool,
    pub timestamp: SystemTime,
}

impl OptimizationResult {
    pub fn new(goal: OptimizationGoal, initial_value: f64, final_value: f64) -> Self {
        let achievement = goal.achievement_percentage(final_value);
        let achieved = goal.is_achieved(final_value);

        Self {
            goal,
            initial_value,
            final_value,
            achievement,
            achieved,
            timestamp: SystemTime::now(),
        }
    }

    pub fn improvement_percentage(&self) -> f64 {
        if self.initial_value == 0.0 {
            0.0
        } else if self.initial_value > self.final_value {
            ((self.initial_value - self.final_value) / self.initial_value) * 100.0
        } else {
            ((self.final_value - self.initial_value) / self.initial_value) * 100.0
        }
    }

    pub fn description(&self) -> String {
        format!(
            "{}: {:.2} → {:.2} ({:.1}% improvement, {:.0}% goal achieved)",
            self.goal.metric_name,
            self.initial_value,
            self.final_value,
            self.improvement_percentage(),
            self.achievement
        )
    }
}

/// 性能验证套件
#[derive(Default)]
pub struct PerformanceValidationSuite {
    goals: Vec<OptimizationGoal>,
    results: Vec<OptimizationResult>,
    comparisons: Vec<CpuGpuComparison>,
}

impl PerformanceValidationSuite {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加优化目标
    pub fn add_goal(&mut self, goal: OptimizationGoal) {
        self.goals.push(goal);
    }

    /// 记录优化结果
    pub fn record_result(&mut self, goal: OptimizationGoal, initial: f64, final_val: f64) {
        let result = OptimizationResult::new(goal, initial, final_val);
        self.results.push(result);
    }

    /// 记录CPU/GPU比较
    pub fn record_comparison(&mut self, comparison: CpuGpuComparison) {
        self.comparisons.push(comparison);
    }

    /// 获取达成的目标
    pub fn get_achieved_goals(&self) -> Vec<&OptimizationResult> {
        self.results.iter().filter(|r| r.achieved).collect()
    }

    /// 获取未达成的目标
    pub fn get_unachieved_goals(&self) -> Vec<&OptimizationResult> {
        self.results.iter().filter(|r| !r.achieved).collect()
    }

    /// 获取有益的GPU操作
    pub fn get_beneficial_gpu_ops(&self) -> Vec<&CpuGpuComparison> {
        self.comparisons
            .iter()
            .filter(|c| c.is_gpu_beneficial())
            .collect()
    }

    /// 获取不值得的GPU操作
    pub fn get_non_beneficial_gpu_ops(&self) -> Vec<&CpuGpuComparison> {
        self.comparisons
            .iter()
            .filter(|c| !c.is_gpu_beneficial())
            .collect()
    }

    /// 获取验证摘要
    pub fn get_summary(&self) -> ValidationSummary {
        let total_goals = self.results.len();
        let achieved_goals = self.results.iter().filter(|r| r.achieved).count();
        let failed_goals = total_goals - achieved_goals;

        let avg_achievement = if total_goals > 0 {
            self.results.iter().map(|r| r.achievement).sum::<f64>() / total_goals as f64
        } else {
            0.0
        };

        let avg_improvement = if total_goals > 0 {
            self.results
                .iter()
                .map(|r| r.improvement_percentage())
                .sum::<f64>()
                / total_goals as f64
        } else {
            0.0
        };

        let beneficial_gpu_ops = self.get_beneficial_gpu_ops().len();
        let total_comparisons = self.comparisons.len();
        let avg_speedup = if total_comparisons > 0 {
            self.comparisons.iter().map(|c| c.speedup).sum::<f64>() / total_comparisons as f64
        } else {
            0.0
        };

        ValidationSummary {
            total_goals,
            achieved_goals,
            failed_goals,
            avg_achievement_percentage: avg_achievement,
            avg_improvement_percentage: avg_improvement,
            beneficial_gpu_operations: beneficial_gpu_ops,
            total_gpu_operations: total_comparisons,
            average_speedup: avg_speedup,
        }
    }

    /// 生成验证报告
    pub fn generate_report(&self) -> String {
        let summary = self.get_summary();

        let mut report = String::from(
            "╔════════════════════════════════════════════════╗\n\
             ║ Performance Optimization Validation Report\n\
             ╠════════════════════════════════════════════════╣\n",
        );

        report.push_str(&format!(
            "║ Goals: {}/{} achieved ({:.1}%)\n",
            summary.achieved_goals,
            summary.total_goals,
            (summary.achieved_goals as f64 / summary.total_goals as f64) * 100.0
        ));

        report.push_str(&format!(
            "║ Avg Achievement: {:.1}%\n",
            summary.avg_achievement_percentage
        ));

        report.push_str(&format!(
            "║ Avg Improvement: {:.1}%\n",
            summary.avg_improvement_percentage
        ));

        report.push_str(&format!(
            "║ GPU Operations: {}/{} beneficial (avg {:.1}x speedup)\n",
            summary.beneficial_gpu_operations,
            summary.total_gpu_operations,
            summary.average_speedup
        ));

        report.push_str("╠════════════════════════════════════════════════╣\n");

        report.push_str("║ Detailed Results:\n");
        for result in &self.results {
            let status = if result.achieved { "✓" } else { "✗" };
            report.push_str(&format!("║ {} {}\n", status, result.description()));
        }

        report.push_str("║\n");
        report.push_str("║ GPU Comparisons:\n");
        for comparison in &self.comparisons {
            report.push_str(&format!("║   {}\n", comparison.description()));
        }

        report.push_str("╚════════════════════════════════════════════════╝\n");
        report
    }

    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    pub fn comparison_count(&self) -> usize {
        self.comparisons.len()
    }
}


/// 验证摘要
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    pub total_goals: usize,
    pub achieved_goals: usize,
    pub failed_goals: usize,
    pub avg_achievement_percentage: f64,
    pub avg_improvement_percentage: f64,
    pub beneficial_gpu_operations: usize,
    pub total_gpu_operations: usize,
    pub average_speedup: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_goal() {
        // 降低延迟的目标
        let goal = OptimizationGoal::new("latency", 16.0, 8.0, "ms");
        assert_eq!(goal.achievement_percentage(12.0), 50.0); // 中点
        assert!(goal.is_achieved(8.0));
        assert!(!goal.is_achieved(16.0));
    }

    #[test]
    fn test_fps_goal() {
        // 提升FPS的目标
        let goal = OptimizationGoal::new("fps", 60.0, 120.0, "fps");
        assert_eq!(goal.achievement_percentage(90.0), 50.0); // 中点
        assert!(goal.is_achieved(120.0));
        assert!(!goal.is_achieved(60.0));
    }

    #[test]
    fn test_cpu_gpu_comparison() {
        let comparison = CpuGpuComparison::new(
            "physics",
            1000,
            Duration::from_millis(10),
            Duration::from_millis(2),
            Duration::from_millis(1),
        );

        assert!(comparison.is_gpu_beneficial());
        assert!(comparison.speedup > 3.0);
    }

    #[test]
    fn test_optimization_result() {
        let goal = OptimizationGoal::new("fps", 60.0, 120.0, "fps");
        let result = OptimizationResult::new(goal, 60.0, 90.0);

        assert!(!result.achieved);
        assert_eq!(result.improvement_percentage(), 50.0);
    }

    #[test]
    fn test_validation_suite() {
        let mut suite = PerformanceValidationSuite::new();

        suite.record_result(
            OptimizationGoal::new("fps", 60.0, 120.0, "fps"),
            60.0,
            100.0,
        );

        suite.record_comparison(CpuGpuComparison::new(
            "physics",
            1000,
            Duration::from_millis(10),
            Duration::from_millis(2),
            Duration::from_millis(1),
        ));

        let summary = suite.get_summary();
        assert_eq!(summary.total_goals, 1);
        assert_eq!(summary.total_gpu_operations, 1);
        assert!(summary.beneficial_gpu_operations > 0);
    }

    #[test]
    fn test_validation_report() {
        let mut suite = PerformanceValidationSuite::new();
        suite.record_result(
            OptimizationGoal::new("latency", 16.0, 8.0, "ms"),
            16.0,
            10.0,
        );

        let report = suite.generate_report();
        assert!(report.contains("Performance Optimization Validation Report"));
        assert!(report.contains("latency"));
    }
}
