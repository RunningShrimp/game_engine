//  内存调试工具模块
//
//  提供内存可视化、性能分析器集成和调试辅助工具。
//
//  ## 调试功能
//
//  ```text
//  ┌─────────────────────────────────────────────────────────┐
//  │                Memory Debug Tools                  │
//  ├─────────────────────────────────────────────────────────┤
//  │  1. 内存可视化                                        │
//  │     - 实时内存使用图表                                │
//  │     - 分配模式热图                                    │
//  │     - 碎片化程度可视化                                │
//  │                                                          │
//  │  2. 性能分析器集成                                    │
//  │     - 与tracing集成                                    │
//  │     - 自定义指标收集                                  │
//  │     - 实时性能仪表盘                                  │
//  │                                                          │
//  │  3. 调试辅助工具                                      │
//  │     - 内存分配跟踪                                    │
//  │     - 分配堆栈分析                                    │
//  │     - 内存泄漏检测工具                                │
//  └─────────────────────────────────────────────────────────┘
//  ```

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use super::memory_monitor::{MemoryMonitor, MemorySnapshot};

// ============================================================================
// 调试配置
// ============================================================================

/// 调试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// 启用内存可视化
    pub enable_memory_visualization: bool,
    /// 启用分配跟踪
    pub enable_allocation_tracking: bool,
    /// 启用堆栈分析
    pub enable_stack_analysis: bool,
    /// 最大跟踪的分配数
    pub max_tracked_allocations: usize,
    /// 堆栈深度限制
    pub max_stack_depth: usize,
    /// 启用实时图表
    pub enable_realtime_charts: bool,
    /// 图表更新间隔 (毫秒)
    pub chart_update_interval_ms: u64,
    /// 启用调试日志
    pub enable_debug_logging: bool,
    /// 日志级别过滤
    pub log_level_filter: String,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enable_memory_visualization: true,
            enable_allocation_tracking: true,
            enable_stack_analysis: true,
            max_tracked_allocations: 10000,
            max_stack_depth: 16,
            enable_realtime_charts: true,
            chart_update_interval_ms: 100,
            enable_debug_logging: true,
            log_level_filter: "debug".to_string(),
        }
    }
}

// ============================================================================
// 分配跟踪信息
// ============================================================================

/// 分配跟踪信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationTrace {
    /// 分配ID
    pub allocation_id: u64,
    /// 分配大小
    pub size: u64,
    /// 对齐要求
    pub alignment: u64,
    /// 分配时间戳
    pub allocation_time: std::time::SystemTime,
    /// 分配延迟 (微秒)
    pub allocation_latency_us: f32,
    /// 分配堆栈
    pub allocation_stack: Vec<String>,
    /// 分配类型
    pub allocation_type: String,
    /// 是否已释放
    pub is_freed: bool,
    /// 释放时间戳
    pub free_time: Option<std::time::SystemTime>,
    /// 生命周期 (微秒)
    pub lifetime_us: Option<u64>,
    /// 释放堆栈
    pub free_stack: Option<Vec<String>>,
}

impl AllocationTrace {
    /// 创建新的分配跟踪
    pub fn new(allocation_id: u64, size: u64, alignment: u64, allocation_type: String) -> Self {
        Self {
            allocation_id,
            size,
            alignment,
            allocation_time: std::time::SystemTime::now(),
            allocation_latency_us: 0.0,
            allocation_stack: Vec::new(),
            allocation_type,
            is_freed: false,
            free_time: None,
            lifetime_us: None,
            free_stack: None,
        }
    }

    /// 标记为已释放
    pub fn mark_freed(&mut self) {
        self.is_freed = true;
        self.free_time = Some(std::time::SystemTime::now());

        if let Ok(allocation_duration) = self.allocation_time.duration_since(std::time::UNIX_EPOCH)
        {
            if let Ok(free_duration) = self.free_time.unwrap().duration_since(std::time::UNIX_EPOCH)
            {
                let lifetime =
                    free_duration.as_micros().saturating_sub(allocation_duration.as_micros());
                self.lifetime_us = Some(lifetime as u64);
            }
        }
    }

