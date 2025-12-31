//! 统一资源加载器Trait
//!
//! 提供运行时动态加载器注册系统，减少条件编译使用

use async_trait::async_trait;
use std::{any::Any, path::Path};

/// 资源加载错误
#[derive(Debug, thiserror::Error)]
pub enum AssetLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Decode error: {0}")]
    Decode(String),

    #[error("Unsupported format")]
    UnsupportedFormat,

    #[error("Loader not found for extension: {0}")]
    LoaderNotFound(String),

    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },
}

/// 资源加载结果 - 类型擦除版本
pub enum BoxedAssetResult {
    Image(image::RgbaImage, bool), // (image, is_linear)
    Bytes(Vec<u8>),
    Custom(Box<dyn Any + Send + Sync>),
}

/// 资源加载器trait - 支持异步加载
#[async_trait]
pub trait AssetLoader: Send + Sync + 'static {
    /// 获取支持的文件扩展名（小写，不含点）
    fn extensions(&self) -> &[&str];

    /// 加载资源（异步）
    async fn load(&self, path: &Path, bytes: Vec<u8>) -> Result<BoxedAssetResult, AssetLoadError>;

    /// 获取加载器名称（用于调试）
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// 克隆为Box trait对象
    fn clone_box(&self) -> Box<dyn AssetLoader>;
}

/// 为了支持clone，需要实现Clone trait
impl Clone for Box<dyn AssetLoader> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// =============================================================================
// 具体加载器实现
// =============================================================================

/// 纹理加载器
#[derive(Clone)]
pub struct TextureAssetLoader;

#[async_trait]
impl AssetLoader for TextureAssetLoader {
    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "bmp", "tga", "gif", "webp"]
    }

    async fn load(&self, _path: &Path, bytes: Vec<u8>) -> Result<BoxedAssetResult, AssetLoadError> {
        // 在阻塞任务中解码图像，避免阻塞异步运行时
        let image = tokio::task::spawn_blocking(move || {
            image::load_from_memory(&bytes)
                .map_err(|e| AssetLoadError::Decode(e.to_string()))
                .map(|img| img.to_rgba8())
        })
        .await
        .map_err(|e| AssetLoadError::Decode(e.to_string()))??;

        Ok(BoxedAssetResult::Image(image, false))
    }

    fn clone_box(&self) -> Box<dyn AssetLoader> {
        Box::new(self.clone())
    }
}

/// 图集加载器
#[derive(Clone)]
pub struct AtlasAssetLoader;

#[async_trait]
impl AssetLoader for AtlasAssetLoader {
    fn extensions(&self) -> &[&str] {
        &["atlas", "json"]
    }

    async fn load(&self, _path: &Path, bytes: Vec<u8>) -> Result<BoxedAssetResult, AssetLoadError> {
        Ok(BoxedAssetResult::Bytes(bytes))
    }

    fn clone_box(&self) -> Box<dyn AssetLoader> {
        Box::new(self.clone())
    }
}

/// GLTF加载器（可选feature）
#[cfg(feature = "gltf")]
pub struct GltfAssetLoaderWrapper {
    pub inner: super::gltf_assets::GltfAssetLoader,
}

#[cfg(feature = "gltf")]
impl Clone for GltfAssetLoaderWrapper {
    fn clone(&self) -> Self {
        // GltfAssetLoader是零成本类型，可以直接克隆
        Self {
            inner: super::gltf_assets::GltfAssetLoader,
        }
    }
}

#[cfg(feature = "gltf")]
#[async_trait]
impl AssetLoader for GltfAssetLoaderWrapper {
    fn extensions(&self) -> &[&str] {
        &["gltf", "glb"]
    }

    async fn load(&self, _path: &Path, bytes: Vec<u8>) -> Result<BoxedAssetResult, AssetLoadError> {
        let scene = super::gltf_assets::GltfAssetLoader::load_from_bytes(bytes)
            .await
            .map_err(AssetLoadError::Decode)?;

        Ok(BoxedAssetResult::Custom(Box::new(scene)))
    }

    fn clone_box(&self) -> Box<dyn AssetLoader> {
        Box::new(self.clone())
    }
}

/// FBX加载器（可选feature）
#[cfg(feature = "fbx")]
pub struct FbxAssetLoaderWrapper {
    pub inner: super::fbx_assets::FbxAssetLoader,
}

#[cfg(feature = "fbx")]
impl Clone for FbxAssetLoaderWrapper {
    fn clone(&self) -> Self {
        Self {
            inner: super::fbx_assets::FbxAssetLoader,
        }
    }
}

#[cfg(feature = "fbx")]
#[async_trait]
impl AssetLoader for FbxAssetLoaderWrapper {
    fn extensions(&self) -> &[&str] {
        &["fbx"]
    }

    async fn load(&self, _path: &Path, bytes: Vec<u8>) -> Result<BoxedAssetResult, AssetLoadError> {
        let scene = super::fbx_assets::FbxAssetLoader::load_from_bytes(bytes)
            .await
            .map_err(AssetLoadError::Decode)?;

        Ok(BoxedAssetResult::Custom(Box::new(scene)))
    }

    fn clone_box(&self) -> Box<dyn AssetLoader> {
        Box::new(self.clone())
    }
}

