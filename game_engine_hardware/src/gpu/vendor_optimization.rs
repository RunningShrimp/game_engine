//! GPU厂商特定优化模块
//!
//! 为不同GPU厂商（NVIDIA、AMD、Apple）提供特定的优化路径

use super::detect::{GpuInfo, GpuVendor};
// Note: WgpuRenderer dependency removed - this function should be implemented in the main crate

/// GPU优化配置
#[derive(Debug, Clone)]
pub struct GpuOptimizationConfig {
    /// 批次大小优化
    pub optimal_batch_size: u32,
    /// 是否启用异步计算
    pub enable_async_compute: bool,
    /// 是否启用GPU驱动剔除
    pub enable_gpu_culling: bool,
    /// 是否启用网格着色器
    pub enable_mesh_shaders: bool,
    /// 是否启用可变速率着色
    pub enable_variable_rate_shading: bool,
    /// 是否启用光线追踪
    pub enable_raytracing: bool,
    /// 最大纹理尺寸
    pub max_texture_size: u32,
    /// 推荐的后处理质量
    pub postprocess_quality: u32,
    /// 推荐的阴影质量
    pub shadow_quality: u32,
    /// 推荐的LOD偏移
    pub lod_bias: f32,
}

impl Default for GpuOptimizationConfig {
    fn default() -> Self {
        Self {
            optimal_batch_size: 1000,
            enable_async_compute: false,
            enable_gpu_culling: true,
            enable_mesh_shaders: false,
            enable_variable_rate_shading: false,
            enable_raytracing: false,
            max_texture_size: 4096,
            postprocess_quality: 2,
            shadow_quality: 2,
            lod_bias: 0.0,
        }
    }
}

/// GPU优化器
pub struct GpuOptimizer {
    gpu_info: GpuInfo,
    config: GpuOptimizationConfig,
}

impl GpuOptimizer {
    /// 创建GPU优化器
    pub fn new(gpu_info: GpuInfo) -> Self {
        let config = Self::generate_optimization_config(&gpu_info);
        
        Self {
            gpu_info,
            config,
        }
    }

    /// 根据GPU信息生成优化配置
    fn generate_optimization_config(gpu_info: &GpuInfo) -> GpuOptimizationConfig {
        match gpu_info.vendor {
            GpuVendor::Nvidia => Self::nvidia_optimization(gpu_info),
            GpuVendor::Amd => Self::amd_optimization(gpu_info),
            GpuVendor::Apple => Self::apple_optimization(gpu_info),
            GpuVendor::Intel => Self::intel_optimization(gpu_info),
            _ => GpuOptimizationConfig::default(),
        }
    }

    /// NVIDIA特定优化
    fn nvidia_optimization(gpu_info: &GpuInfo) -> GpuOptimizationConfig {
        let mut config = GpuOptimizationConfig::default();
        
        // NVIDIA GPU通常有更好的批次处理能力
        config.optimal_batch_size = match gpu_info.tier {
            crate::gpu::detect::GpuTier::Flagship => 5000,
            crate::gpu::detect::GpuTier::High => 3000,
            crate::gpu::detect::GpuTier::MediumHigh => 2000,
            _ => 1000,
        };
        
        // RTX系列支持光线追踪和网格着色器
        if gpu_info.name.to_lowercase().contains("rtx") {
            config.enable_raytracing = gpu_info.supports_raytracing;
            config.enable_mesh_shaders = gpu_info.supports_mesh_shaders;
            config.enable_async_compute = true;
        }
        
        // NVIDIA GPU通常有较大的显存，可以使用更高质量的纹理
        config.max_texture_size = match gpu_info.vram_mb {
            vram if vram >= 16384 => 8192,
            vram if vram >= 8192 => 4096,
            _ => 2048,
        };
        
        // 根据性能等级调整质量
        config.postprocess_quality = match gpu_info.tier {
            crate::gpu::detect::GpuTier::Flagship => 4,
            crate::gpu::detect::GpuTier::High => 3,
            crate::gpu::detect::GpuTier::MediumHigh => 2,
            _ => 1,
        };
        
        config.shadow_quality = match gpu_info.tier {
            crate::gpu::detect::GpuTier::Flagship => 4,
            crate::gpu::detect::GpuTier::High => 3,
            crate::gpu::detect::GpuTier::MediumHigh => 2,
            _ => 1,
        };
        
        // NVIDIA GPU通常有更好的几何处理能力，可以使用更激进的LOD
        config.lod_bias = -0.5;
        
        tracing::info!(target: "gpu_optimization", "Applied NVIDIA-specific optimizations for {}", gpu_info.name);
        
        config
    }

