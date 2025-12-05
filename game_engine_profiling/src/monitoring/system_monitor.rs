//! 系统性能监控器
//!
//! 实时性能监控和数据收集
//! - 帧率监控
//! - 内存跟踪
//! - CPU 使用率
//! - 性能统计
<<<<<<< HEAD

use std::collections::VecDeque;
use std::time::{Duration, Instant};

=======
//! - 多维度性能指标
//! - 性能问题检测
//! - 优化建议生成

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// 性能指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    // CPU 指标
    FrameTime,
    CpuTime,
    UpdateTime,
    RenderTime,

    // GPU 指标
    GpuTime,
    DrawCalls,
    TriangleCount,
    VertexCount,

    // 内存指标
    RamUsage,
    VramUsage,
    AllocCount,

    // 物理指标
    PhysicsTime,
    CollisionChecks,

    // AI 指标
    AiTime,
    PathfindingTime,
}

/// 单个性能指标
#[derive(Debug, Clone)]
pub struct Metric {
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: String,
    pub timestamp: Instant,
}

impl Metric {
    pub fn new(metric_type: MetricType, value: f64, unit: String) -> Self {
        Self {
            metric_type,
            value,
            unit,
            timestamp: Instant::now(),
        }
    }
}

/// 性能统计（一段时间内的聚合统计）
#[derive(Debug, Clone)]
pub struct MetricStats {
    pub metric_type: MetricType,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub median: f64,
    pub stddev: f64,
    pub samples: usize,
}

impl MetricStats {
    pub fn compute(metric_type: MetricType, values: &[f64]) -> Self {
        if values.is_empty() {
            return Self {
                metric_type,
                min: 0.0,
                max: 0.0,
                avg: 0.0,
                median: 0.0,
                stddev: 0.0,
                samples: 0,
            };
        }

        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let sum: f64 = values.iter().sum();
        let avg = sum / values.len() as f64;

        // 计算中位数
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = sorted[sorted.len() / 2];

        // 计算标准差
        let variance: f64 =
            values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / values.len() as f64;
        let stddev = variance.sqrt();

        Self {
            metric_type,
            min,
            max,
            avg,
            median,
            stddev,
            samples: values.len(),
        }
    }
}

/// 性能问题严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 性能问题
#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub severity: IssueSeverity,
    pub message: String,
}

/// 性能优化建议
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub area: String,
    pub issue: String,
    pub recommendation: String,
    pub expected_improvement: String,
}

impl OptimizationRecommendation {
    pub fn generate_recommendations(report: &PerformanceReport) -> Vec<Self> {
        let mut recommendations = Vec::new();

        // 检测 Draw Call 过多
        if let Some(stats) = report.stats.get(&MetricType::DrawCalls) {
            if stats.avg > 500.0 {
                recommendations.push(Self {
                    area: "Rendering".to_string(),
                    issue: "Too many draw calls".to_string(),
                    recommendation: "Enable draw call batching and implement LOD system"
                        .to_string(),
                    expected_improvement: "30-50% reduction in draw calls".to_string(),
                });
            }
        }

        // 检测 CPU 时间过长
        if let Some(update_stats) = report.stats.get(&MetricType::UpdateTime) {
            if update_stats.avg > 10.0 {
                recommendations.push(Self {
                    area: "CPU".to_string(),
                    issue: "High update time".to_string(),
                    recommendation: "Profile and optimize hot paths, consider using SIMD"
                        .to_string(),
                    expected_improvement: "20-40% improvement in update performance".to_string(),
                });
            }
        }

        // 检测内存使用
        if let Some(ram_stats) = report.stats.get(&MetricType::RamUsage) {
            if ram_stats.avg > 1024.0 {
                recommendations.push(Self {
                    area: "Memory".to_string(),
                    issue: "High RAM usage".to_string(),
                    recommendation: "Use Arena allocator and object pooling".to_string(),
                    expected_improvement: "20-30% reduction in memory usage".to_string(),
                });
            }
        }

        recommendations
    }
}

>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
/// 性能指标
#[derive(Debug, Clone, Copy)]
pub struct PerformanceMetrics {
    /// 帧率 (FPS)
    pub fps: f32,
    /// 帧时间 (毫秒)
    pub frame_time_ms: f32,
    /// 内存使用 (MB)
    pub memory_usage_mb: f32,
    /// CPU 使用率 (%)
    pub cpu_usage_percent: f32,
    /// GPU 使用率 (%)
    pub gpu_usage_percent: f32,
}

