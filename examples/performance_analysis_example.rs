//! 性能分析工具示例
//!
//! 演示游戏引擎的性能分析、瓶颈检测和优化建议功能。

use game_engine::profiling::{
    BottleneckDetector, OptimizationAdvisor, PerformanceMetrics,
    bottleneck_detector::{BottleneckDiagnosis, BottleneckSeverity, BottleneckType},
};
use std::time::Duration;

fn main() {
    println!("=== 游戏引擎性能分析工具演示 ===\\n");

    // 示例1: 基础瓶颈检测
    example_1_basic_bottleneck_detection();

    // 示例2: 性能指标分析
    example_2_performance_metrics_analysis();

    // 示例3: 优化建议生成
    example_3_optimization_suggestions();

    // 示例4: 完整性能分析流程
    example_4_complete_analysis_workflow();

    // 示例5: 火焰图使用
    example_5_flamegraph_usage();

    // 示例6: 实时性能监控
    example_6_realtime_monitoring();
}

/// 示例1: 基础瓶颈检测
fn example_1_basic_bottleneck_detection() {
    println!("=== 示例1: 基础瓶颈检测 ===\\n");

    let mut detector = BottleneckDetector::new();

    println!("✓ 模拟游戏性能数据采集:");

    // 模拟记录游戏各阶段性能
    println!("  - 采集物理计算性能数据...");
    for i in 0..100 {
        let duration = Duration::from_micros(5000 + i % 200); // 稳定的物理计算
        detector.record_phase("physics_update", duration);
    }

    println!("  - 采集渲染性能数据...");
    for i in 0..100 {
        let duration = Duration::from_micros(15000 + i * 300); // 不稳定的渲染
        detector.record_phase("render_scene", duration);
    }

    println!("  - 采集AI性能数据...");
    for i in 0..100 {
        let duration = Duration::from_micros(8000 + i % 500); // 中等变异的AI
        detector.record_phase("ai_update", duration);
    }

    println!("  - 采集内存分配数据...");
    for i in 0..100 {
        let duration = Duration::from_micros(2000 + i * 100); // 逐渐增长的内存分配
        detector.record_phase("memory_allocation", duration);
    }

    println!("\\n✓ 检测性能瓶颈:");

    // 检测所有瓶颈
    let bottlenecks = detector.detect_all_bottlenecks();

    if bottlenecks.is_empty() {
        println!("  未检测到明显瓶颈");
    } else {
        for (i, bottleneck) in bottlenecks.iter().enumerate() {
            println!("  {}. {} - {}", i + 1, bottleneck.phase_name, bottleneck.severity.as_str());
            println!("     类型: {:?}", bottleneck.bottleneck_type);
            println!("     方差: {:.2}%", bottleneck.variance * 100.0);
            println!("     平均: {:.2}ms", bottleneck.average_duration.as_secs_f64() * 1000.0);
            println!("     峰值: {:.2}ms", bottleneck.peak_duration.as_secs_f64() * 1000.0);
            println!("     建议: {}", bottleneck.recommendation);
            println!();
        }
    }

    // 获取关键瓶颈
    println!("✓ 关键瓶颈 (前3个):");
    let critical_bottlenecks = detector.get_critical_bottlenecks(3);
    for (i, bottleneck) in critical_bottlenecks.iter().enumerate() {
        println!("  {}. {} - {} ({}%)",
            i + 1,
            bottleneck.phase_name,
            bottleneck.severity.as_str(),
            (bottleneck.variance * 100.0) as i32
        );
    }
    println!();
}