    /// 获取生命周期描述
    pub fn lifetime_description(&self) -> String {
        if let Some(lifetime_us) = self.lifetime_us {
            if lifetime_us < 1000 {
                format!("{}μs", lifetime_us)
            } else if lifetime_us < 1_000_000 {
                format!("{}ms", lifetime_us / 1000)
            } else {
                format!("{}s", lifetime_us / 1_000_000)
            }
        } else {
            "Still active".to_string()
        }
    }
}

// ============================================================================
// 内存可视化数据
// ============================================================================

/// 内存可视化数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryVisualizationData {
    /// 数据时间戳
    pub timestamp: std::time::SystemTime,
    /// 内存块列表
    pub memory_blocks: Vec<MemoryBlockInfo>,
    /// 总容量
    pub total_capacity: u64,
    /// 已使用容量
    pub used_capacity: u64,
    /// 碎片化程度 (0.0-1.0)
    pub fragmentation_ratio: f32,
    /// 热点区域
    pub hotspots: Vec<MemoryHotspot>,
}

/// 内存块信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBlockInfo {
    /// 块ID
    pub block_id: u64,
    /// 起始地址
    pub start_address: u64,
    /// 结束地址
    pub end_address: u64,
    /// 块大小
    pub size: u64,
    /// 是否已使用
    pub is_used: bool,
    /// 块类型
    pub block_type: String,
    /// 分配时间
    pub allocation_time: Option<std::time::SystemTime>,
    /// 访问频率
    pub access_frequency: f32,
}

/// 内存热点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHotspot {
    /// 热点地址范围
    pub address_range: (u64, u64),
    /// 访问频率
    pub access_frequency: f32,
    /// 热点类型
    pub hotspot_type: HotspotType,
    /// 相关分配ID
    pub related_allocations: Vec<u64>,
}

/// 热点类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HotspotType {
    /// 频繁分配
    FrequentAllocation,
    /// 长时间持有
    LongHolding,
    /// 碎片化区域
    FragmentedArea,
    /// 内存泄漏
    MemoryLeak,
}

// ============================================================================
// 性能图表数据
// ============================================================================

/// 性能图表数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceChartData {
    /// 图表类型
    pub chart_type: ChartType,
    /// 数据点
    pub data_points: Vec<DataPoint>,
    /// 时间范围
    pub time_range: Duration,
    /// Y轴范围
    pub y_range: (f32, f32),
    /// 单位
    pub unit: String,
}

/// 图表类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChartType {
    /// 内存使用率
    MemoryUtilization,
    /// 分配延迟
    AllocationLatency,
    /// 分配频率
    AllocationFrequency,
    /// 碎片化程度
    FragmentationRatio,
    /// 预分配命中率
    PreallocationHitRate,
}

/// 数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// 时间戳
    pub timestamp: std::time::SystemTime,
    /// 值
    pub value: f32,
    /// 标签
    pub label: Option<String>,
}

// ============================================================================
// 内存调试器
// ============================================================================

/// 内存调试器
///
/// 提供内存可视化、分配跟踪和性能分析功能。
pub struct MemoryDebugger {
    /// 调试配置
    config: DebugConfig,
    /// 监控的内存监控器
    memory_monitor: Option<Arc<MemoryMonitor>>,
    /// 分配跟踪记录
    allocation_traces: Arc<Mutex<HashMap<u64, AllocationTrace>>>,
    /// 下一个分配ID
    next_allocation_id: Arc<Mutex<u64>>,
    /// 可视化数据历史
    visualization_history: Arc<Mutex<VecDeque<MemoryVisualizationData>>>,
    /// 性能图表数据
    chart_data: Arc<Mutex<HashMap<ChartType, PerformanceChartData>>>,
    /// 调试开始时间
    start_time: Instant,
    /// 上次图表更新时间
    last_chart_update: Arc<Mutex<Instant>>,
    /// 堆栈捕获函数
    stack_capture_fn: Option<Box<dyn Fn() -> Vec<String> + Send + Sync>>,
}

