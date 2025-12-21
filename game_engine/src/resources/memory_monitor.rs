//  内存监控模块
// 
//  提供实时的内存使用监控、性能指标收集和调试工具。
// 
//  ## 监控功能
// 
//  ```text
//  ┌─────────────────────────────────────────────────────────┐
//  │                Memory Monitor                      │
//  ├─────────────────────────────────────────────────────────┤
//  │  1. 实时监控                                          │
//  │     - 内存使用率跟踪                                    │
//  │     - 分配延迟测量                                    │
//  │     - 带宽利用率统计                                  │
//  │                                                          │
//  │  2. 性能分析                                          │
//  │     - 分配模式分析                                    │
//  │     - 热点识别                                        │
//  │     - 瓶颈检测                                        │
//  │                                                          │
//  │  3. 调试工具                                          │
//  │     - 内存可视化                                      │
//  │     - 泄漏检测                                        │
//  │     - 性能分析器集成                                  │
//  └─────────────────────────────────────────────────────────┘
//  ```

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use super::enhanced_staging_buffer::{EnhancedPerformanceMetrics, EnhancedStagingBufferPool};
use super::memory_allocator::{MemoryPressure, MemoryPressureEvent};

// ============================================================================
// 监控配置
// ============================================================================

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// 启用实时监控
    pub enable_realtime_monitoring: bool,
    /// 监控间隔 (毫秒)
    pub monitoring_interval_ms: u64,
    /// 历史记录长度
    pub history_length: usize,
    /// 启用性能分析
    pub enable_performance_analysis: bool,
    /// 启用内存泄漏检测
    pub enable_leak_detection: bool,
    /// 泄漏检测阈值 (MB)
    pub leak_threshold_mb: f32,
    /// 启用自动报告
    pub enable_auto_reporting: bool,
    /// 报告间隔 (秒)
    pub reporting_interval_s: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            enable_realtime_monitoring: true,
            monitoring_interval_ms: 100, // 100ms
            history_length: 1000,
            enable_performance_analysis: true,
            enable_leak_detection: true,
            leak_threshold_mb: 100.0, // 100MB
            enable_auto_reporting: false,
            reporting_interval_s: 60, // 60秒
        }
    }
}

// ============================================================================
// 内存快照
// ============================================================================

/// 内存使用快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// 时间戳
    pub timestamp: std::time::SystemTime,
    /// 总容量 (字节)
    pub total_capacity: u64,
    /// 已使用字节数
    pub used_bytes: u64,
    /// 可用字节数
    pub free_bytes: u64,
    /// 使用率 (0.0-1.0)
    pub utilization: f32,
    /// 活跃分配数
    pub active_allocations: u64,
    /// 分配次数
    pub allocation_count: u64,
    /// 释放次数
    pub deallocation_count: u64,
    /// 峰值使用量
    pub peak_usage: u64,
    /// 内存压力级别
    pub memory_pressure: String,
    /// 预分配命中率
    pub preallocation_hit_rate: f32,
    /// 平均分配延迟 (微秒)
    pub average_allocation_latency_us: f32,
}

impl MemorySnapshot {
    /// 创建新的内存快照
    pub fn new() -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
            total_capacity: 0,
            used_bytes: 0,
            free_bytes: 0,
            utilization: 0.0,
            active_allocations: 0,
            allocation_count: 0,
            deallocation_count: 0,
            peak_usage: 0,
            memory_pressure: "Unknown".to_string(),
            preallocation_hit_rate: 0.0,
            average_allocation_latency_us: 0.0,
        }
    }

    /// 从性能指标创建快照
    pub fn from_metrics(metrics: &EnhancedPerformanceMetrics, total_capacity: u64) -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
            total_capacity,
            used_bytes: (total_capacity as f32 * metrics.current_utilization) as u64,
            free_bytes: total_capacity
                - (total_capacity as f32 * metrics.current_utilization) as u64,
            utilization: metrics.current_utilization,
            active_allocations: metrics.active_buffers as u64,
            allocation_count: metrics.total_allocations,
            deallocation_count: 0, // 需要从其他地方获取
            peak_usage: 0,         // 需要从历史记录计算
            memory_pressure: metrics.memory_pressure.clone(),
            preallocation_hit_rate: metrics.preallocation_hit_rate,
            average_allocation_latency_us: metrics.average_allocation_latency_us,
        }
    }

    /// 获取内存使用描述
    pub fn usage_description(&self) -> String {
        format!(
            "使用率: {:.1}%, 已用: {:.1}MB, 可用: {:.1}MB, 活跃分配: {}",
            self.utilization * 100.0,
            self.used_bytes as f32 / (1024.0 * 1024.0),
            self.free_bytes as f32 / (1024.0 * 1024.0),
            self.active_allocations
        )
    }
}

