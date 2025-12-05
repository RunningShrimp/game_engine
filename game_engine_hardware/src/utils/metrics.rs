/// 增强的性能监控系统
/// 
/// 提供详细的性能指标收集和分析

use crate::utils::ring_buffer::RingBuffer;
use std::time::{Instant, Duration};
use std::collections::HashMap;

/// 详细的性能指标
#[derive(Debug, Clone)]
pub struct DetailedMetrics {
    // 基础指标
    pub frame_time_ms: f32,
    pub fps: f32,
    pub frame_number: u64,
    
    // GPU指标
    pub gpu_utilization: Option<f32>,  // 0.0-1.0
    pub vram_used_mb: Option<u64>,
    pub vram_total_mb: Option<u64>,
    pub vram_usage_percent: Option<f32>,
    
    // CPU指标
    pub cpu_utilization: Option<f32>,
    pub ram_used_mb: Option<u64>,
    pub ram_total_mb: Option<u64>,
    pub ram_usage_percent: Option<f32>,
    
    // 热指标
    pub gpu_temperature: Option<f32>,
    pub cpu_temperature: Option<f32>,
    
    // 功耗（移动端）
    pub power_draw_watts: Option<f32>,
    pub battery_level: Option<f32>,
    pub battery_charging: Option<bool>,
    
    // 渲染指标
    pub draw_calls: Option<u32>,
    pub triangles: Option<u64>,
    pub vertices: Option<u64>,
    
    // 时间戳
    pub timestamp: Instant,
}

impl DetailedMetrics {
    /// 创建新的指标
    pub fn new(frame_time_ms: f32, frame_number: u64) -> Self {
        Self {
            frame_time_ms,
            fps: 1000.0 / frame_time_ms,
            frame_number,
            gpu_utilization: None,
            vram_used_mb: None,
            vram_total_mb: None,
            vram_usage_percent: None,
            cpu_utilization: None,
            ram_used_mb: None,
            ram_total_mb: None,
            ram_usage_percent: None,
            gpu_temperature: None,
            cpu_temperature: None,
            power_draw_watts: None,
            battery_level: None,
            battery_charging: None,
            draw_calls: None,
            triangles: None,
            vertices: None,
            timestamp: Instant::now(),
        }
    }
}

/// 性能分析器
pub struct PerformanceProfiler {
    sections: HashMap<String, SectionMetrics>,
    current_section: Option<String>,
    section_start: Instant,
    frame_start: Instant,
    enabled: bool,
}

/// 分段性能指标
#[derive(Debug, Clone)]
struct SectionMetrics {
    name: String,
    samples: RingBuffer<f32>,
    total_time_ms: f64,
    call_count: u64,
}

impl SectionMetrics {
    fn new(name: String) -> Self {
        Self {
            name,
            samples: RingBuffer::new(300),
            total_time_ms: 0.0,
            call_count: 0,
        }
    }
    
    fn add_sample(&mut self, time_ms: f32) {
        self.samples.push(time_ms);
        self.total_time_ms += time_ms as f64;
        self.call_count += 1;
    }
    
    fn average_ms(&self) -> f32 {
        self.samples.average()
    }
    
    fn min_ms(&self) -> f32 {
        self.samples.min().unwrap_or(0.0)
    }
    
    fn max_ms(&self) -> f32 {
        self.samples.max().unwrap_or(0.0)
    }
}

