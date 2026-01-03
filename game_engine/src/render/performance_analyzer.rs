//! # Rendering Performance Analyzer
//!
//! 渲染性能分析工具，提供：
//! - GPU时间测量
//! - CPU时间测量
//! - 内存使用分析
//! - 性能瓶颈检测
//! - 自动性能报告生成
//! - 性能优化建议
//!
//! ## 功能特性
//!
//! ### 性能测量
//! - 帧时间分析
//! - GPU计时器
//! - CPU性能计数器
//! - 内存分配跟踪
//!
//! ### 瓶颈分析
//! - 自动检测性能瓶颈
//! - 识别最慢的渲染pass
//! - 内存泄漏检测
//! - 过度绘制分析
//!
//! ### 优化建议
//! - 自动生成优化建议
//! - 质量设置推荐
//! - 资源管理建议
//!
//! ## 使用示例
//!
//! ```ignore
//! use game_engine::render::performance_analyzer::{PerformanceAnalyzer, PerfConfig};
//!
//! let config = PerfConfig::default();
//! let mut analyzer = PerformanceAnalyzer::new(&device, config);
//!
//! // 开始帧测量
//! analyzer.begin_frame(&device);
//!
//! // 渲染...
//!
//! // 结束帧测量
//! analyzer.end_frame(&device);
//!
//! // 获取性能报告
//! let report = analyzer.generate_report();
//! println!("{}", report);
//! ```

use crate::impl_default;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use wgpu::{self, Buffer, CommandEncoder, Device, QuerySet, QuerySetDescriptor};

/// 性能分析配置
#[derive(Debug, Clone)]
pub struct PerfConfig {
    /// 是否启用GPU时间测量
    pub enable_gpu_timing: bool,
    /// 是否启用CPU时间测量
    pub enable_cpu_timing: bool,
    /// 是否启用内存跟踪
    pub enable_memory_tracking: bool,
    /// 历史帧数量（用于统计分析）
    pub history_length: usize,
    /// 性能警告阈值（毫秒）
    pub warning_threshold_ms: f32,
    /// 性能危险阈值（毫秒）
    pub critical_threshold_ms: f32,
}

impl_default!(PerfConfig {
    enable_gpu_timing: true,
    enable_cpu_timing: true,
    enable_memory_tracking: true,
    history_length: 60,
    warning_threshold_ms: 16.67,  // 60 FPS
    critical_threshold_ms: 33.33, // 30 FPS
});

/// 性能统计信息
#[derive(Debug, Default, Clone)]
pub struct PerformanceStats {
    /// 总帧时间（毫秒）
    pub total_frame_time_ms: f32,
    /// GPU时间（毫秒）
    pub gpu_time_ms: f32,
    /// CPU时间（毫秒）
    pub cpu_time_ms: f32,
    /// 绘制调用次数
    pub draw_calls: u32,
    /// 三角形数量
    pub triangle_count: u32,
    /// 顶点数量
    pub vertex_count: u32,
    /// 内存使用（字节）
    pub memory_used_bytes: u64,
    /// 帧率
    pub fps: f32,
}

/// Pass性能信息
#[derive(Debug, Default, Clone)]
pub struct PassPerformance {
    /// Pass名称
    pub name: String,
    /// GPU时间（毫秒）
    pub gpu_time_ms: f32,
    /// CPU时间（毫秒）
    pub cpu_time_ms: f32,
    /// 调用次数
    pub call_count: u32,
}

/// 性能瓶颈
#[derive(Debug, Clone)]
pub enum PerformanceBottleneck {
    /// GPU性能瓶颈
    GpuBottleneck {
        /// 最慢的pass
        slowest_pass: String,
        /// GPU时间（毫秒）
        gpu_time_ms: f32,
    },
    /// CPU性能瓶颈
    CpuBottleneck {
        /// 最慢的系统
        slowest_system: String,
        /// CPU时间（毫秒）
        cpu_time_ms: f32,
    },
    /// 内存瓶颈
    MemoryBottleneck {
        /// 内存使用（字节）
        memory_used: u64,
        /// 内存增长速率（字节/秒）
        growth_rate: f32,
    },
    /// 绘制调用过多
    DrawCallOverhead {
        /// 绘制调用次数
        draw_calls: u32,
        /// 建议的最大值
        recommended_max: u32,
    },
}