impl std::fmt::Debug for MemoryDebugger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryDebugger")
            .field("config", &self.config)
            .field("memory_monitor", &self.memory_monitor.is_some())
            .field("start_time", &self.start_time)
            .field("stack_capture_fn", &self.stack_capture_fn.is_some())
            .finish()
    }
}

impl MemoryDebugger {
    /// 创建新的内存调试器
    pub fn new(config: DebugConfig) -> Self {
        Self {
            config,
            memory_monitor: None,
            allocation_traces: Arc::new(Mutex::new(HashMap::new())),
            next_allocation_id: Arc::new(Mutex::new(1)),
            visualization_history: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            chart_data: Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
            last_chart_update: Arc::new(Mutex::new(Instant::now())),
            stack_capture_fn: None,
        }
    }

    /// 设置内存监控器
    pub fn set_memory_monitor(&mut self, monitor: Arc<MemoryMonitor>) {
        self.memory_monitor = Some(monitor);
    }

    /// 设置堆栈捕获函数
    pub fn set_stack_capture_fn<F>(&mut self, capture_fn: F)
    where
        F: Fn() -> Vec<String> + Send + Sync + 'static,
    {
        self.stack_capture_fn = Some(Box::new(capture_fn));
    }

    /// 开始调试
    pub fn start_debugging(&mut self) {
        tracing::info!(target: "memory_debugger", "Memory debugging started");
        self.start_time = Instant::now();
        *self.last_chart_update.lock() = Instant::now();

        // 初始化图表数据
        self.initialize_chart_data();
    }

    /// 停止调试
    pub fn stop_debugging(&mut self) {
        tracing::info!(target: "memory_debugger", "Memory debugging stopped");

        // 生成最终报告
        self.generate_final_report();
    }

    /// 跟踪分配
    pub fn track_allocation(&mut self, size: u64, alignment: u64, allocation_type: String) -> u64 {
        if !self.config.enable_allocation_tracking {
            return 0;
        }

        let allocation_id = *self.next_allocation_id.lock();
        *self.next_allocation_id.lock() = allocation_id + 1;

        // Clone allocation_type for logging before moving it
        let allocation_type_for_log = allocation_type.clone();

        let mut trace = AllocationTrace::new(allocation_id, size, alignment, allocation_type);
        trace.allocation_latency_us = 0.0; // 需要从外部获取

        // 捕获分配堆栈
        if self.config.enable_stack_analysis {
            if let Some(ref capture_fn) = self.stack_capture_fn {
                trace.allocation_stack = capture_fn();
            }
        }

        // 限制跟踪的分配数量
        {
            let mut traces = self.allocation_traces.lock();
            if traces.len() >= self.config.max_tracked_allocations {
                // 移除最旧的记录
                let oldest_id = traces.keys().next().copied();
                if let Some(id) = oldest_id {
                    traces.remove(&id);
                }
            }

            traces.insert(allocation_id, trace);
        }

        // 记录调试日志
        if self.config.enable_debug_logging {
            tracing::debug!(
                target: "memory_debugger",
                "Allocation tracked: ID={}, Size={} bytes, Type={}",
                allocation_id, size, allocation_type_for_log
            );
        }

        allocation_id
    }

    /// 跟踪释放
    pub fn track_deallocation(&mut self, allocation_id: u64) {
        if !self.config.enable_allocation_tracking {
            return;
        }

        let mut traces = self.allocation_traces.lock();
        if let Some(trace) = traces.get_mut(&allocation_id) {
            trace.mark_freed();

            // 捕获释放堆栈
            if self.config.enable_stack_analysis {
                if let Some(ref capture_fn) = self.stack_capture_fn {
                    trace.free_stack = Some(capture_fn());
                }
            }
        }

        // 记录调试日志
        if self.config.enable_debug_logging {
            tracing::debug!(
                target: "memory_debugger",
                "Deallocation tracked: ID={}",
                allocation_id
            );
        }
    }