/// 帧时间采样器
pub struct FrameTimeSampler {
    /// 采样缓冲区
    samples: VecDeque<Duration>,
    /// 最大样本数
    max_samples: usize,
    /// 上一帧时间
    last_frame: Instant,
}

impl FrameTimeSampler {
    /// 创建新的帧时间采样器
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
            last_frame: Instant::now(),
        }
    }

    /// 记录一帧
    pub fn sample_frame(&mut self) -> Duration {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame);
        self.last_frame = now;

        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(frame_time);

        frame_time
    }

    /// 获取平均帧时间
    pub fn average_frame_time(&self) -> Duration {
        if self.samples.is_empty() {
            return Duration::ZERO;
        }

        let total: Duration = self.samples.iter().sum();
        Duration::from_nanos(total.as_nanos() as u64 / self.samples.len() as u64)
    }

    /// 获取 FPS
    pub fn fps(&self) -> f32 {
        let avg = self.average_frame_time();
        if avg.as_secs_f32() == 0.0 {
            0.0
        } else {
            1.0 / avg.as_secs_f32()
        }
    }

    /// 获取最小帧时间
    pub fn min_frame_time(&self) -> Option<Duration> {
        self.samples.iter().copied().min()
    }

    /// 获取最大帧时间
    pub fn max_frame_time(&self) -> Option<Duration> {
        self.samples.iter().copied().max()
    }

    /// 获取第 N 百分位
    pub fn percentile(&self, p: f32) -> Option<Duration> {
        if self.samples.is_empty() {
            return None;
        }

        let mut sorted: Vec<_> = self.samples.iter().copied().collect();
        sorted.sort();

        let index = ((p / 100.0) * sorted.len() as f32) as usize;
        sorted.get(index.min(sorted.len() - 1)).copied()
    }

    /// 清空采样
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// 内存监控器
pub struct MemoryMonitor {
    /// 采样历史
    history: VecDeque<u64>,
    /// 最大历史大小
    max_history: usize,
}

impl MemoryMonitor {
    /// 创建新的内存监控器
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// 记录内存使用
    pub fn sample_memory(&mut self, bytes: u64) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(bytes);
    }

    /// 获取当前内存使用 (MB)
    pub fn current_memory_mb(&self) -> f32 {
        self.history
            .back()
            .map(|&b| b as f32 / (1024.0 * 1024.0))
            .unwrap_or(0.0)
    }

    /// 获取平均内存使用 (MB)
    pub fn average_memory_mb(&self) -> f32 {
        if self.history.is_empty() {
            return 0.0;
        }

        let sum: u64 = self.history.iter().sum();
        let avg = sum as f32 / self.history.len() as f32;
        avg / (1024.0 * 1024.0)
    }

    /// 获取峰值内存 (MB)
    pub fn peak_memory_mb(&self) -> f32 {
        self.history
            .iter()
            .max()
            .map(|&b| b as f32 / (1024.0 * 1024.0))
            .unwrap_or(0.0)
    }
}

/// CPU 监控器
pub struct CPUMonitor {
    /// 采样历史 (%)
    history: VecDeque<f32>,
    /// 最大历史大小
    max_history: usize,
}

impl CPUMonitor {
    /// 创建新的 CPU 监控器
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// 记录 CPU 使用率
    pub fn sample_cpu(&mut self, usage_percent: f32) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(usage_percent.clamp(0.0, 100.0));
    }

    /// 获取平均 CPU 使用率
    pub fn average_cpu_usage(&self) -> f32 {
        if self.history.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.history.iter().sum();
        sum / self.history.len() as f32
    }

    /// 获取峰值 CPU 使用率
    pub fn peak_cpu_usage(&self) -> f32 {
        self.history.iter().copied().fold(0.0, f32::max)
    }
}

/// 综合性能监控器
pub struct SystemPerformanceMonitor {
    /// 帧时间采样器
    pub frame_sampler: FrameTimeSampler,
    /// 内存监控器
    pub memory_monitor: MemoryMonitor,
    /// CPU 监控器
    pub cpu_monitor: CPUMonitor,
<<<<<<< HEAD
    /// 当前指标
    pub metrics: PerformanceMetrics,
=======
    /// 多维度指标存储
    pub metrics: HashMap<MetricType, Vec<f64>>,
    /// 指标历史记录
    pub metric_history: Vec<Metric>,
    /// 最大历史记录大小
    pub max_history_size: usize,
    /// 当前指标
    pub current_metrics: PerformanceMetrics,
}

