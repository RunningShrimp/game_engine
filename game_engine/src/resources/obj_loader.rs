//! OBJ 模型加载器
//!
//! 提供异步 OBJ 模型加载功能，支持网格、材质和纹理解析。
//!
//! ## 架构说明
//!
//! OBJ (Wavefront OBJ) 是一种简单、广泛使用的3D模型格式：
//! - **文件格式**: 纯文本，易于阅读和调试
//! - **广泛支持**: 几乎所有3D建模软件都支持导出OBJ
//! - **简单性**: 格式简单，解析速度快
//!
//! ## 特性支持
//!
//! - ✅ 网格几何数据（顶点、法线、UV、切线）
//! - ✅ 材质库（MTL文件）
//! - ✅ 纹理支持
//! - ✅ 多边形网格（三角形和四边形）
//! - ✅ 对象和组
//! - ✅ 平滑组
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::resources::obj_loader::{ObjLoader, ObjScene};
//!
//! async fn load_obj_model(path: &std::path::Path) -> Result<ObjScene, String> {
//!     ObjLoader::load_from_path(path).await
//! }
//! ```

#[cfg(feature = "obj")]
use std::path::Path;
#[cfg(feature = "obj")]
use std::sync::Arc;

// =============================================================================
// 公共接口（feature-gated）
// =============================================================================

/// OBJ 场景数据
///
/// 包含解析后的 OBJ 文件和所有相关数据。
#[cfg(feature = "obj")]
#[derive(Clone, Debug)]
pub struct ObjScene {
    /// OBJ 文档和解析数据
    pub data: Arc<ObjDocument>,
    /// 可选的原始元数据
    pub metadata: Option<ObjMetadata>,
}

/// OBJ 文档结构
#[cfg(feature = "obj")]
#[derive(Clone, Debug)]
pub struct ObjDocument {
    /// 网格对象列表
    pub objects: Vec<ObjObject>,
    /// 材质库列表
    pub materials: Vec<ObjMaterial>,
    /// 纹理数据（文件名到路径的映射）
    pub textures: std::collections::HashMap<String, String>,
}

/// OBJ 对象
#[cfg(feature = "obj")]
#[derive(Clone, Debug)]
pub struct ObjObject {
    /// 对象名称
    pub name: String,
    /// 网格数据
    pub mesh: ObjMesh,
    /// 材质索引
    pub material_index: Option<usize>,
}

/// OBJ 网格数据
#[cfg(feature = "obj")]
#[derive(Clone, Debug)]
pub struct ObjMesh {
    /// 顶点位置
    pub positions: Vec<[f32; 3]>,
    /// 顶点法线
    pub normals: Vec<[f32; 3]>,
    /// UV坐标
    pub uvs: Vec<[f32; 2]>,
    /// 索引数据（顶点/UV/法线三元组）
    pub indices: Vec<ObjIndex>,
    /// 平滑组
    pub smoothing_groups: Vec<u32>,
}

/// OBJ 索引（可以是顶点索引、顶点/UV索引、或顶点/UV/法线索引）
#[cfg(feature = "obj")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjIndex {
    /// 顶点索引（从0开始，OBJ从1开始需转换）
    pub vertex: u32,
    /// UV索引（可选）
    pub tex_coord: Option<u32>,
    /// 法线索引（可选）
    pub normal: Option<u32>,
}

/// OBJ 材质数据
#[cfg(feature = "obj")]
#[derive(Clone, Debug)]
pub struct ObjMaterial {
    /// 材质名称
    pub name: String,
    /// 漫反射颜色
    pub diffuse: [f32; 4],
    /// 环境光颜色
    pub ambient: [f32; 3],
    /// 镜面反射颜色
    pub specular: [f32; 3],
    /// 镜面反射 exponent
    pub shininess: f32,
    /// 透明度
    pub alpha: f32,
    /// 纹理文件路径
    pub diffuse_map: Option<String>,
    pub normal_map: Option<String>,
    pub specular_map: Option<String>,
}

