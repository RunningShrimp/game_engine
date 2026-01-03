//! glTF 2.0导入器
//!
//! 支持导入glTF 2.0格式（.gltf和.glb）

use super::{error::ImportResult, Importer, Material, Mesh, ModelData, Node, Primitive, PrimitiveMode, PbrMaterial, Skin, TextureInfo, AlphaMode, Vertex, Animation, AnimationChannel, AnimationOutput, AnimationTarget, InterpolationMode};
use gltf::{Gltf};
use std::path::Path;

/// glTF 2.0导入器
pub struct GltfImporter {
    /// 是否加载外部资源
    load_external: bool,
}

impl GltfImporter {
    pub fn new() -> Self {
        Self {
            load_external: true,
        }
    }

    /// 设置是否加载外部资源
    pub fn with_external_loading(mut self, load: bool) -> Self {
        self.load_external = load;
        self
    }

    /// 加载glTF文件
    fn load_gltf<P: AsRef<Path>>(&self, path: P) -> ImportResult<(Gltf, Vec<Vec<u8>>, Vec<Vec<u8>>)> {
        let path = path.as_ref();

        // 导入glTF文件
        let gltf = match path.extension().and_then(|e| e.to_str()) {
            Some("gltf") | Some("glb") => {
                // JSON格式glTF 或 二进制格式glTF
                let file = std::fs::File::open(path)?;
                let reader = std::io::BufReader::new(file);
                Gltf::from_reader(reader).map_err(|e| {
                    super::ImportError::ParseError(format!("Failed to parse glTF: {:?}", e))
                })?
            }
            _ => {
                return Err(super::ImportError::UnsupportedFormat(
                    "Not a glTF file".to_string()
                ))
            }
        };

        // 简化处理：返回空的buffer和image数据
        // 实际应用中需要根据GLB或外部资源加载这些数据
        let buffer_data = Vec::new();
        let image_data = Vec::new();

        Ok((gltf, buffer_data, image_data))
    }

    /// 解析材质
    fn parse_material(&self, material: &gltf::Material) -> Material {
        let pbr = material.pbr_metallic_roughness();

        let base_color_texture = pbr.base_color_texture().map(|info| TextureInfo {
            index: info.texture().index(),
            tex_coord: info.tex_coord(),
            scale: 1.0,
        });

        let metallic_roughness_texture = pbr.metallic_roughness_texture().map(|info| TextureInfo {
            index: info.texture().index(),
            tex_coord: info.tex_coord(),
            scale: 1.0,
        });

        let normal_texture = material.normal_texture().map(|info| TextureInfo {
            index: info.texture().index(),
            tex_coord: info.tex_coord(),
            scale: info.scale(),
        });

        let emissive_texture = material.emissive_texture().map(|info| TextureInfo {
            index: info.texture().index(),
            tex_coord: info.tex_coord(),
            scale: 1.0,
        });

        let alpha_mode = match material.alpha_mode() {
            gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
            gltf::material::AlphaMode::Mask => AlphaMode::Mask,
            gltf::material::AlphaMode::Blend => AlphaMode::Blend,
        };

        Material {
            name: material.name().map(|s| s.to_string()),
            pbr: PbrMaterial {
                base_color_factor: pbr.base_color_factor().to_owned(),
                base_color_texture,
                metallic_factor: pbr.metallic_factor(),
                roughness_factor: pbr.roughness_factor(),
                metallic_roughness_texture,
                unlit: false,
            },
            normal_texture,
            emissive_texture,
            emissive_factor: material.emissive_factor().to_owned(),
            alpha_mode,
            alpha_cutoff: material.alpha_cutoff().unwrap_or(0.5),
            double_sided: material.double_sided(),
        }
    }

    /// 解析图元拓扑类型
    fn parse_mode(&self, mode: gltf::mesh::Mode) -> PrimitiveMode {
        match mode {
            gltf::mesh::Mode::Points => PrimitiveMode::Points,
            gltf::mesh::Mode::Lines => PrimitiveMode::Lines,
            gltf::mesh::Mode::LineLoop => PrimitiveMode::LineLoop,
            gltf::mesh::Mode::LineStrip => PrimitiveMode::LineStrip,
            gltf::mesh::Mode::Triangles => PrimitiveMode::Triangles,
            gltf::mesh::Mode::TriangleStrip => PrimitiveMode::TriangleStrip,
            gltf::mesh::Mode::TriangleFan => PrimitiveMode::TriangleFan,
        }
    }