/// 示例2: 性能指标分析
fn example_2_performance_metrics_analysis() {
    println!("=== 示例2: 性能指标分析 ===\\n");

    // 创建各种性能场景
    let scenarios = vec![
        ("流畅游戏", PerformanceMetrics {
            fps: 60.0,
            frame_time: 16.67,
            draw_calls: 50,
            triangles: 80000,
            memory_mb: 250.0,
            texture_memory_mb: 120.0,
            gc_time_ms: 1.5,
            cpu_time: 12.0,
            gpu_time: 14.0,
        }),
        ("卡顿游戏", PerformanceMetrics {
            fps: 25.0,
            frame_time: 40.0,
            draw_calls: 200,
            triangles: 250000,
            memory_mb: 800.0,
            texture_memory_mb: 450.0,
            gc_time_ms: 12.0,
            cpu_time: 30.0,
            gpu_time: 35.0,
        }),
        ("内存密集", PerformanceMetrics {
            fps: 45.0,
            frame_time: 22.2,
            draw_calls: 80,
            triangles: 100000,
            memory_mb: 1200.0,
            texture_memory_mb: 600.0,
            gc_time_ms: 15.0,
            cpu_time: 18.0,
            gpu_time: 20.0,
        }),
        ("渲染密集", PerformanceMetrics {
            fps: 30.0,
            frame_time: 33.3,
            draw_calls: 300,
            triangles: 400000,
            memory_mb: 400.0,
            texture_memory_mb: 300.0,
            gc_time_ms: 3.0,
            cpu_time: 15.0,
            gpu_time: 45.0,
        }),
    ];

    for (name, metrics) in scenarios {
        println!("✓ 场景: {}", name);
        println!("  帧率: {:.1} FPS", metrics.fps);
        println!("  Draw Calls: {}", metrics.draw_calls);
        println!("  三角形: {}", metrics.triangles);
        println!("  内存: {:.1} MB", metrics.memory_mb);
        println!("  纹理内存: {:.1} MB", metrics.texture_memory_mb);
        println!("  GC时间: {:.2} ms", metrics.gc_time_ms);

        // 评估性能
        let performance_score = calculate_performance_score(&metrics);
        println!("  性能评分: {}/100", performance_score);

        if performance_score >= 80 {
            println!("  状态: ✅ 优秀");
        } else if performance_score >= 60 {
            println!("  状态: ⚠️  良好");
        } else {
            println!("  状态: ❌ 需要优化");
        }
        println!();
    }
}

/// 示例3: 优化建议生成
fn example_3_optimization_suggestions() {
    println!("=== 示例3: 优化建议生成 ===\\n");

    let mut advisor = OptimizationAdvisor::new();

    // 分析有性能问题的游戏
    let problem_metrics = PerformanceMetrics {
        fps: 28.0,        // 低于30 FPS
        frame_time: 35.7,
        draw_calls: 250,  // 过多
        triangles: 350000, // 过多
        memory_mb: 950.0, // 过高
        texture_memory_mb: 550.0, // 过高
        gc_time_ms: 11.0, // 过长
        cpu_time: 32.0,
        gpu_time: 38.0,
    };

    println!("✓ 当前性能问题:");
    println!("  - 帧率: {:.1} FPS (目标: 60 FPS)", problem_metrics.fps);
    println!("  - Draw Calls: {} (目标: <100)", problem_metrics.draw_calls);
    println!("  - 三角形: {} (目标: <100K)", problem_metrics.triangles);
    println!("  - 内存: {:.1} MB (目标: <500 MB)", problem_metrics.memory_mb);
    println!("  - GC时间: {:.2} ms (目标: <5 ms)", problem_metrics.gc_time_ms);
    println!();

    // 生成优化建议
    println!("✓ 生成优化建议...");
    let plan = advisor.analyze_and_suggest(&problem_metrics);

    println!("✓ 优化计划摘要:");
    println!("  - 总建议数: {}", plan.suggestion_count);
    println!("  - 高优先级建议: {}", plan.high_priority_count);
    println!("  - 预估性能提升: {:.1}%", plan.total_estimated_improvement);
    println!("  - 预估实施时间: {:.1} 小时", plan.total_estimated_time_hours);
    println!();

    // 显示前5个建议
    println!("✓ 前5个优化建议:");
    for (i, suggestion) in plan.suggestions.iter().take(5).enumerate() {
        println!("  {}. {} - {}", i + 1, suggestion.title, suggestion.priority.as_str());
        println!("     类型: {:?}", suggestion.opt_type);
        println!("     预估提升: {:.1}%", suggestion.estimated_improvement);
        println!("     难度: {}/10", suggestion.difficulty);
        println!("     风险: {}/10", suggestion.risk_level);
        println!("     预估时间: {:.1} 小时", suggestion.estimated_time_hours);
        println!("     描述: {}", suggestion.description);

        // 显示实施步骤
        if !suggestion.implementation_steps.is_empty() {
            println!("     实施步骤:");
            for (j, step) in suggestion.implementation_steps.iter().enumerate() {
                println!("       {}. {}", j + 1, step);
            }
        }
        println!();
    }
}