/// OBJ加载器（可选feature）
#[cfg(feature = "obj")]
pub struct ObjAssetLoaderWrapper {
    pub inner: super::obj_assets::ObjAssetLoader,
}

#[cfg(feature = "obj")]
impl Clone for ObjAssetLoaderWrapper {
    fn clone(&self) -> Self {
        Self {
            inner: super::obj_assets::ObjAssetLoader,
        }
    }
}

#[cfg(feature = "obj")]
#[async_trait]
impl AssetLoader for ObjAssetLoaderWrapper {
    fn extensions(&self) -> &[&str] {
        &["obj"]
    }

    async fn load(&self, path: &Path, bytes: Vec<u8>) -> Result<BoxedAssetResult, AssetLoadError> {
        let content = String::from_utf8(bytes)
            .map_err(|e| AssetLoadError::Decode(format!("Invalid UTF-8: {e}")))?;

        let base_path = path.to_string_lossy().to_string();
        let scene = super::obj_assets::ObjAssetLoader::load_from_str(&content, &base_path)
            .await
            .map_err(AssetLoadError::Decode)?;

        Ok(BoxedAssetResult::Custom(Box::new(scene)))
    }

    fn clone_box(&self) -> Box<dyn AssetLoader> {
        Box::new(self.clone())
    }
}

/// 创建默认加载器集合
pub fn create_default_loaders() -> Vec<Box<dyn AssetLoader>> {
    let loaders: Vec<Box<dyn AssetLoader>> =
        vec![Box::new(TextureAssetLoader), Box::new(AtlasAssetLoader)];

    // 条件编译：仅在feature启用时添加GLTF加载器
    #[cfg(feature = "gltf")]
    let mut loaders = loaders;

    #[cfg(feature = "gltf")]
    loaders.push(Box::new(GltfAssetLoaderWrapper {
        inner: super::gltf_assets::GltfAssetLoader,
    }));

    #[cfg(feature = "fbx")]
    loaders.push(Box::new(FbxAssetLoaderWrapper {
        inner: super::fbx_assets::FbxAssetLoader,
    }));

    #[cfg(feature = "obj")]
    loaders.push(Box::new(ObjAssetLoaderWrapper {
        inner: super::obj_assets::ObjAssetLoader,
    }));

    #[cfg(not(feature = "gltf"))]
    #[cfg(not(feature = "fbx"))]
    #[cfg(not(feature = "obj"))]
    let loaders = loaders;

    loaders
}

/// 加载器注册表
#[derive(Clone)]
pub struct AssetLoaderRegistry {
    loaders: Vec<Box<dyn AssetLoader>>,
    // 扩展名到加载器索引的映射
    extension_map: std::collections::HashMap<String, usize>,
}

impl AssetLoaderRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            loaders: Vec::new(),
            extension_map: std::collections::HashMap::new(),
        }
    }

    /// 使用默认加载器创建注册表
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        for loader in create_default_loaders() {
            registry.register(loader);
        }
        registry
    }

    /// 注册加载器
    pub fn register(&mut self, loader: Box<dyn AssetLoader>) {
        let index = self.loaders.len();
        for ext in loader.extensions() {
            self.extension_map.insert(ext.to_lowercase(), index);
        }
        self.loaders.push(loader);
    }

    /// 根据文件扩展名获取加载器
    pub fn get_loader(&self, path: &Path) -> Option<&dyn AssetLoader> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.extension_map.get(&ext.to_lowercase()))
            .map(|&index| self.loaders[index].as_ref())
    }

    /// 获取所有加载器
    pub fn loaders(&self) -> &[Box<dyn AssetLoader>] {
        &self.loaders
    }

    /// 检查是否支持某种扩展名
    pub fn supports(&self, extension: &str) -> bool {
        self.extension_map.contains_key(&extension.to_lowercase())
    }
}

impl Default for AssetLoaderRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_with_defaults() {
        let registry = AssetLoaderRegistry::with_defaults();

        // 检查默认支持的扩展名
        assert!(registry.supports("png"));
        assert!(registry.supports("jpg"));
        assert!(registry.supports("atlas"));

        // 检查获取加载器
        let loader = registry.get_loader(Path::new("test.png"));
        assert!(loader.is_some());

        let loader = registry.get_loader(Path::new("test.unknown"));
        assert!(loader.is_none());
    }

    #[test]
    fn test_custom_loader_registration() {
        #[derive(Clone)]
        struct CustomLoader;

        #[async_trait]
        impl AssetLoader for CustomLoader {
            fn extensions(&self) -> &[&str] {
                &["custom"]
            }

            async fn load(
                &self,
                _path: &Path,
                _bytes: Vec<u8>,
            ) -> Result<BoxedAssetResult, AssetLoadError> {
                Ok(BoxedAssetResult::Bytes(vec![]))
            }

            fn clone_box(&self) -> Box<dyn AssetLoader> {
                Box::new(self.clone())
            }
        }

        let mut registry = AssetLoaderRegistry::new();
        registry.register(Box::new(CustomLoader));

        assert!(registry.supports("custom"));
        assert!(registry.get_loader(Path::new("test.custom")).is_some());
    }
}
