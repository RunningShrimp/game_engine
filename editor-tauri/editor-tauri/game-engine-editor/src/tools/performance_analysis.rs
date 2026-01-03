//! P1-2: 性能优化工具和分析
//!
//! 提供全面的性能分析、热点检测、内存优化和并发优化功能

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::atomic::{AtomicU64, Ordering};

/// 性能分析器 - 全面分析各系统性能
pub struct PerformanceAnalyzer {
    /// LSP性能指标
    lsp_metrics: Arc<Mutex<LSPMetrics>>,
    /// C#运行时性能指标
    csharp_metrics: Arc<Mutex<CSharpMetrics>>,
    /// 网络性能指标
    network_metrics: Arc<Mutex<NetworkMetrics>>,
    /// AI性能指标
    ai_metrics: Arc<Mutex<AIMetrics>>,
    /// 编辑器性能指标
    editor_metrics: Arc<Mutex<EditorMetrics>>,
}

/// LSP性能指标
#[derive(Debug, Clone)]
pub struct LSPMetrics {
    /// 补全响应时间（毫秒）
    pub completion_times: Vec<Duration>,
    /// 悬停响应时间（毫秒）
    pub hover_times: Vec<Duration>,
    /// 跳转定义响应时间（毫秒）
    pub goto_definition_times: Vec<Duration>,
    /// 内存占用（MB）
    pub memory_usage_mb: f64,
    /// CPU使用率（%）
    pub cpu_usage_percent: f64,
}

/// C#运行时性能指标
#[derive(Debug, Clone)]
pub struct CSharpMetrics {
    /// 方法调用延迟（微秒）
    pub method_call_latencies: Vec<Duration>,
    /// 类型转换时间（微秒）
    pub type_conversion_times: Vec<Duration>,
    /// GC暂停时间（毫秒）
    pub gc_pause_times: Vec<Duration>,
    /// 程序集加载时间（毫秒）
    pub assembly_load_times: Vec<Duration>,
    /// 内存占用（MB）
    pub memory_usage_mb: f64,
}

/// 网络性能指标
#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    /// TCP吞吐量（MB/s）
    pub tcp_throughput: f64,
    /// UDP吞吐量（MB/s）
    pub udp_throughput: f64,
    /// 延迟（毫秒）
    pub latency_ms: f64,
    /// 丢包率（%）
    pub packet_loss_percent: f64,
    /// 带宽占用（KB/s）
    pub bandwidth_usage_kbps: f64,
    /// 并发连接数
    pub concurrent_connections: usize,
}

/// AI性能指标
#[derive(Debug, Clone)]
pub struct AIMetrics {
    /// NavMesh构建时间（毫秒）
    pub navmesh_build_times: Vec<Duration>,
    /// A*寻路时间（微秒）
    pub astar_times: Vec<Duration>,
    /// Agent更新时间（微秒）
    pub agent_update_times: Vec<Duration>,
    /// 并发Agent数量
    pub concurrent_agents: usize,
}

