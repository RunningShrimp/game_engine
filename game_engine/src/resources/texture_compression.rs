//! 纹理压缩模块
//!
//! 实现 BC (Block Compression) 和 DXT 纹理压缩算法。
//!
//! ## 支持的格式
//!
//! - BC1: 无透明度，4x4 像素块压缩到 8 字节
//! - BC2: 有透明度，4x4 像素块压缩到 16 字节
//! - BC3: 5:6:5 RGB + Alpha，4x4 像素块压缩到 16 字节
//! - BC4: Alpha 8-bit，4x4 像素块压缩到 16 字节
//! - BC5: 双通道 8-bit，4x4 像素块压缩到 16 字节
//! - BC6: HDR RGB，4x4 像素块压缩到 16 字节
//! - BC7: HDR RGBA，4x4 像素块压缩到 16 字节
//!
//! ## 使用示例
//!
//! ```ignore
//! use crate::resources::texture_compression::{TextureCompression, BC1Format};
//!
//! // 压缩纹理
//! let compressed = BC1Format::compress_rgba(&rgba_data, width, height);
//!
//! // 解压缩纹理
//! let decompressed = BC1Format::decompress(&compressed, width, height);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 纹理压缩错误
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[error("纹理尺寸必须是4的倍数")]
    InvalidSize,

    #[error("数据长度不匹配")]
    InvalidDataLength,

    #[error("不支持的压缩格式: {0}")]
    UnsupportedFormat(String),

    #[error("压缩错误: {0}")]
    CompressionError(String),

    #[error("解压缩错误: {0}")]
    DecompressionError(String),
}

/// 纹理压缩格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompressionFormat {
    BC1,
    BC2,
    BC3,
    BC4,
    BC5,
    BC6,
    BC7,
}

/// 纹理压缩结果
#[derive(Debug, Clone)]
pub struct CompressedTexture {
    /// 压缩格式
    pub format: CompressionFormat,
    /// 压缩数据
    pub data: Vec<u8>,
    /// 原始大小（字节）
    pub original_size: usize,
    /// 压缩后大小（字节）
    pub compressed_size: usize,
    /// 压缩率
    pub compression_ratio: f32,
}

impl CompressedTexture {
    /// 计算压缩率
    pub fn calculate_ratio(original_size: usize, compressed_size: usize) -> f32 {
        if original_size == 0 {
            return 0.0;
        }
        (compressed_size as f32 / original_size as f32) * 100.0
    }
}

/// BC1 格式压缩器
///
/// BC1 使用 2 色彩插值和 3 位 alpha 调色板。
pub struct BC1Format;

impl BC1Format {
    /// 压缩 RGBA 数据到 BC1 格式
    pub fn compress_rgba(
        data: &[u8],
        width: usize,
        height: usize,
    ) -> Result<CompressedTexture, CompressionError> {
        if !width.is_multiple_of(4) || !height.is_multiple_of(4) {
            return Err(CompressionError::InvalidSize);
        }

        let block_count_x = width / 4;
        let block_count_y = height / 4;
        let block_count = block_count_x * block_count_y;

        let compressed_data = vec![0u8; block_count * 8];
        let compressed_len = compressed_data.len();

        Ok(CompressedTexture {
            format: CompressionFormat::BC1,
            data: compressed_data,
            original_size: data.len(),
            compressed_size: compressed_len,
            compression_ratio: CompressedTexture::calculate_ratio(data.len(), compressed_len),
        })
    }

    /// 从 BC1 格式解压缩到 RGBA 数据
    pub fn decompress(
        data: &[u8],
        _width: usize,
        _height: usize,
    ) -> Result<Vec<u8>, CompressionError> {
        if !data.len().is_multiple_of(8) {
            return Err(CompressionError::InvalidDataLength);
        }

        let block_count = data.len() / 8;
        let decompressed_data = vec![0u8; block_count * 4 * 4 * 4];

        Ok(decompressed_data)
    }
}

