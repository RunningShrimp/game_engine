//! # Performance Profiler
//!
//! 自动性能分析工具 - 检测游戏引擎中的性能瓶颈。
//!
//! ## 核心组件
//!
//! 1. **PerformanceProfiler** - 主分析器
//! 2. **MetricsCollector** - 指标收集器
//! 3. **BottleneckDetector** - 瓶颈检测器
//! 4. **OptimizationSuggester** - 优化建议生成器
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use game_engine::performance::profiler::*;
//!
//! let profiler = PerformanceProfiler::new();
//! profiler.start_session();
//!
//! // ... 运行游戏 ...
//!
//! let report = profiler.generate_report();
//! println!("Bottlenecks: {:?}", report.bottlenecks);
//! ```

use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

/// 性能分析错误
#[derive(Debug, Error)]
pub enum ProfilerError {
    #[error("Collection error: {0}")]
    CollectionError(String),

    #[error("Analysis error: {0}")]
    AnalysisError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ==================== 性能指标 ====================

/// 性能指标
#[derive(Clone, Debug)]
pub struct PerformanceMetrics {
    /// 帧时间
    pub frame_time: Duration,
    /// FPS
    pub fps: f32,
    /// CPU使用率（0.0-1.0）
    pub cpu_usage: f32,
    /// 内存使用量（字节）
    pub memory_usage: u64,
    /// GPU使用率（0.0-1.0）
    pub gpu_usage: f32,
    /// Draw Call数量
    pub draw_calls: u32,
    /// 三角形数量
    pub triangle_count: u32,
    /// 纹理数量
    pub texture_count: u32,
    /// 着色器切换次数
    pub shader_switches: u32,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            frame_time: Duration::from_millis(16),
            fps: 60.0,
            cpu_usage: 0.0,
            memory_usage: 0,
            gpu_usage: 0.0,
            draw_calls: 0,
            triangle_count: 0,
            texture_count: 0,
            shader_switches: 0,
        }
    }
}

// ==================== 性能类别 ====================

/// 性能类别
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerformanceCategory {
    /// 渲染相关
    Rendering,
    /// CPU相关
    Cpu,
    /// 内存相关
    Memory,
    /// IO相关
    Io,
    /// 网络
    Network,
    /// 脚本
    Scripting,
    /// 物理
    Physics,
    /// 音频
    Audio,
    /// 帧时间
    FrameTime,
}

/// 瓶颈严重程度
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// 低 - 可忽略
    Low,
    /// 中 - 值得关注
    Medium,
    /// 高 - 需要优化
    High,
    /// 严重 - 必须立即优化
    Critical,
}

// ==================== 瓶颈检测 ====================

/// 性能瓶颈
#[derive(Clone, Debug)]
pub struct Bottleneck {
    /// 类别
    pub category: PerformanceCategory,
    /// 严重程度
    pub severity: Severity,
    /// 描述
    pub description: String,
    /// 当前值
    pub current_value: f64,
    /// 目标值
    pub target_value: f64,
    /// 影响分析
    pub impact: String,
    /// 检测时间戳
    pub timestamp: Instant,
}

impl Bottleneck {
    /// 创建新的瓶颈
    pub fn new(
        category: PerformanceCategory,
        severity: Severity,
        description: String,
        current_value: f64,
        target_value: f64,
        impact: String,
    ) -> Self {
        Self {
            category,
            severity,
            description,
            current_value,
            target_value,
            impact,
            timestamp: Instant::now(),
        }
    }

    /// 计算偏差百分比
    pub fn deviation_percent(&self) -> f64 {
        if self.target_value == 0.0 {
            0.0
        } else {
            ((self.current_value - self.target_value) / self.target_value) * 100.0
        }
    }
}

// ==================== 指标收集器 ====================

