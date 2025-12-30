//! GLTF 模型加载器
//!
//! 提供异步 GLTF/GLB 模型加载功能，支持纹理、网格和场景数据解析。
//!
//! 这个模块使用条件编译来提供完整实现或存根实现，具体取决于 `gltf` 特性是否启用。

#[cfg(feature = "gltf")]
#[path = "gltf_loader_impl.rs"]
mod gltf_loader_impl;

// 统一导出接口
#[cfg(feature = "gltf")]
pub use gltf_loader_impl::{GltfLoadError, GltfLoader, GltfScene};

// 存根实现（当 gltf feature 未启用时）
#[cfg(not(feature = "gltf"))]
use std::path::Path;

#[cfg(not(feature = "gltf"))]
/// GLTF 场景数据（存根）
#[derive(Clone, Debug)]
pub struct GltfScene;

#[cfg(not(feature = "gltf"))]
impl GltfScene {
    pub fn from_bytes(_bytes: Vec<u8>, _json: Option<serde_json::Value>) -> Self {
        // GLTF support is not enabled. Add 'gltf' feature to Cargo.toml
        Self
    }
}

#[cfg(not(feature = "gltf"))]
/// GLTF 加载错误类型（存根）
#[derive(Debug, thiserror::Error)]
pub enum GltfLoadError {
    #[error("GLTF support not enabled. Enable the 'gltf' feature to use this function.")]
    FeatureNotEnabled,
}

#[cfg(not(feature = "gltf"))]
/// 异步 GLTF 加载器（存根）
pub struct GltfLoader;

#[cfg(not(feature = "gltf"))]
impl GltfLoader {
    pub async fn load_from_path(_path: &Path) -> Result<GltfScene, String> {
        Err("GLTF support not enabled. Enable the 'gltf' feature to use this function.".to_string())
    }

    pub fn validate_extension(_path: &Path) -> Result<(), GltfLoadError> {
        Err(GltfLoadError::FeatureNotEnabled)
    }
}

