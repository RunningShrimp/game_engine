//! OBJ导入器
//!
//! 支持导入Wavefront OBJ格式的3D模型
//!
//! OBJ格式特点：
//! - 简单的文本格式
//! - 支持网格、材质、纹理坐标、法线等
//! - 适合作为基本的3D模型交换格式

use super::{error::ImportResult, Importer, Material, Mesh, ModelData, Node, Primitive, PrimitiveMode, PbrMaterial, TextureInfo, AlphaMode, Vertex};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufRead, BufReader};

/// OBJ导入器
pub struct ObjImporter {
    /// 是否加载材质库（MTL）
    load_materials: bool,
    /// 是否翻转UV坐标的Y轴
    flip_uv: bool,
    /// 是否翻转法线的方向
    flip_normals: bool,
}

impl ObjImporter {
    pub fn new() -> Self {
        Self {
            load_materials: true,
            flip_uv: true, // 大多数OBJ文件的UV需要翻转
            flip_normals: false,
        }
    }

    /// 设置是否加载材质
    pub fn with_materials(mut self, load: bool) -> Self {
        self.load_materials = load;
        self
    }

    /// 设置是否翻转UV
    pub fn with_uv_flip(mut self, flip: bool) -> Self {
        self.flip_uv = flip;
        self
    }

    /// 设置是否翻转法线
    pub fn with_normals_flip(mut self, flip: bool) -> Self {
        self.flip_normals = flip;
        self
    }

    /// 解析OBJ文件
    fn parse_obj<P: AsRef<Path>>(&self, path: P) -> ImportResult<ObjData> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut objects: Vec<ObjObject> = Vec::new();
        let mut current_object = ObjObject::default();
        let mut current_group_name = String::from("default");
        let mut current_material_name: Option<String> = None;
        let mut mtl_libs: Vec<String> = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            // 跳过空行和注释
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut parts = line.split_whitespace();
            let keyword = parts.next().unwrap_or("");

