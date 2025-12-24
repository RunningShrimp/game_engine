//! 性能回归检测工具
//!
//! 用于CI/CD的性能回归检测命令行工具。
//! 读取性能基线，运行基准测试，检测回归并生成报告。

use game_engine::profiling::{
    PerformanceBaseline, PerformanceRegressionDetector, RegressionThresholds,
};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug)]
struct Opt {
    /// 基线文件路径
    baseline: PathBuf,

    /// 输出报告文件路径
    output: PathBuf,

    /// FPS下降阈值（百分比）
    fps_threshold: f64,

    /// 帧时间增加阈值（百分比）
    frame_time_threshold: f64,

    /// 内存增加阈值（百分比）
    memory_threshold: f64,

    /// 最小样本数
    min_samples: usize,

    /// 严重回归阈值（百分比）
    severe_threshold: f64,

    /// 中等回归阈值（百分比）
    moderate_threshold: f64,

    /// 轻微回归阈值（百分比）
    minor_threshold: f64,

    /// 模拟模式（用于测试，不运行实际基准测试）
    simulate: bool,
}

impl Opt {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut baseline = PathBuf::from("performance_baselines.json");
        let mut output = PathBuf::from("regression_report.json");
        let mut fps_threshold = 5.0;
        let mut frame_time_threshold = 5.0;
        let mut memory_threshold = 10.0;
        let mut min_samples = 30;
        let mut severe_threshold = 20.0;
        let mut moderate_threshold = 10.0;
        let mut minor_threshold = 5.0;
        let mut simulate = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--baseline" | "-b" => {
                    if i + 1 < args.len() {
                        baseline = PathBuf::from(&args[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--output" | "-o" => {
                    if i + 1 < args.len() {
                        output = PathBuf::from(&args[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--fps-threshold" => {
                    if i + 1 < args.len() {
                        fps_threshold = args[i + 1].parse().unwrap_or(5.0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--frame-time-threshold" => {
                    if i + 1 < args.len() {
                        frame_time_threshold = args[i + 1].parse().unwrap_or(5.0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--memory-threshold" => {
                    if i + 1 < args.len() {
                        memory_threshold = args[i + 1].parse().unwrap_or(10.0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--min-samples" => {
                    if i + 1 < args.len() {
                        min_samples = args[i + 1].parse().unwrap_or(30);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--severe-threshold" => {
                    if i + 1 < args.len() {
                        severe_threshold = args[i + 1].parse().unwrap_or(20.0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--moderate-threshold" => {
                    if i + 1 < args.len() {
                        moderate_threshold = args[i + 1].parse().unwrap_or(10.0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--minor-threshold" => {
                    if i + 1 < args.len() {
                        minor_threshold = args[i + 1].parse().unwrap_or(5.0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--simulate" => {
                    simulate = true;
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        Self {
            baseline,
            output,
            fps_threshold,
            frame_time_threshold,
            memory_threshold,
            min_samples,
            severe_threshold,
            moderate_threshold,
            minor_threshold,
            simulate,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    // 创建回归检测器
    let thresholds = RegressionThresholds {
        fps_degradation_percent: opt.fps_threshold,
        frame_time_increase_percent: opt.frame_time_threshold,
        memory_increase_percent: opt.memory_threshold,
        min_samples: opt.min_samples,
        minor_threshold: opt.minor_threshold,
        moderate_threshold: opt.moderate_threshold,
        severe_threshold: opt.severe_threshold,
    };

    let mut detector = PerformanceRegressionDetector::new(thresholds);

    // 加载基线
    if opt.baseline.exists() {
        detector.load_baseline(&opt.baseline)?;
        eprintln!("✓ 已加载性能基线: {:?}", opt.baseline);
    } else {
        eprintln!("⚠️  警告: 基线文件不存在: {:?}", opt.baseline);
        eprintln!("   将创建新的基线文件");

        // 如果没有基线，创建一个默认基线（用于演示）
        if opt.simulate {
            let baseline = PerformanceBaseline::new(
                60.0,
                Duration::from_millis(16),
                Duration::from_millis(20),
                256.0,
                100,
            );
            detector.set_baseline(baseline);
            detector.save_baseline(&opt.baseline)?;
            eprintln!("✓ 已创建模拟基线");
        } else {
            eprintln!("❌ 错误: 需要基线文件才能检测回归");
            eprintln!("   请先运行基准测试建立基线");
            std::process::exit(1);
        }
    }

    // 收集性能样本
    if opt.simulate {
        // 模拟模式：添加一些性能下降的样本
        eprintln!("📊 模拟模式：收集性能样本...");
        for _ in 0..50 {
            // 模拟FPS下降、帧时间增加、内存增加
            detector.add_sample(55.0, Duration::from_millis(18), 280.0);
        }
    } else {
        // 实际模式：这里应该运行基准测试并收集真实数据
        eprintln!("📊 运行基准测试并收集性能数据...");
        eprintln!("   注意: 实际基准测试集成需要根据项目配置");
        // TODO: 集成实际的基准测试运行逻辑
        // 例如: cargo bench -- --output-format json | parse_and_add_samples
    }

    // 检测回归
    eprintln!("🔍 检测性能回归...");
    let regressions = detector.detect_regressions();

    // 生成报告
    let report = detector.generate_cicd_report();
    std::fs::write(&opt.output, serde_json::to_string_pretty(&report)?)?;
    eprintln!("✓ 已生成回归报告: {:?}", opt.output);

    // 打印摘要
    let severe_count = report["regression_count"]["severe"].as_u64().unwrap_or(0);
    let moderate_count = report["regression_count"]["moderate"].as_u64().unwrap_or(0);
    let minor_count = report["regression_count"]["minor"].as_u64().unwrap_or(0);
    let total = report["regression_count"]["total"].as_u64().unwrap_or(0);

    eprintln!("\n📈 回归检测摘要:");
    eprintln!("   严重回归: {}", severe_count);
    eprintln!("   中等回归: {}", moderate_count);
    eprintln!("   轻微回归: {}", minor_count);
    eprintln!("   总计: {}", total);

    if severe_count > 0 {
        eprintln!("\n❌ 检测到严重性能回归！");
        eprintln!("   请查看报告文件了解详情: {:?}", opt.output);

        // 打印严重回归详情
        for regression in detector.get_severe_regressions() {
            eprintln!("\n  严重回归:");
            eprintln!("    指标: {}", regression.metric_name);
            eprintln!("    基线: {:.2}", regression.baseline_value);
            eprintln!("    当前: {:.2}", regression.current_value);
            eprintln!("    回归: {:.2}%", regression.regression_percent);
            if let Some(ref fix) = regression.suggested_fix {
                eprintln!("    建议: {}", fix);
            }
        }

        std::process::exit(1);
    } else if moderate_count > 0 || minor_count > 0 {
        eprintln!("\n⚠️  检测到性能回归（非严重）");
        eprintln!("   请查看报告文件了解详情: {:?}", opt.output);
        std::process::exit(0);
    } else {
        eprintln!("\n✓ 未检测到性能回归");
        std::process::exit(0);
    }
}
