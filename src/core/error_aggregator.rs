//! 错误聚合和报告模块
//!
//! 提供错误统计、聚合和可视化功能。

use crate::core::error::EngineError;
use crate::impl_default;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 错误统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    /// 错误总数
    pub total_count: u64,
    /// 按错误类型分组的计数
    pub by_type: HashMap<String, u64>,
    /// 按错误来源分组的计数
    pub by_source: HashMap<String, u64>,
    /// 最近发生的错误（最多保留N条）
    pub recent_errors: Vec<ErrorRecord>,
    /// 错误率（每秒）
    pub error_rate: f64,
    /// 最后更新时间戳
    pub last_updated: u64,
}

impl Default for ErrorStats {
    fn default() -> Self {
        Self {
            total_count: 0,
            by_type: HashMap::new(),
            by_source: HashMap::new(),
            recent_errors: Vec::new(),
            error_rate: 0.0,
            last_updated: Self::current_timestamp(),
        }
    }
}

impl ErrorStats {
    pub fn new() -> Self {
        Self::default()
    }

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }

    /// 获取最常见的错误类型
    pub fn most_common_error_type(&self) -> Option<(&String, &u64)> {
        self.by_type.iter().max_by_key(|(_, count)| *count)
    }

    /// 获取最常见的错误来源
    pub fn most_common_error_source(&self) -> Option<(&String, &u64)> {
        self.by_source.iter().max_by_key(|(_, count)| *count)
    }

    /// 获取错误趋势（最近N秒内的错误数）
    pub fn error_trend(&self, seconds: u64) -> u64 {
        let cutoff = Self::current_timestamp().saturating_sub(seconds);
        self.recent_errors
            .iter()
            .filter(|record| record.timestamp >= cutoff)
            .count() as u64
    }
}

/// 错误记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    /// 错误类型
    pub error_type: String,
    /// 错误来源（模块名）
    pub source: String,
    /// 错误消息
    pub message: String,
    /// 时间戳（秒）
    pub timestamp: u64,
    /// 错误详情（可选）
    pub details: Option<String>,
}