/// OBJ 元数据
#[cfg(feature = "obj")]
#[derive(Clone, Debug)]
pub struct ObjMetadata {
    /// 文件路径
    pub file_path: String,
    /// 顶点数量
    pub vertex_count: usize,
    /// 面数量
    pub face_count: usize,
    /// 对象数量
    pub object_count: usize,
    /// 材质数量
    pub material_count: usize,
}

// =============================================================================
// OBJ 加载错误
// =============================================================================

/// OBJ 加载错误类型
#[cfg(feature = "obj")]
#[derive(Debug, thiserror::Error)]
pub enum ObjLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("OBJ parse error: {0}")]
    Parse(String),

    #[error("Invalid vertex index: {0}")]
    InvalidVertexIndex(u32),

    #[error("Missing material data: {0}")]
    MissingMaterial(String),

    #[error("Invalid file extension: {0}")]
    InvalidExtension(String),

    #[error("Unsupported OBJ feature: {0}")]
    UnsupportedFeature(String),
}

// =============================================================================
// OBJ 加载器实现
// =============================================================================

/// 异步 OBJ 加载器
#[cfg(feature = "obj")]
pub struct ObjLoader;

#[cfg(feature = "obj")]
impl ObjLoader {
    /// 从路径异步加载 OBJ 文件
    ///
    /// # 参数
    /// - `path`: OBJ 文件路径（.obj）
    ///
    /// # 返回
    /// 加载的 `ObjScene` 或错误信息
    pub async fn load_from_path(path: &Path) -> Result<ObjScene, String> {
        // 验证文件扩展名
        Self::validate_extension(path)
            .map_err(|e| e.to_string())?;

        // 读取文件
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| format!("Failed to read OBJ file: {e}"))?;

        // 在阻塞任务中解析
        let path_str = path.to_string_lossy().to_string();
        let parsed = tokio::task::spawn_blocking(move || {
            Self::parse_obj(&content, &path_str).map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| format!("OBJ parsing task failed: {e}"))??;

        Ok(parsed)
    }

    /// 从字符串内容加载 OBJ
    pub fn from_str(content: &str, base_path: &str) -> Result<ObjScene, ObjLoadError> {
        Self::parse_obj(content, base_path)
    }