/// 编辑器性能指标
#[derive(Debug, Clone)]
pub struct EditorMetrics {
    /// 帧率（FPS）
    pub fps: f64,
    /// 帧时间（毫秒）
    pub frame_time_ms: f64,
    /// 内存占用（MB）
    pub memory_usage_mb: f64,
    /// 渲染时间（毫秒）
    pub render_time_ms: f64,
    /// UI响应时间（毫秒）
    pub ui_response_time_ms: f64,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            lsp_metrics: Arc::new(Mutex::new(LSPMetrics {
                completion_times: Vec::new(),
                hover_times: Vec::new(),
                goto_definition_times: Vec::new(),
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
            })),
            csharp_metrics: Arc::new(Mutex::new(CSharpMetrics {
                method_call_latencies: Vec::new(),
                type_conversion_times: Vec::new(),
                gc_pause_times: Vec::new(),
                assembly_load_times: Vec::new(),
                memory_usage_mb: 0.0,
            })),
            network_metrics: Arc::new(Mutex::new(NetworkMetrics {
                tcp_throughput: 0.0,
                udp_throughput: 0.0,
                latency_ms: 0.0,
                packet_loss_percent: 0.0,
                bandwidth_usage_kbps: 0.0,
                concurrent_connections: 0,
            })),
            ai_metrics: Arc::new(Mutex::new(AIMetrics {
                navmesh_build_times: Vec::new(),
                astar_times: Vec::new(),
                agent_update_times: Vec::new(),
                concurrent_agents: 0,
            })),
            editor_metrics: Arc::new(Mutex::new(EditorMetrics {
                fps: 0.0,
                frame_time_ms: 0.0,
                memory_usage_mb: 0.0,
                render_time_ms: 0.0,
                ui_response_time_ms: 0.0,
            })),
        }
    }

    /// 分析LSP性能
    pub fn analyze_lsp_performance(&self) -> LSPPerformanceReport {
        let metrics = self.lsp_metrics.lock().unwrap();

        let avg_completion = if !metrics.completion_times.is_empty() {
            let total: Duration = metrics.completion_times.iter().sum();
            total.as_millis() as f64 / metrics.completion_times.len() as f64
        } else {
            0.0
        };

        let avg_hover = if !metrics.hover_times.is_empty() {
            let total: Duration = metrics.hover_times.iter().sum();
            total.as_millis() as f64 / metrics.hover_times.len() as f64
        } else {
            0.0
        };

        let avg_goto = if !metrics.goto_definition_times.is_empty() {
            let total: Duration = metrics.goto_definition_times.iter().sum();
            total.as_millis() as f64 / metrics.goto_definition_times.len() as f64
        } else {
            0.0
        };

        LSPPerformanceReport {
            avg_completion_time_ms: avg_completion,
            avg_hover_time_ms: avg_hover,
            avg_goto_definition_time_ms: avg_goto,
            memory_usage_mb: metrics.memory_usage_mb,
            cpu_usage_percent: metrics.cpu_usage_percent,
            target_completion_ms: 50.0,  // 目标：<100ms → <50ms
            target_hover_ms: 25.0,        // 目标：<50ms → <25ms
            target_goto_ms: 15.0,         // 目标：<30ms → <15ms
            optimization_potential: self.calculate_lsp_optimization_potential(&metrics),
        }
    }

    /// 分析C#性能
    pub fn analyze_csharp_performance(&self) -> CSharpPerformanceReport {
        let metrics = self.csharp_metrics.lock().unwrap();

        let avg_call_latency = if !metrics.method_call_latencies.is_empty() {
            let total: Duration = metrics.method_call_latencies.iter().sum();
            total.as_micros() as f64 / metrics.method_call_latencies.len() as f64
        } else {
            0.0
        };

        let avg_gc_pause = if !metrics.gc_pause_times.is_empty() {
            let total: Duration = metrics.gc_pause_times.iter().sum();
            total.as_millis() as f64 / metrics.gc_pause_times.len() as f64
        } else {
            0.0
        };

        CSharpPerformanceReport {
            avg_method_call_latency_us: avg_call_latency,
            avg_type_conversion_time_us: 0.0,  // 类型转换时间（简化计算）
            avg_gc_pause_ms: avg_gc_pause,
            avg_assembly_load_ms: 0.0,         // 程序集加载时间（简化计算）
            memory_usage_mb: metrics.memory_usage_mb,
            target_call_latency_us: 500.0,    // 目标：<1ms → <0.5ms
            optimization_potential: self.calculate_csharp_optimization_potential(&metrics),
        }
    }

    /// 分析网络性能
    pub fn analyze_network_performance(&self) -> NetworkPerformanceReport {
        let metrics = self.network_metrics.lock().unwrap();

        NetworkPerformanceReport {
            tcp_throughput_mbps: metrics.tcp_throughput,
            udp_throughput_mbps: metrics.udp_throughput,
            latency_ms: metrics.latency_ms,
            packet_loss_percent: metrics.packet_loss_percent,
            bandwidth_usage_kbps: metrics.bandwidth_usage_kbps,
            concurrent_connections: metrics.concurrent_connections,
            target_latency_ms: 50.0,           // 目标：<100ms → <50ms
            target_bandwidth_kbps: 50.0,       // 目标：<50KB/s
            optimization_potential: self.calculate_network_optimization_potential(&metrics),
        }
    }

    /// 分析AI性能
    pub fn analyze_ai_performance(&self) -> AIPerformanceReport {
        let metrics = self.ai_metrics.lock().unwrap();

        let avg_navmesh_build = if !metrics.navmesh_build_times.is_empty() {
            let total: Duration = metrics.navmesh_build_times.iter().sum();
            total.as_millis() as f64 / metrics.navmesh_build_times.len() as f64
        } else {
            0.0
        };

        let avg_astar = if !metrics.astar_times.is_empty() {
            let total: Duration = metrics.astar_times.iter().sum();
            total.as_micros() as f64 / metrics.astar_times.len() as f64
        } else {
            0.0
        };

        AIPerformanceReport {
            avg_navmesh_build_ms: avg_navmesh_build,
            avg_astar_time_us: avg_astar,
            avg_agent_update_us: 0.0,
            concurrent_agents: metrics.concurrent_agents,
            target_astar_us: 5000.0,           // 目标：<10ms → <5ms
            target_navmesh_build_ms: 3000.0,   // 目标：<5000ms
            optimization_potential: self.calculate_ai_optimization_potential(&metrics),
        }
    }

    /// 分析编辑器性能
    pub fn analyze_editor_performance(&self) -> EditorPerformanceReport {
        let metrics = self.editor_metrics.lock().unwrap();

        EditorPerformanceReport {
            fps: metrics.fps,
            frame_time_ms: metrics.frame_time_ms,
            memory_usage_mb: metrics.memory_usage_mb,
            render_time_ms: metrics.render_time_ms,
            ui_response_time_ms: metrics.ui_response_time_ms,
            target_fps: 120.0,                // 目标：60 FPS → 120 FPS
            target_frame_time_ms: 8.33,       // 目标：16.67ms → 8.33ms
            optimization_potential: self.calculate_editor_optimization_potential(&metrics),
        }
    }

    /// 生成综合性能报告
    pub fn generate_comprehensive_report(&self) -> ComprehensivePerformanceReport {
        ComprehensivePerformanceReport {
            lsp_report: self.analyze_lsp_performance(),
            csharp_report: self.analyze_csharp_performance(),
            network_report: self.analyze_network_performance(),
            ai_report: self.analyze_ai_performance(),
            editor_report: self.analyze_editor_performance(),
            overall_score: self.calculate_overall_score(),
            recommendations: self.generate_recommendations(),
        }
    }

    /// 计算LSP优化潜力
    fn calculate_lsp_optimization_potential(&self, metrics: &LSPMetrics) -> f64 {
        let current_avg = if !metrics.completion_times.is_empty() {
            let total: Duration = metrics.completion_times.iter().sum();
            total.as_millis() as f64 / metrics.completion_times.len() as f64
        } else {
            0.0
        };

        if current_avg > 100.0 {
            (current_avg - 50.0) / current_avg * 100.0
        } else {
            0.0
        }
    }

    /// 计算C#优化潜力
    fn calculate_csharp_optimization_potential(&self, metrics: &CSharpMetrics) -> f64 {
        let current_avg = if !metrics.method_call_latencies.is_empty() {
            let total: Duration = metrics.method_call_latencies.iter().sum();
            total.as_micros() as f64 / metrics.method_call_latencies.len() as f64
        } else {
            0.0
        };

        if current_avg > 1000.0 {
            (current_avg - 500.0) / current_avg * 100.0
        } else {
            0.0
        }
    }

    /// 计算网络优化潜力
    fn calculate_network_optimization_potential(&self, metrics: &NetworkMetrics) -> f64 {
        if metrics.latency_ms > 100.0 {
            (metrics.latency_ms - 50.0) / metrics.latency_ms * 100.0
        } else {
            0.0
        }
    }

    /// 计算AI优化潜力
    fn calculate_ai_optimization_potential(&self, metrics: &AIMetrics) -> f64 {
        let current_avg = if !metrics.astar_times.is_empty() {
            let total: Duration = metrics.astar_times.iter().sum();
            total.as_micros() as f64 / metrics.astar_times.len() as f64
        } else {
            0.0
        };

        if current_avg > 10000.0 {
            (current_avg - 5000.0) / current_avg * 100.0
        } else {
            0.0
        }
    }

    /// 计算编辑器优化潜力
    fn calculate_editor_optimization_potential(&self, metrics: &EditorMetrics) -> f64 {
        if metrics.fps < 60.0 {
            (120.0 - metrics.fps) / 120.0 * 100.0
        } else if metrics.fps < 120.0 {
            (120.0 - metrics.fps) / 120.0 * 50.0
        } else {
            0.0
        }
    }

    /// 计算总体性能得分
    fn calculate_overall_score(&self) -> f64 {
        let lsp = self.analyze_lsp_performance();
        let csharp = self.analyze_csharp_performance();
        let network = self.analyze_network_performance();
        let ai = self.analyze_ai_performance();
        let editor = self.analyze_editor_performance();

        let lsp_score = if lsp.avg_completion_time_ms <= 50.0 { 100.0 } else { 50.0 };
        let csharp_score = if csharp.avg_method_call_latency_us <= 500.0 { 100.0 } else { 50.0 };
        let network_score = if network.latency_ms <= 50.0 { 100.0 } else { 70.0 };
        let ai_score = if ai.avg_astar_time_us <= 5000.0 { 100.0 } else { 70.0 };
        let editor_score = if editor.fps >= 120.0 { 100.0 } else { 70.0 };

        (lsp_score + csharp_score + network_score + ai_score + editor_score) / 5.0
    }

    /// 生成优化建议
    fn generate_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        let lsp = self.analyze_lsp_performance();
        if lsp.avg_completion_time_ms > 50.0 {
            recommendations.push(OptimizationRecommendation {
                category: "LSP".to_string(),
                priority: "High".to_string(),
                description: "LSP补全响应时间超过50ms目标".to_string(),
                actions: vec![
                    "实现索引缓存机制".to_string(),
                    "使用增量解析避免全量分析".to_string(),
                    "优化补全项排序算法".to_string(),
                ],
                expected_improvement: "30-50%性能提升".to_string(),
            });
        }

        let csharp = self.analyze_csharp_performance();
        if csharp.avg_method_call_latency_us > 500.0 {
            recommendations.push(OptimizationRecommendation {
                category: "C# Runtime".to_string(),
                priority: "High".to_string(),
                description: "C#方法调用延迟超过0.5ms目标".to_string(),
                actions: vec![
                    "实现方法指针缓存".to_string(),
                    "优化类型转换逻辑".to_string(),
                    "减少P/Invoke调用开销".to_string(),
                ],
                expected_improvement: "40-60%延迟降低".to_string(),
            });
        }

        let network = self.analyze_network_performance();
        if network.latency_ms > 50.0 {
            recommendations.push(OptimizationRecommendation {
                category: "Network".to_string(),
                priority: "Medium".to_string(),
                description: "网络延迟超过50ms目标".to_string(),
                actions: vec![
                    "实现客户端预测".to_string(),
                    "优化Delta序列化".to_string(),
                    "使用UDP代替TCP（部分场景）".to_string(),
                ],
                expected_improvement: "20-40%延迟降低".to_string(),
            });
        }

        let ai = self.analyze_ai_performance();
        if ai.avg_astar_time_us > 5000.0 {
            recommendations.push(OptimizationRecommendation {
                category: "AI".to_string(),
                priority: "Medium".to_string(),
                description: "A*寻路时间超过5ms目标".to_string(),
                actions: vec![
                    "实现路径缓存".to_string(),
                    "使用分层寻路".to_string(),
                    "并行化多个Agent的寻路".to_string(),
                ],
                expected_improvement: "50-70%性能提升".to_string(),
            });
        }

        let editor = self.analyze_editor_performance();
        if editor.fps < 120.0 {
            recommendations.push(OptimizationRecommendation {
                category: "Editor".to_string(),
                priority: "High".to_string(),
                description: "编辑器帧率低于120FPS目标".to_string(),
                actions: vec![
                    "优化渲染管线".to_string(),
                    "实现遮挡剔除".to_string(),
                    "使用实例化渲染".to_string(),
                ],
                expected_improvement: "50-100%帧率提升".to_string(),
            });
        }

        recommendations
    }
}