impl PerformanceReport {
    /// 打印性能报告摘要
    pub fn print_summary(&self) {
        tracing::info!(target: "performance", "\n=== Performance Report ===\n");

        // CPU 指标
        tracing::info!(target: "performance", "--- CPU Metrics ---");
        if let Some(stats) = self.stats.get(&MetricType::FrameTime) {
            tracing::info!(
                target: "performance",
                "Frame Time: {:.2}ms (avg) {:.2}ms (min) {:.2}ms (max)",
                stats.avg, stats.min, stats.max
            );
        }
        if let Some(stats) = self.stats.get(&MetricType::UpdateTime) {
            tracing::info!(target: "performance", "Update Time: {:.2}ms (avg)", stats.avg);
        }
        if let Some(stats) = self.stats.get(&MetricType::RenderTime) {
            tracing::info!(target: "performance", "Render Time: {:.2}ms (avg)", stats.avg);
        }

        // GPU 指标
        tracing::info!(target: "performance", "\n--- GPU Metrics ---");
        if let Some(stats) = self.stats.get(&MetricType::GpuTime) {
            tracing::info!(target: "performance", "GPU Time: {:.2}ms (avg)", stats.avg);
        }
        if let Some(stats) = self.stats.get(&MetricType::DrawCalls) {
            tracing::info!(target: "performance", "Draw Calls: {:.0} (avg)", stats.avg);
        }
        if let Some(stats) = self.stats.get(&MetricType::TriangleCount) {
            tracing::info!(target: "performance", "Triangles: {:.0} (avg)", stats.avg);
        }

        // 内存指标
        tracing::info!(target: "performance", "\n--- Memory Metrics ---");
        if let Some(stats) = self.stats.get(&MetricType::RamUsage) {
            tracing::info!(target: "performance", "RAM Usage: {:.0}MB (avg) {:.0}MB (peak)", stats.avg, stats.max);
        }
        if let Some(stats) = self.stats.get(&MetricType::VramUsage) {
            tracing::info!(target: "performance", "VRAM Usage: {:.0}MB (avg) {:.0}MB (peak)", stats.avg, stats.max);
        }

        // 物理指标
        tracing::info!(target: "performance", "\n--- Physics Metrics ---");
        if let Some(stats) = self.stats.get(&MetricType::PhysicsTime) {
            tracing::info!(target: "performance", "Physics Time: {:.2}ms (avg)", stats.avg);
        }
        if let Some(stats) = self.stats.get(&MetricType::CollisionChecks) {
            tracing::info!(target: "performance", "Collision Checks: {:.0} (avg)", stats.avg);
        }

        // AI 指标
        tracing::info!(target: "performance", "\n--- AI Metrics ---");
        if let Some(stats) = self.stats.get(&MetricType::AiTime) {
            tracing::info!(target: "performance", "AI Time: {:.2}ms (avg)", stats.avg);
        }
        if let Some(stats) = self.stats.get(&MetricType::PathfindingTime) {
            tracing::info!(target: "performance", "Pathfinding Time: {:.2}ms (avg)", stats.avg);
        }
    }

    /// 检测性能问题
    pub fn detect_issues(&self) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        // 检测帧时间问题
        if let Some(stats) = self.stats.get(&MetricType::FrameTime) {
            if stats.avg > 33.0 {
                // 低于 30fps
                issues.push(PerformanceIssue {
                    severity: IssueSeverity::High,
                    message: format!(
                        "Low frame rate: {:.1}ms ({:.1}fps)",
                        stats.avg,
                        1000.0 / stats.avg
                    ),
                });
            }
            if stats.stddev > stats.avg * 0.5 {
                // 帧时间波动大
                issues.push(PerformanceIssue {
                    severity: IssueSeverity::Medium,
                    message: format!("High frame time variance: {:.1}ms stddev", stats.stddev),
                });
            }
        }

        // 检测 Draw Call 问题
        if let Some(stats) = self.stats.get(&MetricType::DrawCalls) {
            if stats.avg > 1000.0 {
                issues.push(PerformanceIssue {
                    severity: IssueSeverity::Medium,
                    message: format!("High draw call count: {:.0}", stats.avg),
                });
            }
        }