impl PerformanceProfiler {
    /// 创建新的性能分析器
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
            current_section: None,
            section_start: Instant::now(),
            frame_start: Instant::now(),
            enabled: true,
        }
    }
    
    /// 启用/禁用分析器
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// 开始新帧
    pub fn start_frame(&mut self) {
        if !self.enabled {
            return;
        }
        
        self.frame_start = Instant::now();
        self.current_section = None;
    }
    
    /// 标记分段
    pub fn mark_section(&mut self, name: &str) {
        if !self.enabled {
            return;
        }
        
        // 结束上一个分段
        if let Some(prev_name) = &self.current_section {
            let elapsed = self.section_start.elapsed().as_secs_f32() * 1000.0;
            
            self.sections
                .entry(prev_name.clone())
                .or_insert_with(|| SectionMetrics::new(prev_name.clone()))
                .add_sample(elapsed);
        }
        
        // 开始新分段
        self.current_section = Some(name.to_string());
        self.section_start = Instant::now();
    }
    
    /// 结束帧
    pub fn end_frame(&mut self) {
        if !self.enabled {
            return;
        }
        
        // 结束当前分段
        if self.current_section.is_some() {
            self.mark_section("__end__");
        }
    }
    
    /// 获取分段统计
    pub fn get_section_stats(&self, name: &str) -> Option<SectionStats> {
        self.sections.get(name).map(|metrics| SectionStats {
            name: metrics.name.clone(),
            average_ms: metrics.average_ms(),
            min_ms: metrics.min_ms(),
            max_ms: metrics.max_ms(),
            call_count: metrics.call_count,
            total_time_ms: metrics.total_time_ms,
        })
    }
    
    /// 获取所有分段统计
    pub fn get_all_stats(&self) -> Vec<SectionStats> {
        let mut stats: Vec<_> = self.sections
            .values()
            .map(|metrics| SectionStats {
                name: metrics.name.clone(),
                average_ms: metrics.average_ms(),
                min_ms: metrics.min_ms(),
                max_ms: metrics.max_ms(),
                call_count: metrics.call_count,
                total_time_ms: metrics.total_time_ms,
            })
            .collect();
        
        // 按平均时间排序
        stats.sort_by(|a, b| b.average_ms.partial_cmp(&a.average_ms).unwrap());
        
        stats
    }
    
    /// 生成报告
    pub fn generate_report(&self) -> ProfileReport {
        let stats = self.get_all_stats();
        let total_time_ms: f32 = stats.iter().map(|s| s.average_ms).sum();
        
        ProfileReport {
            sections: stats,
            total_time_ms,
            frame_count: self.sections.values()
                .map(|m| m.call_count)
                .max()
                .unwrap_or(0),
        }
    }
    
    /// 重置统计
    pub fn reset(&mut self) {
        self.sections.clear();
        self.current_section = None;
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// 分段统计
#[derive(Debug, Clone)]
pub struct SectionStats {
    pub name: String,
    pub average_ms: f32,
    pub min_ms: f32,
    pub max_ms: f32,
    pub call_count: u64,
    pub total_time_ms: f64,
}

impl SectionStats {
    /// 获取占比
    pub fn percentage(&self, total_ms: f32) -> f32 {
        if total_ms > 0.0 {
            (self.average_ms / total_ms) * 100.0
        } else {
            0.0
        }
    }
}

/// 性能报告
#[derive(Debug, Clone)]
pub struct ProfileReport {
    pub sections: Vec<SectionStats>,
    pub total_time_ms: f32,
    pub frame_count: u64,
}

impl ProfileReport {
    /// 打印报告
    pub fn print(&self) {
        println!("\n=== 性能分析报告 ===");
        println!("总帧数: {}", self.frame_count);
        println!("总耗时: {:.2}ms", self.total_time_ms);
        println!("\n分段统计:");
        println!("{:<30} {:>10} {:>10} {:>10} {:>10} {:>10}", 
                 "分段", "平均(ms)", "最小(ms)", "最大(ms)", "占比(%)", "调用次数");
        println!("{}", "-".repeat(90));
        
        for stat in &self.sections {
            if stat.name == "__end__" {
                continue;
            }
            
            println!("{:<30} {:>10.2} {:>10.2} {:>10.2} {:>10.1} {:>10}",
                     stat.name,
                     stat.average_ms,
                     stat.min_ms,
                     stat.max_ms,
                     stat.percentage(self.total_time_ms),
                     stat.call_count);
        }
        println!();
    }
}

/// 性能监控器
pub struct PerformanceMonitor {
    metrics_history: RingBuffer<DetailedMetrics>,
    profiler: PerformanceProfiler,
    frame_number: u64,
    last_update: Instant,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            metrics_history: RingBuffer::new(300),
            profiler: PerformanceProfiler::new(),
            frame_number: 0,
            last_update: Instant::now(),
        }
    }
    
    /// 更新性能数据
    pub fn update(&mut self, frame_time_ms: f32) {
        self.frame_number += 1;
        
        let mut metrics = DetailedMetrics::new(frame_time_ms, self.frame_number);
        
        // 尝试获取系统指标
        self.collect_system_metrics(&mut metrics);
        
        self.metrics_history.push(metrics);
        self.last_update = Instant::now();
    }
    
    /// 收集系统指标
    fn collect_system_metrics(&self, _metrics: &mut DetailedMetrics) {
        // 简化实现，实际应用可以集成系统监控库
        // 例如：sysinfo, sys-info等
    }
    
    /// 获取平均FPS
    pub fn average_fps(&self) -> f32 {
        if self.metrics_history.is_empty() {
            return 0.0;
        }
        
        let sum: f32 = self.metrics_history.iter().map(|m| m.fps).sum();
        sum / self.metrics_history.len() as f32
    }
    
    /// 获取平均帧时间
    pub fn average_frame_time(&self) -> f32 {
        if self.metrics_history.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.metrics_history.iter().map(|m| m.frame_time_ms).sum();
        sum / self.metrics_history.len() as f32
    }
    
    /// 获取最新指标
    pub fn latest_metrics(&self) -> Option<&DetailedMetrics> {
        self.metrics_history.last()
    }
    
    /// 获取性能分析器
    pub fn profiler(&mut self) -> &mut PerformanceProfiler {
        &mut self.profiler
    }
    
    /// 生成性能摘要
    pub fn generate_summary(&self) -> PerformanceSummary {
        let avg_fps = self.average_fps();
        let avg_frame_time = self.average_frame_time();
        
        let min_fps = self.metrics_history.iter()
            .map(|m| m.fps)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        let max_fps = self.metrics_history.iter()
            .map(|m| m.fps)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        PerformanceSummary {
            average_fps: avg_fps,
            min_fps,
            max_fps,
            average_frame_time_ms: avg_frame_time,
            frame_count: self.frame_number,
            monitoring_duration: self.last_update.elapsed(),
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能摘要
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub average_fps: f32,
    pub min_fps: f32,
    pub max_fps: f32,
    pub average_frame_time_ms: f32,
    pub frame_count: u64,
    pub monitoring_duration: Duration,
}

impl PerformanceSummary {
    /// 打印摘要
    pub fn print(&self) {
        println!("\n=== 性能摘要 ===");
        println!("总帧数: {}", self.frame_count);
        println!("监控时长: {:.1}秒", self.monitoring_duration.as_secs_f32());
        println!("平均FPS: {:.1}", self.average_fps);
        println!("最低FPS: {:.1}", self.min_fps);
        println!("最高FPS: {:.1}", self.max_fps);
        println!("平均帧时间: {:.2}ms", self.average_frame_time_ms);
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        
        // 模拟一些帧
        for i in 0..100 {
            let frame_time = 16.67 + (i as f32 * 0.1);
            monitor.update(frame_time);
        }
        
        let summary = monitor.generate_summary();
        summary.print();
        
        assert!(summary.average_fps > 0.0);
        assert_eq!(summary.frame_count, 100);
    }
    
    #[test]
    fn test_performance_profiler() {
        let mut profiler = PerformanceProfiler::new();
        
        // 模拟几帧
        for _ in 0..10 {
            profiler.start_frame();
            
            profiler.mark_section("update");
            std::thread::sleep(Duration::from_millis(5));
            
            profiler.mark_section("render");
            std::thread::sleep(Duration::from_millis(10));
            
            profiler.mark_section("present");
            std::thread::sleep(Duration::from_millis(2));
            
            profiler.end_frame();
        }
        
        let report = profiler.generate_report();
        report.print();
        
        assert!(!report.sections.is_empty());
    }
}