/// 示例4: 完整性能分析流程
fn example_4_complete_analysis_workflow() {
    println!("=== 示例4: 完整性能分析流程 ===\\n");

    println!("✓ 性能分析工作流程:");
    println!();
    println!("  阶段1: 数据采集");
    println!("    1. 启用性能分析器");
    println!("    2. 采集帧率、Draw Calls、内存等数据");
    println!("    3. 记录各阶段耗时（物理、渲染、AI等）");
    println!("    4. 采集时间: 30-60秒");
    println!();

    println!("  阶段2: 瓶颈检测");
    println!("    1. 分析性能数据");
    println!("    2. 识别性能瓶颈");
    println!("    3. 评估瓶颈严重程度");
    println!("    4. 确定瓶颈类型（CPU/GPU/内存）");
    println!();

    println!("  阶段3: 优化建议");
    println!("    1. 生成针对性优化建议");
    println!("    2. 估算性能提升");
    println!("    3. 评估实施难度和风险");
    println!("    4. 提供详细实施步骤");
    println!();

    println!("  阶段4: 优化实施");
    println!("    1. 选择高优先级优化项");
    println!("    2. 逐步实施优化");
    println!("    3. 验证优化效果");
    println!("    4. 必要时调整优化策略");
    println!();

    println!("  阶段5: 持续监控");
    println!("    1. 定期重新分析性能");
    println!("    2. 监控优化后的稳定性");
    println!("    3. 检测性能回归");
    println!("    4. 记录优化历史");
    println!();

    // 模拟完整流程
    println!("✓ 执行完整分析流程...");

    // 阶段1: 数据采集
    let mut detector = BottleneckDetector::new();
    for i in 0..200 {
        detector.record_phase("game_loop", Duration::from_micros(16667 + i % 5000));
        detector.record_phase("physics", Duration::from_micros(3000 + i % 1000));
        detector.record_phase("render", Duration::from_micros(10000 + i * 100));
        detector.record_phase("ui", Duration::from_micros(1000 + i % 500));
    }
    println!("  ✓ 数据采集完成 (200帧)");

    // 阶段2: 瓶颈检测
    let bottlenecks = detector.get_critical_bottlenecks(5);
    println!("  ✓ 检测到 {} 个瓶颈", bottlenecks.len());

    // 阶段3: 优化建议
    let mut advisor = OptimizationAdvisor::new();
    let metrics = PerformanceMetrics {
        fps: 35.0,
        frame_time: 28.6,
        draw_calls: 180,
        triangles: 200000,
        memory_mb: 650.0,
        texture_memory_mb: 280.0,
        gc_time_ms: 7.0,
        cpu_time: 22.0,
        gpu_time: 28.0,
    };
    let plan = advisor.analyze_and_suggest(&metrics);
    println!("  ✓ 生成 {} 条优化建议", plan.suggestion_count);

    // 阶段4-5: 总结
    println!();
    println!("✓ 分析结果摘要:");
    println!("  检测到的瓶颈:");
    for bottleneck in &bottlenecks {
        println!("    - {} ({})", bottleneck.phase_name, bottleneck.severity.as_str());
    }

    println!("  优化建议:");
    for suggestion in plan.suggestions.iter().take(3) {
        println!("    - {} (预估提升: {:.1}%)", suggestion.title, suggestion.estimated_improvement);
    }
    println!();
}