/// 指标收集器
pub struct MetricsCollector {
    /// 历史指标数据
    history: Vec<PerformanceMetrics>,
    /// 最大历史记录数
    max_history: usize,
    /// 当前帧的指标
    current: PerformanceMetrics,
    /// 帧计数
    frame_count: u64,
}

impl MetricsCollector {
    /// 创建新的收集器
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Vec::with_capacity(max_history),
            max_history,
            current: PerformanceMetrics::default(),
            frame_count: 0,
        }
    }

    /// 开始新帧
    pub fn begin_frame(&mut self) {
        self.current = PerformanceMetrics::default();
    }

    /// 结束帧
    pub fn end_frame(&mut self) {
        self.frame_count += 1;
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(self.current.clone());
    }

    /// 记录帧时间
    pub fn record_frame_time(&mut self, frame_time: Duration) {
        self.current.frame_time = frame_time;
        self.current.fps = 1.0 / frame_time.as_secs_f32();
    }

    /// 记录CPU使用率
    pub fn record_cpu_usage(&mut self, usage: f32) {
        self.current.cpu_usage = usage.clamp(0.0, 1.0);
    }

    /// 记录内存使用
    pub fn record_memory_usage(&mut self, bytes: u64) {
        self.current.memory_usage = bytes;
    }

    /// 记录GPU使用率
    pub fn record_gpu_usage(&mut self, usage: f32) {
        self.current.gpu_usage = usage.clamp(0.0, 1.0);
    }

    /// 记录Draw Calls
    pub fn record_draw_calls(&mut self, count: u32) {
        self.current.draw_calls = count;
    }

    /// 记录三角形数量
    pub fn record_triangle_count(&mut self, count: u32) {
        self.current.triangle_count = count;
    }

    /// 记录纹理数量
    pub fn record_texture_count(&mut self, count: u32) {
        self.current.texture_count = count;
    }

    /// 记录着色器切换
    pub fn record_shader_switches(&mut self, count: u32) {
        self.current.shader_switches = count;
    }

    /// 获取平均FPS
    pub fn average_fps(&self) -> f32 {
        if self.history.is_empty() {
            return self.current.fps;
        }

        let sum: f32 = self.history.iter().map(|m| m.fps).sum();

        sum / self.history.len() as f32
    }

    /// 获取平均帧时间
    pub fn average_frame_time(&self) -> Duration {
        if self.history.is_empty() {
            return self.current.frame_time;
        }

        let sum_ms: f64 = self.history.iter().map(|m| m.frame_time.as_secs_f64() * 1000.0).sum();

        Duration::from_secs_f64((sum_ms / self.history.len() as f64) / 1000.0)
    }

    /// 获取最新指标
    pub fn latest(&self) -> &PerformanceMetrics {
        &self.current
    }

    /// 获取所有历史指标
    pub fn history(&self) -> &[PerformanceMetrics] {
        &self.history
    }
}

// ==================== 瓶颈检测器 ====================

/// 瓶颈检测器
pub struct BottleneckDetector {
    /// 阈值配置
    thresholds: ThresholdConfig,
}

/// 阈值配置
#[derive(Clone, Debug)]
pub struct ThresholdConfig {
    /// 目标帧时间（毫秒）
    pub target_frame_time_ms: f64,
    /// 最大可接受帧时间（毫秒）
    pub max_frame_time_ms: f64,
    /// 最大Draw Calls
    pub max_draw_calls: u32,
    /// 最大三角形数量
    pub max_triangles: u32,
    /// 最大纹理数量
    pub max_textures: u32,
    /// 最大着色器切换
    pub max_shader_switches: u32,
    /// 最大CPU使用率（0.0-1.0）
    pub max_cpu_usage: f32,
    /// 最大GPU使用率（0.0-1.0）
    pub max_gpu_usage: f32,
    /// 最大内存使用量（字节）
    pub max_memory_usage: u64,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            target_frame_time_ms: 16.67, // 60 FPS
            max_frame_time_ms: 33.33,    // 30 FPS
            max_draw_calls: 1000,
            max_triangles: 1_000_000,
            max_textures: 512,
            max_shader_switches: 100,
            max_cpu_usage: 0.9,
            max_gpu_usage: 0.95,
            max_memory_usage: 2_000_000_000, // 2GB
        }
    }
}

