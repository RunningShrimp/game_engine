// 统一剖析接口
//
// 提供统一的性能剖析接口，集成多种剖析后端

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ============================================================================
// 剖析后端类型
// ============================================================================

/// 剖析后端
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProfilingBackend {
    /// Tracy剖析器
    Tracy,
    /// 内置轻量级剖析器
    BuiltIn,
    /// Chrome tracing格式
    ChromeTracing,
    /// 禁用剖析
    Disabled,
}

// ============================================================================
// Tracy集成
// ============================================================================

// Tracy集成代码被注释掉，因为需要tracy feature
// 当启用tracy feature时，可以使用tracy-client crate

// ============================================================================
// 帧时间剖析
// ============================================================================

/// 帧时间剖析器
#[derive(Debug, Clone)]
pub struct FrameTimeProfiler {
    /// 帧时间历史
    frame_times: Vec<Duration>,
    /// 最大历史记录数
    max_history: usize,
    /// 当前帧开始时间
    current_frame_start: Option<Instant>,
    /// 帧计数
    frame_count: u64,
}

impl FrameTimeProfiler {
    /// 创建新的帧时间剖析器
    pub fn new(max_history: usize) -> Self {
        Self {
            frame_times: Vec::with_capacity(max_history),
            max_history,
            current_frame_start: None,
            frame_count: 0,
        }
    }

    /// 开始帧
    pub fn begin_frame(&mut self) {
        self.current_frame_start = Some(Instant::now());
    }

    /// 结束帧
    pub fn end_frame(&mut self) {
        if let Some(start) = self.current_frame_start.take() {
            let frame_time = start.elapsed();
            self.frame_times.push(frame_time);
            self.frame_count += 1;

            // 保持历史记录大小
            if self.frame_times.len() > self.max_history {
                self.frame_times.remove(0);
            }
        }
    }

    /// 获取平均帧时间
    pub fn average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }
        let sum: Duration = self.frame_times.iter().sum();
        sum / self.frame_times.len() as u32
    }

    /// 获取FPS
    pub fn fps(&self) -> f64 {
        let avg_time = self.average_frame_time();
        if avg_time == Duration::ZERO {
            0.0
        } else {
            1_000_000_000.0 / avg_time.as_nanos() as f64
        }
    }

    /// 获取帧时间百分位数
    pub fn percentile(&self, percentile: f64) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }

        let mut sorted = self.frame_times.clone();
        sorted.sort();
        let index = ((sorted.len() as f64 - 1.0) * percentile / 100.0) as usize;
        sorted[index]
    }

    /// 获取帧时间统计
    pub fn statistics(&self) -> FrameTimeStatistics {
        if self.frame_times.is_empty() {
            return FrameTimeStatistics::default();
        }

        let min = *self.frame_times.iter().min().unwrap();
        let max = *self.frame_times.iter().max().unwrap();
        let avg = self.average_frame_time();
        let p50 = self.percentile(50.0);
        let p95 = self.percentile(95.0);
        let p99 = self.percentile(99.0);

        FrameTimeStatistics {
            frame_count: self.frame_count,
            min,
            max,
            average: avg,
            p50,
            p95,
            p99,
        }
    }

    /// 清空历史记录
    pub fn clear(&mut self) {
        self.frame_times.clear();
        self.frame_count = 0;
    }
}

/// 帧时间统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameTimeStatistics {
    /// 总帧数
    pub frame_count: u64,
    /// 最小帧时间
    pub min: Duration,
    /// 最大帧时间
    pub max: Duration,
    /// 平均帧时间
    pub average: Duration,
    /// 50百分位（中位数）
    pub p50: Duration,
    /// 95百分位
    pub p95: Duration,
    /// 99百分位
    pub p99: Duration,
}

impl Default for FrameTimeStatistics {
    fn default() -> Self {
        Self {
            frame_count: 0,
            min: Duration::ZERO,
            max: Duration::ZERO,
            average: Duration::ZERO,
            p50: Duration::ZERO,
            p95: Duration::ZERO,
            p99: Duration::ZERO,
        }
    }
}

// ============================================================================
// 内存分配追踪
// ============================================================================

/// 内存分配追踪器
#[derive(Debug, Default)]
pub struct MemoryAllocationTracker {
    /// 总分配字节数
    total_allocated: AtomicU64,
    /// 总释放字节数
    total_freed: AtomicU64,
    /// 当前使用字节数
    current_usage: AtomicU64,
    /// 分配计数
    allocation_count: AtomicU64,
    /// 释放计数
    deallocation_count: AtomicU64,
    /// 峰值使用量
    peak_usage: AtomicU64,
}

use std::sync::atomic::{AtomicU64, Ordering};

