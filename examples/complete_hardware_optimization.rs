/// 完整的硬件优化系统综合示例
/// 
/// 展示所有硬件优化功能的集成使用

use game_engine::performance::hardware::*;
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║        游戏引擎完整硬件优化系统 - 综合演示                   ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    
    // ========== 第一部分：硬件检测 ==========
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ 第一部分：硬件检测与分析                                    │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();
    
    let hardware_info = get_hardware_info();
    print_hardware_info();
    
    // ========== 第二部分：GPU特定优化 ==========
    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ 第二部分：GPU特定渲染优化策略                               │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();
    
    let gpu_opt = GpuOptimization::for_gpu(&hardware_info.gpu);
    println!("GPU优化策略:");
    println!("  渲染管线模式: {:?}", gpu_opt.preferred_pipeline_mode);
    println!("  异步计算: {}", if gpu_opt.use_async_compute { "启用" } else { "禁用" });
    println!("  Bindless纹理: {}", if gpu_opt.use_bindless_textures { "启用" } else { "禁用" });
    println!("  纹理压缩格式: {:?}", gpu_opt.texture_compression_format);
    println!("  最大DrawCall数/帧: {}", gpu_opt.max_draw_calls_per_frame);
    println!("  粒子预算: {}", gpu_opt.particle_budget);
    println!("  最大光源数: {}", gpu_opt.max_lights_per_frame);
    println!("  阴影级联数: {}", gpu_opt.shadow_cascade_count);
    println!("  推荐纹理最大尺寸: {}x{}", gpu_opt.max_texture_size(), gpu_opt.max_texture_size());
    println!("  推荐MSAA采样数: {}x", gpu_opt.recommended_msaa_samples());
    println!();
    
    // ========== 第三部分：NPU加速 ==========
    if hardware_info.npu.is_some() {
        println!("\n┌─────────────────────────────────────────────────────────────┐");
        println!("│ 第三部分：NPU加速应用                                       │");
        println!("└─────────────────────────────────────────────────────────────┘");
        println!();
        
        let npu_accelerator = NpuAccelerator::new(hardware_info.npu.clone());
        
        println!("NPU状态: {}", if npu_accelerator.is_enabled() { "✓ 已启用" } else { "✗ 未启用" });
        
        if npu_accelerator.is_enabled() {
            println!("\n推荐使用场景:");
            for use_case in npu_accelerator.recommended_use_cases() {
                println!("  • {}", use_case);
            }
            
            // 演示物理预测
            println!("\n演示：物理预测");
            if let Some(prediction) = npu_accelerator.predict_physics(
                1,
                [0.0, 10.0, 0.0],
                [5.0, 0.0, 0.0],
                1.0,
            ) {
                println!("  对象ID: {}", prediction.object_id);
                println!("  预测位置: [{:.2}, {:.2}, {:.2}]", 
                    prediction.predicted_position[0],
                    prediction.predicted_position[1],
                    prediction.predicted_position[2]);
                println!("  预测速度: [{:.2}, {:.2}, {:.2}]",
                    prediction.predicted_velocity[0],
                    prediction.predicted_velocity[1],
                    prediction.predicted_velocity[2]);
                println!("  置信度: {:.2}%", prediction.confidence * 100.0);
            }
            
            // 演示NPC行为决策
            println!("\n演示：NPC行为决策");
            if let Some(decision) = npu_accelerator.decide_npc_behavior(
                1,
                [10.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                0.3,
                5,
            ) {
                println!("  NPC ID: {}", decision.npc_id);
                println!("  决策行动: {:?}", decision.action);
                println!("  优先级: {:.2}", decision.priority);
            }
        }
        println!();
    }
    
    // ========== 第四部分：功耗管理 ==========
    if hardware_info.soc.is_some() {
        println!("\n┌─────────────────────────────────────────────────────────────┐");
        println!("│ 第四部分：SoC功耗管理与热节流                               │");
        println!("└─────────────────────────────────────────────────────────────┘");
        println!();
        
        let mut power_manager = PowerManager::new(hardware_info.soc.clone());
        
        println!("平台类型: 移动平台");
        println!("功耗模式: {:?}", power_manager.power_mode());
        
        // 模拟一些帧
        for i in 0..60 {
            let frame_time = 16.67 + (i as f32 * 0.05);
            power_manager.update(frame_time);
        }
        
        let stats = power_manager.get_stats();
        println!("\n性能统计:");
        println!("  平均帧时间: {:.2} ms", stats.average_frame_time_ms);
        println!("  平均帧率: {:.1} FPS", stats.average_fps);
        println!("  估算温度: {:.1}°C", stats.estimated_temperature);
        println!("  热状态: {:?}", stats.thermal_state);
        
        if let Some(adjustment) = power_manager.get_adjustment_recommendation() {
            println!("\n性能调整建议:");
            println!("  原因: {}", adjustment.reason);
            println!("  分辨率缩放: {:.2}", adjustment.resolution_scale);
            println!("  目标帧率: {} FPS", adjustment.target_fps);
            println!("  阴影质量: {}", adjustment.shadow_quality);
        }
        
        println!("\n电池优化建议:");
        for tip in power_manager.get_battery_optimization() {
            println!("  {}", tip);
        }
        
        println!("\nSoC特定优化:");
        for tip in power_manager.get_soc_specific_tips() {
            println!("  {}", tip);
        }
        println!();
    }
    
    // ========== 第五部分：超分辨率技术 ==========
    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ 第五部分：超分辨率技术集成                                  │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();
    
    let upscaling_manager = UpscalingManager::new(hardware_info.gpu.clone());
    
    println!("可用的超分辨率技术:");
    for tech in upscaling_manager.available_techs() {
        let marker = if *tech == upscaling_manager.active_tech() { "→" } else { " " };
        println!("  {} {:?}: {}", marker, tech, upscaling_manager.tech_description(*tech));
    }
    
    println!("\n当前配置:");
    println!("  激活技术: {:?}", upscaling_manager.active_tech());
    println!("  质量模式: {:?}", upscaling_manager.quality_mode());
    
    let (render_w, render_h) = upscaling_manager.calculate_render_resolution(3840, 2160);
    println!("\n分辨率计算（4K输出）:");
    println!("  输出分辨率: 3840x2160");
    println!("  内部渲染: {}x{}", render_w, render_h);
    println!("  渲染像素节省: {:.1}%", 
        (1.0 - (render_w * render_h) as f32 / (3840.0 * 2160.0)) * 100.0);
    println!("  预期性能提升: {:.2}x", upscaling_manager.estimated_performance_gain());
    
    println!("\n推荐设置:");
    for rec in upscaling_manager.get_recommendations() {
        println!("  • {}", rec);
    }
    println!();
    
    // ========== 第六部分：自适应性能系统 ==========
    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ 第六部分：自适应性能系统（动态画质调整）                    │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();
    
    let config = hardware_info.recommended_config.clone();
    let power_manager = PowerManager::new(hardware_info.soc.clone());
    let mut adaptive = AdaptivePerformance::new(config, power_manager);
    
    println!("初始配置:");
    println!("  目标帧率: {} FPS", adaptive.config().target_fps);
    println!("  分辨率缩放: {:.2}", adaptive.config().resolution_scale);
    println!("  阴影质量: {:?}", adaptive.config().shadow_quality);
    
    println!("\n模拟场景1：性能不足（高负载场景）");
    for _ in 0..100 {
        adaptive.update(25.0); // 40fps，低于目标60fps
    }
    thread::sleep(Duration::from_secs(4));
    adaptive.update(25.0);
    
    let stats = adaptive.stats();
    println!("  当前帧率: {:.1} FPS", stats.current_fps);
    println!("  目标帧率: {:.1} FPS", stats.target_fps);
    println!("  调整次数: {}", stats.total_adjustments);
    println!("  当前分辨率缩放: {:.2}", stats.current_resolution_scale);
    
    println!("\n模拟场景2：性能恢复（低负载场景）");
    adaptive.reset_adjustment_count();
    for _ in 0..100 {
        adaptive.update(10.0); // 100fps，高于目标60fps
    }
    thread::sleep(Duration::from_secs(4));
    adaptive.update(10.0);
    
    let stats = adaptive.stats();
    println!("  当前帧率: {:.1} FPS", stats.current_fps);
    println!("  目标帧率: {:.1} FPS", stats.target_fps);
    println!("  调整次数: {}", stats.total_adjustments);
    println!("  当前分辨率缩放: {:.2}", stats.current_resolution_scale);
    println!();
    
    // ========== 总结 ==========
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║ 总结：开发者无需优化即可获得高性能                           ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("本引擎提供的自动优化功能：");
    println!();
    println!("✓ 自动硬件检测");
    println!("  - 识别GPU、NPU、SoC型号和性能等级");
    println!("  - 检测支持的高级特性（光追、网格着色器等）");
    println!();
    println!("✓ GPU特定优化");
    println!("  - 针对NVIDIA、AMD、Intel、Apple等厂商的专属优化");
    println!("  - 自动选择最佳渲染管线（前向/延迟/Tile-based）");
    println!("  - 优化DrawCall、粒子、光源等预算");
    println!();
    println!("✓ NPU智能加速");
    println!("  - AI驱动的物理预测");
    println!("  - 智能NPC行为决策");
    println!("  - 程序化内容生成");
    println!();
    println!("✓ 移动平台优化");
    println!("  - 功耗管理和热节流");
    println!("  - 电池优化建议");
    println!("  - SoC特定优化策略");
    println!();
    println!("✓ 超分辨率技术");
    println!("  - 自动选择最佳技术（DLSS/FSR/XeSS/MetalFX）");
    println!("  - 智能质量模式选择");
    println!("  - 显著提升性能（最高2-3倍）");
    println!();
    println!("✓ 自适应性能");
    println!("  - 运行时动态调整画质");
    println!("  - 自动维持目标帧率");
    println!("  - 热节流自动降频");
    println!();
    println!("开发者只需：");
    println!("  1. 在游戏启动时调用 get_hardware_info()");
    println!("  2. 应用推荐的配置");
    println!("  3. （可选）启用自适应性能系统");
    println!();
    println!("引擎会自动处理所有硬件适配和性能优化！");
    println!();
}
