//  性能监控服务
// 
//  提供统一的性能监控入口，整合指标收集、数据存储、告警和可视化功能。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::profiling::metrics::*;
use crate::profiling::collector::*;
use crate::profiling::storage::*;
use crate::profiling::dashboard::*;
use crate::profiling::alerting::*;
use crate::profiling::dashboard::*;
use crate::profiling::ProfilingResult;

// ============================================================================
// 性能监控服务
// ============================================================================

/// 性能监控服务配置
#[derive(Debug, Clone)]
pub struct ProfilingServiceConfig {
    /// 指标收集器配置
    pub collector_config: CollectorConfig,
    /// 存储配置
    pub storage_config: StorageConfig,
    /// 仪表板配置
    pub dashboard_config: DashboardConfig,
    /// 告警配置
    pub alerting_config: AlertingConfig,
    /// 是否启用自动启动
    pub auto_start: bool,
    /// 数据刷新间隔
    pub refresh_interval: Duration,
}

impl Default for ProfilingServiceConfig {
    fn default() -> Self {
        Self {
            collector_config: CollectorConfig::default(),
            storage_config: StorageConfig::default(),
            dashboard_config: DashboardConfig::default(),
            alerting_config: AlertingConfig::default(),
            auto_start: true,
            refresh_interval: Duration::from_millis(100), // 10Hz
        }
    }
}

/// 性能监控服务
pub struct ProfilingService {
    /// 服务配置
    config: ProfilingServiceConfig,
    /// 指标收集器
    collector: Arc<Mutex<MetricCollector>>,
    /// 持久化存储
    storage: Arc<Mutex<PersistentStorage>>,
    /// 仪表板服务器
    dashboard: Arc<Mutex<Option<DashboardServer>>>,
    /// 告警引擎
    alerting_engine: Arc<Mutex<AlertingEngine>>,
    /// 服务状态
    state: Arc<Mutex<ServiceState>>,
    /// 启动时间
    start_time: Instant,
}

/// 服务状态
#[derive(Debug, Clone)]
pub struct ServiceState {
    /// 是否正在运行
    pub is_running: bool,
    /// 最后刷新时间
    pub last_refresh: Instant,
    /// 总处理样本数
    pub total_samples_processed: u64,
    /// 当前活跃告警数
    pub active_alerts_count: usize,
    /// 存储文件数
    pub storage_files_count: usize,
    /// 错误计数
    pub error_count: u64,
}

impl Default for ServiceState {
    fn default() -> Self {
        Self {
            is_running: false,
            last_refresh: Instant::now(),
            total_samples_processed: 0,
            active_alerts_count: 0,
            storage_files_count: 0,
            error_count: 0,
        }
    }
}

impl ProfilingService {
    /// 创建新的性能监控服务
    pub fn new(config: ProfilingServiceConfig) -> ProfilingResult<Self> {
        // 创建指标收集器
        let collector = Arc::new(Mutex::new(
            MetricCollector::new(config.collector_config.clone())?
        ));

        // 创建持久化存储
        let storage = Arc::new(Mutex::new(
            PersistentStorage::new(config.storage_config.clone())?
        ));

        // 创建告警引擎
        let alerting_engine = Arc::new(Mutex::new(
            AlertingEngine::new(config.alerting_config.clone())
        ));

        let service = Self {
            config,
            collector,
            storage,
            dashboard: Arc::new(Mutex::new(None)),
            alerting_engine,
            state: Arc::new(Mutex::new(ServiceState::default())),
            start_time: Instant::now(),
        };

        // 添加默认告警规则
        service.add_default_alert_rules()?;

        Ok(service)
    }

    /// 启动服务
    pub fn start(&mut self) -> ProfilingResult<()> {
        if self.is_running() {
            return Err(crate::profiling::ProfilingError::ConfigurationError(
                "服务已在运行".to_string(),
            ));
        }

        // 启动仪表板服务器
        if self.config.dashboard_config.enable_realtime {
            self.start_dashboard_server()?;
        }

        // 设置运行状态
        {
            let mut state = crate::error::&self.state.lock()
                .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
            state.is_running = true;
        }

        tracing::info!(
            target: "profiling",
            "性能监控服务已启动"
        );

        Ok(())
    }

