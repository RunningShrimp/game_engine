//! # Texture Optimizer - 纹理优化器
//!
//! 本模块实现纹理压缩和优化功能。

use super::pipeline::{OptimizationError};
use std::path::{Path, PathBuf};
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use std::fs;

/// 纹理压缩格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionFormat {
    /// BC1 (DXT1) - 4:1压缩比，无透明度
    BC1,
    /// BC3 (DXT5) - 4:1压缩比，有透明度
    BC3,
    /// BC4 (RGTC1) - 单通道
    BC4,
    /// BC5 (RGTC2) - 双通道
    BC5,
    /// BC7 - 高质量，4:1压缩比
    BC7,
    /// ASTC 4x4 - 自适应压缩
    ASTC4x4,
    /// ETC2 - 移动平台
    ETC2,
}

impl CompressionFormat {
    /// 获取每像素位数（bpp）
    pub fn bits_per_pixel(&self) -> f32 {
        match self {
            CompressionFormat::BC1 => 4.0,
            CompressionFormat::BC3 => 8.0,
            CompressionFormat::BC4 => 4.0,
            CompressionFormat::BC5 => 8.0,
            CompressionFormat::BC7 => 8.0,
            CompressionFormat::ASTC4x4 => 8.0,
            CompressionFormat::ETC2 => 4.0,
        }
    }

    /// 获取块大小（字节）
    pub fn block_size(&self) -> u32 {
        match self {
            CompressionFormat::BC1 => 8,
            CompressionFormat::BC3 => 16,
            CompressionFormat::BC4 => 8,
            CompressionFormat::BC5 => 16,
            CompressionFormat::BC7 => 16,
            CompressionFormat::ASTC4x4 => 16,
            CompressionFormat::ETC2 => 8,
        }
    }

    /// 获取块尺寸
    pub fn block_dimensions(&self) -> (u32, u32) {
        match self {
            CompressionFormat::BC1 => (4, 4),
            CompressionFormat::BC3 => (4, 4),
            CompressionFormat::BC4 => (4, 4),
            CompressionFormat::BC5 => (4, 4),
            CompressionFormat::BC7 => (4, 4),
            CompressionFormat::ASTC4x4 => (4, 4),
            CompressionFormat::ETC2 => (4, 4),
        }
    }
}

/// 纹理优化选项
#[derive(Debug, Clone)]
pub struct TextureOptimizerOptions {
    /// 压缩格式
    pub compression_format: CompressionFormat,

    /// 生成MIP链
    pub generate_mipmaps: bool,

    /// 最大MIP级别
    pub max_mip_levels: Option<u32>,

    /// 质量设置 (0-100)
    pub quality: u32,

    /// 保持原始尺寸
    pub preserve_size: bool,

    /// 最大尺寸
    pub max_resolution: Option<(u32, u32)>,

    /// 使用SRGB颜色空间
    pub srgb: bool,
}

impl Default for TextureOptimizerOptions {
    fn default() -> Self {
        Self {
            compression_format: CompressionFormat::BC7,
            generate_mipmaps: true,
            max_mip_levels: None,
            quality: 80,
            preserve_size: false,
            max_resolution: Some((2048, 2048)),
            srgb: true,
        }
    }
}

/// 压缩后的纹理
#[derive(Debug, Clone)]
pub struct CompressedTexture {
    pub format: CompressionFormat,
    pub width: u32,
    pub height: u32,
    pub mipmaps: Vec<MipmapLevel>,
    pub original_size: u64,
    pub compressed_size: u64,
}

/// MIP级别
#[derive(Debug, Clone)]
pub struct MipmapLevel {
    pub level: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub size_bytes: usize,
}

/// 纹理优化器
pub struct TextureOptimizer {
    options: TextureOptimizerOptions,
}

impl TextureOptimizer {
    /// 创建新的纹理优化器
    pub fn new(options: TextureOptimizerOptions) -> Self {
        Self { options }
    }

    /// 使用默认选项创建
    pub fn with_defaults() -> Self {
        Self::new(TextureOptimizerOptions::default())
    }

