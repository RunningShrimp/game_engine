/// 性能监测和报告系统
///
/// 统一收集、分析和报告性能数据
use std::collections::HashMap;
use std::time::Instant;

/// 性能指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    // CPU 指标
    FrameTime,
    CpuTime,
    UpdateTime,
    RenderTime,

    // GPU 指标
    GpuTime,
    DrawCalls,
    TriangleCount,
    VertexCount,

    // 内存指标
    RamUsage,
    VramUsage,
    AllocCount,

    // 物理指标
    PhysicsTime,
    CollisionChecks,

    // AI 指标
    AiTime,
    PathfindingTime,
}

/// 单个性能指标
#[derive(Debug, Clone)]
pub struct Metric {
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: String,
    pub timestamp: Instant,
}

impl Metric {
    pub fn new(metric_type: MetricType, value: f64, unit: String) -> Self {
        Self {
            metric_type,
            value,
            unit,
            timestamp: Instant::now(),
        }
    }
}

/// 性能统计（一段时间内的聚合统计）
#[derive(Debug, Clone)]
pub struct MetricStats {
    pub metric_type: MetricType,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub median: f64,
    pub stddev: f64,
    pub samples: usize,
}

impl MetricStats {
    pub fn compute(metric_type: MetricType, values: &[f64]) -> Self {
        if values.is_empty() {
            return Self {
                metric_type,
                min: 0.0,
                max: 0.0,
                avg: 0.0,
                median: 0.0,
                stddev: 0.0,
                samples: 0,
            };
        }

        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let sum: f64 = values.iter().sum();
        let avg = sum / values.len() as f64;

        // 计算中位数
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = sorted[sorted.len() / 2];

        // 计算标准差
        let variance: f64 =
            values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / values.len() as f64;
        let stddev = variance.sqrt();

        Self {
            metric_type,
            min,
            max,
            avg,
            median,
            stddev,
            samples: values.len(),
        }
    }
}

/// 性能监测器
pub struct PerformanceMonitor {
    metrics: HashMap<MetricType, Vec<f64>>,
    metric_history: Vec<Metric>,
    max_history_size: usize,
}

impl PerformanceMonitor {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            metrics: HashMap::new(),
            metric_history: Vec::new(),
            max_history_size,
        }
    }

    /// 记录指标
    pub fn record(&mut self, metric_type: MetricType, value: f64, unit: &str) {
        self.metrics
            .entry(metric_type)
            .or_insert_with(Vec::new)
            .push(value);

        let metric = Metric::new(metric_type, value, unit.to_string());
        self.metric_history.push(metric);

        // 限制历史记录大小
        if self.metric_history.len() > self.max_history_size {
            self.metric_history.remove(0);
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self, metric_type: MetricType) -> Option<MetricStats> {
        self.metrics
            .get(&metric_type)
            .map(|values| MetricStats::compute(metric_type, values))
    }

    /// 清空指定类型的数据
    pub fn clear_metric(&mut self, metric_type: MetricType) {
        self.metrics.remove(&metric_type);
    }

    /// 清空所有数据
    pub fn clear_all(&mut self) {
        self.metrics.clear();
        self.metric_history.clear();
    }

    /// 生成性能报告
    pub fn generate_report(&self) -> PerformanceReport {
        let mut stats = HashMap::new();

        for (&metric_type, values) in &self.metrics {
            stats.insert(metric_type, MetricStats::compute(metric_type, values));
        }

        PerformanceReport {
            stats,
            timestamp: Instant::now(),
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new(60 * 60) // 1小时 (60fps)
    }
}

/// 性能报告
pub struct PerformanceReport {
    pub stats: HashMap<MetricType, MetricStats>,
    pub timestamp: Instant,
}

impl PerformanceReport {
    pub fn print_summary(&self) {
        tracing::info!(target: "performance", "\n=== Performance Report ===\n");

        // CPU 指标
        tracing::info!(target: "performance", "--- CPU Metrics ---");
        if let Some(stats) = self.stats.get(&MetricType::FrameTime) {
            tracing::info!(
                target: "performance",
                "Frame Time: {:.2}ms (avg) {:.2}ms (min) {:.2}ms (max)",
                stats.avg, stats.min, stats.max
            );
        }
        if let Some(stats) = self.stats.get(&MetricType::UpdateTime) {
            tracing::info!(target: "performance", "Update Time: {:.2}ms (avg)", stats.avg);
        }
        if let Some(stats) = self.stats.get(&MetricType::RenderTime) {
            tracing::info!(target: "performance", "Render Time: {:.2}ms (avg)", stats.avg);
        }

        // GPU 指标
        tracing::info!(target: "performance", "\n--- GPU Metrics ---");
        if let Some(stats) = self.stats.get(&MetricType::GpuTime) {
            tracing::info!(target: "performance", "GPU Time: {:.2}ms (avg)", stats.avg);
        }
        if let Some(stats) = self.stats.get(&MetricType::DrawCalls) {
            tracing::info!(target: "performance", "Draw Calls: {:.0} (avg)", stats.avg);
        }
        if let Some(stats) = self.stats.get(&MetricType::TriangleCount) {
            tracing::info!(target: "performance", "Triangles: {:.0} (avg)", stats.avg);
        }

        // 内存指标
        tracing::info!(target: "performance", "\n--- Memory Metrics ---");
        if let Some(stats) = self.stats.get(&MetricType::RamUsage) {
            tracing::info!(target: "performance", "RAM Usage: {:.0}MB (avg) {:.0}MB (peak)", stats.avg, stats.max);
        }
        if let Some(stats) = self.stats.get(&MetricType::VramUsage) {
            tracing::info!(target: "performance", "VRAM Usage: {:.0}MB (avg) {:.0}MB (peak)", stats.avg, stats.max);
        }

        // 物理指标
        tracing::info!(target: "performance", "\n--- Physics Metrics ---");
        if let Some(stats) = self.stats.get(&MetricType::PhysicsTime) {
            tracing::info!(target: "performance", "Physics Time: {:.2}ms (avg)", stats.avg);
        }
        if let Some(stats) = self.stats.get(&MetricType::CollisionChecks) {
            tracing::info!(target: "performance", "Collision Checks: {:.0} (avg)", stats.avg);
        }

        // AI 指标
        tracing::info!(target: "performance", "\n--- AI Metrics ---");
        if let Some(stats) = self.stats.get(&MetricType::AiTime) {
            tracing::info!(target: "performance", "AI Time: {:.2}ms (avg)", stats.avg);
        }
        if let Some(stats) = self.stats.get(&MetricType::PathfindingTime) {
            tracing::info!(target: "performance", "Pathfinding Time: {:.2}ms (avg)", stats.avg);
        }
    }

    /// 检测性能问题
    pub fn detect_issues(&self) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        // 检测帧时间问题
        if let Some(stats) = self.stats.get(&MetricType::FrameTime) {
            if stats.avg > 33.0 {
                // 低于 30fps
                issues.push(PerformanceIssue {
                    severity: IssueSeverity::High,
                    message: format!(
                        "Low frame rate: {:.1}ms ({:.1}fps)",
                        stats.avg,
                        1000.0 / stats.avg
                    ),
                });
            }
            if stats.stddev > stats.avg * 0.5 {
                // 帧时间波动大
                issues.push(PerformanceIssue {
                    severity: IssueSeverity::Medium,
                    message: format!("High frame time variance: {:.1}ms stddev", stats.stddev),
                });
            }
        }

        // 检测 Draw Call 问题
        if let Some(stats) = self.stats.get(&MetricType::DrawCalls) {
            if stats.avg > 1000.0 {
                issues.push(PerformanceIssue {
                    severity: IssueSeverity::Medium,
                    message: format!("High draw call count: {:.0}", stats.avg),
                });
            }
        }

        // 检测内存问题
        if let Some(stats) = self.stats.get(&MetricType::RamUsage) {
            if stats.max > 2048.0 {
                // 超过 2GB
                issues.push(PerformanceIssue {
                    severity: IssueSeverity::High,
                    message: format!("High RAM usage: {:.0}MB", stats.max),
                });
            }
        }

        issues
    }
}

