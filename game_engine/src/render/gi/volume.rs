//! # DDGI体积配置和管理
//!
//! 定义DDGI体积的配置参数和默认实现。

use crate::impl_default;
use glam::UVec3;

// 简化的错误类型用于配置验证
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

/// DDGI配置
#[derive(Debug, Clone)]
pub struct DDGIConfig {
    /// 探针间距（世界空间单位）
    pub probe_spacing: f32,
    /// 探针数量（X, Y, Z）
    pub probe_counts: UVec3,
    /// 辐照度纹理分辨率（每个探针每个面的纹理大小）
    pub irradiance_resolution: u32,
    /// 深度纹理分辨率
    pub depth_resolution: u32,
    /// 最大深度
    pub max_depth: f32,
    /// 法线偏移（防止光泄漏）
    pub normal_bias: f32,
    /// 更新率（每N帧更新一次）
    pub update_rate: u32,
    /// 是否启用时序滤波
    pub enable_temporal_filter: bool,
    /// 时序滤波强度（0-1）
    pub temporal_filter_alpha: f32,
    /// 是否启用光照传播
    pub enable_light_propagation: bool,
    /// 光照传播迭代次数
    pub propagation_iterations: u32,
}

impl_default! {
    DDGIConfig {
        probe_spacing: 2.0,
        probe_counts: UVec3::new(10, 10, 10),
        irradiance_resolution: 16,
        depth_resolution: 16,
        max_depth: 50.0,
        normal_bias: 0.05,
        update_rate: 3,
        enable_temporal_filter: true,
        temporal_filter_alpha: 0.9,
        enable_light_propagation: true,
        propagation_iterations: 2,
    }
}

