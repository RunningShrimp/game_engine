//! # 资源验证器（Asset Validator）
//!
//! 验证资源文件的完整性和检测常见问题。

use crate::tools::asset_importer::detector::{AssetDetector, AssetFormat};
use std::fs;
use std::path::Path;

/// 资源验证器
pub struct AssetValidator;

impl AssetValidator {
    /// 验证资源文件
    pub fn validate(path: &Path) -> ValidationResult {
        let mut issues = Vec::new();

        // 检查文件是否存在
        if !path.exists() {
            issues.push(ValidationIssue::FileNotFound);
            return ValidationResult {
                is_valid: false,
                issues,
                suggestions: vec![],
            };
        }

        // 检查文件大小
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.len() == 0 {
                issues.push(ValidationIssue::EmptyFile);
            }
        }

        // 检测格式并执行格式特定验证
        match AssetDetector::detect_format(path) {
            Ok(format) => match format {
                AssetFormat::GLTF => Self::validate_gltf(path, &mut issues),
                AssetFormat::FBX => Self::validate_fbx(path, &mut issues),
                AssetFormat::OBJ => Self::validate_obj(path, &mut issues),
                AssetFormat::Texture => Self::validate_texture(path, &mut issues),
                AssetFormat::Audio => Self::validate_audio(path, &mut issues),
                _ => {}
            },
            Err(_) => {
                issues.push(ValidationIssue::UnknownFormat);
            }
        }

        let suggestions = Self::generate_fix_suggestions(&issues);

        ValidationResult {
            is_valid: issues.is_empty(),
            issues,
            suggestions,
        }
    }

    /// 验证GLTF文件
    fn validate_gltf(path: &Path, issues: &mut Vec<ValidationIssue>) {
        use std::fs::File;
        use std::io::Read;

        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(_) => {
                issues.push(ValidationIssue::CannotRead);
                return;
            }
        };

        let mut content = String::new();
        if file.read_to_string(&mut content).is_err() {
            issues.push(ValidationIssue::CannotRead);
            return;
        }

        // 尝试解析JSON
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(obj) = value.as_object() {
                // 检查asset.version
                if let Some(asset) = obj.get("asset") {
                    if let Some(asset_obj) = asset.as_object() {
                        if asset_obj.get("version").is_none() {
                            issues.push(ValidationIssue::MissingVersion);
                        }
                    }
                } else {
                    issues.push(ValidationIssue::MissingAsset);
                }

                // 检查场景
                if obj.get("scenes").is_none() && obj.get("scene").is_none() {
                    issues.push(ValidationIssue::MissingScene);
                }

                // 检查是否有网格但没有材质
                if obj.get("meshes").is_some() && obj.get("materials").is_none() {
                    issues.push(ValidationIssue::MissingMaterials);
                }
            }
        } else {
            issues.push(ValidationIssue::InvalidJson);
        }
    }

    /// 验证FBX文件
    fn validate_fbx(path: &Path, issues: &mut Vec<ValidationIssue>) {
        // 基本检查
        if let Ok(metadata) = fs::metadata(path) {
            // FBX文件通常不会太小
            if metadata.len() < 100 {
                issues.push(ValidationIssue::CorruptedGeometry);
            }
        }
    }

    /// 验证OBJ文件
    fn validate_obj(path: &Path, issues: &mut Vec<ValidationIssue>) {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => {
                issues.push(ValidationIssue::CannotRead);
                return;
            }
        };

        let reader = BufReader::new(file);
        let mut has_vertices = false;
        let mut has_faces = false;
        let mut has_normals = false;

        for line in reader.lines().take(1000) {
            if let Ok(line) = line {
                let line = line.trim();
                if line.starts_with("v ") {
                    has_vertices = true;
                } else if line.starts_with("f ") {
                    has_faces = true;
                } else if line.starts_with("vn ") {
                    has_normals = true;
                }
            }
        }

        if !has_vertices {
            issues.push(ValidationIssue::NoGeometry);
        }

        if !has_faces {
            issues.push(ValidationIssue::NoFaces);
        }

        if has_vertices && !has_normals {
            issues.push(ValidationIssue::MissingNormals);
        }
    }

    /// 验证纹理文件
    fn validate_texture(path: &Path, issues: &mut Vec<ValidationIssue>) {
        use image::ImageReader;

        // 尝试打开并解码图片
        if let Err(_) = ImageReader::open(path).and_then(|reader| reader.decode()) {
            issues.push(ValidationIssue::CorruptedTexture);
        }

        // 检查纹理尺寸是否是2的幂
        if let Ok(reader) = ImageReader::open(path) {
            if let Ok(dimensions) = reader.dimensions() {
                let (width, height) = dimensions;
                if !width.is_power_of_two() || !height.is_power_of_two() {
                    issues.push(ValidationIssue::NonPowerOfTwo);
                }
            }
        }
    }

    /// 验证音频文件
    fn validate_audio(_path: &Path, _issues: &mut Vec<ValidationIssue>) {
        // 基本验证，更详细的验证需要专门的音频库
    }

    /// 生成修复建议
    fn generate_fix_suggestions(issues: &[ValidationIssue]) -> Vec<FixSuggestion> {
        issues
            .iter()
            .map(|issue| match issue {
                ValidationIssue::MissingTexture => FixSuggestion {
                    action: "Add default texture".to_string(),
                    description: "Add a default white texture to missing material slots"
                        .to_string(),
                    automatic: true,
                },
                ValidationIssue::MissingNormals => FixSuggestion {
                    action: "Generate normals".to_string(),
                    description: "Automatically calculate vertex normals from geometry".to_string(),
                    automatic: true,
                },
                ValidationIssue::CorruptedGeometry => FixSuggestion {
                    action: "Repair geometry".to_string(),
                    description: "Attempt to repair corrupted mesh data".to_string(),
                    automatic: false,
                },
                ValidationIssue::NonPowerOfTwo => FixSuggestion {
                    action: "Resize texture".to_string(),
                    description:
                        "Resize texture to power of two dimensions for better GPU compatibility"
                            .to_string(),
                    automatic: true,
                },
                ValidationIssue::MissingMaterials => FixSuggestion {
                    action: "Create default material".to_string(),
                    description: "Create a default white material for meshes without materials"
                        .to_string(),
                    automatic: true,
                },
                ValidationIssue::InvalidNormals => FixSuggestion {
                    action: "Recalculate normals".to_string(),
                    description: "Recalculate face and vertex normals with proper smoothing"
                        .to_string(),
                    automatic: true,
                },
                _ => FixSuggestion {
                    action: "Manual review required".to_string(),
                    description: format!("Issue '{:?}' requires manual review", issue),
                    automatic: false,
                },
            })
            .collect()
    }
}