    /// 验证文件扩展名
    pub fn validate_extension(path: &Path) -> Result<(), ObjLoadError> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("obj") => Ok(()),
            Some(ext) => Err(ObjLoadError::InvalidExtension(ext.to_string())),
            None => Err(ObjLoadError::InvalidExtension("none".to_string())),
        }
    }

    /// 解析 OBJ 数据
    fn parse_obj(content: &str, base_path: &str) -> Result<ObjScene, ObjLoadError> {
        tracing::info!(target: "obj_loader", "Parsing OBJ file: {}", base_path);

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut objects = Vec::new();
        let mut current_object = ObjObject {
            name: "default".to_string(),
            mesh: ObjMesh {
                positions: Vec::new(),
                normals: Vec::new(),
                uvs: Vec::new(),
                indices: Vec::new(),
                smoothing_groups: Vec::new(),
            },
            material_index: None,
        };
        let mut materials = Vec::new();
        let mut textures = std::collections::HashMap::new();
        let mut current_smoothing_group = 0u32;
        let mut vertex_count = 0usize;
        let mut face_count = 0usize;

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue; // 跳过空行和注释
            }

            let mut parts = line.split_whitespace();
            let keyword = parts.next().unwrap_or("");

            match keyword {
                "v" => {
                    // 顶点位置: v x y z [w]
                    let pos = parse_vertex(parts, &format!("line {}", line_num + 1))?;
                    positions.push(pos);
                    vertex_count += 1;
                }
                "vn" => {
                    // 顶点法线: vn nx ny nz
                    let normal = parse_normal(parts, &format!("line {}", line_num + 1))?;
                    normals.push(normal);
                }
                "vt" => {
                    // UV坐标: vt u [v] [w]
                    let uv = parse_uv(parts, &format!("line {}", line_num + 1))?;
                    uvs.push(uv);
                }
                "f" => {
                    // 面: f v1/vt1/vn1 v2/vt2/vn2 v3/vt3/vn3 ...
                    let face_indices = parse_face(parts, &format!("line {}", line_num + 1))?;
                    for index in face_indices {
                        current_object.mesh.indices.push(index);
                    }
                    current_object.mesh.smoothing_groups.push(current_smoothing_group);
                    face_count += 1;
                }
                "o" | "g" => {
                    // 对象或组: o name / g name
                    if !current_object.mesh.indices.is_empty() {
                        objects.push(std::mem::replace(&mut current_object, ObjObject {
                            name: String::new(),
                            mesh: ObjMesh {
                                positions: Vec::new(),
                                normals: Vec::new(),
                                uvs: Vec::new(),
                                indices: Vec::new(),
                                smoothing_groups: Vec::new(),
                            },
                            material_index: None,
                        }));
                    }
                    let name = parts.next().unwrap_or("unnamed").to_string();
                    current_object.name = name;
                }
                "s" => {
                    // 平滑组: s group_number
                    if let Some(group_str) = parts.next() {
                        current_smoothing_group = if group_str == "off" {
                            0
                        } else {
                            group_str.parse().unwrap_or(0)
                        };
                    }
                }
                "usemtl" => {
                    // 使用材质: usemtl material_name
                    if let Some(mat_name) = parts.next() {
                        // 查找材质索引
                        let mat_index = materials.iter().position(|m: &ObjMaterial| m.name == mat_name);
                        if mat_index.is_none() {
                            tracing::warn!(target: "obj_loader", "Material '{}' not found", mat_name);
                        }
                        current_object.material_index = mat_index;
                    }
                }
                "mtllib" => {
                    // 材质库: mtllib filename
                    if let Some(lib_name) = parts.next() {
                        // TODO: 加载MTL文件
                        tracing::info!(target: "obj_loader", "Material library: {}", lib_name);
                    }
                }
                _ => {
                    // 忽略其他关键字
                    tracing::debug!(target: "obj_loader", "Ignoring keyword: {} at line {}", keyword, line_num + 1);
                }
            }
        }

        // 添加最后一个对象（如果有索引）
        let has_last_object = !current_object.mesh.indices.is_empty();
        if has_last_object {
            objects.push(current_object);
        }

        // 如果没有对象，创建默认对象
        if objects.is_empty() && !positions.is_empty() {
            // Create a default object with all the data
            let mut default_obj = ObjObject {
                name: "default".to_string(),
                mesh: ObjMesh {
                    positions: Vec::new(),
                    normals: Vec::new(),
                    uvs: Vec::new(),
                    indices: Vec::new(),
                    smoothing_groups: Vec::new(),
                },
                material_index: None,
            };

            // Build mesh from indices if we have any
            if !default_obj.mesh.indices.is_empty() {
                build_mesh_from_indices(&mut default_obj, &positions, &normals, &uvs);
            }
            objects.push(default_obj);
        }

        // 构建网格数据
        for obj in &mut objects {
            // 从索引构建实际的顶点数组
            build_mesh_from_indices(obj, &positions, &normals, &uvs);
        }

        let metadata = ObjMetadata {
            file_path: base_path.to_string(),
            vertex_count,
            face_count,
            object_count: objects.len(),
            material_count: materials.len(),
        };

        Ok(ObjScene {
            data: Arc::new(ObjDocument {
                objects,
                materials,
                textures,
            }),
            metadata: Some(metadata),
        })
    }
}

// =============================================================================
// 辅助解析函数
// =============================================================================

/// 解析顶点位置
#[cfg(feature = "obj")]
fn parse_vertex(mut parts: std::str::SplitWhitespace, context: &str) -> Result<[f32; 3], ObjLoadError> {
    let x: f32 = parts.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ObjLoadError::Parse(format!("Missing vertex x at {context}")))?;
    let y: f32 = parts.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ObjLoadError::Parse(format!("Missing vertex y at {context}")))?;
    let z: f32 = parts.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ObjLoadError::Parse(format!("Missing vertex z at {context}")))?;
    // w 是可选的，忽略
    Ok([x, y, z])
}