/// 示例5: 火焰图使用
fn example_5_flamegraph_usage() {
    println!("=== 示例5: 火焰图使用 ===\\n");

    println!("✓ Tracy Profiler火焰图功能:");
    println!();
    println!("  功能特性:");
    println!("    1. 函数耗时可视化");
    println!("    2. 调用栈分析");
    println!("    3. 热点识别");
    println!("    4. 实时性能监控");
    println!("    5. 内存分配跟踪");
    println!("    6. GPU性能分析");
    println!();

    println!("✓ 使用方法:");
    println!("  // 在代码中添加性能分析作用域");
    println!("  use game_engine::profiling::TracyProfiler;");
    println!();
    println!("  // 全局分析器实例");
    println!("  let profiler = TracyProfiler::new();");
    println!();
    println!("  // 方法1: 使用scope自动管理作用域");
    println!("  {");
    println!("      let _scope = profiler.scope(\"function_name\");");
    println!("      // 你的代码...");
    println!("  } // 作用域结束时自动记录");
    println!();
    println!("  // 方法2: 使用宏更便捷");
    println!("  use game_engine::profiling::profile_scope;");
    println!("  profile_scope!(\"function_name\");");
    println!("  // 你的代码...");
    println!();

    println!("✓ 火焰图分析技巧:");
    println!("    1. 寻找宽的横条 - 这些是耗时长的函数");
    println!("    2. 关注调用栈深度 - 过深的调用可能需要优化");
    println!("    3. 比较不同帧 - 识别不一致的性能");
    println!("    4. 使用过滤功能 - 专注于特定模块");
    println!("    5. 查看内存分配 - 找到意外的内存分配");
    println!();

    println!("✓ 常见性能模式:");
    println!("    - 单热函数: 优化该函数逻辑");
    println!("    - 多个小火函数: 考虑批处理");
    println!("    - 帧尖峰: 找到触发尖峰的代码");
    println!("    - 内存增长: 找到内存泄漏位置");
    println!();
}

/// 示例6: 实时性能监控
fn example_6_realtime_monitoring() {
    println!("=== 示例6: 实时性能监控 ===\\n");

    println!("✓ 实时性能监控系统:");
    println!();
    println!("  监控指标:");
    println!("    • FPS: 帧率");
    println!("    • Frame Time: 帧时间");
    println!("    • Draw Calls: 绘制调用");
    println!("    • Triangles: 三角形数量");
    println!("    • Memory: 内存使用");
    println!("    • CPU Time: CPU时间");
    println!("    • GPU Time: GPU时间");
    println!("    • GC Time: 垃圾回收时间");
    println!();

    println!("✓ 性能警告阈值:");
    println!("    • FPS < 30: 红色警告");
    println!("    • FPS < 60: 黄色提示");
    println!("    • Draw Calls > 100: 黄色提示");
    println!("    • Draw Calls > 200: 红色警告");
    println!("    • Memory > 500MB: 黄色提示");
    println!("    • GC Time > 5ms: 红色警告");
    println!();

    println!("✓ 性能趋势分析:");
    println!("    • 最小/最大/平均帧率");
    println!("    • 帧率分布直方图");
    println!("    • 内存使用趋势");
    println!("    • 性能回归检测");
    println!();

    // 模拟实时监控
    println!("✓ 模拟实时监控 (10帧):");
    let mut fps_values = Vec::new();

    for frame in 0..10 {
        let fps = 55.0 + (frame as f32 % 20.0) - 5.0; // 模拟帧率波动
        fps_values.push(fps);

        let status = if fps < 30.0 {
            "❌"
        } else if fps < 60.0 {
            "⚠️ "
        } else {
            "✅"
        };

        println!("  帧 {}: {:.1} FPS {}", frame + 1, fps, status);
    }

    let avg_fps: f32 = fps_values.iter().sum::<f32>() / fps_values.len() as f32;
    let min_fps = fps_values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max_fps = fps_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

    println!();
    println!("✓ 统计摘要:");
    println!("  平均帧率: {:.1} FPS", avg_fps);
    println!("  最低帧率: {:.1} FPS", min_fps);
    println!("  最高帧率: {:.1} FPS", max_fps);
    println!("  帧率方差: {:.2}", calculate_variance(&fps_values));
    println!();
}

