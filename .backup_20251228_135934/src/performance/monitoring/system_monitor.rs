//  系统性能监控器
// 
//  实时性能监控和数据收集
//  - 帧率监控
//  - 内存跟踪
//  - CPU 使用率
//  - 性能统计

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// 性能指标
#[derive(Debug, Clone, Copy, Default)]
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
    /// GPU渲染时间细分（毫秒）
    pub gpu_render_time_ms: f32,
    /// GPU几何处理时间（毫秒）
    pub gpu_geometry_time_ms: f32,
    /// GPU光照计算时间（毫秒）
    pub gpu_lighting_time_ms: f32,
    /// GPU后处理时间（毫秒）
    pub gpu_postprocess_time_ms: f32,
    /// ECS系统执行时间（毫秒）
    pub ecs_system_time_ms: f32,
    /// 物理引擎更新时间（毫秒）
    pub physics_update_time_ms: f32,
    /// 网络同步延迟（毫秒）
    pub network_sync_latency_ms: f32,
    /// 内存分配次数
    pub memory_allocations: u64,
    /// 内存释放次数
    pub memory_deallocations: u64,
}

/// 帧时间采样器
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
pub struct SystemPerformanceMonitor {
    /// 帧时间采样器
    pub frame_sampler: FrameTimeSampler,
    /// 内存监控器
    pub memory_monitor: MemoryMonitor,
    /// CPU 监控器
    pub cpu_monitor: CPUMonitor,
    /// 当前指标
    pub metrics: PerformanceMetrics,
}

impl SystemPerformanceMonitor {
    /// 创建新的系统性能监控器
    pub fn new() -> Self {
        Self {
            frame_sampler: FrameTimeSampler::new(300), // 300 帧缓冲
            memory_monitor: MemoryMonitor::new(300),
            cpu_monitor: CPUMonitor::new(300),
            metrics: PerformanceMetrics::default(),
        }
    }

    /// 更新一帧
    pub fn update_frame(&mut self) {
        let frame_time = self.frame_sampler.sample_frame();
        self.metrics.frame_time_ms = frame_time.as_secs_f32() * 1000.0;
        self.metrics.fps = self.frame_sampler.fps();
    }

    /// 更新内存使用
    pub fn update_memory(&mut self, bytes: u64) {
        self.memory_monitor.sample_memory(bytes);
        self.metrics.memory_usage_mb = self.memory_monitor.current_memory_mb();
    }

    /// 更新 CPU 使用率
    pub fn update_cpu(&mut self, usage_percent: f32) {
        self.cpu_monitor.sample_cpu(usage_percent);
        self.metrics.cpu_usage_percent = self.cpu_monitor.average_cpu_usage();
    }

    /// 获取性能报告
    pub fn get_report(&self) -> PerformanceReport {
        PerformanceReport {
            current_fps: self.metrics.fps,
            average_frame_time_ms: self.frame_sampler.average_frame_time().as_secs_f32() * 1000.0,
            min_frame_time_ms: self.frame_sampler.min_frame_time().map(|d| d.as_secs_f32() * 1000.0),
            max_frame_time_ms: self.frame_sampler.max_frame_time().map(|d| d.as_secs_f32() * 1000.0),
            p99_frame_time_ms: self.frame_sampler.percentile(99.0).map(|d| d.as_secs_f32() * 1000.0),
            current_memory_mb: self.memory_monitor.current_memory_mb(),
            average_memory_mb: self.memory_monitor.average_memory_mb(),
            peak_memory_mb: self.memory_monitor.peak_memory_mb(),
            average_cpu_usage: self.cpu_monitor.average_cpu_usage(),
            peak_cpu_usage: self.cpu_monitor.peak_cpu_usage(),
        }
    }

    /// 返回当前性能指标快照
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics
    }

    /// 更新GPU渲染时间细分
    pub fn update_gpu_render_times(&mut self, render_time_ms: f32, geometry_time_ms: f32, lighting_time_ms: f32, postprocess_time_ms: f32) {
        self.metrics.gpu_render_time_ms = render_time_ms;
        self.metrics.gpu_geometry_time_ms = geometry_time_ms;
        self.metrics.gpu_lighting_time_ms = lighting_time_ms;
        self.metrics.gpu_postprocess_time_ms = postprocess_time_ms;
    }

    /// 更新ECS系统执行时间
    pub fn update_ecs_system_time(&mut self, time_ms: f32) {
        self.metrics.ecs_system_time_ms = time_ms;
    }

    /// 更新物理引擎更新时间
    pub fn update_physics_time(&mut self, time_ms: f32) {
        self.metrics.physics_update_time_ms = time_ms;
    }

    /// 更新网络同步延迟
    pub fn update_network_latency(&mut self, latency_ms: f32) {
        self.metrics.network_sync_latency_ms = latency_ms;
    }

    /// 更新内存分配统计
    pub fn update_memory_stats(&mut self, allocations: u64, deallocations: u64) {
        self.metrics.memory_allocations = allocations;
        self.metrics.memory_deallocations = deallocations;
    }

    /// 重置所有监控器
    pub fn reset(&mut self) {
        self.frame_sampler.clear();
        self.memory_monitor = MemoryMonitor::new(300);
        self.cpu_monitor = CPUMonitor::new(300);
        self.metrics = PerformanceMetrics::default();
    }
}

impl Default for SystemPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能报告
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// 当前 FPS
    pub current_fps: f32,
    /// 平均帧时间 (ms)
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
