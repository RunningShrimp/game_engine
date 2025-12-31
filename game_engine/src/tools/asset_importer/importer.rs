//! # 资源导入器（Asset Importer）
//!
//! 负责实际的资源导入和转换工作。

use crate::tools::asset_importer::{
    CompressionFormat, PreviewData,
    detector::{AssetDetector, AssetFormat},
    validator::{AssetValidator, ValidationResult},
};
use std::path::{Path, PathBuf};

/// 资源导入器
pub struct AssetImporter {
    output_directory: PathBuf,
    options: ImportOptions,
}

impl AssetImporter {
    /// 创建新的资源导入器
    pub fn new(output_directory: PathBuf) -> Self {
        Self {
            output_directory,
            options: ImportOptions::default(),
        }
    }

    /// 设置导入选项
    pub fn with_options(mut self, options: ImportOptions) -> Self {
        self.options = options;
        self
    }

    /// 导入单个资源文件
    pub fn import(&self, source_path: &Path) -> Result<ImportResult, ImportError> {
        log::info!("Importing asset: {:?}", source_path);

        // 验证文件
        let validation = AssetValidator::validate(source_path);
        if !validation.is_valid {
            log::warn!("Asset validation failed: {:?}", validation.issues);
            if !self.options.skip_validation {
                return Err(ImportError::ValidationFailed(validation.issues));
            }
        }

        // 检测格式
        let format = AssetDetector::detect_format(source_path)
            .map_err(|e| ImportError::IoError(e.to_string()))?;

        // 生成预览数据
        let preview = self.generate_preview(source_path, format)?;

        // 计算输出路径
        let output_path = self.generate_output_path(source_path, &format)?;

        // 根据格式执行导入
        match format {
            AssetFormat::GLTF => self.import_gltf(source_path, &output_path),
            AssetFormat::FBX => self.import_fbx(source_path, &output_path),
            AssetFormat::OBJ => self.import_obj(source_path, &output_path),
            AssetFormat::Texture => self.import_texture(source_path, &output_path),
            AssetFormat::Audio => self.import_audio(source_path, &output_path),
            AssetFormat::Font => self.import_font(source_path, &output_path),
            AssetFormat::Shader => self.import_shader(source_path, &output_path),
            AssetFormat::Unknown => Err(ImportError::UnknownFormat),
        }?;

        log::info!("Asset imported successfully to: {:?}", output_path);

        Ok(ImportResult {
            source_path: source_path.to_path_buf(),
            output_path,
            format,
            preview,
            validation,
        })
    }

