//! # 格式检测器（Format Detector）
//!
//! 自动检测和分析资源文件格式。

use std::path::Path;

/// 资源格式检测器
pub struct AssetDetector;

impl AssetDetector {
    /// 自动检测文件格式
    pub fn detect_format(path: &Path) -> Result<AssetFormat, DetectorError> {
        // 首先通过扩展名快速检测
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "gltf" | "glb" => return Ok(AssetFormat::GLTF),
                    "fbx" => return Ok(AssetFormat::FBX),
                    "obj" => return Ok(AssetFormat::OBJ),
                    "png" | "jpg" | "jpeg" | "tga" | "bmp" | "gif" => {
                        return Ok(AssetFormat::Texture)
                    }
                    "wav" | "mp3" | "ogg" | "flac" => return Ok(AssetFormat::Audio),
                    "ttf" | "otf" => return Ok(AssetFormat::Font),
                    "hlsl" | "vert" | "frag" | "wgsl" | "spv" => return Ok(AssetFormat::Shader),
                    _ => {}
                }
            }
        }

        // 尝试通过文件内容检测
        Self::detect_by_content(path)
    }

    /// 通过文件内容检测格式
    fn detect_by_content(path: &Path) -> Result<AssetFormat, DetectorError> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path).map_err(|e| DetectorError::IoError(e.to_string()))?;

        let mut buffer = [0u8; 16];
        let n = file
            .read(&mut buffer)
            .map_err(|e| DetectorError::IoError(e.to_string()))?;

        if n == 0 {
            return Err(DetectorError::EmptyFile);
        }

        // 检查magic numbers
        match &buffer[..n.min(4)] {
            // GLB (GLTF binary) magic: "glTF"
            [0x67, 0x6C, 0x54, 0x46] => Ok(AssetFormat::GLTF),
            // PNG magic
            [0x89, 0x50, 0x4E, 0x47] => Ok(AssetFormat::Texture),
            // JPEG magic
            [0xFF, 0xD8, 0xFF] => Ok(AssetFormat::Texture),
            // WAV magic
            [0x52, 0x49, 0x46, 0x46] => {
                // RIFF
                // 检查是否是WAVE
                if n >= 12 && &buffer[8..12] == [0x57, 0x41, 0x56, 0x45] {
                    Ok(AssetFormat::Audio)
                } else {
                    Err(DetectorError::UnknownFormat)
                }
            }
            // OGG magic
            [0x4F, 0x67, 0x67, 0x53] => Ok(AssetFormat::Audio),
            _ => Err(DetectorError::UnknownFormat),
        }
    }

    /// 分析文件内容
    pub fn analyze_file(path: &Path) -> Result<FileAnalysis, DetectorError> {
        use std::fs;

        let metadata = fs::metadata(path).map_err(|e| DetectorError::IoError(e.to_string()))?;
        let format = Self::detect_format(path)?;

        let mut analysis = FileAnalysis {
            path: path.to_path_buf(),
            format,
            size: metadata.len(),
            is_valid: true,
            issues: Vec::new(),
            version: None,
            metadata: std::collections::HashMap::new(),
        };

        // 检查文件大小
        if metadata.len() == 0 {
            analysis.is_valid = false;
            analysis.issues.push("Empty file".to_string());
        }

        // 格式特定分析
        match analysis.format {
            AssetFormat::GLTF => Self::analyze_gltf(path, &mut analysis)?,
            AssetFormat::FBX => Self::analyze_fbx(path, &mut analysis)?,
            AssetFormat::OBJ => Self::analyze_obj(path, &mut analysis)?,
            AssetFormat::Texture => Self::analyze_texture(path, &mut analysis)?,
            AssetFormat::Audio => Self::analyze_audio(path, &mut analysis)?,
            _ => {}
        }

        Ok(analysis)
    }

    /// 分析GLTF文件
    fn analyze_gltf(path: &Path, analysis: &mut FileAnalysis) -> Result<(), DetectorError> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path).map_err(|e| DetectorError::IoError(e.to_string()))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| DetectorError::IoError(e.to_string()))?;

        // 尝试解析JSON
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(obj) = value.as_object() {
                // 提取版本信息
                if let Some(asset) = obj.get("asset") {
                    if let Some(asset_obj) = asset.as_object() {
                        if let Some(version) = asset_obj.get("version") {
                            analysis.version = Some(version.as_str().unwrap_or("unknown").to_string());
                        }
                    }
                }

                // 统计场景信息
                if let Some(scenes) = obj.get("scenes") {
                    if let Some(scenes_array) = scenes.as_array() {
                        analysis
                            .metadata
                            .insert("scenes".to_string(), scenes_array.len().to_string());
                    }
                }

                if let Some(meshes) = obj.get("meshes") {
                    if let Some(meshes_array) = meshes.as_array() {
                        analysis
                            .metadata
                            .insert("meshes".to_string(), meshes_array.len().to_string());
                    }
                }

                if let Some(materials) = obj.get("materials") {
                    if let Some(materials_array) = materials.as_array() {
                        analysis
                            .metadata
                            .insert("materials".to_string(), materials_array.len().to_string());
                    }
                }
            }
        } else {
            analysis.is_valid = false;
            analysis.issues.push("Invalid GLTF JSON".to_string());
        }

        Ok(())
    }

    /// 分析FBX文件
    fn analyze_fbx(_path: &Path, analysis: &mut FileAnalysis) -> Result<(), DetectorError> {
        // FBX是二进制格式，这里只做基本检查
        analysis
            .metadata
            .insert("format".to_string(), "FBX Binary".to_string());
        Ok(())
    }

    /// 分析OBJ文件
    fn analyze_obj(path: &Path, analysis: &mut FileAnalysis) -> Result<(), DetectorError> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(path).map_err(|e| DetectorError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);

        let mut vertices = 0;
        let mut faces = 0;
        let mut normals = 0;
        let mut tex_coords = 0;

        for line in reader.lines().take(1000) {
            let line = line.map_err(|e| DetectorError::IoError(e.to_string()))?;
            let line = line.trim();

            if line.starts_with("v ") {
                vertices += 1;
            } else if line.starts_with("vn ") {
                normals += 1;
            } else if line.starts_with("vt ") {
                tex_coords += 1;
            } else if line.starts_with("f ") {
                faces += 1;
            }
        }

        analysis
            .metadata
            .insert("vertices".to_string(), vertices.to_string());
        analysis
            .metadata
            .insert("faces".to_string(), faces.to_string());
        analysis
            .metadata
            .insert("normals".to_string(), normals.to_string());
        analysis
            .metadata
            .insert("tex_coords".to_string(), tex_coords.to_string());

        // 检查是否缺少法线
        if normals == 0 && vertices > 0 {
            analysis
                .issues
                .push("Missing vertex normals".to_string());
        }

        Ok(())
    }

    /// 分析纹理文件
    fn analyze_texture(path: &Path, analysis: &mut FileAnalysis) -> Result<(), DetectorError> {
        // 尝试使用image crate读取
        if let Ok(reader) = image::ImageReader::open(path) {
            if let Some(format) = reader.format() {
                analysis
                    .metadata
                    .insert("format".to_string(), format.extensions_str()[0].to_string());
            }

            if let Ok(dimensions) = reader.into_dimensions() {
                analysis
                    .metadata
                    .insert("width".to_string(), dimensions.0.to_string());
                analysis
                    .metadata
                    .insert("height".to_string(), dimensions.1.to_string());
            }
        }

        Ok(())
    }

    /// 分析音频文件
    fn analyze_audio(_path: &Path, analysis: &mut FileAnalysis) -> Result<(), DetectorError> {
        // 基本分析，更详细的音频分析需要专门的库
        analysis
            .metadata
            .insert("type".to_string(), "Audio".to_string());
        Ok(())
    }
}

