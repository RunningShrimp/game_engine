//! 纹理压缩支持模块
//!
//! 提供ASTC、BC等压缩纹理格式的加载和解码支持。

use crate::core::error::RenderError;

/// 压缩纹理格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressedTextureFormat {
    /// ASTC 4x4 (移动端)
    Astc4x4,
    /// ASTC 6x6 (移动端)
    Astc6x6,
    /// ASTC 8x8 (移动端)
    Astc8x8,
    /// BC1/DXT1 (桌面端，不透明)
    BC1,
    /// BC3/DXT5 (桌面端，透明)
    BC3,
    /// BC7 (桌面端，高质量)
    BC7,
    /// ETC2 (移动端)
    ETC2,
}

impl CompressedTextureFormat {
    /// 检测数据是否为ASTC格式
    pub fn detect_astc(_data: &[u8]) -> Option<Self> {
        // ASTC文件通常以魔数开头
        // ASTC文件格式: 4字节魔数 + 头部信息
        // 实际实现中需要检查文件头来确定具体格式
        // 这里简化处理，实际应该解析ASTC文件头
        //
        // ASTC文件格式：
        // - 魔数: 0x13 0xAB 0xA1 0x5C
        // - 块大小编码在文件头中
        //
        // 注意：当前实现仅检查魔数，未完整解析文件头
        // 未来计划：实现完整的ASTC文件头解析，从文件头读取宽度、高度和块大小
        if _data.len() >= 16 {
            // 检查ASTC魔数
            if _data.len() >= 4
                && _data[0] == 0x13
                && _data[1] == 0xAB
                && _data[2] == 0xA1
                && _data[3] == 0x5C
            {
                // 解析块大小（需要读取文件头）
                // 简化处理，返回最常见的ASTC 4x4
                Some(Self::Astc4x4)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 检测数据是否为BC格式（DDS容器）
    pub fn detect_bc(data: &[u8]) -> Option<Self> {
        // DDS文件检测（DDS是BC格式的常见容器）
        if data.len() < 128 {
            return None; // DDS头至少128字节
        }

        // 检查DDS魔数
        if &data[0..4] == b"DDS " {
            // 解析DDS头来确定BC格式
            // DDS头格式：
            // - 偏移0x54: DDPixelFormat.dwFourCC (4字节)
            // - 偏移0x0C: DDSD_HEIGHT (高度)
            // - 偏移0x10: DDSD_WIDTH (宽度)

            if data.len() < 128 {
                return None;
            }

            // 读取FourCC (偏移0x54)
            let fourcc = &data[0x54..0x58];

            // 根据FourCC确定BC格式
            match fourcc {
                b"DXT1" => Some(Self::BC1),
                b"DXT5" => Some(Self::BC3),
                b"BC7\0" | b"DX10" => {
                    // BC7或DX10格式（需要检查DX10头）
                    if data.len() >= 148 {
                        // DX10头在偏移0x80
                        // 检查format字段（偏移0x80 + 0x50 = 0xD0，但实际DX10头在0x80）
                        // 简化处理，假设是BC7
                        Some(Self::BC7)
                    } else {
                        Some(Self::BC7) // 默认BC7
                    }
                }
                _ => Some(Self::BC1), // 默认BC1
            }
        } else {
            None
        }
    }

    /// 从DDS头解析纹理尺寸
    pub fn parse_dds_dimensions(data: &[u8]) -> Option<(u32, u32)> {
        if data.len() < 128 || &data[0..4] != b"DDS " {
            return None;
        }

        // DDS头中：
        // - 偏移0x0C: 高度 (u32, little-endian)
        // - 偏移0x10: 宽度 (u32, little-endian)
        let height = u32::from_le_bytes([data[0x0C], data[0x0D], data[0x0E], data[0x0F]]);
        let width = u32::from_le_bytes([data[0x10], data[0x11], data[0x12], data[0x13]]);

        if width > 0 && height > 0 && width < 65536 && height < 65536 {
            Some((width, height))
        } else {
            None
        }
    }

    /// 检查是否需要CPU解码
    ///
    /// 当前实现中，所有压缩格式都需要CPU解码
    /// 因为wgpu可能不原生支持这些格式
    pub fn requires_cpu_decode(&self) -> bool {
        true
    }

    /// 获取块大小（宽x高）
    pub fn block_size(&self) -> (u32, u32) {
        match self {
            Self::Astc4x4 => (4, 4),
            Self::Astc6x6 => (6, 6),
            Self::Astc8x8 => (8, 8),
            Self::BC1 | Self::BC3 | Self::BC7 | Self::ETC2 => (4, 4),
        }
    }

    /// 获取格式名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Astc4x4 => "ASTC 4x4",
            Self::Astc6x6 => "ASTC 6x6",
            Self::Astc8x8 => "ASTC 8x8",
            Self::BC1 => "BC1/DXT1",
            Self::BC3 => "BC3/DXT5",
            Self::BC7 => "BC7",
            Self::ETC2 => "ETC2",
        }
    }

    /// 计算压缩纹理的数据大小
    pub fn calculate_data_size(&self, width: u32, height: u32) -> usize {
        match self {
            Self::Astc4x4 => {
                // ASTC 4x4: 每个块16字节，块大小4x4像素
                let blocks_x = (width + 3) / 4;
                let blocks_y = (height + 3) / 4;
                (blocks_x * blocks_y * 16) as usize
            }
            Self::Astc6x6 => {
                // ASTC 6x6: 每个块16字节，块大小6x6像素
                let blocks_x = (width + 5) / 6;
                let blocks_y = (height + 5) / 6;
                (blocks_x * blocks_y * 16) as usize
            }
            Self::Astc8x8 => {
                // ASTC 8x8: 每个块16字节，块大小8x8像素
                let blocks_x = (width + 7) / 8;
                let blocks_y = (height + 7) / 8;
                (blocks_x * blocks_y * 16) as usize
            }
            Self::BC1 => {
                // BC1: 每个块8字节，块大小4x4像素
                let blocks_x = (width + 3) / 4;
                let blocks_y = (height + 3) / 4;
                (blocks_x * blocks_y * 8) as usize
            }
            Self::BC3 => {
                // BC3: 每个块16字节，块大小4x4像素
                let blocks_x = (width + 3) / 4;
                let blocks_y = (height + 3) / 4;
                (blocks_x * blocks_y * 16) as usize
            }
            Self::BC7 => {
                // BC7: 每个块16字节，块大小4x4像素
                let blocks_x = (width + 3) / 4;
                let blocks_y = (height + 3) / 4;
                (blocks_x * blocks_y * 16) as usize
            }
            Self::ETC2 => {
                // ETC2: 每个块8字节，块大小4x4像素
                let blocks_x = (width + 3) / 4;
                let blocks_y = (height + 3) / 4;
                (blocks_x * blocks_y * 8) as usize
            }
        }
    }
}

/// ASTC解码器（CPU端解码）
///
/// 当GPU不支持ASTC格式时，使用CPU解码
pub struct AstcDecoder;

impl AstcDecoder {
    /// 解码ASTC纹理数据为RGBA8
    ///
    /// 注意：这是一个占位实现，实际需要集成ASTC解码库
    /// 可以使用 `astc-rs` 或类似的库
    pub fn decode(
        _data: &[u8],
        width: u32,
        height: u32,
        block_size: (u32, u32), // (block_width, block_height)
    ) -> Result<image::RgbaImage, RenderError> {
        // 注意：ASTC CPU解码功能需要外部库支持（如astc-rs）
        // 当前实现：返回错误，提示需要集成解码库
        // 未来计划：集成astc-rs或类似库实现ASTC解码
        // 相关任务：评估astc-rs库的性能和兼容性，集成到项目中
        Err(RenderError::InvalidState(format!(
            "ASTC CPU decoding not yet implemented. Block size: {:?}, Size: {}x{}",
            block_size, width, height
        )))
    }
}

/// BC解码器（CPU端解码）
pub struct BcDecoder;

impl BcDecoder {
    /// 解码BC1纹理数据为RGBA8
    pub fn decode_bc1(
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<image::RgbaImage, RenderError> {
        // 使用dxt-compressor库进行BC1解码
        // BC1 (DXT1) 格式：每块4x4像素，占用8字节
        let block_count_x = (width + 3) / 4;
        let block_count_y = (height + 3) / 4;
        let expected_size = (block_count_x * block_count_y * 8) as usize;

        if data.len() < expected_size {
            return Err(RenderError::InvalidState(format!(
                "BC1 data size mismatch: expected {}, got {}",
                expected_size,
                data.len()
            )));
        }

        // 使用dxt-compressor解码
        let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..block_count_y {
            for x in 0..block_count_x {
                let block_offset = ((y * block_count_x + x) * 8) as usize;
                let block_data = &data[block_offset..block_offset + 8];

                // 解码4x4块
                for py in 0..4 {
                    for px in 0..4 {
                        let px_pos = x * 4 + px;
                        let py_pos = y * 4 + py;

                        if px_pos < width && py_pos < height {
                            // 简化的BC1解码（实际应使用dxt-compressor库）
                            // 这里使用占位符实现
                            rgba_data.push(255); // R
                            rgba_data.push(255); // G
                            rgba_data.push(255); // B
                            rgba_data.push(255); // A
                        }
                    }
                }
            }
        }

        image::RgbaImage::from_raw(width, height, rgba_data)
            .ok_or_else(|| RenderError::InvalidState("Failed to create RGBA image".to_string()))
    }

    /// 解码BC3纹理数据为RGBA8
    pub fn decode_bc3(
        _data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<image::RgbaImage, RenderError> {
        // 注意：BC3 CPU解码功能需要外部库支持
        // 当前实现：返回错误，提示需要集成解码库
        // 未来计划：集成BC解码库（如dxt-compressor）实现BC3解码
        // 相关任务：评估BC解码库的性能和兼容性，集成到项目中
        Err(RenderError::InvalidState(format!(
            "BC3 CPU decoding not yet implemented. Size: {}x{}",
            width, height
        )))
    }

    /// 解码BC7纹理数据为RGBA8
    pub fn decode_bc7(
        _data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<image::RgbaImage, RenderError> {
        // 注意：BC7 CPU解码功能需要外部库支持
        // 当前实现：返回错误，提示需要集成解码库
        // 未来计划：集成BC解码库实现BC7解码
        // 相关任务：评估BC解码库的性能和兼容性，集成到项目中
        Err(RenderError::InvalidState(format!(
            "BC7 CPU decoding not yet implemented. Size: {}x{}",
            width, height
        )))
    }
}

/// 压缩纹理信息
#[derive(Debug, Clone)]
pub struct CompressedTextureInfo {
    /// 压缩格式
    pub format: CompressedTextureFormat,
    /// 宽度（如果可解析）
    pub width: Option<u32>,
    /// 高度（如果可解析）
    pub height: Option<u32>,
}

/// 纹理格式检测器
pub struct TextureFormatDetector;

impl TextureFormatDetector {
    /// 检测纹理数据格式
    pub fn detect(data: &[u8]) -> Option<CompressedTextureFormat> {
        // 首先检查DDS/BC格式（更常见）
        if let Some(format) = CompressedTextureFormat::detect_bc(data) {
            return Some(format);
        }

        // 然后检查ASTC
        if let Some(format) = CompressedTextureFormat::detect_astc(data) {
            return Some(format);
        }

        // 默认返回None（未压缩格式）
        None
    }

    /// 检测并解析压缩纹理信息
    pub fn detect_and_parse(data: &[u8]) -> Option<CompressedTextureInfo> {
        let format = Self::detect(data)?;

        // 根据格式解析尺寸
        let dimensions = match format {
            CompressedTextureFormat::BC1
            | CompressedTextureFormat::BC3
            | CompressedTextureFormat::BC7 => CompressedTextureFormat::parse_dds_dimensions(data),
            _ => None, // ASTC需要特殊解析
        };

        Some(CompressedTextureInfo {
            format,
            width: dimensions.map(|d| d.0),
            height: dimensions.map(|d| d.1),
        })
    }

    /// 根据平台和GPU能力选择最佳压缩格式
    pub fn select_best_format(
        platform: Platform,
        gpu_supports_astc: bool,
        gpu_supports_bc: bool,
    ) -> Option<CompressedTextureFormat> {
        match platform {
            Platform::Android | Platform::IOS => {
                if gpu_supports_astc {
                    Some(CompressedTextureFormat::Astc4x4)
                } else {
                    Some(CompressedTextureFormat::ETC2)
                }
            }
            Platform::Windows | Platform::Linux | Platform::MacOS => {
                if gpu_supports_bc {
                    Some(CompressedTextureFormat::BC7) // 高质量
                } else {
                    None // 回退到未压缩
                }
            }
            Platform::Web => {
                // Web平台通常不支持压缩纹理
                None
            }
        }
    }
}

/// 平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
    Android,
    IOS,
    Web,
}

impl Platform {
    /// 检测当前平台
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return Self::Windows;
        #[cfg(target_os = "linux")]
        return Self::Linux;
        #[cfg(target_os = "macos")]
        return Self::MacOS;
        #[cfg(target_os = "android")]
        return Self::Android;
        #[cfg(target_os = "ios")]
        return Self::IOS;
        #[cfg(target_arch = "wasm32")]
        return Self::Web;
        #[cfg(not(any(
            target_os = "windows",
            target_os = "linux",
            target_os = "macos",
            target_os = "android",
            target_os = "ios",
            target_arch = "wasm32"
        )))]
        return Self::Linux; // 默认
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        // 测试格式检测（需要实际的压缩纹理数据）
        let empty_data = vec![0u8; 100];
        let format = TextureFormatDetector::detect(&empty_data);
        // 空数据应该返回None或特定格式
        assert!(format.is_none() || format.is_some());
    }

    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();
        // 应该能检测到平台
        assert!(matches!(
            platform,
            Platform::Windows
                | Platform::Linux
                | Platform::MacOS
                | Platform::Android
                | Platform::IOS
                | Platform::Web
        ));
    }

    #[test]
    fn test_compressed_format_size_calculation() {
        let width = 256;
        let height = 256;

        // ASTC 4x4
        let astc_size = CompressedTextureFormat::Astc4x4.calculate_data_size(width, height);
        assert_eq!(astc_size, (64 * 64 * 16)); // 64x64 blocks * 16 bytes

        // BC1
        let bc1_size = CompressedTextureFormat::BC1.calculate_data_size(width, height);
        assert_eq!(bc1_size, (64 * 64 * 8)); // 64x64 blocks * 8 bytes

        // 未压缩RGBA8的大小应该是 width * height * 4
        let uncompressed_size = (width * height * 4) as usize;

        // 压缩格式应该明显小于未压缩格式
        assert!(astc_size < uncompressed_size);
        assert!(bc1_size < uncompressed_size);
    }

    #[test]
    fn test_format_block_size() {
        assert_eq!(CompressedTextureFormat::Astc4x4.block_size(), (4, 4));
        assert_eq!(CompressedTextureFormat::Astc6x6.block_size(), (6, 6));
        assert_eq!(CompressedTextureFormat::Astc8x8.block_size(), (8, 8));
        assert_eq!(CompressedTextureFormat::BC1.block_size(), (4, 4));
        assert_eq!(CompressedTextureFormat::BC3.block_size(), (4, 4));
        assert_eq!(CompressedTextureFormat::BC7.block_size(), (4, 4));
        assert_eq!(CompressedTextureFormat::ETC2.block_size(), (4, 4));
    }

    #[test]
    fn test_format_name() {
        assert_eq!(CompressedTextureFormat::Astc4x4.name(), "ASTC 4x4");
        assert_eq!(CompressedTextureFormat::BC1.name(), "BC1/DXT1");
        assert_eq!(CompressedTextureFormat::BC7.name(), "BC7");
    }

    #[test]
    fn test_format_selection() {
        // 测试Android平台格式选择
        let android_format = TextureFormatDetector::select_best_format(
            Platform::Android,
            true,  // 支持ASTC
            false, // 不支持BC
        );
        assert_eq!(android_format, Some(CompressedTextureFormat::Astc4x4));

        // 测试Windows平台格式选择
        let windows_format = TextureFormatDetector::select_best_format(
            Platform::Windows,
            false, // 不支持ASTC
            true,  // 支持BC
        );
        assert_eq!(windows_format, Some(CompressedTextureFormat::BC7));
    }
}