    /// 压缩纹理
    pub async fn compress_texture(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<bool, OptimizationError> {
        // 1. 加载原始纹理
        let img = image::open(input_path)
            .map_err(|e| OptimizationError::TextureError(format!("Failed to load image: {}", e)))?;

        // 2. 转换为RGBA8
        let rgba_img = self.to_rgba8(&img);

        // 3. 调整大小（如果需要）
        let processed_img = self.resize_if_needed(&rgba_img);

        // 4. 生成MIP链
        let mipmaps = if self.options.generate_mipmaps {
            self.generate_mipmaps(&processed_img)?
        } else {
            vec![MipmapLevel {
                level: 0,
                width: processed_img.width(),
                height: processed_img.height(),
                data: self.raw_data(&processed_img),
                size_bytes: (processed_img.width() * processed_img.height() * 4) as usize,
            }]
        };

        // 5. 压缩每个MIP级别
        let compressed_mipmaps: Result<Vec<_>, OptimizationError> = mipmaps
            .iter()
            .map(|mip| self.compress_mipmap(mip))
            .collect();

        let compressed_mipmaps = compressed_mipmaps?;

        // 6. 计算大小
        let original_size = self.calculate_original_size(&processed_img, &mipmaps);
        let compressed_size: u64 = compressed_mipmaps.iter().map(|m| m.data.len() as u64).sum();

        // 7. 保存压缩纹理
        self.save_compressed_texture(
            &CompressedTexture {
                format: self.options.compression_format,
                width: processed_img.width(),
                height: processed_img.height(),
                mipmaps: compressed_mipmaps,
                original_size,
                compressed_size,
            },
            output_path,
        )?;

        let compression_ratio = compressed_size as f64 / original_size as f64;

        if self.options.quality > 50 {
            println!(
                "  Compressed: {} -> {} ({:.1}%)",
                input_path.display(),
                output_path.display(),
                compression_ratio * 100.0
            );
        }

        Ok(true)
    }

    /// 转换为RGBA8格式
    fn to_rgba8(&self, img: &DynamicImage) -> RgbaImage {
        img.to_rgba8()
    }

    /// 调整大小（如果需要）
    fn resize_if_needed(&self, img: &RgbaImage) -> RgbaImage {
        if self.options.preserve_size {
            return img.clone();
        }

        if let Some((max_w, max_h)) = self.options.max_resolution {
            if img.width() > max_w || img.height() > max_h {
                // 计算缩放比例
                let scale = (max_w as f32 / img.width() as f32)
                    .min(max_h as f32 / img.height() as f32);

                let new_width = (img.width() as f32 * scale).round() as u32;
                let new_height = (img.height() as f32 * scale).round() as u32;

                return image::imageops::resize(
                    img,
                    new_width,
                    new_height,
                    image::imageops::FilterType::Lanczos3,
                );
            }
        }

        img.clone()
    }

    /// 生成MIP链
    fn generate_mipmaps(&self, img: &RgbaImage) -> Result<Vec<MipmapLevel>, OptimizationError> {
        let mut mipmaps = Vec::new();

        let max_levels = self.options.max_mip_levels.unwrap_or_else(|| {
            let dim = img.width().max(img.height());
            (dim as f32).log2().floor() as u32 + 1
        });

        let mut current_img = img.clone();

        for level in 0..max_levels {
            let width = current_img.width();
            let height = current_img.height();

            if width < 1 || height < 1 {
                break;
            }

            mipmaps.push(MipmapLevel {
                level,
                width,
                height,
                data: self.raw_data(&current_img),
                size_bytes: (width * height * 4) as usize,
            });

            // 继续生成下一个MIP级别
            if width == 1 && height == 1 {
                break;
            }

            let new_width = width.max(1) / 2;
            let new_height = height.max(1) / 2;

            current_img = image::imageops::resize(
                &current_img,
                new_width,
                new_height,
                image::imageops::FilterType::Lanczos3,
            );
        }

        Ok(mipmaps)
    }

    /// 获取原始数据
    fn raw_data(&self, img: &RgbaImage) -> Vec<u8> {
        img.as_raw().clone()
    }

    /// 压缩单个MIP级别
    fn compress_mipmap(&self, mip: &MipmapLevel) -> Result<MipmapLevel, OptimizationError> {
        let compressed_data = match self.options.compression_format {
            CompressionFormat::BC1 => self.compress_bc1(&mip.data, mip.width, mip.height)?,
            CompressionFormat::BC3 => self.compress_bc3(&mip.data, mip.width, mip.height)?,
            CompressionFormat::BC7 => self.compress_bc7(&mip.data, mip.width, mip.height)?,
            _ => {
                // 其他格式暂时不压缩（返回原始数据）
                return Ok(MipmapLevel {
                    level: mip.level,
                    width: mip.width,
                    height: mip.height,
                    data: mip.data.clone(),
                    size_bytes: mip.data.len(),
                });
            }
        };

        Ok(MipmapLevel {
            level: mip.level,
            width: mip.width,
            height: mip.height,
            data: compressed_data,
            size_bytes: compressed_data.len(),
        })
    }

    /// BC1压缩（简化实现）
    fn compress_bc1(&self, _data: &[u8], width: u32, height: u32) -> Result<Vec<u8>, OptimizationError> {
        // 简化实现：计算压缩后的大小并返回零数据
        let (block_w, block_h) = self.options.compression_format.block_dimensions();
        let blocks_x = (width + block_w - 1) / block_w;
        let blocks_y = (height + block_h - 1) / block_h;
        let num_blocks = blocks_x * blocks_y;
        let compressed_size = (num_blocks * self.options.compression_format.block_size()) as usize;

        // 实际项目应使用真正的BC1压缩算法
        Ok(vec![0; compressed_size])
    }

    /// BC3压缩（简化实现）
    fn compress_bc3(&self, data: &[u8], width: u32, height: u32) -> Result<Vec<u8>, OptimizationError> {
        // BC3 = BC1 (颜色) + BC4 (Alpha)
        let color_data = self.compress_bc1(data, width, height)?;

        let (block_w, block_h) = self.options.compression_format.block_dimensions();
        let blocks_x = (width + block_w - 1) / block_w;
        let blocks_y = (height + block_h - 1) / block_h;
        let num_blocks = blocks_x * blocks_y;
        let alpha_size = (num_blocks * 8) as usize; // BC4使用8字节每块

        let mut compressed = Vec::with_capacity(color_data.len() + alpha_size);
        compressed.extend_from_slice(&color_data);
        compressed.extend_from_slice(&vec![0; alpha_size]);

        Ok(compressed)
    }

    /// BC7压缩（简化实现）
    fn compress_bc7(&self, _data: &[u8], width: u32, height: u32) -> Result<Vec<u8>, OptimizationError> {
        let (block_w, block_h) = self.options.compression_format.block_dimensions();
        let blocks_x = (width + block_w - 1) / block_w;
        let blocks_y = (height + block_h - 1) / block_h;
        let num_blocks = blocks_x * blocks_y;
        let compressed_size = (num_blocks * self.options.compression_format.block_size()) as usize;

        // 实际项目应使用真正的BC7压缩算法
        Ok(vec![0; compressed_size])
    }

    /// 计算原始大小
    fn calculate_original_size(&self, img: &RgbaImage, mipmaps: &[MipmapLevel]) -> u64 {
        mipmaps.iter().map(|m| m.data.len() as u64).sum()
    }

    /// 保存压缩纹理
    fn save_compressed_texture(
        &self,
        texture: &CompressedTexture,
        output_path: &Path,
    ) -> Result<(), OptimizationError> {
        // 创建输出目录
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| OptimizationError::IoError(format!("Failed to create directory: {}", e)))?;
        }

