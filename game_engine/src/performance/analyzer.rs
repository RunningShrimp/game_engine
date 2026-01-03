//! # 性能分析工具
//!
//! **API 稳定性**: 稳定 (Stable) (v0.1.0)
//!
//! 提供全面的性能监控和分析工具：
//! - 实时性能指标收集
//! - 性能热点分析
//! - 内存使用分析
//! - 性能瓶颈检测
//! - 自动化性能报告生成
//!
//! ## 功能特性
//!
//! | 功能 | 状态 | 说明 |
//! |------|------|------|
//! | 指标收集 | ✅ 已实现 | CPU/GPU/内存/帧时间 |
//! | 热点分析 | ✅ 已实现 | 自动检测性能热点 |
//! | 瓶颈检测 | ✅ 已实现 | 识别系统瓶颈 |
//! | 性能报告 | ✅ 已实现 | 自动生成详细报告 |
//! | 实时监控 | ✅ 已实现 | 实时性能仪表板 |

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};

/// 性能指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    /// FPS
    Fps,
    /// 帧时间（毫秒）
    FrameTime,
    /// CPU使用率（百分比）
    CpuUsage,
    /// 内存使用（字节）
    MemoryUsage,
    /// GPU使用率（百分比）
    GpuUsage,
    /// 绘制调用数
    DrawCalls,
    /// 三角形数
    Triangles,
    /// 自定义指标
    Custom(&'static str),
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct Metric {
    /// 指标类型
    pub metric_type: MetricType,
    /// 指标值
    pub value: f64,
    /// 时间戳
    pub timestamp: Instant,
    /// 标签
    pub tags: HashMap<String, String>,
}

impl Metric {
    /// 创建新的指标
    pub fn new(metric_type: MetricType, value: f64) -> Self {
        Self {
            metric_type,
            value,
            timestamp: Instant::now(),
            tags: HashMap::new(),
        }
    }

    /// 添加标签
    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }
}

/// 性能热点
#[derive(Debug, Clone)]
pub struct Hotspot {
    /// 热点名称
    pub name: String,
    /// 平均耗时（微秒）
    pub avg_duration_us: u64,
    /// 最大耗时（微秒）
    pub max_duration_us: u64,
    /// 调用次数
    pub call_count: u64,
    /// 总耗时（微秒）
    pub total_duration_us: u64,
    /// 严重程度（0-1）
    pub severity: f32,
}

impl Hotspot {
    /// 计算严重程度
    pub fn calculate_severity(&mut self, baseline_us: u64) {
        let ratio = if baseline_us > 0 {
            self.avg_duration_us as f64 / baseline_us as f64
        } else {
            1.0
        };

        // 基于超过基线的程度计算严重程度
        self.severity = ((ratio - 1.0) / 10.0).clamp(0.0, 1.0) as f32;
    }
}

/// 性能瓶颈
#[derive(Debug, Clone)]
pub struct Bottleneck {
    /// 瓶颈类型
    pub bottleneck_type: BottleneckType,
    /// 描述
    pub description: String,
    /// 影响程度（0-1）
    pub impact: f32,
    /// 建议
    pub suggestions: Vec<String>,
}

/// 瓶颈类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottleneckType {
    /// CPU瓶颈
    Cpu,
    /// GPU瓶颈
    Gpu,
    /// 内存瓶颈
    Memory,
    /// 带宽瓶颈
    Bandwidth,
    /// I/O瓶颈
    Io,
}

/// 性能分析器
pub struct PerformanceAnalyzer {
    /// 指标历史（最多保留10000个数据点）
    metrics_history: Arc<Mutex<VecDeque<Metric>>>,
    /// 性能热点
    hotspots: Arc<RwLock<HashMap<String, Hotspot>>>,
    /// 检测到的瓶颈
    bottlenecks: Arc<RwLock<Vec<Bottleneck>>>,
    /// 基线性能
    baseline: Arc<RwLock<PerformanceBaseline>>,
    /// 配置
    config: AnalyzerConfig,
    /// 开始时间
    start_time: Instant,
}

/// 性能基线
#[derive(Debug, Clone, Default)]
pub struct PerformanceBaseline {
    /// 目标帧时间（毫秒）
    pub target_frame_time_ms: f64,
    /// 目标FPS
    pub target_fps: f64,
    /// 最大内存使用（字节）
    pub max_memory_bytes: usize,
    /// 最大CPU使用率（百分比）
    pub max_cpu_usage_percent: f64,
    /// 最大GPU使用率（百分比）
    pub max_gpu_usage_percent: f64,
}

