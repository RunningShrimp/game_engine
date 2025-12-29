// Phase 4 Integrated Performance Framework Demo
// 这是一个完整示例，展示如何使用Phase 4的所有新模块

use std::collections::HashMap;
use std::time::Duration;

/// 完整的性能优化工作流示例
pub fn phase4_complete_workflow() {
    println!("═══════════════════════════════════════════════════════");
    println!("Phase 4: Advanced Performance Analysis & CI/CD Framework");
    println!("═══════════════════════════════════════════════════════\n");

    // 1. 帧级分析
    println!("1️⃣  Frame-Level Performance Analysis");
    println!("────────────────────────────────────");
    frame_analysis_example();

    // 2. 瓶颈检测
    println!("\n2️⃣  Bottleneck Detection");
    println!("────────────────────────");
    bottleneck_detection_example();

    // 3. 可视化仪表板
    println!("\n3️⃣  Visualization Dashboard");
    println!("──────────────────────────");
    visualization_example();

    // 4. 回归测试
    println!("\n4️⃣  Regression Testing");
    println!("─────────────────────");
    regression_testing_example();

    // 5. CI/CD 集成
    println!("\n5️⃣  CI/CD Pipeline Management");
    println!("────────────────────────────");
    cicd_example();

    // 6. 性能优化验证
    println!("\n6️⃣  Performance Optimization Validation");
    println!("──────────────────────────────────────");
    optimization_validation_example();
}

fn frame_analysis_example() {
    use crate::performance::*;

    let mut analyzer = FrameAnalyzer::new(300);

    // 模拟10帧
    for frame_num in 0..10 {
        let frame_duration = Duration::from_millis(16);
        analyzer.start_frame(frame_num, frame_duration);

        // 添加各个阶段的性能数据
        analyzer.add_phase(PhaseMetrics::new("physics", Duration::from_micros(3000))).ok();
        analyzer.add_phase(PhaseMetrics::new("rendering", Duration::from_micros(11000))).ok();
        analyzer.add_phase(PhaseMetrics::new("ai", Duration::from_micros(1000))).ok();

        analyzer.end_frame().ok();
    }

    println!("📊 Frame Analysis Results:");
    println!("  • Total frames: {}", analyzer.get_frame_count());
    println!("  • Average FPS: {:.1}", analyzer.average_fps());

    if let Some((min, max)) = analyzer.fps_range() {
        println!("  • FPS Range: {:.1} - {:.1}", min, max);
    }

    if let Some(p95) = analyzer.frame_time_percentile_95() {
        println!("  • 95th percentile frame time: {:.2}ms", p95.as_secs_f64() * 1000.0);
    }

    if let Some(variance) = analyzer.phase_variation_coefficient("rendering") {
        println!("  • Rendering stability (CV): {:.3}", variance);
    }
}

fn bottleneck_detection_example() {
    use crate::performance::*;

    let mut detector = BottleneckDetector::new();

    // 记录稳定的物理计算
    for i in 0..50 {
        detector.record_phase("physics", Duration::from_micros(5000 + i % 100));
    }

    // 记录高度不稳定的渲染
    for i in 0..50 {
        detector.record_phase("rendering", Duration::from_micros(10000 + i * 1000));
    }

    println!("🎯 Bottleneck Detection Results:");

    let critical_bottlenecks = detector.get_critical_bottlenecks(5);
    if !critical_bottlenecks.is_empty() {
        println!("  • Critical bottlenecks found: {}", critical_bottlenecks.len());
        for bottleneck in critical_bottlenecks {
            println!("    - {}", bottleneck.description());
            println!("      Recommendation: {}", bottleneck.recommendation);
        }
    }

    let gpu_bottlenecks = detector.get_gpu_bottlenecks();
    println!("  • GPU bottlenecks: {}", gpu_bottlenecks.len());

    let cpu_bottlenecks = detector.get_cpu_bottlenecks();
    println!("  • CPU bottlenecks: {}", cpu_bottlenecks.len());
}

fn visualization_example() {
    use crate::performance::*;

    let layout = DashboardLayout::new("Performance Dashboard", 2);
    let mut dashboard = VisualizationDashboard::new(layout);

    // 创建图表
    let fps_chart_idx = dashboard.create_chart("FPS", ChartType::LineChart);
    let memory_chart_idx = dashboard.create_chart("Memory", ChartType::LineChart);

    // 设置仪表值
    dashboard.set_gauge("Current FPS", 60.0);
    dashboard.set_gauge("Memory (MB)", 256.0);
    dashboard.set_gauge("GPU Util", 85.0);

    // 添加数据点
    for i in 0..100 {
        let fps = 60.0 + (i as f64 * 0.1);
        dashboard.add_data_to_chart(fps_chart_idx, format!("frame_{}", i), fps, i as u64).ok();
    }

    for i in 0..100 {
        let memory = 256.0 + (i as f64 * 2.0);
        dashboard.add_data_to_chart(memory_chart_idx, format!("sample_{}", i), memory, i as u64).ok();
    }

    println!("📈 Dashboard Summary:");
    let summary = dashboard.get_summary();
    println!("  • Total charts: {}", summary.total_charts);
    println!("  • Total data points: {}", summary.total_data_points);

    for chart_stat in &summary.chart_stats {
        println!("  • {} [{}]", chart_stat.name, chart_stat.point_count);
        if let Some(avg) = chart_stat.average {
            println!("    Average: {:.2}", avg);
        }
        if let Some(max) = chart_stat.max {
            println!("    Max: {:.2}", max);
        }
    }

    println!("\n📊 ASCII Dashboard:");
    println!("{}", dashboard.render_ascii());
}