        // 检测内存问题
        if let Some(stats) = self.stats.get(&MetricType::RamUsage) {
            if stats.max > 2048.0 {
                // 超过 2GB
                issues.push(PerformanceIssue {
                    severity: IssueSeverity::High,
                    message: format!("High RAM usage: {:.0}MB", stats.max),
                });
            }
        }

        issues
    }
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
}

impl SystemPerformanceMonitor {
    /// 创建新的系统性能监控器
    pub fn new() -> Self {
<<<<<<< HEAD
=======
        Self::with_max_history(60 * 60) // 默认1小时历史记录
    }

    /// 创建具有自定义历史记录大小的监控器
    pub fn with_max_history(max_history: usize) -> Self {
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
        Self {
            frame_sampler: FrameTimeSampler::new(300), // 300 帧缓冲
            memory_monitor: MemoryMonitor::new(300),
            cpu_monitor: CPUMonitor::new(300),
<<<<<<< HEAD
            metrics: PerformanceMetrics {
=======
            metrics: HashMap::new(),
            metric_history: Vec::new(),
            max_history_size: max_history,
            current_metrics: PerformanceMetrics {
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
                fps: 0.0,
                frame_time_ms: 0.0,
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
                gpu_usage_percent: 0.0,
            },
        }
    }

    /// 更新一帧
    pub fn update_frame(&mut self) {
        let frame_time = self.frame_sampler.sample_frame();
<<<<<<< HEAD
        self.metrics.frame_time_ms = frame_time.as_secs_f32() * 1000.0;
        self.metrics.fps = self.frame_sampler.fps();
=======
        self.current_metrics.frame_time_ms = frame_time.as_secs_f32() * 1000.0;
        self.current_metrics.fps = self.frame_sampler.fps();
        
        // 同时记录到多维度指标系统
        self.record(MetricType::FrameTime, frame_time.as_secs_f64() * 1000.0, "ms");
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }

    /// 更新内存使用
    pub fn update_memory(&mut self, bytes: u64) {
        self.memory_monitor.sample_memory(bytes);
<<<<<<< HEAD
        self.metrics.memory_usage_mb = self.memory_monitor.current_memory_mb();
=======
        self.current_metrics.memory_usage_mb = self.memory_monitor.current_memory_mb();
        
        // 同时记录到多维度指标系统
        self.record(MetricType::RamUsage, bytes as f64 / (1024.0 * 1024.0), "MB");
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }

    /// 更新 CPU 使用率
    pub fn update_cpu(&mut self, usage_percent: f32) {
        self.cpu_monitor.sample_cpu(usage_percent);
<<<<<<< HEAD
        self.metrics.cpu_usage_percent = self.cpu_monitor.average_cpu_usage();
=======
        self.current_metrics.cpu_usage_percent = self.cpu_monitor.average_cpu_usage();
        
        // 同时记录到多维度指标系统
        self.record(MetricType::CpuTime, usage_percent as f64, "%");
    }

    /// 记录多维度性能指标
    pub fn record(&mut self, metric_type: MetricType, value: f64, unit: &str) {
        // 保存到指标历史
        let metric = Metric::new(metric_type, value, unit.to_string());
        self.metric_history.push(metric);
        
        // 保存到指标统计
        self.metrics
            .entry(metric_type)
            .or_insert_with(Vec::new)
            .push(value);
        
        // 限制历史记录大小
        if self.metric_history.len() > self.max_history_size {
            self.metric_history.remove(0);
        }
    }

    /// 获取特定类型指标的统计信息
    pub fn get_stats(&self, metric_type: MetricType) -> Option<MetricStats> {
        self.metrics
            .get(&metric_type)
            .map(|values| MetricStats::compute(metric_type, values))
    }

    /// 清空指定类型的指标数据
    pub fn clear_metric(&mut self, metric_type: MetricType) {
        self.metrics.remove(&metric_type);
    }

    /// 清空所有指标数据
    pub fn clear_all(&mut self) {
        self.metrics.clear();
        self.metric_history.clear();
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }

    /// 获取性能报告
    pub fn get_report(&self) -> PerformanceReport {
<<<<<<< HEAD
        PerformanceReport {
            current_fps: self.metrics.fps,
=======
        // 构建多维度指标统计
        let mut stats = HashMap::new();
        for (&metric_type, values) in &self.metrics {
            stats.insert(metric_type, MetricStats::compute(metric_type, values));
        }

        PerformanceReport {
            stats,
            current_fps: self.current_metrics.fps,
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
            average_frame_time_ms: self.frame_sampler.average_frame_time().as_secs_f32() * 1000.0,
            min_frame_time_ms: self
                .frame_sampler
                .min_frame_time()
                .map(|d| d.as_secs_f32() * 1000.0),
            max_frame_time_ms: self
                .frame_sampler
                .max_frame_time()
                .map(|d| d.as_secs_f32() * 1000.0),
            p99_frame_time_ms: self
                .frame_sampler
                .percentile(99.0)
                .map(|d| d.as_secs_f32() * 1000.0),
            current_memory_mb: self.memory_monitor.current_memory_mb(),
            average_memory_mb: self.memory_monitor.average_memory_mb(),
            peak_memory_mb: self.memory_monitor.peak_memory_mb(),
            average_cpu_usage: self.cpu_monitor.average_cpu_usage(),
            peak_cpu_usage: self.cpu_monitor.peak_cpu_usage(),
        }
    }

    /// 重置所有监控器
    pub fn reset(&mut self) {
        self.frame_sampler.clear();
        self.memory_monitor = MemoryMonitor::new(300);
        self.cpu_monitor = CPUMonitor::new(300);
    }
}

/// 性能报告
#[derive(Debug, Clone)]
pub struct PerformanceReport {
<<<<<<< HEAD
    /// 当前 FPS
    pub current_fps: f32,
    /// 平均帧时间 (ms)
=======
/// 多维度指标统计
pub stats: HashMap<MetricType, MetricStats>,
/// 当前 FPS
pub current_fps: f32,
/// 平均帧时间 (ms)
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    pub average_frame_time_ms: f32,
    /// 最小帧时间 (ms)
    pub min_frame_time_ms: Option<f32>,
    /// 最大帧时间 (ms)
    pub max_frame_time_ms: Option<f32>,
    /// P99 帧时间 (ms)
    pub p99_frame_time_ms: Option<f32>,
    /// 当前内存 (MB)
    pub current_memory_mb: f32,
    /// 平均内存 (MB)
    pub average_memory_mb: f32,
    /// 峰值内存 (MB)
    pub peak_memory_mb: f32,
    /// 平均 CPU 使用率 (%)
    pub average_cpu_usage: f32,
    /// 峰值 CPU 使用率 (%)
    pub peak_cpu_usage: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_frame_sampler() {
        let mut sampler = FrameTimeSampler::new(10);

        for _ in 0..5 {
            let _ = sampler.sample_frame();
            thread::sleep(Duration::from_millis(16));
        }

        assert!(sampler.fps() > 0.0);
        assert!(sampler.average_frame_time() > Duration::ZERO);
    }

    #[test]
    fn test_memory_monitor() {
        let mut monitor = MemoryMonitor::new(10);

        monitor.sample_memory(1024 * 1024); // 1 MB
        monitor.sample_memory(2 * 1024 * 1024); // 2 MB
        monitor.sample_memory(3 * 1024 * 1024); // 3 MB

        assert!(monitor.current_memory_mb() > 0.0);
        assert!(monitor.average_memory_mb() > 0.0);
        assert!(monitor.peak_memory_mb() > 0.0);
    }

    #[test]
    fn test_cpu_monitor() {
        let mut monitor = CPUMonitor::new(10);

        monitor.sample_cpu(50.0);
        monitor.sample_cpu(75.0);
        monitor.sample_cpu(25.0);

        assert!(monitor.average_cpu_usage() > 0.0);
        assert_eq!(monitor.peak_cpu_usage(), 75.0);
    }

    #[test]
    fn test_system_monitor() {
        let mut monitor = SystemPerformanceMonitor::new();

        for _ in 0..10 {
            monitor.update_frame();
            monitor.update_memory(1024 * 1024 * 100); // 100 MB
            monitor.update_cpu(50.0);
            thread::sleep(Duration::from_millis(16));
        }

        let report = monitor.get_report();
        assert!(report.current_fps > 0.0);
        assert!(report.current_memory_mb > 0.0);
    }

    #[test]
    fn test_percentile() {
        let mut sampler = FrameTimeSampler::new(100);

        for _ in 0..100 {
            sampler.samples.push_back(Duration::from_millis(16));
        }

        let p50 = sampler.percentile(50.0);
        assert!(p50.is_some());
        assert_eq!(p50.unwrap(), Duration::from_millis(16));
    }
}