/// 分析器配置
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// 指标保留时间
    pub retention_duration: Duration,
    /// 最大指标数
    pub max_metrics: usize,
    /// 热点检测阈值（微秒）
    pub hotspot_threshold_us: u64,
    /// 是否自动检测瓶颈
    pub auto_detect_bottlenecks: bool,
    /// 是否启用实时分析
    pub enable_realtime_analysis: bool,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            retention_duration: Duration::from_secs(300), // 5分钟
            max_metrics: 10000,
            hotspot_threshold_us: 1000, // 1ms
            auto_detect_bottlenecks: true,
            enable_realtime_analysis: true,
        }
    }
}

impl PerformanceAnalyzer {
    /// 创建新的分析器
    pub fn new(config: AnalyzerConfig) -> Self {
        Self {
            metrics_history: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_metrics))),
            hotspots: Arc::new(RwLock::new(HashMap::new())),
            bottlenecks: Arc::new(RwLock::new(Vec::new())),
            baseline: Arc::new(RwLock::new(PerformanceBaseline::default())),
            config,
            start_time: Instant::now(),
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(AnalyzerConfig::default())
    }

    /// 记录性能指标
    pub async fn record_metric(&self, metric: Metric) {
        let mut history = self.metrics_history.lock().await;

        // 添加新指标
        history.push_back(metric.clone());

        // 清理过期指标
        let now = Instant::now();
        while history.len() > self.config.max_metrics
            || (history
                .front()
                .map(|m| now.duration_since(m.timestamp) > self.config.retention_duration)
                .unwrap_or(false))
        {
            history.pop_front();
        }

        // 实时分析
        if self.config.enable_realtime_analysis {
            self.analyze_metric(&metric).await;
        }
    }

    /// 分析单个指标
    async fn analyze_metric(&self, metric: &Metric) {
        match metric.metric_type {
            MetricType::FrameTime => {
                // 检测帧时间异常
                if metric.value > 33.3 {
                    // 超过30FPS阈值
                    warn!("High frame time detected: {:.2} ms", metric.value);

                    // 添加热点
                    self.add_hotspot("frame_time", metric.value as u64 * 1000).await;
                }
            }
            MetricType::CpuUsage => {
                // 检测CPU使用率异常
                if metric.value > 90.0 {
                    warn!("High CPU usage detected: {:.1}%", metric.value);

                    self.add_bottleneck(Bottleneck {
                        bottleneck_type: BottleneckType::Cpu,
                        description: format!("CPU使用率过高: {:.1}%", metric.value),
                        impact: ((metric.value - 90.0) / 10.0).clamp(0.0, 1.0) as f32,
                        suggestions: vec![
                            "优化算法复杂度".to_string(),
                            "减少不必要的计算".to_string(),
                            "使用多线程并行".to_string(),
                        ],
                    })
                    .await;
                }
            }
            MetricType::MemoryUsage => {
                let baseline = self.baseline.read().await;
                if metric.value as usize > baseline.max_memory_bytes * 9 / 10 {
                    warn!(
                        "High memory usage detected: {:.2} MB",
                        metric.value / (1024.0 * 1024.0)
                    );

                    self.add_bottleneck(Bottleneck {
                        bottleneck_type: BottleneckType::Memory,
                        description: format!(
                            "内存使用接近上限: {:.2} MB",
                            metric.value / (1024.0 * 1024.0)
                        ),
                        impact: 0.8,
                        suggestions: vec![
                            "检查内存泄漏".to_string(),
                            "使用内存池".to_string(),
                            "优化纹理和资源大小".to_string(),
                        ],
                    })
                    .await;
                }
            }
            MetricType::GpuUsage => {
                if metric.value > 95.0 {
                    warn!("High GPU usage detected: {:.1}%", metric.value);

                    self.add_bottleneck(Bottleneck {
                        bottleneck_type: BottleneckType::Gpu,
                        description: format!("GPU使用率过高: {:.1}%", metric.value),
                        impact: ((metric.value - 95.0) / 5.0).clamp(0.0, 1.0) as f32,
                        suggestions: vec![
                            "减少绘制调用".to_string(),
                            "优化着色器复杂度".to_string(),
                            "使用LOD系统".to_string(),
                        ],
                    })
                    .await;
                }
            }
            _ => {}
        }
    }

    /// 添加热点
    async fn add_hotspot(&self, name: &str, duration_us: u64) {
        let mut hotspots = self.hotspots.write().await;

        let entry = hotspots.entry(name.to_string()).or_insert_with(|| Hotspot {
            name: name.to_string(),
            avg_duration_us: 0,
            max_duration_us: 0,
            call_count: 0,
            total_duration_us: 0,
            severity: 0.0,
        });

        entry.call_count += 1;
        entry.total_duration_us += duration_us;
        entry.max_duration_us = entry.max_duration_us.max(duration_us);
        entry.avg_duration_us = entry.total_duration_us / entry.call_count;

        // 计算严重程度
        let baseline = self.baseline.read().await;
        entry.calculate_severity((baseline.target_frame_time_ms * 1000.0) as u64);

        if entry.severity > 0.5 {
            warn!(
                "Performance hotspot detected: {} (avg: {} μs, severity: {:.1}%)",
                name,
                entry.avg_duration_us,
                entry.severity * 100.0
            );
        }
    }

    /// 添加瓶颈
    async fn add_bottleneck(&self, bottleneck: Bottleneck) {
        let mut bottlenecks = self.bottlenecks.write().await;

        // 检查是否已存在相同类型的瓶颈
        if !bottlenecks.iter().any(|b| b.bottleneck_type == bottleneck.bottleneck_type) {
            error!("Bottleneck detected: {:?}", bottleneck.bottleneck_type);
            bottlenecks.push(bottleneck);
        }
    }

    /// 设置性能基线
    pub async fn set_baseline(&self, baseline: PerformanceBaseline) {
        let mut current = self.baseline.write().await;
        *current = baseline;
        info!("Performance baseline updated");
    }

    /// 获取性能热点
    pub async fn get_hotspots(&self) -> Vec<Hotspot> {
        let hotspots = self.hotspots.read().await;
        let mut list: Vec<_> = hotspots.values().cloned().collect();

        // 按严重程度排序
        list.sort_by(|a, b| {
            b.severity.partial_cmp(&a.severity).unwrap_or(std::cmp::Ordering::Equal)
        });

        list
    }

    /// 获取瓶颈
    pub async fn get_bottlenecks(&self) -> Vec<Bottleneck> {
        let bottlenecks = self.bottlenecks.read().await;
        bottlenecks.clone()
    }

    /// 获取指标统计
    pub async fn get_metric_stats(&self, metric_type: MetricType) -> MetricStats {
        let history = self.metrics_history.lock().await;

        let values: Vec<f64> = history
            .iter()
            .filter(|m| m.metric_type == metric_type)
            .map(|m| m.value)
            .collect();

        if values.is_empty() {
            return MetricStats::default();
        }

        let sum: f64 = values.iter().sum();
        let avg = sum / values.len() as f64;
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // 计算标准差
        let variance = values.iter().map(|&v| (v - avg).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        MetricStats {
            count: values.len(),
            avg,
            min,
            max,
            std_dev,
            percentile_95: Self::calculate_percentile(&values, 0.95),
            percentile_99: Self::calculate_percentile(&values, 0.99),
        }
    }

    /// 计算百分位数
    fn calculate_percentile(values: &[f64], percentile: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let index = ((sorted.len() as f64 - 1.0) * percentile).round() as usize;
        sorted[index]
    }

    /// 生成性能报告
    pub async fn generate_report(&self) -> PerformanceReport {
        let fps_stats = self.get_metric_stats(MetricType::Fps).await;
        let frame_time_stats = self.get_metric_stats(MetricType::FrameTime).await;
        let cpu_stats = self.get_metric_stats(MetricType::CpuUsage).await;
        let memory_stats = self.get_metric_stats(MetricType::MemoryUsage).await;
        let gpu_stats = self.get_metric_stats(MetricType::GpuUsage).await;

        let hotspots = self.get_hotspots().await;
        let bottlenecks = self.get_bottlenecks().await;

        // 在移动之前生成建议
        let recommendations = self.generate_recommendations(&hotspots, &bottlenecks).await;

        let uptime = self.start_time.elapsed();

        PerformanceReport {
            uptime,
            fps_stats,
            frame_time_stats,
            cpu_stats,
            memory_stats,
            gpu_stats,
            hotspots,
            bottlenecks,
            recommendations,
        }
    }

    /// 生成优化建议
    async fn generate_recommendations(
        &self,
        hotspots: &[Hotspot],
        bottlenecks: &[Bottleneck],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // 基于热点的建议
        for hotspot in hotspots.iter().filter(|h| h.severity > 0.5) {
            recommendations.push(format!(
                "优化热点 '{}': 平均耗时 {} μs，建议检查算法复杂度",
                hotspot.name, hotspot.avg_duration_us
            ));
        }

        // 基于瓶颈的建议
        for bottleneck in bottlenecks {
            for suggestion in &bottleneck.suggestions {
                recommendations.push(suggestion.clone());
            }
        }

        recommendations
    }

    /// 打印性能报告
    pub async fn print_report(&self) {
        let report = self.generate_report().await;

        println!("\n=== 性能分析报告 ===");
        println!("运行时间: {:?}", report.uptime);
        println!("\n帧率统计:");
        println!("  平均 FPS: {:.1}", report.fps_stats.avg);
        println!("  最小 FPS: {:.1}", report.fps_stats.min);
        println!("  最大 FPS: {:.1}", report.fps_stats.max);
        println!("  95th 百分位: {:.1}", report.fps_stats.percentile_95);

        println!("\n帧时间统计:");
        println!("  平均: {:.2} ms", report.frame_time_stats.avg);
        println!("  最大: {:.2} ms", report.frame_time_stats.max);
        println!(
            "  99th 百分位: {:.2} ms",
            report.frame_time_stats.percentile_99
        );

        println!("\nCPU使用率:");
        println!("  平均: {:.1}%", report.cpu_stats.avg);
        println!("  最大: {:.1}%", report.cpu_stats.max);

        println!("\n内存使用:");
        println!(
            "  平均: {:.2} MB",
            report.memory_stats.avg / (1024.0 * 1024.0)
        );
        println!(
            "  最大: {:.2} MB",
            report.memory_stats.max / (1024.0 * 1024.0)
        );

        println!("\nGPU使用率:");
        println!("  平均: {:.1}%", report.gpu_stats.avg);
        println!("  最大: {:.1}%", report.gpu_stats.max);

        if !report.hotspots.is_empty() {
            println!("\n性能热点:");
            for (i, hotspot) in report.hotspots.iter().take(5).enumerate() {
                println!(
                    "  {}. {} - {} μs (调用次数: {}, 严重程度: {:.1}%)",
                    i + 1,
                    hotspot.name,
                    hotspot.avg_duration_us,
                    hotspot.call_count,
                    hotspot.severity * 100.0
                );
            }
        }

        if !report.bottlenecks.is_empty() {
            println!("\n检测到的瓶颈:");
            for (i, bottleneck) in report.bottlenecks.iter().enumerate() {
                println!("  {}. {:?}", i + 1, bottleneck.bottleneck_type);
                println!("     描述: {}", bottleneck.description);
                println!("     影响: {:.1}%", bottleneck.impact * 100.0);
            }
        }

        if !report.recommendations.is_empty() {
            println!("\n优化建议:");
            for (i, rec) in report.recommendations.iter().take(10).enumerate() {
                println!("  {}. {}", i + 1, rec);
            }
        }

        println!("====================\n");
    }

    /// 清除所有数据
    pub async fn clear(&self) {
        self.metrics_history.lock().await.clear();
        self.hotspots.write().await.clear();
        self.bottlenecks.write().await.clear();
    }
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::with_default_config()
    }
}

