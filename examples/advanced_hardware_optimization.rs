/// 高级硬件优化功能综合示例
/// 
/// 展示异步检测、错误处理、性能监控、NPU和超分辨率SDK集成

use game_engine::performance::hardware::{
    async_detect::start_async_detection,
    error::{ErrorHandler, HardwareError},
    metrics::{PerformanceMonitor, PerformanceProfiler},
    npu_sdk::{NpuSdkManager, NpuInferenceEngine},
    upscaling_sdk::{UpscalingSdkManager, UpscalingQuality},
    detect_gpu, detect_npu,
};
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== 高级硬件优化功能演示 ===\n");
    
    // 1. 异步硬件检测
    demo_async_detection();
    
    // 2. 错误处理
    demo_error_handling();
    
    // 3. 性能监控
    demo_performance_monitoring();
    
    // 4. NPU SDK集成
    demo_npu_sdk();
    
    // 5. 超分辨率SDK集成
    demo_upscaling_sdk();
    
    println!("\n=== 演示完成 ===");
}

/// 演示异步硬件检测
fn demo_async_detection() {
    println!("--- 1. 异步硬件检测 ---");
    
    // 启动异步检测
    let detector = start_async_detection();
    
    println!("硬件检测已在后台启动...");
    
    // 在检测的同时可以做其他事情
    println!("主线程可以继续执行其他任务...");
    for i in 1..=5 {
        thread::sleep(Duration::from_millis(20));
        println!("  任务 {} 完成，检测进度: {:.0}%", i, detector.progress() * 100.0);
    }
    
    // 等待检测完成
    let result = detector.wait_for_result();
    
    println!("\n检测完成！");
    println!("  GPU: {}", result.gpu.name);
    println!("  NPU: {:?}", result.npu.as_ref().map(|n| &n.name));
    println!("  SoC: {:?}", result.soc.as_ref().map(|s| &s.name));
    println!("  检测耗时: {:.2}ms", result.detection_time_ms);
    println!("  来自缓存: {}", result.from_cache);
    println!();
}

/// 演示错误处理
fn demo_error_handling() {
    println!("--- 2. 错误处理机制 ---");
    
    let handler = ErrorHandler::new();
    
    // 模拟几种不同的错误
    let errors = vec![
        HardwareError::GpuDetectionFailed {
            reason: "wgpu初始化失败".to_string(),
            attempted_methods: vec!["wgpu".to_string(), "系统API".to_string()],
        },
        HardwareError::NpuAccelerationError {
            operation: "模型加载".to_string(),
            reason: "模型文件不存在".to_string(),
        },
        HardwareError::UpscalingError {
            technology: "DLSS".to_string(),
            reason: "驱动版本过低".to_string(),
        },
    ];
    
    for error in &errors {
        let (strategy, suggestion) = handler.handle_with_suggestion(error);
        println!("错误: {}", error);
        println!("  恢复策略: {:?}", strategy);
        println!("  建议: {}", suggestion);
        println!();
    }
}

/// 演示性能监控
fn demo_performance_monitoring() {
    println!("--- 3. 性能监控系统 ---");
    
    let mut monitor = PerformanceMonitor::new();
    let mut profiler = PerformanceProfiler::new();
    
    println!("模拟60帧游戏循环...");
    
    for frame in 0..60 {
        profiler.start_frame();
        
        // 模拟不同的游戏阶段
        profiler.mark_section("input");
        thread::sleep(Duration::from_millis(1));
        
        profiler.mark_section("update");
        thread::sleep(Duration::from_millis(3));
        
        profiler.mark_section("physics");
        thread::sleep(Duration::from_millis(2));
        
        profiler.mark_section("render");
        thread::sleep(Duration::from_millis(10));
        
        profiler.mark_section("present");
        thread::sleep(Duration::from_millis(1));
        
        profiler.end_frame();
        
        // 更新监控器
        let frame_time = 16.67 + (frame as f32 * 0.1);
        monitor.update(frame_time);
    }
    
    // 生成报告
    println!("\n性能摘要:");
    let summary = monitor.generate_summary();
    summary.print();
    
    println!("性能分析:");
    let report = profiler.generate_report();
    report.print();
}

/// 演示NPU SDK集成
fn demo_npu_sdk() {
    println!("--- 4. NPU SDK集成 ---");
    
    let npu_info = detect_npu();
    let manager = NpuSdkManager::new(npu_info);
    
    println!("可用的NPU后端:");
    for backend in manager.available_backends() {
        println!("  - {:?}", backend);
        println!("    推荐格式: {:?}", manager.recommended_format(*backend));
    }
    
    // 尝试创建推理引擎
    match manager.create_engine(None) {
        Ok(mut engine) => {
            println!("\n成功创建NPU引擎: {:?}", engine.backend());
            println!("  输入形状: {:?}", engine.input_shape());
            println!("  输出形状: {:?}", engine.output_shape());
            
            // 预热
            if let Err(e) = engine.warmup() {
                println!("  预热失败: {}", e);
            } else {
                println!("  预热完成");
            }
            
            // 测试推理
            let input = vec![0.0; engine.input_shape().iter().product()];
            match engine.infer(&input) {
                Ok(output) => {
                    println!("  推理成功，输出大小: {}", output.len());
                }
                Err(e) => {
                    println!("  推理失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("\n创建NPU引擎失败: {}", e);
        }
    }
    
    println!();
}

/// 演示超分辨率SDK集成
fn demo_upscaling_sdk() {
    println!("--- 5. 超分辨率SDK集成 ---");
    
    let gpu = detect_gpu();
    let manager = UpscalingSdkManager::new(gpu);
    
    println!("可用的超分辨率技术:");
    for tech in manager.available_technologies() {
        println!("  - {:?}", tech);
    }
    
    println!("\n推荐技术: {:?}", manager.recommend_technology());
    println!("推荐质量: {:?}", manager.recommend_quality());
    
    // 测试不同的质量模式
    println!("\n质量模式对比:");
    for quality in [
        UpscalingQuality::Performance,
        UpscalingQuality::Balanced,
        UpscalingQuality::Quality,
        UpscalingQuality::UltraQuality,
    ] {
        println!("  {:?}:", quality);
        println!("    渲染缩放: {:.0}%", quality.render_scale() * 100.0);
        println!("    性能提升: {:.1}x", quality.performance_gain());
    }
    
    // 尝试创建超分辨率引擎
    let display_width = 1920;
    let display_height = 1080;
    
    match manager.create_engine(
        None,
        display_width,
        display_height,
        UpscalingQuality::Balanced,
    ) {
        Ok(engine) => {
            println!("\n成功创建超分辨率引擎: {:?}", engine.technology());
            println!("  显示分辨率: {:?}", engine.display_resolution());
            println!("  渲染分辨率: {:?}", engine.render_resolution());
            println!("  支持运动矢量: {}", engine.supports_motion_vectors());
            println!("  支持深度缓冲: {}", engine.supports_depth_buffer());
            
            let (render_w, render_h) = engine.render_resolution();
            let pixel_savings = 1.0 - (render_w * render_h) as f32 / (display_width * display_height) as f32;
            println!("  像素节省: {:.1}%", pixel_savings * 100.0);
        }
        Err(e) => {
            println!("\n创建超分辨率引擎失败: {}", e);
        }
    }
    
    println!();
}