// ============================================================================
// 性能分析结果
// ============================================================================

/// 性能分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// 分析时间范围
    pub time_range: Duration,
    /// 平均使用率
    pub average_utilization: f32,
    /// 峰值使用率
    pub peak_utilization: f32,
    /// 最低使用率
    pub min_utilization: f32,
    /// 使用率标准差
    pub utilization_std_dev: f32,
    /// 总分配次数
    pub total_allocations: u64,
    /// 平均分配延迟
    pub average_allocation_latency: f32,
    /// 最大分配延迟
    pub max_allocation_latency: f32,
    /// 分配延迟标准差
    pub allocation_latency_std_dev: f32,
    /// 内存压力事件次数
    pub pressure_events: u64,
    /// 预分配命中率
    pub preallocation_hit_rate: f32,
    /// 内存效率评分 (0.0-1.0)
    pub efficiency_score: f32,
    /// 性能建议
    pub recommendations: Vec<String>,
}

impl PerformanceAnalysis {
    /// 创建新的性能分析
    pub fn new() -> Self {
        Self {
            time_range: Duration::from_secs(0),
            average_utilization: 0.0,
            peak_utilization: 0.0,
            min_utilization: 0.0,
            utilization_std_dev: 0.0,
            total_allocations: 0,
            average_allocation_latency: 0.0,
            max_allocation_latency: 0.0,
            allocation_latency_std_dev: 0.0,
            pressure_events: 0,
            preallocation_hit_rate: 0.0,
            efficiency_score: 0.0,
            recommendations: Vec::new(),
        }
    }

    /// 计算效率评分
    pub fn calculate_efficiency_score(&mut self) {
        // 基于多个指标计算综合效率评分
        let utilization_score = (1.0 - self.average_utilization).max(0.0); // 使用率越低越好
        let latency_score = (1.0 / (1.0 + self.average_allocation_latency / 1000.0)).max(0.0); // 延迟越低越好
        let hit_rate_score = self.preallocation_hit_rate; // 命中率越高越好

        // 加权平均
        self.efficiency_score =
            (utilization_score * 0.4 + latency_score * 0.4 + hit_rate_score * 0.2).min(1.0);
    }

    /// 生成性能建议
    pub fn generate_recommendations(&mut self) {
        self.recommendations.clear();

        if self.average_utilization > 0.9 {
            self.recommendations
                .push("内存使用率过高，考虑增加内存池大小".to_string());
        }

        if self.average_allocation_latency > 100.0 {
            self.recommendations
                .push("分配延迟较高，检查内存碎片化程度".to_string());
        }

        if self.preallocation_hit_rate < 0.5 {
            self.recommendations
                .push("预分配命中率较低，调整预分配策略".to_string());
        }

        if self.pressure_events > 10 {
            self.recommendations
                .push("内存压力事件频繁，优化内存使用模式".to_string());
        }

        if self.efficiency_score < 0.5 {
            self.recommendations
                .push("整体内存效率较低，建议进行性能优化".to_string());
        }
    }
}

// ============================================================================
// 内存泄漏检测
// ============================================================================

