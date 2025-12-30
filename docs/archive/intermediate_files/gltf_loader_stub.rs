//! GLTF 加载器存根实现（当 gltf feature 未启用时）
//!
//! 这个模块提供存根实现，用于在 GLTF 特性未启用时提供编译时错误提示。

use std::path::Path;

/// GLTF 场景数据（存根）
#[derive(Clone, Debug)]
pub struct GltfScene;

impl GltfScene {
    pub fn from_bytes(_bytes: Vec<u8>, _json: Option<serde_json::Value>) -> Self {
        // GLTF support is not enabled. Add 'gltf' feature to Cargo.toml
        Self
    }
}

/// GLTF 加载错误类型（存根）
#[derive(Debug, thiserror::Error)]
pub enum GltfLoadError {
    #[error("GLTF support not enabled. Enable the 'gltf' feature to use this function.")]
    FeatureNotEnabled,
}

/// 异步 GLTF 加载器（存根）
pub struct GltfLoader;

impl GltfLoader {
    pub async fn load_from_path(_path: &Path) -> Result<GltfScene, String> {
        Err("GLTF support not enabled. Enable the 'gltf' feature to use this function.".to_string())
    }

    pub fn validate_extension(_path: &Path) -> Result<(), GltfLoadError> {
        Err(GltfLoadError::FeatureNotEnabled)
    }
}
