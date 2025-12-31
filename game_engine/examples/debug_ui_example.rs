//! 调试UI示例程序
//!
//! 演示如何使用DebugUI进行游戏引擎调试。

use bevy_ecs::prelude::*;
use game_engine::{
    core::engine::Engine,
    debug::DebugUI,
    ecs::{Transform, Velocity},
};

/// 主示例函数
pub async fn run_debug_ui_example() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    log::info!("Starting Debug UI Example");

    // 创建引擎
    let mut engine = Engine::new();

    // 创建调试UI
    let mut debug_ui = DebugUI::new();

    // 添加一些示例日志
    debug_ui.log("Debug UI initialized".to_string());
    debug_ui.log("Engine created successfully".to_string());

    // 创建测试实体
    create_test_entities(&mut engine.world);

    log::info!("Created test entities");

    // 模拟游戏循环
    let mut frame_count = 0u64;
    let mut last_time = std::time::Instant::now();

    log::info!("Starting game loop");

    // 模拟运行
    for i in 0..100 {
        // 计算帧时间
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(last_time).as_secs_f32();
        last_time = now;

        // 更新引擎
        engine.update().await?;

        // 模拟一些工作
        simulate_game_work(&mut engine.world, i);

        // 更新性能面板
        if let Some(perf_panel) = debug_ui.performance_panel() {
            perf_panel.update_metrics(delta_time, frame_count);

            // 模拟Draw Calls
            if i % 10 == 0 {
                perf_panel.update_draw_calls(50 + (i % 20) as usize, 10000 + i * 100);
            }
        }

        frame_count += 1;

        // 每30帧输出一次日志
        if i % 30 == 0 {
            debug_ui.log(format!("Frame {}", i));
        }

        // 模拟错误
        if i == 50 {
            debug_ui.log_error("Simulated error at frame 50".to_string());
        }
    }

    // 最终统计
    if let Some(perf_panel) = debug_ui.performance_panel() {
        if let Some(fps) = perf_panel.current_fps() {
            log::info!("Final FPS: {:.1}", fps);
        }
    }

    log::info!("Debug UI Example completed successfully");

    Ok(())
}

/// 创建测试实体
fn create_test_entities(world: &mut World) {
    // 创建一些测试实体
    for i in 0..10 {
        let entity = world.spawn((
            Transform {
                translation: glam::Vec3::new(i as f32, 0.0, 0.0),
                rotation: glam::Quat::IDENTITY,
                scale: glam::Vec3::ONE,
            },
            Velocity {
                linear: glam::Vec3::new(1.0, 0.0, 0.0),
                angular: glam::Vec3::ZERO,
            },
        ));

        log::debug!("Created entity: {:?}", entity);
    }
}

/// 模拟游戏工作
fn simulate_game_work(world: &mut World, frame: usize) {
    // 更新实体位置
    let mut query = world.query::<(&mut Transform, &Velocity)>();

    for (mut transform, velocity) in query.iter_mut(world) {
        transform.translation += velocity.linear * 0.016; // 假设60FPS
        transform.rotation += velocity.angular * 0.016;
    }

    // 每100帧创建一个新实体
    if frame % 100 == 0 && frame > 0 {
        let entity = world.spawn((
            Transform {
                translation: glam::Vec3::new(frame as f32, 0.0, 0.0),
                rotation: glam::Quat::IDENTITY,
                scale: glam::Vec3::ONE,
            },
            Velocity {
                linear: glam::Vec3::new(1.0, 0.0, 0.0),
                angular: glam::Vec3::ZERO,
            },
        ));
        log::debug!("Created new entity at frame {}: {:?}", frame, entity);
    }
}

/// 控制台UI示例（不需要引擎）
#[allow(dead_code)]
fn console_ui_example() {
    use game_engine::debug::panels::ConsolePanel;

    let mut console = ConsolePanel::new();

    // 添加各种日志
    console.add_log("Application started".to_string());
    console.add_debug("Debug message".to_string());
    console.add_warning("This is a warning".to_string());
    console.add_error("This is an error".to_string());

    println!("Console has {} messages", console.log_count());
    println!("Errors: {}", console.error_count());
    println!("Warnings: {}", console.warning_count());
}

/// 性能监控示例
#[allow(dead_code)]
fn performance_monitoring_example() {
    use game_engine::debug::panels::PerformancePanel;
    use std::time::Duration;

    let mut perf_panel = PerformancePanel::new();

    // 模拟100帧的性能数据
    for i in 0..100 {
        let frame_time = if i < 50 {
            0.016 // 60 FPS
        } else if i < 80 {
            0.033 // 30 FPS
        } else {
            0.050 // 20 FPS
        };

        perf_panel.update_metrics(frame_time, i);

        if i % 10 == 0 {
            perf_panel.update_draw_calls(50 + (i % 20) as usize, 10000 + i * 100);
        }

        std::thread::sleep(Duration::from_millis(16));
    }

    // 获取统计信息
    if let Some(avg_fps) = perf_panel.calculate_average_fps() {
        println!("Average FPS: {:.1}", avg_fps);
    }

    if let Some(min_fps) = perf_panel.calculate_min_fps() {
        println!("Min FPS: {:.1}", min_fps);
    }

    if let Some(max_fps) = perf_panel.calculate_max_fps() {
        println!("Max FPS: {:.1}", max_fps);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_panel() {
        console_ui_example();
    }

    #[test]
    fn test_performance_monitoring() {
        performance_monitoring_example();
    }

    #[tokio::test]
    async fn test_debug_ui_example() {
        let result = run_debug_ui_example().await;
        assert!(result.is_ok(), "Debug UI example should run successfully");
    }
}
