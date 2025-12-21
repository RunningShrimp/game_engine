//  性能回归检测工具
// 
//  读取 criterion 基准测试结果并与基线进行比较
//  如果检测到性能回归，返回非零退出码

use std::fs;
use std::path::Path;
use std::process;
use serde_json::Value;

/// 从 criterion 结果目录读取基准测试结果
fn read_criterion_results(bench_name: &str, results_dir: &Path) -> Option<f64> {
    let bench_dir = results_dir.join(bench_name);
    
    // criterion 将结果存储在 base/estimates.json
    let estimates_path = bench_dir.join("base").join("estimates.json");
    
    if !estimates_path.exists() {
        eprintln!("Warning: No estimates found for {}", bench_name);
        return None;
    }
    
    let content = match fs::read_to_string(&estimates_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", estimates_path.display(), e);
            return None;
        }
    };
    
    let json: Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            return None;
        }
    };
    
    // 提取 mean.point_estimate (纳秒)
    json.get("mean")?
        .get("point_estimate")?
        .as_f64()
}

/// 比较两个基准测试结果
fn compare_results(baseline: f64, current: f64, threshold_warning: f64, threshold_critical: f64) -> (f64, bool) {
    if baseline == 0.0 || current == 0.0 {
        return (0.0, false);
    }
    
    let regression_ratio = (current - baseline) / baseline;
    let regression_percent = regression_ratio * 100.0;
    
    let is_critical = regression_percent > threshold_critical;
    
    (regression_percent, is_critical)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <baseline_dir> <current_dir> [threshold_warning] [threshold_critical]", args[0]);
        eprintln!("  baseline_dir: Path to baseline criterion results");
        eprintln!("  current_dir: Path to current criterion results");
        eprintln!("  threshold_warning: Warning threshold percentage (default: 10.0)");
        eprintln!("  threshold_critical: Critical threshold percentage (default: 20.0)");
        process::exit(1);
    }
    
    let baseline_dir = Path::new(&args[1]);
    let current_dir = Path::new(&args[2]);
    let threshold_warning = args.get(3)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(10.0);
    let threshold_critical = args.get(4)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(20.0);
    
    if !baseline_dir.exists() {
        eprintln!("Warning: Baseline directory does not exist: {}", baseline_dir.display());
        eprintln!("Skipping comparison (this is normal for first run)");
        process::exit(0);
    }
    
    if !current_dir.exists() {
        eprintln!("Error: Current results directory does not exist: {}", current_dir.display());
        process::exit(1);
    }
    
    // criterion 结果存储在 target/criterion/<bench_name>/
    // 我们需要遍历所有基准测试
    let criterion_dir = current_dir.join("target").join("criterion");
    
    if !criterion_dir.exists() {
        eprintln!("Error: Criterion results directory not found: {}", criterion_dir.display());
        process::exit(1);
    }
    
    let mut has_regression = false;
    let mut has_critical_regression = false;
    
    println!("Comparing performance benchmarks...\n");
    println!("{:<40} {:>15} {:>15} {:>15} {:>10}", 
             "Benchmark", "Baseline (ns)", "Current (ns)", "Change (%)", "Status");
    println!("{}", "-".repeat(95));
    
    // 遍历所有基准测试组
    if let Ok(entries) = fs::read_dir(&criterion_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let bench_name = entry.file_name();
                let bench_name_str = bench_name.to_string_lossy();
                
                let baseline_result = read_criterion_results(&bench_name_str, 
                    &baseline_dir.join("target").join("criterion"));
                let current_result = read_criterion_results(&bench_name_str, &criterion_dir);
                
                if let (Some(baseline), Some(current)) = (baseline_result, current_result) {
                    let (regression_percent, is_critical) = compare_results(
                        baseline, current, threshold_warning, threshold_critical
                    );
                    
                    let status = if is_critical {
                        has_critical_regression = true;
                        "🔴 CRITICAL"
                    } else if regression_percent > threshold_warning {
                        has_regression = true;
                        "🟡 WARNING"
                    } else if regression_percent < -5.0 {
                        "🟢 IMPROVED"
                    } else {
                        "✅ OK"
                    };
                    
                    println!("{:<40} {:>15.2} {:>15.2} {:>14.2}% {:>10}",
                        bench_name_str, baseline, current, regression_percent, status);
                }
            }
        }
    }
    
    println!();
    
    if has_critical_regression {
        eprintln!("❌ Critical performance regression detected!");
        eprintln!("Some benchmarks are more than {:.1}% slower than baseline.", threshold_critical);
        process::exit(1);
    } else if has_regression {
        eprintln!("⚠️  Performance regression detected!");
        eprintln!("Some benchmarks are more than {:.1}% slower than baseline.", threshold_warning);
        eprintln!("This is a warning, not a failure.");
        process::exit(0);
    } else {
        println!("✅ All benchmarks are within acceptable performance range.");
        process::exit(0);
    }
}