/// 验证问题类型
#[derive(Clone, Debug, PartialEq)]
pub enum ValidationIssue {
    FileNotFound,
    EmptyFile,
    CannotRead,
    UnknownFormat,
    InvalidJson,
    MissingAsset,
    MissingVersion,
    MissingScene,
    MissingMaterials,
    MissingTexture,
    MissingNormals,
    InvalidNormals,
    NoGeometry,
    NoFaces,
    CorruptedGeometry,
    CorruptedTexture,
    NonPowerOfTwo,
}

/// 验证结果
#[derive(Clone, Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub suggestions: Vec<FixSuggestion>,
}

/// 修复建议
#[derive(Clone, Debug)]
pub struct FixSuggestion {
    pub action: String,
    pub description: String,
    pub automatic: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_validate_nonexistent_file() {
        let result = AssetValidator::validate(Path::new("/nonexistent/file.gltf"));
        assert!(!result.is_valid);
        assert!(result.issues.contains(&ValidationIssue::FileNotFound));
    }

    #[test]
    fn test_validate_empty_file() {
        let test_file = "/tmp/test_empty.gltf";
        let mut file = fs::File::create(test_file).unwrap();
        drop(file);

        let result = AssetValidator::validate(Path::new(test_file));
        assert!(!result.is_valid);
        assert!(result.issues.contains(&ValidationIssue::EmptyFile));

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_validate_valid_gltf() {
        let test_file = "/tmp/test_valid.gltf";
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

        let result = AssetValidator::validate(Path::new(test_file));
        assert!(result.is_valid);

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_validate_invalid_gltf() {
        let test_file = "/tmp/test_invalid.gltf";
        let mut file = fs::File::create(test_file).unwrap();
        writeln!(file, "not valid json {{{}").unwrap();
        drop(file);

        let result = AssetValidator::validate(Path::new(test_file));
        assert!(!result.is_valid);
        assert!(result.issues.contains(&ValidationIssue::InvalidJson));

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_validate_obj_without_normals() {
        let test_file = "/tmp/test_no_normals.obj";
        let mut file = fs::File::create(test_file).unwrap();
        writeln!(file, "v 0.0 0.0 0.0").unwrap();
        writeln!(file, "v 1.0 0.0 0.0").unwrap();
        writeln!(file, "v 1.0 1.0 0.0").unwrap();
        writeln!(file, "f 1 2 3").unwrap();
        drop(file);

        let result = AssetValidator::validate(Path::new(test_file));
        assert!(result.issues.contains(&ValidationIssue::MissingNormals));

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_generate_fix_suggestions() {
        let issues = vec![
            ValidationIssue::MissingNormals,
            ValidationIssue::NonPowerOfTwo,
        ];

        let suggestions = AssetValidator::generate_fix_suggestions(&issues);
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions[0].automatic); // Generate normals
        assert!(suggestions[1].automatic); // Resize texture
    }
}
