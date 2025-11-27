/// 自动配置系统
/// 
/// 根据硬件能力自动生成最优配置

use super::capability::{HardwareCapability, PerformanceTier};
use serde::{Serialize, Deserialize};

/// 质量预设
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityPreset {
    Low,
    Medium,
    High,
    Ultra,
    Custom,
}

/// 自动配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoConfig {
    pub quality_preset: QualityPreset,
    
    // 渲染设置
    pub resolution_scale: f32,
    pub target_fps: u32,
    pub vsync_enabled: bool,
    
    // 图形质量
    pub shadow_quality: ShadowQuality,
    pub texture_quality: TextureQuality,
    pub anti_aliasing: AntiAliasingMode,
    pub ambient_occlusion: bool,
    pub bloom: bool,
    pub motion_blur: bool,
    pub depth_of_field: bool,
    
    // 高级特性
    pub raytracing_enabled: bool,
    pub dlss_enabled: bool,
    pub fsr_enabled: bool,
    pub mesh_shaders_enabled: bool,
    pub vrs_enabled: bool,
    
    // 性能优化
    pub use_npu_acceleration: bool,
    pub parallel_task_count: usize,
    pub batch_size: usize,
    pub culling_distance: f32,
    pub lod_bias: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowQuality {
    Off,
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AntiAliasingMode {
    Off,
    FXAA,
    TAA,
    MSAA2x,
    MSAA4x,
    MSAA8x,
}

impl AutoConfig {
    /// 从硬件能力生成配置
    pub fn from_capability(capability: &HardwareCapability) -> Self {
        match capability.tier {
            PerformanceTier::Flagship => Self::ultra_preset(capability),
            PerformanceTier::High => Self::high_preset(capability),
            PerformanceTier::MediumHigh => Self::medium_high_preset(capability),
            PerformanceTier::Medium => Self::medium_preset(capability),
            PerformanceTier::MediumLow => Self::low_medium_preset(capability),
            PerformanceTier::Low => Self::low_preset(capability),
        }
    }
    
    fn ultra_preset(capability: &HardwareCapability) -> Self {
        Self {
            quality_preset: QualityPreset::Ultra,
            resolution_scale: 2.0,
            target_fps: 60,
            vsync_enabled: false,
            shadow_quality: ShadowQuality::Ultra,
            texture_quality: TextureQuality::Ultra,
            anti_aliasing: AntiAliasingMode::TAA,
            ambient_occlusion: true,
            bloom: true,
            motion_blur: true,
            depth_of_field: true,
            raytracing_enabled: capability.supports_raytracing,
            dlss_enabled: capability.supports_raytracing,
            fsr_enabled: false,
            mesh_shaders_enabled: capability.supports_mesh_shaders,
            vrs_enabled: capability.supports_vrs,
            use_npu_acceleration: capability.should_use_npu(),
            parallel_task_count: capability.recommended_parallel_tasks(),
            batch_size: capability.recommended_batch_size(),
            culling_distance: 1000.0,
            lod_bias: 0.0,
        }
    }
    
    fn high_preset(capability: &HardwareCapability) -> Self {
        Self {
            quality_preset: QualityPreset::High,
            resolution_scale: 1.5,
            target_fps: 60,
            vsync_enabled: false,
            shadow_quality: ShadowQuality::High,
            texture_quality: TextureQuality::High,
            anti_aliasing: AntiAliasingMode::TAA,
            ambient_occlusion: true,
            bloom: true,
            motion_blur: false,
            depth_of_field: false,
            raytracing_enabled: capability.supports_raytracing && !capability.is_mobile,
            dlss_enabled: capability.supports_raytracing,
            fsr_enabled: !capability.supports_raytracing,
            mesh_shaders_enabled: capability.supports_mesh_shaders,
            vrs_enabled: capability.supports_vrs,
            use_npu_acceleration: capability.should_use_npu(),
            parallel_task_count: capability.recommended_parallel_tasks(),
            batch_size: capability.recommended_batch_size(),
            culling_distance: 800.0,
            lod_bias: 0.0,
        }
    }
    
    fn medium_high_preset(capability: &HardwareCapability) -> Self {
        Self {
            quality_preset: QualityPreset::High,
            resolution_scale: 1.0,
            target_fps: 60,
            vsync_enabled: !capability.is_mobile,
            shadow_quality: ShadowQuality::Medium,
            texture_quality: TextureQuality::High,
            anti_aliasing: AntiAliasingMode::FXAA,
            ambient_occlusion: true,
            bloom: true,
            motion_blur: false,
            depth_of_field: false,
            raytracing_enabled: false,
            dlss_enabled: false,
            fsr_enabled: true,
            mesh_shaders_enabled: false,
            vrs_enabled: capability.supports_vrs,
            use_npu_acceleration: capability.should_use_npu(),
            parallel_task_count: capability.recommended_parallel_tasks(),
            batch_size: capability.recommended_batch_size(),
            culling_distance: 600.0,
            lod_bias: 0.5,
        }
    }
    
    fn medium_preset(capability: &HardwareCapability) -> Self {
        Self {
            quality_preset: QualityPreset::Medium,
            resolution_scale: 1.0,
            target_fps: 60,
            vsync_enabled: true,
            shadow_quality: ShadowQuality::Medium,
            texture_quality: TextureQuality::Medium,
            anti_aliasing: AntiAliasingMode::FXAA,
            ambient_occlusion: false,
            bloom: false,
            motion_blur: false,
            depth_of_field: false,
            raytracing_enabled: false,
            dlss_enabled: false,
            fsr_enabled: capability.is_mobile,
            mesh_shaders_enabled: false,
            vrs_enabled: false,
            use_npu_acceleration: capability.should_use_npu(),
            parallel_task_count: capability.recommended_parallel_tasks(),
            batch_size: capability.recommended_batch_size(),
            culling_distance: 400.0,
            lod_bias: 1.0,
        }
    }
    
    fn low_medium_preset(capability: &HardwareCapability) -> Self {
        Self {
            quality_preset: QualityPreset::Medium,
            resolution_scale: 0.75,
            target_fps: 30,
            vsync_enabled: true,
            shadow_quality: ShadowQuality::Low,
            texture_quality: TextureQuality::Medium,
            anti_aliasing: AntiAliasingMode::Off,
            ambient_occlusion: false,
            bloom: false,
            motion_blur: false,
            depth_of_field: false,
            raytracing_enabled: false,
            dlss_enabled: false,
            fsr_enabled: true,
            mesh_shaders_enabled: false,
            vrs_enabled: false,
            use_npu_acceleration: false,
            parallel_task_count: capability.recommended_parallel_tasks(),
            batch_size: capability.recommended_batch_size(),
            culling_distance: 300.0,
            lod_bias: 1.5,
        }
    }
    
    fn low_preset(capability: &HardwareCapability) -> Self {
        Self {
            quality_preset: QualityPreset::Low,
            resolution_scale: 0.5,
            target_fps: 30,
            vsync_enabled: true,
            shadow_quality: ShadowQuality::Off,
            texture_quality: TextureQuality::Low,
            anti_aliasing: AntiAliasingMode::Off,
            ambient_occlusion: false,
            bloom: false,
            motion_blur: false,
            depth_of_field: false,
            raytracing_enabled: false,
            dlss_enabled: false,
            fsr_enabled: true,
            mesh_shaders_enabled: false,
            vrs_enabled: false,
            use_npu_acceleration: false,
            parallel_task_count: capability.recommended_parallel_tasks(),
            batch_size: capability.recommended_batch_size(),
            culling_distance: 200.0,
            lod_bias: 2.0,
        }
    }
    
    /// 保存配置到文件
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// 从文件加载配置
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&json)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::hardware::{detect_gpu, detect_npu, detect_soc, HardwareCapability};

    #[test]
    fn test_auto_config() {
        let gpu = detect_gpu();
        let npu = detect_npu();
        let soc = detect_soc();
        
        let capability = HardwareCapability::evaluate(&gpu, &npu, &soc);
        let config = AutoConfig::from_capability(&capability);
        
        println!("Auto Config: {:#?}", config);
        
        assert!(config.resolution_scale > 0.0);
        assert!(config.target_fps > 0);
    }

    #[test]
    fn test_save_load_config() {
        let gpu = detect_gpu();
        let npu = detect_npu();
        let soc = detect_soc();
        
        let capability = HardwareCapability::evaluate(&gpu, &npu, &soc);
        let config = AutoConfig::from_capability(&capability);
        
        let path = "/tmp/test_config.json";
        config.save_to_file(path).unwrap();
        
        let loaded = AutoConfig::load_from_file(path).unwrap();
        assert_eq!(config.quality_preset, loaded.quality_preset);
        
        std::fs::remove_file(path).ok();
    }
}