    /// 更新可视化数据
    pub fn update_visualization(&mut self) {
        if !self.config.enable_memory_visualization {
            return;
        }

        let now = Instant::now();
        let last_update = *self.last_chart_update.lock();

        // 检查是否到了更新间隔
        if now.duration_since(last_update).as_millis()
            < self.config.chart_update_interval_ms as u128
        {
            return;
        }

        *self.last_chart_update.lock() = now;

        // 收集当前内存状态
        let visualization_data = self.collect_visualization_data();

        // 添加到历史记录
        {
            let mut history = self.visualization_history.lock();
            history.push_back(visualization_data.clone());

            // 保持历史记录长度
            while history.len() > 1000 {
                history.pop_front();
            }
        }

        // 更新图表数据
        self.update_chart_data(&visualization_data);
    }

    /// 收集可视化数据
    fn collect_visualization_data(&self) -> MemoryVisualizationData {
        let mut total_capacity = 0u64;
        let mut used_capacity = 0u64;
        let mut memory_blocks = Vec::new();
        let mut hotspots = Vec::new();

        // 从内存监控器获取数据
        if let Some(ref monitor) = self.memory_monitor {
            let snapshot = monitor.current_snapshot();
            total_capacity = snapshot.total_capacity;
            used_capacity = snapshot.used_bytes;

            // 生成内存块信息（简化实现）
            memory_blocks.push(MemoryBlockInfo {
                block_id: 1,
                start_address: 0,
                end_address: used_capacity,
                size: used_capacity,
                is_used: true,
                block_type: "Used".to_string(),
                allocation_time: Some(snapshot.timestamp),
                access_frequency: 1.0,
            });

            memory_blocks.push(MemoryBlockInfo {
                block_id: 2,
                start_address: used_capacity,
                end_address: total_capacity,
                size: total_capacity - used_capacity,
                is_used: false,
                block_type: "Free".to_string(),
                allocation_time: None,
                access_frequency: 0.0,
            });

            // 生成热点（简化实现）
            if snapshot.utilization > 0.8 {
                hotspots.push(MemoryHotspot {
                    address_range: (0, used_capacity),
                    access_frequency: snapshot.utilization,
                    hotspot_type: HotspotType::FrequentAllocation,
                    related_allocations: vec![1],
                });
            }
        }

        let fragmentation_ratio = if total_capacity > 0 {
            1.0 - (used_capacity as f32 / total_capacity as f32)
        } else {
            0.0
        };

        MemoryVisualizationData {
            timestamp: std::time::SystemTime::now(),
            memory_blocks,
            total_capacity,
            used_capacity,
            fragmentation_ratio,
            hotspots,
        }
    }

    /// 初始化图表数据
    fn initialize_chart_data(&mut self) {
        let mut charts = self.chart_data.lock();

        charts.insert(
            ChartType::MemoryUtilization,
            PerformanceChartData {
                chart_type: ChartType::MemoryUtilization,
                data_points: Vec::new(),
                time_range: Duration::from_secs(0),
                y_range: (0.0, 1.0),
                unit: "%".to_string(),
            },
        );

        charts.insert(
            ChartType::AllocationLatency,
            PerformanceChartData {
                chart_type: ChartType::AllocationLatency,
                data_points: Vec::new(),
                time_range: Duration::from_secs(0),
                y_range: (0.0, 1000.0),
                unit: "μs".to_string(),
            },
        );

        charts.insert(
            ChartType::AllocationFrequency,
            PerformanceChartData {
                chart_type: ChartType::AllocationFrequency,
                data_points: Vec::new(),
                time_range: Duration::from_secs(0),
                y_range: (0.0, 1000.0),
                unit: "alloc/s".to_string(),
            },
        );

        charts.insert(
            ChartType::FragmentationRatio,
            PerformanceChartData {
                chart_type: ChartType::FragmentationRatio,
                data_points: Vec::new(),
                time_range: Duration::from_secs(0),
                y_range: (0.0, 1.0),
                unit: "%".to_string(),
            },
        );

        charts.insert(
            ChartType::PreallocationHitRate,
            PerformanceChartData {
                chart_type: ChartType::PreallocationHitRate,
                data_points: Vec::new(),
                time_range: Duration::from_secs(0),
                y_range: (0.0, 1.0),
                unit: "%".to_string(),
            },
        );
    }

