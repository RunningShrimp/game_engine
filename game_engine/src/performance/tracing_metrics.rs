//! 统一的Tracing和Metrics入口
//!
//! 提供游戏引擎的统一可观测性接口，包括：
//! - Tracing spans for performance monitoring
//! - Metrics collection and reporting
//! - Performance profiling integration
//! - 集成 game_engine_performance crate

use crate::performance::alerting::PerformanceAlertSystem;
use crate::performance::metrics_storage::MetricsStorage;
use crate::performance::monitoring::system_monitor::SystemPerformanceMonitor;
use crate::profiling::{Bottleneck, ContinuousProfiler, PerformanceAnalysis};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 统一的tracing和metrics管理器
#[derive(Debug)]
pub struct TracingMetricsManager {
    system_monitor: SystemPerformanceMonitor,
    start_time: Instant,
    continuous_profiler: Option<ContinuousProfiler>,
    /// Metrics存储系统
    metrics_storage: Arc<MetricsStorage>,
    /// 性能告警系统
    alert_system: PerformanceAlertSystem,
}

impl TracingMetricsManager {
    /// 创建新的tracing/metrics管理器
    pub fn new() -> Self {
        Self {
            system_monitor: SystemPerformanceMonitor::new(),
            start_time: Instant::now(),
            continuous_profiler: Some(ContinuousProfiler::new(300)),
            metrics_storage: Arc::new(MetricsStorage::new(1000)),
            alert_system: PerformanceAlertSystem::new(),
        }
    }

    /// 初始化tracing和metrics系统
    pub fn init() {
        // 初始化tracing subscriber（如果还没有初始化）
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();

        // Continuous profiler is created for later sampling; it does not require explicit start()
        tracing::info!(target: "performance", "Tracing and metrics system initialized");
    }

    /// 创建帧渲染span
    #[tracing::instrument(target = "render", skip_all, fields(entity_count, window_scale))]
    pub fn frame_span(entity_count: usize, window_scale: f64) -> tracing::Span {
        tracing::info_span!("frame", entity_count, window_scale)
    }

    /// 创建渲染提交span
    #[tracing::instrument(target = "render", skip_all, fields(batch_count, primitive_count))]
    pub fn render_submit_span(batch_count: usize, primitive_count: usize) -> tracing::Span {
        tracing::info_span!("render_submit", batch_count, primitive_count)
    }

    /// 创建资源加载span
    #[tracing::instrument(target = "asset", skip_all, fields(path, asset_type))]
    pub fn asset_load_span(path: &str, asset_type: &str) -> tracing::Span {
        tracing::info_span!("asset_load", path, asset_type)
    }

