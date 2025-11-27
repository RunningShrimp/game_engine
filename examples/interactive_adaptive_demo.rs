/// 交互式自适应性能演示
/// 
/// 实时展示引擎如何动态调整画质以维持目标帧率

use game_engine::performance::hardware::*;
use game_engine::performance::hardware::simulated_renderer::{SimulatedRenderer, SceneComplexity, RenderResult};
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║     游戏引擎自适应性能系统 - 交互式实时演示                  ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("本演示将模拟一个游戏从低负载场景过渡到高负载场景，");
    println!("并展示引擎如何自动调整画质以维持目标帧率。");
    println!();
    
    // 初始化硬件信息
    println!("正在检测硬件...");
    let hardware_info = get_hardware_info();
    println!("✓ 检测完成");
    println!();
    
    // 创建GPU优化和超分辨率管理器
    let gpu_opt = GpuOptimization::for_gpu(&hardware_info.gpu);
    let upscaling = UpscalingManager::new(hardware_info.gpu.clone());
    
    // 创建模拟渲染器
    let mut renderer = SimulatedRenderer::new(gpu_opt, upscaling);
    
    // 创建功耗管理器和自适应性能系统
    let power_manager = PowerManager::new(hardware_info.soc.clone());
    let config = hardware_info.recommended_config.clone();
    let mut adaptive = AdaptivePerformance::new(config, power_manager);
    
    println!("═══════════════════════════════════════════════════════════════");
    println!("初始配置:");
    println!("═══════════════════════════════════════════════════════════════");
    print_config(&adaptive);
    println!();
    
    // 模拟游戏循环
    println!("开始模拟游戏...");
    println!();
    
    // 阶段1：低负载场景（探索场景）
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ 阶段 1: 低负载场景（探索空旷区域）                          │");
    println!("└─────────────────────────────────────────────────────────────┘");
    run_simulation_phase(
        &mut renderer,
        &mut adaptive,
        0.5,  // 低负载
        60,   // 60帧
        "低负载"
    );
    
    // 阶段2：中等负载场景（城镇）
    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ 阶段 2: 中等负载场景（进入城镇，NPC和建筑增多）             │");
    println!("└─────────────────────────────────────────────────────────────┘");
    run_simulation_phase(
        &mut renderer,
        &mut adaptive,
        1.2,  // 中等负载
        60,
        "中等负载"
    );
    
    // 阶段3：高负载场景（大规模战斗）
    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ 阶段 3: 高负载场景（大规模战斗，粒子和光效爆炸）            │");
    println!("└─────────────────────────────────────────────────────────────┘");
    run_simulation_phase(
        &mut renderer,
        &mut adaptive,
        2.5,  // 高负载
        60,
        "高负载"
    );
    
    // 阶段4：极限负载场景（Boss战）
    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ 阶段 4: 极限负载场景（Boss战，屏幕特效拉满）                │");
    println!("└─────────────────────────────────────────────────────────────┘");
    run_simulation_phase(
        &mut renderer,
        &mut adaptive,
        3.5,  // 极限负载
        60,
        "极限负载"
    );
    
    // 阶段5：负载恢复（战斗结束）
    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ 阶段 5: 负载恢复（战斗结束，返回平静场景）                  │");
    println!("└─────────────────────────────────────────────────────────────┘");
    adaptive.reset_adjustment_count();
    run_simulation_phase(
        &mut renderer,
        &mut adaptive,
        0.7,  // 低负载
        60,
        "负载恢复"
    );
    
    // 最终统计
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║ 演示总结                                                      ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    
    let final_stats = adaptive.stats();
    println!("最终性能统计:");
    println!("  当前帧率: {:.1} FPS", final_stats.current_fps);
    println!("  目标帧率: {:.1} FPS", final_stats.target_fps);
    println!("  总调整次数: {}", final_stats.total_adjustments);
    println!("  当前分辨率缩放: {:.2}x", final_stats.current_resolution_scale);
    println!("  热状态: {:?}", final_stats.thermal_state);
    println!();
    
    println!("最终配置:");
    print_config(&adaptive);
    println!();
    
    println!("═══════════════════════════════════════════════════════════════");
    println!("关键观察:");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    println!("1. 当负载增加时，引擎自动降低画质以维持帧率");
    println!("2. 调整是渐进式的，优先降低对视觉影响较小的设置");
    println!("3. 当负载降低时，引擎会谨慎地恢复画质");
    println!("4. 整个过程无需开发者干预，完全自动化");
    println!();
    println!("这就是\"零配置、自适应\"性能系统的威力！");
    println!();
}

