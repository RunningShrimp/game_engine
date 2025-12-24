use game_engine::performance::{
    BenchmarkResult, BenchmarkSuite, DefaultThresholds, Threshold, ThresholdType,
};
use std::thread;
use std::time::Duration;

fn expensive_operation() {
    thread::sleep(Duration::from_millis(5));
}

fn physics_simulation_step() {
    thread::sleep(Duration::from_micros(500));
}

fn render_frame() {
    thread::sleep(Duration::from_millis(1));
}

fn main() {
    println!("=== 性能基准测试示例 ===\n");

    let mut suite = BenchmarkSuite::new();

    println!("1. 基础基准测试");
    println!("----------------------");
    suite.run_benchmark("expensive_operation", || {
        expensive_operation();
    });

    if let Some(result) = suite.get_benchmark().get_result("expensive_operation") {
        println!("执行时间: {:.2} {}", result.current_value, result.unit);
        println!("基准时间: {:.2} {}", result.baseline, result.unit);
        println!("性能变化: {:.2}%", result.regression_percent);
        println!(
            "状态: {}",
            if result.has_regression {
                "回退"
            } else {
                "正常"
            }
        );
    }

    println!("\n2. 多次运行取平均");
    println!("----------------------");
    suite.run_benchmark("physics_step", || {
        physics_simulation_step();
    });

    if let Some(result) = suite.get_benchmark().get_result("physics_step") {
        println!("物理步进时间: {:.3} {}", result.current_value, result.unit);
    }

    println!("\n3. 自定义阈值配置");
    println!("----------------------");

    let absolute_threshold = Threshold::new_absolute(16.0);
    println!("绝对阈值示例:");
    println!("  基准: 16.0ms");
    println!("  超过此值即认为回退");

    let relative_threshold = Threshold::new_relative(10.0, 20.0);
    println!("\n相对阈值示例:");
    println!("  基准: 10.0ms");
    println!("  允许偏差: 20%");
    println!("  回退阈值: 12.0ms");

    println!("\n4. 默认阈值配置");
    println!("----------------------");
    let thresholds = DefaultThresholds::get_default_thresholds();
    println!("引擎默认配置的阈值:");
    for (name, threshold) in thresholds.iter().take(5) {
        println!(
            "  {}: 基准={:.2}ms, 偏差={}%",
            name, threshold.baseline, threshold.tolerance_percent
        );
    }

    println!("\n5. 性能改进检测");
    println!("----------------------");

    let mut benchmark = suite.get_benchmark().clone();
    benchmark.set_relative_threshold("render_improvement", 20.0, 0.0);

    benchmark.run(
        "render_improvement",
        || {
            render_frame();
        },
        "ms",
    );

    if let Some(result) = benchmark.get_result("render_improvement") {
        if result.has_improvement() {
            println!(
                "性能改进: {:.2}% ({} -> {} {})",
                (1.0 - result.current_value / result.baseline) * 100.0,
                result.baseline,
                result.current_value,
                result.unit
            );
        }
    }

    println!("\n6. 完整报告");
    println!("----------------------");
    println!("{}", suite.generate_report());

    if suite.has_any_regression() {
        println!("\n警告: 检测到性能回退!");
        println!("请检查最近的代码更改是否影响了性能。");
    } else {
        println!("\n所有性能指标正常。");
    }
}