    /// 导入GLTF文件
    fn import_gltf(&self, source: &Path, output: &Path) -> Result<(), ImportError> {
        use std::fs;

        // 创建输出目录
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|e| ImportError::IoError(e.to_string()))?;
        }

        // 复制GLTF文件
        fs::copy(source, output).map_err(|e| ImportError::IoError(e.to_string()))?;

        // 处理关联的资源文件（纹理、二进制数据等）
        if let Some(source_dir) = source.parent() {
            self.copy_associated_assets(source_dir, output.parent().unwrap())?;
        }

        Ok(())
    }

    /// 导入FBX文件
    fn import_fbx(&self, source: &Path, output: &Path) -> Result<(), ImportError> {
        use std::fs;

        // 创建输出目录
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|e| ImportError::IoError(e.to_string()))?;
        }

        // 复制FBX文件
        fs::copy(source, output).map_err(|e| ImportError::IoError(e.to_string()))?;

        Ok(())
    }

    /// 导入OBJ文件
    fn import_obj(&self, source: &Path, output: &Path) -> Result<(), ImportError> {
        use std::fs;

        // 创建输出目录
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|e| ImportError::IoError(e.to_string()))?;
        }

        // 复制OBJ文件
        fs::copy(source, output).map_err(|e| ImportError::IoError(e.to_string()))?;

        // 复制MTL材质文件
        let mtl_source = source.with_extension("mtl");
        if mtl_source.exists() {
            let mtl_output = output.with_extension("mtl");
            fs::copy(&mtl_source, &mtl_output).map_err(|e| ImportError::IoError(e.to_string()))?;
        }

        Ok(())
    }

    /// 导入纹理文件
    fn import_texture(&self, source: &Path, output: &Path) -> Result<(), ImportError> {
        use std::fs;

        // 创建输出目录
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|e| ImportError::IoError(e.to_string()))?;
        }

        // 加载纹理
        let mut img = image::open(source).map_err(|e| ImportError::IoError(e.to_string()))?;

        // 应用导入选项
        if self.options.generate_mipmaps {
            // Mipmap生成在运行时处理
        }

        // 纹理压缩
        match self.options.compression {
            CompressionFormat::None => {
                // 不压缩，保存原始格式
                img.save(output).map_err(|e| ImportError::IoError(e.to_string()))?;
            }
            _ => {
                // 其他压缩格式需要专门的库
                // 这里先保存原始格式
                img.save(output).map_err(|e| ImportError::IoError(e.to_string()))?;
                log::warn!("Texture compression not yet implemented, saving uncompressed");
            }
        }

        Ok(())
    }

    /// 导入音频文件
    fn import_audio(&self, source: &Path, output: &Path) -> Result<(), ImportError> {
        use std::fs;

        // 创建输出目录
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|e| ImportError::IoError(e.to_string()))?;
        }

        // 复制音频文件
        fs::copy(source, output).map_err(|e| ImportError::IoError(e.to_string()))?;

        Ok(())
    }

    /// 导入字体文件
    fn import_font(&self, source: &Path, output: &Path) -> Result<(), ImportError> {
        use std::fs;

        // 创建输出目录
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|e| ImportError::IoError(e.to_string()))?;
        }

        // 复制字体文件
        fs::copy(source, output).map_err(|e| ImportError::IoError(e.to_string()))?;

        Ok(())
    }

    /// 导入着色器文件
    fn import_shader(&self, source: &Path, output: &Path) -> Result<(), ImportError> {
        use std::fs;

        // 创建输出目录
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|e| ImportError::IoError(e.to_string()))?;
        }

        // 复制着色器文件
        fs::copy(source, output).map_err(|e| ImportError::IoError(e.to_string()))?;

        Ok(())
    }

    /// 复制关联的资源文件
    fn copy_associated_assets(
        &self,
        source_dir: &Path,
        output_dir: &Path,
    ) -> Result<(), ImportError> {
        use std::fs;

        // 查找并复制纹理和其他资源
        for entry in fs::read_dir(source_dir).map_err(|e| ImportError::IoError(e.to_string()))? {
            let entry = entry.map_err(|e| ImportError::IoError(e.to_string()))?;
            let path = entry.path();

            // 只复制图片文件
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    match ext_str.to_lowercase().as_str() {
                        "png" | "jpg" | "jpeg" | "tga" | "bmp" => {
                            let filename = path.file_name().unwrap();
                            let dest = output_dir.join(filename);
                            fs::copy(&path, &dest)
                                .map_err(|e| ImportError::IoError(e.to_string()))?;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    /// 生成预览数据
    fn generate_preview(
        &self,
        path: &Path,
        format: AssetFormat,
    ) -> Result<PreviewData, ImportError> {
        match format {
            AssetFormat::Texture => self.generate_texture_preview(path),
            AssetFormat::GLTF | AssetFormat::FBX | AssetFormat::OBJ => {
                self.generate_model_preview(path, format)
            }
            AssetFormat::Audio => self.generate_audio_preview(path),
            _ => {
                let metadata =
                    std::fs::metadata(path).map_err(|e| ImportError::IoError(e.to_string()))?;
                Ok(PreviewData::Unknown {
                    size: metadata.len() as usize,
                    format: format!("{:?}", format),
                })
            }
        }
    }

    /// 生成纹理预览
    fn generate_texture_preview(&self, path: &Path) -> Result<PreviewData, ImportError> {
        let reader =
            image::ImageReader::open(path).map_err(|e| ImportError::IoError(e.to_string()))?;
        let dimensions =
            reader.into_dimensions().map_err(|e| ImportError::IoError(e.to_string()))?;

        Ok(PreviewData::Texture {
            width: dimensions.0,
            height: dimensions.1,
            format: "RGBA8".to_string(),
            size: dimensions.0 as usize * dimensions.1 as usize * 4,
        })
    }

    /// 生成模型预览
    fn generate_model_preview(
        &self,
        path: &Path,
        format: AssetFormat,
    ) -> Result<PreviewData, ImportError> {
        match format {
            AssetFormat::GLTF => self.generate_gltf_preview(path),
            AssetFormat::OBJ => self.generate_obj_preview(path),
            _ => Ok(PreviewData::Model {
                vertices: 0,
                triangles: 0,
                materials: 0,
                animations: 0,
            }),
        }
    }

    /// 生成GLTF预览
    fn generate_gltf_preview(&self, path: &Path) -> Result<PreviewData, ImportError> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path).map_err(|e| ImportError::IoError(e.to_string()))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| ImportError::IoError(e.to_string()))?;

        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(obj) = value.as_object() {
                let meshes =
                    obj.get("meshes").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);

                let materials =
                    obj.get("materials").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);

                let animations =
                    obj.get("animations").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);

                return Ok(PreviewData::Model {
                    vertices: 0,
                    triangles: 0,
                    materials,
                    animations,
                });
            }
        }

        Ok(PreviewData::Model {
            vertices: 0,
            triangles: 0,
            materials: 0,
            animations: 0,
        })
    }

    /// 生成OBJ预览
    fn generate_obj_preview(&self, path: &Path) -> Result<PreviewData, ImportError> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(path).map_err(|e| ImportError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);

        let mut vertices = 0;
        let mut faces = 0;

        for line in reader.lines().take(1000) {
            if let Ok(line) = line {
                let line = line.trim();
                if line.starts_with("v ") {
                    vertices += 1;
                } else if line.starts_with("f ") {
                    faces += 1;
                }
            }
        }

        Ok(PreviewData::Model {
            vertices,
            triangles: faces,
            materials: 0,
            animations: 0,
        })
    }

    /// 生成音频预览
    fn generate_audio_preview(&self, _path: &Path) -> Result<PreviewData, ImportError> {
        // 音频预览需要专门的音频库
        Ok(PreviewData::Audio {
            duration: 0.0,
            channels: 2,
            sample_rate: 44100,
            format: "Unknown".to_string(),
        })
    }

    /// 生成输出路径
    fn generate_output_path(
        &self,
        source: &Path,
        format: &AssetFormat,
    ) -> Result<PathBuf, ImportError> {
        let filename = source
            .file_name()
            .ok_or_else(|| ImportError::InvalidPath("No filename".to_string()))?;

        let output_dir = match format {
            AssetFormat::GLTF | AssetFormat::FBX | AssetFormat::OBJ => {
                self.output_directory.join("models")
            }
            AssetFormat::Texture => self.output_directory.join("textures"),
            AssetFormat::Audio => self.output_directory.join("audio"),
            AssetFormat::Font => self.output_directory.join("fonts"),
            AssetFormat::Shader => self.output_directory.join("shaders"),
            AssetFormat::Unknown => self.output_directory.join("other"),
        };

        Ok(output_dir.join(filename))
    }
}