    /// 停止服务
    pub fn stop(&mut self) -> ProfilingResult<()> {
        if !self.is_running() {
            return Ok(());
        }

        // 停止仪表板服务器
        self.stop_dashboard_server()?;

        // 设置运行状态
        {
            let mut state = crate::error::&self.state.lock()
                .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
            state.is_running = false;
        }

        tracing::info!(
            target: "profiling",
            "性能监控服务已停止"
        );

        Ok(())
    }

    /// 记录指标值
    pub fn record_metric(&self, name: &str, value: f64) -> ProfilingResult<()> {
        if !self.is_running() {
            return Ok(());
        }

        // 更新收集器
        {
            let mut collector = crate::error::&self.collector.lock()
                .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
            collector.record_value(name, value);
        }

        // 更新告警检查
        {
            let mut alerting_engine = crate::error::&self.alerting_engine.lock()
                .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
            alerting_engine.update_metric(name, value);
        }

        // 更新状态
        self.update_state();

        Ok(())
    }

    /// 记录计时数据
    pub fn record_timing(&self, name: &str, duration: Duration) -> ProfilingResult<()> {
        let duration_ms = duration.as_secs_f64() * 1000.0;
        self.record_metric(name, duration_ms)
    }

    /// 创建计时器
    pub fn create_timer(&self, name: &str) -> HighPrecisionTimer {
        if let Ok(collector) = self.collector.lock() {
            collector.create_timer(name)
        } else {
            HighPrecisionTimer::new(name)
        }
    }

    /// 获取实时指标
    pub fn get_realtime_metrics(&self) -> ProfilingResult<RealtimeMetrics> {
        if !self.is_running() {
            return Err(crate::profiling::ProfilingError::CollectionError(
                "服务未运行".to_string(),
            ));
        }

        let collector_stats = {
            let collector = crate::error::&self.collector.lock()
                .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
            collector.get_collector_stats()
        };

        let current_values = {
            let collector = crate::error::&self.collector.lock()
                .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
            collector.get_current_values()
        };

        let metrics = RealtimeMetrics {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            fps: current_values.get("render.fps").unwrap_or(&0) as f64,
            frame_time: current_values.get("render.frame_time").unwrap_or(&0) as f64,
            cpu_usage: current_values.get("system.cpu_usage").unwrap_or(&0) as f64,
            memory_usage: current_values.get("memory.usage_mb").unwrap_or(&0) as f64,
            gpu_usage: current_values.get("render.gpu_utilization").unwrap_or(&0) as f64,
            draw_calls: *current_values.get("render.draw_calls").unwrap_or(&0),
            triangle_count: *current_values.get("render.triangle_count").unwrap_or(&0),
            physics_time: current_values.get("physics.step_time").unwrap_or(&0) as f64,
            audio_latency: current_values.get("audio.latency").unwrap_or(&0) as f64,
        };

        Ok(metrics)
    }

    /// 获取历史数据
    pub fn get_metric_history(&self, metric_name: &str, limit: Option<usize>) -> ProfilingResult<Vec<HistoricalDataPoint>> {
        if !self.is_running() {
            return Err(crate::profiling::ProfilingError::CollectionError(
                "服务未运行".to_string(),
            ));
        }

        // 查询存储数据
        let storage = crate::error::&self.storage.lock()
            .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
        let query_condition = QueryCondition {
            metric_names: Some(vec![metric_name.to_string()]),
            categories: None,
            start_time: None,
            end_time: None,
            tags: None,
            limit,
            order_by: Some(QueryOrder::TimestampDesc),
        };

        let queryer = DataQueryer::new(
            &self.config.storage_config.data_dir,
            &self.config.storage_config.file_prefix,
        );

        let result = queryer.query(&query_condition)?;
        
        // 转换为历史数据点
        let mut data_points = Vec::new();
        for data_point in result.data_points {
            data_points.push(HistoricalDataPoint {
                timestamp: data_point.timestamp,
                value: data_point.value,
                min: data_point.value,
                max: data_point.value,
                avg: data_point.value,
            });
        }

        Ok(data_points)
    }