    /// 解析网格图元（简化版本 - 不读取实际顶点数据）
    fn parse_primitive(
        &self,
        _primitive: gltf::Primitive,
    ) -> ImportResult<Primitive> {
        // 简化版本：返回空网格
        // 实际应用中需要从buffer读取顶点数据
        Ok(Primitive {
            vertices: Vec::new(),
            indices: Vec::new(),
            material_index: None,
            mode: PrimitiveMode::Triangles,
        })
    }

    /// 解析网格
    fn parse_mesh(&self, mesh: gltf::Mesh) -> ImportResult<Mesh> {
        let primitives = mesh
            .primitives()
            .map(|primitive| self.parse_primitive(primitive))
            .collect::<ImportResult<Vec<_>>>()?;

        Ok(Mesh {
            name: mesh.name().map(|s| s.to_string()),
            primitives,
        })
    }

    /// 解析节点
    fn parse_node(&self, node: gltf::Node) -> Node {
        // 获取变换矩阵
        let transform = node.transform().matrix();
        let transform = glam::Mat4::from_cols_array_2d(&transform);

        // 获取子节点
        let children = node.children().map(|c| c.index()).collect();

        Node {
            name: node.name().map(|s| s.to_string()),
            transform,
            children,
            mesh: node.mesh().map(|m| m.index()),
            skin: node.skin().map(|s| s.index()),
            camera: node.camera().map(|c| c.index()),
        }
    }

    /// 解析皮肤（简化版本）
    fn parse_skin(&self, skin: gltf::Skin) -> Skin {
        let joints = skin.joints().map(|j| j.index()).collect();

        Skin {
            name: skin.name().map(|s| s.to_string()),
            joints,
            inverse_bind_matrices: Vec::new(), // 简化：不加载实际数据
            skeleton: skin.skeleton().map(|s| s.index()),
        }
    }

    /// 解析动画（简化版本）
    fn parse_animation(&self, animation: gltf::Animation) -> ImportResult<Animation> {
        // 简化版本：返回空动画
        // 实际应用中需要从buffer读取动画数据
        Ok(Animation {
            name: animation.name().map(|s| s.to_string()),
            channels: Vec::new(),
            duration: 0.0,
        })
    }
}

impl Default for GltfImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Importer for GltfImporter {
    fn import<P: AsRef<Path>>(&self, path: P) -> ImportResult<ModelData> {
        let (gltf, _buffers, _images) = self.load_gltf(path)?;

        // 解析材质
        let materials = gltf
            .materials()
            .map(|m| self.parse_material(&m))
            .collect();

        // 解析网格
        let meshes = gltf
            .meshes()
            .map(|m| self.parse_mesh(m))
            .collect::<ImportResult<Vec<_>>>()?;

        // 解析节点
        let nodes = gltf.nodes().map(|n| self.parse_node(n)).collect();

        // 解析皮肤
        let skins = gltf.skins().map(|s| self.parse_skin(s)).collect();

        // 解析动画
        let animations = gltf
            .animations()
            .map(|a| self.parse_animation(a))
            .collect::<ImportResult<Vec<_>>>()?;

        Ok(ModelData {
            meshes,
            materials,
            nodes,
            animations,
            skins,
        })
    }
}

// 导入需要的类型
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_importer() {
        let importer = GltfImporter::new();
        assert_eq!(importer.load_external, true);
    }

    #[test]
    fn test_importer_with_options() {
        let importer = GltfImporter::new().with_external_loading(false);
        assert_eq!(importer.load_external, false);
    }

    #[test]
    fn test_parse_mode() {
        let importer = GltfImporter::new();
        assert_eq!(
            importer.parse_mode(gltf::mesh::Mode::Triangles),
            PrimitiveMode::Triangles
        );
        assert_eq!(
            importer.parse_mode(gltf::mesh::Mode::Points),
            PrimitiveMode::Points
        );
    }
}