    /// 更新图表数据
    fn update_chart_data(&mut self, visualization_data: &MemoryVisualizationData) {
        let mut charts = self.chart_data.lock();

        // 更新内存使用率图表
        if let Some(chart) = charts.get_mut(&ChartType::MemoryUtilization) {
            let utilization = if visualization_data.total_capacity > 0 {
                visualization_data.used_capacity as f32 / visualization_data.total_capacity as f32
            } else {
                0.0
            };

            chart.data_points.push(DataPoint {
                timestamp: visualization_data.timestamp,
                value: utilization,
                label: None,
            });

            // 保持数据点数量限制
            if chart.data_points.len() > 1000 {
                chart.data_points.remove(0);
            }

            // 更新Y轴范围
            chart.y_range = (0.0, 1.0);
        }

        // 更新碎片化程度图表
        if let Some(chart) = charts.get_mut(&ChartType::FragmentationRatio) {
            chart.data_points.push(DataPoint {
                timestamp: visualization_data.timestamp,
                value: visualization_data.fragmentation_ratio,
                label: None,
            });

            if chart.data_points.len() > 1000 {
                chart.data_points.remove(0);
            }

            chart.y_range = (0.0, 1.0);
        }
    }

    /// 生成最终报告
    fn generate_final_report(&self) {
        let tracing_info = self.tracing_info();

        tracing::info!(
            target: "memory_debugger",
            "Final debugging report:\n{}",
            tracing_info
        );
    }

    /// 获取调试信息
    pub fn tracing_info(&self) -> String {
        let traces = self.allocation_traces.lock();
        let total_allocations = traces.len();
        let active_allocations = traces.values().filter(|t| !t.is_freed).count();
        let leaked_allocations = traces
            .values()
            .filter(|t| {
                !t.is_freed
                    && t.allocation_time.elapsed().unwrap_or(Duration::from_secs(0)).as_secs() > 60
            })
            .count();

        format!(
            "Total allocations: {}\nActive allocations: {}\nPotentially leaked: {}\nDebugging duration: {:?}",
            total_allocations,
            active_allocations,
            leaked_allocations,
            self.start_time.elapsed()
        )
    }

    /// 获取分配跟踪信息
    pub fn allocation_traces(&self) -> Vec<AllocationTrace> {
        self.allocation_traces.lock().values().cloned().collect()
    }

    /// 获取可视化数据
    pub fn visualization_data(&self) -> Vec<MemoryVisualizationData> {
        self.visualization_history.lock().iter().cloned().collect()
    }

    /// 获取图表数据
    pub fn chart_data(&self) -> HashMap<ChartType, PerformanceChartData> {
        self.chart_data.lock().clone()
    }

    /// 获取当前内存状态
    pub fn current_memory_state(&self) -> Option<MemorySnapshot> {
        self.memory_monitor.as_ref().map(|m| m.current_snapshot())
    }

    /// 导出调试数据
    pub fn export_debug_data(&self) -> DebugExportData {
        DebugExportData {
            config: self.config.clone(),
            allocation_traces: self.allocation_traces(),
            visualization_data: self.visualization_data(),
            chart_data: self.chart_data(),
            current_memory_state: self.current_memory_state(),
            debugging_duration: self.start_time.elapsed(),
        }
    }

    /// 清理调试数据
    pub fn clear_debug_data(&mut self) {
        self.allocation_traces.lock().clear();
        self.visualization_history.lock().clear();

        // 重置图表数据
        self.initialize_chart_data();

        tracing::debug!(target: "memory_debugger", "Debug data cleared");
    }
}

