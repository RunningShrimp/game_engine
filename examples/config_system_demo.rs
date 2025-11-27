/// 配置系统演示
/// 
/// 展示如何使用统一配置系统

use game_engine::config::*;

fn main() {
    println!("=== 游戏引擎配置系统演示 ===\n");
    
    // 1. 创建默认配置
    println!("1. 创建默认配置");
    let mut config = EngineConfig::default();
    println!("   默认分辨率: {}x{}", config.graphics.resolution.width, config.graphics.resolution.height);
    println!("   默认目标帧率: {}", config.performance.target_fps);
    println!();
    
    // 2. 修改配置
    println!("2. 修改配置");
    config.graphics.resolution.width = 2560;
    config.graphics.resolution.height = 1440;
    config.graphics.fullscreen = true;
    config.graphics.upscaling.enabled = true;
    config.graphics.upscaling.technology = graphics::UpscalingTechnology::FSR;
    config.performance.target_fps = 120;
    config.performance.npu.enabled = true;
    config.performance.npu.ai_upscaling = true;
    println!("   新分辨率: {}x{}", config.graphics.resolution.width, config.graphics.resolution.height);
    println!("   新目标帧率: {}", config.performance.target_fps);
    println!("   超分辨率: {:?}", config.graphics.upscaling.technology);
    println!();
    
    // 3. 保存为TOML文件
    println!("3. 保存为TOML文件");
    if let Err(e) = config.save_toml("example_config.toml") {
        eprintln!("   保存失败: {}", e);
    } else {
        println!("   已保存到 example_config.toml");
    }
    println!();
    
    // 4. 保存为JSON文件
    println!("4. 保存为JSON文件");
    if let Err(e) = config.save_json("example_config.json") {
        eprintln!("   保存失败: {}", e);
    } else {
        println!("   已保存到 example_config.json");
    }
    println!();
    
    // 5. 从TOML文件加载
    println!("5. 从TOML文件加载");
    match EngineConfig::from_toml_file("example_config.toml") {
        Ok(loaded_config) => {
            println!("   加载成功");
            println!("   分辨率: {}x{}", loaded_config.graphics.resolution.width, loaded_config.graphics.resolution.height);
            println!("   目标帧率: {}", loaded_config.performance.target_fps);
        }
        Err(e) => {
            eprintln!("   加载失败: {}", e);
        }
    }
    println!();
    
    // 6. 应用环境变量覆盖
    println!("6. 应用环境变量覆盖");
    println!("   设置环境变量: ENGINE_GRAPHICS_WIDTH=3840");
    std::env::set_var("ENGINE_GRAPHICS_WIDTH", "3840");
    std::env::set_var("ENGINE_GRAPHICS_HEIGHT", "2160");
    std::env::set_var("ENGINE_PERFORMANCE_TARGET_FPS", "144");
    
    let mut config = EngineConfig::default();
    config.apply_env_overrides();
    println!("   应用后的分辨率: {}x{}", config.graphics.resolution.width, config.graphics.resolution.height);
    println!("   应用后的目标帧率: {}", config.performance.target_fps);
    println!();
    
    // 7. 验证配置
    println!("7. 验证配置");
    match config.validate() {
        Ok(_) => println!("   配置有效"),
        Err(e) => eprintln!("   配置无效: {}", e),
    }
    println!();
    
    // 8. 自动加载配置
    println!("8. 自动加载配置");
    let config = EngineConfig::load_or_default();
    println!("   已加载配置");
    println!("   分辨率: {}x{}", config.graphics.resolution.width, config.graphics.resolution.height);
    println!();
    
    // 9. 展示完整配置
    println!("9. 完整配置示例");
    println!();
    println!("图形配置:");
    println!("  分辨率: {}x{}", config.graphics.resolution.width, config.graphics.resolution.height);
    println!("  全屏: {}", config.graphics.fullscreen);
    println!("  垂直同步: {}", config.graphics.vsync);
    println!("  抗锯齿: {:?}", config.graphics.anti_aliasing);
    println!("  阴影质量: {:?}", config.graphics.shadow_quality);
    println!("  超分辨率: 启用={}, 技术={:?}", config.graphics.upscaling.enabled, config.graphics.upscaling.technology);
    println!();
    
    println!("性能配置:");
    println!("  目标帧率: {}", config.performance.target_fps);
    println!("  自动优化: {}", config.performance.auto_optimize);
    println!("  SIMD: 启用={}", config.performance.simd.enabled);
    println!("  NPU: 启用={}, 后端={:?}", config.performance.npu.enabled, config.performance.npu.backend);
    println!("  工作线程数: {}", config.performance.threading.worker_threads);
    println!();
    
    println!("音频配置:");
    println!("  主音量: {}", config.audio.master_volume);
    println!("  音乐音量: {}", config.audio.music_volume);
    println!("  采样率: {} Hz", config.audio.sample_rate);
    println!();
    
    println!("输入配置:");
    println!("  鼠标灵敏度: {}", config.input.mouse_sensitivity);
    println!("  手柄死区: {}", config.input.gamepad_deadzone);
    println!("  前进键: {}", config.input.key_bindings.forward);
    println!();
    
    println!("=== 演示完成 ===");
}
