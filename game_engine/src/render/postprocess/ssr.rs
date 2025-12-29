//! 屏幕空间反射 (Screen Space Reflection, SSR)
//!
//! 在屏幕空间中计算反射，提供实时反射效果。
//! 适用于光滑表面、水面等需要反射的场景。

use crate::impl_default;
use wgpu::{BindGroupLayout, RenderPipeline};

/// SSR 配置
#[derive(Debug, Clone)]
pub struct SsrConfig {
    /// 是否启用 SSR
    pub enabled: bool,
    /// 最大步进距离
    pub max_distance: f32,
    /// 步进数量
    pub step_count: u32,
    /// 步进大小
    pub step_size: f32,
    /// 二进制搜索迭代次数
    pub binary_search_iterations: u32,
    /// 反射强度 (0.0 - 1.0)
    pub intensity: f32,
    /// 边缘衰减
    pub edge_fade: f32,
    /// 深度阈值
    pub depth_threshold: f32,
    /// 法线阈值
    pub normal_threshold: f32,
}

impl_default!(SsrConfig {
    enabled: false,
    max_distance: 100.0,
    step_count: 32,
    step_size: 0.5,
    binary_search_iterations: 8,
    intensity: 0.8,
    edge_fade: 0.1,
    depth_threshold: 0.01,
    normal_threshold: 0.1,
});

/// SSR 后处理通道
pub struct SsrPass {
    config: SsrConfig,
    pipeline: Option<RenderPipeline>,
    bind_group_layout: Option<BindGroupLayout>,
}

impl SsrPass {
    /// 创建新的 SSR 通道
    pub fn new(config: SsrConfig) -> Self {
        Self {
            config,
            pipeline: None,
            bind_group_layout: None,
        }
    }

    /// 设置配置
    pub fn set_config(&mut self, config: SsrConfig) {
        self.config = config;
    }

    /// 获取配置
    pub fn config(&self) -> &SsrConfig {
        &self.config
    }

    /// 获取配置（可变）
    pub fn config_mut(&mut self) -> &mut SsrConfig {
        &mut self.config
    }
}

/// SSR Uniform 数据
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SsrUniforms {
    /// 最大距离
    pub max_distance: f32,
    /// 步进数量
    pub step_count: u32,
    /// 步进大小
    pub step_size: f32,
    /// 二进制搜索迭代次数
    pub binary_search_iterations: u32,
    /// 反射强度
    pub intensity: f32,
    /// 边缘衰减
    pub edge_fade: f32,
    /// 深度阈值
    pub depth_threshold: f32,
    /// 法线阈值
    pub normal_threshold: f32,
    /// 屏幕尺寸
    pub screen_size: [f32; 2],
    /// 投影矩阵
    pub projection_matrix: [[f32; 4]; 4],
    /// 视图矩阵
    pub view_matrix: [[f32; 4]; 4],
}

impl Default for SsrUniforms {
    fn default() -> Self {
        Self {
            max_distance: 100.0,
            step_count: 32,
            step_size: 0.5,
            binary_search_iterations: 8,
            intensity: 0.8,
            edge_fade: 0.1,
            depth_threshold: 0.01,
            normal_threshold: 0.1,
            screen_size: [1920.0, 1080.0],
            projection_matrix: [[1.0; 4]; 4],
            view_matrix: [[1.0; 4]; 4],
        }
    }
}
