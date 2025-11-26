use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// 性能样本
#[derive(Debug, Clone)]
pub struct PerformanceSample {
    pub timestamp: Instant,
    pub frame_time_ms: f32,
    pub fps: f32,
    pub cpu_usage: f32,
    pub memory_mb: f32,
}

/// 持续性能分析器
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
    pub fn begin_frame(&mut self) {
        if !self.enabled {
            return;
        }
        
        self.frame_count += 1;
        
        // 只在采样间隔时记录
        if self.frame_count % self.sample_interval != 0 {
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
    
    /// 获取所有样本
    pub fn get_samples(&self) -> &VecDeque<PerformanceSample> {
        &self.samples
    }
    
    /// 获取平均FPS
    pub fn get_average_fps(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        
        let sum: f32 = self.samples.iter().map(|s| s.fps).sum();
        sum / self.samples.len() as f32
    }
    
    /// 获取平均帧时间
    pub fn get_average_frame_time(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        
        let sum: f32 = self.samples.iter().map(|s| s.frame_time_ms).sum();
        sum / self.samples.len() as f32
    }
    
    /// 获取FPS范围
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
            anomalies.push(format!("High FPS variance: {:.1} (min: {:.1}, max: {:.1})", 
                fps_variance, min_fps, max_fps));
        }
        
        // 检测帧时间尖峰
        let avg_frame_time = self.get_average_frame_time();
        for sample in self.samples.iter().rev().take(10) {
            if sample.frame_time_ms > avg_frame_time * 2.0 {
                anomalies.push(format!("Frame time spike detected: {:.2}ms (avg: {:.2}ms)", 
                    sample.frame_time_ms, avg_frame_time));
                break;
            }
        }
        
        anomalies
    }
    
    /// 生成性能报告
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
    
    /// 清空样本
    pub fn clear(&mut self) {
        self.samples.clear();
        self.frame_count = 0;
    }
    
    /// 设置是否启用
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// 设置采样间隔
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
        println!("Detected anomalies: {:?}", anomalies);
    }
}
