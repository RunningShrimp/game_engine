//! 混合模式游戏循环演示
//!
//! 展示 HybridGameLoop 的使用方法和性能优势

use game_engine::core::engine::HybridGameLoop;
use bevy_ecs::prelude::*;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("========================================");
    tracing::info!("混合模式游戏循环演示");
    tracing::info!("========================================\n");

    // 创建混合模式游戏循环 (60 FPS)
    let mut game_loop = HybridGameLoop::new(60);

    tracing::info!("目标: 演示 3 秒 (180帧 @ 60fps)");
    tracing::info!("特点: 主循环同步 + 异步任务后台处理\n");

    // 计数器
    let mut frame_count = 0u64;
    let max_frames = 180; // 3秒 @ 60fps

    // 提交一些异步任务
    tracing::info!("提交异步任务...");
    game_loop.submit_resource_load("texture1.png", "/assets/texture1.png");
    game_loop.submit_network_request("https://example.com/api");
    let entity = Entity::from_raw(100);
    game_loop.submit_ai_computation(entity, "pathfinding");

    // 运行游戏循环
    let start = std::time::Instant::now();

    // 我们创建一个有限版本的循环用于演示
    let fixed_timestep = Duration::from_secs_f64(1.0 / 60.0);
    let mut last_frame_time = std::time::Instant::now();
    let mut accumulator = Duration::ZERO;
    let mut world = World::new();

    tracing::info!("开始游戏循环...\n");

    while frame_count < max_frames {
        let frame_start = std::time::Instant::now();

        // === 1. 计算帧时间 ===
        let frame_time = frame_start.duration_since(last_frame_time);
        last_frame_time = frame_start;
        accumulator = accumulator.saturating_add(frame_time);

        // === 2. 固定时间步物理更新 (同步) ===
        let mut physics_steps = 0u32;
        while accumulator >= fixed_timestep {
            // 模拟物理更新
            let mut sum = 0.0f32;
            for i in 0..1000 {
                sum += (i as f32).sqrt();
            }
            std::hint::black_box(sum);

            accumulator = accumulator.saturating_sub(fixed_timestep);
            physics_steps += 1;

            if physics_steps > 10 {
                accumulator = Duration::ZERO;
                break;
            }
        }

        // === 3. 游戏逻辑更新 (同步) ===
        let mut sum = 0u32;
        for i in 0..500 {
            sum = sum.wrapping_add(i);
        }
        std::hint::black_box(sum);

        // === 4. 轮询异步任务 (非阻塞) ===
        game_loop.poll_async_tasks(&mut world);

        // === 5. 渲染 (同步) ===
        let mut data = Vec::with_capacity(100);
        for i in 0..100 {
            data.push(i);
        }
        std::hint::black_box(data);

        // === 6. 帧率控制 ===
        let total_frame_time = frame_start.elapsed();
        if total_frame_time < fixed_timestep {
            std::thread::sleep(fixed_timestep - total_frame_time);
        }

        frame_count += 1;

        // 每60帧打印一次
        if frame_count % 60 == 0 {
            let elapsed = start.elapsed();
            let avg_fps = frame_count as f64 / elapsed.as_secs_f64();
            tracing::info!(
                "进度: {}/{} 帧, 平均FPS: {:.1}, 物理步数: {}",
                frame_count,
                max_frames,
                avg_fps,
                physics_steps
            );
        }
    }

    let total_elapsed = start.elapsed();
    let actual_fps = frame_count as f64 / total_elapsed.as_secs_f64();

    tracing::info!("\n========================================");
    tracing::info!("演示完成!");
    tracing::info!("========================================");
    tracing::info!("总帧数: {}", frame_count);
    tracing::info!("总时间: {:.2}s (目标: 3.00s)", total_elapsed.as_secs_f64());
    tracing::info!("实际FPS: {:.2}", actual_fps);
    tracing::info!("目标FPS: 60.00");
    tracing::info!("帧时间偏差: {:.2}%", (actual_fps / 60.0 - 1.0) * 100.0);

    // 打印性能统计
    game_loop.print_performance_report();

    tracing::info!("\n演示成功! 混合模式游戏循环运行正常。");
    tracing::info!("异步任务在后台处理，主循环保持同步执行。");

    Ok(())
}