/// 指标统计
#[derive(Debug, Clone, Default)]
pub struct MetricStats {
    /// 数据点数量
    pub count: usize,
    /// 平均值
    pub avg: f64,
    /// 最小值
    pub min: f64,
    /// 最大值
    pub max: f64,
    /// 标准差
    pub std_dev: f64,
    /// 95th百分位
    pub percentile_95: f64,
    /// 99th百分位
    pub percentile_99: f64,
}

/// 性能报告
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// 运行时间
    pub uptime: Duration,
    /// FPS统计
    pub fps_stats: MetricStats,
    /// 帧时间统计
    pub frame_time_stats: MetricStats,
    /// CPU统计
    pub cpu_stats: MetricStats,
    /// 内存统计
    pub memory_stats: MetricStats,
    /// GPU统计
    pub gpu_stats: MetricStats,
    /// 性能热点
    pub hotspots: Vec<Hotspot>,
    /// 瓶颈
    pub bottlenecks: Vec<Bottleneck>,
    /// 优化建议
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyzer_creation() {
        let analyzer = PerformanceAnalyzer::with_default_config();
        let report = analyzer.generate_report().await;
        assert_eq!(report.hotspots.len(), 0);
    }

    #[tokio::test]
    async fn test_metric_recording() {
        let analyzer = PerformanceAnalyzer::with_default_config();

        let metric = Metric::new(MetricType::Fps, 60.0);
        analyzer.record_metric(metric).await;

        let stats = analyzer.get_metric_stats(MetricType::Fps).await;
        assert_eq!(stats.count, 1);
        assert_eq!(stats.avg, 60.0);
    }

    #[tokio::test]
    async fn test_hotspot_detection() {
        let analyzer = PerformanceAnalyzer::with_default_config();

        analyzer.add_hotspot("test_function", 5000).await;

        let hotspots = analyzer.get_hotspots().await;
        assert_eq!(hotspots.len(), 1);
        assert_eq!(hotspots[0].name, "test_function");
    }
}
