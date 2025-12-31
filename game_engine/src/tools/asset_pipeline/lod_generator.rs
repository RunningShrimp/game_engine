//! # LOD Generator - 多级细节生成器
//!
//! 本模块实现自动LOD（Level of Detail）生成功能。

use super::pipeline::{AssetMetadata, OptimizationError};
use gltf::{Gltf, Node};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// LOD生成器
pub struct LODGenerator {
    quality_thresholds: Vec<f32>,
}

impl LODGenerator {
    /// 创建新的LOD生成器
    pub fn new(quality_thresholds: Vec<f32>) -> Self {
        Self { quality_thresholds }
    }

    /// 为模型生成多级LOD
    pub async fn generate_lods(
        &self,
        model_path: &Path,
    ) -> Result<Vec<LODModel>, OptimizationError> {
        // 读取原始模型
        let original = self.load_model(model_path).await?;

        let mut lods = vec![original.clone()]; // LOD0 = 原始

        println!("Generating LODs for: {}", model_path.display());

        for (i, threshold) in self.quality_thresholds.iter().enumerate() {
            if *threshold >= 1.0 {
                continue; // 跳过LOD0
            }

            let lod_level = i + 1;
            println!(
                "  Generating LOD{} (threshold: {:.2})...",
                lod_level, threshold
            );

            let lod = self.generate_lod_level(&original, *threshold, lod_level).await?;
            let reduction = (1.0 - threshold) * 100.0;

            println!(
                "  LOD{} complete: {} triangles ({:.1}% reduction)",
                lod_level,
                lod.triangle_count(),
                reduction
            );

            lods.push(lod);
        }

        Ok(lods)
    }

