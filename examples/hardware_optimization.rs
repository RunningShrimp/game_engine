/// 硬件优化演示程序
/// 
/// 展示自动硬件检测和优化配置功能

use game_engine::performance::hardware::{
    get_hardware_info, print_hardware_info,
    gpu_detect::GpuVendor,
    auto_config::{AutoConfig, QualityPreset},
};

fn main() {
    println!("=== 游戏引擎硬件优化演示 ===\n");
    
    // 1. 检测硬件信息
    println!("步骤 1: 检测硬件信息");
    println!("{}", "=".repeat(60));
    print_hardware_info();
    
    let hardware_info = get_hardware_info();
    
    // 2. 显示自动配置
    println!("\n步骤 2: 自动生成优化配置");
    println!("{}", "=".repeat(60));
    let config = &hardware_info.recommended_config;
    
    println!("质量预设: {:?}", config.quality_preset);
    println!();
    
    println!("渲染设置:");
    println!("  分辨率缩放: {}x", config.resolution_scale);
    println!("  目标帧率: {} FPS", config.target_fps);
    println!("  垂直同步: {}", if config.vsync_enabled { "开启" } else { "关闭" });
    println!();
    
    println!("图形质量:");
    println!("  阴影质量: {:?}", config.shadow_quality);
    println!("  纹理质量: {:?}", config.texture_quality);
    println!("  抗锯齿: {:?}", config.anti_aliasing);
    println!("  环境光遮蔽: {}", if config.ambient_occlusion { "开启" } else { "关闭" });
    println!("  泛光: {}", if config.bloom { "开启" } else { "关闭" });
    println!("  动态模糊: {}", if config.motion_blur { "开启" } else { "关闭" });
    println!("  景深: {}", if config.depth_of_field { "开启" } else { "关闭" });
    println!();
    
    println!("高级特性:");
    println!("  光线追踪: {}", if config.raytracing_enabled { "开启" } else { "关闭" });
    println!("  DLSS: {}", if config.dlss_enabled { "开启" } else { "关闭" });
    println!("  FSR: {}", if config.fsr_enabled { "开启" } else { "关闭" });
    println!("  网格着色器: {}", if config.mesh_shaders_enabled { "开启" } else { "关闭" });
    println!("  可变速率着色: {}", if config.vrs_enabled { "开启" } else { "关闭" });
    println!();
    
    println!("性能优化:");
    println!("  NPU加速: {}", if config.use_npu_acceleration { "开启" } else { "关闭" });
    println!("  并行任务数: {}", config.parallel_task_count);
    println!("  批处理大小: {}", config.batch_size);
    println!("  剔除距离: {} 米", config.culling_distance);
    println!("  LOD偏移: {}", config.lod_bias);
    println!();
    
    // 3. GPU特定优化建议
    println!("\n步骤 3: GPU特定优化建议");
    println!("{}", "=".repeat(60));
    
    match hardware_info.gpu.vendor {
        GpuVendor::Nvidia => {
            println!("检测到NVIDIA GPU，建议:");
            println!("  ✓ 启用DLSS超分辨率技术");
            println!("  ✓ 使用Tensor Core加速AI功能");
            println!("  ✓ 启用光线追踪（如果支持）");
            println!("  ✓ 使用NVIDIA Reflex降低延迟");
        }
        GpuVendor::Amd => {
            println!("检测到AMD GPU，建议:");
            println!("  ✓ 启用FSR超分辨率技术");
            println!("  ✓ 使用Radeon Anti-Lag降低延迟");
            println!("  ✓ 启用FidelityFX特效");
            println!("  ✓ 优化异步计算管线");
        }
        GpuVendor::Intel => {
            println!("检测到Intel GPU，建议:");
            println!("  ✓ 启用XeSS超分辨率技术");
            println!("  ✓ 降低阴影和后处理质量");
            println!("  ✓ 使用较低的分辨率缩放");
            println!("  ✓ 启用动态分辨率");
        }
        GpuVendor::Apple => {
            println!("检测到Apple GPU，建议:");
            println!("  ✓ 使用Metal API优化");
            println!("  ✓ 启用MetalFX上采样");
            println!("  ✓ 利用统一内存架构");
            println!("  ✓ 使用Neural Engine加速AI");
        }
        GpuVendor::Qualcomm => {
            println!("检测到Qualcomm Adreno GPU，建议:");
            println!("  ✓ 启用移动端优化");
            println!("  ✓ 降低分辨率和特效");
            println!("  ✓ 使用Hexagon DSP加速");
            println!("  ✓ 注意热节流管理");
        }
        GpuVendor::Mali => {
            println!("检测到ARM Mali GPU，建议:");
            println!("  ✓ 启用移动端优化");
            println!("  ✓ 使用tile-based渲染优化");
            println!("  ✓ 降低带宽使用");
            println!("  ✓ 优化纹理压缩");
        }
        _ => {
            println!("使用通用优化策略");
        }
    }
    println!();
    
    // 4. NPU加速建议
    if hardware_info.npu.is_some() {
        println!("\n步骤 4: NPU加速建议");
        println!("{}", "=".repeat(60));
        
        let npu = hardware_info.npu.as_ref().unwrap();
        println!("检测到NPU: {} ({:.2} TOPS)", npu.name, npu.tops);
        println!();
        println!("可以使用NPU加速的功能:");
        println!("  ✓ AI驱动的物理预测");
        println!("  ✓ 智能NPC行为");
        println!("  ✓ 程序化内容生成");
        println!("  ✓ 图像超分辨率");
        println!("  ✓ 动态难度调整");
        println!();
    }
    
    // 5. 保存配置
    println!("\n步骤 5: 保存配置");
    println!("{}", "=".repeat(60));
    
    let config_path = "/tmp/game_config.json";
    match config.save_to_file(config_path) {
        Ok(_) => println!("✓ 配置已保存到: {}", config_path),
        Err(e) => println!("✗ 保存配置失败: {}", e),
    }
    println!();
    
    // 6. 性能预测
    println!("\n步骤 6: 性能预测");
    println!("{}", "=".repeat(60));
    
    let capability = &hardware_info.capability;
    
    println!("预期性能:");
    match capability.tier {
        game_engine::performance::hardware::capability::PerformanceTier::Flagship => {
            println!("  🚀 旗舰级性能");
            println!("  预期帧率: 144+ FPS @ 4K");
            println!("  可以开启所有特效");
        }
        game_engine::performance::hardware::capability::PerformanceTier::High => {
            println!("  ⚡ 高端性能");
            println!("  预期帧率: 60+ FPS @ 4K 或 144+ FPS @ 1440p");
            println!("  可以开启大部分特效");
        }
        game_engine::performance::hardware::capability::PerformanceTier::MediumHigh => {
            println!("  ✨ 中高端性能");
            println!("  预期帧率: 60 FPS @ 1440p");
            println!("  建议中高画质");
        }
        game_engine::performance::hardware::capability::PerformanceTier::Medium => {
            println!("  📊 中端性能");
            println!("  预期帧率: 60 FPS @ 1080p");
            println!("  建议中等画质");
        }
        game_engine::performance::hardware::capability::PerformanceTier::MediumLow => {
            println!("  📉 中低端性能");
            println!("  预期帧率: 30-60 FPS @ 1080p");
            println!("  建议中低画质");
        }
        game_engine::performance::hardware::capability::PerformanceTier::Low => {
            println!("  ⚠️  入门级性能");
            println!("  预期帧率: 30 FPS @ 720p");
            println!("  建议低画质");
        }
    }
    println!();
    
    println!("=== 演示完成 ===");
    println!("\n提示: 引擎会在游戏启动时自动应用这些优化，");
    println!("      开发者无需手动配置即可获得最佳性能！");
}