fn run_simulation_phase(
    renderer: &mut SimulatedRenderer,
    adaptive: &mut AdaptivePerformance,
    load_factor: f32,
    frame_count: u32,
    phase_name: &str,
) {
    renderer.set_load_factor(load_factor);
    
    println!("负载系数: {:.1}x", load_factor);
    println!();
    
    let mut frame_times = Vec::new();
    
    // 模拟帧
    for frame in 0..frame_count {
        let scene = SceneComplexity::from_config(adaptive.config());
        let result = renderer.render_frame(scene);
        
        adaptive.update(result.frame_time_ms);
        frame_times.push(result.frame_time_ms);
        
        // 每10帧打印一次状态
        if frame % 10 == 0 {
            print_frame_status(frame, &result, adaptive);
        }
        
        // 模拟帧间隔
        thread::sleep(Duration::from_millis(5));
    }
    
    // 等待调整冷却
    thread::sleep(Duration::from_secs(4));
    
    // 触发一次更新以应用可能的调整
    let scene = SceneComplexity::from_config(adaptive.config());
    let result = renderer.render_frame(scene);
    adaptive.update(result.frame_time_ms);
    
    // 阶段总结
    let avg_frame_time = frame_times.iter().sum::<f32>() / frame_times.len() as f32;
    let avg_fps = 1000.0 / avg_frame_time;
    
    println!();
    println!("阶段总结 ({}):", phase_name);
    println!("  平均帧时间: {:.2} ms", avg_frame_time);
    println!("  平均帧率: {:.1} FPS", avg_fps);
    println!("  目标帧率: {:.1} FPS", adaptive.stats().target_fps);
    
    let stats = adaptive.stats();
    if stats.total_adjustments > 0 {
        println!("  ⚠ 触发了 {} 次画质调整", stats.total_adjustments);
    } else {
        println!("  ✓ 无需调整，性能良好");
    }
}

fn print_frame_status(frame: u32, result: &RenderResult, adaptive: &AdaptivePerformance) {
    let stats = adaptive.stats();
    let status = if result.fps() >= stats.target_fps * 0.9 {
        "✓"
    } else if result.fps() >= stats.target_fps * 0.7 {
        "⚠"
    } else {
        "✗"
    };
    
    println!(
        "  帧 {:3} {} | {:.1} FPS ({:.2}ms) | 目标: {:.0} FPS | 分辨率: {:.2}x | DrawCalls: {} | 粒子: {}",
        frame,
        status,
        result.fps(),
        result.frame_time_ms,
        stats.target_fps,
        stats.current_resolution_scale,
        result.draw_calls,
        result.particles_rendered
    );
}

fn print_config(adaptive: &AdaptivePerformance) {
    let config = adaptive.config();
    println!("  目标帧率: {} FPS", config.target_fps);
    println!("  分辨率缩放: {:.2}x", config.resolution_scale);
    println!("  阴影质量: {:?}", config.shadow_quality);
    println!("  纹理质量: {:?}", config.texture_quality);
    println!("  抗锯齿: {:?}", config.anti_aliasing);
    println!("  环境光遮蔽: {}", if config.ambient_occlusion { "开启" } else { "关闭" });
    println!("  泛光: {}", if config.bloom { "开启" } else { "关闭" });
    println!("  动态模糊: {}", if config.motion_blur { "开启" } else { "关闭" });
    println!("  景深: {}", if config.depth_of_field { "开启" } else { "关闭" });
    println!("  光线追踪: {}", if config.raytracing_enabled { "开启" } else { "关闭" });
    println!("  DLSS: {}", if config.dlss_enabled { "开启" } else { "关闭" });
    println!("  FSR: {}", if config.fsr_enabled { "开启" } else { "关闭" });
}
