use crate::impl_default;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// 帧时间 (毫秒)
    pub frame_time: f32,
    /// FPS
    pub fps: f32,
    /// 渲染时间 (毫秒)
    pub render_time: f32,
    /// 更新时间 (毫秒)
    pub update_time: f32,
    /// 物理时间 (毫秒)
    pub physics_time: f32,
    /// 内存使用 (MB)
    pub memory_usage: f32,
    /// 绘制调用次数
    pub draw_calls: u32,
    /// 三角形数量
    pub triangle_count: u32,
}

impl_default!(PerformanceMetrics {
    frame_time: 0.0,
    fps: 0.0,
    render_time: 0.0,
    update_time: 0.0,
    physics_time: 0.0,
    memory_usage: 0.0,
    draw_calls: 0,
    triangle_count: 0,
});

/// 高级性能分析器
pub struct AdvancedProfiler {
    /// 性能指标历史记录
    metrics_history: Vec<PerformanceMetrics>,
    /// 最大历史记录数量
    max_history: usize,
    /// 计时器
    timers: HashMap<String, Instant>,
    /// 计时结果
    timing_results: HashMap<String, Duration>,
    /// 帧开始时间
    frame_start: Instant,
}

impl AdvancedProfiler {
    pub fn new(max_history: usize) -> Self {
        Self {
            metrics_history: Vec::with_capacity(max_history),
            max_history,
            timers: HashMap::new(),
            timing_results: HashMap::new(),
            frame_start: Instant::now(),
        }
    }

    /// 开始新的一帧
    pub fn begin_frame(&mut self) {
        self.frame_start = Instant::now();
        self.timers.clear();
        self.timing_results.clear();
    }

    /// 结束当前帧并记录指标
    pub fn end_frame(&mut self, metrics: PerformanceMetrics) {
        // 添加到历史记录
        if self.metrics_history.len() >= self.max_history {
            self.metrics_history.remove(0);
        }
        self.metrics_history.push(metrics);
    }

    /// 开始计时
    pub fn begin_scope(&mut self, name: impl Into<String>) {
        self.timers.insert(name.into(), Instant::now());
    }

    /// 结束计时
    pub fn end_scope(&mut self, name: impl Into<String>) {
        let name = name.into();
        if let Some(start) = self.timers.remove(&name) {
            let duration = start.elapsed();
            self.timing_results.insert(name, duration);
        }
    }

    /// 获取计时结果 (毫秒)
    pub fn get_timing(&self, name: &str) -> Option<f32> {
        self.timing_results
            .get(name)
            .map(|d| d.as_secs_f32() * 1000.0)
    }

    /// 获取平均帧时间 (毫秒)
    pub fn get_average_frame_time(&self) -> f32 {
        if self.metrics_history.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.metrics_history.iter().map(|m| m.frame_time).sum();
        sum / self.metrics_history.len() as f32
    }

    /// 获取平均FPS
    pub fn get_average_fps(&self) -> f32 {
        if self.metrics_history.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.metrics_history.iter().map(|m| m.fps).sum();
        sum / self.metrics_history.len() as f32
    }

