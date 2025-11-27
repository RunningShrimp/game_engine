/// NPU超分辨率综合演示
/// 
/// 展示Intel、AMD、华为、高通、联发科NPU的集成和AI超分辨率功能

use game_engine::performance::hardware::{
    detect_npu, detect_gpu,
    npu_sdk::{NpuSdkManager, NpuBackend},
    npu_upscaling::{NpuUpscalingManager, AiUpscalingModel, HybridUpscalingStrategy},
    upscaling_sdk::{UpscalingSdkManager, UpscalingQuality},
};

fn main() {
    println!("=== NPU与AI超分辨率综合演示 ===\n");
    
    // 1. NPU检测和SDK支持
    demo_npu_detection();
    
    // 2. AI超分辨率模型
    demo_ai_upscaling_models();
    
    // 3. NPU超分辨率引擎
    demo_npu_upscaling_engine();
    
    // 4. 混合超分辨率策略
    demo_hybrid_upscaling();
    
    // 5. 性能对比
    demo_performance_comparison();
    
    println!("\n=== 演示完成 ===");
}

/// 演示NPU检测和SDK支持
fn demo_npu_detection() {
    println!("--- 1. NPU检测和SDK支持 ---\n");
    
    let npu_info = detect_npu();
    let npu_manager = NpuSdkManager::new(npu_info.clone());
    
    if let Some(npu) = npu_info {
        println!("检测到NPU:");
        println!("  厂商: {:?}", npu.vendor);
        println!("  名称: {}", npu.name);
        println!("  算力: {:.2} TOPS", npu.compute_units as f32 * 0.1);
    } else {
        println!("未检测到专用NPU");
    }
    
    println!("\n可用的NPU后端:");
    for backend in npu_manager.available_backends() {
        println!("  - {:?}", backend);
        print_backend_info(*backend);
    }
    
    println!();
}

/// 打印后端信息
fn print_backend_info(backend: NpuBackend) {
    match backend {
        NpuBackend::OpenVINO => {
            println!("    平台: Intel (CPU/GPU/VPU)");
            println!("    特点: 跨平台，支持多种硬件加速");
        }
        NpuBackend::ROCm => {
            println!("    平台: AMD GPU");
            println!("    特点: 开源，支持CDNA/RDNA架构");
        }
        NpuBackend::Ascend => {
            println!("    平台: 华为昇腾 (麒麟芯片)");
            println!("    特点: 高性能NPU，支持CANN框架");
        }
        NpuBackend::SNPE => {
            println!("    平台: 高通骁龙 (Hexagon DSP + Adreno GPU)");
            println!("    特点: 移动端优化，支持DSP/GPU/NPU多种运行时");
        }
        NpuBackend::NeuroPilot => {
            println!("    平台: 联发科天玑 (APU)");
            println!("    特点: 移动端AI加速，支持TFLite");
        }
        NpuBackend::TensorRT => {
            println!("    平台: NVIDIA GPU (Tensor Core)");
            println!("    特点: 高性能推理，支持混合精度");
        }
        NpuBackend::CoreML => {
            println!("    平台: Apple (Neural Engine)");
            println!("    特点: iOS/macOS原生支持");
        }
        NpuBackend::NNAPI => {
            println!("    平台: Android (通用)");
            println!("    特点: 跨厂商，自动选择最佳加速器");
        }
        NpuBackend::OnnxRuntime => {
            println!("    平台: 跨平台");
            println!("    特点: 通用ONNX模型支持");
        }
        NpuBackend::CpuFallback => {
            println!("    平台: CPU");
            println!("    特点: 回退方案，兼容性最好");
        }
    }
}

/// 演示AI超分辨率模型
fn demo_ai_upscaling_models() {
    println!("--- 2. AI超分辨率模型 ---\n");
    
    let models = [
        AiUpscalingModel::ESRGAN,
        AiUpscalingModel::RealESRGAN,
        AiUpscalingModel::EDSR,
        AiUpscalingModel::SwinIR,
        AiUpscalingModel::Lightweight,
    ];
    
    println!("可用的AI超分辨率模型:\n");
    println!("{:<20} {:>10} {:>15} {:>15} {:>10}", 
             "模型", "放大倍数", "移动端友好", "推理耗时(ms)", "文件名");
    println!("{}", "-".repeat(75));
    
    for model in models {
        println!("{:<20} {:>10}x {:>15} {:>15.1} {:>10}",
                 format!("{:?}", model),
                 model.scale_factor(),
                 if model.is_mobile_friendly() { "是" } else { "否" },
                 model.estimated_inference_time_ms(),
                 model.model_filename());
    }
    
    println!("\n推荐:");
    println!("  桌面端: RealESRGAN (高质量，适中性能)");
    println!("  移动端: Lightweight (低延迟，实时处理)");
    println!();
}