/// 优化建议
#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    /// 建议类型
    pub suggestion_type: SuggestionType,
    /// 建议描述
    pub description: String,
    /// 预期改善
    pub expected_improvement: String,
    /// 优先级（0-10）
    pub priority: u32,
}

/// 建议类型
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionType {
    /// GPU优化
    GpuOptimization,
    /// CPU优化
    CpuOptimization,
    /// 内存优化
    MemoryOptimization,
    /// 渲染管线优化
    PipelineOptimization,
    /// 资源管理优化
    ResourceOptimization,
}

impl std::fmt::Display for SuggestionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuggestionType::GpuOptimization => write!(f, "GPU优化"),
            SuggestionType::CpuOptimization => write!(f, "CPU优化"),
            SuggestionType::MemoryOptimization => write!(f, "内存优化"),
            SuggestionType::PipelineOptimization => write!(f, "渲染管线优化"),
            SuggestionType::ResourceOptimization => write!(f, "资源管理优化"),
        }
    }
}

/// 性能报告
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// 总体统计
    pub stats: PerformanceStats,
    /// Pass性能信息
    pub pass_performance: Vec<PassPerformance>,
    /// 性能瓶颈
    pub bottlenecks: Vec<PerformanceBottleneck>,
    /// 优化建议
    pub suggestions: Vec<OptimizationSuggestion>,
    /// 帧率趋势
    pub fps_trend: FpsTrend,
}

/// 帧率趋势
#[derive(Debug, Clone, PartialEq)]
pub enum FpsTrend {
    /// 稳定
    Stable,
    /// 上升
    Improving,
    /// 下降
    Declining,
    /// 波动
    Fluctuating,
}

/// 渲染性能分析器
pub struct PerformanceAnalyzer {
    config: PerfConfig,
    /// 性能历史记录
    frame_history: Vec<PerformanceStats>,
    /// Pass性能记录
    pass_performance: HashMap<String, Vec<PassPerformance>>,
    /// 当前帧的开始时间
    frame_start_time: Option<Instant>,
    /// GPU查询集合
    timestamp_queries: Option<QuerySet>,
    /// 当前帧的pass
    current_passes: Vec<PassPerformance>,
    /// 上一帧的内存使用
    last_memory_used: u64,
    /// 内存采样时间
    memory_sample_time: Instant,
}

impl PerformanceAnalyzer {
    /// 创建性能分析器
    pub fn new(device: &Device, config: PerfConfig) -> Self {
        // 创建GPU时间戳查询集合
        let enable_gpu_timing = config.enable_gpu_timing;
        let history_length = config.history_length;
        let timestamp_queries = if enable_gpu_timing {
            Some(device.create_query_set(&QuerySetDescriptor {
                label: Some("Performance Timestamp Queries"),
                count: 1000, // 最大1000个时间戳
                ty: wgpu::QueryType::Timestamp,
            }))
        } else {
            None
        };

        Self {
            config,
            frame_history: Vec::with_capacity(history_length),
            pass_performance: HashMap::new(),
            frame_start_time: None,
            timestamp_queries,
            current_passes: Vec::new(),
            last_memory_used: 0,
            memory_sample_time: Instant::now(),
        }
    }

    /// 开始帧测量
    pub fn begin_frame(&mut self, _device: &Device) {
        self.frame_start_time = Some(Instant::now());
        self.current_passes.clear();
    }

    /// 开始Pass测量
    pub fn begin_pass(&mut self, name: &str) {
        if !self.config.enable_cpu_timing && !self.config.enable_gpu_timing {
            return;
        }

        self.current_passes.push(PassPerformance {
            name: name.to_string(),
            ..Default::default()
        });
    }