impl DDGIConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.probe_spacing <= 0.0 {
            return Err(ConfigError::Invalid(
                "Probe spacing must be positive".to_string(),
            ));
        }

        if self.probe_counts.x == 0 || self.probe_counts.y == 0 || self.probe_counts.z == 0 {
            return Err(ConfigError::Invalid(
                "Probe counts must be non-zero".to_string(),
            ));
        }

        if !self.irradiance_resolution.is_power_of_two() {
            return Err(ConfigError::Invalid(
                "Irradiance resolution must be power of 2".to_string(),
            ));
        }

        if !self.depth_resolution.is_power_of_two() {
            return Err(ConfigError::Invalid(
                "Depth resolution must be power of 2".to_string(),
            ));
        }

        if self.max_depth <= 0.0 {
            return Err(ConfigError::Invalid(
                "Max depth must be positive".to_string(),
            ));
        }

        if self.normal_bias < 0.0 {
            return Err(ConfigError::Invalid(
                "Normal bias must be non-negative".to_string(),
            ));
        }

        if self.update_rate == 0 {
            return Err(ConfigError::Invalid(
                "Update rate must be positive".to_string(),
            ));
        }

        if !(0.0..=1.0).contains(&self.temporal_filter_alpha) {
            return Err(ConfigError::Invalid(
                "Temporal filter alpha must be in range [0, 1]".to_string(),
            ));
        }

        Ok(())
    }

    /// 计算总探针数量
    pub fn total_probes(&self) -> u32 {
        self.probe_counts.x * self.probe_counts.y * self.probe_counts.z
    }

    /// 计算总体积大小
    pub fn volume_size(&self) -> glam::Vec3 {
        glam::Vec3::new(
            (self.probe_counts.x - 1) as f32 * self.probe_spacing,
            (self.probe_counts.y - 1) as f32 * self.probe_spacing,
            (self.probe_counts.z - 1) as f32 * self.probe_spacing,
        )
    }

    /// 创建低质量配置
    pub fn low_quality() -> Self {
        Self {
            probe_spacing: 4.0,
            probe_counts: UVec3::new(5, 5, 5),
            irradiance_resolution: 8,
            depth_resolution: 8,
            max_depth: 50.0,
            normal_bias: 0.1,
            update_rate: 6,
            enable_temporal_filter: true,
            temporal_filter_alpha: 0.95,
            enable_light_propagation: true,
            propagation_iterations: 1,
        }
    }

    /// 创建中等质量配置
    pub fn medium_quality() -> Self {
        Self {
            probe_spacing: 2.0,
            probe_counts: UVec3::new(10, 10, 10),
            irradiance_resolution: 16,
            depth_resolution: 16,
            max_depth: 50.0,
            normal_bias: 0.05,
            update_rate: 3,
            enable_temporal_filter: true,
            temporal_filter_alpha: 0.9,
            enable_light_propagation: true,
            propagation_iterations: 2,
        }
    }

    /// 创建高质量配置
    pub fn high_quality() -> Self {
        Self {
            probe_spacing: 1.0,
            probe_counts: UVec3::new(20, 20, 20),
            irradiance_resolution: 32,
            depth_resolution: 32,
            max_depth: 100.0,
            normal_bias: 0.02,
            update_rate: 1,
            enable_temporal_filter: true,
            temporal_filter_alpha: 0.85,
            enable_light_propagation: true,
            propagation_iterations: 4,
        }
    }

    /// 创建超高质量配置
    pub fn ultra_quality() -> Self {
        Self {
            probe_spacing: 0.5,
            probe_counts: UVec3::new(40, 40, 40),
            irradiance_resolution: 64,
            depth_resolution: 64,
            max_depth: 100.0,
            normal_bias: 0.01,
            update_rate: 1,
            enable_temporal_filter: true,
            temporal_filter_alpha: 0.8,
            enable_light_propagation: true,
            propagation_iterations: 6,
        }
    }

    /// 计算内存占用（字节）
    pub fn memory_usage(&self) -> u64 {
        let probe_count = self.total_probes() as u64;
        let face_count = 6u64;

        // 辐照度纹理：RGBA32Float = 16字节/像素
        let irradiance_memory = probe_count
            * face_count
            * (self.irradiance_resolution as u64 * self.irradiance_resolution as u64)
            * 16;

        // 深度纹理：R32Float = 4字节/像素
        let depth_memory = probe_count
            * face_count
            * (self.depth_resolution as u64 * self.depth_resolution as u64)
            * 4;

        // 偏移纹理：RG32Float = 8字节/像素
        let offset_memory = probe_count
            * face_count
            * (self.depth_resolution as u64 * self.depth_resolution as u64)
            * 8;

        irradiance_memory + depth_memory + offset_memory
    }

    /// 获取质量描述
    pub fn quality_description(&self) -> &str {
        let probe_count = self.total_probes();
        match probe_count {
            n if n <= 125 => "Low",
            n if n <= 1000 => "Medium",
            n if n <= 8000 => "High",
            _ => "Ultra",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DDGIConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_quality_presets() {
        let low = DDGIConfig::low_quality();
        let medium = DDGIConfig::medium_quality();
        let high = DDGIConfig::high_quality();

        assert!(low.probe_spacing > medium.probe_spacing);
        assert!(medium.probe_spacing > high.probe_spacing);

        assert!(low.total_probes() < medium.total_probes());
        assert!(medium.total_probes() < high.total_probes());
    }

    #[test]
    fn test_memory_usage() {
        let config = DDGIConfig::low_quality();
        let memory = config.memory_usage();
        assert!(memory > 0);
    }

    #[test]
    fn test_validation() {
        let mut config = DDGIConfig::default();

        // 无效的探针间距
        config.probe_spacing = 0.0;
        assert!(config.validate().is_err());

        // 无效的探针数量
        config.probe_spacing = 2.0;
        config.probe_counts = UVec3::ZERO;
        assert!(config.validate().is_err());

        // 无效的纹理分辨率
        config.probe_counts = UVec3::new(10, 10, 10);
        config.irradiance_resolution = 15; // 不是2的幂
        assert!(config.validate().is_err());
    }
}
