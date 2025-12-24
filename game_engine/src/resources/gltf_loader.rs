//! GLTF 模型加载器
//!
//! 提供异步 GLTF/GLB 模型加载功能，支持纹理、网格和场景数据解析。

#[cfg(feature = "gltf")]
use std::sync::Arc;
#[cfg(feature = "gltf")]
use std::path::Path;

#[cfg(feature = "gltf")]
use serde_json::Value;

/// GLTF 场景数据
///
/// 包含解析后的 GLTF 文档、缓冲区数据和图像数据。
#[cfg(feature = "gltf")]
#[derive(Clone, Debug)]
pub struct GltfScene {
    /// GLTF 文档和缓冲区数据
    pub data: Arc<(
        gltf::Document,
        Vec<gltf::buffer::Data>,
        Vec<gltf::image::Data>,
    )>,
    /// 可选的原始 JSON 数据（用于调试或元数据访问）
    pub json: Option<Value>,
}

#[cfg(feature = "gltf")]
impl GltfScene {
    /// 从文件字节加载 GLTF 场景
    ///
    /// # 参数
    /// - `bytes`: GLTF/GLB 文件的原始字节数据
    /// - `json`: 可选的解析后的 JSON 数据
    ///
    /// # 返回
    /// 包含解析后文档、缓冲区和图像的 `GltfScene`
    pub fn from_bytes(bytes: Vec<u8>, json: Option<Value>) -> Self {
        let data = gltf::import_slice(&bytes).expect("Failed to import GLTF data");
        Self {
            data: Arc::new(data),
            json,
        }
    }

    /// 获取 GLTF 文档
    pub fn document(&self) -> &gltf::Document {
        &self.data.0
    }

    /// 获取缓冲区数据
    pub fn buffers(&self) -> &[gltf::buffer::Data] {
        &self.data.1
    }

    /// 获取图像数据
    pub fn images(&self) -> &[gltf::image::Data] {
        &self.data.2
    }

    /// 获取场景数量
    pub fn scene_count(&self) -> usize {
        self.document().scenes().len()
    }

    /// 获取网格数量
    pub fn mesh_count(&self) -> usize {
        self.document().meshes().len()
    }

    /// 获取纹理数量
    pub fn texture_count(&self) -> usize {
        self.document().textures().len()
    }
}

#[cfg(feature = "gltf")]
/// GLTF 加载错误类型
#[derive(Debug, thiserror::Error)]
pub enum GltfLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("GLTF parse error: {0}")]
    Parse(String),

    #[error("Missing buffer data for index {0}")]
    MissingBuffer(usize),

    #[error("Invalid file extension: {0}")]
    InvalidExtension(String),
}

#[cfg(feature = "gltf")]
impl GltfLoadError {
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }
}

#[cfg(feature = "gltf")]
/// 异步 GLTF 加载器
pub struct GltfLoader;

#[cfg(feature = "gltf")]
impl GltfLoader {
    /// 从路径异步加载 GLTF 文件
    ///
    /// # 参数
    /// - `path`: GLTF/GLB 文件路径
    ///
    /// # 返回
    /// 加载的 `GltfScene` 或错误信息
    pub async fn load_from_path(path: &Path) -> Result<GltfScene, String> {
        let bytes = tokio::fs::read(path).await
            .map_err(|e| format!("Failed to read GLTF file: {}", e))?;

        let json = String::from_utf8(bytes.clone())
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok());

        let data = gltf::import_slice(&bytes)
            .map_err(|e| format!("Failed to import GLTF: {}", e))?;

        Ok(GltfScene {
            data: Arc::new(data),
            json,
        })
    }

    /// 验证文件扩展名
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 如果扩展名有效则返回 `Ok(())`
    pub fn validate_extension(path: &Path) -> Result<(), GltfLoadError> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("gltf") | Some("glb") => Ok(()),
            Some(ext) => Err(GltfLoadError::InvalidExtension(ext.to_string())),
            None => Err(GltfLoadError::InvalidExtension("none".to_string())),
        }
    }
}

#[cfg(all(test, feature = "gltf"))]
mod tests {
    use super::*;

    #[test]
    fn test_validate_extension() {
        assert!(GltfLoader::validate_extension(Path::new("model.gltf")).is_ok());
        assert!(GltfLoader::validate_extension(Path::new("model.glb")).is_ok());
        assert!(GltfLoader::validate_extension(Path::new("model.obj")).is_err());
        assert!(GltfLoader::validate_extension(Path::new("model")).is_err());
    }

    #[test]
    fn test_gltf_scene_info() {
        let gltf_data = br#"{
            "scenes": [
                { "nodes": [0] }
            ],
            "nodes": [
                {
                    "name": "TestNode",
                    "translation": [0, 0, 0]
                }
            ]
        }"#;

        let scene = GltfScene::from_bytes(
            gltf_data.to_vec(),
            serde_json::from_str(std::str::from_utf8(gltf_data).unwrap()).ok(),
        );

        assert_eq!(scene.scene_count(), 1);
        assert_eq!(scene.mesh_count(), 0);
        assert!(scene.json.is_some());
    }
}
