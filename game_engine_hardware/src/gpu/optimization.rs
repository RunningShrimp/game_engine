/// GPU特定渲染优化策略
/// 
/// 针对不同GPU厂商和型号的特定优化

use super::detect::{GpuInfo, GpuVendor, GpuTier};
use serde::{Serialize, Deserialize};

/// GPU优化策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuOptimization {
    pub vendor: GpuVendor,
    pub tier: GpuTier,
    
    // 渲染管线优化
    pub preferred_pipeline_mode: PipelineMode,
    pub use_async_compute: bool,
    pub use_bindless_textures: bool,
    
    // 内存优化
    pub texture_streaming_enabled: bool,
    pub texture_compression_format: TextureCompressionFormat,
    pub buffer_pooling_enabled: bool,
    
    // 批处理优化
    pub max_draw_calls_per_frame: u32,
    pub instancing_threshold: u32,
    pub use_indirect_drawing: bool,
    
    // 特效优化
    pub particle_budget: u32,
    pub max_lights_per_frame: u32,
    pub shadow_cascade_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineMode {
    /// 前向渲染（移动端优先）
    Forward,
    /// 延迟渲染（桌面端优先）
    Deferred,
    /// 前向+延迟混合
    ForwardPlus,
    /// Tile-based延迟渲染（移动端优化）
    TiledDeferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureCompressionFormat {
    /// 无压缩
    None,
    /// BC压缩（桌面端）
    BC,
    /// ASTC压缩（移动端）
    ASTC,
    /// ETC2压缩（移动端）
    ETC2,
}

impl GpuOptimization {
    /// 为特定GPU生成优化策略
    pub fn for_gpu(gpu: &GpuInfo) -> Self {
        match gpu.vendor {
            GpuVendor::Nvidia => Self::nvidia_optimization(gpu),
            GpuVendor::Amd => Self::amd_optimization(gpu),
            GpuVendor::Intel => Self::intel_optimization(gpu),
            GpuVendor::Apple => Self::apple_optimization(gpu),
            GpuVendor::Qualcomm => Self::qualcomm_optimization(gpu),
            GpuVendor::Mali => Self::mali_optimization(gpu),
            _ => Self::generic_optimization(gpu),
        }
    }
    
    fn nvidia_optimization(gpu: &GpuInfo) -> Self {
        let is_high_end = gpu.tier >= GpuTier::High;
        
        Self {
            vendor: GpuVendor::Nvidia,
            tier: gpu.tier,
            preferred_pipeline_mode: if is_high_end {
                PipelineMode::Deferred
            } else {
                PipelineMode::ForwardPlus
            },
            use_async_compute: is_high_end,
            use_bindless_textures: true, // NVIDIA对bindless支持很好
            texture_streaming_enabled: true,
            texture_compression_format: TextureCompressionFormat::BC,
            buffer_pooling_enabled: true,
            max_draw_calls_per_frame: if is_high_end { 10000 } else { 5000 },
            instancing_threshold: 10,
            use_indirect_drawing: is_high_end,
            particle_budget: match gpu.tier {
                GpuTier::Flagship => 100000,
                GpuTier::High => 50000,
                GpuTier::MediumHigh => 25000,
                _ => 10000,
            },
            max_lights_per_frame: if is_high_end { 1024 } else { 256 },
            shadow_cascade_count: match gpu.tier {
                GpuTier::Flagship | GpuTier::High => 4,
                GpuTier::MediumHigh => 3,
                _ => 2,
            },
        }
    }
    
    fn amd_optimization(gpu: &GpuInfo) -> Self {
        let is_high_end = gpu.tier >= GpuTier::High;
        
        Self {
            vendor: GpuVendor::Amd,
            tier: gpu.tier,
            preferred_pipeline_mode: if is_high_end {
                PipelineMode::Deferred
            } else {
                PipelineMode::ForwardPlus
            },
            use_async_compute: true, // AMD的异步计算很强
            use_bindless_textures: is_high_end,
            texture_streaming_enabled: true,
            texture_compression_format: TextureCompressionFormat::BC,
            buffer_pooling_enabled: true,
            max_draw_calls_per_frame: if is_high_end { 8000 } else { 4000 },
            instancing_threshold: 15,
            use_indirect_drawing: is_high_end,
            particle_budget: match gpu.tier {
                GpuTier::Flagship => 80000,
                GpuTier::High => 40000,
                GpuTier::MediumHigh => 20000,
                _ => 8000,
            },
            max_lights_per_frame: if is_high_end { 768 } else { 192 },
            shadow_cascade_count: match gpu.tier {
                GpuTier::Flagship | GpuTier::High => 4,
                GpuTier::MediumHigh => 3,
                _ => 2,
            },
        }
    }
    
    fn intel_optimization(gpu: &GpuInfo) -> Self {
        let is_arc = gpu.name.to_lowercase().contains("arc");
        let is_high_end = gpu.tier >= GpuTier::MediumHigh;
        
        Self {
            vendor: GpuVendor::Intel,
            tier: gpu.tier,
            preferred_pipeline_mode: if is_arc && is_high_end {
                PipelineMode::Deferred
            } else {
                PipelineMode::Forward
            },
            use_async_compute: is_arc,
            use_bindless_textures: is_arc,
            texture_streaming_enabled: true,
            texture_compression_format: TextureCompressionFormat::BC,
            buffer_pooling_enabled: true,
            max_draw_calls_per_frame: if is_arc { 5000 } else { 2000 },
            instancing_threshold: 20,
            use_indirect_drawing: is_arc && is_high_end,
            particle_budget: match gpu.tier {
                GpuTier::MediumHigh => 15000,
                GpuTier::Medium => 8000,
                _ => 3000,
            },
            max_lights_per_frame: if is_arc { 256 } else { 64 },
            shadow_cascade_count: if is_high_end { 3 } else { 2 },
        }
    }
    
    fn apple_optimization(gpu: &GpuInfo) -> Self {
        let is_high_end = gpu.tier >= GpuTier::MediumHigh;
        
        Self {
            vendor: GpuVendor::Apple,
            tier: gpu.tier,
            // Apple GPU使用tile-based架构
            preferred_pipeline_mode: PipelineMode::TiledDeferred,
            use_async_compute: true,
            use_bindless_textures: true, // Metal支持argument buffers
            texture_streaming_enabled: true,
            texture_compression_format: TextureCompressionFormat::ASTC,
            buffer_pooling_enabled: true,
            max_draw_calls_per_frame: if is_high_end { 8000 } else { 4000 },
            instancing_threshold: 10,
            use_indirect_drawing: is_high_end,
            particle_budget: match gpu.tier {
                GpuTier::High => 50000,
                GpuTier::MediumHigh => 30000,
                GpuTier::Medium => 15000,
                _ => 5000,
            },
            max_lights_per_frame: if is_high_end { 512 } else { 128 },
            shadow_cascade_count: if is_high_end { 4 } else { 3 },
        }
    }
    
    fn qualcomm_optimization(gpu: &GpuInfo) -> Self {
        Self {
            vendor: GpuVendor::Qualcomm,
            tier: gpu.tier,
            // 移动端优先使用前向渲染
            preferred_pipeline_mode: PipelineMode::Forward,
            use_async_compute: false,
            use_bindless_textures: false,
            texture_streaming_enabled: true,
            texture_compression_format: TextureCompressionFormat::ASTC,
            buffer_pooling_enabled: true,
            max_draw_calls_per_frame: match gpu.tier {
                GpuTier::MediumHigh => 2000,
                GpuTier::Medium => 1000,
                _ => 500,
            },
            instancing_threshold: 30,
            use_indirect_drawing: false,
            particle_budget: match gpu.tier {
                GpuTier::MediumHigh => 10000,
                GpuTier::Medium => 5000,
                _ => 2000,
            },
            max_lights_per_frame: match gpu.tier {
                GpuTier::MediumHigh => 32,
                _ => 16,
            },
            shadow_cascade_count: 2,
        }
    }
    
    fn mali_optimization(gpu: &GpuInfo) -> Self {
        Self {
            vendor: GpuVendor::Mali,
            tier: gpu.tier,
            // Mali也是tile-based
            preferred_pipeline_mode: PipelineMode::Forward,
            use_async_compute: false,
            use_bindless_textures: false,
            texture_streaming_enabled: true,
            texture_compression_format: TextureCompressionFormat::ASTC,
            buffer_pooling_enabled: true,
            max_draw_calls_per_frame: match gpu.tier {
                GpuTier::Medium => 1500,
                _ => 800,
            },
            instancing_threshold: 25,
            use_indirect_drawing: false,
            particle_budget: match gpu.tier {
                GpuTier::Medium => 8000,
                _ => 3000,
            },
            max_lights_per_frame: 16,
            shadow_cascade_count: 2,
        }
    }
    
    fn generic_optimization(gpu: &GpuInfo) -> Self {
        Self {
            vendor: GpuVendor::Unknown,
            tier: gpu.tier,
            preferred_pipeline_mode: PipelineMode::Forward,
            use_async_compute: false,
            use_bindless_textures: false,
            texture_streaming_enabled: true,
            texture_compression_format: TextureCompressionFormat::BC,
            buffer_pooling_enabled: true,
            max_draw_calls_per_frame: 2000,
            instancing_threshold: 20,
            use_indirect_drawing: false,
            particle_budget: 5000,
            max_lights_per_frame: 64,
            shadow_cascade_count: 2,
        }
    }
    
    /// 获取推荐的纹理最大尺寸
    pub fn max_texture_size(&self) -> u32 {
        match self.tier {
            GpuTier::Flagship | GpuTier::High => 8192,
            GpuTier::MediumHigh => 4096,
            GpuTier::Medium => 2048,
            _ => 1024,
        }
    }
    
    /// 获取推荐的MSAA采样数
    pub fn recommended_msaa_samples(&self) -> u32 {
        match self.tier {
            GpuTier::Flagship => 8,
            GpuTier::High => 4,
            GpuTier::MediumHigh => 2,
            _ => 1,
        }
    }
    
    /// 是否应该使用计算着色器
    pub fn should_use_compute_shaders(&self) -> bool {
        self.tier >= GpuTier::Medium && 
        !matches!(self.vendor, GpuVendor::Qualcomm | GpuVendor::Mali)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detect_gpu;

    #[test]
    fn test_gpu_optimization() {
        let gpu = detect_gpu();
        let optimization = GpuOptimization::for_gpu(&gpu);
        
        println!("GPU Optimization: {:#?}", optimization);
        
        assert!(optimization.max_draw_calls_per_frame > 0);
        assert!(optimization.particle_budget > 0);
    }

    #[test]
    fn test_vendor_specific() {
        let gpu_info = GpuInfo {
            vendor: GpuVendor::Nvidia,
            name: "RTX 4090".to_string(),
            tier: GpuTier::Flagship,
            ..Default::default()
        };
        
        let opt = GpuOptimization::for_gpu(&gpu_info);
        assert_eq!(opt.preferred_pipeline_mode, PipelineMode::Deferred);
        assert!(opt.use_async_compute);
    }
}
