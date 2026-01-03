//! 3D资源导入器模块
//!
//! 支持多种3D模型格式的导入：
//! - glTF 2.0 (.gltf, .glb)
//! - FBX (.fbx)
//! - OBJ (.obj)

pub mod error;
pub mod gltf;
pub mod fbx;
pub mod obj;

pub use error::{ImportError, ImportResult};
pub use gltf::GltfImporter;
pub use fbx::FbxImporter;
pub use obj::ObjImporter;

use std::path::Path;

/// 导入器特征
pub trait Importer {
    /// 导入3D模型
    fn import<P: AsRef<Path>>(&self, path: P) -> ImportResult<ModelData>;
}

/// 3D模型数据
#[derive(Debug, Clone)]
pub struct ModelData {
    /// 网格数据列表
    pub meshes: Vec<Mesh>,
    /// 材质数据列表
    pub materials: Vec<Material>,
    /// 节点层级
    pub nodes: Vec<Node>,
    /// 动画数据（可选）
    pub animations: Vec<Animation>,
    /// 骨骼和蒙皮数据（可选）
    pub skins: Vec<Skin>,
}

/// 网格数据
#[derive(Debug, Clone)]
pub struct Mesh {
    /// 网格名称
    pub name: Option<String>,
    /// 图元数据
    pub primitives: Vec<Primitive>,
}

/// 图元数据
#[derive(Debug, Clone)]
pub struct Primitive {
    /// 顶点数据
    pub vertices: Vec<Vertex>,
    /// 索引数据
    pub indices: Vec<u32>,
    /// 材质索引
    pub material_index: Option<usize>,
    /// 拓扑类型
    pub mode: PrimitiveMode,
}

/// 顶点数据
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    /// 位置
    pub position: [f32; 3],
    /// 法线
    pub normal: [f32; 3],
    /// UV坐标
    pub uv: [f32; 2],
    /// 切线
    pub tangent: [f32; 4],
    /// 颜色
    pub color: [f32; 4],
    /// 关节索引（用于蒙皮）
    pub joints: [u16; 4],
    /// 关节权重
    pub weights: [f32; 4],
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0; 2],
            tangent: [0.0; 4],
            color: [1.0; 4],
            joints: [0; 4],
            weights: [0.0; 4],
        }
    }
}

/// 图元拓扑类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveMode {
    Points,
    Lines,
    LineLoop,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
}

/// 材质数据
#[derive(Debug, Clone)]
pub struct Material {
    /// 材质名称
    pub name: Option<String>,
    /// PBR材质参数
    pub pbr: PbrMaterial,
    /// 法线贴图
    pub normal_texture: Option<TextureInfo>,
    /// 发光贴图
    pub emissive_texture: Option<TextureInfo>,
    /// 发光颜色
    pub emissive_factor: [f32; 3],
    /// Alpha模式
    pub alpha_mode: AlphaMode,
    /// Alpha截断值
    pub alpha_cutoff: f32,
    /// 双面渲染
    pub double_sided: bool,
}

/// PBR材质参数
#[derive(Debug, Clone)]
pub struct PbrMaterial {
    /// 基础颜色因子
    pub base_color_factor: [f32; 4],
    /// 基础颜色贴图
    pub base_color_texture: Option<TextureInfo>,
    /// 金属度因子
    pub metallic_factor: f32,
    /// 粗糙度因子
    pub roughness_factor: f32,
    /// 金属度/粗糙度贴图
    pub metallic_roughness_texture: Option<TextureInfo>,
    /// 物理光照
    pub unlit: bool,
}

/// 贴图信息
#[derive(Debug, Clone)]
pub struct TextureInfo {
    /// 贴图索引
    pub index: usize,
    ///纹理坐标集索引
    pub tex_coord: u32,
    /// 缩放
    pub scale: f32,
}

/// Alpha模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaMode {
    Opaque,
    Mask,
    Blend,
}

/// 节点数据
#[derive(Debug, Clone)]
pub struct Node {
    /// 节点名称
    pub name: Option<String>,
    /// 局部变换矩阵
    pub transform: glam::Mat4,
    /// 子节点索引
    pub children: Vec<usize>,
    /// 网格索引
    pub mesh: Option<usize>,
    /// 皮肤索引
    pub skin: Option<usize>,
    /// 摄像机索引
    pub camera: Option<usize>,
}

/// 动画数据
#[derive(Debug, Clone)]
pub struct Animation {
    /// 动画名称
    pub name: Option<String>,
    /// 动画通道
    pub channels: Vec<AnimationChannel>,
    /// 持续时间（秒）
    pub duration: f32,
}

/// 动画通道
#[derive(Debug, Clone)]
pub struct AnimationChannel {
    /// 目标节点索引
    pub target_node: usize,
    /// 插值模式
    pub interpolation: InterpolationMode,
    /// 输入（时间关键帧）
    pub input: Vec<f32>,
    /// 输出（值关键帧）
    pub output: AnimationOutput,
    /// 目标属性
    pub target: AnimationTarget,
}

/// 插值模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationMode {
    Linear,
    Step,
    CubicSpline,
}

/// 动画输出数据
#[derive(Debug, Clone)]
pub enum AnimationOutput {
    Translation(Vec<glam::Vec3>),
    Rotation(Vec<glam::Quat>),
    Scale(Vec<glam::Vec3>),
    Weights(Vec<f32>),
}

/// 动画目标属性
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationTarget {
    Translation,
    Rotation,
    Scale,
    Weights,
}

/// 皮肤数据（蒙皮）
#[derive(Debug, Clone)]
pub struct Skin {
    /// 皮肤名称
    pub name: Option<String>,
    /// 骨骼节点索引
    pub joints: Vec<usize>,
    /// 骨骼逆绑定矩阵
    pub inverse_bind_matrices: Vec<glam::Mat4>,
    /// 根骨骼节点索引
    pub skeleton: Option<usize>,
}

/// 根据文件扩展名选择合适的导入器并导入模型
pub fn import_model<P: AsRef<Path>>(path: P) -> ImportResult<ModelData> {
    let path = path.as_ref();
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| ImportError::UnsupportedFormat("No file extension".to_string()))?;

    match extension.to_lowercase().as_str() {
        "gltf" | "glb" => {
            let importer = GltfImporter::new();
            importer.import(path)
        }
        "fbx" => {
            let importer = FbxImporter::new();
            importer.import(path)
        }
        "obj" => {
            let importer = ObjImporter::new();
            importer.import(path)
        }
        _ => Err(ImportError::UnsupportedFormat(extension.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_model_gltf() {
        // 测试不会因为不支持格式而失败
        let result = import_model("test.gltf");
        // 应该返回IO错误或解析错误，但不是格式不支持错误
        match result {
            Err(ImportError::UnsupportedFormat(_)) => panic!("Should support gltf format"),
            _ => {}
        }
    }

    #[test]
    fn test_import_model_fbx() {
        let result = import_model("test.fbx");
        match result {
            Err(ImportError::UnsupportedFormat(_)) => panic!("Should support fbx format"),
            _ => {}
        }
    }

    #[test]
    fn test_import_model_obj() {
        let result = import_model("test.obj");
        match result {
            Err(ImportError::UnsupportedFormat(_)) => panic!("Should support obj format"),
            _ => {}
        }
    }

    #[test]
    fn test_import_model_unsupported() {
        let result = import_model("test.unknown");
        assert!(matches!(result, Err(ImportError::UnsupportedFormat(_))));
    }
}
