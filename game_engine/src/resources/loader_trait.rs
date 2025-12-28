//! 资源加载器实现
//!
//! 提供常用资源类型的加载器实现，包括纹理、模型、音频等。

use super::resource_trait::{Resource, ResourceError, ResourceLoader, ResourceMetadata};
use std::path::Path;

/// 纹理资源
#[derive(Debug, Clone)]
pub struct TextureResource {
    metadata: ResourceMetadata,
    image: image::RgbaImage,
    is_linear: bool,
}

impl Resource for TextureResource {
    fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }

    fn size_bytes(&self) -> usize {
        self.image.width() as usize * self.image.height() as usize * 4
    }
}

impl TextureResource {
    /// 获取图像数据
    pub fn image(&self) -> &image::RgbaImage {
        &self.image
    }

    /// 检查是否为线性颜色空间
    pub fn is_linear(&self) -> bool {
        self.is_linear
    }
}

/// 纹理加载器
pub struct TextureLoader {
    default_linear: bool,
}

impl TextureLoader {
    /// 创建新的纹理加载器
    pub fn new(default_linear: bool) -> Self {
        Self { default_linear }
    }
}

impl ResourceLoader for TextureLoader {
    type Resource = TextureResource;
    type Context = (); // 纹理加载不需要特殊上下文

    async fn load(
        &self,
        path: &Path,
        _ctx: &Self::Context,
    ) -> Result<Self::Resource, ResourceError> {
        let bytes = tokio::fs::read(path).await?;
        let image = image::load_from_memory(&bytes)
            .map_err(|e| ResourceError::Decode(e.to_string()))?
            .to_rgba8();

        let metadata = ResourceMetadata::new(
            path.to_path_buf(),
            image.width() as usize * image.height() as usize * 4,
            "texture",
        );

        Ok(TextureResource {
            metadata,
            image,
            is_linear: self.default_linear,
        })
    }

    async fn metadata(&self, path: &Path) -> Result<ResourceMetadata, ResourceError> {
        let path_buf = path.to_path_buf();
        let file_metadata = std::fs::metadata(path)?;
        let size_bytes = file_metadata.len() as usize;
        let last_modified = file_metadata.modified().ok();

        // 尝试读取图像尺寸
        let bytes = tokio::fs::read(path).await?;
        let img =
            image::load_from_memory(&bytes).map_err(|e| ResourceError::Decode(e.to_string()))?;

        let mut metadata = ResourceMetadata::new(path_buf, size_bytes, "texture");
        metadata.last_modified = last_modified;
        metadata.custom.insert("width".to_string(), img.width().to_string());
        metadata.custom.insert("height".to_string(), img.height().to_string());

        Ok(metadata)
    }
}

/// 模型资源
#[derive(Debug, Clone)]
pub struct ModelResource {
    metadata: ResourceMetadata,
    data: Vec<u8>,
}

impl Resource for ModelResource {
    fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }
}

impl ModelResource {
    /// 获取模型数据
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// 模型加载器
pub struct ModelLoader;

impl ResourceLoader for ModelLoader {
    type Resource = ModelResource;
    type Context = ();

    async fn load(
        &self,
        path: &Path,
        _ctx: &Self::Context,
    ) -> Result<Self::Resource, ResourceError> {
        let data = tokio::fs::read(path).await?;
        let metadata = ResourceMetadata::new(path.to_path_buf(), data.len(), "model");

        Ok(ModelResource { metadata, data })
    }
}

/// 音频资源
#[derive(Debug, Clone)]
pub struct AudioResource {
    metadata: ResourceMetadata,
    data: Vec<u8>,
}

impl Resource for AudioResource {
    fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }
}

impl AudioResource {
    /// 获取音频数据
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// 音频加载器
pub struct AudioLoader;

impl ResourceLoader for AudioLoader {
    type Resource = AudioResource;
    type Context = ();

    async fn load(
        &self,
        path: &Path,
        _ctx: &Self::Context,
    ) -> Result<Self::Resource, ResourceError> {
        let data = tokio::fs::read(path).await?;
        let metadata = ResourceMetadata::new(path.to_path_buf(), data.len(), "audio");

        Ok(AudioResource { metadata, data })
    }
}