            match keyword {
                "v" => {
                    // 顶点位置
                    let x: f32 = parts.next().unwrap_or("0").parse().unwrap_or(0.0);
                    let y: f32 = parts.next().unwrap_or("0").parse().unwrap_or(0.0);
                    let z: f32 = parts.next().unwrap_or("0").parse().unwrap_or(0.0);
                    positions.push([x, y, z]);
                }
                "vn" => {
                    // 顶点法线
                    let x: f32 = parts.next().unwrap_or("0").parse().unwrap_or(0.0);
                    let y: f32 = parts.next().unwrap_or("0").parse().unwrap_or(0.0);
                    let z: f32 = parts.next().unwrap_or("0").parse().unwrap_or(0.0);
                    normals.push([x, y, z]);
                }
                "vt" => {
                    // 纹理坐标
                    let u: f32 = parts.next().unwrap_or("0").parse().unwrap_or(0.0);
                    let v: f32 = parts.next().unwrap_or("0").parse().unwrap_or(0.0);
                    uvs.push([u, v]);
                }
                "f" => {
                    // 面（三角形/多边形）
                    let indices: Vec<_> = parts
                        .map(|s| self.parse_vertex_index(s))
                        .collect();

                    // 三角化面（简单处理：假设是凸多边形）
                    for i in 1..indices.len() - 1 {
                        current_object.add_face([
                            indices[0].clone(),
                            indices[i].clone(),
                            indices[i + 1].clone(),
                        ]);
                    }
                }
                "o" => {
                    // 新对象
                    if !current_object.is_empty() {
                        objects.push(current_object.clone());
                    }
                    let name = parts.next().unwrap_or("object").to_string();
                    current_object = ObjObject::new(&name);
                    current_group_name = name;
                }
                "g" => {
                    // 新组
                    if !current_object.is_empty() {
                        objects.push(current_object.clone());
                    }
                    let name = parts.next().unwrap_or("group").to_string();
                    current_object = ObjObject::new(&name);
                    current_group_name = name;
                }
                "usemtl" => {
                    // 使用材质
                    current_material_name = parts.next().map(|s| s.to_string());
                }
                "mtllib" => {
                    // 材质库
                    if let Some(lib) = parts.next() {
                        mtl_libs.push(lib.to_string());
                    }
                }
                "s" => {
                    // 平滑组（忽略）
                }
                _ => {
                    // 忽略其他关键字
                }
            }
        }

        // 添加最后一个对象
        if !current_object.is_empty() {
            objects.push(current_object);
        }

        Ok(ObjData {
            positions,
            normals,
            uvs,
            objects,
            mtl_libs,
        })
    }

    /// 解析顶点索引（支持 v/vt/vn 格式）
    fn parse_vertex_index(&self, s: &str) -> VertexIndex {
        let parts: Vec<&str> = s.split('/').collect();

        let v_idx = parts.get(0)
            .and_then(|s| s.parse::<usize>().ok())
            .map(|i| if i == 0 { i } else { i - 1 }); // OBJ索引从1开始

        let vt_idx = parts.get(1)
            .and_then(|s| if s.is_empty() { None } else { s.parse::<usize>().ok() })
            .map(|i| if i == 0 { i } else { i - 1 });

        let vn_idx = if parts.len() > 2 {
            parts.get(2)
                .and_then(|s| if s.is_empty() { None } else { s.parse::<usize>().ok() })
                .map(|i| if i == 0 { i } else { i - 1 })
        } else {
            parts.get(1)
                .and_then(|s| if s.is_empty() { None } else { s.parse::<usize>().ok() })
                .map(|i| if i == 0 { i } else { i - 1 })
        };

        VertexIndex { v: v_idx.unwrap_or(0), vt: vt_idx, vn: vn_idx }
    }

    /// 构建网格数据
    fn build_mesh(&self, obj_data: &ObjData) -> ImportResult<Mesh> {
        let mut primitives = Vec::new();

        for object in &obj_data.objects {
            if object.faces.is_empty() {
                continue;
            }

            // 构建顶点数据
            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            let mut vertex_map = std::collections::HashMap::new();

            for face in &object.faces {
                for vi in face {
                    // 使用顶点索引的组合来创建唯一顶点
                    let key = *vi;

                    if let Some(&idx) = vertex_map.get(&key) {
                        indices.push(idx as u32);
                    } else {
                        let idx = vertices.len();

                        let position = obj_data.positions.get(key.v)
                            .copied()
                            .unwrap_or([0.0, 0.0, 0.0]);

                        let mut uv = key.vt
                            .and_then(|i| obj_data.uvs.get(i).copied())
                            .unwrap_or([0.0, 0.0]);

                        if self.flip_uv {
                            uv[1] = 1.0 - uv[1];
                        }

                        let mut normal = key.vn
                            .and_then(|i| obj_data.normals.get(i).copied())
                            .unwrap_or([0.0, 0.0, 1.0]);

                        if self.flip_normals {
                            normal[0] = -normal[0];
                            normal[1] = -normal[1];
                            normal[2] = -normal[2];
                        }

                        vertices.push(Vertex {
                            position,
                            normal,
                            uv,
                            tangent: [1.0, 0.0, 0.0, 1.0],
                            color: [1.0, 1.0, 1.0, 1.0],
                            joints: [0, 0, 0, 0],
                            weights: [0.0, 0.0, 0.0, 0.0],
                        });

                        vertex_map.insert(key, idx as u32);
                        indices.push(idx as u32);
                    }
                }
            }

            primitives.push(Primitive {
                vertices,
                indices,
                material_index: None, // 材质索引将在解析MTL后设置
                mode: PrimitiveMode::Triangles,
            });
        }

        Ok(Mesh {
            name: Some("OBJ Mesh".to_string()),
            primitives,
        })
    }

    /// 解析MTL材质文件
    fn parse_mtl<P: AsRef<Path>>(&self, obj_path: P, mtl_lib: &str) -> ImportResult<Vec<Material>> {
        if !self.load_materials {
            return Ok(vec![]);
        }

        let obj_path = obj_path.as_ref();
        let mtl_path = obj_path.parent()
            .map(|p| p.join(mtl_lib))
            .unwrap_or_else(|| PathBuf::from(mtl_lib));

        let file = File::open(&mtl_path)?;
        let reader = BufReader::new(file);

        let mut materials = Vec::new();
        let mut current_material: Option<Material> = None;

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut parts = line.split_whitespace();
            let keyword = parts.next().unwrap_or("");

            match keyword {
                "newmtl" => {
                    // 新材质
                    if let Some(mat) = current_material.take() {
                        materials.push(mat);
                    }
                    let name = parts.next().unwrap_or("material").to_string();
                    current_material = Some(Material {
                        name: Some(name),
                        pbr: PbrMaterial {
                            base_color_factor: [1.0, 1.0, 1.0, 1.0],
                            base_color_texture: None,
                            metallic_factor: 0.0,
                            roughness_factor: 1.0,
                            metallic_roughness_texture: None,
                            unlit: false,
                        },
                        normal_texture: None,
                        emissive_texture: None,
                        emissive_factor: [0.0, 0.0, 0.0],
                        alpha_mode: AlphaMode::Opaque,
                        alpha_cutoff: 0.5,
                        double_sided: false,
                    });
                }
                "Ka" => {
                    // 环境光颜色（暂时忽略）
                }
                "Kd" => {
                    // 漫反射颜色
                    if let Some(ref mut mat) = current_material {
                        let r: f32 = parts.next().unwrap_or("1").parse().unwrap_or(1.0);
                        let g: f32 = parts.next().unwrap_or("1").parse().unwrap_or(1.0);
                        let b: f32 = parts.next().unwrap_or("1").parse().unwrap_or(1.0);
                        mat.pbr.base_color_factor = [r, g, b, 1.0];
                    }
                }
                "Ks" => {
                    // 镜面反射颜色（暂时忽略）
                }
                "Ns" => {
                    // 高光指数（暂时忽略）
                }
                "d" | "Tr" => {
                    // 透明度
                    if let Some(ref mut mat) = current_material {
                        let alpha: f32 = parts.next().unwrap_or("1").parse().unwrap_or(1.0);
                        mat.pbr.base_color_factor[3] = alpha;
                    }
                }
                "map_Kd" => {
                    // 漫反射贴图
                    if let Some(ref mut mat) = current_material {
                        if let Some(tex) = parts.next() {
                            mat.pbr.base_color_texture = Some(TextureInfo {
                                index: 0, // 简化处理
                                tex_coord: 0,
                                scale: 1.0,
                            });
                        }
                    }
                }
                "map_Bump" | "bump" => {
                    // 法线贴图
                    if let Some(ref mut mat) = current_material {
                        if let Some(_) = parts.next() {
                            mat.normal_texture = Some(TextureInfo {
                                index: 0,
                                tex_coord: 0,
                                scale: 1.0,
                            });
                        }
                    }
                }
                _ => {
                    // 忽略其他关键字
                }
            }
        }

        // 添加最后一个材质
        if let Some(mat) = current_material {
            materials.push(mat);
        }

        Ok(materials)
    }
}

