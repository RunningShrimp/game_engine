//! # 错误修复工具（Asset Fixer）
//!
//! 自动修复资源文件中的常见问题。

use crate::tools::asset_importer::detector::AssetDetector;
use crate::tools::asset_importer::validator::ValidationIssue;
use std::fs;
use std::path::Path;

/// 资源修复器
pub struct AssetFixer;

impl AssetFixer {
    /// 自动修复资源问题
    pub fn auto_fix(path: &Path, issues: &[ValidationIssue]) -> Result<(), FixerError> {
        for issue in issues {
            match issue {
                ValidationIssue::MissingNormals => Self::fix_normals(path)?,
                ValidationIssue::InvalidNormals => Self::fix_normals(path)?,
                ValidationIssue::CorruptedGeometry => Self::fix_geometry(path)?,
                ValidationIssue::MissingMaterials => Self::add_default_material(path)?,
                ValidationIssue::NonPowerOfTwo => Self::resize_texture(path)?,
                ValidationIssue::EmptyFile => {
                    return Err(FixerError::CannotFix("Empty file cannot be fixed".to_string()))
                }
                ValidationIssue::FileNotFound => {
                    return Err(FixerError::CannotFix("File not found".to_string()))
                }
                _ => {
                    // 其他问题可能需要手动修复
                    log::warn!("Issue '{:?}' cannot be automatically fixed", issue);
                }
            }
        }
        Ok(())
    }

    /// 修复法线
    fn fix_normals(path: &Path) -> Result<(), FixerError> {
        let format = AssetDetector::detect_format(path)
            .map_err(|e| FixerError::IoError(e.to_string()))?;
        match format {
            crate::tools::asset_importer::detector::AssetFormat::OBJ => {
                Self::fix_obj_normals(path)
            }
            _ => Ok(()), // 其他格式的法线修复需要专门的库
        }
    }

    /// 修复OBJ文件法线
    fn fix_obj_normals(path: &Path) -> Result<(), FixerError> {
        use std::fs::File;
        use std::io::{BufRead, BufReader, Write};

        let file = File::open(path).map_err(|e| FixerError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);

        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut faces: Vec<Vec<usize>> = Vec::new();
        let mut other_lines = Vec::new();

        // 解析OBJ文件
        for line in reader.lines() {
            let line = line.map_err(|e| FixerError::IoError(e.to_string()))?;
            let line = line.trim();

            if line.starts_with("v ") {
                // 解析顶点
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let v: Vec<f32> = parts[1..4]
                        .iter()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if v.len() == 3 {
                        vertices.push([v[0], v[1], v[2]]);
                    }
                }
            } else if line.starts_with("f ") {
                // 解析面
                let parts: Vec<&str> = line.split_whitespace().collect();
                let face_indices: Vec<usize> = parts[1..]
                    .iter()
                    .filter_map(|s| {
                        s.split('/')
                            .next()
                            .and_then(|idx| idx.parse::<usize>().ok())
                    })
                    .collect();
                if !face_indices.is_empty() {
                    faces.push(face_indices);
                }
            } else {
                other_lines.push(line.to_string());
            }
        }

        // 计算面法线
        let mut normals = vec![[0.0f32; 3]; vertices.len()];
        for face in &faces {
            if face.len() >= 3 {
                for i in 0..face.len() {
                    let v0 = &vertices[face[i] - 1];
                    let v1 = &vertices[face[(i + 1) % face.len()] - 1];
                    let v2 = &vertices[face[(i + 2) % face.len()] - 1];

                    let edge1 = [
                        v1[0] - v0[0],
                        v1[1] - v0[1],
                        v1[2] - v0[2],
                    ];
                    let edge2 = [
                        v2[0] - v0[0],
                        v2[1] - v0[1],
                        v2[2] - v0[2],
                    ];

                    // 叉积计算法线
                    let normal = [
                        edge1[1] * edge2[2] - edge1[2] * edge2[1],
                        edge1[2] * edge2[0] - edge1[0] * edge2[2],
                        edge1[0] * edge2[1] - edge1[1] * edge2[0],
                    ];

                    // 归一化
                    let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
                        .sqrt();
                    if len > 0.0 {
                        let normalized = [
                            normal[0] / len,
                            normal[1] / len,
                            normal[2] / len,
                        ];
                        normals[face[i] - 1][0] += normalized[0];
                        normals[face[i] - 1][1] += normalized[1];
                        normals[face[i] - 1][2] += normalized[2];
                    }
                }
            }
        }

