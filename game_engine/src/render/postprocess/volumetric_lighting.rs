//! 体积光 (Volumetric Lighting)
//!
//! 实现体积光效果，包括：
//! - 光线散射 (Light Scattering)
//! - 体积雾 (Volumetric Fog)
//! - 光轴效果 (God Rays)
//! - 体积阴影 (Volumetric Shadows)

use crate::impl_default;
use glam::Vec3;
use wgpu::{BindGroupLayout, RenderPipeline};

/// 体积光配置
#[derive(Debug, Clone)]
pub struct VolumetricLightingConfig {
    /// 是否启用体积光
    pub enabled: bool,
    /// 散射强度 (0.0 - 1.0)
    pub scattering_intensity: f32,
    /// 吸收系数
    pub absorption_coefficient: f32,
    /// 散射系数
    pub scattering_coefficient: f32,
    /// 采样数量
    pub sample_count: u32,
    /// 采样步长
    pub sample_step: f32,
    /// 最大距离
    pub max_distance: f32,
    /// 光轴强度 (0.0 - 1.0)
    pub god_ray_intensity: f32,
    /// 光轴衰减
    pub god_ray_decay: f32,
    /// 光轴权重
    pub god_ray_weight: f32,
    /// 光轴曝光
    pub god_ray_exposure: f32,
    /// 体积雾密度
    pub fog_density: f32,
    /// 体积雾颜色
    pub fog_color: Vec3,
    /// 体积雾高度衰减
    pub fog_height_falloff: f32,
}

impl_default!(VolumetricLightingConfig {
    enabled: false,
    scattering_intensity: 0.5,
    absorption_coefficient: 0.1,
    scattering_coefficient: 0.3,
    sample_count: 32,
    sample_step: 0.5,
    max_distance: 100.0,
    god_ray_intensity: 0.3,
    god_ray_decay: 0.95,
    god_ray_weight: 0.5,
    god_ray_exposure: 0.5,
    fog_density: 0.01,
    fog_color: Vec3::new(0.5, 0.6, 0.7),
    fog_height_falloff: 0.1,
});

/// 体积光后处理通道
pub struct VolumetricLightingPass {
    config: VolumetricLightingConfig,
    pipeline: Option<RenderPipeline>,
    bind_group_layout: Option<BindGroupLayout>,
}

impl VolumetricLightingPass {
    /// 创建新的体积光通道
    pub fn new(config: VolumetricLightingConfig) -> Self {
        Self {
            config,
            pipeline: None,
            bind_group_layout: None,
        }
    }

    /// 设置配置
    pub fn set_config(&mut self, config: VolumetricLightingConfig) {
        self.config = config;
    }

    /// 获取配置
    pub fn config(&self) -> &VolumetricLightingConfig {
        &self.config
    }

    /// 获取配置（可变）
    pub fn config_mut(&mut self) -> &mut VolumetricLightingConfig {
        &mut self.config
    }
}

/// 体积光 Uniform 数据
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VolumetricLightingUniforms {
    /// 散射强度
    pub scattering_intensity: f32,
    /// 吸收系数
    pub absorption_coefficient: f32,
    /// 散射系数
    pub scattering_coefficient: f32,
    /// 采样数量
    pub sample_count: u32,
    /// 采样步长
    pub sample_step: f32,
    /// 最大距离
    pub max_distance: f32,
    /// 光轴强度
    pub god_ray_intensity: f32,
    /// 光轴衰减
    pub god_ray_decay: f32,
    /// 光轴权重
    pub god_ray_weight: f32,
    /// 光轴曝光
    pub god_ray_exposure: f32,
    /// 体积雾密度
    pub fog_density: f32,
    /// 体积雾颜色
    pub fog_color: [f32; 3],
    /// 体积雾高度衰减
    pub fog_height_falloff: f32,
    /// 光源位置
    pub light_position: [f32; 3],
    /// 光源方向
    pub light_direction: [f32; 3],
    /// 光源颜色
    pub light_color: [f32; 3],
    /// 光源强度
    pub light_intensity: f32,
    /// 视图矩阵
    pub view_matrix: [[f32; 4]; 4],
    /// 投影矩阵
    pub projection_matrix: [[f32; 4]; 4],
}

impl Default for VolumetricLightingUniforms {
    fn default() -> Self {
        Self {
            scattering_intensity: 0.5,
            absorption_coefficient: 0.1,
            scattering_coefficient: 0.3,
            sample_count: 32,
            sample_step: 0.5,
            max_distance: 100.0,
            god_ray_intensity: 0.3,
            god_ray_decay: 0.95,
            god_ray_weight: 0.5,
            god_ray_exposure: 0.5,
            fog_density: 0.01,
            fog_color: [0.5, 0.6, 0.7],
            fog_height_falloff: 0.1,
            light_position: [0.0, 10.0, 0.0],
            light_direction: [0.0, -1.0, 0.0],
            light_color: [1.0, 1.0, 1.0],
            light_intensity: 1.0,
            view_matrix: [[1.0; 4]; 4],
            projection_matrix: [[1.0; 4]; 4],
        }
    }
}