fn regression_testing_example() {
    use crate::performance::*;

    let mut suite = RegressionTestSuite::new();

    // 注册基线
    suite.register_baselines(vec![
        PerformanceBaseline::new("fps", 60.0, "fps"),
        PerformanceBaseline::new("latency", 16.0, "ms"),
        PerformanceBaseline::new("memory", 256.0, "MB"),
    ]);

    // 测试当前性能
    let fps_result = suite.test_metric("fps", 58.0).expect("Test: operation should succeed");
    let latency_result = suite.test_metric("latency", 18.0).expect("Test: operation should succeed");
    let memory_result = suite.test_metric("memory", 280.0).expect("Test: operation should succeed");

    println!("📋 Regression Test Results:");
    println!("  • FPS: {}", if fps_result.passed() { "✓ PASSED" } else { "✗ FAILED" });
    println!("  • Latency: {}", if latency_result.warned() { "⚠️  WARNING" } else { "✓ PASSED" });
    println!("  • Memory: {}", if memory_result.warned() { "⚠️  WARNING" } else { "✓ PASSED" });

    let summary = suite.get_summary();
    println!("\n  Regression Summary:");
    println!("    - Total: {}", summary.total_tests);
    println!("    - Passed: {}", summary.passed);
    println!("    - Warned: {}", summary.warned);
    println!("    - Failed: {}", summary.failed);
    println!("    - Pass Rate: {:.1}%", summary.pass_rate);
}

fn cicd_example() {
    use crate::performance::*;

    let mut manager = CicdManager::new();

    // 创建流水线
    let pipeline_id = manager.create_pipeline("abc123def456", "main");

    if let Some(pipeline) = manager.get_pipeline_mut(&pipeline_id) {
        pipeline.add_stage(CicdStage::Checkout);
        pipeline.add_stage(CicdStage::Build);
        pipeline.add_stage(CicdStage::UnitTest);
        pipeline.add_stage(CicdStage::BenchmarkTest);
        pipeline.add_stage(CicdStage::RegressionTest);

        pipeline.start();

        // 模拟阶段执行
        pipeline.update_stage(CicdStage::Checkout, StageStatus::Passed, "Repository ready".into()).ok();
        pipeline.update_stage(CicdStage::Build, StageStatus::Passed, "Build successful".into()).ok();
        pipeline.update_stage(CicdStage::UnitTest, StageStatus::Passed, "All unit tests passed".into()).ok();
        pipeline.update_stage(CicdStage::BenchmarkTest, StageStatus::Passed, "Benchmarks OK".into()).ok();
        pipeline.update_stage(CicdStage::RegressionTest, StageStatus::Passed, "No regressions".into()).ok();

        pipeline.complete();

        println!("🔄 CI/CD Pipeline Report:");
        println!("{}", pipeline.generate_report());
    }

    // 统计
    let stats = manager.get_statistics();
    println!("📊 Pipeline Statistics:");
    println!("  • Total pipelines: {}", stats.total_pipelines);
    println!("  • Success rate: {:.1}%", stats.success_rate);
}

fn optimization_validation_example() {
    use crate::performance::*;

    let mut suite = PerformanceValidationSuite::new();

    // 记录优化结果
    suite.record_result(
        OptimizationGoal::new("FPS", 60.0, 120.0, "fps"),
        60.0,
        100.0,
    );

    suite.record_result(
        OptimizationGoal::new("Latency", 16.0, 8.0, "ms"),
        16.0,
        10.0,
    );

    suite.record_result(
        OptimizationGoal::new("Memory", 512.0, 256.0, "MB"),
        512.0,
        350.0,
    );

    // 记录GPU性能比较
    suite.record_comparison(CpuGpuComparison::new(
        "Physics Simulation",
        10000,
        Duration::from_millis(50),
        Duration::from_millis(10),
        Duration::from_millis(2),
    ));

    suite.record_comparison(CpuGpuComparison::new(
        "Particle System",
        50000,
        Duration::from_millis(100),
        Duration::from_millis(15),
        Duration::from_millis(3),
    ));

    println!("✅ Optimization Validation Report:");
    println!("{}", suite.generate_report());
}

// 完整集成测试示例
#[cfg(test)]
mod integration_tests {
    use crate::performance::*;
    use std::time::Duration;

    #[test]
    fn test_phase4_integration() {
        // 帧分析
        let mut analyzer = FrameAnalyzer::new(300);
        analyzer.start_frame(0, Duration::from_millis(16));
        analyzer.add_phase(PhaseMetrics::new("test", Duration::from_micros(1000))).ok();
        analyzer.end_frame().ok();

        assert_eq!(analyzer.get_frame_count(), 1);

        // 瓶颈检测
        let mut detector = BottleneckDetector::new();
        detector.record_phase("test", Duration::from_micros(1000));
        assert_eq!(detector.phase_count(), 1);

        // 仪表板
        let layout = DashboardLayout::new("Test", 1);
        let mut dashboard = VisualizationDashboard::new(layout);
        dashboard.set_gauge("test", 100.0);
        assert_eq!(dashboard.get_gauge("test"), Some(100.0));

        // 回归测试
        let mut suite = RegressionTestSuite::new();
        suite.register_baseline(PerformanceBaseline::new("fps", 60.0, "fps"));
        let result = suite.test_metric("fps", 60.5).expect("Test: operation should succeed");
        assert!(result.passed());

        // CI/CD
        let mut manager = CicdManager::new();
        let id = manager.create_pipeline("test", "main");
        assert!(manager.get_pipeline(&id).is_some());

        // 优化验证
        let mut validation = PerformanceValidationSuite::new();
        validation.record_result(
            OptimizationGoal::new("fps", 60.0, 120.0, "fps"),
            60.0,
            90.0,
        );
        assert_eq!(validation.result_count(), 1);
    }
}