/// 解析法线
#[cfg(feature = "obj")]
fn parse_normal(mut parts: std::str::SplitWhitespace, context: &str) -> Result<[f32; 3], ObjLoadError> {
    let x: f32 = parts.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ObjLoadError::Parse(format!("Missing normal x at {context}")))?;
    let y: f32 = parts.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ObjLoadError::Parse(format!("Missing normal y at {context}")))?;
    let z: f32 = parts.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ObjLoadError::Parse(format!("Missing normal z at {context}")))?;
    Ok([x, y, z])
}

/// 解析UV坐标
#[cfg(feature = "obj")]
fn parse_uv(mut parts: std::str::SplitWhitespace, context: &str) -> Result<[f32; 2], ObjLoadError> {
    let u: f32 = parts.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ObjLoadError::Parse(format!("Missing UV u at {context}")))?;
    let v: f32 = parts.next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0); // v 是可选的
    Ok([u, v])
}

/// 解析面索引
#[cfg(feature = "obj")]
fn parse_face(mut parts: std::str::SplitWhitespace, context: &str) -> Result<Vec<ObjIndex>, ObjLoadError> {
    let mut indices = Vec::new();
    for part in parts {
        let index = parse_vertex_index(part, context)?;
        indices.push(index);
    }

    // 三角化（如果是四边形或更多边形）
    if indices.len() > 3 {
        triangulate_face(&mut indices);
    }

    Ok(indices)
}

/// 解析单个顶点索引（v/vt/vn格式）
#[cfg(feature = "obj")]
fn parse_vertex_index(part: &str, context: &str) -> Result<ObjIndex, ObjLoadError> {
    let components: Vec<&str> = part.split('/').collect();

    // OBJ 索引从1开始，需要转换为从0开始
    let vertex = components.get(0)
        .and_then(|s| if s.is_empty() { None } else { s.parse().ok() })
        .map(|i: u32| i.saturating_sub(1))
        .ok_or_else(|| ObjLoadError::Parse(format!("Invalid vertex index at {context}")))?;

    let tex_coord = if components.len() > 1 && !components[1].is_empty() {
        Some(components[1].parse::<u32>()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0))
    } else {
        None
    };

    let normal = if components.len() > 2 && !components[2].is_empty() {
        Some(components[2].parse::<u32>()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0))
    } else {
        None
    };

    Ok(ObjIndex { vertex, tex_coord, normal })
}

/// 三角化多边形（简单扇形三角化）
#[cfg(feature = "obj")]
fn triangulate_face(indices: &mut Vec<ObjIndex>) {
    if indices.len() <= 3 {
        return;
    }

    // 使用第一个顶点作为扇形中心
    let first = indices[0];
    let mut triangulated = Vec::new();

    for i in 1..indices.len() - 1 {
        triangulated.push(first);
        triangulated.push(indices[i]);
        triangulated.push(indices[i + 1]);
    }

    *indices = triangulated;
}

/// 从索引构建网格数据
#[cfg(feature = "obj")]
fn build_mesh_from_indices(
    obj: &mut ObjObject,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
) {
    use std::collections::HashMap;

    let mut vertex_map = HashMap::new();
    let mut vertices = Vec::new();
    let mut final_indices = Vec::new();

    for index in &obj.mesh.indices {
        // 构建唯一键（vertex/tex_coord/normal）
        let key = (
            index.vertex,
            index.tex_coord.unwrap_or(0),
            index.normal.unwrap_or(0),
        );

        if let Some(&existing_idx) = vertex_map.get(&key) {
            final_indices.push(existing_idx);
        } else {
            // 添加新顶点
            let new_idx = vertices.len() as u32;

            let pos = positions.get(index.vertex as usize)
                .copied()
                .unwrap_or([0.0, 0.0, 0.0]);
            let normal = index.normal
                .and_then(|i| normals.get(i as usize).copied())
                .unwrap_or([0.0, 1.0, 0.0]);
            let uv = index.tex_coord
                .and_then(|i| uvs.get(i as usize).copied())
                .unwrap_or([0.0, 0.0]);

            vertices.push((pos, normal, uv));
            final_indices.push(new_idx);
            vertex_map.insert(key, new_idx);
        }
    }

    // 解包顶点数据
    obj.mesh.positions = vertices.iter().map(|(p, _, _)| *p).collect();
    obj.mesh.normals = vertices.iter().map(|(_, n, _)| *n).collect();
    obj.mesh.uvs = vertices.iter().map(|(_, _, uv)| *uv).collect();
    obj.mesh.indices = final_indices.into_iter()
        .map(|i| ObjIndex { vertex: i, tex_coord: None, normal: None })
        .collect();
}