    /// AMD特定优化
    fn amd_optimization(gpu_info: &GpuInfo) -> GpuOptimizationConfig {
        let mut config = GpuOptimizationConfig::default();
        
        // AMD GPU通常有更多的计算单元，适合并行计算
        config.optimal_batch_size = match gpu_info.tier {
            crate::gpu::detect::GpuTier::Flagship => 4000,
            crate::gpu::detect::GpuTier::High => 2500,
            crate::gpu::detect::GpuTier::MediumHigh => 1500,
            _ => 1000,
        };
        
        // RDNA架构支持异步计算
        if gpu_info.name.to_lowercase().contains("rx 6") || 
           gpu_info.name.to_lowercase().contains("rx 7") {
            config.enable_async_compute = true;
            config.enable_raytracing = gpu_info.supports_raytracing;
        }
        
        // AMD GPU通常有较大的显存带宽
        config.max_texture_size = match gpu_info.vram_mb {
            vram if vram >= 16384 => 8192,
            vram if vram >= 8192 => 4096,
            _ => 2048,
        };
        
        // AMD GPU在计算着色器方面表现优秀
        config.enable_gpu_culling = true;
        
        // 根据性能等级调整质量
        config.postprocess_quality = match gpu_info.tier {
            crate::gpu::detect::GpuTier::Flagship => 4,
            crate::gpu::detect::GpuTier::High => 3,
            crate::gpu::detect::GpuTier::MediumHigh => 2,
            _ => 1,
        };
        
        config.shadow_quality = match gpu_info.tier {
            crate::gpu::detect::GpuTier::Flagship => 4,
            crate::gpu::detect::GpuTier::High => 3,
            crate::gpu::detect::GpuTier::MediumHigh => 2,
            _ => 1,
        };
        
        // AMD GPU在几何处理方面略逊于NVIDIA，使用保守的LOD
        config.lod_bias = 0.0;
        
        tracing::info!(target: "gpu_optimization", "Applied AMD-specific optimizations for {}", gpu_info.name);
        
        config
    }

    /// Apple Silicon特定优化
    fn apple_optimization(gpu_info: &GpuInfo) -> GpuOptimizationConfig {
        let mut config = GpuOptimizationConfig::default();
        
        // Apple Silicon使用统一内存架构，批次大小可以更大
        config.optimal_batch_size = match gpu_info.tier {
            crate::gpu::detect::GpuTier::High => 4000,
            crate::gpu::detect::GpuTier::MediumHigh => 2500,
            crate::gpu::detect::GpuTier::Medium => 1500,
            _ => 1000,
        };
        
        // Apple Silicon支持Metal的tile-based延迟渲染
        config.enable_gpu_culling = true;
        
        // M系列芯片支持光线追踪和网格着色器
        if gpu_info.name.to_lowercase().contains("m1") ||
           gpu_info.name.to_lowercase().contains("m2") ||
           gpu_info.name.to_lowercase().contains("m3") {
            config.enable_raytracing = gpu_info.supports_raytracing;
            config.enable_mesh_shaders = gpu_info.supports_mesh_shaders;
        }
        
        // Apple Silicon使用统一内存，纹理大小受系统内存限制
        config.max_texture_size = match gpu_info.tier {
            crate::gpu::detect::GpuTier::High => 8192,
            crate::gpu::detect::GpuTier::MediumHigh => 4096,
            _ => 2048,
        };
        
        // Apple Silicon在功耗优化方面优秀，可以使用更高质量
        config.postprocess_quality = match gpu_info.tier {
            crate::gpu::detect::GpuTier::High => 4,
            crate::gpu::detect::GpuTier::MediumHigh => 3,
            crate::gpu::detect::GpuTier::Medium => 2,
            _ => 1,
        };
        
        config.shadow_quality = match gpu_info.tier {
            crate::gpu::detect::GpuTier::High => 4,
            crate::gpu::detect::GpuTier::MediumHigh => 3,
            crate::gpu::detect::GpuTier::Medium => 2,
            _ => 1,
        };
        
        // Apple Silicon在几何处理方面表现优秀
        config.lod_bias = -0.3;
        
        tracing::info!(target: "gpu_optimization", "Applied Apple-specific optimizations for {}", gpu_info.name);
        
        config
    }

    /// Intel特定优化
    fn intel_optimization(gpu_info: &GpuInfo) -> GpuOptimizationConfig {
        let mut config = GpuOptimizationConfig::default();
        
        // Intel GPU通常是集成显卡，性能有限
        config.optimal_batch_size = match gpu_info.tier {
            crate::gpu::detect::GpuTier::MediumHigh => 1500,
            crate::gpu::detect::GpuTier::Medium => 1000,
            _ => 500,
        };
        
        // Intel Arc支持光线追踪
        if gpu_info.name.to_lowercase().contains("arc") {
            config.enable_raytracing = gpu_info.supports_raytracing;
        }
        
        // Intel GPU显存有限
        config.max_texture_size = match gpu_info.vram_mb {
            vram if vram >= 8192 => 4096,
            _ => 2048,
        };
        
        // 集成显卡使用较低质量设置
        config.postprocess_quality = match gpu_info.tier {
            crate::gpu::detect::GpuTier::MediumHigh => 2,
            _ => 1,
        };
        
        config.shadow_quality = match gpu_info.tier {
            crate::gpu::detect::GpuTier::MediumHigh => 2,
            _ => 1,
        };
        
        // Intel GPU使用保守的LOD设置
        config.lod_bias = 0.5;
        
        tracing::info!(target: "gpu_optimization", "Applied Intel-specific optimizations for {}", gpu_info.name);
        
        config
    }

