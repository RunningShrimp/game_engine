//  错误监控和报告机制
//
//  提供错误统计收集、分析和报告功能，支持实时监控和历史追踪。

use crate::error::{EngineError, ErrorCategory, ErrorSeverity, safe_lock};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// 错误统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    /// 总错误数
    pub total_errors: u64,
    /// 按严重级别分类的错误数
    pub errors_by_severity: HashMap<ErrorSeverity, u64>,
    /// 按分类分类的错误数
    pub errors_by_category: HashMap<ErrorCategory, u64>,
    /// 按小时分类的错误数
    pub errors_by_hour: HashMap<String, u64>,
    /// 平均错误率（错误/小时）
    pub error_rate_per_hour: f64,
    /// 最后更新时间
    pub last_updated: std::time::SystemTime,
}

impl Default for ErrorStats {
    fn default() -> Self {
        Self {
            total_errors: 0,
            errors_by_severity: HashMap::new(),
            errors_by_category: HashMap::new(),
            errors_by_hour: HashMap::new(),
            error_rate_per_hour: 0.0,
            last_updated: std::time::SystemTime::now(),
        }
    }
}

impl ErrorStats {
    /// 创建新的错误统计
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录错误
    pub fn record_error(&mut self, error: &EngineError) {
        self.total_errors += 1;

        // 按严重级别统计
        *self.errors_by_severity.entry(error.severity()).or_insert(0) += 1;

        // 按分类统计
        *self.errors_by_category.entry(error.category()).or_insert(0) += 1;

        // 按小时统计
        let now = std::time::SystemTime::now();
        let hour_key = format!(
            "{:02}:00",
            now.elapsed().unwrap_or_default().as_secs() / 3600 % 24
        );
        *self.errors_by_hour.entry(hour_key).or_insert(0) += 1;

        // 更新最后时间
        self.last_updated = now;

        // 计算错误率
        self.update_error_rate();
    }

    /// 更新错误率
    fn update_error_rate(&mut self) {
        if self.errors_by_hour.is_empty() {
            self.error_rate_per_hour = 0.0;
            return;
        }

        let total_hourly_errors: u64 = self.errors_by_hour.values().sum();
        let hours_with_data = self.errors_by_hour.len() as f64;

        if hours_with_data > 0.0 {
            self.error_rate_per_hour = total_hourly_errors as f64 / hours_with_data;
        }
    }

    /// 获取指定时间范围内的错误数
    pub fn errors_in_time_range(&self, duration: std::time::Duration) -> u64 {
        self.errors_by_hour
            .iter()
            .filter(|(hour, _)| {
                // 解析小时并计算时间戳
                if let Some((hours, _)) = hour.split_once(':') {
                    if let Ok(h) = hours.parse::<u32>() {
                        let hour_timestamp = h as u64 * 3600;
                        let current_timestamp = self
                            .last_updated
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        // 检查是否在时间范围内
                        current_timestamp.saturating_sub(duration.as_secs()) <= hour_timestamp
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .map(|(_, &count)| count)
            .sum()
    }

    /// 获取最频繁的错误类型
    pub fn most_frequent_severity(&self) -> Option<ErrorSeverity> {
        self.errors_by_severity
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(severity, _)| *severity)
    }

    /// 获取最频繁的错误分类
    pub fn most_frequent_category(&self) -> Option<ErrorCategory> {
        self.errors_by_category
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(category, _)| *category)
    }
}

/// 错误报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    /// 报告ID
    pub id: String,
    /// 报告时间
    pub timestamp: std::time::SystemTime,
    /// 时间范围
    pub time_range: std::time::Duration,
    /// 错误统计
    pub stats: ErrorStats,
    /// 错误详情（最近的错误）
    pub recent_errors: Vec<ErrorDetail>,
    /// 错误趋势
    pub trends: Vec<ErrorTrend>,
    /// 建议和洞察
    pub insights: Vec<String>,
}

/// 错误详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// 错误ID
    pub id: String,
    /// 错误消息
    pub message: String,
    /// 错误严重级别
    pub severity: ErrorSeverity,
    /// 错误分类
    pub category: ErrorCategory,
    /// 发生时间
    pub timestamp: std::time::SystemTime,
    /// 错误上下文
    pub context: HashMap<String, String>,
    /// 错误堆栈
    pub stack_trace: Option<String>,
    /// 恢复尝试次数
    pub recovery_attempts: u32,
    /// 是否已恢复
    pub recovered: bool,
}

