// C# 热重载示例
//
// 演示如何使用 C# 脚本热重载功能。

#[cfg(feature = "csharp")]
use {
    game_engine::scripting::csharp::{CSharpConfig, CSharpContext},
    std::{path::PathBuf, time::Duration},
};

#[cfg(feature = "csharp")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();

    tracing::info!("🔥 C# Hot Reload Example");
    tracing::info!("========================");

    // 创建 C# 上下文
    let mut ctx = CSharpContext::new();

    // 确保 .NET 运行时已初始化
    ctx.ensure_runtime_initialized()
        .map_err(|e| format!("Failed to initialize .NET runtime: {}", e))?;

    // 创建示例脚本目录
    let script_dir = PathBuf::from("./examples/scripts/csharp");
    std::fs::create_dir_all(&script_dir)?;

    // 创建示例脚本
    let example_script = script_dir.join("GameScript.cs");
    let script_content = r#"
using System;

public class GameScript {
    public static int Main() {
        Console.WriteLine("Hello from C# GameScript!");
        Console.WriteLine("Version: 1.0");
        return 42;
    }
}
"#;

    std::fs::write(&example_script, script_content)?;

    tracing::info!("Created example script: {}", example_script.display());

    // 启用热重载
    tracing::info!("Enabling hot reload...");
    ctx.enable_hot_reload(
        vec![script_dir.clone()],
        100, // 100ms 防抖动
    )?;

    // 主循环
    let mut iteration = 0;
    loop {
        iteration += 1;

        tracing::info!("--- Iteration {} ---", iteration);

        // 检查热重载
        match ctx.check_hot_reload() {
            Ok(reloaded) => {
                if !reloaded.is_empty() {
                    tracing::info!("🔄 Reloaded {} scripts:", reloaded.len());
                    for path in &reloaded {
                        tracing::info!("  - {}", path.display());
                    }

                    // 执行重新加载的脚本
                    if let Some(script_path) = reloaded.first() {
                        let script_name =
                            script_path.file_stem().and_then(|s| s.to_str()).unwrap_or("script");

                        match ctx.execute(script_name, None) {
                            game_engine::scripting::ScriptResult::Success(value) => {
                                tracing::info!("✅ Script executed successfully: {:?}", value);
                            }
                            game_engine::scripting::ScriptResult::Error(e) => {
                                tracing::error!("❌ Script execution error: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Hot reload error: {}", e);
            }
        }

        // 每10次循环修改一次脚本
        if iteration % 10 == 0 {
            tracing::info!("Modifying script to test hot reload...");

            let new_version = iteration / 10;
            let new_content = format!(
                r#"
using System;

public class GameScript {{
    public static int Main() {{
        Console.WriteLine("Hello from C# GameScript!");
        Console.WriteLine("Version: {}", new_version);
        Console.WriteLine("Auto-reloaded at iteration {}", iteration);
        return {};
    }}
}}
"#,
                new_version * 10
            );

            std::fs::write(&example_script, new_content)?;
            tracing::info!("✏️  Script updated - watch for hot reload!");
        }

        // 运行30次后退出
        if iteration >= 30 {
            tracing::info!("✅ Demo complete");
            break;
        }

        // 等待1秒
        std::thread::sleep(Duration::from_secs(1));
    }

    // 清理
    tracing::info!("Cleaning up...");
    ctx.disable_hot_reload();

    // 删除临时文件
    let _ = std::fs::remove_file(&example_script);

    tracing::info!("Done!");

    Ok(())
}

#[cfg(not(feature = "csharp"))]
fn main() {
    eprintln!("This example requires the 'csharp' feature.");
    eprintln!("Run with: cargo run --example csharp_hot_reload_example --features csharp");
}