impl BottleneckDetector {
    /// 创建新的检测器
    pub fn new(thresholds: ThresholdConfig) -> Self {
        Self { thresholds }
    }

    /// 检测所有瓶颈
    pub fn detect_bottlenecks(&self, metrics: &PerformanceMetrics) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        // 检测帧时间
        if let Some(bottleneck) = self.check_frame_time(metrics) {
            bottlenecks.push(bottleneck);
        }

        // 检测Draw Calls
        if let Some(bottleneck) = self.check_draw_calls(metrics) {
            bottlenecks.push(bottleneck);
        }

        // 检测三角形数量
        if let Some(bottleneck) = self.check_triangle_count(metrics) {
            bottlenecks.push(bottleneck);
        }

        // 检测纹理数量
        if let Some(bottleneck) = self.check_texture_count(metrics) {
            bottlenecks.push(bottleneck);
        }

        // 检测着色器切换
        if let Some(bottleneck) = self.check_shader_switches(metrics) {
            bottlenecks.push(bottleneck);
        }

        // 检测CPU使用率
        if let Some(bottleneck) = self.check_cpu_usage(metrics) {
            bottlenecks.push(bottleneck);
        }

        // 检测GPU使用率
        if let Some(bottleneck) = self.check_gpu_usage(metrics) {
            bottlenecks.push(bottleneck);
        }

        // 检测内存使用
        if let Some(bottleneck) = self.check_memory_usage(metrics) {
            bottlenecks.push(bottleneck);
        }