/// 导入选项
#[derive(Clone, Debug)]
pub struct ImportOptions {
    /// 是否跳过验证
    pub skip_validation: bool,
    /// 是否生成mipmaps
    pub generate_mipmaps: bool,
    /// 是否归一化法线
    pub normalize_normals: bool,
    /// 压缩格式
    pub compression: CompressionFormat,
    /// 质量设置 (0.0 - 1.0)
    pub quality: f32,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            skip_validation: false,
            generate_mipmaps: true,
            normalize_normals: true,
            compression: CompressionFormat::None,
            quality: 1.0,
        }
    }
}

/// 导入结果
#[derive(Clone, Debug)]
pub struct ImportResult {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub format: AssetFormat,
    pub preview: PreviewData,
    pub validation: ValidationResult,
}

/// 导入错误类型
#[derive(thiserror::Error, Debug, Clone)]
pub enum ImportError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Unknown format")]
    UnknownFormat,

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Validation failed: {0:?}")]
    ValidationFailed(Vec<crate::tools::asset_importer::validator::ValidationIssue>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_importer_creation() {
        let importer = AssetImporter::new(PathBuf::from("/tmp/output"));
        assert_eq!(importer.output_directory, PathBuf::from("/tmp/output"));
    }

    #[test]
    fn test_import_options() {
        let options = ImportOptions::default();
        assert!(options.generate_mipmaps);
        assert!(options.normalize_normals);
    }

    #[test]
    fn test_import_texture() {
        // 创建测试图片
        let test_file = "/tmp/test_import.png";
        let img = image::RgbImage::new(100, 100);
        img.save(test_file).unwrap();

        let output_dir = "/tmp/test_output";
        fs::create_dir_all(output_dir).ok();

        let importer = AssetImporter::new(PathBuf::from(output_dir));
        let result = importer.import(Path::new(test_file));

        assert!(result.is_ok());
        let import_result = result.unwrap();
        assert!(import_result.output_path.exists());

        // 清理
        fs::remove_file(test_file).ok();
        fs::remove_dir_all(output_dir).ok();
    }

    #[test]
    fn test_import_obj() {
        let test_file = "/tmp/test_import.obj";
        let mut file = fs::File::create(test_file).unwrap();
        writeln!(file, "v 0.0 0.0 0.0").unwrap();
        writeln!(file, "v 1.0 0.0 0.0").unwrap();
        writeln!(file, "v 1.0 1.0 0.0").unwrap();
        writeln!(file, "f 1 2 3").unwrap();
        drop(file);

        let output_dir = "/tmp/test_output_obj";
        fs::create_dir_all(output_dir).ok();

        let importer = AssetImporter::new(PathBuf::from(output_dir));
        let result = importer.import(Path::new(test_file));

        assert!(result.is_ok());
        let import_result = result.unwrap();
        assert!(import_result.output_path.exists());

        // 清理
        fs::remove_file(test_file).ok();
        fs::remove_dir_all(output_dir).ok();
    }
}
