/// 图形配置

use serde::{Deserialize, Serialize};
use super::{ConfigResult, ConfigError};

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
        }
    }
}

impl GraphicsConfig {
    /// 验证配置
    pub fn validate(&self) -> ConfigResult<()> {
        if self.resolution.width == 0 || self.resolution.height == 0 {
            return Err(ConfigError::ValidationError("Invalid resolution".to_string()));
        }
        Ok(())
    }
}

/// 分辨率
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Default for Resolution {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
        }
    }
}

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl Default for RayTracingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            shadows: false,
            reflections: false,
            global_illumination: false,
            ambient_occlusion: false,
        }
    }
}

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

impl Default for UpscalingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            technology: UpscalingTechnology::Auto,
            quality: UpscalingQuality::Quality,
            render_scale: 0.67, // 67% 渲染分辨率
        }
    }
}

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