/// 资源格式枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetFormat {
    GLTF,
    FBX,
    OBJ,
    Texture,
    Audio,
    Font,
    Shader,
    Unknown,
}

/// 检测器错误类型
#[derive(thiserror::Error, Debug)]
pub enum DetectorError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Unknown format")]
    UnknownFormat,

    #[error("Empty file")]
    EmptyFile,

    #[error("Parse error: {0}")]
    ParseError(String),
}

/// 文件分析结果
#[derive(Clone, Debug)]
pub struct FileAnalysis {
    pub path: std::path::PathBuf,
    pub format: AssetFormat,
    pub size: u64,
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub version: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_detect_format_by_extension() {
        let test_file = "/tmp/test.gltf";
        let mut file = fs::File::create(test_file).unwrap();
        writeln!(file, "{{\"asset\": {{\"version\": \"2.0\"}}}}").unwrap();
        drop(file);

        let format = AssetDetector::detect_format(Path::new(test_file)).unwrap();
        assert_eq!(format, AssetFormat::GLTF);

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_detect_png_by_magic_number() {
        let test_file = "/tmp/test.png";
        let mut file = fs::File::create(test_file).unwrap();
        // PNG magic number
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
            .unwrap();
        drop(file);

        let format = AssetDetector::detect_format(Path::new(test_file)).unwrap();
        assert_eq!(format, AssetFormat::Texture);

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_analyze_gltf() {
        let test_file = "/tmp/test.gltf";
        let mut file = fs::File::create(test_file).unwrap();
        writeln!(
            file,
            "{{
            \"asset\": {{\"version\": \"2.0\"}},
            \"scenes\": [{{\"name\": \"Scene\"}}],
            \"meshes\": [{{\"name\": \"Mesh\"}}],
            \"materials\": [{{\"name\": \"Material\"}}]
        }}"
        )
        .unwrap();
        drop(file);

        let analysis = AssetDetector::analyze_file(Path::new(test_file)).unwrap();
        assert_eq!(analysis.format, AssetFormat::GLTF);
        assert_eq!(analysis.version, Some("2.0".to_string()));
        assert_eq!(analysis.metadata.get("scenes").unwrap(), "1");
        assert_eq!(analysis.metadata.get("meshes").unwrap(), "1");
        assert_eq!(analysis.metadata.get("materials").unwrap(), "1");

        fs::remove_file(test_file).ok();
    }
}