    /// 获取优化配置
    pub fn config(&self) -> &GpuOptimizationConfig {
        &self.config
    }

    /// 获取GPU信息
    pub fn gpu_info(&self) -> &GpuInfo {
        &self.gpu_info
    }

    /// 应用优化配置到渲染器
    /// 应用到渲染器（需要在主crate中实现）
    /// 此方法已移除，因为WgpuRenderer属于主crate
    #[allow(dead_code)]
    pub fn apply_to_renderer_placeholder(&self) {
        // 这里可以添加将配置应用到渲染器的逻辑
        // 例如：设置批次大小、启用/禁用特定功能等
        tracing::info!(target: "gpu_optimization", 
            "Applying GPU optimizations: batch_size={}, async_compute={}, gpu_culling={}",
            self.config.optimal_batch_size,
            self.config.enable_async_compute,
            self.config.enable_gpu_culling
        );
    }

    /// 获取优化建议
    pub fn get_optimization_tips(&self) -> Vec<String> {
        let mut tips = Vec::new();
        
        tips.push(format!("GPU: {} ({:?})", self.gpu_info.name, self.gpu_info.vendor));
        tips.push(format!("推荐批次大小: {}", self.config.optimal_batch_size));
        
        if self.config.enable_async_compute {
            tips.push("✓ 启用异步计算以提升性能".to_string());
        }
        
        if self.config.enable_gpu_culling {
            tips.push("✓ 启用GPU驱动剔除以提升性能".to_string());
        }
        
        if self.config.enable_raytracing {
            tips.push("✓ GPU支持光线追踪，可启用RT功能".to_string());
        }
        
        if self.config.enable_mesh_shaders {
            tips.push("✓ GPU支持网格着色器，可启用Mesh Shader功能".to_string());
        }
        
        tips.push(format!("最大纹理尺寸: {}x{}", 
            self.config.max_texture_size, 
            self.config.max_texture_size));
        tips.push(format!("后处理质量: {}", self.config.postprocess_quality));
        tips.push(format!("阴影质量: {}", self.config.shadow_quality));
        tips.push(format!("LOD偏移: {:.1}", self.config.lod_bias));
        
        tips
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::detect::{GpuTier, detect_gpu};

    #[test]
    fn test_nvidia_optimization() {
        let gpu_info = GpuInfo {
            vendor: GpuVendor::Nvidia,
            name: "NVIDIA GeForce RTX 4090".to_string(),
            tier: GpuTier::Flagship,
            vram_mb: 24576,
            driver_version: "550.54.15".to_string(),
            supports_raytracing: true,
            supports_mesh_shaders: true,
            supports_variable_rate_shading: true,
            compute_units: 0,
        };
        
        let optimizer = GpuOptimizer::new(gpu_info);
        let config = optimizer.config();
        
        assert_eq!(config.optimal_batch_size, 5000);
        assert!(config.enable_raytracing);
        assert!(config.enable_mesh_shaders);
        assert_eq!(config.max_texture_size, 8192);
    }

    #[test]
    fn test_amd_optimization() {
        let gpu_info = GpuInfo {
            vendor: GpuVendor::Amd,
            name: "AMD Radeon RX 7900 XTX".to_string(),
            tier: GpuTier::Flagship,
            vram_mb: 24576,
            driver_version: "23.12.1".to_string(),
            supports_raytracing: true,
            supports_mesh_shaders: false,
            supports_variable_rate_shading: false,
            compute_units: 0,
        };
        
        let optimizer = GpuOptimizer::new(gpu_info);
        let config = optimizer.config();
        
        assert_eq!(config.optimal_batch_size, 4000);
        assert!(config.enable_async_compute);
        assert!(config.enable_gpu_culling);
    }

    #[test]
    fn test_apple_optimization() {
        let gpu_info = GpuInfo {
            vendor: GpuVendor::Apple,
            name: "Apple M3 Max".to_string(),
            tier: GpuTier::High,
            vram_mb: 32768,
            driver_version: "Metal".to_string(),
            supports_raytracing: true,
            supports_mesh_shaders: true,
            supports_variable_rate_shading: false,
            compute_units: 0,
        };
        
        let optimizer = GpuOptimizer::new(gpu_info);
        let config = optimizer.config();
        
        assert_eq!(config.optimal_batch_size, 4000);
        assert!(config.enable_gpu_culling);
        assert!(config.enable_raytracing);
    }

    #[test]
    fn test_optimization_tips() {
        let gpu = detect_gpu();
        let optimizer = GpuOptimizer::new(gpu);
        let tips = optimizer.get_optimization_tips();
        
        assert!(!tips.is_empty());
        tracing::info!(target: "gpu_optimization", "Optimization tips: {:?}", tips);
    }
}

