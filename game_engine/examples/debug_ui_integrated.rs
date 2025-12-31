//! 调试UI完整集成示例
//!
//! 演示如何在完整的游戏引擎中集成和使用DebugUI。

use bevy_ecs::prelude::*;
use game_engine::{
    core::engine::Engine,
    debug::DebugUI,
    ecs::{Transform, Velocity},
};
use std::time::{Duration, Instant};

/// 调试UI集成示例
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Debug UI Integration Example ===\n");

    // 运行基础示例
    run_basic_example()?;

    // 运行性能监控示例
    run_performance_example()?;

    // 运行控制台示例
    run_console_example()?;

    // 运行完整引擎示例
    run_full_engine_example().await?;

    println!("\n=== All examples completed successfully! ===");

    Ok(())
}

/// 基础使用示例
fn run_basic_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Basic DebugUI Example");

    let mut debug_ui = DebugUI::new();

    // 添加日志
    debug_ui.log("DebugUI initialized".to_string());
    debug_ui.log("Basic example started".to_string());

    // 切换面板
    debug_ui.toggle_panel("entities");
    debug_ui.toggle_panel("performance");

    println!("   ✓ DebugUI created and configured");
    println!("   ✓ Logs added: {}", debug_ui.console_panel().log_count());

    Ok(())
}

/// 性能监控示例
fn run_performance_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. Performance Monitoring Example");

    let mut debug_ui = DebugUI::new();
    let perf_panel = debug_ui.performance_panel();

    // 模拟100帧，性能逐渐下降
    for frame in 0..100 {
        let frame_time = if frame < 30 {
            0.016 // 60 FPS
        } else if frame < 60 {
            0.033 // 30 FPS
        } else {
            0.050 // 20 FPS
        };

        perf_panel.update_metrics(frame_time, frame);

        if frame % 10 == 0 {
            perf_panel.update_draw_calls(50 + (frame % 20) as usize, 10000 + frame * 100);
        }
    }

    // 获取统计信息
    if let Some(avg_fps) = perf_panel.calculate_average_fps() {
        println!("   ✓ Average FPS: {:.1}", avg_fps);
    }

    if let Some(min_fps) = perf_panel.calculate_min_fps() {
        println!("   ✓ Min FPS: {:.1}", min_fps);
    }

    if let Some(max_fps) = perf_panel.calculate_max_fps() {
        println!("   ✓ Max FPS: {:.1}", max_fps);
    }

    Ok(())
}

/// 控制台示例
fn run_console_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n3. Console Panel Example");

    let console = debug_ui.console_panel();

    // 添加不同级别的日志
    console.add_log("Application started".to_string());
    console.add_debug("Debug mode enabled".to_string());
    console.add_warning("Low memory warning".to_string());
    console.add_error("Critical error occurred".to_string());

    println!("   ✓ Total logs: {}", console.log_count());
    println!("   ✓ Errors: {}", console.error_count());
    println!("   ✓ Warnings: {}", console.warning_count());

    Ok(())
}