/// 性能问题严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 性能问题
#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub severity: IssueSeverity,
    pub message: String,
}

/// 性能优化建议
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub area: String,
    pub issue: String,
    pub recommendation: String,
    pub expected_improvement: String,
}

impl OptimizationRecommendation {
    pub fn generate_recommendations(report: &PerformanceReport) -> Vec<Self> {
        let mut recommendations = Vec::new();

        // 检测 Draw Call 过多
        if let Some(stats) = report.stats.get(&MetricType::DrawCalls) {
            if stats.avg > 500.0 {
                recommendations.push(Self {
                    area: "Rendering".to_string(),
                    issue: "Too many draw calls".to_string(),
                    recommendation: "Enable draw call batching and implement LOD system"
                        .to_string(),
                    expected_improvement: "30-50% reduction in draw calls".to_string(),
                });
            }
        }

        // 检测 CPU 时间过长
        if let Some(update_stats) = report.stats.get(&MetricType::UpdateTime) {
            if update_stats.avg > 10.0 {
                recommendations.push(Self {
                    area: "CPU".to_string(),
                    issue: "High update time".to_string(),
                    recommendation: "Profile and optimize hot paths, consider using SIMD"
                        .to_string(),
                    expected_improvement: "20-40% improvement in update performance".to_string(),
                });
            }
        }

        // 检测内存使用
        if let Some(ram_stats) = report.stats.get(&MetricType::RamUsage) {
            if ram_stats.avg > 1024.0 {
                recommendations.push(Self {
                    area: "Memory".to_string(),
                    issue: "High RAM usage".to_string(),
                    recommendation: "Use Arena allocator and object pooling".to_string(),
                    expected_improvement: "20-30% reduction in memory usage".to_string(),
                });
            }
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_stats() {
        let values = vec![10.0, 15.0, 20.0, 25.0, 30.0];
        let stats = MetricStats::compute(MetricType::FrameTime, &values);

        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
        assert_eq!(stats.avg, 20.0);
        assert_eq!(stats.samples, 5);
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new(100);

        for i in 0..10 {
            monitor.record(MetricType::FrameTime, 16.0 + i as f64, "ms");
            monitor.record(MetricType::DrawCalls, 500.0 + i as f64 * 10.0, "calls");
        }

        let report = monitor.generate_report();

        assert_eq!(report.stats.len(), 2);
        assert!(report.stats.contains_key(&MetricType::FrameTime));
        assert!(report.stats.contains_key(&MetricType::DrawCalls));
    }

    #[test]
    fn test_issue_detection() {
        let mut monitor = PerformanceMonitor::new(100);

        // 模拟低帧率
        for _ in 0..10 {
            monitor.record(MetricType::FrameTime, 50.0, "ms"); // 20fps
        }

        let report = monitor.generate_report();
        let issues = report.detect_issues();

        assert!(!issues.is_empty());
        assert!(issues[0].message.contains("Low frame rate"));
    }

    #[test]
    fn test_recommendations() {
        let mut monitor = PerformanceMonitor::new(100);

        for _ in 0..10 {
            monitor.record(MetricType::DrawCalls, 1000.0, "calls");
        }

        let report = monitor.generate_report();
        let recommendations = OptimizationRecommendation::generate_recommendations(&report);

        assert!(!recommendations.is_empty());
    }
}