/// 错误趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTrend {
    /// 趋势类型
    pub trend_type: TrendType,
    /// 时间范围
    pub time_range: std::time::Duration,
    /// 变化率
    pub change_rate: f64,
    /// 趋势描述
    pub description: String,
}

/// 趋势类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendType {
    /// 增加
    Increasing,
    /// 减少
    Decreasing,
    /// 稳定
    Stable,
    /// 波动
    Fluctuating,
}

/// 错误监控器
pub struct ErrorMonitor {
    /// 错误历史（保留最近的错误）
    error_history: Arc<Mutex<VecDeque<ErrorDetail>>>,
    /// 错误统计
    stats: Arc<Mutex<ErrorStats>>,
    /// 配置
    config: MonitorConfig,
    /// 报告生成器
    report_generators: Vec<Box<dyn ErrorReportGenerator>>,
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// 最大历史记录数
    pub max_history_size: usize,
    /// 统计更新间隔
    pub stats_update_interval: std::time::Duration,
    /// 自动报告间隔
    pub auto_report_interval: std::time::Duration,
    /// 是否启用实时监控
    pub enable_real_time_monitoring: bool,
    /// 错误阈值配置
    pub thresholds: ErrorThresholds,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            max_history_size: 1000,
            stats_update_interval: Duration::from_secs(60),
            auto_report_interval: Duration::from_secs(300), // 5分钟
            enable_real_time_monitoring: true,
            thresholds: ErrorThresholds::default(),
        }
    }
}

/// 错误阈值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorThresholds {
    /// 错误率阈值（错误/分钟）
    pub error_rate_threshold: f64,
    /// 严重错误阈值
    pub critical_error_threshold: u32,
    /// 总错误数阈值
    pub total_error_threshold: u32,
    /// 时间窗口（分钟）
    pub time_window_minutes: u32,
}

impl Default for ErrorThresholds {
    fn default() -> Self {
        Self {
            error_rate_threshold: 1.0,   // 1个错误/分钟
            critical_error_threshold: 5, // 5个严重错误
            total_error_threshold: 100,  // 100个总错误
            time_window_minutes: 5,      // 5分钟窗口
        }
    }
}

/// 错误报告生成器
pub trait ErrorReportGenerator {
    /// 生成报告
    fn generate_report(&self, monitor: &ErrorMonitor) -> ErrorReport;
    /// 获取生成器名称
    fn name(&self) -> &str;
}

/// 默认错误报告生成器
pub struct DefaultReportGenerator;