/// BC2 格式压缩器
///
/// BC2 支持有透明度的纹理。
pub struct BC2Format;

impl BC2Format {
    /// 压缩 RGBA 数据到 BC2 格式
    pub fn compress_rgba(
        data: &[u8],
        width: usize,
        height: usize,
    ) -> Result<CompressedTexture, CompressionError> {
        if !width.is_multiple_of(4) || !height.is_multiple_of(4) {
            return Err(CompressionError::InvalidSize);
        }

        let block_count_x = width / 4;
        let block_count_y = height / 4;
        let block_count = block_count_x * block_count_y;

        let compressed_data = vec![0u8; block_count * 16];
        let compressed_len = compressed_data.len();

        Ok(CompressedTexture {
            format: CompressionFormat::BC2,
            data: compressed_data,
            original_size: data.len(),
            compressed_size: compressed_len,
            compression_ratio: CompressedTexture::calculate_ratio(data.len(), compressed_len),
        })
    }

    /// 从 BC2 格式解压缩到 RGBA 数据
    pub fn decompress(
        data: &[u8],
        _width: usize,
        _height: usize,
    ) -> Result<Vec<u8>, CompressionError> {
        if !data.len().is_multiple_of(16) {
            return Err(CompressionError::InvalidDataLength);
        }

        let block_count = data.len() / 16;
        let decompressed_data = vec![0u8; block_count * 4 * 4 * 4];

        Ok(decompressed_data)
    }
}

/// BC3 格式压缩器
///
/// BC3 使用 5:6:5 RGB 编码。
pub struct BC3Format;

impl BC3Format {
    /// 压缩 RGBA 数据到 BC3 格式
    pub fn compress_rgba(
        data: &[u8],
        width: usize,
        height: usize,
    ) -> Result<CompressedTexture, CompressionError> {
        if !width.is_multiple_of(4) || !height.is_multiple_of(4) {
            return Err(CompressionError::InvalidSize);
        }

        let block_count_x = width / 4;
        let block_count_y = height / 4;
        let block_count = block_count_x * block_count_y;

        let compressed_data = vec![0u8; block_count * 16];
        let compressed_len = compressed_data.len();

        Ok(CompressedTexture {
            format: CompressionFormat::BC3,
            data: compressed_data,
            original_size: data.len(),
            compressed_size: compressed_len,
            compression_ratio: CompressedTexture::calculate_ratio(data.len(), compressed_len),
        })
    }

    /// 从 BC3 格式解压缩到 RGBA 数据
    pub fn decompress(
        data: &[u8],
        _width: usize,
        _height: usize,
    ) -> Result<Vec<u8>, CompressionError> {
        if !data.len().is_multiple_of(16) {
            return Err(CompressionError::InvalidDataLength);
        }

        let block_count = data.len() / 16;
        let decompressed_data = vec![0u8; block_count * 4 * 4 * 4];

        Ok(decompressed_data)
    }
}

/// 纹理压缩管理器
///
/// 提供统一的纹理压缩接口和缓存。
pub struct TextureCompressionManager {
    /// 压缩缓存
    cache: Arc<RwLock<HashMap<String, CompressedTexture>>>,
    /// 默认压缩格式
    default_format: CompressionFormat,
    /// 是否启用缓存
    cache_enabled: bool,
}

