/// ONNX Runtime真实集成示例
/// 
/// 展示如何使用ort crate进行AI推理

fn main() {
    println!("=== ONNX Runtime真实集成示例 ===\n");
    
    println!("本示例展示如何使用ort crate进行AI推理。");
    println!();
    
    // 1. 依赖配置
    println!("1. 添加依赖到Cargo.toml:");
    println!("   [dependencies]");
    println!("   ort = \"=2.0.0-rc.10\"");
    println!("   ndarray = \"0.16\"");
    println!();
    
    // 2. 基本使用
    println!("2. 基本使用示例:");
    println!("   ```rust");
    println!("   use ort::session::{{builder::GraphOptimizationLevel, Session}};");
    println!("   use ndarray::Array4;");
    println!();
    println!("   // 创建会话");
    println!("   let session = Session::builder()?");
    println!("       .with_optimization_level(GraphOptimizationLevel::Level3)?");
    println!("       .with_intra_threads(4)?");
    println!("       .commit_from_file(\"model.onnx\")?;");
    println!();
    println!("   // 准备输入");
    println!("   let input = Array4::<f32>::zeros((1, 3, 224, 224));");
    println!();
    println!("   // 执行推理");
    println!("   let outputs = session.run(ort::inputs![\"input\" => input.view()]?)?;");
    println!("   let result = outputs[\"output\"].try_extract_tensor::<f32>()?;");
    println!("   ```");
    println!();
    
    // 3. 使用CUDA加速
    println!("3. 使用CUDA加速:");
    println!("   ```rust");
    println!("   use ort::{{session::Session, ExecutionProvider}};");
    println!();
    println!("   let session = Session::builder()?");
    println!("       .with_execution_providers([");
    println!("           ExecutionProvider::CUDA(Default::default())");
    println!("       ])?");
    println!("       .commit_from_file(\"model.onnx\")?;");
    println!("   ```");
    println!();
    
    // 4. 图像分类完整示例
    println!("4. 图像分类完整示例:");
    println!("   ```rust");
    println!("   use ort::session::Session;");
    println!("   use ndarray::{{Array4, s}};");
    println!("   use image::{{DynamicImage, GenericImageView}};");
    println!();
    println!("   // 加载模型");
    println!("   let session = Session::builder()?");
    println!("       .commit_from_file(\"resnet50.onnx\")?;");
    println!();
    println!("   // 加载并预处理图像");
    println!("   let img = image::open(\"cat.jpg\")?;");
    println!("   let img = img.resize_exact(224, 224, image::imageops::FilterType::Triangle);");
    println!();
    println!("   // 转换为模型输入格式");
    println!("   let mut input = Array4::<f32>::zeros((1, 3, 224, 224));");
    println!("   for (x, y, pixel) in img.pixels() {{");
    println!("       let rgb = pixel.0;");
    println!("       input[[0, 0, y as usize, x as usize]] = rgb[0] as f32 / 255.0;");
    println!("       input[[0, 1, y as usize, x as usize]] = rgb[1] as f32 / 255.0;");
    println!("       input[[0, 2, y as usize, x as usize]] = rgb[2] as f32 / 255.0;");
    println!("   }}");
    println!();
    println!("   // 执行推理");
    println!("   let outputs = session.run(ort::inputs![\"input\" => input.view()]?)?;");
    println!("   let predictions = outputs[\"output\"].try_extract_tensor::<f32>()?;");
    println!();
    println!("   // 获取预测类别");
    println!("   let class_id = predictions.iter()");
    println!("       .enumerate()");
    println!("       .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())");
    println!("       .map(|(idx, _)| idx)");
    println!("       .unwrap();");
    println!();
    println!("   println!(\"Predicted class: {{}}\", class_id);");
    println!("   ```");
    println!();
    
    // 5. 文本处理示例
    println!("5. 文本处理（BERT）示例:");
    println!("   ```rust");
    println!("   use ort::session::Session;");
    println!("   use ndarray::Array2;");
    println!();
    println!("   // 加载BERT模型");
    println!("   let session = Session::builder()?");
    println!("       .commit_from_file(\"bert-base-uncased.onnx\")?;");
    println!();
    println!("   // 准备输入（已经tokenized）");
    println!("   let input_ids = Array2::<i64>::from_shape_vec(");
    println!("       (1, 128),");
    println!("       vec![101, 2023, 2003, 1037, 3231, /* ... */, 102]");
    println!("   )?;");
    println!();
    println!("   let attention_mask = Array2::<i64>::ones((1, 128));");
    println!();
    println!("   // 执行推理");
    println!("   let outputs = session.run(ort::inputs![");
    println!("       \"input_ids\" => input_ids.view(),");
    println!("       \"attention_mask\" => attention_mask.view()");
    println!("   ]?)?;");
    println!();
    println!("   let embeddings = outputs[\"last_hidden_state\"].try_extract_tensor::<f32>()?;");
    println!("   ```");
    println!();
    
    // 6. 性能优化建议
    println!("6. 性能优化建议:");
    println!("   - 使用GraphOptimizationLevel::Level3获得最佳性能");
    println!("   - 根据CPU核心数设置intra_threads");
    println!("   - 使用批量推理提高吞吐量");
    println!("   - 对于GPU，使用CUDA或TensorRT执行提供者");
    println!("   - 使用IoBinding预分配内存减少拷贝");
    println!();
    
    // 7. 支持的执行提供者
    println!("7. 支持的执行提供者:");
    println!("   - CPU: 默认，无需配置");
    println!("   - CUDA: NVIDIA GPU加速");
    println!("   - TensorRT: NVIDIA GPU优化推理");
    println!("   - CoreML: Apple平台（macOS/iOS）");
    println!("   - DirectML: Windows DirectX加速");
    println!("   - OpenVINO: Intel硬件加速");
    println!("   - QNN: Qualcomm NPU加速");
    println!("   - CANN: Huawei Ascend NPU");
    println!();
    
    // 8. 模型转换
    println!("8. 将模型转换为ONNX:");
    println!("   PyTorch:");
    println!("     torch.onnx.export(model, dummy_input, \"model.onnx\")");
    println!();
    println!("   TensorFlow:");
    println!("     python -m tf2onnx.convert --saved-model model_dir --output model.onnx");
    println!();
    println!("   Hugging Face:");
    println!("     optimum-cli export onnx --model bert-base-uncased bert-onnx/");
    println!();
    
    // 9. 常见问题
    println!("9. 常见问题:");
    println!("   Q: 如何查看模型的输入输出名称？");
    println!("   A: 使用Netron查看器：https://netron.app/");
    println!();
    println!("   Q: 推理速度慢？");
    println!("   A: 1) 使用GPU执行提供者");
    println!("      2) 启用图优化");
    println!("      3) 使用批量推理");
    println!();
    println!("   Q: 内存占用高？");
    println!("   A: 1) 使用量化模型");
    println!("      2) 减小批量大小");
    println!("      3) 使用memory_pattern(false)");
    println!();
    
    println!("=== 示例完成 ===");
    println!();
    println!("更多信息请访问:");
    println!("  - ort文档: https://ort.pyke.io/");
    println!("  - ONNX Runtime: https://onnxruntime.ai/");
    println!("  - 示例代码: https://github.com/pykeio/ort/tree/main/examples");
}