    /// 创建着色器编译span
    #[tracing::instrument(
        target = "render",
        skip_all,
        fields(shader_label, source_size, enable_cache)
    )]
    pub fn shader_compile_span(
        shader_label: &str,
        source_size: usize,
        enable_cache: bool,
    ) -> tracing::Span {
        tracing::info_span!("shader_compile", shader_label, source_size, enable_cache)
    }

    /// 创建网络tick span
    #[tracing::instrument(target = "network", skip_all, fields(tick))]
    pub fn network_tick_span(tick: u64) -> tracing::Span {
        tracing::info_span!("network_tick", tick)
    }

    /// 记录性能指标
    pub fn record_metric(&self, name: &str, value: f64) {
        tracing::info!(target: "metrics", %name, value);
        // 集成到metrics存储系统
        self.metrics_storage.record(name, value, None);
    }

    /// 获取系统性能快照
    pub fn get_performance_snapshot(
        &self,
    ) -> crate::performance::monitoring::system_monitor::PerformanceMetrics {
        self.system_monitor.get_metrics()
    }

    /// 获取continuous profiler数据
    pub fn get_continuous_profile_data(&self) -> Option<PerformanceAnalysis> {
        if let Some(ref profiler) = self.continuous_profiler {
            // 获取profiler的统计数据并转换为PerformanceAnalysis
            let samples = profiler.get_samples();
            if samples.is_empty() {
                return None;
            }

            let avg_fps = profiler.get_average_fps();
            let avg_frame_time = profiler.get_average_frame_time();
            let anomalies = profiler.detect_anomalies();

            let mut metrics = std::collections::HashMap::new();
            metrics.insert("avg_fps".to_string(), avg_fps as f64);
            metrics.insert(
                "avg_frame_time_ms".to_string(),
                (avg_frame_time * 1000.0) as f64,
            );
            metrics.insert("sample_count".to_string(), samples.len() as f64);

            let bottlenecks = anomalies
                .into_iter()
                .map(|anomaly| Bottleneck {
                    name: "Performance Anomaly".to_string(),
                    severity: 50,
                    description: format!("Performance anomaly detected: {:?}", anomaly),
                    suggestion: "Investigate recent changes or system load".to_string(),
                })
                .collect();

            Some(PerformanceAnalysis {
                name: "engine_runtime".to_string(),
                metrics,
                bottlenecks,
                recommendations: vec!["Monitor performance trends".to_string()],
            })
        } else {
            None
        }
    }

    /// 报告综合性能指标
    pub fn report_comprehensive_metrics(&self) {
        let snapshot = self.get_performance_snapshot();

        tracing::info!(target: "performance",
            cpu_usage = snapshot.cpu_usage_percent,
            memory_mb = snapshot.memory_usage_mb,
            fps = snapshot.fps,
            frame_time_ms = snapshot.frame_time_ms,
            "System performance snapshot"
        );

        if let Some(analysis) = self.get_continuous_profile_data() {
            for bottleneck in analysis.bottlenecks {
                tracing::warn!(target: "performance",
                    bottleneck_name = %bottleneck.name,
                    severity = ?bottleneck.severity,
                    description = bottleneck.description,
                    "Performance bottleneck detected"
                );
            }
        }
    }

    /// 更新性能指标并检查告警
    pub fn update_and_check_alerts(&mut self) {
        let snapshot = self.get_performance_snapshot();
        self.alert_system.update(&snapshot);
    }

    /// 获取告警系统
    pub fn alert_system(&self) -> &PerformanceAlertSystem {
        &self.alert_system
    }

    /// 获取可变告警系统
    pub fn alert_system_mut(&mut self) -> &mut PerformanceAlertSystem {
        &mut self.alert_system
    }

    /// 获取最近的告警事件
    pub fn get_recent_alerts(&self, limit: usize) -> Vec<crate::performance::alerting::AlertEvent> {
        self.alert_system.get_recent_alerts(limit)
    }

    /// 获取告警统计
    pub fn get_alert_statistics(&self) -> crate::performance::alerting::AlertStatistics {
        self.alert_system.get_statistics()
    }

    /// 记录帧时间
    pub fn record_frame_time(&self, frame_time: Duration) {
        self.record_metric("frame_time", frame_time.as_secs_f64() * 1000.0);
    }

    /// 记录渲染统计
    pub fn record_render_stats(&self, draw_calls: u32, instances: u32, triangles: u32) {
        self.record_metric("draw_calls", draw_calls as f64);
        self.record_metric("instances", instances as f64);
        self.record_metric("triangles", triangles as f64);
    }

    /// 获取运行时间
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// 获取metrics存储系统
    pub fn metrics_storage(&self) -> Arc<MetricsStorage> {
        Arc::clone(&self.metrics_storage)
    }

    /// 查询指定metric的数据
    ///
    /// # 参数
    ///
    /// * `name` - metric名称
    ///
    /// # 返回
    ///
    /// metric的所有数据点
    pub fn query_metrics(
        &self,
        name: &str,
    ) -> Vec<crate::performance::metrics_storage::MetricDataPoint> {
        self.metrics_storage.get_metrics(name)
    }

    /// 查询指定metric在时间窗口内的数据
    ///
    /// # 参数
    ///
    /// * `name` - metric名称
    /// * `duration` - 时间范围
    ///
    /// # 返回
    ///
    /// 时间范围内的数据点
    pub fn query_metrics_in_window(
        &self,
        name: &str,
        duration: Duration,
    ) -> Vec<crate::performance::metrics_storage::MetricDataPoint> {
        self.metrics_storage.get_metrics_in_window(name, duration)
    }

    /// 获取metric的聚合统计
    ///
    /// # 参数
    ///
    /// * `name` - metric名称
    /// * `duration` - 可选的时间范围
    ///
    /// # 返回
    ///
    /// 聚合统计
    pub fn query_metric_aggregate(
        &self,
        name: &str,
        duration: Option<Duration>,
    ) -> Option<crate::performance::metrics_storage::MetricAggregate> {
        self.metrics_storage.aggregate(name, duration)
    }
}

impl Default for TracingMetricsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局tracing/metrics管理器实例
static TRACING_METRICS_MANAGER: std::sync::OnceLock<std::sync::Mutex<TracingMetricsManager>> =
    std::sync::OnceLock::new();

/// 获取全局tracing/metrics管理器
pub fn global_tracing_metrics() -> &'static std::sync::Mutex<TracingMetricsManager> {
    TRACING_METRICS_MANAGER.get_or_init(|| std::sync::Mutex::new(TracingMetricsManager::new()))
}

/// 初始化全局tracing/metrics系统
pub fn init_global_tracing_metrics() {
    TracingMetricsManager::init();
}

/// 便捷宏用于创建tracing spans
#[macro_export]
macro_rules! trace_span {
    ($target:expr, $name:expr $(, $field:ident = $value:expr)* $(,)?) => {
        tracing::info_span!(target: $target, $name, $($field = $value),*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_metrics_manager() {
        let manager = TracingMetricsManager::new();
        let metrics = manager.get_performance_snapshot();

        // 验证基本功能
        assert!(manager.uptime() >= Duration::from_secs(0));
        // FPS可能为0（如果没有足够的采样）
        assert!(metrics.fps >= 0.0);
    }

    #[test]
    fn test_global_manager() {
        let manager = global_tracing_metrics();
        if let Ok(m) = manager.lock() {
            let uptime = m.uptime();
            assert!(uptime >= Duration::from_secs(0));
        }
    }

    #[test]
    fn test_alert_system_integration() {
        let mut manager = TracingMetricsManager::new();

        // 更新性能指标并检查告警
        manager.update_and_check_alerts();

        // 获取告警统计
        let stats = manager.get_alert_statistics();
        assert_eq!(stats.total_alerts, 0);

        // 获取最近的告警
        let alerts = manager.get_recent_alerts(10);
        assert!(alerts.is_empty());
    }
}