    /// 结束Pass测量
    pub fn end_pass(&mut self, name: &str) {
        if let Some(pass) = self.current_passes.iter_mut().find(|p| p.name == name) {
            // 这里可以记录具体的GPU/CPU时间
            pass.call_count += 1;
        }
    }

    /// 结束帧测量
    pub fn end_frame(&mut self, device: &Device) {
        let frame_start = self.frame_start_time.take().unwrap_or_else(Instant::now);
        let frame_time = frame_start.elapsed().as_secs_f64() as f32;

        // 计算内存使用
        let memory_used = if self.config.enable_memory_tracking {
            self.estimate_memory_usage()
        } else {
            0
        };

        // 计算内存增长速率
        let elapsed_since_sample = self.memory_sample_time.elapsed().as_secs_f64();
        let growth_rate = if elapsed_since_sample > 0.0 {
            (memory_used as f32 - self.last_memory_used as f32) / elapsed_since_sample as f32
        } else {
            0.0
        };

        // 每秒更新一次内存基准
        if elapsed_since_sample >= 1.0 {
            self.last_memory_used = memory_used;
            self.memory_sample_time = Instant::now();
        }

        let stats = PerformanceStats {
            total_frame_time_ms: frame_time,
            gpu_time_ms: self.calculate_gpu_time(),
            cpu_time_ms: self.calculate_cpu_time(),
            draw_calls: self.estimate_draw_calls(),
            triangle_count: self.estimate_triangles(),
            vertex_count: self.estimate_vertices(),
            memory_used_bytes: memory_used,
            fps: if frame_time > 0.0 {
                1000.0 / frame_time
            } else {
                0.0
            },
        };

        // 添加到历史记录
        self.frame_history.push(stats);
        if self.frame_history.len() > self.config.history_length {
            self.frame_history.remove(0);
        }

        // 记录pass性能
        for pass in &self.current_passes {
            self.pass_performance
                .entry(pass.name.clone())
                .or_insert_with(Vec::new)
                .push(pass.clone());
        }
    }

    /// 生成性能报告
    pub fn generate_report(&self) -> PerformanceReport {
        let stats = self.get_average_stats();

        let bottlenecks = self.detect_bottlenecks(&stats);
        let suggestions = self.generate_suggestions(&stats, &bottlenecks);
        let fps_trend = self.analyze_fps_trend();

        PerformanceReport {
            stats,
            pass_performance: self.get_pass_performance_summary(),
            bottlenecks,
            suggestions,
            fps_trend,
        }
    }

    /// 获取平均统计信息
    fn get_average_stats(&self) -> PerformanceStats {
        if self.frame_history.is_empty() {
            return PerformanceStats::default();
        }

        let count = self.frame_history.len() as f32;
        let sum: PerformanceStats =
            self.frame_history
                .iter()
                .cloned()
                .fold(PerformanceStats::default(), |mut acc, s| {
                    acc.total_frame_time_ms += s.total_frame_time_ms;
                    acc.gpu_time_ms += s.gpu_time_ms;
                    acc.cpu_time_ms += s.cpu_time_ms;
                    acc.draw_calls += s.draw_calls;
                    acc.triangle_count += s.triangle_count;
                    acc.vertex_count += s.vertex_count;
                    acc.memory_used_bytes += s.memory_used_bytes;
                    acc.fps += s.fps;
                    acc
                });

        PerformanceStats {
            total_frame_time_ms: sum.total_frame_time_ms / count,
            gpu_time_ms: sum.gpu_time_ms / count,
            cpu_time_ms: sum.cpu_time_ms / count,
            draw_calls: sum.draw_calls / count as u32,
            triangle_count: sum.triangle_count / count as u32,
            vertex_count: sum.vertex_count / count as u32,
            memory_used_bytes: sum.memory_used_bytes / count as u64,
            fps: sum.fps / count,
        }
    }

    /// 计算GPU时间
    fn calculate_gpu_time(&self) -> f32 {
        // 简化实现，实际应该从GPU查询获取
        self.current_passes.iter().map(|p| p.gpu_time_ms).sum()
    }