        // 归一化顶点法线
        for normal in &mut normals {
            let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            if len > 0.0 {
                normal[0] /= len;
                normal[1] /= len;
                normal[2] /= len;
            }
        }

        // 写回文件
        let mut output = File::create(path).map_err(|e| FixerError::IoError(e.to_string()))?;

        // 写入顶点
        for v in &vertices {
            writeln!(output, "v {} {} {}", v[0], v[1], v[2])
                .map_err(|e| FixerError::IoError(e.to_string()))?;
        }

        // 写入法线
        for n in &normals {
            writeln!(output, "vn {} {} {}", n[0], n[1], n[2])
                .map_err(|e| FixerError::IoError(e.to_string()))?;
        }

        // 写入其他行
        for line in &other_lines {
            writeln!(output, "{}", line).map_err(|e| FixerError::IoError(e.to_string()))?;
        }

        // 写入面（更新索引包含法线）
        for face in &faces {
            let face_str: Vec<String> = face.iter().map(|i| format!("{}/{}", i, i)).collect();
            writeln!(output, "f {}", face_str.join(" "))
                .map_err(|e| FixerError::IoError(e.to_string()))?;
        }

        Ok(())
    }

    /// 修复几何体
    fn fix_geometry(path: &Path) -> Result<(), FixerError> {
        // 几何修复需要专门的3D模型库
        log::warn!("Geometry fixing requires specialized 3D model library");
        Err(FixerError::NotImplemented)
    }

    /// 添加默认材质
    fn add_default_material(path: &Path) -> Result<(), FixerError> {
        let format = AssetDetector::detect_format(path)
            .map_err(|e| FixerError::IoError(e.to_string()))?;
        match format {
            crate::tools::asset_importer::detector::AssetFormat::GLTF => {
                Self::add_default_gltf_material(path)
            }
            crate::tools::asset_importer::detector::AssetFormat::OBJ => {
                Self::add_default_obj_material(path)
            }
            _ => Ok(()),
        }
    }

    /// 为GLTF添加默认材质
    fn add_default_gltf_material(path: &Path) -> Result<(), FixerError> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path).map_err(|e| FixerError::IoError(e.to_string()))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| FixerError::IoError(e.to_string()))?;

        if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(obj) = value.as_object_mut() {
                // 添加默认材质
                let default_material = serde_json::json!({
                    "name": "DefaultMaterial",
                    "pbrMetallicRoughness": {
                        "baseColorFactor": [1.0, 1.0, 1.0, 1.0],
                        "metallicFactor": 0.0,
                        "roughnessFactor": 1.0
                    }
                });

                if obj.get("materials").is_none() {
                    obj.insert("materials".to_string(), serde_json::json!([default_material]));
                }

                // 更新文件
                let updated = serde_json::to_string_pretty(&value)
                    .map_err(|e| FixerError::IoError(e.to_string()))?;
                fs::write(path, updated)
                    .map_err(|e| FixerError::IoError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// 为OBJ添加默认材质
    fn add_default_obj_material(path: &Path) -> Result<(), FixerError> {
        // OBJ材质定义在.mtl文件中，这里创建一个简单的.mtl文件
        let mtl_path = path.with_extension("mtl");
        let mtl_content = r#"# Default material
newmtl default
Ka 1.0 1.0 1.0
Kd 1.0 1.0 1.0
Ks 0.0 0.0 0.0
Ns 10.0
d 1.0
"#;

        fs::write(&mtl_path, mtl_content)
            .map_err(|e| FixerError::IoError(e.to_string()))?;

        // 在OBJ文件中添加材质库引用
        use std::fs::File;
        use std::io::{BufRead, BufReader, Write};

        let file = File::open(path).map_err(|e| FixerError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);
        let mut lines: Vec<String> = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| FixerError::IoError(e.to_string()))?;
            lines.push(line);
        }

        // 在开头添加mtllib
        let mtl_line = format!("mtllib {}", mtl_path.file_name().unwrap().to_str().unwrap());
        lines.insert(0, mtl_line);
        lines.insert(1, "usemtl default".to_string());

        // 写回文件
        let mut output = File::create(path).map_err(|e| FixerError::IoError(e.to_string()))?;
        for line in lines {
            writeln!(output, "{}", line).map_err(|e| FixerError::IoError(e.to_string()))?;
        }

        Ok(())
    }

    /// 调整纹理大小到2的幂
    fn resize_texture(path: &Path) -> Result<(), FixerError> {
        use image::ImageReader;

        let reader = ImageReader::open(path).map_err(|e| FixerError::IoError(e.to_string()))?;

        if let Ok(dimensions) = reader.into_dimensions() {
            let (width, height) = dimensions;

            if !width.is_power_of_two() || !height.is_power_of_two() {
                let new_width = width.next_power_of_two();
                let new_height = height.next_power_of_two();

                log::info!(
                    "Resizing texture from {}x{} to {}x{}",
                    width,
                    height,
                    new_width,
                    new_height
                );

                // 使用image crate进行缩放
                if let Ok(img) = image::open(path) {
                    let resized = image::imageops::resize(
                        &img,
                        new_width,
                        new_height,
                        image::imageops::FilterType::Lanczos3,
                    );

                    let new_path = path.with_extension(format!("resized.{}", path.extension().unwrap().to_str().unwrap()));
                    resized
                        .save(&new_path)
                        .map_err(|e| FixerError::IoError(e.to_string()))?;

                    log::info!("Resized texture saved to: {:?}", new_path);
                }
            }
        }

        Ok(())
    }
}

