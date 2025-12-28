// 性能基准测试示例 - 简化版
//
// 这个示例展示了如何使用基准测试框架进行性能测试

use game_engine::performance::benchmarking::{Benchmark, PerformanceRegression};
use std::time::Duration;

fn expensive_operation() {
    std::thread::sleep(Duration::from_millis(5));
}

fn physics_simulation_step() {
    std::thread::sleep(Duration::from_micros(500));
}

fn render_frame() {
    std::thread::sleep(Duration::from_millis(1));
}

fn main() {
    println!("=== 性能基准测试示例 ===\n");

    // 创建基准测试器
    let mut bench = Benchmark::new();

    // 1. 基础基准测试
    println!("1. 基础基准测试");
    println!("----------------------");
    let result = bench.run("expensive_operation", 10, || {
        expensive_operation();
    });
    println!("执行时间: {:.3}ms", result.avg_duration.as_secs_f64() * 1000.0);
    println!("最小时间: {:.3}ms", result.min_duration.as_secs_f64() * 1000.0);
    println!("最大时间: {:.3}ms", result.max_duration.as_secs_f64() * 1000.0);

    // 2. 物理模拟测试
    println!("\n2. 物理模拟测试");
    println!("----------------------");
    let physics_result = bench.run("physics_step", 100, || {
        physics_simulation_step();
    });
    println!("物理步进时间: {:.3}μs", physics_result.avg_duration.as_secs_f64() * 1_000_000.0);

    // 3. 渲染测试
    println!("\n3. 渲染帧测试");
    println!("----------------------");
    let render_result = bench.run("render_frame", 100, || {
        render_frame();
    });
    println!("渲染帧时间: {:.3}μs", render_result.avg_duration.as_secs_f64() * 1_000_000.0);

    // 4. 性能回归检测
    println!("\n4. 性能回归检测");
    println!("----------------------");
    let mut regression = PerformanceRegression::new(0.2); // 20%阈值

    // 设置基线
    regression.set_baseline("render_frame", Duration::from_millis(1));

    // 检查当前性能
    let current_performance = render_result.avg_duration;
    if regression.check_regression("render_frame", current_performance) {
        println!("警告: 检测到性能回退!");
    } else {
        println!("性能正常，无回归。");
    }

    if let Some(percent) = regression.get_regression_percent("render_frame", current_performance) {
        println!("性能变化: {:.1}%", percent);
    }

    // 5. 打印所有结果
    println!("\n5. 完整结果");
    println!("----------------------");
    bench.print_results();

    println!("\n示例完成!");
}