        // 简化实现：保存为PNG（实际项目应保存为专用的压缩纹理格式）
        // 这里我们创建一个占位文件
        fs::write(
            output_path,
            format!(
                "{{\"format\": {:?}, \"width\": {}, \"height\": {}, \"mipmaps\": {}, \"compressed_size\": {}}}",
                texture.format,
                texture.width,
                texture.height,
                texture.mipmaps.len(),
                texture.compressed_size
            ),
        )
        .map_err(|e| OptimizationError::IoError(format!("Failed to write file: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_format_properties() {
        assert_eq!(CompressionFormat::BC1.bits_per_pixel(), 4.0);
        assert_eq!(CompressionFormat::BC7.bits_per_pixel(), 8.0);

        assert_eq!(CompressionFormat::BC1.block_size(), 8);
        assert_eq!(CompressionFormat::BC7.block_size(), 16);
    }

    #[test]
    fn test_mipmap_generation() {
        let optimizer = TextureOptimizer::with_defaults();

        // 创建4x4测试图像
        let img: RgbaImage = ImageBuffer::new(4, 4);

        let mipmaps = optimizer.generate_mipmaps(&img).unwrap();

        // 应该有3个MIP级别：4x4, 2x2, 1x1
        assert_eq!(mipmaps.len(), 3);
        assert_eq!(mipmaps[0].width, 4);
        assert_eq!(mipmaps[1].width, 2);
        assert_eq!(mipmaps[2].width, 1);
    }
}
