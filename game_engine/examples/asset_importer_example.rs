//! 资源导入工具示例
//!
//! 展示如何使用资源导入工具的各种功能。

use game_engine::tools::asset_importer::*;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 资源导入工具示例 ===\n");

    // 1. 格式检测示例
    println!("1. 格式检测示例");
    let test_files = vec!["test.gltf", "test.fbx", "test.obj", "test.png", "test.wav"];

    for file in test_files {
        let path = PathBuf::from(file);
        match AssetDetector::detect_format(&path) {
            Ok(format) => println!("  {:?} -> {:?}", file, format),
            Err(e) => println!("  {:?} -> Error: {:?}", file, e),
        }
    }

    // 2. 文件分析示例
    println!("\n2. 文件分析示例");
    let gltf_file = PathBuf::from("test.gltf");

    // 创建测试GLTF文件
    std::fs::write(
        &gltf_file,
        r#"{
        "asset": {"version": "2.0"},
        "scenes": [{"name": "TestScene"}],
        "meshes": [{"name": "TestMesh"}],
        "materials": [{"name": "TestMaterial"}]
    }"#,
    )?;

    if let Ok(analysis) = AssetDetector::analyze_file(&gltf_file) {
        println!("  Format: {:?}", analysis.format);
        println!("  Size: {} bytes", analysis.size);
        println!("  Valid: {}", analysis.is_valid);
        println!("  Metadata:");
        for (key, value) in &analysis.metadata {
            println!("    {}: {}", key, value);
        }
    }

    // 3. 资源验证示例
    println!("\n3. 资源验证示例");
    let validation = AssetValidator::validate(&gltf_file);
    println!("  Valid: {}", validation.is_valid);
    if !validation.issues.is_empty() {
        println!("  Issues:");
        for issue in &validation.issues {
            println!("    - {:?}", issue);
        }
    }
    if !validation.suggestions.is_empty() {
        println!("  Suggestions:");
        for suggestion in &validation.suggestions {
            println!(
                "    - {} (auto: {})",
                suggestion.action, suggestion.automatic
            );
        }
    }

    // 4. 资源导入示例
    println!("\n4. 资源导入示例");
    let output_dir = PathBuf::from("assets/imported");
    std::fs::create_dir_all(&output_dir)?;

    let importer = AssetImporter::new(output_dir.clone()).with_options(ImportOptions {
        generate_mipmaps: true,
        normalize_normals: true,
        compression: CompressionFormat::None,
        quality: 1.0,
        skip_validation: false,
    });

    match importer.import(&gltf_file) {
        Ok(result) => {
            println!("  Import successful!");
            println!("  Source: {:?}", result.source_path);
            println!("  Output: {:?}", result.output_path);
            println!("  Format: {:?}", result.format);
            println!("  Preview: {:?}", result.preview);
        }
        Err(e) => println!("  Import failed: {:?}", e),
    }

    // 5. 批量导入示例
    println!("\n5. 批量导入示例");
    let files = vec![
        PathBuf::from("test.gltf"),
        PathBuf::from("test2.gltf"),
        PathBuf::from("test3.gltf"),
    ];

    // 创建额外的测试文件
    for i in 1..3 {
        std::fs::write(
            format!("test{}.gltf", i + 1),
            r#"{"asset": {"version": "2.0"}}"#,
        )?;
    }

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let mut batch = BatchImporter::new(files.clone(), output_dir.clone()).with_settings(
            BatchImportSettings {
                continue_on_error: true,
                parallel: false,
                max_parallel: 4,
                output_directory: output_dir.clone(),
            },
        );

        match batch.import_all().await {
            Ok(report) => {
                println!("  Total files: {}", report.total_files);
                println!("  Successful: {}", report.successful_imports);
                println!("  Failed: {}", report.failed_imports);
            }
            Err(e) => println!("  Batch import failed: {:?}", e),
        }
    });

    // 6. 导入设置示例
    println!("\n6. 导入设置示例");
    let settings = ImportSettings::default();
    println!("  Generate mipmaps: {}", settings.generate_mipmaps);
    println!("  Normalize normals: {}", settings.normalize_normals);
    println!("  Compression: {:?}", settings.compression);
    println!("  Quality: {:.1}", settings.quality);

    // 7. 压缩格式示例
    println!("\n7. 压缩格式示例");
    let formats = vec![
        CompressionFormat::None,
        CompressionFormat::BC1,
        CompressionFormat::BC3,
    ];
    for format in formats {
        println!("  {:?}", format);
    }

    // 清理测试文件
    println!("\n清理测试文件...");
    std::fs::remove_file("test.gltf").ok();
    std::fs::remove_file("test2.gltf").ok();
    std::fs::remove_file("test3.gltf").ok();
    std::fs::remove_dir_all("assets").ok();

    println!("\n=== 示例完成 ===");
    Ok(())
}