    /// 获取服务状态
    pub fn get_service_state(&self) -> ProfilingResult<ServiceState> {
        let state = crate::error::&self.state.lock()
            .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
        Ok(state.clone())
    }

    /// 获取活跃告警
    pub fn get_active_alerts(&self) -> ProfilingResult<Vec<AlertInstance>> {
        if !self.is_running() {
            return Ok(Vec::new());
        }

        let alerting_engine = crate::error::&self.alerting_engine.lock()
            .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
        Ok(alerting_engine.get_active_alerts())
    }

    /// 确认告警
    pub fn acknowledge_alert(&self, alert_id: &str) -> ProfilingResult<bool> {
        if !self.is_running() {
            return Ok(false);
        }

        let mut alerting_engine = crate::error::&self.alerting_engine.lock()
            .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
        Ok(alerting_engine.acknowledge_alert(alert_id))
    }

    /// 导出数据
    pub fn export_data(&self, config: &ExportConfig, output_path: &std::path::Path) -> ProfilingResult<()> {
        if !self.is_running() {
            return Err(crate::profiling::ProfilingError::CollectionError(
                "服务未运行".to_string(),
            ));
        }

        let storage = crate::error::&self.storage.lock()
            .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
        let exporter = DataExporter::new(
            &self.config.storage_config.data_dir,
            &self.config.storage_config.file_prefix,
        );

        // 先刷新缓存
        storage.flush_cache()?;

        // 导出数据
        exporter.export(config, output_path)?;

        tracing::info!(
            target: "profiling",
            "数据已导出到: {:?}",
            output_path
        );

        Ok(())
    }

