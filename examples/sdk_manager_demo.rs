/// SDK管理器使用示例
/// 
/// 展示如何使用统一的SDK管理器来自动选择和使用最优的NPU SDK

fn main() {
    println!("=== SDK Manager Demo ===\n");
    
    // 1. 创建SDK管理器
    println!("1. Creating SDK Manager...");
    let manager = match game_engine::performance::hardware::sdk_manager::SdkManager::new() {
        Ok(m) => {
            println!("   ✓ SDK Manager created successfully\n");
            m
        }
        Err(e) => {
            eprintln!("   ✗ Failed to create SDK Manager: {:?}", e);
            return;
        }
    };
    
    // 2. 打印可用后端信息
    manager.print_info();
    
    // 3. 获取首选后端
    if let Some(backend) = manager.preferred_backend() {
        println!("Using preferred backend: {:?}\n", backend);
    } else {
        println!("No preferred backend available\n");
        return;
    }
    
    // 4. 创建推理引擎
    println!("2. Creating inference engine...");
    let mut engine = match manager.create_engine() {
        Ok(e) => {
            println!("   ✓ Engine created successfully\n");
            e
        }
        Err(e) => {
            eprintln!("   ✗ Failed to create engine: {:?}", e);
            return;
        }
    };
    
    // 5. 模拟加载模型（实际使用中需要真实的模型文件）
    println!("3. Loading model...");
    println!("   (In real usage, you would load an actual model file)");
    println!("   Example: engine.load_model(Path::new(\"model.onnx\"))?;\n");
    
    // 6. 模拟推理
    println!("4. Performing inference...");
    println!("   (In real usage, you would provide actual input data)");
    println!("   Example:");
    println!("   let input = vec![0.0f32; 224 * 224 * 3];");
    println!("   let output = engine.infer(&input)?;\n");
    
    // 7. 使用便捷函数
    println!("5. Using convenience functions:");
    println!();
    println!("   // Auto-create engine");
    println!("   let engine = auto_create_engine()?;");
    println!();
    println!("   // Load model and create engine in one step");
    println!("   let engine = load_model(\"model.onnx\")?;");
    println!();
    
    // 8. 后端对比
    println!("6. Backend Comparison:");
    println!();
    println!("   ┌──────────────────────┬──────────────┬──────────┬────────────┐");
    println!("   │ Backend              │ Performance  │ Power    │ Platform   │");
    println!("   ├──────────────────────┼──────────────┼──────────┼────────────┤");
    println!("   │ Apple Core ML        │ ★★★★★        │ ★★★★★    │ iOS/macOS  │");
    println!("   │ Huawei CANN          │ ★★★★★        │ ★★★★     │ Kirin      │");
    println!("   │ Qualcomm SNPE        │ ★★★★         │ ★★★★★    │ Snapdragon │");
    println!("   │ MediaTek NeuroPilot  │ ★★★★         │ ★★★★     │ Dimensity  │");
    println!("   │ Intel OpenVINO       │ ★★★★         │ ★★★      │ Intel      │");
    println!("   │ AMD ROCm             │ ★★★★★        │ ★★       │ AMD GPU    │");
    println!("   │ ONNX Runtime         │ ★★★          │ ★★★      │ Universal  │");
    println!("   └──────────────────────┴──────────────┴──────────┴────────────┘");
    println!();
    
    // 9. 最佳实践
    println!("7. Best Practices:");
    println!();
    println!("   ✓ Let SDK Manager auto-select the best backend");
    println!("   ✓ Use quantized models (INT8/FP16) for better performance");
    println!("   ✓ Warm up the engine before actual inference");
    println!("   ✓ Reuse the engine for multiple inferences");
    println!("   ✓ Use batch inference when possible");
    println!("   ✓ Profile your application to find bottlenecks");
    println!();
    
    // 10. 性能提示
    println!("8. Performance Tips:");
    println!();
    println!("   • Model Optimization:");
    println!("     - Use model quantization (INT8 > FP16 > FP32)");
    println!("     - Prune unnecessary layers");
    println!("     - Fuse operations when possible");
    println!();
    println!("   • Runtime Optimization:");
    println!("     - Prefer NPU/DSP over GPU/CPU for sustained workloads");
    println!("     - Use GPU for burst performance");
    println!("     - Use CPU as fallback");
    println!();
    println!("   • Memory Optimization:");
    println!("     - Reuse input/output buffers");
    println!("     - Use in-place operations");
    println!("     - Avoid unnecessary data copies");
    println!();
    
    // 11. 平台特定建议
    println!("9. Platform-Specific Recommendations:");
    println!();
    println!("   iOS/macOS:");
    println!("   - Use Core ML with Neural Engine");
    println!("   - Convert models to .mlmodel format");
    println!("   - Use FP16 for best balance");
    println!();
    println!("   Android (Qualcomm):");
    println!("   - Use SNPE with DSP runtime");
    println!("   - Quantize to INT8 for best power efficiency");
    println!("   - Use GPU for image processing");
    println!();
    println!("   Android (MediaTek):");
    println!("   - Use NeuroPilot via Android NN API");
    println!("   - Use TFLite with NNAPI delegate");
    println!("   - Prefer APU for sustained inference");
    println!();
    println!("   Desktop (Intel):");
    println!("   - Use OpenVINO for best performance");
    println!("   - Leverage iGPU when available");
    println!("   - Use INT8 quantization");
    println!();
    println!("   Desktop (AMD):");
    println!("   - Use ROCm for GPU acceleration");
    println!("   - Use ONNX Runtime as fallback");
    println!("   - Consider mixed precision");
    println!();
    
    println!("=== Demo Complete ===");
}