    /// 加载模型
    async fn load_model(&self, path: &Path) -> Result<LODModel, OptimizationError> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| OptimizationError::LODError("No file extension".to_string()))?;

        match extension.to_lowercase().as_str() {
            "gltf" | "glb" => self.load_gltf(path).await,
            "obj" => self.load_obj(path).await,
            _ => Err(OptimizationError::LODError(format!(
                "Unsupported model format: {}",
                extension
            ))),
        }
    }

    /// 加载GLTF模型
    async fn load_gltf(&self, path: &Path) -> Result<LODModel, OptimizationError> {
        let file = fs::File::open(path)
            .map_err(|e| OptimizationError::LODError(format!("Failed to open GLTF: {}", e)))?;

        let gltf = Gltf::from_reader(file)
            .map_err(|e| OptimizationError::LODError(format!("Failed to parse GLTF: {}", e)))?;

        let meshes = self.extract_meshes_from_gltf(&gltf)?;

        Ok(LODModel {
            path: path.to_path_buf(),
            meshes,
            lod_level: 0,
        })
    }

    /// 从GLTF提取网格数据
    fn extract_meshes_from_gltf(&self, gltf: &Gltf) -> Result<Vec<Mesh>, OptimizationError> {
        let mut meshes = Vec::new();

        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| {
                    let buffer_data = match buffer.source() {
                        gltf::buffer::Source::Bin => None,
                        gltf::buffer::Source::Uri(uri) => {
                            // TODO: 从URI加载外部buffer
                            None
                        }
                    };
                    buffer_data.map(|d: &[u8]| d).map(std::borrow::Cow::Borrowed)
                });

                // 提取位置数据
                let positions = if let Some(iter) = reader.read_positions() {
                    iter.collect()
                } else {
                    continue;
                };

                // 提取索引数据
                let indices = if let Some(iter) = reader.read_indices() {
                    iter.into_u32().collect()
                } else {
                    // 生成默认索引
                    (0..(positions.len() as u32)).collect()
                };

                // 提取法线数据
                let normals = if let Some(iter) = reader.read_normals() {
                    Some(iter.collect())
                } else {
                    None
                };

                // 提取UV数据
                let uvs = if let Some(iter) = reader.read_tex_coords(0) {
                    Some(iter.into_f32().collect())
                } else {
                    None
                };

                meshes.push(Mesh {
                    positions,
                    normals,
                    uvs,
                    indices,
                });
            }
        }

        Ok(meshes)
    }

    /// 加载OBJ模型
    async fn load_obj(&self, path: &Path) -> Result<LODModel, OptimizationError> {
        let content = fs::read_to_string(path)
            .map_err(|e| OptimizationError::LODError(format!("Failed to read OBJ: {}", e)))?;

        let mesh = self.parse_obj(&content)?;

        Ok(LODModel {
            path: path.to_path_buf(),
            meshes: vec![mesh],
            lod_level: 0,
        })
    }

    /// 解析OBJ文件
    fn parse_obj(&self, content: &str) -> Result<Mesh, OptimizationError> {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        let mut vertex_data: Vec<(usize, Option<usize>, Option<usize>)> = Vec::new();
        let mut temp_vertices = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "v" => {
                    // 顶点位置
                    if parts.len() >= 4 {
                        let x = parts[1].parse().unwrap_or(0.0);
                        let y = parts[2].parse().unwrap_or(0.0);
                        let z = parts[3].parse().unwrap_or(0.0);
                        positions.push([x, y, z]);
                    }
                }
                "vn" => {
                    // 顶点法线
                    if parts.len() >= 4 {
                        let x = parts[1].parse().unwrap_or(0.0);
                        let y = parts[2].parse().unwrap_or(0.0);
                        let z = parts[3].parse().unwrap_or(0.0);
                        normals.push([x, y, z]);
                    }
                }
                "vt" => {
                    // 纹理坐标
                    if parts.len() >= 3 {
                        let u = parts[1].parse().unwrap_or(0.0);
                        let v = parts[2].parse().unwrap_or(0.0);
                        uvs.push([u, v]);
                    }
                }
                "f" => {
                    // 面（三角形）
                    if parts.len() >= 4 {
                        let mut face_vertices = Vec::new();
                        for i in 1..4.min(parts.len()) {
                            let vertex = self.parse_obj_vertex(parts[i]);
                            face_vertices.push(vertex);
                        }

                        // 三角化四边形
                        if face_vertices.len() == 4 {
                            temp_vertices.push(face_vertices[0]);
                            temp_vertices.push(face_vertices[1]);
                            temp_vertices.push(face_vertices[2]);

                            temp_vertices.push(face_vertices[0]);
                            temp_vertices.push(face_vertices[2]);
                            temp_vertices.push(face_vertices[3]);
                        } else {
                            temp_vertices.extend(&face_vertices);
                        }
                    }
                }
                _ => {}
            }
        }

        // 构建最终网格数据
        for (v_idx, vt_idx, vn_idx) in temp_vertices {
            let idx = vertex_data.len() as u32;
            vertex_data.push((v_idx, vt_idx, vn_idx));
            indices.push(idx);
        }

        let mesh_positions = vertex_data.iter().map(|(v_idx, _, _)| positions[*v_idx]).collect();

        let mesh_normals = if !normals.is_empty() {
            Some(
                vertex_data
                    .iter()
                    .map(|(_, _, vn_idx)| vn_idx.map_or([0.0, 0.0, 0.0], |i| normals[i]))
                    .collect(),
            )
        } else {
            None
        };

        let mesh_uvs = if !uvs.is_empty() {
            Some(
                vertex_data
                    .iter()
                    .map(|(_, vt_idx, _)| vt_idx.map_or([0.0, 0.0], |i| uvs[i]))
                    .collect(),
            )
        } else {
            None
        };

        Ok(Mesh {
            positions: mesh_positions,
            normals: mesh_normals,
            uvs: mesh_uvs,
            indices,
        })
    }

    /// 解析OBJ顶点定义
    fn parse_obj_vertex(&self, vertex_str: &str) -> (usize, Option<usize>, Option<usize>) {
        let parts: Vec<&str> = vertex_str.split('/').collect();

        let v_idx = if !parts[0].is_empty() {
            parts[0].parse().unwrap_or(1)
        } else {
            1
        };

        let vt_idx = if parts.len() > 1 && !parts[1].is_empty() {
            Some(parts[1].parse().unwrap_or(1))
        } else {
            None
        };

        let vn_idx = if parts.len() > 2 && !parts[2].is_empty() {
            Some(parts[2].parse().unwrap_or(1))
        } else {
            None
        };

        // OBJ索引从1开始，转换为0开始
        (
            v_idx.saturating_sub(1),
            vt_idx.map(|i| i.saturating_sub(1)),
            vn_idx.map(|i| i.saturating_sub(1)),
        )
    }

    /// 生成指定LOD级别
    async fn generate_lod_level(
        &self,
        original: &LODModel,
        threshold: f32,
        lod_level: usize,
    ) -> Result<LODModel, OptimizationError> {
        let mut lod_meshes = Vec::new();

        for mesh in &original.meshes {
            let simplified = self.simplify_mesh(mesh, threshold).await?;
            lod_meshes.push(simplified);
        }

        Ok(LODModel {
            path: original.path.clone(),
            meshes: lod_meshes,
            lod_level,
        })
    }

    /// 简化网格（使用简化算法）
    async fn simplify_mesh(&self, mesh: &Mesh, threshold: f32) -> Result<Mesh, OptimizationError> {
        let target_triangles = (mesh.triangle_count() as f32 * threshold) as usize;

        if target_triangles >= mesh.triangle_count() {
            return Ok(mesh.clone());
        }

        // 使用简化的边折叠算法
        let simplifier = MeshSimplifier::new();
        let simplified = simplifier.simplify(mesh, target_triangles)?;

        Ok(simplified)
    }
}