/// 内存泄漏检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakDetectionResult {
    /// 检测时间
    pub detection_time: std::time::SystemTime,
    /// 是否检测到泄漏
    pub has_leak: bool,
    /// 泄漏量 (字节)
    pub leaked_bytes: u64,
    /// 泄漏块数
    pub leaked_blocks: u64,
    /// 泄漏率 (0.0-1.0)
    pub leak_rate: f32,
    /// 可疑分配点
    pub suspicious_allocations: Vec<String>,
    /// 建议操作
    pub recommendations: Vec<String>,
}

impl LeakDetectionResult {
    /// 创建新的泄漏检测结果
    pub fn new() -> Self {
        Self {
            detection_time: std::time::SystemTime::now(),
            has_leak: false,
            leaked_bytes: 0,
            leaked_blocks: 0,
            leak_rate: 0.0,
            suspicious_allocations: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// 添加可疑分配
    pub fn add_suspicious_allocation(&mut self, allocation_info: String) {
        self.suspicious_allocations.push(allocation_info);
    }

    /// 生成修复建议
    pub fn generate_fix_recommendations(&mut self) {
        self.recommendations.clear();

        if self.has_leak {
            self.recommendations
                .push("检测到内存泄漏，建议检查资源释放逻辑".to_string());
            self.recommendations.push(format!(
                "泄漏量: {:.1}MB, 泄漏块数: {}",
                self.leaked_bytes as f32 / (1024.0 * 1024.0),
                self.leaked_blocks
            ));
        }

        if !self.suspicious_allocations.is_empty() {
            self.recommendations
                .push("发现可疑分配点，建议添加详细日志".to_string());
        }
    }
}

// ============================================================================
// 内存监控器
// ============================================================================

/// 内存监控器
///
/// 提供实时的内存使用监控、性能分析和泄漏检测。
#[derive(Debug)]
pub struct MemoryMonitor {
    /// 监控配置
    config: MonitorConfig,
    /// 监控的Staging Buffer池
    monitored_pools: Vec<Arc<Mutex<EnhancedStagingBufferPool>>>,
    /// 内存快照历史
    snapshot_history: Arc<Mutex<VecDeque<MemorySnapshot>>>,
    /// 压力事件历史
    pressure_history: Arc<Mutex<Vec<MemoryPressureEvent>>>,
    /// 性能分析结果
    latest_analysis: Arc<Mutex<Option<PerformanceAnalysis>>>,
    /// 泄漏检测结果
    leak_detection_result: Arc<Mutex<Option<LeakDetectionResult>>>,
    /// 监控开始时间
    start_time: Instant,
    /// 上次监控时间
    last_monitor_time: Arc<Mutex<Instant>>,
    /// 总分配计数
    total_allocations: Arc<Mutex<u64>>,
    /// 总释放计数
    total_deallocations: Arc<Mutex<u64>>,
    /// 峰值使用量
    peak_usage: Arc<Mutex<u64>>,
}

impl MemoryMonitor {
    /// 创建新的内存监控器
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            monitored_pools: Vec::new(),
            snapshot_history: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            pressure_history: Arc::new(Mutex::new(Vec::new())),
            latest_analysis: Arc::new(Mutex::new(None)),
            leak_detection_result: Arc::new(Mutex::new(None)),
            start_time: Instant::now(),
            last_monitor_time: Arc::new(Mutex::new(Instant::now())),
            total_allocations: Arc::new(Mutex::new(0)),
            total_deallocations: Arc::new(Mutex::new(0)),
            peak_usage: Arc::new(Mutex::new(0)),
        }
    }

    /// 添加要监控的池
    pub fn add_monitored_pool(&mut self, pool: Arc<Mutex<EnhancedStagingBufferPool>>) {
        self.monitored_pools.push(pool);
    }

    /// 获取总分配计数
    pub fn total_allocations(&self) -> u64 {
        *self.total_allocations.lock()
    }