impl DefaultReportGenerator {
    /// Creates a new instance of the default error report generator
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorReportGenerator for DefaultReportGenerator {
    fn generate_report(&self, monitor: &ErrorMonitor) -> ErrorReport {
        let stats = monitor.get_stats();
        let recent_errors = monitor.get_recent_errors(50);
        let trends = monitor.analyze_trends();
        let insights = monitor.generate_insights(&stats, &trends);

        ErrorReport {
            id: format!(
                "report_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
            timestamp: std::time::SystemTime::now(),
            time_range: Duration::from_secs(3600), // 1小时
            stats,
            recent_errors,
            trends,
            insights,
        }
    }

    fn name(&self) -> &str {
        "DefaultReportGenerator"
    }
}

impl ErrorMonitor {
    /// 创建新的错误监控器
    pub fn new() -> Self {
        Self::with_config(MonitorConfig::default())
    }

    /// 使用配置创建错误监控器
    pub fn with_config(config: MonitorConfig) -> Self {
        Self {
            error_history: Arc::new(Mutex::new(VecDeque::new())),
            stats: Arc::new(Mutex::new(ErrorStats::new())),
            config,
            report_generators: vec![Box::new(DefaultReportGenerator::new())],
        }
    }

    /// 记录错误
    pub fn record_error(&self, error: EngineError) {
        let error_detail = ErrorDetail {
            id: format!(
                "error_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
            message: error.to_string(),
            severity: error.severity(),
            category: error.category(),
            timestamp: std::time::SystemTime::now(),
            context: HashMap::new(),
            stack_trace: None, // 可以通过其他方式获取
            recovery_attempts: 0,
            recovered: false,
        };

        // 添加到历史记录
        {
            let history =
                &mut safe_lock(&self.error_history, "ErrorMonitor.error_history").unwrap();
            history.push_front(error_detail);

            // 限制历史记录大小
            while history.len() > self.config.max_history_size {
                history.pop_back();
            }
        }

        // 更新统计
        {
            let stats = &mut safe_lock(&self.stats, "ErrorMonitor.stats").unwrap();
            stats.record_error(&error);
        }

        // 检查阈值
        self.check_thresholds(&error);
    }

    /// 记录带上下文的错误
    pub fn record_error_with_context(&self, error: EngineError, context: HashMap<String, String>) {
        let error_detail = ErrorDetail {
            id: format!(
                "error_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
            message: error.to_string(),
            severity: error.severity(),
            category: error.category(),
            timestamp: std::time::SystemTime::now(),
            context,
            stack_trace: None,
            recovery_attempts: 0,
            recovered: false,
        };

        // 添加到历史记录
        {
            let history =
                &mut safe_lock(&self.error_history, "ErrorMonitor.error_history").unwrap();
            history.push_front(error_detail);

            // 限制历史记录大小
            while history.len() > self.config.max_history_size {
                history.pop_back();
            }
        }

        // 更新统计
        {
            let Ok(mut stats) = safe_lock(&self.stats, "ErrorMonitor.stats") else {
                tracing::error!("Failed to acquire stats lock in record_error");
                return;
            };
            stats.record_error(&error);
        }

        // 检查阈值
        self.check_thresholds(&error);
    }

    /// 记录恢复尝试
    pub fn record_recovery_attempt(&self, error_id: &str) {
        if let Some(mut error_detail) = self.get_error_detail(error_id) {
            error_detail.recovery_attempts += 1;
            self.update_error_detail(error_detail);
        }
    }

    /// 记录恢复成功
    pub fn record_recovery_success(&self, error_id: &str) {
        if let Some(mut error_detail) = self.get_error_detail(error_id) {
            error_detail.recovered = true;
            self.update_error_detail(error_detail);
        }
    }

    /// 获取错误详情
    fn get_error_detail(&self, error_id: &str) -> Option<ErrorDetail> {
        let history = &self.error_history.lock().ok()?;
        history.iter().find(|e| e.id == error_id).cloned()
    }

    /// 更新错误详情
    fn update_error_detail(&self, error_detail: ErrorDetail) {
        let Ok(mut history) = safe_lock(&self.error_history, "ErrorMonitor.error_history") else {
            tracing::error!("Failed to acquire error_history lock in update_error_detail");
            return;
        };
        if let Some(pos) = history.iter().position(|e| e.id == error_detail.id) {
            history[pos] = error_detail;
        }
    }

    /// 获取错误统计
    pub fn get_stats(&self) -> ErrorStats {
        safe_lock(&self.stats, "ErrorMonitor.stats").unwrap().clone()
    }

    /// 获取最近的错误
    pub fn get_recent_errors(&self, count: usize) -> Vec<ErrorDetail> {
        let history = &safe_lock(&self.error_history, "ErrorMonitor.error_history").unwrap();
        history.iter().take(count).cloned().collect()
    }

    /// 分析趋势
    pub fn analyze_trends(&self) -> Vec<ErrorTrend> {
        let stats = self.get_stats();
        let mut trends = Vec::new();

        // 分析错误率趋势
        if stats.errors_by_hour.len() > 1 {
            let error_rates: Vec<f64> =
                stats.errors_by_hour.iter().map(|(_, &count)| count as f64).collect();

            let avg_rate = error_rates.iter().sum::<f64>() / error_rates.len() as f64;
            let variance = error_rates.iter().map(|rate| (rate - avg_rate).powi(2)).sum::<f64>()
                / error_rates.len() as f64;
            let std_dev = variance.sqrt();

            // 简单趋势分析
            let trend_type = if std_dev < 0.1 {
                TrendType::Stable
            } else if error_rates.last() > error_rates.first() {
                TrendType::Increasing
            } else {
                TrendType::Decreasing
            };

            trends.push(ErrorTrend {
                trend_type: trend_type.clone(),
                time_range: Duration::from_secs(3600),
                change_rate: std_dev,
                description: format!("Error rate trend: {:?}", trend_type),
            });
        }

        // 分析严重级别趋势
        if let Some(most_severe) = stats.most_frequent_severity() {
            trends.push(ErrorTrend {
                trend_type: TrendType::Fluctuating,
                time_range: Duration::from_secs(3600),
                change_rate: 0.0,
                description: format!("Most frequent severity: {:?}", most_severe),
            });
        }

        // 分析分类趋势
        if let Some(most_frequent) = stats.most_frequent_category() {
            trends.push(ErrorTrend {
                trend_type: TrendType::Fluctuating,
                time_range: Duration::from_secs(3600),
                change_rate: 0.0,
                description: format!("Most frequent category: {:?}", most_frequent),
            });
        }

        trends
    }

    /// 生成洞察
    pub fn generate_insights(&self, stats: &ErrorStats, trends: &[ErrorTrend]) -> Vec<String> {
        let mut insights = Vec::new();

        // 错误率洞察
        if stats.error_rate_per_hour > 10.0 {
            insights.push("High error rate detected: > 10 errors/hour".to_string());
        }

        // 严重错误洞察
        if let Some(critical_count) = stats.errors_by_severity.get(&ErrorSeverity::Critical)
            && *critical_count > 5 {
                insights.push("Critical error threshold exceeded: > 5 critical errors".to_string());
            }

        // 趋势洞察
        for trend in trends {
            match trend.trend_type {
                TrendType::Increasing => {
                    insights.push(format!("Increasing trend detected: {}", trend.description));
                }
                TrendType::Decreasing => {
                    insights.push(format!("Decreasing trend detected: {}", trend.description));
                }
                _ => {}
            }
        }

        // 总错误数洞察
        if stats.total_errors > 1000 {
            insights.push("High total error count: > 1000 errors".to_string());
        }

        insights
    }

    /// 检查阈值
    fn check_thresholds(&self, error: &EngineError) {
        let thresholds = &self.config.thresholds;

        // 检查严重错误阈值
        if error.severity() >= ErrorSeverity::Critical {
            let critical_count = &safe_lock(&self.stats, "ErrorMonitor.stats")
                .unwrap()
                .errors_by_severity
                .get(&ErrorSeverity::Critical)
                .copied()
                .unwrap_or(0);

            if *critical_count >= thresholds.critical_error_threshold as u64 {
                self.trigger_alert(format!(
                    "Critical error threshold exceeded: {} >= {}",
                    critical_count, thresholds.critical_error_threshold
                ));
            }
        }

        // 检查总错误数阈值
        let total_errors = &safe_lock(&self.stats, "ErrorMonitor.stats").unwrap().total_errors;
        if *total_errors >= thresholds.total_error_threshold as u64 {
            self.trigger_alert(format!(
                "Total error threshold exceeded: {} >= {}",
                total_errors, thresholds.total_error_threshold
            ));
        }
    }

    /// 触发告警
    fn trigger_alert(&self, message: String) {
        // 这里可以实现告警逻辑，如发送到监控系统、日志记录等
        eprintln!("ERROR ALERT: {}", message);

        // 可以扩展为实际的告警系统
        // 例如：发送到监控系统、邮件通知、Slack等
    }

    /// 生成报告
    pub fn generate_report(&self) -> ErrorReport {
        let mut reports = Vec::new();

        for generator in &self.report_generators {
            reports.push(generator.generate_report(self));
        }

        // 合并多个报告生成器的结果
        if reports.len() == 1 {
            reports.into_iter().next().unwrap()
        } else {
            // 简单合并：使用第一个生成器作为主报告
            reports.into_iter().next().unwrap()
        }
    }

    /// 添加报告生成器
    pub fn add_report_generator(&mut self, generator: Box<dyn ErrorReportGenerator>) {
        self.report_generators.push(generator);
    }

    /// 清除历史记录
    pub fn clear_history(&self) {
        if let Ok(mut history) = self.error_history.lock() {
            history.clear();
        }
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_errors = 0;
            stats.errors_by_severity.clear();
            stats.errors_by_category.clear();
            stats.errors_by_hour.clear();
            stats.error_rate_per_hour = 0.0;
        }
    }

    /// 启动后台监控线程
    pub fn start_monitoring(&self) {
        if !self.config.enable_real_time_monitoring {
            return;
        }

        let config = self.config.clone();
        let stats = self.stats.clone(); // 克隆stats以在线程中使用

        thread::spawn(move || {
            loop {
                thread::sleep(config.stats_update_interval);

                // 定期更新统计
                {
                    let mut stats = safe_lock(&stats, "ErrorMonitor.stats").unwrap();

                    // 这里可以定期重新计算错误率
                    stats.update_error_rate();
                }
            }
        });
    }
}

impl Default for ErrorMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_stats() {
        let mut stats = ErrorStats::new();

        let error1 = EngineError::general("Test error 1");
        let error2 = EngineError::general("Test error 2");
        let error3 = EngineError::general_with_severity("Critical error", ErrorSeverity::Critical);

        stats.record_error(&error1);
        stats.record_error(&error2);
        stats.record_error(&error3);

        assert_eq!(stats.total_errors, 3);
        assert_eq!(
            stats.errors_by_severity.get(&ErrorSeverity::Error),
            Some(&2)
        );
        assert_eq!(
            stats.errors_by_severity.get(&ErrorSeverity::Critical),
            Some(&1)
        );
        assert_eq!(stats.most_frequent_severity(), Some(ErrorSeverity::Error));
    }

    #[test]
    fn test_error_monitor() {
        let monitor = ErrorMonitor::new();

        let error = EngineError::general("Test error");
        monitor.record_error(error.clone());

        let stats = monitor.get_stats();
        assert_eq!(stats.total_errors, 1);

        let recent_errors = monitor.get_recent_errors(1);
        assert_eq!(recent_errors.len(), 1);
        assert_eq!(recent_errors[0].message, "Test error");
    }

    #[test]
    fn test_error_thresholds() {
        let config = MonitorConfig {
            thresholds: ErrorThresholds {
                critical_error_threshold: 2,
                ..Default::default()
            },
            ..Default::default()
        };

        let monitor = ErrorMonitor::with_config(config);

        // 记录临界错误
        for _ in 0..3 {
            monitor.record_error(EngineError::general_with_severity(
                "Critical error",
                ErrorSeverity::Critical,
            ));
        }

        let stats = monitor.get_stats();
        assert_eq!(
            stats.errors_by_severity.get(&ErrorSeverity::Critical),
            Some(&3)
        );
    }

    #[test]
    fn test_error_trends() {
        let monitor = ErrorMonitor::new();

        // 模拟不同时间的错误
        let error1 = EngineError::general("Error 1");
        let error2 = EngineError::general("Error 2");

        monitor.record_error(error1);
        monitor.record_error(error2);

        let trends = monitor.analyze_trends();
        assert!(!trends.is_empty());
    }

    #[test]
    fn test_error_insights() {
        let monitor = ErrorMonitor::new();

        // 模拟高错误率
        for i in 0..20 {
            monitor.record_error(EngineError::general(format!("Error {}", i)));
        }

        let stats = monitor.get_stats();
        let trends = monitor.analyze_trends();
        let insights = monitor.generate_insights(&stats, &trends);

        assert!(!insights.is_empty());
        assert!(insights.iter().any(|i| i.contains("High error rate")));
    }
}
