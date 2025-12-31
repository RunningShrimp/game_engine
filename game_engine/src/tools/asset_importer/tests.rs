//! # 资源导入工具集成测试
//!
//! 测试整个导入工作流程。

use crate::tools::asset_importer::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// 创建测试用的OBJ文件
fn create_test_obj(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    writeln!(file, "# Test OBJ file")?;
    writeln!(file, "v 0.0 0.0 0.0")?;
    writeln!(file, "v 1.0 0.0 0.0")?;
    writeln!(file, "v 1.0 1.0 0.0")?;
    writeln!(file, "v 0.0 1.0 0.0")?;
    writeln!(file, "f 1 2 3 4")?;
    Ok(())
}

/// 创建测试用的PNG文件
fn create_test_png(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::RgbImage::new(100, 100);
    img.save(path)?;
    Ok(())
}

/// 创建测试用的GLTF文件
fn create_test_gltf(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "{{
        \"asset\": {{\"version\": \"2.0\"}},
        \"scenes\": [{{\"name\": \"TestScene\"}}],
        \"meshes\": [{{\"name\": \"TestMesh\"}}],
        \"materials\": [{{\"name\": \"TestMaterial\"}}]
    }}"
    )?;
    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_import_workflow() {
        // 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        let input_dir = temp_dir.path().join("input");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // 创建测试文件
        let obj_file = input_dir.join("test.obj");
        let png_file = input_dir.join("test.png");
        let gltf_file = input_dir.join("test.gltf");

        create_test_obj(&obj_file).unwrap();
        create_test_png(&png_file).unwrap();
        create_test_gltf(&gltf_file).unwrap();

        // 1. 测试格式检测
        let obj_format = AssetDetector::detect_format(&obj_file).unwrap();
        assert_eq!(obj_format, AssetFormat::OBJ);

        let png_format = AssetDetector::detect_format(&png_file).unwrap();
        assert_eq!(png_format, AssetFormat::Texture);

        let gltf_format = AssetDetector::detect_format(&gltf_file).unwrap();
        assert_eq!(gltf_format, AssetFormat::GLTF);

        // 2. 测试文件分析
        let obj_analysis = AssetDetector::analyze_file(&obj_file).unwrap();
        assert_eq!(obj_analysis.format, AssetFormat::OBJ);
        assert!(obj_analysis.is_valid);

        let gltf_analysis = AssetDetector::analyze_file(&gltf_file).unwrap();
        assert_eq!(gltf_analysis.format, AssetFormat::GLTF);
        assert!(gltf_analysis.is_valid);

        // 3. 测试验证
        let obj_validation = AssetValidator::validate(&obj_file);
        assert!(obj_validation.is_valid);

        let gltf_validation = AssetValidator::validate(&gltf_file);
        assert!(gltf_validation.is_valid);

        // 4. 测试导入
        let importer = AssetImporter::new(output_dir.clone());
        let obj_result = importer.import(&obj_file);
        assert!(obj_result.is_ok());

        let png_result = importer.import(&png_file);
        assert!(png_result.is_ok());

        let gltf_result = importer.import(&gltf_file);
        assert!(gltf_result.is_ok());

        // 验证输出文件存在
        assert!(output_dir.join("models/test.obj").exists());
        assert!(output_dir.join("textures/test.png").exists());
        assert!(output_dir.join("models/test.gltf").exists());
    }

    #[test]
    fn test_validation_and_fix_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let input_dir = temp_dir.path().join("input");
        fs::create_dir_all(&input_dir).unwrap();

        // 创建一个缺少法线的OBJ文件
        let obj_file = input_dir.join("no_normals.obj");
        let mut file = File::create(&obj_file).unwrap();
        writeln!(file, "v 0.0 0.0 0.0").unwrap();
        writeln!(file, "v 1.0 0.0 0.0").unwrap();
        writeln!(file, "v 1.0 1.0 0.0").unwrap();
        writeln!(file, "f 1 2 3").unwrap();
        drop(file);

        // 验证应该检测到缺少法线
        let validation = AssetValidator::validate(&obj_file);
        assert!(!validation.is_valid);
        assert!(validation
            .issues
            .contains(&ValidationIssue::MissingNormals));

        // 修复应该添加法线
        let result = AssetFixer::auto_fix(&obj_file, &validation.issues);
        assert!(result.is_ok());

        // 重新验证应该通过
        let validation_after = AssetValidator::validate(&obj_file);
        assert!(validation_after.is_valid);
    }

    #[test]
    fn test_batch_import() {
        use tokio::runtime::Runtime;

        let temp_dir = TempDir::new().unwrap();
        let input_dir = temp_dir.path().join("input");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // 创建多个测试文件
        let files = vec![
            input_dir.join("test1.obj"),
            input_dir.join("test2.obj"),
            input_dir.join("test3.obj"),
        ];

        for file in &files {
            create_test_obj(file).unwrap();
        }

        // 批量导入
        let rt = Runtime::new().unwrap();
        let mut batch = BatchImporter::new(files.clone(), output_dir.clone());

        rt.block_on(async {
            let report = batch.import_all().await.unwrap();
            assert_eq!(report.total_files, 3);
            assert_eq!(report.successful_imports, 3);
            assert_eq!(report.failed_imports, 0);
        });
    }

    #[test]
    fn test_format_detection_by_magic_number() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.bin");

        // 创建PNG magic number文件
        let mut file = File::create(&test_file).unwrap();
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
            .unwrap();
        drop(file);

        let format = AssetDetector::detect_format(&test_file).unwrap();
        assert_eq!(format, AssetFormat::Texture);
    }

    #[test]
    fn test_import_options() {
        let options = ImportOptions::default();
        assert!(options.generate_mipmaps);
        assert!(options.normalize_normals);
        assert_eq!(options.compression, CompressionFormat::None);
        assert_eq!(options.quality, 1.0);
    }

    #[test]
    fn test_preview_data() {
        let texture_preview = PreviewData::Texture {
            width: 512,
            height: 512,
            format: "RGBA8".to_string(),
            size: 512 * 512 * 4,
        };

        match texture_preview {
            PreviewData::Texture { width, height, .. } => {
                assert_eq!(width, 512);
                assert_eq!(height, 512);
            }
            _ => panic!("Expected texture preview"),
        }

        let model_preview = PreviewData::Model {
            vertices: 1000,
            triangles: 500,
            materials: 2,
            animations: 0,
        };

        match model_preview {
            PreviewData::Model { vertices, triangles, .. } => {
                assert_eq!(vertices, 1000);
                assert_eq!(triangles, 500);
            }
            _ => panic!("Expected model preview"),
        }
    }

    #[test]
    fn test_wizard_step_transitions() {
        let wizard = AssetImportWizard::new();
        assert_eq!(wizard.current_step(), WizardStep::FileSelection);
    }

    #[test]
    fn test_compression_format_variants() {
        let formats = vec![
            CompressionFormat::None,
            CompressionFormat::BC1,
            CompressionFormat::BC2,
            CompressionFormat::BC3,
            CompressionFormat::BC4,
            CompressionFormat::BC5,
        ];

        for format in formats {
            // 确保所有格式都能正确创建和比较
            assert_eq!(format, format);
        }
    }

    #[test]
    fn test_asset_format_detection() {
        let temp_dir = TempDir::new().unwrap();

        let test_cases = vec![
            ("test.gltf", AssetFormat::GLTF),
            ("test.glb", AssetFormat::GLTF),
            ("test.fbx", AssetFormat::FBX),
            ("test.obj", AssetFormat::OBJ),
            ("test.png", AssetFormat::Texture),
            ("test.jpg", AssetFormat::Texture),
            ("test.wav", AssetFormat::Audio),
            ("test.mp3", AssetFormat::Audio),
        ];

        for (filename, expected_format) in test_cases {
            let file_path = temp_dir.path().join(filename);
            File::create(&file_path).unwrap();

            let detected = AssetDetector::detect_format(&file_path).unwrap();
            assert_eq!(
                detected, expected_format,
                "Failed for {}",
                filename
            );
        }
    }
}

// 扩展AssetImportWizard以支持测试
#[cfg(test)]
impl AssetImportWizard {
    pub fn current_step(&self) -> WizardStep {
        self.current_step
    }

    pub fn files_count(&self) -> usize {
        self.files.len()
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }
}