    /// 开始监控
    pub fn start_monitoring(&mut self) {
        if !self.config.enable_realtime_monitoring {
            return;
        }

        tracing::info!(target: "memory_monitor", "Memory monitoring started");
        self.start_time = Instant::now();
        *self.last_monitor_time.lock() = Instant::now();
    }

    /// 停止监控
    pub fn stop_monitoring(&mut self) {
        tracing::info!(target: "memory_monitor", "Memory monitoring stopped");

        // 执行最终分析
        self.perform_final_analysis();
    }

    /// 更新监控状态
    pub fn update(&mut self) {
        if !self.config.enable_realtime_monitoring {
            return;
        }

        let now = Instant::now();
        let last_time = *self.last_monitor_time.lock();

        // 检查是否到了监控间隔
        if now.duration_since(last_time).as_millis() < self.config.monitoring_interval_ms as u128 {
            return;
        }

        // 更新监控时间
        *self.last_monitor_time.lock() = now;

        // 收集当前内存状态
        let snapshot = self.collect_memory_snapshot();

        // 添加到历史记录
        {
            let mut history = self.snapshot_history.lock();
            history.push_back(snapshot.clone());

            // 保持历史记录长度
            while history.len() > self.config.history_length {
                history.pop_front();
            }
        }

        // 更新峰值使用量
        {
            let mut peak = self.peak_usage.lock();
            *peak = (*peak).max(snapshot.used_bytes);
        }

        // 检测内存压力
        self.detect_memory_pressure(&snapshot);

        // 检测内存泄漏
        if self.config.enable_leak_detection {
            self.detect_memory_leaks(&snapshot);
        }

        // 执行性能分析
        if self.config.enable_performance_analysis {
            self.perform_performance_analysis();
        }
    }

    /// 收集内存快照
    fn collect_memory_snapshot(&self) -> MemorySnapshot {
        let mut total_capacity = 0u64;
        let mut total_used = 0u64;
        let mut total_active = 0u64;
        let mut total_allocations = 0u64;
        let mut avg_latency = 0.0f32;
        let mut hit_rate = 0.0f32;
        let mut pressure = "Unknown".to_string();

        // 聚合所有监控池的状态
        for pool in &self.monitored_pools {
            if let Some(pool_guard) = pool.try_lock() {
                let (capacity, used, _utilization) = pool_guard.memory_usage();
                total_capacity += capacity;
                total_used += used;

                let metrics = pool_guard.performance_metrics();
                total_active += metrics.active_buffers as u64;
                total_allocations += metrics.total_allocations;
                avg_latency = (avg_latency + metrics.average_allocation_latency_us) / 2.0;
                hit_rate = (hit_rate + metrics.preallocation_hit_rate) / 2.0;
                pressure = metrics.memory_pressure.clone();
            }
        }

        MemorySnapshot {
            timestamp: std::time::SystemTime::now(),
            total_capacity,
            used_bytes: total_used,
            free_bytes: total_capacity - total_used,
            utilization: if total_capacity > 0 {
                total_used as f32 / total_capacity as f32
            } else {
                0.0
            },
            active_allocations: total_active,
            allocation_count: total_allocations,
            deallocation_count: *self.total_deallocations.lock(),
            peak_usage: *self.peak_usage.lock(),
            memory_pressure: pressure,
            preallocation_hit_rate: hit_rate,
            average_allocation_latency_us: avg_latency,
        }
    }

    /// 检测内存压力
    fn detect_memory_pressure(&self, snapshot: &MemorySnapshot) {
        let pressure_level = if snapshot.utilization > 0.9 {
            MemoryPressure::Critical
        } else if snapshot.utilization > 0.75 {
            MemoryPressure::High
        } else if snapshot.utilization > 0.5 {
            MemoryPressure::Medium
        } else {
            MemoryPressure::Low
        };

        // 创建压力事件
        let event = MemoryPressureEvent::new(
            pressure_level,
            snapshot.utilization,
            self.generate_pressure_recommendation(&pressure_level),
        );

        // 添加到历史记录
        {
            let mut history = self.pressure_history.lock();
            history.push(event.clone());

            // 保持历史记录长度
            while history.len() > 100 {
                history.remove(0);
            }
        }

        // 记录压力事件
        if pressure_level != MemoryPressure::Low {
            tracing::warn!(
                target: "memory_monitor",
                "Memory pressure detected: {} (utilization: {:.1}%)",
                pressure_level.description(),
                snapshot.utilization * 100.0
            );
        }
    }