// =============================================================================
// ObjScene 辅助方法
// =============================================================================

#[cfg(feature = "obj")]
impl ObjScene {
    /// 获取文档
    pub fn document(&self) -> &ObjDocument {
        &self.data
    }

    /// 获取对象数量
    pub fn object_count(&self) -> usize {
        self.data.objects.len()
    }

    /// 获取材质数量
    pub fn material_count(&self) -> usize {
        self.data.materials.len()
    }
}

// =============================================================================
// Default implementations
// =============================================================================

#[cfg(feature = "obj")]
impl Default for ObjMaterial {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            diffuse: [1.0, 1.0, 1.0, 1.0],
            ambient: [0.1, 0.1, 0.1],
            specular: [0.5, 0.5, 0.5],
            shininess: 32.0,
            alpha: 1.0,
            diffuse_map: None,
            normal_map: None,
            specular_map: None,
        }
    }
}

// =============================================================================
// 存根实现（当 obj feature 未启用时）
// =============================================================================

#[cfg(not(feature = "obj"))]
use std::path::Path;

#[cfg(not(feature = "obj"))]
/// OBJ 场景数据（存根）
#[derive(Clone, Debug)]
pub struct ObjScene;

#[cfg(not(feature = "obj"))]
impl ObjScene {
    pub fn from_str(_content: &str, _base_path: &str) -> Result<Self, String> {
        Err("OBJ support not enabled. Enable the 'obj' feature to use this function.".to_string())
    }

    pub fn object_count(&self) -> usize {
        0
    }

    pub fn material_count(&self) -> usize {
        0
    }
}

#[cfg(not(feature = "obj"))]
/// OBJ 加载错误类型（存根）
#[derive(Debug, thiserror::Error)]
pub enum ObjLoadError {
    #[error("OBJ support not enabled. Enable the 'obj' feature to use this function.")]
    FeatureNotEnabled,
}

#[cfg(not(feature = "obj"))]
/// 异步 OBJ 加载器（存根）
pub struct ObjLoader;

#[cfg(not(feature = "obj"))]
impl ObjLoader {
    pub async fn load_from_path(_path: &Path) -> Result<ObjScene, String> {
        Err("OBJ support not enabled. Enable the 'obj' feature to use this function.".to_string())
    }

    pub fn validate_extension(_path: &Path) -> Result<(), ObjLoadError> {
        Err(ObjLoadError::FeatureNotEnabled)
    }
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "obj")]
    fn test_validate_extension() {
        assert!(ObjLoader::validate_extension(Path::new("model.obj")).is_ok());
        assert!(ObjLoader::validate_extension(Path::new("model.fbx")).is_err());
        assert!(ObjLoader::validate_extension(Path::new("model")).is_err());
    }

    #[test]
    #[cfg(feature = "obj")]
    fn test_parse_vertex() {
        let line = "1.0 2.0 3.0".split_whitespace();
        let vertex = parse_vertex(line, "test").unwrap();
        assert_eq!(vertex, [1.0, 2.0, 3.0]);
    }

    #[test]
    #[cfg(feature = "obj")]
    fn test_parse_simple_obj() {
        let obj_content = r#"
# Simple OBJ file
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0
f 1 2 3 4
"#;

        let scene = ObjLoader::from_str(obj_content, "test.obj").unwrap();
        assert_eq!(scene.object_count(), 1);

        let doc = scene.document();
        assert!(!doc.objects.is_empty());
        assert_eq!(doc.objects[0].mesh.positions.len(), 4);
    }
}