/// 修复器错误类型
#[derive(thiserror::Error, Debug)]
pub enum FixerError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Cannot fix: {0}")]
    CannotFix(String),

    #[error("Not implemented")]
    NotImplemented,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_fix_obj_normals() {
        let test_file = "/tmp/test_fix_normals.obj";
        let mut file = fs::File::create(test_file).unwrap();
        writeln!(file, "v 0.0 0.0 0.0").unwrap();
        writeln!(file, "v 1.0 0.0 0.0").unwrap();
        writeln!(file, "v 1.0 1.0 0.0").unwrap();
        writeln!(file, "f 1 2 3").unwrap();
        drop(file);

        let result = AssetFixer::fix_normals(Path::new(test_file));
        assert!(result.is_ok());

        // 检查是否添加了法线
        let content = fs::read_to_string(test_file).unwrap();
        assert!(content.contains("vn "));

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_add_default_gltf_material() {
        let test_file = "/tmp/test_add_material.gltf";
        let mut file = fs::File::create(test_file).unwrap();
        writeln!(file, "{{\"asset\": {{\"version\": \"2.0\"}}}}").unwrap();
        drop(file);

        let result = AssetFixer::add_default_material(Path::new(test_file));
        assert!(result.is_ok());

        // 检查是否添加了materials
        let content = fs::read_to_string(test_file).unwrap();
        assert!(content.contains("materials"));

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_auto_fix() {
        let test_file = "/tmp/test_auto_fix.obj";
        let mut file = fs::File::create(test_file).unwrap();
        writeln!(file, "v 0.0 0.0 0.0").unwrap();
        writeln!(file, "v 1.0 0.0 0.0").unwrap();
        writeln!(file, "v 1.0 1.0 0.0").unwrap();
        writeln!(file, "f 1 2 3").unwrap();
        drop(file);

        let issues = vec![ValidationIssue::MissingNormals];
        let result = AssetFixer::auto_fix(Path::new(test_file), &issues);
        assert!(result.is_ok());

        fs::remove_file(test_file).ok();
    }
}