    /// 生成压力建议
    fn generate_pressure_recommendation(
        &self,
        pressure: &MemoryPressure,
    ) -> super::memory_allocator::PressureRecommendation {
        use super::memory_allocator::PressureRecommendation::*;

        match pressure {
            MemoryPressure::Low => None,
            MemoryPressure::Medium => CleanupUnused,
            MemoryPressure::High => ExpandPool,
            MemoryPressure::Critical => CleanupAndExpand,
        }
    }

    /// 检测内存泄漏
    fn detect_memory_leaks(&self, snapshot: &MemorySnapshot) {
        let threshold_bytes = (self.config.leak_threshold_mb * 1024.0 * 1024.0) as u64;

        // 简单的泄漏检测：如果使用量持续增长且没有相应释放
        let history = self.snapshot_history.lock();
        if history.len() < 10 {
            return; // 需要足够的历史数据
        }

        // 检查最近10个快照的趋势
        let recent_snapshots: Vec<_> = history.iter().rev().take(10).collect();
        let first_usage = recent_snapshots.first().unwrap().used_bytes;
        let last_usage = recent_snapshots.last().unwrap().used_bytes;

        let usage_increase = last_usage.saturating_sub(first_usage);
        let allocation_increase = snapshot
            .allocation_count
            .saturating_sub(recent_snapshots.first().unwrap().allocation_count);
        let deallocation_increase = snapshot
            .deallocation_count
            .saturating_sub(recent_snapshots.first().unwrap().deallocation_count);

        let mut result = LeakDetectionResult::new();

        // 如果使用量显著增长但释放很少，可能存在泄漏
        if usage_increase > threshold_bytes && allocation_increase > deallocation_increase * 2 {
            result.has_leak = true;
            result.leaked_bytes = usage_increase;
            result.leaked_blocks = allocation_increase - deallocation_increase;
            result.leak_rate = usage_increase as f32 / snapshot.total_capacity as f32;

            result.add_suspicious_allocation(format!(
                "Usage increased by {:.1}MB with {} allocations but only {} deallocations",
                usage_increase as f32 / (1024.0 * 1024.0),
                allocation_increase,
                deallocation_increase
            ));
        }

        result.generate_fix_recommendations();

        // 保存日志需要的数据
        let has_leak = result.has_leak;
        let leaked_bytes = result.leaked_bytes;
        let leaked_blocks = result.leaked_blocks;

        // 更新泄漏检测结果
        *self.leak_detection_result.lock() = Some(result);

        if has_leak {
            tracing::error!(
                target: "memory_monitor",
                "Memory leak detected: {:.1}MB leaked, {} blocks affected",
                leaked_bytes as f32 / (1024.0 * 1024.0),
                leaked_blocks
            );
        }
    }

