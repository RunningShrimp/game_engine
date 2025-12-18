//! 性能仪表板
//!
//! 实时性能数据可视化
//! - 指标跟踪
//! - 趋势分析
//! - 告警系统
//! - 历史数据

use std::collections::VecDeque;
use std::time::SystemTime;

/// 性能指标快照
#[derive(Debug, Clone)]
pub struct MetricSnapshot {
    /// 指标名称
    pub name: String,
    /// 数值
    pub value: f64,
    /// 单位
    pub unit: String,
    /// 时间戳
    pub timestamp: u64,
}

impl MetricSnapshot {
    /// 创建新快照
    pub fn new(name: String, value: f64, unit: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            name,
            value,
            unit,
            timestamp,
        }
    }
}

/// 性能告警
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    /// 告警级别
    pub level: AlertLevel,
    /// 告警消息
    pub message: String,
    /// 关联指标
    pub metric: String,
    /// 当前值
    pub current_value: f64,
    /// 阈值
    pub threshold: f64,
}

/// 告警级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertLevel {
    /// 信息
    Info = 0,
    /// 警告
    Warning = 1,
    /// 严重
    Critical = 2,
}

impl PerformanceAlert {
    /// 创建新告警
    pub fn new(
        level: AlertLevel,
        message: String,
        metric: String,
        current_value: f64,
        threshold: f64,
    ) -> Self {
        Self {
            level,
            message,
            metric,
            current_value,
            threshold,
        }
    }
}

/// 性能指标追踪器
pub struct MetricTracker {
    /// 指标历史
    history: VecDeque<MetricSnapshot>,
    /// 最大历史记录数
    max_history: usize,
    /// 告警列表
    alerts: Vec<PerformanceAlert>,
    /// 警告阈值
    warning_threshold: f64,
    /// 严重阈值
    critical_threshold: f64,
}

impl MetricTracker {
    /// 创建新追踪器
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
            alerts: Vec::new(),
            warning_threshold: 1.5,  // 正常值的 1.5 倍
            critical_threshold: 3.0, // 正常值的 3 倍
        }
    }

    /// 记录指标
    pub fn record_metric(&mut self, snapshot: MetricSnapshot) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }

        // 检查告警条件
        if let Some(avg) = self.get_average() {
            if snapshot.value > self.critical_threshold * avg {
                self.alerts.push(PerformanceAlert::new(
                    AlertLevel::Critical,
                    format!("指标 {} 超过严重阈值", snapshot.name),
                    snapshot.name.clone(),
                    snapshot.value,
                    self.critical_threshold * avg,
                ));
            } else if snapshot.value > self.warning_threshold * avg {
                self.alerts.push(PerformanceAlert::new(
                    AlertLevel::Warning,
                    format!("指标 {} 超过警告阈值", snapshot.name),
                    snapshot.name.clone(),
                    snapshot.value,
                    self.warning_threshold * avg,
                ));
            }
        }

        self.history.push_back(snapshot);
    }

    /// 获取平均值
    pub fn get_average(&self) -> Option<f64> {
        if self.history.is_empty() {
            return None;
        }

        let sum: f64 = self.history.iter().map(|s| s.value).sum();
        Some(sum / self.history.len() as f64)
    }

    /// 获取最大值
    pub fn get_max(&self) -> Option<f64> {
        self.history
            .iter()
            .map(|s| s.value)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    /// 获取最小值
    pub fn get_min(&self) -> Option<f64> {
        self.history
            .iter()
            .map(|s| s.value)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    /// 计算趋势（上升/下降/平稳）
    pub fn get_trend(&self) -> Trend {
        if self.history.len() < 2 {
            return Trend::Stable;
        }

        let recent_avg: f64 = self
            .history
            .iter()
            .rev()
            .take(self.history.len() / 2)
            .map(|s| s.value)
            .sum::<f64>()
            / ((self.history.len() / 2).max(1) as f64);

        let older_avg: f64 = self
            .history
            .iter()
            .take(self.history.len() / 2)
            .map(|s| s.value)
            .sum::<f64>()
            / ((self.history.len() / 2).max(1) as f64);

        let diff_percent = ((recent_avg - older_avg) / older_avg.max(1.0)) * 100.0;

        if diff_percent > 5.0 {
            Trend::Increasing
        } else if diff_percent < -5.0 {
            Trend::Decreasing
        } else {
            Trend::Stable
        }
    }

    /// 获取最近的告警
    pub fn get_recent_alerts(&self, count: usize) -> Vec<PerformanceAlert> {
        self.alerts.iter().rev().take(count).cloned().collect()
    }

    /// 清空告警
    pub fn clear_alerts(&mut self) {
        self.alerts.clear();
    }

    /// 获取指标摘要
    pub fn get_summary(&self) -> MetricSummary {
        MetricSummary {
            count: self.history.len(),
            average: self.get_average(),
            max: self.get_max(),
            min: self.get_min(),
            trend: self.get_trend(),
            alert_count: self.alerts.len(),
        }
    }
}

/// 趋势
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    /// 上升
    Increasing,
    /// 下降
    Decreasing,
    /// 平稳
    Stable,
}

/// 指标摘要
#[derive(Debug, Clone)]
pub struct MetricSummary {
    /// 数据点数
    pub count: usize,
    /// 平均值
    pub average: Option<f64>,
    /// 最大值
    pub max: Option<f64>,
    /// 最小值
    pub min: Option<f64>,
    /// 趋势
    pub trend: Trend,
    /// 告警数
    pub alert_count: usize,
}

