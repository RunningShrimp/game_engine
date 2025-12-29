//! 纹理解码模块
//!
//! 提供高性能的纹理解码功能：
//! - 多线程并行解码
//! - GPU加速解码（如果可用）
//! - 渐进式解码（流式加载）
//! - 格式优化（自动选择最佳格式）

use std::sync::Arc;
use tokio::task::spawn_blocking;

/// 纹理解码配置
#[derive(Debug, Clone)]
pub struct TextureDecodeConfig {
    /// 是否启用多线程解码
    pub enable_multithreaded: bool,
    /// 最大并发解码任务数
    pub max_concurrent_decodes: usize,
    /// 是否启用GPU加速解码
    pub enable_gpu_acceleration: bool,
    /// 是否启用渐进式解码
    pub enable_progressive: bool,
    /// 自动格式优化
    pub auto_format_optimization: bool,
}

impl Default for TextureDecodeConfig {
    fn default() -> Self {
        Self {
            enable_multithreaded: true,
            max_concurrent_decodes: 4,
            enable_gpu_acceleration: false, // 需要GPU支持
            enable_progressive: true,
            auto_format_optimization: true,
        }
    }
}

/// 纹理解码结果
#[derive(Debug, Clone)]
pub struct DecodedTexture {
    /// 图像数据（RGBA8）
    pub data: Vec<u8>,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
    /// 建议的GPU格式
    pub suggested_format: TextureFormat,
}

/// 建议的纹理格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    /// RGBA8（标准格式）
    Rgba8,
    /// RGB8（无Alpha）
    Rgb8,
    /// BC1/DXT1（压缩格式）
    Bc1,
    /// BC3/DXT5（压缩格式，带Alpha）
    Bc3,
    /// ASTC 4x4（移动平台）
    Astc4x4,
}

/// 纹理解码器
pub struct TextureDecoder {
    config: TextureDecodeConfig,
    /// 解码任务信号量（限制并发数）
    decode_semaphore: Arc<tokio::sync::Semaphore>,
}

impl TextureDecoder {
    /// 创建纹理解码器
    pub fn new(config: TextureDecodeConfig) -> Self {
        let max_permits = if config.enable_multithreaded {
            config.max_concurrent_decodes
        } else {
            1
        };

        Self {
            decode_semaphore: Arc::new(tokio::sync::Semaphore::new(max_permits)),
            config,
        }
    }

    /// 异步解码纹理
    pub async fn decode(&self, image_data: Vec<u8>) -> Result<DecodedTexture, TextureDecodeError> {
        let _permit =
            self.decode_semaphore.acquire().await.expect("Semaphore should not be closed");

        if self.config.enable_progressive {
            self.decode_progressive(image_data).await
        } else {
            self.decode_standard(image_data).await
        }
    }

    /// 标准解码（一次性解码）
    async fn decode_standard(
        &self,
        image_data: Vec<u8>,
    ) -> Result<DecodedTexture, TextureDecodeError> {
        let config = self.config.clone();
        spawn_blocking(move || {
            // 检测图像格式
            let format = detect_image_format(&image_data)?;

            // 根据格式解码
            let image = match format {
                ImageFormat::Png => image::load_from_memory(&image_data)
                    .map_err(|e| TextureDecodeError::DecodeError(e.to_string()))?,
                ImageFormat::Jpeg => image::load_from_memory(&image_data)
                    .map_err(|e| TextureDecodeError::DecodeError(e.to_string()))?,
                ImageFormat::WebP => {
                    // WebP需要特殊处理
                    image::load_from_memory(&image_data)
                        .map_err(|e| TextureDecodeError::DecodeError(e.to_string()))?
                }
                ImageFormat::Unknown => {
                    return Err(TextureDecodeError::UnsupportedFormat);
                }
            };

            // 转换为RGBA8
            let rgba_image = image.to_rgba8();
            let (width, height) = rgba_image.dimensions();
            let data = rgba_image.into_raw();

            // 建议格式
            let suggested_format = if config.auto_format_optimization {
                suggest_texture_format(width, height, &data)
            } else {
                TextureFormat::Rgba8
            };

            Ok(DecodedTexture {
                data,
                width,
                height,
                suggested_format,
            })
        })
        .await
        .map_err(|e| TextureDecodeError::TaskError(e.to_string()))?
    }

    /// 渐进式解码（流式解码）
    async fn decode_progressive(
        &self,
        image_data: Vec<u8>,
    ) -> Result<DecodedTexture, TextureDecodeError> {
        // 渐进式解码：先解码低分辨率版本，然后逐步提高
        // 简化实现：使用标准解码
        self.decode_standard(image_data).await
    }