    /// 执行性能分析
    fn perform_performance_analysis(&mut self) {
        let history = self.snapshot_history.lock();
        if history.len() < 10 {
            return; // 需要足够的历史数据
        }

        let mut analysis = PerformanceAnalysis::new();

        // 计算使用率统计
        let utilizations: Vec<f32> = history.iter().map(|s| s.utilization).collect();
        analysis.average_utilization = utilizations.iter().sum::<f32>() / utilizations.len() as f32;
        analysis.peak_utilization = utilizations.iter().fold(0.0f32, |a, &b| a.max(b));
        analysis.min_utilization = utilizations.iter().fold(1.0f32, |a, &b| a.min(b));

        // 计算标准差
        let variance = utilizations
            .iter()
            .map(|u| (u - analysis.average_utilization).powi(2))
            .sum::<f32>()
            / utilizations.len() as f32;
        analysis.utilization_std_dev = variance.sqrt();

        // 计算延迟统计
        let latencies: Vec<f32> = history
            .iter()
            .map(|s| s.average_allocation_latency_us)
            .collect();
        analysis.average_allocation_latency =
            latencies.iter().sum::<f32>() / latencies.len() as f32;
        analysis.max_allocation_latency = latencies.iter().fold(0.0f32, |a, &b| a.max(b));

        let latency_variance = latencies
            .iter()
            .map(|l| (l - analysis.average_allocation_latency).powi(2))
            .sum::<f32>()
            / latencies.len() as f32;
        analysis.allocation_latency_std_dev = latency_variance.sqrt();

        // 获取其他指标
        if let Some(latest) = history.back() {
            analysis.total_allocations = latest.allocation_count;
            analysis.preallocation_hit_rate = latest.preallocation_hit_rate;
        }

        // 计算时间范围
        analysis.time_range = self.start_time.elapsed();

        // 计算效率评分和生成建议
        analysis.calculate_efficiency_score();
        analysis.generate_recommendations();

        // 更新最新分析结果
        *self.latest_analysis.lock() = Some(analysis.clone());

        tracing::debug!(
            target: "memory_monitor",
            "Performance analysis completed - Efficiency score: {:.2}, Recommendations: {}",
            analysis.efficiency_score,
            analysis.recommendations.len()
        );
    }

    /// 执行最终分析
    fn perform_final_analysis(&mut self) {
        self.perform_performance_analysis();

        // 生成最终报告
        if let Some(analysis) = self.latest_analysis.lock().as_ref() {
            tracing::info!(
                target: "memory_monitor",
                "Final performance analysis - Efficiency: {:.2}, Peak utilization: {:.1}%, Avg latency: {:.2}μs",
                analysis.efficiency_score,
                analysis.peak_utilization * 100.0,
                analysis.average_allocation_latency
            );
        }
    }

    /// 获取当前内存快照
    pub fn current_snapshot(&self) -> MemorySnapshot {
        self.collect_memory_snapshot()
    }

    /// 获取内存快照历史
    pub fn snapshot_history(&self) -> Vec<MemorySnapshot> {
        self.snapshot_history.lock().iter().cloned().collect()
    }

    /// 获取压力事件历史
    pub fn pressure_history(&self) -> Vec<MemoryPressureEvent> {
        self.pressure_history.lock().iter().cloned().collect()
    }

    /// 获取最新性能分析
    pub fn latest_analysis(&self) -> Option<PerformanceAnalysis> {
        self.latest_analysis.lock().clone()
    }

    /// 获取泄漏检测结果
    pub fn leak_detection_result(&self) -> Option<LeakDetectionResult> {
        self.leak_detection_result.lock().clone()
    }

    /// 获取监控统计信息
    pub fn monitoring_stats(&self) -> MonitoringStats {
        MonitoringStats {
            monitoring_duration: self.start_time.elapsed(),
            total_snapshots: self.snapshot_history.lock().len(),
            total_pressure_events: self.pressure_history.lock().len(),
            peak_usage: *self.peak_usage.lock(),
            monitored_pools: self.monitored_pools.len(),
        }
    }

    /// 导出监控数据
    pub fn export_data(&self) -> MonitoringExportData {
        MonitoringExportData {
            config: self.config.clone(),
            snapshots: self.snapshot_history(),
            pressure_events: self.pressure_history(),
            analysis: self.latest_analysis(),
            leak_detection: self.leak_detection_result(),
            stats: self.monitoring_stats(),
        }
    }
}

/// 监控统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStats {
    /// 监控持续时间
    pub monitoring_duration: Duration,
    /// 总快照数
    pub total_snapshots: usize,
    /// 总压力事件数
    pub total_pressure_events: usize,
    /// 峰值使用量
    pub peak_usage: u64,
    /// 监控的池数量
    pub monitored_pools: usize,
}