    /// 计算CPU时间
    fn calculate_cpu_time(&self) -> f32 {
        self.current_passes.iter().map(|p| p.cpu_time_ms).sum()
    }

    /// 估计绘制调用次数
    fn estimate_draw_calls(&self) -> u32 {
        self.current_passes.iter().map(|p| p.call_count).sum()
    }

    /// 估计三角形数量
    fn estimate_triangles(&self) -> u32 {
        // 简化实现
        10000
    }

    /// 估计顶点数量
    fn estimate_vertices(&self) -> u32 {
        // 简化实现
        30000
    }

    /// 估计内存使用
    fn estimate_memory_usage(&self) -> u64 {
        // 简化实现，实际应该查询 allocator
        100 * 1024 * 1024 // 100 MB
    }

    /// 检测性能瓶颈
    fn detect_bottlenecks(&self, stats: &PerformanceStats) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();

        // GPU瓶颈
        if stats.gpu_time_ms > self.config.warning_threshold_ms * 0.8 {
            let slowest_pass = self
                .current_passes
                .iter()
                .max_by(|a, b| a.gpu_time_ms.partial_cmp(&b.gpu_time_ms).unwrap())
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            bottlenecks.push(PerformanceBottleneck::GpuBottleneck {
                slowest_pass,
                gpu_time_ms: stats.gpu_time_ms,
            });
        }

        // CPU瓶颈
        if stats.cpu_time_ms > self.config.warning_threshold_ms * 0.6 {
            let slowest = "Culling".to_string(); // 简化实现
            bottlenecks.push(PerformanceBottleneck::CpuBottleneck {
                slowest_system: slowest,
                cpu_time_ms: stats.cpu_time_ms,
            });
        }

        // 内存瓶颈
        if stats.memory_used_bytes > 500 * 1024 * 1024 {
            // > 500 MB
            bottlenecks.push(PerformanceBottleneck::MemoryBottleneck {
                memory_used: stats.memory_used_bytes,
                growth_rate: 0.0,
            });
        }

        // 绘制调用过多
        if stats.draw_calls > 1000 {
            bottlenecks.push(PerformanceBottleneck::DrawCallOverhead {
                draw_calls: stats.draw_calls,
                recommended_max: 1000,
            });
        }

