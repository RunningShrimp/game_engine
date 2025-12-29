//! 程序化噪声后处理效果
//!
//! 提供基于程序化噪声的后处理效果，包括：
//! - 胶片颗粒 (Film Grain)
//! - 色差 (Chromatic Aberration)
//! - 扫描线 (Scanlines)
//! - 噪点 (Noise)
//! - 失真效果 (Distortion)

use crate::impl_default;
use wgpu::{BindGroupLayout, RenderPipeline};

/// 程序化噪声配置
#[derive(Debug, Clone)]
pub struct ProceduralNoiseConfig {
    /// 是否启用程序化噪声
    pub enabled: bool,
    /// 胶片颗粒强度 (0.0 - 1.0)
    pub film_grain_intensity: f32,
    /// 胶片颗粒大小
    pub film_grain_size: f32,
    /// 色差强度 (0.0 - 1.0)
    pub chromatic_aberration_intensity: f32,
    /// 色差偏移
    pub chromatic_aberration_offset: f32,
    /// 扫描线强度 (0.0 - 1.0)
    pub scanline_intensity: f32,
    /// 扫描线频率
    pub scanline_frequency: f32,
    /// 噪点强度 (0.0 - 1.0)
    pub noise_intensity: f32,
    /// 噪点缩放
    pub noise_scale: f32,
    /// 失真强度 (0.0 - 1.0)
    pub distortion_intensity: f32,
    /// 失真频率
    pub distortion_frequency: f32,
    /// 时间因子（用于动画）
    pub time_factor: f32,
}

impl_default!(ProceduralNoiseConfig {
    enabled: false,
    film_grain_intensity: 0.1,
    film_grain_size: 1.0,
    chromatic_aberration_intensity: 0.0,
    chromatic_aberration_offset: 0.002,
    scanline_intensity: 0.0,
    scanline_frequency: 240.0,
    noise_intensity: 0.0,
    noise_scale: 100.0,
    distortion_intensity: 0.0,
    distortion_frequency: 10.0,
    time_factor: 0.0,
});

/// 程序化噪声后处理通道
pub struct ProceduralNoisePass {
    config: ProceduralNoiseConfig,
    pipeline: Option<RenderPipeline>,
    bind_group_layout: Option<BindGroupLayout>,
}

impl ProceduralNoisePass {
    /// 创建新的程序化噪声通道
    pub fn new(config: ProceduralNoiseConfig) -> Self {
        Self {
            config,
            pipeline: None,
            bind_group_layout: None,
        }
    }

    /// 设置配置
    pub fn set_config(&mut self, config: ProceduralNoiseConfig) {
        self.config = config;
    }

    /// 获取配置
    pub fn config(&self) -> &ProceduralNoiseConfig {
        &self.config
    }

    /// 获取配置（可变）
    pub fn config_mut(&mut self) -> &mut ProceduralNoiseConfig {
        &mut self.config
    }

    /// 更新时间因子（用于动画效果）
    pub fn update_time(&mut self, delta_time: f32) {
        self.config.time_factor += delta_time;
    }
}

/// 程序化噪声 Uniform 数据
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProceduralNoiseUniforms {
    /// 胶片颗粒强度
    pub film_grain_intensity: f32,
    /// 胶片颗粒大小
    pub film_grain_size: f32,
    /// 色差强度
    pub chromatic_aberration_intensity: f32,
    /// 色差偏移
    pub chromatic_aberration_offset: f32,
    /// 扫描线强度
    pub scanline_intensity: f32,
    /// 扫描线频率
    pub scanline_frequency: f32,
    /// 噪点强度
    pub noise_intensity: f32,
    /// 噪点缩放
    pub noise_scale: f32,
    /// 失真强度
    pub distortion_intensity: f32,
    /// 失真频率
    pub distortion_frequency: f32,
    /// 时间因子
    pub time_factor: f32,
    /// 屏幕尺寸
    pub screen_size: [f32; 2],
}

impl Default for ProceduralNoiseUniforms {
    fn default() -> Self {
        Self {
            film_grain_intensity: 0.1,
            film_grain_size: 1.0,
            chromatic_aberration_intensity: 0.0,
            chromatic_aberration_offset: 0.002,
            scanline_intensity: 0.0,
            scanline_frequency: 240.0,
            noise_intensity: 0.0,
            noise_scale: 100.0,
            distortion_intensity: 0.0,
            distortion_frequency: 10.0,
            time_factor: 0.0,
            screen_size: [1920.0, 1080.0],
        }
    }
}