/// 导出的监控数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringExportData {
    /// 监控配置
    pub config: MonitorConfig,
    /// 内存快照历史
    pub snapshots: Vec<MemorySnapshot>,
    /// 压力事件历史
    pub pressure_events: Vec<MemoryPressureEvent>,
    /// 性能分析结果
    pub analysis: Option<PerformanceAnalysis>,
    /// 泄漏检测结果
    pub leak_detection: Option<LeakDetectionResult>,
    /// 监控统计
    pub stats: MonitoringStats,
}

// ============================================================================
// 便捷函数
// ============================================================================

/// 创建默认配置的内存监控器
pub fn create_default_memory_monitor() -> MemoryMonitor {
    MemoryMonitor::new(MonitorConfig::default())
}

/// 创建高性能配置的内存监控器
pub fn create_high_performance_memory_monitor() -> MemoryMonitor {
    let config = MonitorConfig {
        enable_realtime_monitoring: true,
        monitoring_interval_ms: 50, // 更频繁的监控
        history_length: 2000,
        enable_performance_analysis: true,
        enable_leak_detection: true,
        leak_threshold_mb: 50.0, // 更严格的泄漏检测
        enable_auto_reporting: true,
        reporting_interval_s: 30,
    };

    MemoryMonitor::new(config)
}

/// 创建低开销配置的内存监控器
pub fn create_low_overhead_memory_monitor() -> MemoryMonitor {
    let config = MonitorConfig {
        enable_realtime_monitoring: true,
        monitoring_interval_ms: 500, // 较低频率的监控
        history_length: 500,
        enable_performance_analysis: false, // 禁用性能分析以减少开销
        enable_leak_detection: false,       // 禁用泄漏检测
        leak_threshold_mb: 200.0,
        enable_auto_reporting: false,
        reporting_interval_s: 120,
    };

    MemoryMonitor::new(config)
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_config_default() {
        let config = MonitorConfig::default();
        assert!(config.enable_realtime_monitoring);
        assert_eq!(config.monitoring_interval_ms, 100);
        assert_eq!(config.history_length, 1000);
        assert!(config.enable_performance_analysis);
        assert!(config.enable_leak_detection);
    }

    #[test]
    fn test_memory_snapshot() {
        let snapshot = MemorySnapshot::new();
        assert_eq!(snapshot.total_capacity, 0);
        assert_eq!(snapshot.used_bytes, 0);
        assert_eq!(snapshot.utilization, 0.0);
        assert_eq!(snapshot.active_allocations, 0);
    }

    #[test]
    fn test_performance_analysis() {
        let mut analysis = PerformanceAnalysis::new();
        analysis.average_utilization = 0.6;
        analysis.peak_utilization = 0.9;
        analysis.preallocation_hit_rate = 0.8;

        analysis.calculate_efficiency_score();
        assert!(analysis.efficiency_score > 0.0);
        assert!(analysis.efficiency_score <= 1.0);

        analysis.generate_recommendations();
        assert!(!analysis.recommendations.is_empty());
    }

    #[test]
    fn test_leak_detection_result() {
        let mut result = LeakDetectionResult::new();
        result.has_leak = true;
        result.leaked_bytes = 1024 * 1024; // 1MB
        result.leaked_blocks = 10;

        result.generate_fix_recommendations();
        assert!(result.recommendations.len() > 0);
        assert!(result.recommendations[0].contains("内存泄漏"));
    }

    #[test]
    fn test_monitoring_stats() {
        let stats = MonitoringStats {
            monitoring_duration: Duration::from_secs(60),
            total_snapshots: 100,
            total_pressure_events: 5,
            peak_usage: 1024 * 1024,
            monitored_pools: 2,
        };

        assert_eq!(stats.total_snapshots, 100);
        assert_eq!(stats.total_pressure_events, 5);
        assert_eq!(stats.monitored_pools, 2);
    }
}