impl MemoryAllocationTracker {
    /// 创建新的内存追踪器
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录分配
    pub fn record_allocation(&self, size: usize) {
        let size = size as u64;
        self.total_allocated.fetch_add(size, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        // 更新当前使用量和峰值
        let old_usage = self.current_usage.fetch_add(size, Ordering::Relaxed);
        let new_usage = old_usage + size;

        // 更新峰值
        let mut current_peak = self.peak_usage.load(Ordering::Relaxed);
        while new_usage > current_peak {
            match self.peak_usage.compare_exchange_weak(
                current_peak,
                new_usage,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }
    }

    /// 记录释放
    pub fn record_deallocation(&self, size: usize) {
        let size = size as u64;
        self.total_freed.fetch_add(size, Ordering::Relaxed);
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
        self.current_usage.fetch_sub(size, Ordering::Relaxed);
    }

    /// 获取总分配量
    pub fn total_allocated(&self) -> u64 {
        self.total_allocated.load(Ordering::Relaxed)
    }

    /// 获取总释放量
    pub fn total_freed(&self) -> u64 {
        self.total_freed.load(Ordering::Relaxed)
    }

    /// 获取当前使用量
    pub fn current_usage(&self) -> u64 {
        self.current_usage.load(Ordering::Relaxed)
    }

    /// 获取峰值使用量
    pub fn peak_usage(&self) -> u64 {
        self.peak_usage.load(Ordering::Relaxed)
    }

    /// 获取分配计数
    pub fn allocation_count(&self) -> u64 {
        self.allocation_count.load(Ordering::Relaxed)
    }

    /// 获取释放计数
    pub fn deallocation_count(&self) -> u64 {
        self.deallocation_count.load(Ordering::Relaxed)
    }

    /// 重置统计信息
    pub fn reset(&self) {
        self.total_allocated.store(0, Ordering::Relaxed);
        self.total_freed.store(0, Ordering::Relaxed);
        self.current_usage.store(0, Ordering::Relaxed);
        self.allocation_count.store(0, Ordering::Relaxed);
        self.deallocation_count.store(0, Ordering::Relaxed);
        self.peak_usage.store(0, Ordering::Relaxed);
    }
}

// ============================================================================
// 函数耗时统计
// ============================================================================

/// 函数统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionStatistics {
    /// 函数名
    pub name: String,
    /// 调用次数
    pub call_count: u64,
    /// 总耗时
    pub total_time: Duration,
    /// 最小耗时
    pub min_time: Duration,
    /// 最大耗时
    pub max_time: Duration,
    /// 平均耗时
    pub average_time: Duration,
}

/// 函数耗时统计器
#[derive(Debug, Default)]
pub struct FunctionProfiler {
    /// 函数统计信息映射
    functions: HashMap<String, FunctionStats>,
}

#[derive(Debug)]
struct FunctionStats {
    call_count: u64,
    total_time: Duration,
    min_time: Duration,
    max_time: Duration,
}

impl FunctionProfiler {
    /// 创建新的函数剖析器
    pub fn new() -> Self {
        Self::default()
    }

    /// 开始函数计时
    pub fn begin_function(&mut self, _name: &str) -> FunctionGuard {
        FunctionGuard {
            profiler: self,
            name: _name.to_string(),
            start: Instant::now(),
        }
    }

    /// 记录函数调用
    fn record_function(&mut self, name: String, duration: Duration) {
        let stats = self.functions.entry(name).or_insert(FunctionStats {
            call_count: 0,
            total_time: Duration::ZERO,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
        });

        stats.call_count += 1;
        stats.total_time += duration;
        stats.min_time = stats.min_time.min(duration);
        stats.max_time = stats.max_time.max(duration);
    }

    /// 获取函数统计信息
    pub fn get_statistics(&self, name: &str) -> Option<FunctionStatistics> {
        let stats = self.functions.get(name)?;
        Some(FunctionStatistics {
            name: name.to_string(),
            call_count: stats.call_count,
            total_time: stats.total_time,
            min_time: stats.min_time,
            max_time: stats.max_time,
            average_time: if stats.call_count > 0 {
                stats.total_time / stats.call_count as u32
            } else {
                Duration::ZERO
            },
        })
    }

    /// 获取所有统计信息
    pub fn get_all_statistics(&self) -> Vec<FunctionStatistics> {
        self.functions
            .iter()
            .map(|(name, stats)| FunctionStatistics {
                name: name.clone(),
                call_count: stats.call_count,
                total_time: stats.total_time,
                min_time: stats.min_time,
                max_time: stats.max_time,
                average_time: if stats.call_count > 0 {
                    stats.total_time / stats.call_count as u32
                } else {
                    Duration::ZERO
                },
            })
            .collect()
    }