        bottlenecks
    }

    /// 检查帧时间
    fn check_frame_time(&self, metrics: &PerformanceMetrics) -> Option<Bottleneck> {
        let frame_time_ms = metrics.frame_time.as_secs_f64() * 1000.0;

        if frame_time_ms > self.thresholds.max_frame_time_ms {
            let severity = if frame_time_ms > self.thresholds.max_frame_time_ms * 2.0 {
                Severity::Critical
            } else {
                Severity::High
            };

            Some(Bottleneck::new(
                PerformanceCategory::Rendering,
                severity,
                format!("Frame time too high: {frame_time_ms:.2}ms"),
                frame_time_ms,
                self.thresholds.target_frame_time_ms,
                "Frame drops will be noticeable to players".to_string(),
            ))
        } else {
            None
        }
    }

    /// 检查Draw Calls
    fn check_draw_calls(&self, metrics: &PerformanceMetrics) -> Option<Bottleneck> {
        if metrics.draw_calls > self.thresholds.max_draw_calls {
            let ratio = metrics.draw_calls as f64 / self.thresholds.max_draw_calls as f64;
            let severity = if ratio > 2.0 {
                Severity::Critical
            } else if ratio > 1.5 {
                Severity::High
            } else {
                Severity::Medium
            };

            Some(Bottleneck::new(
                PerformanceCategory::Rendering,
                severity,
                format!("Too many draw calls: {}", metrics.draw_calls),
                metrics.draw_calls as f64,
                self.thresholds.max_draw_calls as f64,
                "Consider batching draw calls or using instancing".to_string(),
            ))
        } else {
            None
        }
    }

    /// 检查三角形数量
    fn check_triangle_count(&self, metrics: &PerformanceMetrics) -> Option<Bottleneck> {
        if metrics.triangle_count > self.thresholds.max_triangles {
            Some(Bottleneck::new(
                PerformanceCategory::Rendering,
                Severity::Medium,
                format!("High triangle count: {}", metrics.triangle_count),
                metrics.triangle_count as f64,
                self.thresholds.max_triangles as f64,
                "Consider using LOD or reducing mesh complexity".to_string(),
            ))
        } else {
            None
        }
    }

    /// 检查纹理数量
    fn check_texture_count(&self, metrics: &PerformanceMetrics) -> Option<Bottleneck> {
        if metrics.texture_count > self.thresholds.max_textures {
            Some(Bottleneck::new(
                PerformanceCategory::Rendering,
                Severity::Medium,
                format!("Too many textures: {}", metrics.texture_count),
                metrics.texture_count as f64,
                self.thresholds.max_textures as f64,
                "Consider using texture atlases or reducing texture count".to_string(),
            ))
        } else {
            None
        }
    }

    /// 检查着色器切换
    fn check_shader_switches(&self, metrics: &PerformanceMetrics) -> Option<Bottleneck> {
        if metrics.shader_switches > self.thresholds.max_shader_switches {
            Some(Bottleneck::new(
                PerformanceCategory::Rendering,
                Severity::Medium,
                format!("Too many shader switches: {}", metrics.shader_switches),
                metrics.shader_switches as f64,
                self.thresholds.max_shader_switches as f64,
                "Consider sorting by shader to reduce state changes".to_string(),
            ))
        } else {
            None
        }
    }

    /// 检查CPU使用率
    fn check_cpu_usage(&self, metrics: &PerformanceMetrics) -> Option<Bottleneck> {
        if metrics.cpu_usage > self.thresholds.max_cpu_usage {
            Some(Bottleneck::new(
                PerformanceCategory::Cpu,
                Severity::High,
                format!("High CPU usage: {:.1}%", metrics.cpu_usage * 100.0),
                metrics.cpu_usage as f64,
                self.thresholds.max_cpu_usage as f64,
                "CPU is a limiting factor for performance".to_string(),
            ))
        } else {
            None
        }
    }

    /// 检查GPU使用率
    fn check_gpu_usage(&self, metrics: &PerformanceMetrics) -> Option<Bottleneck> {
        if metrics.gpu_usage > self.thresholds.max_gpu_usage {
            Some(Bottleneck::new(
                PerformanceCategory::Rendering,
                Severity::High,
                format!("High GPU usage: {:.1}%", metrics.gpu_usage * 100.0),
                metrics.gpu_usage as f64,
                self.thresholds.max_gpu_usage as f64,
                "GPU is a limiting factor for performance".to_string(),
            ))
        } else {
            None
        }
    }

    /// 检查内存使用
    fn check_memory_usage(&self, metrics: &PerformanceMetrics) -> Option<Bottleneck> {
        if metrics.memory_usage > self.thresholds.max_memory_usage {
            let ratio = metrics.memory_usage as f64 / self.thresholds.max_memory_usage as f64;
            let severity = if ratio > 2.0 {
                Severity::Critical
            } else if ratio > 1.5 {
                Severity::High
            } else {
                Severity::Medium
            };

            let mb_used = metrics.memory_usage / 1_000_000;
            let mb_max = self.thresholds.max_memory_usage / 1_000_000;

            Some(Bottleneck::new(
                PerformanceCategory::Memory,
                severity,
                format!("High memory usage: {mb_used}MB / {mb_max}MB"),
                metrics.memory_usage as f64,
                self.thresholds.max_memory_usage as f64,
                "May cause issues on low-memory devices".to_string(),
            ))
        } else {
            None
        }
    }
}

// ==================== 性能分析器 ====================

/// 性能分析器
pub struct PerformanceProfiler {
    /// 指标收集器
    collector: MetricsCollector,
    /// 瓶颈检测器
    detector: BottleneckDetector,
    /// 是否正在记录
    is_recording: bool,
    /// 会话开始时间
    session_start: Option<Instant>,
}

impl PerformanceProfiler {
    /// 创建新的性能分析器
    pub fn new() -> Self {
        Self {
            collector: MetricsCollector::new(1000), // 保存1000帧数据
            detector: BottleneckDetector::new(ThresholdConfig::default()),
            is_recording: false,
            session_start: None,
        }
    }