/// LSP性能报告
#[derive(Debug)]
pub struct LSPPerformanceReport {
    pub avg_completion_time_ms: f64,
    pub avg_hover_time_ms: f64,
    pub avg_goto_definition_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub target_completion_ms: f64,
    pub target_hover_ms: f64,
    pub target_goto_ms: f64,
    pub optimization_potential: f64,
}

/// C#性能报告
#[derive(Debug)]
pub struct CSharpPerformanceReport {
    pub avg_method_call_latency_us: f64,
    pub avg_type_conversion_time_us: f64,
    pub avg_gc_pause_ms: f64,
    pub avg_assembly_load_ms: f64,
    pub memory_usage_mb: f64,
    pub target_call_latency_us: f64,
    pub optimization_potential: f64,
}

/// 网络性能报告
#[derive(Debug)]
pub struct NetworkPerformanceReport {
    pub tcp_throughput_mbps: f64,
    pub udp_throughput_mbps: f64,
    pub latency_ms: f64,
    pub packet_loss_percent: f64,
    pub bandwidth_usage_kbps: f64,
    pub concurrent_connections: usize,
    pub target_latency_ms: f64,
    pub target_bandwidth_kbps: f64,
    pub optimization_potential: f64,
}

/// AI性能报告
#[derive(Debug)]
pub struct AIPerformanceReport {
    pub avg_navmesh_build_ms: f64,
    pub avg_astar_time_us: f64,
    pub avg_agent_update_us: f64,
    pub concurrent_agents: usize,
    pub target_astar_us: f64,
    pub target_navmesh_build_ms: f64,
    pub optimization_potential: f64,
}