impl Default for ObjImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Importer for ObjImporter {
    fn import<P: AsRef<Path>>(&self, path: P) -> ImportResult<ModelData> {
        let obj_data = self.parse_obj(&path)?;

        // 构建网格
        let meshes = vec![self.build_mesh(&obj_data)?];

        // 解析材质
        let mut materials = Vec::new();
        if self.load_materials {
            for mtl_lib in &obj_data.mtl_libs {
                let mtl_materials = self.parse_mtl(&path, mtl_lib)?;
                materials.extend(mtl_materials);
            }
        }

        // OBJ没有节点层级，创建一个默认节点
        let nodes = vec![Node {
            name: Some("OBJ Root".to_string()),
            transform: glam::Mat4::IDENTITY,
            children: vec![],
            mesh: Some(0),
            skin: None,
            camera: None,
        }];

        Ok(ModelData {
            meshes,
            materials,
            nodes,
            animations: vec![],
            skins: vec![],
        })
    }
}

/// OBJ文件数据结构
#[derive(Debug, Clone)]
struct ObjData {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    objects: Vec<ObjObject>,
    mtl_libs: Vec<String>,
}

/// OBJ对象
#[derive(Debug, Clone, Default)]
struct ObjObject {
    name: String,
    faces: Vec<[VertexIndex; 3]>,
}

impl ObjObject {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            faces: Vec::new(),
        }
    }

    fn add_face(&mut self, indices: [VertexIndex; 3]) {
        self.faces.push(indices);
    }

    fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }
}

/// 顶点索引（支持 v/vt/vn 格式）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct VertexIndex {
    v: usize,
    vt: Option<usize>,
    vn: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_importer() {
        let importer = ObjImporter::new();
        assert_eq!(importer.load_materials, true);
        assert_eq!(importer.flip_uv, true);
        assert_eq!(importer.flip_normals, false);
    }

    #[test]
    fn test_importer_with_options() {
        let importer = ObjImporter::new()
            .with_materials(false)
            .with_uv_flip(false)
            .with_normals_flip(true);

        assert_eq!(importer.load_materials, false);
        assert_eq!(importer.flip_uv, false);
        assert_eq!(importer.flip_normals, true);
    }

    #[test]
    fn test_parse_vertex_index() {
        let importer = ObjImporter::new();

        // 测试 "v" 格式
        let vi = importer.parse_vertex_index("1");
        assert_eq!(vi.v, 0);
        assert_eq!(vi.vt, None);
        assert_eq!(vi.vn, None);

        // 测试 "v/vt" 格式
        let vi = importer.parse_vertex_index("2/3");
        assert_eq!(vi.v, 1);
        assert_eq!(vi.vt, Some(2));
        assert_eq!(vi.vn, None);

        // 测试 "v/vt/vn" 格式
        let vi = importer.parse_vertex_index("4/5/6");
        assert_eq!(vi.v, 3);
        assert_eq!(vi.vt, Some(4));
        assert_eq!(vi.vn, Some(5));
    }
}
