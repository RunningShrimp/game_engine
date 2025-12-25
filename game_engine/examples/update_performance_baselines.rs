//! 更新性能基线示例程序
//!
//! 运行所有基准测试并更新performance_baselines.json文件

use game_engine::performance::benchmarking::baseline_updater::BaselineUpdater;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 游戏引擎性能基线更新工具");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 确定基线文件和结果目录
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let baseline_file = project_root.join("performance_baselines.json");
    let results_dir = project_root.join("target").join("benchmark_results");

    // 创建基线更新器
    let updater = BaselineUpdater::new(&baseline_file, &results_dir);

    // 运行更新
    updater.update_baselines()?;

    println!("");
    println!("✅ 完成！基线文件已更新: {:?}", baseline_file);
    println!("");
    println!("📊 基线摘要:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 读取并显示更新的基线
    if baseline_file.exists() {
        let content = std::fs::read_to_string(&baseline_file)?;
        let baselines: serde_json::Value = serde_json::from_str(&content)?;
        
        if let Some(benchmarks) = baselines.get("benchmarks").and_then(|b| b.as_object()) {
            for (name, _) in benchmarks {
                println!("  • {}", name);
            }
        }
    }

    Ok(())
}

