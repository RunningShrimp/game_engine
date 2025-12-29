//! 统一的Tracing和Metrics入口
//!
//! 提供游戏引擎的统一可观测性接口，包括：
//! - Tracing spans for performance monitoring
//! - Metrics collection and reporting
//! - Performance profiling integration
//! - 集成 game_engine_performance crate

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::performance::monitoring::system_monitor::SystemPerformanceMonitor;
use crate::profiling::{
    Bottleneck, ContinuousProfiler, PerformanceAnalysis
};

/// Metric值类型
#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
}

/// Metric数据点
#[derive(Debug, Clone)]
pub struct MetricDataPoint {
    pub timestamp: Instant,
    pub value: MetricValue,
}

/// 统一的tracing和metrics管理器
#[derive(Debug)]
pub struct TracingMetricsManager {
    system_monitor: SystemPerformanceMonitor,
    start_time: Instant,
    continuous_profiler: Option<ContinuousProfiler>,
    /// Metrics存储：metric_name -> list of data points
    metrics_storage: Arc<Mutex<HashMap<String, Vec<MetricDataPoint>>>>,
}

impl TracingMetricsManager {
    /// 创建新的tracing/metrics管理器
    pub fn new() -> Self {
        Self {
            system_monitor: SystemPerformanceMonitor::new(),
            start_time: Instant::now(),
            continuous_profiler: Some(ContinuousProfiler::new(300)),
            metrics_storage: Arc::new(Mutex::new(HashMap::new())),
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
    #[tracing::instrument(target = "render", skip_all, fields(shader_label, source_size, enable_cache))]
    pub fn shader_compile_span(shader_label: &str, source_size: usize, enable_cache: bool) -> tracing::Span {
        tracing::info_span!("shader_compile", shader_label, source_size, enable_cache)
    }

    /// 创建网络tick span
    #[tracing::instrument(target = "network", skip_all, fields(tick))]
    pub fn network_tick_span(tick: u64) -> tracing::Span {
        tracing::info_span!("network_tick", tick)
    }

    /// 记录性能指标（计数器类型）
    pub fn record_counter(&self, name: &str, value: u64) {
        let timestamp = Instant::now();
        tracing::info!(target: "metrics", %name, value);

        let mut storage = self.metrics_storage.lock().unwrap();
        storage.entry(name.to_string())
            .or_default()
            .push(MetricDataPoint {
                timestamp,
                value: MetricValue::Counter(value),
            });

        // 限制存储大小（保留最近1000个数据点）
        if let Some(metrics) = storage.get_mut(name)
            && metrics.len() > 1000 {
                metrics.drain(0..metrics.len() - 1000);
            }
    }

    /// 记录性能指标（仪表类型）
    pub fn record_gauge(&self, name: &str, value: f64) {
        let timestamp = Instant::now();
        tracing::info!(target: "metrics", %name, value);

        let mut storage = self.metrics_storage.lock().unwrap();
        storage.entry(name.to_string())
            .or_default()
            .push(MetricDataPoint {
                timestamp,
                value: MetricValue::Gauge(value),
            });

        // 限制存储大小
        if let Some(metrics) = storage.get_mut(name)
            && metrics.len() > 1000 {
                metrics.drain(0..metrics.len() - 1000);
            }
    }

    /// 记录性能指标（通用方法，保留向后兼容）
    pub fn record_metric(&self, name: &str, value: f64) {
        self.record_gauge(name, value);
    }

    /// 获取指定metric的所有数据点
    pub fn get_metric_data(&self, name: &str) -> Option<Vec<MetricDataPoint>> {
        let storage = self.metrics_storage.lock().unwrap();
        storage.get(name).cloned()
    }

    /// 获取指定metric的最新值
    pub fn get_latest_metric(&self, name: &str) -> Option<MetricDataPoint> {
        let storage = self.metrics_storage.lock().unwrap();
        storage.get(name).and_then(|v| v.last().cloned())
    }

    /// 清除指定metric的所有数据
    pub fn clear_metric(&self, name: &str) {
        let mut storage = self.metrics_storage.lock().unwrap();
        storage.remove(name);
    }

    /// 获取所有metric名称
    pub fn get_all_metric_names(&self) -> Vec<String> {
        let storage = self.metrics_storage.lock().unwrap();
        storage.keys().cloned().collect()
    }

    /// 获取系统性能快照
    pub fn get_performance_snapshot(&self) -> crate::performance::monitoring::system_monitor::PerformanceMetrics {
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
            metrics.insert("avg_frame_time_ms".to_string(), (avg_frame_time * 1000.0) as f64);
            metrics.insert("sample_count".to_string(), samples.len() as f64);
            
            let bottlenecks = anomalies.into_iter().map(|anomaly| {
                Bottleneck {
                    name: "Performance Anomaly".to_string(),
                    severity: 50,
                    description: format!("Performance anomaly detected: {:?}", anomaly),
                    suggestion: "Investigate recent changes or system load".to_string(),
                }
            }).collect();
            
            // 创建一个PerformanceAnalysis实例并返回
            let analysis = PerformanceAnalysis {
                name: "engine_runtime".to_string(),
                metrics,
                bottlenecks,
                recommendations: vec!["Monitor performance trends".to_string()],
            };
            Some(analysis)
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
}

impl Default for TracingMetricsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局tracing/metrics管理器实例
static TRACING_METRICS_MANAGER: std::sync::OnceLock<TracingMetricsManager> = std::sync::OnceLock::new();

/// 获取全局tracing/metrics管理器
pub fn global_tracing_metrics() -> &'static TracingMetricsManager {
    TRACING_METRICS_MANAGER.get_or_init(TracingMetricsManager::new)
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
        let uptime = manager.uptime();
        assert!(uptime >= Duration::from_secs(0));
    }
}