/// 编辑器性能报告
#[derive(Debug)]
pub struct EditorPerformanceReport {
    pub fps: f64,
    pub frame_time_ms: f64,
    pub memory_usage_mb: f64,
    pub render_time_ms: f64,
    pub ui_response_time_ms: f64,
    pub target_fps: f64,
    pub target_frame_time_ms: f64,
    pub optimization_potential: f64,
}

/// 综合性能报告
#[derive(Debug)]
pub struct ComprehensivePerformanceReport {
    pub lsp_report: LSPPerformanceReport,
    pub csharp_report: CSharpPerformanceReport,
    pub network_report: NetworkPerformanceReport,
    pub ai_report: AIPerformanceReport,
    pub editor_report: EditorPerformanceReport,
    pub overall_score: f64,
    pub recommendations: Vec<OptimizationRecommendation>,
}

/// 优化建议
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub category: String,
    pub priority: String,
    pub description: String,
    pub actions: Vec<String>,
    pub expected_improvement: String,
}

impl ComprehensivePerformanceReport {
    /// 打印性能报告
    pub fn print(&self) {
        println!("\n{}", "=".repeat(80));
        println!("📊 综合性能分析报告");
        println!("{}", "=".repeat(80));

        println!("\n## 总体评分: {:.1}/100", self.overall_score);

        println!("\n## LSP性能");
        println!("  补全响应时间: {:.2}ms (目标: <{}ms)", self.lsp_report.avg_completion_time_ms, self.lsp_report.target_completion_ms);
        println!("  悬停响应时间: {:.2}ms (目标: <{}ms)", self.lsp_report.avg_hover_time_ms, self.lsp_report.target_hover_ms);
        println!("  跳转定义时间: {:.2}ms (目标: <{}ms)", self.lsp_report.avg_goto_definition_time_ms, self.lsp_report.target_goto_ms);
        println!("  优化潜力: {:.1}%", self.lsp_report.optimization_potential);

        println!("\n## C#运行时性能");
        println!("  方法调用延迟: {:.2}μs (目标: <{}μs)", self.csharp_report.avg_method_call_latency_us, self.csharp_report.target_call_latency_us);
        println!("  GC暂停时间: {:.2}ms", self.csharp_report.avg_gc_pause_ms);
        println!("  内存占用: {:.2}MB", self.csharp_report.memory_usage_mb);
        println!("  优化潜力: {:.1}%", self.csharp_report.optimization_potential);

        println!("\n## 网络性能");
        println!("  TCP吞吐量: {:.2}MB/s", self.network_report.tcp_throughput_mbps);
        println!("  UDP吞吐量: {:.2}MB/s", self.network_report.udp_throughput_mbps);
        println!("  延迟: {:.2}ms (目标: <{}ms)", self.network_report.latency_ms, self.network_report.target_latency_ms);
        println!("  带宽占用: {:.2}KB/s (目标: <{}KB/s)", self.network_report.bandwidth_usage_kbps, self.network_report.target_bandwidth_kbps);
        println!("  优化潜力: {:.1}%", self.network_report.optimization_potential);

        println!("\n## AI性能");
        println!("  A*寻路时间: {:.2}μs (目标: <{}μs)", self.ai_report.avg_astar_time_us, self.ai_report.target_astar_us);
        println!("  NavMesh构建: {:.2}ms", self.ai_report.avg_navmesh_build_ms);
        println!("  并发Agent: {}", self.ai_report.concurrent_agents);
        println!("  优化潜力: {:.1}%", self.ai_report.optimization_potential);

        println!("\n## 编辑器性能");
        println!("  帧率: {:.1} FPS (目标: >{} FPS)", self.editor_report.fps, self.editor_report.target_fps);
        println!("  帧时间: {:.2}ms (目标: <{}ms)", self.editor_report.frame_time_ms, self.editor_report.target_frame_time_ms);
        println!("  渲染时间: {:.2}ms", self.editor_report.render_time_ms);
        println!("  内存占用: {:.2}MB", self.editor_report.memory_usage_mb);
        println!("  优化潜力: {:.1}%", self.editor_report.optimization_potential);

        println!("\n## 优化建议");
        for (i, rec) in self.recommendations.iter().enumerate() {
            println!("\n  {}. [{} - 优先级: {}] {}", i+1, rec.category, rec.priority, rec.description);
            println!("     行动:");
            for action in &rec.actions {
                println!("       • {}", action);
            }
            println!("     预期效果: {}", rec.expected_improvement);
        }

        println!("\n{}", "=".repeat(80));
    }
}