impl ErrorRecord {
    pub fn new(
        error_type: impl Into<String>,
        source: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            error_type: error_type.into(),
            source: source.into(),
            message: message.into(),
            timestamp: Self::current_timestamp(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

/// 错误聚合器
///
/// 收集、统计和报告引擎中的错误信息。
/// 线程安全，可以在多个线程中并发使用。
#[derive(bevy_ecs::prelude::Resource)]
pub struct ErrorAggregator {
    /// 错误统计（线程安全）
    stats: Arc<Mutex<ErrorStats>>,
    /// 最大保留的错误记录数
    max_recent_errors: usize,
    /// 错误率计算窗口（秒）
    error_rate_window: u64,
}

impl_default!(ErrorAggregator {
    stats: Arc::new(Mutex::new(ErrorStats::default())),
    max_recent_errors: 1000,
    error_rate_window: 60,
});

impl ErrorAggregator {
    /// 创建新的错误聚合器
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建带配置的错误聚合器
    pub fn with_config(max_recent_errors: usize, error_rate_window: u64) -> Self {
        Self {
            stats: Arc::new(Mutex::new(ErrorStats::default())),
            max_recent_errors,
            error_rate_window,
        }
    }

    /// 记录错误
    pub fn record_error(&self, error: &EngineError, source: impl Into<String>) {
        let source_str = source.into();
        let error_type = self.error_type_name(error);
        let message = error.to_string();

        let record = ErrorRecord::new(&error_type, &source_str, &message);

        let mut stats = self.stats.lock().unwrap();
        stats.total_count += 1;

        // 更新按类型统计
        *stats.by_type.entry(error_type.clone()).or_insert(0) += 1;

        // 更新按来源统计
        *stats.by_source.entry(source_str.clone()).or_insert(0) += 1;

        // 添加最近错误记录
        stats.recent_errors.push(record);
        if stats.recent_errors.len() > self.max_recent_errors {
            stats.recent_errors.remove(0);
        }

        // 计算错误率
        stats.error_rate = self.calculate_error_rate(&stats);
        stats.last_updated = ErrorStats::current_timestamp();
    }

    /// 记录自定义错误
    pub fn record_custom_error(
        &self,
        error_type: impl Into<String>,
        source: impl Into<String>,
        message: impl Into<String>,
        details: Option<String>,
    ) {
        let source_str = source.into();
        let error_type_str = error_type.into();
        let message_str = message.into();

        let mut record = ErrorRecord::new(&error_type_str, &source_str, &message_str);
        if let Some(d) = details {
            record = record.with_details(d);
        }

        let mut stats = self.stats.lock().unwrap();
        stats.total_count += 1;

        *stats.by_type.entry(error_type_str.clone()).or_insert(0) += 1;
        *stats.by_source.entry(source_str.clone()).or_insert(0) += 1;

        stats.recent_errors.push(record);
        if stats.recent_errors.len() > self.max_recent_errors {
            stats.recent_errors.remove(0);
        }

        stats.error_rate = self.calculate_error_rate(&stats);
        stats.last_updated = ErrorStats::current_timestamp();
    }

    /// 获取错误统计
    pub fn get_stats(&self) -> ErrorStats {
        self.stats.lock().unwrap().clone()
    }

    /// 获取错误摘要
    pub fn get_summary(&self) -> ErrorSummary {
        let stats = self.stats.lock().unwrap();
        ErrorSummary {
            total_errors: stats.total_count,
            error_rate: stats.error_rate,
            most_common_type: stats.most_common_error_type().map(|(t, c)| (t.clone(), *c)),
            most_common_source: stats
                .most_common_error_source()
                .map(|(s, c)| (s.clone(), *c)),
            recent_error_count: stats.recent_errors.len(),
            last_updated: stats.last_updated,
        }
    }

    /// 清除所有统计
    pub fn clear(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = ErrorStats::default();
    }

    /// 导出错误报告（JSON格式）
    pub fn export_report(&self) -> Result<String, serde_json::Error> {
        let stats = self.get_stats();
        serde_json::to_string_pretty(&stats)
    }

    /// 导出错误报告到文件
    pub fn export_report_to_file(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), std::io::Error> {
        let report = self
            .export_report()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, report)
    }

    /// 获取错误类型名称
    fn error_type_name(&self, error: &EngineError) -> String {
        match error {
            EngineError::Init(_) => "Init".to_string(),
            EngineError::Render(_) => "Render".to_string(),
            EngineError::Asset(_) => "Asset".to_string(),
            EngineError::Physics(_) => "Physics".to_string(),
            EngineError::Audio(_) => "Audio".to_string(),
            EngineError::Script(_) => "Script".to_string(),
            EngineError::Platform(_) => "Platform".to_string(),
            EngineError::Window(_) => "Window".to_string(),
            EngineError::RenderInit(_) => "RenderInit".to_string(),
            EngineError::EventLoop(_) => "EventLoop".to_string(),
            EngineError::Io(_) => "Io".to_string(),
            EngineError::General(_) => "General".to_string(),
        }
    }

    /// 计算错误率（每秒）
    fn calculate_error_rate(&self, stats: &ErrorStats) -> f64 {
        if stats.recent_errors.is_empty() {
            return 0.0;
        }

        let now = ErrorStats::current_timestamp();
        let window_start = now.saturating_sub(self.error_rate_window);

        let errors_in_window = stats
            .recent_errors
            .iter()
            .filter(|record| record.timestamp >= window_start)
            .count();

        errors_in_window as f64 / self.error_rate_window as f64
    }
}

/// 错误摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSummary {
    /// 错误总数
    pub total_errors: u64,
    /// 错误率（每秒）
    pub error_rate: f64,
    /// 最常见的错误类型
    pub most_common_type: Option<(String, u64)>,
    /// 最常见的错误来源
    pub most_common_source: Option<(String, u64)>,
    /// 最近错误数量
    pub recent_error_count: usize,
    /// 最后更新时间戳
    pub last_updated: u64,
}

impl ErrorSummary {
    /// 格式化错误摘要为字符串
    pub fn format(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("总错误数: {}", self.total_errors));
        lines.push(format!("错误率: {:.2} 错误/秒", self.error_rate));

        if let Some((ref error_type, count)) = self.most_common_type {
            lines.push(format!("最常见错误类型: {} ({}次)", error_type, count));
        }

        if let Some((ref source, count)) = self.most_common_source {
            lines.push(format!("最常见错误来源: {} ({}次)", source, count));
        }

        lines.push(format!("最近错误数: {}", self.recent_error_count));

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::{AssetError, RenderError};

    #[test]
    fn test_error_aggregator() {
        let aggregator = ErrorAggregator::new();

        // 记录一些错误
        let render_err = EngineError::Render(RenderError::NoAdapter);
        aggregator.record_error(&render_err, "render_system");

        let asset_err = EngineError::Asset(AssetError::NotFound {
            path: "test.png".to_string(),
        });
        aggregator.record_error(&asset_err, "asset_manager");

        // 获取统计
        let stats = aggregator.get_stats();
        assert_eq!(stats.total_count, 2);
        assert_eq!(stats.by_type.get("Render"), Some(&1));
        assert_eq!(stats.by_type.get("Asset"), Some(&1));
        assert_eq!(stats.by_source.get("render_system"), Some(&1));
        assert_eq!(stats.by_source.get("asset_manager"), Some(&1));
    }

    #[test]
    fn test_error_summary() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_custom_error("TestError", "test_module", "Test message", None);

        let summary = aggregator.get_summary();
        assert_eq!(summary.total_errors, 1);
        assert_eq!(summary.most_common_type, Some(("TestError".to_string(), 1)));
        assert_eq!(
            summary.most_common_source,
            Some(("test_module".to_string(), 1))
        );
    }

    #[test]
    fn test_error_export() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_custom_error("TestError", "test_module", "Test message", None);

        let report = aggregator.export_report().unwrap();
        assert!(report.contains("TestError"));
        assert!(report.contains("test_module"));
    }
}