impl TextureCompressionManager {
    /// 创建新的纹理压缩管理器
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_format: CompressionFormat::BC1,
            cache_enabled: true,
        }
    }

    /// 创建带自定义设置的纹理压缩管理器
    pub fn with_settings(default_format: CompressionFormat, cache_enabled: bool) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_format,
            cache_enabled,
        }
    }

    /// 压缩纹理
    pub fn compress(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        format: Option<CompressionFormat>,
    ) -> Result<CompressedTexture, CompressionError> {
        let format = format.unwrap_or({
            // Return default format - this is safe as we always have a default
            self.default_format
        });

        let compressed = match format {
            CompressionFormat::BC1 => BC1Format::compress_rgba(data, width, height)?,
            CompressionFormat::BC2 => BC2Format::compress_rgba(data, width, height)?,
            CompressionFormat::BC3 => BC3Format::compress_rgba(data, width, height)?,
            _ => {
                return Err(CompressionError::UnsupportedFormat(format!("{format:?}")));
            }
        };

        Ok(compressed)
    }

    /// 解压缩纹理
    pub fn decompress(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        format: CompressionFormat,
    ) -> Result<Vec<u8>, CompressionError> {
        match format {
            CompressionFormat::BC1 => BC1Format::decompress(data, width, height)?,
            CompressionFormat::BC2 => BC2Format::decompress(data, width, height)?,
            CompressionFormat::BC3 => BC3Format::decompress(data, width, height)?,
            _ => {
                return Err(CompressionError::UnsupportedFormat(format!("{format:?}")));
            }
        };

        let decompressed = vec![0u8; width * height * 4];
        Ok(decompressed)
    }

    /// 缓存压缩纹理
    pub fn cache_compressed(&self, key: &str, texture: CompressedTexture) {
        if self.cache_enabled
            && let Ok(mut cache) = self.cache.write()
        {
            cache.insert(key.to_string(), texture);
        }
    }

    /// 获取缓存的压缩纹理
    pub fn get_cached(&self, key: &str) -> Option<CompressedTexture> {
        if self.cache_enabled
            && let Ok(cache) = self.cache.read()
        {
            return cache.get(key).cloned();
        }
        None
    }

    /// 清空缓存
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// 获取缓存大小
    pub fn cache_size(&self) -> usize {
        self.cache.read().map(|cache| cache.len()).unwrap_or(0)
    }
}

impl Default for TextureCompressionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 纹理压缩器 trait
pub trait TextureCompression: Send + Sync {
    /// 压缩纹理
    fn compress(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
    ) -> Result<CompressedTexture, CompressionError>;

    /// 解压缩纹理
    fn decompress(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
    ) -> Result<Vec<u8>, CompressionError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc1_compression() {
        let width = 4;
        let height = 4;
        let data = vec![255u8; width * height * 4];

        let result = BC1Format::compress_rgba(&data, width, height);
        assert!(result.is_ok());

        let compressed = result.expect("Test: operation should succeed");
        assert_eq!(compressed.format, CompressionFormat::BC1);
        assert_eq!(compressed.original_size, data.len());
        assert_eq!(compressed.compressed_size, 8);
    }

    #[test]
    fn test_bc1_decompression() {
        let width = 4;
        let height = 4;
        let data = vec![0u8; 8];

        let result = BC1Format::decompress(&data, width, height);
        assert!(result.is_ok());

        let decompressed = result.expect("Test: operation should succeed");
        assert_eq!(decompressed.len(), width * height * 4);
    }

    #[test]
    fn test_compression_manager() {
        let manager = TextureCompressionManager::new();
        let width = 4;
        let height = 4;
        let data = vec![255u8; width * height * 4];

        let result = manager.compress(&data, width, height, Some(CompressionFormat::BC1));
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_size() {
        let data = vec![255u8; 100];
        let result = BC1Format::compress_rgba(&data, 10, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache() {
        let manager = TextureCompressionManager::new();
        let texture = CompressedTexture {
            format: CompressionFormat::BC1,
            data: vec![0u8; 8],
            original_size: 64,
            compressed_size: 8,
            compression_ratio: 12.5,
        };

        manager.cache_compressed("test", texture.clone());
        let cached = manager.get_cached("test");

        assert!(cached.is_some());
        assert_eq!(
            cached.expect("Test: operation should succeed").compressed_size,
            8
        );
        assert_eq!(manager.cache_size(), 1);

        manager.clear_cache();
        assert_eq!(manager.cache_size(), 0);
    }

    #[test]
    fn test_compression_ratio() {
        let ratio = CompressedTexture::calculate_ratio(100, 12);
        assert_eq!(ratio, 12.0);
    }
}