/// 计算性能评分
fn calculate_performance_score(metrics: &PerformanceMetrics) -> u32 {
    let mut score = 100u32;

    // FPS评分 (40分)
    if metrics.fps >= 60.0 {
        score += 0;
    } else if metrics.fps >= 45.0 {
        score -= 10;
    } else if metrics.fps >= 30.0 {
        score -= 25;
    } else {
        score -= 40;
    }

    // Draw Calls评分 (20分)
    if metrics.draw_calls <= 50 {
        score += 0;
    } else if metrics.draw_calls <= 100 {
        score -= 5;
    } else if metrics.draw_calls <= 200 {
        score -= 12;
    } else {
        score -= 20;
    }

    // 内存评分 (20分)
    if metrics.memory_mb <= 250.0 {
        score += 0;
    } else if metrics.memory_mb <= 500.0 {
        score -= 8;
    } else if metrics.memory_mb <= 1000.0 {
        score -= 15;
    } else {
        score -= 20;
    }

    // GC时间评分 (20分)
    if metrics.gc_time_ms <= 2.0 {
        score += 0;
    } else if metrics.gc_time_ms <= 5.0 {
        score -= 5;
    } else if metrics.gc_time_ms <= 10.0 {
        score -= 12;
    } else {
        score -= 20;
    }

    score.max(0)
}

/// 计算方差
fn calculate_variance(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    let mean = values.iter().sum::<f32>() / values.len() as f32;
    let variance = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f32>() / values.len() as f32;

    variance.sqrt()
}

/// 性能分析工作流示例
pub struct PerformanceAnalysisWorkflow {
    detector: BottleneckDetector,
    advisor: OptimizationAdvisor,
    monitoring_duration_secs: u32,
}

impl PerformanceAnalysisWorkflow {
    /// 创建新的分析工作流
    pub fn new() -> Self {
        Self {
            detector: BottleneckDetector::new(),
            advisor: OptimizationAdvisor::new(),
            monitoring_duration_secs: 30,
        }
    }

    /// 设置监控时长
    pub fn with_monitoring_duration(mut self, duration_secs: u32) -> Self {
        self.monitoring_duration_secs = duration_secs;
        self
    }

