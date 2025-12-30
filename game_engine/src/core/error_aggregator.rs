//  错误聚合和报告模块
//
//  提供错误统计、聚合和可视化功能。

use crate::{
    error::{EngineError, safe_lock},
    impl_default,
};
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
    /// 创建新的错误统计信息
    ///
    /// # 返回
    ///
    /// 返回初始化的错误统计信息实例，所有计数器都设置为0。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::ErrorStats;
    ///
    /// let stats = ErrorStats::new();
    /// assert_eq!(stats.total_count, 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }

    /// 获取最常见的错误类型
    ///
    /// # 返回
    ///
    /// 返回最常见的错误类型及其计数，如果没有错误则返回`None`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::ErrorStats;
    ///
    /// let stats = ErrorStats::new();
    /// if let Some((error_type, count)) = stats.most_common_error_type() {
    ///     println!("最常见的错误类型: {} ({}次)", error_type, count);
    /// }
    /// ```
    pub fn most_common_error_type(&self) -> Option<(&String, &u64)> {
        self.by_type.iter().max_by_key(|(_, count)| *count)
    }

    /// 获取最常见的错误来源
    ///
    /// # 返回
    ///
    /// 返回最常见的错误来源（模块名）及其计数，如果没有错误则返回`None`。
    pub fn most_common_error_source(&self) -> Option<(&String, &u64)> {
        self.by_source.iter().max_by_key(|(_, count)| *count)
    }

    /// 获取错误趋势（最近N秒内的错误数）
    ///
    /// # 参数
    ///
    /// * `seconds` - 时间窗口（秒）
    ///
    /// # 返回
    ///
    /// 返回指定时间窗口内的错误数量。
    pub fn error_trend(&self, seconds: u64) -> u64 {
        let cutoff = Self::current_timestamp().saturating_sub(seconds);
        self.recent_errors.iter().filter(|record| record.timestamp >= cutoff).count() as u64
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
    /// 创建新的错误记录
    ///
    /// # 参数
    ///
    /// * `error_type` - 错误类型名称
    /// * `source` - 错误来源（模块名）
    /// * `message` - 错误消息
    ///
    /// # 返回
    ///
    /// 返回新创建的错误记录，时间戳自动设置为当前时间。
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

    /// 添加错误详情
    ///
    /// # 参数
    ///
    /// * `details` - 错误详情信息
    ///
    /// # 返回
    ///
    /// 返回更新后的错误记录。
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
    ///
    /// # 参数
    ///
    /// * `max_recent_errors` - 最大保留的错误记录数
    /// * `error_rate_window` - 错误率计算窗口（秒）
    ///
    /// # 返回
    ///
    /// 返回配置好的错误聚合器实例。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::ErrorAggregator;
    ///
    /// // 创建保留500条错误记录、60秒窗口的聚合器
    /// let aggregator = ErrorAggregator::with_config(500, 60);
    /// ```
    pub fn with_config(max_recent_errors: usize, error_rate_window: u64) -> Self {
        Self {
            stats: Arc::new(Mutex::new(ErrorStats::default())),
            max_recent_errors,
            error_rate_window,
        }
    }

    /// 记录错误
    ///
    /// 将错误记录到聚合器中，更新统计信息。
    ///
    /// # 参数
    ///
    /// * `error` - 要记录的错误
    /// * `source` - 错误来源（模块名）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::ErrorAggregator;
    /// use game_engine::error::EngineError;
    ///
    /// let aggregator = ErrorAggregator::new();
    /// let error = EngineError::general("Test error");
    /// aggregator.record_error(&error, "test_module");
    /// ```
    pub fn record_error(&self, error: &EngineError, source: impl Into<String>) {
        let source_str = source.into();
        let error_type = self.error_type_name(error);
        let message = error.to_string();

        let record = ErrorRecord::new(&error_type, &source_str, &message);

        let Ok(mut stats) = safe_lock(&self.stats, "ErrorAggregator.stats") else {
            tracing::error!("Failed to acquire stats lock in record_error");
            return;
        };
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
    ///
    /// 记录一个自定义错误，不依赖于`EngineError`类型。
    ///
    /// # 参数
    ///
    /// * `error_type` - 错误类型名称
    /// * `source` - 错误来源（模块名）
    /// * `message` - 错误消息
    /// * `details` - 可选的错误详情
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::ErrorAggregator;
    ///
    /// let aggregator = ErrorAggregator::new();
    /// aggregator.record_custom_error(
    ///     "CustomError",
    ///     "my_module",
    ///     "Something went wrong",
    ///     Some("Additional details".to_string())
    /// );
    /// ```
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

        let Ok(mut stats) = safe_lock(&self.stats, "ErrorAggregator.stats") else {
            tracing::error!("Failed to acquire stats lock in record_custom_error");
            return;
        };
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
    ///
    /// 获取当前的错误统计信息快照。
    ///
    /// # 返回
    ///
    /// 返回错误统计信息的克隆副本。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::ErrorAggregator;
    /// use game_engine::error::EngineError;
    ///
    /// let aggregator = ErrorAggregator::new();
    /// aggregator.record_error(&EngineError::general("Test"), "module");
    ///
    /// let stats = aggregator.get_stats();
    /// assert_eq!(stats.total_count, 1);
    /// ```
    pub fn get_stats(&self) -> ErrorStats {
        match safe_lock(&self.stats, "ErrorAggregator.stats") {
            Ok(stats) => stats.clone(),
            Err(e) => {
                tracing::error!("Failed to acquire stats lock in get_stats: {:?}", e);
                // Return empty stats on error to avoid panic
                ErrorStats::default()
            }
        }
    }

    /// 获取错误摘要
    pub fn get_summary(&self) -> ErrorSummary {
        let Ok(stats) = safe_lock(&self.stats, "ErrorAggregator.stats") else {
            tracing::error!("Failed to acquire stats lock in get_summary");
            // Return empty summary on error
            return ErrorSummary {
                total_errors: 0,
                error_rate: 0.0,
                most_common_type: None,
                most_common_source: None,
                recent_error_count: 0,
                last_updated: 0,
            };
        };
        ErrorSummary {
            total_errors: stats.total_count,
            error_rate: stats.error_rate,
            most_common_type: stats.most_common_error_type().map(|(t, c)| (t.clone(), *c)),
            most_common_source: stats.most_common_error_source().map(|(s, c)| (s.clone(), *c)),
            recent_error_count: stats.recent_errors.len(),
            last_updated: stats.last_updated,
        }
    }

    /// 清除所有统计
    pub fn clear(&self) {
        let Ok(mut stats) = safe_lock(&self.stats, "ErrorAggregator.stats") else {
            tracing::error!("Failed to acquire stats lock in clear");
            return;
        };
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
            EngineError::Render(_) => "Render".to_string(),
            EngineError::Physics(_) => "Physics".to_string(),
            EngineError::Audio(_) => "Audio".to_string(),
            EngineError::Resource(_) => "Resource".to_string(),
            EngineError::Input(_) => "Input".to_string(),
            EngineError::System(_) => "System".to_string(),
            EngineError::General { .. } => "General".to_string(),
            EngineError::Multiple { .. } => "Multiple".to_string(),
            EngineError::Chain { .. } => "Chain".to_string(),
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
            lines.push(format!("最常见错误类型: {error_type} ({count}次)"));
        }

        if let Some((ref source, count)) = self.most_common_source {
            lines.push(format!("最常见错误来源: {source} ({count}次)"));
        }

        lines.push(format!("最近错误数: {}", self.recent_error_count));

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{ErrorSeverity, RenderError};

    #[test]
    fn test_error_aggregator() {
        let aggregator = ErrorAggregator::new();

        // 记录一些错误
        let render_err = EngineError::Render(RenderError::Adapter {
            message: "No adapter found".to_string(),
            severity: crate::error::ErrorSeverity::Critical,
        });
        aggregator.record_error(&render_err, "render_system");

        // Note: AssetError doesn't exist, using a different error for testing
        let resource_err =
            EngineError::Resource(crate::error::resource_error::ResourceError::NotFound {
                path: "test.png".to_string(),
                severity: ErrorSeverity::Error,
            });
        aggregator.record_error(&resource_err, "asset_manager");

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

        let report = aggregator.export_report().expect(
            "Failed to export error report: serialization should not fail for valid error data",
        );
        assert!(report.contains("TestError"));
        assert!(report.contains("test_module"));
    }
}
