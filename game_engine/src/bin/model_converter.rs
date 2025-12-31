//! # 3D模型格式转换工具库

/// 模型格式枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelFormat {
    GlTF,
    GLB,
    OBJ,
    FBX,
}

impl ModelFormat {
    /// 从文件扩展名解析格式
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "gltf" => Some(ModelFormat::GlTF),
            "glb" => Some(ModelFormat::GLB),
            "obj" => Some(ModelFormat::OBJ),
            "fbx" => Some(ModelFormat::FBX),
            _ => None,
        }
    }

    /// 获取文件扩展名
    #[allow(dead_code)]
    pub fn extension(&self) -> &str {
        match self {
            ModelFormat::GlTF => "gltf",
            ModelFormat::GLB => "glb",
            ModelFormat::OBJ => "obj",
            ModelFormat::FBX => "fbx",
        }
    }
}