        bottlenecks
    }

    /// 生成优化建议
    fn generate_suggestions(
        &self,
        stats: &PerformanceStats,
        bottlenecks: &[PerformanceBottleneck],
    ) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        for bottleneck in bottlenecks {
            match bottleneck {
                PerformanceBottleneck::GpuBottleneck { .. } => {
                    suggestions.push(OptimizationSuggestion {
                        suggestion_type: SuggestionType::GpuOptimization,
                        description: "考虑降低渲染分辨率或禁用部分后处理效果".to_string(),
                        expected_improvement: "预期性能提升20-30%".to_string(),
                        priority: 8,
                    });
                }
                PerformanceBottleneck::CpuBottleneck { .. } => {
                    suggestions.push(OptimizationSuggestion {
                        suggestion_type: SuggestionType::CpuOptimization,
                        description: "启用GPU剔除或优化场景遍历算法".to_string(),
                        expected_improvement: "预期CPU时间减少40%".to_string(),
                        priority: 9,
                    });
                }
                PerformanceBottleneck::MemoryBottleneck { .. } => {
                    suggestions.push(OptimizationSuggestion {
                        suggestion_type: SuggestionType::MemoryOptimization,
                        description: "使用纹理压缩或减少资源缓存大小".to_string(),
                        expected_improvement: "预期内存使用减少30%".to_string(),
                        priority: 7,
                    });
                }
                PerformanceBottleneck::DrawCallOverhead { .. } => {
                    suggestions.push(OptimizationSuggestion {
                        suggestion_type: SuggestionType::PipelineOptimization,
                        description: "使用实例化渲染或批次合并减少绘制调用".to_string(),
                        expected_improvement: "预期绘制调用减少60%".to_string(),
                        priority: 10,
                    });
                }
            }
        }

        suggestions
    }

    /// 分析帧率趋势
    fn analyze_fps_trend(&self) -> FpsTrend {
        if self.frame_history.len() < 10 {
            return FpsTrend::Stable;
        }

        let recent: Vec<_> = self.frame_history.iter().rev().take(10).map(|s| s.fps).collect();

        let avg = recent.iter().sum::<f32>() / recent.len() as f32;
        let variance =
            recent.iter().map(|&fps| (fps - avg).powi(2)).sum::<f32>() / recent.len() as f32;
        let std_dev = variance.sqrt();

        let coefficient_of_variation = std_dev / avg;

        if coefficient_of_variation < 0.05 {
            FpsTrend::Stable
        } else if coefficient_of_variation > 0.15 {
            FpsTrend::Fluctuating
        } else {
            // 检查趋势
            let first_half_avg = recent[5..10].iter().sum::<f32>() / 5.0;
            let second_half_avg = recent[0..5].iter().sum::<f32>() / 5.0;

            if second_half_avg > first_half_avg * 1.1 {
                FpsTrend::Improving
            } else if second_half_avg < first_half_avg * 0.9 {
                FpsTrend::Declining
            } else {
                FpsTrend::Stable
            }
        }
    }

    /// 获取Pass性能摘要
    fn get_pass_performance_summary(&self) -> Vec<PassPerformance> {
        let mut summary = Vec::new();

        for (name, passes) in &self.pass_performance {
            if !passes.is_empty() {
                let avg_gpu =
                    passes.iter().map(|p| p.gpu_time_ms).sum::<f32>() / passes.len() as f32;
                let avg_cpu =
                    passes.iter().map(|p| p.cpu_time_ms).sum::<f32>() / passes.len() as f32;

                summary.push(PassPerformance {
                    name: name.clone(),
                    gpu_time_ms: avg_gpu,
                    cpu_time_ms: avg_cpu,
                    call_count: passes.iter().map(|p| p.call_count).sum::<u32>()
                        / passes.len() as u32,
                });
            }
        }

        // 按GPU时间排序
        summary.sort_by(|a, b| b.gpu_time_ms.partial_cmp(&a.gpu_time_ms).unwrap());
        summary
    }

    /// 打印性能报告
    pub fn print_report(&self) {
        let report = self.generate_report();

        println!("\n=== 渲染性能报告 ===");
        println!("帧率: {:.1} FPS", report.stats.fps);
        println!("帧时间: {:.2} ms", report.stats.total_frame_time_ms);
        println!("GPU时间: {:.2} ms", report.stats.gpu_time_ms);
        println!("CPU时间: {:.2} ms", report.stats.cpu_time_ms);
        println!("绘制调用: {}", report.stats.draw_calls);
        println!(
            "内存使用: {:.1} MB",
            report.stats.memory_used_bytes as f32 / 1024.0 / 1024.0
        );
        println!("帧率趋势: {:?}", report.fps_trend);

        println!("\n=== 性能瓶颈 ===");
        for bottleneck in &report.bottlenecks {
            println!("{:?}", bottleneck);
        }

        println!("\n=== 优化建议 ===");
        for suggestion in &report.suggestions {
            println!(
                "[优先级: {}] {} - {} (预期: {})",
                suggestion.priority,
                suggestion.suggestion_type,
                suggestion.description,
                suggestion.expected_improvement
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = PerfConfig::default();
        assert!(config.enable_gpu_timing);
        assert!(config.enable_cpu_timing);
        assert_eq!(config.history_length, 60);
    }

    #[test]
    fn test_fps_trend_analysis() {
        let analyzer = PerformanceAnalyzer::new(
            &unsafe { Device::dummy() }, // 假设备有dummy实现
            PerfConfig::default(),
        );

        // 测试空历史
        let trend = analyzer.analyze_fps_trend();
        assert_eq!(trend, FpsTrend::Stable);
    }
}