    /// 获取最小FPS
    pub fn get_min_fps(&self) -> f32 {
        self.metrics_history
            .iter()
            .map(|m| m.fps)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    /// 获取最大FPS
    pub fn get_max_fps(&self) -> f32 {
        self.metrics_history
            .iter()
            .map(|m| m.fps)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    /// 获取最新的性能指标
    pub fn get_latest_metrics(&self) -> Option<&PerformanceMetrics> {
        self.metrics_history.last()
    }

    /// 获取性能指标历史记录
    pub fn get_metrics_history(&self) -> &[PerformanceMetrics] {
        &self.metrics_history
    }

    /// 检测性能瓶颈
    pub fn detect_bottlenecks(&self) -> Vec<String> {
        let mut bottlenecks = Vec::new();

        if let Some(metrics) = self.get_latest_metrics() {
            // 检测低FPS
            if metrics.fps < 30.0 {
                bottlenecks.push(format!("Low FPS: {:.1}", metrics.fps));
            }

            // 检测高帧时间
            if metrics.frame_time > 33.0 {
                bottlenecks.push(format!("High frame time: {:.1}ms", metrics.frame_time));
            }

            // 检测渲染瓶颈
            if metrics.render_time > metrics.frame_time * 0.7 {
                bottlenecks.push(format!(
                    "Render bottleneck: {:.1}ms ({:.0}% of frame)",
                    metrics.render_time,
                    metrics.render_time / metrics.frame_time * 100.0
                ));
            }

            // 检测物理瓶颈
            if metrics.physics_time > metrics.frame_time * 0.3 {
                bottlenecks.push(format!(
                    "Physics bottleneck: {:.1}ms ({:.0}% of frame)",
                    metrics.physics_time,
                    metrics.physics_time / metrics.frame_time * 100.0
                ));
            }

            // 检测过多的绘制调用
            if metrics.draw_calls > 1000 {
                bottlenecks.push(format!("High draw calls: {}", metrics.draw_calls));
            }
        }

        bottlenecks
    }

    /// 生成性能报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Performance Report ===\n\n");

        // 基本统计
        report.push_str(&format!("Average FPS: {:.1}\n", self.get_average_fps()));
        report.push_str(&format!("Min FPS: {:.1}\n", self.get_min_fps()));
        report.push_str(&format!("Max FPS: {:.1}\n", self.get_max_fps()));
        report.push_str(&format!(
            "Average Frame Time: {:.2}ms\n\n",
            self.get_average_frame_time()
        ));

        // 最新指标
        if let Some(metrics) = self.get_latest_metrics() {
            report.push_str("Latest Metrics:\n");
            report.push_str(&format!("  Frame Time: {:.2}ms\n", metrics.frame_time));
            report.push_str(&format!("  Render Time: {:.2}ms\n", metrics.render_time));
            report.push_str(&format!("  Update Time: {:.2}ms\n", metrics.update_time));
            report.push_str(&format!("  Physics Time: {:.2}ms\n", metrics.physics_time));
            report.push_str(&format!("  Memory Usage: {:.1}MB\n", metrics.memory_usage));
            report.push_str(&format!("  Draw Calls: {}\n", metrics.draw_calls));
            report.push_str(&format!("  Triangles: {}\n\n", metrics.triangle_count));
        }

        // 瓶颈检测
        let bottlenecks = self.detect_bottlenecks();
        if !bottlenecks.is_empty() {
            report.push_str("Detected Bottlenecks:\n");
            for bottleneck in bottlenecks {
                report.push_str(&format!("  - {}\n", bottleneck));
            }
        } else {
            report.push_str("No bottlenecks detected.\n");
        }

        report
    }
}

impl Default for AdvancedProfiler {
    fn default() -> Self {
        Self::new(300) // 保存5秒的历史记录 (60 FPS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_profiler() {
        let mut profiler = AdvancedProfiler::new(10);

        // 模拟几帧
        for i in 0..5 {
            profiler.begin_frame();

            profiler.begin_scope("render");
            thread::sleep(Duration::from_millis(5));
            profiler.end_scope("render");

            let metrics = PerformanceMetrics {
                frame_time: 16.0,
                fps: 60.0,
                render_time: profiler.get_timing("render").unwrap_or(0.0),
                update_time: 2.0,
                physics_time: 1.0,
                memory_usage: 100.0,
                draw_calls: 50,
                triangle_count: 10000,
            };

            profiler.end_frame(metrics);
        }

        assert_eq!(profiler.get_metrics_history().len(), 5);
        assert!(profiler.get_average_fps() > 0.0);
    }

    #[test]
    fn test_bottleneck_detection() {
        let mut profiler = AdvancedProfiler::new(10);

        // 模拟低FPS场景
        let metrics = PerformanceMetrics {
            frame_time: 50.0,
            fps: 20.0,
            render_time: 40.0,
            update_time: 5.0,
            physics_time: 5.0,
            memory_usage: 100.0,
            draw_calls: 1500,
            triangle_count: 100000,
        };

        profiler.begin_frame();
        profiler.end_frame(metrics);

        let bottlenecks = profiler.detect_bottlenecks();
        assert!(!bottlenecks.is_empty());
    }
}