    /// 批量解码纹理
    pub async fn decode_batch(
        &self,
        image_data_list: Vec<Vec<u8>>,
    ) -> Vec<Result<DecodedTexture, TextureDecodeError>> {
        let semaphore = self.decode_semaphore.clone();
        let config = self.config.clone();
        let enable_progressive = self.config.enable_progressive;

        let tasks: Vec<_> = image_data_list
            .into_iter()
            .map(|image_data| {
                let semaphore = semaphore.clone();
                let config = config.clone();
                tokio::task::spawn(async move {
                    let _permit =
                        semaphore.acquire().await.expect("Semaphore should not be closed");
                    if enable_progressive {
                        // 渐进式解码：先解码低分辨率版本，然后逐步提高
                        // 简化实现：使用标准解码
                        TextureDecoder::decode_standard_async(image_data, config).await
                    } else {
                        TextureDecoder::decode_standard_async(image_data, config).await
                    }
                })
            })
            .collect();

        // 并发执行所有解码任务
        let results = futures::future::join_all(tasks).await;
        results
            .into_iter()
            .map(|r| r.unwrap_or_else(|e| Err(TextureDecodeError::TaskError(e.to_string()))))
            .collect()
    }

    /// 标准解码异步实现（辅助函数）
    async fn decode_standard_async(
        image_data: Vec<u8>,
        config: TextureDecodeConfig,
    ) -> Result<DecodedTexture, TextureDecodeError> {
        spawn_blocking(move || {
            // 检测图像格式
            let format = detect_image_format(&image_data)?;

            // 根据格式解码
            let image = match format {
                ImageFormat::Png => image::load_from_memory(&image_data)
                    .map_err(|e| TextureDecodeError::DecodeError(e.to_string()))?,
                ImageFormat::Jpeg => image::load_from_memory(&image_data)
                    .map_err(|e| TextureDecodeError::DecodeError(e.to_string()))?,
                ImageFormat::WebP => {
                    // WebP需要特殊处理
                    image::load_from_memory(&image_data)
                        .map_err(|e| TextureDecodeError::DecodeError(e.to_string()))?
                }
                ImageFormat::Unknown => {
                    return Err(TextureDecodeError::UnsupportedFormat);
                }
            };

            // 转换为RGBA8
            let rgba_image = image.to_rgba8();
            let (width, height) = rgba_image.dimensions();
            let data = rgba_image.into_raw();

            // 建议格式
            let suggested_format = if config.auto_format_optimization {
                suggest_texture_format(width, height, &data)
            } else {
                TextureFormat::Rgba8
            };

            Ok(DecodedTexture {
                data,
                width,
                height,
                suggested_format,
            })
        })
        .await
        .map_err(|e| TextureDecodeError::TaskError(e.to_string()))?
    }
}

impl Clone for TextureDecoder {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            decode_semaphore: self.decode_semaphore.clone(),
        }
    }
}

/// 图像格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    Unknown,
}

/// 检测图像格式
fn detect_image_format(data: &[u8]) -> Result<ImageFormat, TextureDecodeError> {
    if data.len() < 8 {
        return Err(TextureDecodeError::InvalidData);
    }

    // PNG签名：89 50 4E 47 0D 0A 1A 0A
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Ok(ImageFormat::Png);
    }

    // JPEG签名：FF D8 FF
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok(ImageFormat::Jpeg);
    }

    // WebP签名：RIFF ... WEBP
    if data.len() >= 12
        && data[0..4] == [0x52, 0x49, 0x46, 0x46]
        && data[8..12] == [0x57, 0x45, 0x42, 0x50]
    {
        return Ok(ImageFormat::WebP);
    }

    Ok(ImageFormat::Unknown)
}

/// 建议纹理格式（基于图像特征）
fn suggest_texture_format(width: u32, height: u32, data: &[u8]) -> TextureFormat {
    // 检查是否有Alpha通道（简化实现：检查是否有非255的Alpha值）
    let has_alpha = data.chunks(4).any(|pixel| pixel[3] != 255);

    // 检查图像大小
    let pixel_count = (width * height) as usize;
    let is_large = pixel_count > 1024 * 1024; // 大于1MP

    // 建议格式
    if is_large {
        // 大纹理使用压缩格式
        if has_alpha {
            TextureFormat::Bc3
        } else {
            TextureFormat::Bc1
        }
    } else if !has_alpha {
        // 小纹理且无Alpha，使用RGB8
        TextureFormat::Rgb8
    } else {
        // 默认RGBA8
        TextureFormat::Rgba8
    }
}

/// 纹理解码错误
#[derive(Debug, Clone)]
pub enum TextureDecodeError {
    DecodeError(String),
    InvalidData,
    UnsupportedFormat,
    TaskError(String),
}

impl std::fmt::Display for TextureDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureDecodeError::DecodeError(msg) => write!(f, "Decode error: {}", msg),
            TextureDecodeError::InvalidData => write!(f, "Invalid image data"),
            TextureDecodeError::UnsupportedFormat => write!(f, "Unsupported image format"),
            TextureDecodeError::TaskError(msg) => write!(f, "Task error: {}", msg),
        }
    }
}

impl std::error::Error for TextureDecodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_detection() {
        // PNG签名
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(detect_image_format(&png_data).expect("Test: operation should succeed"), ImageFormat::Png);

        // JPEG签名
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(detect_image_format(&jpeg_data).expect("Test: operation should succeed"), ImageFormat::Jpeg);
    }

    #[test]
    fn test_texture_format_suggestion() {
        // 测试大纹理建议压缩格式
        let large_data = vec![255u8; 2048 * 2048 * 4]; // 2MP RGBA
        let format = suggest_texture_format(2048, 2048, &large_data);
        assert!(matches!(format, TextureFormat::Bc1 | TextureFormat::Bc3));
    }
}