/// 完整引擎集成示例
async fn run_full_engine_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n4. Full Engine Integration Example");

    // 创建引擎
    let mut engine = Engine::new();
    println!("   ✓ Engine created");

    // 创建调试UI
    let mut debug_ui = DebugUI::new();
    debug_ui.log("Engine initialized".to_string());
    debug_ui.log("Creating test entities...".to_string());
    println!("   ✓ DebugUI initialized");

    // 创建测试实体
    create_entities(&mut engine.world, 50);
    debug_ui.log("Created 50 test entities".to_string());
    println!("   ✓ Created 50 entities");

    // 模拟游戏循环
    let mut frame_count = 0u64;
    let start_time = Instant::now();
    let target_duration = Duration::from_secs(2); // 运行2秒

    debug_ui.log("Starting game loop...".to_string());

    while start_time.elapsed() < target_duration {
        let frame_start = Instant::now();

        // 更新引擎
        engine.update().await?;

        // 模拟游戏逻辑
        update_game_logic(&mut engine.world);

        // 更新性能指标
        let frame_time = frame_start.elapsed().as_secs_f32();
        debug_ui.performance_panel().update_metrics(frame_time, frame_count);

        // 模拟Draw Calls
        if frame_count % 10 == 0 {
            debug_ui.performance_panel().update_draw_calls(
                30 + (frame_count % 15) as usize,
                5000 + frame_count as usize * 50,
            );
        }

        // 定期输出日志
        if frame_count % 30 == 0 {
            debug_ui.log(format!("Frame {}", frame_count));
        }

        // 模拟一些错误
        if frame_count == 50 {
            debug_ui.log_error("Simulated rendering error".to_string());
        }

        if frame_count == 100 {
            debug_ui
                .console_panel()
                .add_warning("Performance degradation detected".to_string());
        }

        frame_count += 1;

        // 控制帧率
        let elapsed = frame_start.elapsed();
        if elapsed < Duration::from_secs_f32(0.016) {
            std::thread::sleep(Duration::from_secs_f32(0.016) - elapsed);
        }
    }

    // 最终统计
    let elapsed = start_time.elapsed();
    let avg_fps = frame_count as f32 / elapsed.as_secs_f32();

    println!(
        "   ✓ Ran {} frames in {:.2}s",
        frame_count,
        elapsed.as_secs_f32()
    );
    println!("   ✓ Average FPS: {:.1}", avg_fps);
    println!("   ✓ Total logs: {}", debug_ui.console_panel().log_count());
    println!("   ✓ Errors: {}", debug_ui.console_panel().error_count());

    debug_ui.log("Game loop completed".to_string());
    debug_ui.log(format!(
        "Final stats: {} frames, {:.1} FPS",
        frame_count, avg_fps
    ));

    // 更新资源统计
    update_resource_stats(&mut debug_ui);
    println!("   ✓ Resource statistics updated");

    Ok(())
}

/// 创建测试实体
fn create_entities(world: &mut World, count: usize) {
    for i in 0..count {
        world.spawn((
            Transform {
                translation: glam::Vec3::new(
                    (i as f32 % 10.0) * 2.0,
                    ((i as f32 / 10.0).floor() * 2.0),
                    0.0,
                ),
                rotation: glam::Quat::IDENTITY,
                scale: glam::Vec3::ONE,
            },
            Velocity {
                linear: glam::Vec3::new(1.0, 0.0, 0.0),
                angular: glam::Vec3::ZERO,
            },
        ));
    }
}

/// 更新游戏逻辑
fn update_game_logic(world: &mut World) {
    let mut query = world.query::<(&mut Transform, &Velocity)>();

    for (mut transform, velocity) in query.iter_mut(world) {
        transform.translation += velocity.linear * 0.016;

        // 边界检查
        if transform.translation.x > 20.0 {
            transform.translation.x = -20.0;
        }
    }
}

/// 更新资源统计
fn update_resource_stats(debug_ui: &mut DebugUI) {
    use game_engine::debug::panels::ResourceStats;

    let texture_stats = ResourceStats {
        resource_type: "Texture".to_string(),
        total_count: 20,
        loaded_count: 18,
        failed_count: 0,
        total_size: 50 * 1024 * 1024, // 50MB
        loading_count: 2,
    };

    let mesh_stats = ResourceStats {
        resource_type: "Mesh".to_string(),
        total_count: 15,
        loaded_count: 15,
        failed_count: 0,
        total_size: 10 * 1024 * 1024, // 10MB
        loading_count: 0,
    };

    debug_ui.resource_panel().update_stats("Texture".to_string(), texture_stats);

    debug_ui.resource_panel().update_stats("Mesh".to_string(), mesh_stats);
}

// 修复console_panel获取问题
trait DebugUIConsoleExt {
    fn console_panel(&mut self) -> &mut game_engine::debug::panels::ConsolePanel;
}

impl DebugUIConsoleExt for DebugUI {
    fn console_panel(&mut self) -> &mut game_engine::debug::panels::ConsolePanel {
        // 这是一个临时解决方案
        // 在实际使用中，应该通过DebugUI的公开方法访问
        panic!("This is just a placeholder - use DebugUI's public methods instead");
    }
}