/// 完整的性能仪表板
pub struct PerformanceDashboard {
    /// 指标追踪器映射
    trackers: std::collections::HashMap<String, MetricTracker>,
    /// 全局告警
    global_alerts: Vec<PerformanceAlert>,
}

impl PerformanceDashboard {
    /// 创建新仪表板
    pub fn new() -> Self {
        Self {
            trackers: std::collections::HashMap::new(),
            global_alerts: Vec::new(),
        }
    }

    /// 添加追踪器
    pub fn add_tracker(&mut self, name: String, max_history: usize) {
        self.trackers.insert(name, MetricTracker::new(max_history));
    }

    /// 记录指标
    pub fn record_metric(&mut self, tracker_name: String, snapshot: MetricSnapshot) {
        if let Some(tracker) = self.trackers.get_mut(&tracker_name) {
            tracker.record_metric(snapshot);

            // 传播告警到全局
            if let Some(tracker) = self.trackers.get(&tracker_name) {
                for alert in tracker.get_recent_alerts(5) {
                    if alert.level >= AlertLevel::Warning {
                        self.global_alerts.push(alert);
                    }
                }
            }
        }
    }

    /// 获取指标摘要
    pub fn get_metric_summary(&self, tracker_name: &str) -> Option<MetricSummary> {
        self.trackers.get(tracker_name).map(|t| t.get_summary())
    }

    /// 获取所有指标摘要
    pub fn get_all_summaries(&self) -> std::collections::HashMap<String, MetricSummary> {
        self.trackers
            .iter()
            .map(|(name, tracker)| (name.clone(), tracker.get_summary()))
            .collect()
    }

    /// 获取全局告警
    pub fn get_alerts(&self) -> Vec<PerformanceAlert> {
        self.global_alerts.clone()
    }

    /// 生成仪表板报告
    pub fn generate_report(&self) -> String {
        let mut report = String::from("# 性能仪表板报告\n\n");

        report.push_str("## 指标概览\n\n");

        let summaries = self.get_all_summaries();
        for (name, summary) in summaries {
            report.push_str(&format!("### {}\n", name));
            report.push_str(&format!("- 数据点: {}\n", summary.count));
            if let Some(avg) = summary.average {
                report.push_str(&format!("- 平均值: {:.2}\n", avg));
            }
            if let Some(max) = summary.max {
                report.push_str(&format!("- 最大值: {:.2}\n", max));
            }
            if let Some(min) = summary.min {
                report.push_str(&format!("- 最小值: {:.2}\n", min));
            }
            report.push_str(&format!("- 趋势: {:?}\n", summary.trend));
            report.push_str(&format!("- 告警: {}\n\n", summary.alert_count));
        }

        if !self.global_alerts.is_empty() {
            report.push_str("## 活跃告警\n\n");
            for alert in &self.global_alerts {
                report.push_str(&format!(
                    "- [{:?}] {}: {}/{}\n",
                    alert.level, alert.metric, alert.current_value, alert.threshold
                ));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_snapshot() {
        let snapshot = MetricSnapshot::new("fps".to_string(), 60.0, "fps".to_string());
        assert_eq!(snapshot.name, "fps");
        assert_eq!(snapshot.value, 60.0);
    }

    #[test]
    fn test_metric_tracker() {
        let mut tracker = MetricTracker::new(10);

        for i in 0..10 {
            let snapshot =
                MetricSnapshot::new("fps".to_string(), 50.0 + i as f64, "fps".to_string());
            tracker.record_metric(snapshot);
        }

        assert!(tracker.get_average().is_some());
        assert!(tracker.get_max().is_some());
        assert!(tracker.get_min().is_some());
    }

    #[test]
    fn test_trend_detection() {
        let mut tracker = MetricTracker::new(100);

        // 模拟上升趋势
        for i in 0..50 {
            let snapshot =
                MetricSnapshot::new("fps".to_string(), 30.0 + i as f64 * 0.1, "fps".to_string());
            tracker.record_metric(snapshot);
        }

        let trend = tracker.get_trend();
        assert_eq!(trend, Trend::Increasing);
    }

    #[test]
    fn test_performance_alert() {
        let alert = PerformanceAlert::new(
            AlertLevel::Critical,
            "High FPS latency".to_string(),
            "fps".to_string(),
            100.0,
            50.0,
        );

        assert_eq!(alert.level, AlertLevel::Critical);
        assert!(alert.current_value > alert.threshold);
    }

    #[test]
    fn test_performance_dashboard() {
        let mut dashboard = PerformanceDashboard::new();
        dashboard.add_tracker("fps".to_string(), 100);

        for i in 0..10 {
            let snapshot =
                MetricSnapshot::new("fps".to_string(), 60.0 + i as f64, "fps".to_string());
            dashboard.record_metric("fps".to_string(), snapshot);
        }

        let summary = dashboard.get_metric_summary("fps");
        assert!(summary.is_some());
    }

    #[test]
    fn test_dashboard_report() {
        let mut dashboard = PerformanceDashboard::new();
        dashboard.add_tracker("fps".to_string(), 10);

        for _ in 0..5 {
            let snapshot = MetricSnapshot::new("fps".to_string(), 60.0, "fps".to_string());
            dashboard.record_metric("fps".to_string(), snapshot);
        }

        let report = dashboard.generate_report();
        assert!(report.contains("fps"));
    }
}