    /// 执行完整分析
    pub fn analyze(&mut self, metrics: &PerformanceMetrics) -> String {
        let mut report = String::new();

        report.push_str("═════════════════════════════════════════════════════\\n");
        report.push_str("         性能分析报告\\n");
        report.push_str("═════════════════════════════════════════════════════\\n\\n");

        // 当前性能
        report.push_str("📊 当前性能:\\n");
        report.push_str(&format!("  FPS: {:.1}\\n", metrics.fps));
        report.push_str(&format!("  Draw Calls: {}\\n", metrics.draw_calls));
        report.push_str(&format!("  三角形: {}\\n", metrics.triangles));
        report.push_str(&format!("  内存: {:.1} MB\\n", metrics.memory_mb));
        report.push_str(&format!("  GC时间: {:.2} ms\\n\\n", metrics.gc_time_ms));

        // 性能评分
        let score = calculate_performance_score(metrics);
        report.push_str(&format!("📈 性能评分: {}/100\\n\\n", score));

        // 优化建议
        let plan = self.advisor.analyze_and_suggest(metrics);

        report.push_str("💡 优化建议:\\n\\n");
        report.push_str(&format!("  总建议数: {}\\n", plan.suggestion_count));
        report.push_str(&format!("  高优先级: {}\\n", plan.high_priority_count));
        report.push_str(&format!("  预估提升: {:.1}%\\n", plan.total_estimated_improvement));
        report.push_str(&format!("  预估耗时: {:.1} 小时\\n\\n", plan.total_estimated_time_hours));

        // Top 3建议
        report.push_str("🔑 关键优化:\\n");
        for (i, suggestion) in plan.suggestions.iter().take(3).enumerate() {
            report.push_str(&format!("\\n  {}. {} ({})\\n",
                i + 1,
                suggestion.title,
                suggestion.priority.as_str()
            ));
            report.push_str(&format!("     预估提升: {:.1}%\\n", suggestion.estimated_improvement));
            report.push_str(&format!("     难度: {}/10\\n", suggestion.difficulty));

            if !suggestion.implementation_steps.is_empty() {
                report.push_str("     步骤:\\n");
                for (j, step) in suggestion.implementation_steps.iter().take(3).enumerate() {
                    report.push_str(&format!("       {}. {}\\n", j + 1, step));
                }
            }
        }

        report.push_str("\\n═════════════════════════════════════════════════════\\n");

        report
    }
}

impl Default for PerformanceAnalysisWorkflow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_score_calculation() {
        let good_metrics = PerformanceMetrics {
            fps: 60.0,
            frame_time: 16.67,
            draw_calls: 50,
            triangles: 50000,
            memory_mb: 200.0,
            texture_memory_mb: 100.0,
            gc_time_ms: 1.0,
            cpu_time: 10.0,
            gpu_time: 12.0,
        };

        let score = calculate_performance_score(&good_metrics);
        assert!(score >= 90, "优秀性能应该得到高分");

        let bad_metrics = PerformanceMetrics {
            fps: 20.0,
            frame_time: 50.0,
            draw_calls: 300,
            triangles: 500000,
            memory_mb: 1500.0,
            texture_memory_mb: 800.0,
            gc_time_ms: 20.0,
            cpu_time: 40.0,
            gpu_time: 50.0,
        };

        let score = calculate_performance_score(&bad_metrics);
        assert!(score <= 40, "差性能应该得到低分");
    }

    #[test]
    fn test_optimization_plan_generation() {
        let mut advisor = OptimizationAdvisor::new();

        let problem_metrics = PerformanceMetrics {
            fps: 25.0,
            frame_time: 40.0,
            draw_calls: 200,
            triangles: 200000,
            memory_mb: 700.0,
            texture_memory_mb: 350.0,
            gc_time_ms: 10.0,
            cpu_time: 30.0,
            gpu_time: 35.0,
        };

        let plan = advisor.analyze_and_suggest(&problem_metrics);

        assert!(!plan.suggestions.is_empty(), "应该生成优化建议");
        assert!(plan.total_estimated_improvement > 0.0, "应该有预估性能提升");
        assert!(plan.suggestion_count > 0, "应该有建议数量统计");
    }

    #[test]
    fn test_workflow_report_generation() {
        let mut workflow = PerformanceAnalysisWorkflow::new();

        let metrics = PerformanceMetrics {
            fps: 45.0,
            frame_time: 22.2,
            draw_calls: 120,
            triangles: 150000,
            memory_mb: 450.0,
            texture_memory_mb: 200.0,
            gc_time_ms: 6.0,
            cpu_time: 18.0,
            gpu_time: 22.0,
        };

        let report = workflow.analyze(&metrics);

        assert!(report.contains("性能分析报告"));
        assert!(report.contains("优化建议"));
    }
}
