use super::{ConfigError, ConfigResult};
use crate::impl_default;
use serde::{Deserialize, Serialize};

/// 图形配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    /// 分辨率
    pub resolution: Resolution,

    /// 垂直同步
    pub vsync: bool,

    /// 全屏模式
    pub fullscreen: bool,

    /// 抗锯齿
    pub anti_aliasing: AntiAliasing,

    /// 阴影质量
    pub shadow_quality: QualityLevel,

    /// 纹理质量
    pub texture_quality: QualityLevel,

    /// 特效质量
    pub effects_quality: QualityLevel,

    /// 光线追踪
    pub ray_tracing: RayTracingConfig,

    /// 超分辨率
    pub upscaling: UpscalingConfig,

    /// 纹理压缩配置
    ///
    /// 控制纹理压缩格式和质量设置
    pub texture_compression: TextureCompressionConfig,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            resolution: Resolution::default(),
            vsync: true,
            fullscreen: false,
            anti_aliasing: AntiAliasing::TAA,
            shadow_quality: QualityLevel::High,
            texture_quality: QualityLevel::High,
            effects_quality: QualityLevel::High,
            ray_tracing: RayTracingConfig::default(),
            upscaling: UpscalingConfig::default(),
            texture_compression: TextureCompressionConfig::default(),
        }
    }
}

impl GraphicsConfig {
    pub fn new() -> Self {
        Self::default()
    }
    /// 验证配置
    pub fn validate(&self) -> ConfigResult<()> {
        if self.resolution.width == 0 || self.resolution.height == 0 {
            return Err(ConfigError::ValidationError(
                "Invalid resolution".to_string(),
            ));
        }
        Ok(())
    }
}

/// 分辨率
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    /// 宽度（像素）
    pub width: u32,
    /// 高度（像素）
    pub height: u32,
}

impl_default!(Resolution {
    width: 1920,
    height: 1080,
});

/// 抗锯齿方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AntiAliasing {
    /// 无抗锯齿
    None,
    /// FXAA
    FXAA,
    /// SMAA
    SMAA,
    /// TAA (时间抗锯齿)
    TAA,
    /// MSAA 2x
    MSAA2x,
    /// MSAA 4x
    MSAA4x,
    /// MSAA 8x
    MSAA8x,
}

/// 质量等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum QualityLevel {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 超高
    Ultra,
}

/// 光线追踪配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RayTracingConfig {
    /// 是否启用
    pub enabled: bool,

    /// 光线追踪阴影
    pub shadows: bool,

    /// 光线追踪反射
    pub reflections: bool,

    /// 光线追踪全局光照
    pub global_illumination: bool,

    /// 光线追踪环境光遮蔽
    pub ambient_occlusion: bool,
}

impl_default!(RayTracingConfig {
    enabled: false,
    shadows: false,
    reflections: false,
    global_illumination: false,
    ambient_occlusion: false,
});

/// 超分辨率配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpscalingConfig {
    /// 是否启用
    pub enabled: bool,

    /// 技术选择
    pub technology: UpscalingTechnology,

    /// 质量模式
    pub quality: UpscalingQuality,

    /// 渲染分辨率缩放
    pub render_scale: f32,
}

impl_default!(UpscalingConfig {
    enabled: false,
    technology: UpscalingTechnology::Auto,
    quality: UpscalingQuality::Quality,
    render_scale: 0.67, // 67% 渲染分辨率
});

/// 超分辨率技术
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpscalingTechnology {
    /// 自动选择
    Auto,
    /// NVIDIA DLSS
    DLSS,
    /// AMD FSR
    FSR,
    /// Intel XeSS
    XeSS,
    /// NPU AI超分
    NpuAI,
}

/// 超分辨率质量
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpscalingQuality {
    /// 超高质量
    UltraQuality,
    /// 质量
    Quality,
    /// 平衡
    Balanced,
    /// 性能
    Performance,
    /// 超性能
    UltraPerformance,
}