/// 网格简化器
struct MeshSimplifier;

impl MeshSimplifier {
    fn new() -> Self {
        Self
    }

    /// 简化网格到目标三角形数
    fn simplify(&self, mesh: &Mesh, target_triangles: usize) -> Result<Mesh, OptimizationError> {
        if mesh.triangle_count() <= target_triangles {
            return Ok(mesh.clone());
        }

        // 简单实现：每隔n个三角形保留一个
        let current_triangles = mesh.triangle_count();
        let step = if target_triangles > 0 {
            (current_triangles / target_triangles).max(2)
        } else {
            2
        };

        let mut new_indices = Vec::new();
        for (i, chunk) in mesh.indices.chunks(3).enumerate() {
            if i % step == 0 {
                new_indices.extend_from_slice(chunk);
            }
        }

        // 确保索引是3的倍数
        while new_indices.len() % 3 != 0 {
            new_indices.pop();
        }

        Ok(Mesh {
            positions: mesh.positions.clone(),
            normals: mesh.normals.clone(),
            uvs: mesh.uvs.clone(),
            indices: new_indices,
        })
    }
}

/// LOD模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LODModel {
    pub path: PathBuf,
    pub meshes: Vec<Mesh>,
    pub lod_level: usize,
}

impl LODModel {
    /// 保存LOD模型
    pub async fn save(&self, output_path: &Path) -> Result<(), OptimizationError> {
        // 创建输出目录
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                OptimizationError::IoError(format!("Failed to create directory: {}", e))
            })?;
        }

        // 简单实现：保存为GLTF JSON格式
        let gltf_data = self.export_to_gltf_json()?;

        fs::write(output_path, gltf_data)
            .map_err(|e| OptimizationError::IoError(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// 导出为GLTF JSON格式
    fn export_to_gltf_json(&self) -> Result<String, OptimizationError> {
        // 简化的GLTF结构
        let mut meshes_json = Vec::new();

        for (mesh_idx, mesh) in self.meshes.iter().enumerate() {
            let primitives = serde_json::json!([{
                "attributes": {
                    "POSITION": mesh_idx
                },
                "indices": mesh_idx
            }]);
            meshes_json.push(primitives);
        }

        let gltf = serde_json::json!({
            "asset": {
                "version": "2.0"
            },
            "scenes": [{
                "nodes": [0]
            }],
            "nodes": [{
                "mesh": 0
            }],
            "meshes": meshes_json
        });

        serde_json::to_string_pretty(&gltf)
            .map_err(|e| OptimizationError::Other(format!("JSON serialization error: {}", e)))
    }

    /// 计算总三角形数
    pub fn triangle_count(&self) -> usize {
        self.meshes.iter().map(|m| m.triangle_count()).sum()
    }
}

/// 网格数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub uvs: Option<Vec<[f32; 2]>>,
    pub indices: Vec<u32>,
}

impl Mesh {
    /// 计算三角形数量
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// 计算顶点数量
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

/// 三角形
#[derive(Debug, Clone, Copy)]
struct Triangle {
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
}

impl Triangle {
    fn new(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> Self {
        Self { v0, v1, v2 }
    }

    /// 计算面积
    fn area(&self) -> f32 {
        let a = [
            self.v1[0] - self.v0[0],
            self.v1[1] - self.v0[1],
            self.v1[2] - self.v0[2],
        ];
        let b = [
            self.v2[0] - self.v0[0],
            self.v2[1] - self.v0[1],
            self.v2[2] - self.v0[2],
        ];

        let cross = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ];

        (cross[0].powi(2) + cross[1].powi(2) + cross[2].powi(2)).sqrt() * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_area() {
        let triangle = Triangle::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((triangle.area() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_mesh_triangle_count() {
        let mesh = Mesh {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: None,
            uvs: None,
            indices: vec![0, 1, 2],
        };

        assert_eq!(mesh.triangle_count(), 1);
        assert_eq!(mesh.vertex_count(), 3);
    }

    #[test]
    fn test_mesh_simplification() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let indices = vec![0, 1, 2, 1, 3, 2];

        let mesh = Mesh {
            positions: positions.clone(),
            normals: None,
            uvs: None,
            indices,
        };

        let simplifier = MeshSimplifier::new();
        let simplified = simplifier.simplify(&mesh, 1).unwrap();

        assert!(simplified.triangle_count() < mesh.triangle_count());
    }
}
