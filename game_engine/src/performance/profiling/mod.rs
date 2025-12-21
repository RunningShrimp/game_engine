//  性能分析模块
// 
//  注意：此模块的功能已统一到 game_engine::profiling 模块。
// 为了向后兼容，这里重新导出主模块的功能。

// 重新导出主模块的功能（统一API）
pub use crate::profiling::*;

// 向后兼容：保留原有的导出
mod profiler;
pub use profiler::Profiler;

// 持续性能分析器模块（对外导出）
pub mod continuous_profiler;
pub use continuous_profiler::ContinuousProfiler;

use std::collections::HashMap;
use std::time::Instant;

/// 高级性能指标
#[derive(Debug, Clone, Default)]
pub struct AdvancedPerfMetrics {
    /// 帧时间（毫秒）
    pub frame_time: f32,
    /// FPS
    pub fps: f32,
    /// 更新时间（毫秒）
    pub update_time: f32,
    /// 渲染时间（毫秒）
    pub render_time: f32,
    /// 内存使用（MB）
    pub memory_usage: f32,
    /// 绘制调用次数
    pub draw_calls: u32,
    /// 三角形数量
    pub triangle_count: u32,
}

/// 高级分析器
pub struct AdvancedProfiler {
    /// 最新指标
    latest_metrics: Option<AdvancedPerfMetrics>,
    /// 历史指标
    history: Vec<AdvancedPerfMetrics>,
    /// 最大历史长度
    max_history_length: usize,
    /// 帧开始时间
    frame_start_time: Option<Instant>,
}

impl AdvancedProfiler {
    /// 创建新的高级分析器
    pub fn new(max_history_length: usize) -> Self {
        Self {
            latest_metrics: None,
            history: Vec::with_capacity(max_history_length),
            max_history_length,
            frame_start_time: None,
        }
    }

    /// 开始帧
    pub fn begin_frame(&mut self) {
        self.frame_start_time = Some(Instant::now());
    }

    /// 结束帧
    pub fn end_frame(&mut self, metrics: AdvancedPerfMetrics) {
        self.latest_metrics = Some(metrics.clone());

        self.history.push(metrics);
        if self.history.len() > self.max_history_length {
            self.history.remove(0);
        }
    }

    /// 获取最新指标
    pub fn get_latest_metrics(&self) -> Option<&AdvancedPerfMetrics> {
        self.latest_metrics.as_ref()
    }
}

/// 内存分析器
pub struct MemoryProfiler {
    /// 当前内存使用量
    current_memory: usize,
    /// 峰值内存使用量
    peak_memory: usize,
    /// 分配统计
    allocation_stats: HashMap<String, (usize, usize)>, // (count, size)
}

impl MemoryProfiler {
    /// 创建新的内存分析器
    pub fn new() -> Self {
        Self {
            current_memory: 0,
            peak_memory: 0,
            allocation_stats: HashMap::new(),
        }
    }

    /// 获取当前内存使用量
    pub fn get_current_memory_usage(&self) -> usize {
        self.current_memory
    }

    /// 获取峰值内存使用量
    pub fn get_peak_memory_usage(&self) -> usize {
        self.peak_memory
    }

    /// 获取分配统计
    pub fn get_allocation_stats(&self) -> &HashMap<String, (usize, usize)> {
        &self.allocation_stats
    }

    /// 清除统计数据
    pub fn clear(&mut self) {
        self.current_memory = 0;
        self.peak_memory = 0;
        self.allocation_stats.clear();
    }

    /// 生成报告
    pub fn generate_report(&self) -> String {
        format!(
            "=== Memory Profiler Report ===\nCurrent Memory: {} bytes\nPeak Memory: {} bytes\nAllocations: {}\n",
            self.current_memory,
            self.peak_memory,
            self.allocation_stats.len()
        )
    }
}

/// GPU分析器
pub struct GpuProfiler {
    /// 查询结果
    queries: HashMap<String, f32>, // name -> time_ms
}

impl GpuProfiler {
    /// 创建新的GPU分析器
    pub fn new() -> Self {
        Self {
            queries: HashMap::new(),
        }
    }

    /// 获取所有查询
    pub fn get_all_queries(&self) -> &HashMap<String, f32> {
        &self.queries
    }

    /// 清除查询
    pub fn clear(&mut self) {
        self.queries.clear();
    }

    /// 生成报告
    pub fn generate_report(&self) -> String {
        format!(
            "=== GPU Profiler Report ===\nQueries: {}\n",
            self.queries.len()
        )
    }
}