/// 性能基准测试运行器
pub struct BenchmarkRunner {
    analyzer: Arc<PerformanceAnalyzer>,
}

impl BenchmarkRunner {
    pub fn new(analyzer: Arc<PerformanceAnalyzer>) -> Self {
        Self { analyzer }
    }

    /// 运行LSP基准测试
    pub fn run_lsp_benchmark(&self, iterations: usize) -> BenchmarkResult {
        println!("🏃 运行LSP基准测试 ({} 次迭代)...", iterations);

        let mut times = Vec::new();
        for i in 0..iterations {
            let start = Instant::now();

            // 模拟LSP补全操作
            self.simulate_completion();

            let duration = start.elapsed();
            times.push(duration);

            if (i + 1) % 100 == 0 {
                println!("  完成 {}/{}", i + 1, iterations);
            }
        }

        let total: Duration = times.iter().sum();
        let avg = total / iterations as u32;
        let min = times.iter().min().unwrap();
        let max = times.iter().max().unwrap();

        BenchmarkResult {
            name: "LSP Completion".to_string(),
            iterations,
            total_time: total,
            avg_time: avg,
            min_time: *min,
            max_time: *max,
            throughput: iterations as f64 / total.as_secs_f64(),
        }
    }

    /// 运行C#基准测试
    pub fn run_csharp_benchmark(&self, iterations: usize) -> BenchmarkResult {
        println!("🏃 运行C#基准测试 ({} 次迭代)...", iterations);

        let mut times = Vec::new();
        for i in 0..iterations {
            let start = Instant::now();

            // 模拟C#方法调用
            self.simulate_csharp_call();

            let duration = start.elapsed();
            times.push(duration);

            if (i + 1) % 1000 == 0 {
                println!("  完成 {}/{}", i + 1, iterations);
            }
        }

        let total: Duration = times.iter().sum();
        let avg = total / iterations as u32;
        let min = times.iter().min().unwrap();
        let max = times.iter().max().unwrap();

        BenchmarkResult {
            name: "C# Method Call".to_string(),
            iterations,
            total_time: total,
            avg_time: avg,
            min_time: *min,
            max_time: *max,
            throughput: iterations as f64 / total.as_secs_f64(),
        }
    }

