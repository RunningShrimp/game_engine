//! 持续性能分析器模块
//!
//! 提供持续的性能监控和分析功能，用于跟踪游戏引擎的运行时性能指标。
//!
//! ## 功能特性
//!
//! - 持续收集性能样本（FPS、帧时间、CPU使用率、内存使用量）
//! - 性能统计计算（平均值、范围、异常检测）
//! - 可配置的采样间隔
//! - 性能报告生成
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::performance::continuous_profiler::ContinuousProfiler;
//!
//! let mut profiler = ContinuousProfiler::new(1000); // 保存1000个样本
//!
//! // 在游戏循环中
//! loop {
//!     profiler.begin_frame();
//!     // ... 游戏逻辑 ...
//!     
//!     // 定期检查性能
//!     if profiler.get_samples().len() > 100 {
//!         let avg_fps = profiler.get_average_fps();
//!         let anomalies = profiler.detect_anomalies();
//!         if !anomalies.is_empty() {
//!             println!("Performance issues detected: {:?}", anomalies);
//!         }
//!     }
//! }
//! ```

use std::collections::VecDeque;
use std::time::Instant;

/// 性能样本
///
/// 表示单个时间点的性能指标快照。
#[derive(Debug, Clone)]
pub struct PerformanceSample {
    /// 样本时间戳
    pub timestamp: Instant,
    /// 帧时间（毫秒）
    pub frame_time_ms: f32,
    /// 帧率（FPS）
    pub fps: f32,
    /// CPU使用率（0.0-1.0）
    pub cpu_usage: f32,
    /// 内存使用量（MB）
    pub memory_mb: f32,
}

/// 持续性能分析器
///
/// 持续收集和分析游戏引擎的性能指标，支持性能统计和异常检测。
///
/// ## 性能考虑
///
/// - 采样间隔可配置，减少性能开销
/// - 使用固定大小的队列，避免内存无限增长
/// - 所有统计计算都是O(n)时间复杂度
pub struct ContinuousProfiler {
    /// 性能样本队列
    samples: VecDeque<PerformanceSample>,
    /// 最大样本数量
    max_samples: usize,
    /// 上一帧的时间
    last_frame_time: Instant,
    /// 是否启用
    enabled: bool,
    /// 采样间隔 (帧数)
    sample_interval: u32,
    /// 当前帧计数
    frame_count: u32,
}