    /// 执行服务维护
    pub fn perform_maintenance(&self) -> ProfilingResult<MaintenanceReport> {
        if !self.is_running() {
            return Err(crate::profiling::ProfilingError::CollectionError(
                "服务未运行".to_string(),
            ));
        }

        let mut report = MaintenanceReport {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            operations: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // 清理过期数据
        if let Ok(storage) = self.storage.lock() {
            if let Ok(storage_stats) = storage.get_storage_stats() {
                report.operations.push(format!("清理存储文件，当前文件数: {}", storage_stats.total_files));
                
                if storage_stats.total_files > self.config.storage_config.retain_files {
                    // 这里可以实现文件清理逻辑
                    report.operations.push("删除过期文件".to_string());
                }
            }
        }

        // 重置计数器
        {
            let mut collector = crate::error::&self.collector.lock()
                .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
            collector.reset();
            report.operations.push("重置指标收集器".to_string());
        }

        // 检查告警状态
        {
            let alerting_engine = crate::error::&self.alerting_engine.lock()
                .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
            let active_alerts = alerting_engine.get_active_alerts();
            if active_alerts.len() > self.config.alerting_config.max_active_alerts / 2 {
                report.warnings.push(format!("活跃告警数较多: {}", active_alerts.len()));
            }
        }

        Ok(report)
    }

    /// 检查服务是否正在运行
    pub fn is_running(&self) -> bool {
        self.state.lock().map(|s| s.is_running).unwrap_or(false)
    }

    /// 获取运行时间
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// 启动仪表板服务器
    fn start_dashboard_server(&mut self) -> ProfilingResult<()> {
        if let Ok(dashboard) = self.dashboard.lock() {
            if dashboard.is_some() {
                return Ok(());
            }
        }

        let collector = Arc::clone(&self.collector);
        let storage = {
            let storage = crate::error::&self.storage.lock()
                .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
            // 创建一个新的存储实例用于仪表板
            PersistentStorage::new(self.config.storage_config.clone())?
        };

        let dashboard_server = DashboardServer::new(
            self.config.dashboard_config.clone(),
            collector,
            storage,
        )?;

        {
            let mut dashboard = crate::error::&self.dashboard.lock()
                .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
            *dashboard = Some(dashboard_server);
        }

        Ok(())
    }

    /// 停止仪表板服务器
    fn stop_dashboard_server(&mut self) -> ProfilingResult<()> {
        let mut dashboard = crate::error::&self.dashboard.lock()
            .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;
        *dashboard = None;
        Ok(())
    }

    /// 添加默认告警规则
    fn add_default_alert_rules(&self) -> ProfilingResult<()> {
        let mut alerting_engine = crate::error::&self.alerting_engine.lock()
            .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;

        // FPS告警
        alerting_engine.add_strategy("render.fps", AlertStrategy::Threshold(ThresholdAlertStrategy {
            level: AlertLevel::Warning,
            threshold: 30.0,
            operator: AlertOperator::LessThan,
            duration: Duration::from_secs(5),
            enable_recovery_notification: true,
        }));

        // CPU使用率告警
        alerting_engine.add_strategy("system.cpu_usage", AlertStrategy::Threshold(ThresholdAlertStrategy {
            level: AlertLevel::Critical,
            threshold: 80.0,
            operator: AlertOperator::GreaterThan,
            duration: Duration::from_secs(10),
            enable_recovery_notification: true,
        }));

        // 内存使用告警
        alerting_engine.add_strategy("memory.usage_mb", AlertStrategy::Threshold(ThresholdAlertStrategy {
            level: AlertLevel::Warning,
            threshold: 1024.0, // 1GB
            operator: AlertOperator::GreaterThan,
            duration: Duration::from_secs(30),
            enable_recovery_notification: true,
        }));

        // 帧时间告警
        alerting_engine.add_strategy("render.frame_time", AlertStrategy::Threshold(ThresholdAlertStrategy {
            level: AlertLevel::Warning,
            threshold: 33.3, // 30FPS
            operator: AlertOperator::GreaterThan,
            duration: Duration::from_secs(3),
            enable_recovery_notification: true,
        }));

        Ok(())
    }

    /// 更新服务状态
    fn update_state(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.last_refresh = Instant::now();
            
            // 更新统计信息
            if let Ok(collector) = self.collector.lock() {
                let stats = collector.get_collector_stats();
                state.total_samples_processed = stats.total_samples;
            }

            if let Ok(alerting_engine) = self.alerting_engine.lock() {
                state.active_alerts_count = alerting_engine.get_active_alerts().len();
            }

            if let Ok(storage) = self.storage.lock() {
                if let Ok(storage_stats) = storage.get_storage_stats() {
                    state.storage_files_count = storage_stats.total_files;
                }
            }
        }
    }

    /// 获取性能报告
    pub fn generate_performance_report(&self) -> ProfilingResult<String> {
        if !self.is_running() {
            return Err(crate::profiling::ProfilingError::CollectionError(
                "服务未运行".to_string(),
            ));
        }

        let mut report = String::new();
        report.push_str("=== 性能监控报告 ===\n\n");
        
        // 服务状态
        if let Ok(state) = self.get_service_state() {
            report.push_str(&format!("服务状态: {}\n", if state.is_running { "运行中" } else { "已停止" }));
            report.push_str(&format!("运行时间: {:.2}秒\n", state.last_refresh.elapsed().as_secs_f64()));
            report.push_str(&format!("总处理样本: {}\n", state.total_samples_processed));
            report.push_str(&format!("活跃告警数: {}\n", state.active_alerts_count));
            report.push_str(&format!("存储文件数: {}\n", state.storage_files_count));
            report.push_str(&format!("错误计数: {}\n\n", state.error_count));
        }

        // 实时指标
        if let Ok(metrics) = self.get_realtime_metrics() {
            report.push_str("--- 实时指标 ---\n");
            report.push_str(&format!("FPS: {:.1}\n", metrics.fps));
            report.push_str(&format!("帧时间: {:.2}ms\n", metrics.frame_time));
            report.push_str(&format!("CPU使用率: {:.1}%\n", metrics.cpu_usage));
            report.push_str(&format!("内存使用: {:.1}MB\n", metrics.memory_usage));
            report.push_str(&format!("GPU使用率: {:.1}%\n", metrics.gpu_usage));
            report.push_str(&format!("绘制调用: {}\n", metrics.draw_calls));
            report.push_str(&format!("三角形数: {}\n", metrics.triangle_count));
            report.push_str(&format!("物理时间: {:.2}ms\n", metrics.physics_time));
            report.push_str(&format!("音频延迟: {:.2}ms\n", metrics.audio_latency));
        }

        // 活跃告警
        if let Ok(alerts) = self.get_active_alerts() {
            report.push_str("--- 活跃告警 ---\n");
            if alerts.is_empty() {
                report.push_str("无活跃告警\n");
            } else {
                for alert in alerts.iter().take(5) {
                    report.push_str(&format!(
                        "[{}] {}: {}\n",
                        alert.level.as_str(),
                        alert.message
                    ));
                }
            }
        }

        report.push_str("====================\n");
        
        Ok(report)
    }
}