/// 演示NPU超分辨率引擎
fn demo_npu_upscaling_engine() {
    println!("--- 3. NPU超分辨率引擎 ---\n");
    
    let npu_info = detect_npu();
    let npu_manager = NpuSdkManager::new(npu_info);
    let upscaling_manager = NpuUpscalingManager::new(npu_manager);
    
    // 测试不同分辨率
    let test_cases = [
        (1280, 720, "720p"),
        (1920, 1080, "1080p"),
        (2560, 1440, "1440p"),
    ];
    
    println!("创建NPU超分辨率引擎:\n");
    
    for (width, height, name) in test_cases {
        println!("{}:", name);
        
        match upscaling_manager.create_engine(
            Some(AiUpscalingModel::RealESRGAN),
            width,
            height,
            UpscalingQuality::Balanced,
        ) {
            Ok(engine) => {
                let (render_w, render_h) = engine.render_resolution();
                let (display_w, display_h) = engine.display_resolution();
                
                println!("  ✓ 引擎创建成功");
                println!("    NPU后端: {:?}", engine.npu_backend());
                println!("    模型: {:?}", engine.model_type());
                println!("    渲染分辨率: {}x{}", render_w, render_h);
                println!("    显示分辨率: {}x{}", display_w, display_h);
                
                let pixel_savings = 1.0 - (render_w * render_h) as f32 / (display_w * display_h) as f32;
                println!("    像素节省: {:.1}%", pixel_savings * 100.0);
            }
            Err(e) => {
                println!("  ✗ 引擎创建失败: {}", e);
            }
        }
        println!();
    }
}

/// 演示混合超分辨率策略
fn demo_hybrid_upscaling() {
    println!("--- 4. 混合超分辨率策略 ---\n");
    
    let mut strategy = HybridUpscalingStrategy::new();
    
    println!("混合策略说明:");
    println!("  - 性能充足时 (<16.67ms): 使用NPU AI超分，获得最佳画质");
    println!("  - 性能不足时 (>16.67ms): 使用传统超分 (FSR/DLSS)，保证流畅度");
    println!();
    
    // 模拟不同的帧时间场景
    let scenarios = [
        (10.0, "轻度场景"),
        (16.67, "60fps阈值"),
        (20.0, "中度负载"),
        (33.33, "30fps"),
    ];
    
    println!("场景测试:\n");
    println!("{:<20} {:>15} {:>20}", "场景", "帧时间(ms)", "选择策略");
    println!("{}", "-".repeat(60));
    
    for (frame_time, scenario) in scenarios {
        let selected = if frame_time < 16.67 {
            "NPU AI超分"
        } else {
            "传统超分 (FSR/DLSS)"
        };
        
        println!("{:<20} {:>15.2} {:>20}", scenario, frame_time, selected);
    }
    
    println!();
}

/// 演示性能对比
fn demo_performance_comparison() {
    println!("--- 5. 性能对比 ---\n");
    
    println!("不同超分辨率方案的性能对比:\n");
    
    // 模拟数据
    let comparisons = [
        ("原生渲染", 1.0, 100.0, "基准"),
        ("FSR 2.0", 2.2, 95.0, "开源，跨平台"),
        ("DLSS 2.0", 2.5, 98.0, "NVIDIA专属，高质量"),
        ("XeSS", 2.3, 96.0, "Intel专属"),
        ("NPU AI超分 (桌面)", 2.0, 99.0, "高质量，需NPU"),
        ("NPU AI超分 (移动)", 1.8, 92.0, "移动端优化"),
    ];
    
    println!("{:<25} {:>15} {:>15} {:>20}", 
             "方案", "性能提升", "画质评分", "备注");
    println!("{}", "-".repeat(80));
    
    for (name, perf_gain, quality, note) in comparisons {
        println!("{:<25} {:>14.1}x {:>14.0}% {:>20}",
                 name, perf_gain, quality, note);
    }
    
    println!("\n结论:");
    println!("  1. 传统超分 (DLSS/FSR) 性能最优，适合实时游戏");
    println!("  2. NPU AI超分画质最佳，适合对画质要求高的场景");
    println!("  3. 混合策略可以在性能和画质之间取得最佳平衡");
    println!();
}