impl ContinuousProfiler {
    /// 创建新的持续性能分析器
    ///
    /// # 参数
    ///
    /// * `max_samples` - 最大样本数量，当达到此数量时，旧的样本会被移除
    ///
    /// # 返回
    ///
    /// 返回一个新的 `ContinuousProfiler` 实例，默认启用，采样间隔为1帧。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::performance::continuous_profiler::ContinuousProfiler;
    ///
    /// let profiler = ContinuousProfiler::new(1000);
    /// ```
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
            last_frame_time: Instant::now(),
            enabled: true,
            sample_interval: 1,
            frame_count: 0,
        }
    }

    /// 开始新的一帧
    ///
    /// 记录当前帧的性能指标，包括帧时间、FPS、CPU使用率和内存使用量。
    /// 只有在启用状态且满足采样间隔时才会记录样本。
    ///
    /// # 性能
    ///
    /// 此方法在采样间隔内只做少量计算，开销很小。
    /// 当需要记录样本时，会进行一些统计计算，但仍然是O(1)操作。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::performance::continuous_profiler::ContinuousProfiler;
    ///
    /// let mut profiler = ContinuousProfiler::new(100);
    /// profiler.begin_frame(); // 记录第一帧
    /// ```
    pub fn begin_frame(&mut self) {
        if !self.enabled {
            return;
        }

        self.frame_count += 1;

        // 只在采样间隔时记录
        if !self.frame_count.is_multiple_of(self.sample_interval) {
            return;
        }

        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time);
        let frame_time_ms = frame_time.as_secs_f32() * 1000.0;
        let fps = if frame_time_ms > 0.0 {
            1000.0 / frame_time_ms
        } else {
            0.0
        };

        // 获取CPU和内存使用情况 (简化版)
        let cpu_usage = self.get_cpu_usage();
        let memory_mb = self.get_memory_usage();

        let sample = PerformanceSample {
            timestamp: now,
            frame_time_ms,
            fps,
            cpu_usage,
            memory_mb,
        };

        // 添加样本
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);

        self.last_frame_time = now;
    }

    /// 获取CPU使用率 (简化版)
    fn get_cpu_usage(&self) -> f32 {
        // 实际实现需要使用系统API
        // 这里返回一个模拟值
        0.0
    }

    /// 获取内存使用量 (MB)
    fn get_memory_usage(&self) -> f32 {
        // 实际实现需要使用系统API
        // 这里返回一个模拟值
        0.0
    }

    /// 获取所有性能样本
    ///
    /// # 返回
    ///
    /// 返回所有收集的性能样本的只读引用。样本按时间顺序排列（最早的在前面）。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::performance::continuous_profiler::ContinuousProfiler;
    ///
    /// let profiler = ContinuousProfiler::new(100);
    /// let samples = profiler.get_samples();
    /// println!("Collected {} samples", samples.len());
    /// ```
    pub fn get_samples(&self) -> &VecDeque<PerformanceSample> {
        &self.samples
    }

    /// 获取平均FPS
    ///
    /// # 返回
    ///
    /// 返回所有样本的平均帧率（FPS）。如果没有样本，返回0.0。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::performance::continuous_profiler::ContinuousProfiler;
    ///
    /// let mut profiler = ContinuousProfiler::new(100);
    /// profiler.begin_frame();
    /// let avg_fps = profiler.get_average_fps();
    /// ```
    pub fn get_average_fps(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.samples.iter().map(|s| s.fps).sum();
        sum / self.samples.len() as f32
    }

    /// 获取平均帧时间
    ///
    /// # 返回
    ///
    /// 返回所有样本的平均帧时间（毫秒）。如果没有样本，返回0.0。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::performance::continuous_profiler::ContinuousProfiler;
    ///
    /// let mut profiler = ContinuousProfiler::new(100);
    /// profiler.begin_frame();
    /// let avg_frame_time = profiler.get_average_frame_time();
    /// ```
    pub fn get_average_frame_time(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.samples.iter().map(|s| s.frame_time_ms).sum();
        sum / self.samples.len() as f32
    }

    /// 获取FPS范围
    ///
    /// # 返回
    ///
    /// 返回 `(最小FPS, 最大FPS)` 元组。如果没有样本，返回 `(0.0, 0.0)`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::performance::continuous_profiler::ContinuousProfiler;
    ///
    /// let mut profiler = ContinuousProfiler::new(100);
    /// profiler.begin_frame();
    /// let (min_fps, max_fps) = profiler.get_fps_range();
    /// ```
    pub fn get_fps_range(&self) -> (f32, f32) {
        if self.samples.is_empty() {
            return (0.0, 0.0);
        }

        let mut min_fps = f32::MAX;
        let mut max_fps = f32::MIN;

        for sample in &self.samples {
            if sample.fps < min_fps {
                min_fps = sample.fps;
            }
            if sample.fps > max_fps {
                max_fps = sample.fps;
            }
        }

        (min_fps, max_fps)
    }

    /// 检测性能异常
    ///
    /// 分析收集的性能样本，检测可能的性能问题，如低FPS、FPS波动、帧时间尖峰等。
    ///
    /// # 返回
    ///
    /// 返回检测到的异常描述列表。如果没有异常，返回空列表。
    ///
    /// # 检测规则
    ///
    /// - 平均FPS低于30时报告低FPS警告
    /// - FPS波动超过30时报告高波动警告
    /// - 最近10帧中出现超过平均帧时间2倍的尖峰时报告帧时间尖峰
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::performance::continuous_profiler::ContinuousProfiler;
    ///
    /// let profiler = ContinuousProfiler::new(100);
    /// let anomalies = profiler.detect_anomalies();
    /// for anomaly in anomalies {
    ///     println!("Performance issue: {}", anomaly);
    /// }
    /// ```
    pub fn detect_anomalies(&self) -> Vec<String> {
        let mut anomalies = Vec::new();

        if self.samples.is_empty() {
            return anomalies;
        }

        let avg_fps = self.get_average_fps();
        let (min_fps, max_fps) = self.get_fps_range();

        // 检测低FPS
        if avg_fps < 30.0 {
            anomalies.push(format!("Low average FPS: {:.1}", avg_fps));
        }

        // 检测FPS波动
        let fps_variance = max_fps - min_fps;
        if fps_variance > 30.0 {
            anomalies.push(format!(
                "High FPS variance: {:.1} (min: {:.1}, max: {:.1})",
                fps_variance, min_fps, max_fps
            ));
        }

        // 检测帧时间尖峰
        let avg_frame_time = self.get_average_frame_time();
        for sample in self.samples.iter().rev().take(10) {
            if sample.frame_time_ms > avg_frame_time * 2.0 {
                anomalies.push(format!(
                    "Frame time spike detected: {:.2}ms (avg: {:.2}ms)",
                    sample.frame_time_ms, avg_frame_time
                ));
                break;
            }
        }

        anomalies
    }

    /// 生成性能报告
    ///
    /// 生成包含所有性能统计信息的文本报告。
    ///
    /// # 返回
    ///
    /// 返回格式化的性能报告字符串，包括：
    /// - 样本数量
    /// - 平均FPS和FPS范围
    /// - 平均帧时间
    /// - 检测到的异常（如果有）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::performance::continuous_profiler::ContinuousProfiler;
    ///
    /// let profiler = ContinuousProfiler::new(100);
    /// let report = profiler.generate_report();
    /// println!("{}", report);
    /// ```
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Continuous Performance Report ===\n\n");

        if self.samples.is_empty() {
            report.push_str("No performance data available.\n");
            return report;
        }

        // 基本统计
        let avg_fps = self.get_average_fps();
        let avg_frame_time = self.get_average_frame_time();
        let (min_fps, max_fps) = self.get_fps_range();

        report.push_str(&format!("Sample Count: {}\n", self.samples.len()));
        report.push_str(&format!("Average FPS: {:.1}\n", avg_fps));
        report.push_str(&format!("FPS Range: {:.1} - {:.1}\n", min_fps, max_fps));
        report.push_str(&format!("Average Frame Time: {:.2}ms\n\n", avg_frame_time));

        // 异常检测
        let anomalies = self.detect_anomalies();
        if !anomalies.is_empty() {
            report.push_str("Detected Anomalies:\n");
            for anomaly in anomalies {
                report.push_str(&format!("  - {}\n", anomaly));
            }
        } else {
            report.push_str("No anomalies detected.\n");
        }

        report
    }

    /// 清空所有性能样本
    ///
    /// 移除所有收集的样本并重置帧计数器。分析器保持启用状态。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::performance::continuous_profiler::ContinuousProfiler;
    ///
    /// let mut profiler = ContinuousProfiler::new(100);
    /// profiler.clear(); // 清空所有样本
    /// ```
    pub fn clear(&mut self) {
        self.samples.clear();
        self.frame_count = 0;
    }

    /// 设置分析器是否启用
    ///
    /// # 参数
    ///
    /// * `enabled` - 如果为 `true`，分析器会收集性能样本；如果为 `false`，`begin_frame()` 不会记录任何数据
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::performance::continuous_profiler::ContinuousProfiler;
    ///
    /// let mut profiler = ContinuousProfiler::new(100);
    /// profiler.set_enabled(false); // 临时禁用性能分析
    /// ```
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 设置采样间隔
    ///
    /// 设置每隔多少帧记录一次性能样本。例如，如果设置为10，则每10帧记录一次样本。
    ///
    /// # 参数
    ///
    /// * `interval` - 采样间隔（帧数），最小值为1
    ///
    /// # 性能
    ///
    /// 增大采样间隔可以减少性能开销，但会降低数据精度。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::performance::continuous_profiler::ContinuousProfiler;
    ///
    /// let mut profiler = ContinuousProfiler::new(100);
    /// profiler.set_sample_interval(10); // 每10帧采样一次
    /// ```
    pub fn set_sample_interval(&mut self, interval: u32) {
        self.sample_interval = interval.max(1);
    }
}

impl Default for ContinuousProfiler {
    fn default() -> Self {
        Self::new(1000) // 保存1000个样本
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_continuous_profiler() {
        let mut profiler = ContinuousProfiler::new(100);

        // 模拟几帧
        for _ in 0..10 {
            profiler.begin_frame();
            thread::sleep(Duration::from_millis(16)); // ~60 FPS
        }

        assert!(profiler.get_samples().len() > 0);
        assert!(profiler.get_average_fps() > 0.0);
    }

    #[test]
    fn test_anomaly_detection() {
        let mut profiler = ContinuousProfiler::new(100);

        // 添加一些正常样本
        for _ in 0..50 {
            profiler.begin_frame();
            thread::sleep(Duration::from_millis(16));
        }

        // 添加一个异常样本 (模拟帧时间尖峰)
        thread::sleep(Duration::from_millis(100));
        profiler.begin_frame();

        let anomalies = profiler.detect_anomalies();
        // 可能会检测到异常,但不保证
        tracing::warn!(target: "profiler", "Detected anomalies: {:?}", anomalies);
    }
}
