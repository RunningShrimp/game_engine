//! 使用构建管理器进行构建的示例
//!
//! 演示如何使用BuildManager进行增量构建、并行构建和进度显示

use game_engine::build::{BuildConfig, BuildManager, BuildProfile};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 使用构建管理器进行构建");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 创建构建配置
    let config = BuildConfig {
        profile: BuildProfile::Release,
        incremental: true,
        max_parallel: num_cpus::get(),
        show_progress: true,
        packages: None, // 构建所有包
        target: None,
        features: Vec::new(),
        all_features: false,
    };

    // 创建构建管理器
    let manager = BuildManager::new(config);

    // 获取进度追踪器
    let progress = manager.progress();

    // 启动进度显示任务
    let progress_task = tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(500)).await;
            let prog = progress.lock().unwrap();
            let completed = prog.completed;
            let total = prog.total;
            let current = prog.current_packages.clone();
            drop(prog);

            if total > 0 {
                let percentage = (completed as f32 / total as f32) * 100.0;
                print!("\r进度: [{}/{}] {:.1}%", completed, total, percentage);
                if !current.is_empty() {
                    print!(" | 正在构建: {}", current.join(", "));
                }
                std::io::Write::flush(&mut std::io::stdout()).unwrap();

                if completed >= total {
                    println!();
                    break;
                }
            }
        }
    });

    // 执行构建
    let build_result = manager.build().await;

    // 等待进度显示任务完成
    let _ = progress_task.await;

    // 处理结果
    match build_result {
        Ok(stats) => {
            println!("\n✅ 构建完成！");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("总时间: {:.2}秒", stats.total_time);
            println!("成功: {} | 失败: {}", stats.success_count, stats.failure_count);
            println!("并行度: {}", stats.parallelism);
            Ok(())
        }
        Err(e) => {
            eprintln!("\n❌ 构建失败: {}", e);
            Err(e.into())
        }
    }
}