/// 导出的调试数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugExportData {
    /// 调试配置
    pub config: DebugConfig,
    /// 分配跟踪信息
    pub allocation_traces: Vec<AllocationTrace>,
    /// 可视化数据
    pub visualization_data: Vec<MemoryVisualizationData>,
    /// 图表数据
    pub chart_data: HashMap<ChartType, PerformanceChartData>,
    /// 当前内存状态
    pub current_memory_state: Option<MemorySnapshot>,
    /// 调试持续时间
    pub debugging_duration: Duration,
}

// ============================================================================
// 便捷函数
// ============================================================================

/// 创建默认配置的内存调试器
pub fn create_default_memory_debugger() -> MemoryDebugger {
    MemoryDebugger::new(DebugConfig::default())
}

/// 创建高性能配置的内存调试器
pub fn create_high_performance_memory_debugger() -> MemoryDebugger {
    let config = DebugConfig {
        enable_memory_visualization: true,
        enable_allocation_tracking: true,
        enable_stack_analysis: false, // 禁用以减少开销
        max_tracked_allocations: 5000,
        max_stack_depth: 8,
        enable_realtime_charts: true,
        chart_update_interval_ms: 50, // 更频繁的更新
        enable_debug_logging: false,  // 禁用以减少开销
        log_level_filter: "warn".to_string(),
    };

    MemoryDebugger::new(config)
}

/// 创建详细调试配置的内存调试器
pub fn create_verbose_memory_debugger() -> MemoryDebugger {
    let config = DebugConfig {
        enable_memory_visualization: true,
        enable_allocation_tracking: true,
        enable_stack_analysis: true,
        max_tracked_allocations: 20000,
        max_stack_depth: 32,
        enable_realtime_charts: true,
        chart_update_interval_ms: 200, // 较慢的更新以减少开销
        enable_debug_logging: true,
        log_level_filter: "trace".to_string(),
    };

    MemoryDebugger::new(config)
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_config_default() {
        let config = DebugConfig::default();
        assert!(config.enable_memory_visualization);
        assert!(config.enable_allocation_tracking);
        assert!(config.enable_stack_analysis);
        assert_eq!(config.max_tracked_allocations, 10000);
        assert_eq!(config.max_stack_depth, 16);
    }

    #[test]
    fn test_allocation_trace() {
        let mut trace = AllocationTrace::new(1, 1024, 256, "Test".to_string());
        assert_eq!(trace.allocation_id, 1);
        assert_eq!(trace.size, 1024);
        assert!(!trace.is_freed);

        trace.mark_freed();
        assert!(trace.is_freed);
        assert!(trace.free_time.is_some());
        assert!(trace.lifetime_us.is_some());
    }

    #[test]
    fn test_memory_block_info() {
        let block = MemoryBlockInfo {
            block_id: 1,
            start_address: 0x1000,
            end_address: 0x2000,
            size: 0x1000,
            is_used: true,
            block_type: "Test".to_string(),
            allocation_time: Some(std::time::SystemTime::now()),
            access_frequency: 1.0,
        };

        assert_eq!(block.block_id, 1);
        assert_eq!(block.size, 0x1000);
        assert!(block.is_used);
    }

    #[test]
    fn test_memory_hotspot() {
        let hotspot = MemoryHotspot {
            address_range: (0x1000, 0x2000),
            access_frequency: 0.8,
            hotspot_type: HotspotType::FrequentAllocation,
            related_allocations: vec![1, 2, 3],
        };

        assert_eq!(hotspot.address_range, (0x1000, 0x2000));
        assert_eq!(hotspot.access_frequency, 0.8);
        assert_eq!(hotspot.related_allocations.len(), 3);
    }

    #[test]
    fn test_performance_chart_data() {
        let chart = PerformanceChartData {
            chart_type: ChartType::MemoryUtilization,
            data_points: vec![DataPoint {
                timestamp: std::time::SystemTime::now(),
                value: 0.5,
                label: Some("50%".to_string()),
            }],
            time_range: Duration::from_secs(60),
            y_range: (0.0, 1.0),
            unit: "%".to_string(),
        };

        assert_eq!(chart.data_points.len(), 1);
        assert_eq!(chart.unit, "%");
        assert_eq!(chart.y_range, (0.0, 1.0));
    }
}