    fn simulate_completion(&self) {
        // 模拟LSP补全操作
        std::thread::sleep(Duration::from_micros(100));
    }

    fn simulate_csharp_call(&self) {
        // 模拟C#方法调用
        std::thread::sleep(Duration::from_micros(50));
    }
}

/// 基准测试结果
#[derive(Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_time: Duration,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub throughput: f64, // ops/sec
}

impl BenchmarkResult {
    pub fn print(&self) {
        println!("\n📊 基准测试结果: {}", self.name);
        println!("{}", "-".repeat(60));
        println!("  迭代次数: {}", self.iterations);
        println!("  总时间: {:.2}s", self.total_time.as_secs_f64());
        println!("  平均时间: {:.2}μs", self.avg_time.as_micros() as f64);
        println!("  最小时间: {:.2}μs", self.min_time.as_micros() as f64);
        println!("  最大时间: {:.2}μs", self.max_time.as_micros() as f64);
        println!("  吞吐量: {:.2} ops/s", self.throughput);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_analyzer_creation() {
        let analyzer = PerformanceAnalyzer::new();
        let report = analyzer.generate_comprehensive_report();
        assert!(report.overall_score >= 0.0 && report.overall_score <= 100.0);
    }

    #[test]
    fn test_lsp_performance_report() {
        let analyzer = PerformanceAnalyzer::new();
        let report = analyzer.analyze_lsp_performance();
        assert!(report.avg_completion_time_ms >= 0.0);
        assert!(report.avg_hover_time_ms >= 0.0);
        assert!(report.avg_goto_definition_time_ms >= 0.0);
    }

    #[test]
    fn test_benchmark_runner() {
        let analyzer = Arc::new(PerformanceAnalyzer::new());
        let runner = BenchmarkRunner::new(analyzer);
        let result = runner.run_lsp_benchmark(100);
        assert_eq!(result.iterations, 100);
        assert!(result.avg_time.as_micros() > 0);
    }
}