/// 纹理压缩配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureCompressionConfig {
    /// 是否启用纹理压缩
    pub enabled: bool,

    /// 首选压缩格式（按优先级排序）
    pub preferred_formats: Vec<TextureCompressionFormat>,

    /// 压缩质量（影响压缩时间和文件大小）
    pub compression_quality: CompressionQuality,

    /// 是否在运行时压缩（如果源文件未压缩）
    pub runtime_compression: bool,

    /// 最大纹理尺寸（超过此尺寸的纹理会被压缩）
    pub max_uncompressed_size: u32,
}

impl Default for TextureCompressionConfig {
    fn default() -> Self {
        use crate::render::texture_compression::Platform;
        let platform = Platform::current();

        // 根据平台设置默认格式优先级
        let preferred_formats = match platform {
            Platform::Android | Platform::IOS => {
                vec![
                    TextureCompressionFormat::Astc4x4,
                    TextureCompressionFormat::ETC2,
                ]
            }
            Platform::Windows | Platform::Linux | Platform::MacOS => {
                vec![
                    TextureCompressionFormat::BC7,
                    TextureCompressionFormat::BC3,
                    TextureCompressionFormat::BC1,
                ]
            }
            Platform::Web => {
                vec![] // Web平台通常不支持压缩纹理
            }
        };

        Self {
            enabled: true,
            preferred_formats,
            compression_quality: CompressionQuality::Balanced,
            runtime_compression: false, // 默认不启用运行时压缩（性能考虑）
            max_uncompressed_size: 1024, // 1024x1024以上的纹理才压缩
        }
    }
}

/// 纹理压缩格式（用于配置）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureCompressionFormat {
    /// ASTC 4x4
    Astc4x4,
    /// ASTC 6x6
    Astc6x6,
    /// ASTC 8x8
    Astc8x8,
    /// BC1/DXT1
    BC1,
    /// BC3/DXT5
    BC3,
    /// BC7
    BC7,
    /// ETC2
    ETC2,
}

impl From<TextureCompressionFormat>
    for crate::render::texture_compression::CompressedTextureFormat
{
    fn from(format: TextureCompressionFormat) -> Self {
        match format {
            TextureCompressionFormat::Astc4x4 => Self::Astc4x4,
            TextureCompressionFormat::Astc6x6 => Self::Astc6x6,
            TextureCompressionFormat::Astc8x8 => Self::Astc8x8,
            TextureCompressionFormat::BC1 => Self::BC1,
            TextureCompressionFormat::BC3 => Self::BC3,
            TextureCompressionFormat::BC7 => Self::BC7,
            TextureCompressionFormat::ETC2 => Self::ETC2,
        }
    }
}

impl From<crate::render::texture_compression::CompressedTextureFormat>
    for TextureCompressionFormat
{
    fn from(format: crate::render::texture_compression::CompressedTextureFormat) -> Self {
        match format {
            crate::render::texture_compression::CompressedTextureFormat::Astc4x4 => Self::Astc4x4,
            crate::render::texture_compression::CompressedTextureFormat::Astc6x6 => Self::Astc6x6,
            crate::render::texture_compression::CompressedTextureFormat::Astc8x8 => Self::Astc8x8,
            crate::render::texture_compression::CompressedTextureFormat::BC1 => Self::BC1,
            crate::render::texture_compression::CompressedTextureFormat::BC3 => Self::BC3,
            crate::render::texture_compression::CompressedTextureFormat::BC7 => Self::BC7,
            crate::render::texture_compression::CompressedTextureFormat::ETC2 => Self::ETC2,
        }
    }
}

/// 压缩质量
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionQuality {
    /// 最快（低质量，小文件）
    Fastest,
    /// 快速（较低质量）
    Fast,
    /// 平衡（默认）
    Balanced,
    /// 慢速（高质量）
    Slow,
    /// 最慢（最高质量，大文件）
    Slowest,
}