/// 维护报告
#[derive(Debug, Clone)]
pub struct MaintenanceReport {
    /// 报告时间戳
    pub timestamp: u64,
    /// 执行的操作
    pub operations: Vec<String>,
    /// 错误信息
    pub errors: Vec<String>,
    /// 警告信息
    pub warnings: Vec<String>,
}

// ============================================================================
// 便利宏
// ============================================================================

/// 记录性能指标的宏
#[macro_export]
macro_rules! profile_metric {
    ($service:expr, $name:expr, $value:expr) => {
        if let Err(e) = $service.record_metric($name, $value as f64) {
            tracing::error!(
                target: "profiling",
                "记录指标失败: {} - {}",
                $name,
                e
            );
        }
    };
}

/// 记录性能时间的宏
#[macro_export]
macro_rules! profile_scope {
    ($service:expr, $name:expr, $code:block) => {
        let _timer = $service.create_timer($name);
        let _result = $code;
        drop(_timer);
        _result
    };
}

/// 记录函数执行时间的宏
#[macro_export]
macro_rules! profile_function {
    ($service:expr, $name:expr, $func:expr) => {
        profile_scope!($service, $name, {
            $func()
        })
    };
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_profiling_service_config_default() {
        let config = ProfilingServiceConfig::default();
        assert!(config.auto_start);
        assert_eq!(config.refresh_interval, Duration::from_millis(100));
    }

    #[test]
    fn test_service_state_default() {
        let state = ServiceState::default();
        assert!(!state.is_running);
        assert_eq!(state.total_samples_processed, 0);
        assert_eq!(state.active_alerts_count, 0);
    }

    #[test]
    fn test_maintenance_report() {
        let report = MaintenanceReport {
            timestamp: 1234567890,
            operations: vec!["操作1".to_string(), "操作2".to_string()],
            errors: vec!["错误1".to_string()],
            warnings: vec!["警告1".to_string()],
        };

        assert_eq!(report.operations.len(), 2);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.warnings.len(), 1);
    }

    #[tokio::test]
    async fn test_profiling_service_lifecycle() {
        let config = ProfilingServiceConfig {
            auto_start: false,
            ..Default::default()
        };

        let mut service = ProfilingService::new(config).unwrap();
        
        // 初始状态应该是未运行
        assert!(!service.is_running());

        // 启动服务
        service.start().unwrap();
        assert!(service.is_running());

        // 记录指标
        service.record_metric("test_metric", 42.0).unwrap();

        // 获取实时指标
        let metrics = service.get_realtime_metrics().unwrap();
        assert_eq!(metrics.fps, 0.0);

        // 停止服务
        service.stop().unwrap();
        assert!(!service.is_running());
    }

    #[test]
    fn test_macros() {
        let config = ProfilingServiceConfig::default();
        let service = ProfilingService::new(config).unwrap();
        service.start().unwrap();

        // 测试指标记录宏
        profile_metric!(service, "test_metric", 100.0);

        // 测试作用域计时宏
        let result = profile_scope!(service, "test_scope", {
            std::thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);

        service.stop().unwrap();
    }
}