    /// 清空统计信息
    pub fn clear(&mut self) {
        self.functions.clear();
    }
}

/// 函数计时守卫
pub struct FunctionGuard<'a> {
    profiler: &'a mut FunctionProfiler,
    name: String,
    start: Instant,
}

impl<'a> Drop for FunctionGuard<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.profiler.record_function(self.name.clone(), duration);
    }
}

// ============================================================================
// 统一剖析器
// ============================================================================

/// 统一剖析器
pub struct UnifiedProfiler {
    /// 帧时间剖析器
    frame_profiler: FrameTimeProfiler,
    /// 内存分配追踪器
    memory_tracker: MemoryAllocationTracker,
    /// 函数剖析器
    function_profiler: FunctionProfiler,
    /// 剖析后端
    backend: ProfilingBackend,
}

impl UnifiedProfiler {
    /// 创建新的统一剖析器
    pub fn new(backend: ProfilingBackend) -> Self {
        Self {
            frame_profiler: FrameTimeProfiler::new(1000),
            memory_tracker: MemoryAllocationTracker::new(),
            function_profiler: FunctionProfiler::new(),
            backend,
        }
    }

    /// 获取帧时间剖析器
    pub fn frame_profiler(&mut self) -> &mut FrameTimeProfiler {
        &mut self.frame_profiler
    }

    /// 获取内存追踪器
    pub fn memory_tracker(&self) -> &MemoryAllocationTracker {
        &self.memory_tracker
    }

    /// 获取函数剖析器
    pub fn function_profiler(&mut self) -> &mut FunctionProfiler {
        &mut self.function_profiler
    }

    /// 开始帧
    pub fn begin_frame(&mut self) {
        self.frame_profiler.begin_frame();
    }

    /// 结束帧
    pub fn end_frame(&mut self) {
        self.frame_profiler.end_frame();
    }

    /// 记录内存分配
    pub fn record_allocation(&self, size: usize) {
        self.memory_tracker.record_allocation(size);
    }

    /// 记录内存释放
    pub fn record_deallocation(&self, size: usize) {
        self.memory_tracker.record_deallocation(size);
    }

    /// 生成报告
    pub fn generate_report(&self) -> ProfilingReport {
        ProfilingReport {
            frame_stats: self.frame_profiler.statistics(),
            memory_usage: self.memory_tracker.current_usage(),
            memory_peak: self.memory_tracker.peak_usage(),
            allocation_count: self.memory_tracker.allocation_count(),
            function_stats: self.function_profiler.get_all_statistics(),
        }
    }
}

/// 剖析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingReport {
    /// 帧统计信息
    pub frame_stats: FrameTimeStatistics,
    /// 内存使用量
    pub memory_usage: u64,
    /// 内存峰值
    pub memory_peak: u64,
    /// 分配计数
    pub allocation_count: u64,
    /// 函数统计信息
    pub function_stats: Vec<FunctionStatistics>,
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_time_profiler() {
        let mut profiler = FrameTimeProfiler::new(10);

        profiler.begin_frame();
        std::thread::sleep(Duration::from_millis(10));
        profiler.end_frame();

        profiler.begin_frame();
        std::thread::sleep(Duration::from_millis(20));
        profiler.end_frame();

        let stats = profiler.statistics();
        assert_eq!(stats.frame_count, 2);
        assert!(stats.average >= Duration::from_millis(15));
        assert!(stats.min >= Duration::from_millis(10));
        assert!(stats.max >= Duration::from_millis(20));
    }

    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryAllocationTracker::new();

        tracker.record_allocation(100);
        tracker.record_allocation(200);
        assert_eq!(tracker.current_usage(), 300);
        assert_eq!(tracker.peak_usage(), 300);
        assert_eq!(tracker.allocation_count(), 2);

        tracker.record_deallocation(100);
        assert_eq!(tracker.current_usage(), 200);
        assert_eq!(tracker.deallocation_count(), 1);
    }

    #[test]
    fn test_function_profiler() {
        let mut profiler = FunctionProfiler::new();

        {
            let _guard = profiler.begin_function("test_function");
            std::thread::sleep(Duration::from_millis(10));
        }

        let stats = profiler.get_statistics("test_function").unwrap();
        assert_eq!(stats.call_count, 1);
        assert!(stats.total_time >= Duration::from_millis(10));
    }

    #[test]
    fn test_unified_profiler() {
        let mut profiler = UnifiedProfiler::new(ProfilingBackend::BuiltIn);

        profiler.begin_frame();
        profiler.record_allocation(1000);
        std::thread::sleep(Duration::from_millis(5));
        profiler.end_frame();

        let report = profiler.generate_report();
        assert_eq!(report.frame_stats.frame_count, 1);
        assert_eq!(report.memory_usage, 1000);
    }
}