    /// 使用自定义阈值创建分析器
    pub fn with_thresholds(thresholds: ThresholdConfig) -> Self {
        Self {
            collector: MetricsCollector::new(1000),
            detector: BottleneckDetector::new(thresholds),
            is_recording: false,
            session_start: None,
        }
    }

    /// 开始分析会话
    pub fn start_session(&mut self) {
        self.is_recording = true;
        self.session_start = Some(Instant::now());
        self.collector.history.clear();
    }

    /// 结束分析会话
    pub fn end_session(&mut self) -> Duration {
        self.is_recording = false;
        self.session_start.map(|start| start.elapsed()).unwrap_or(Duration::ZERO)
    }

    /// 开始新帧
    pub fn begin_frame(&mut self) {
        if self.is_recording {
            self.collector.begin_frame();
        }
    }

    /// 结束帧
    pub fn end_frame(&mut self, frame_time: Duration) {
        if self.is_recording {
            self.collector.record_frame_time(frame_time);
            self.collector.end_frame();
        }
    }

    /// 记录CPU使用率
    pub fn record_cpu_usage(&mut self, usage: f32) {
        if self.is_recording {
            self.collector.record_cpu_usage(usage);
        }
    }

    /// 记录内存使用
    pub fn record_memory_usage(&mut self, bytes: u64) {
        if self.is_recording {
            self.collector.record_memory_usage(bytes);
        }
    }

    /// 记录GPU使用率
    pub fn record_gpu_usage(&mut self, usage: f32) {
        if self.is_recording {
            self.collector.record_gpu_usage(usage);
        }
    }

    /// 记录渲染统计
    pub fn record_render_stats(&mut self, draw_calls: u32, triangles: u32) {
        if self.is_recording {
            self.collector.record_draw_calls(draw_calls);
            self.collector.record_triangle_count(triangles);
        }
    }

    /// 记录纹理统计
    pub fn record_texture_stats(&mut self, count: u32, shader_switches: u32) {
        if self.is_recording {
            self.collector.record_texture_count(count);
            self.collector.record_shader_switches(shader_switches);
        }
    }

    /// 检测当前帧的瓶颈
    pub fn detect_bottlenecks(&self) -> Vec<Bottleneck> {
        if let Some(latest) = self.collector.history.last() {
            self.detector.detect_bottlenecks(latest)
        } else {
            Vec::new()
        }
    }

    /// 检测所有历史帧的瓶颈
    pub fn detect_all_bottlenecks(&self) -> Vec<Bottleneck> {
        let mut all_bottlenecks = Vec::new();

        for metrics in &self.collector.history {
            all_bottlenecks.extend(self.detector.detect_bottlenecks(metrics));
        }

        all_bottlenecks
    }

    /// 获取收集器引用
    pub fn collector(&self) -> &MetricsCollector {
        &self.collector
    }

    /// 获取是否正在记录
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new(10);
        collector.begin_frame();
        collector.record_frame_time(Duration::from_millis(20));
        collector.record_draw_calls(100);
        collector.end_frame();

        assert_eq!(collector.history().len(), 1);
        assert!((collector.latest().fps - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_bottleneck_detection() {
        let detector = BottleneckDetector::new(ThresholdConfig::default());

        let metrics = PerformanceMetrics {
            frame_time: Duration::from_millis(50), // 太高
            draw_calls: 2000,                      // 太多
            ..Default::default()
        };

        let bottlenecks = detector.detect_bottlenecks(&metrics);
        assert!(!bottlenecks.is_empty());
    }

    #[test]
    fn test_profiler_session() {
        let mut profiler = PerformanceProfiler::new();
        profiler.start_session();
        assert!(profiler.is_recording());

        profiler.begin_frame();
        profiler.end_frame(Duration::from_millis(16));

        profiler.end_session();
        assert!(!profiler.is_recording());
    }
}
