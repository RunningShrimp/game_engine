/// 硬件能力评估系统

use super::gpu_detect::{GpuInfo, GpuTier};
use super::npu_detect::NpuInfo;
use super::soc_detect::SocInfo;

/// 性能等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PerformanceTier {
    /// 低端（入门级）
    Low,
    /// 中低端
    MediumLow,
    /// 中端
    Medium,
    /// 中高端
    MediumHigh,
    /// 高端
    High,
    /// 旗舰
    Flagship,
}

/// 硬件能力
#[derive(Debug, Clone)]
pub struct HardwareCapability {
    /// 综合性能等级
    pub tier: PerformanceTier,
    
    /// GPU能力
    pub gpu_tier: GpuTier,
    pub gpu_vram_mb: u64,
    pub supports_raytracing: bool,
    pub supports_mesh_shaders: bool,
    pub supports_vrs: bool,
    
    /// NPU能力
    pub has_npu: bool,
    pub npu_tops: f32,
    
    /// 内存
    pub system_ram_mb: u64,
    
    /// 移动平台特性
    pub is_mobile: bool,
    pub thermal_limited: bool,
    
    /// 推荐设置
    pub max_resolution_scale: f32,
    pub max_shadow_quality: u32,
    pub max_texture_quality: u32,
    pub recommended_vsync: bool,
}

impl HardwareCapability {
    /// 评估硬件能力
    pub fn evaluate(gpu: &GpuInfo, npu: &Option<NpuInfo>, soc: &Option<SocInfo>) -> Self {
        let gpu_tier = gpu.tier;
        let is_mobile = soc.is_some();
        
        // 综合评估性能等级
        let tier = Self::evaluate_tier(gpu_tier, is_mobile);
        
        // 获取系统内存
        let system_ram_mb = Self::get_system_ram_mb();
        
        // NPU信息
        let has_npu = npu.is_some();
        let npu_tops = npu.as_ref().map(|n| n.tops).unwrap_or(0.0);
        
        // 移动平台通常有热限制
        let thermal_limited = is_mobile;
        
        // 根据性能等级推荐设置
        let (max_resolution_scale, max_shadow_quality, max_texture_quality, recommended_vsync) =
            Self::recommend_settings(tier, is_mobile);
        
        Self {
            tier,
            gpu_tier,
            gpu_vram_mb: gpu.vram_mb,
            supports_raytracing: gpu.supports_raytracing,
            supports_mesh_shaders: gpu.supports_mesh_shaders,
            supports_vrs: gpu.supports_variable_rate_shading,
            has_npu,
            npu_tops,
            system_ram_mb,
            is_mobile,
            thermal_limited,
            max_resolution_scale,
            max_shadow_quality,
            max_texture_quality,
            recommended_vsync,
        }
    }
    
    fn evaluate_tier(gpu_tier: GpuTier, is_mobile: bool) -> PerformanceTier {
        // 移动平台降一级
        if is_mobile {
            match gpu_tier {
                GpuTier::Flagship => PerformanceTier::High,
                GpuTier::High => PerformanceTier::MediumHigh,
                GpuTier::MediumHigh => PerformanceTier::Medium,
                GpuTier::Medium => PerformanceTier::MediumLow,
                GpuTier::MediumLow => PerformanceTier::Low,
                GpuTier::Low => PerformanceTier::Low,
            }
        } else {
            match gpu_tier {
                GpuTier::Flagship => PerformanceTier::Flagship,
                GpuTier::High => PerformanceTier::High,
                GpuTier::MediumHigh => PerformanceTier::MediumHigh,
                GpuTier::Medium => PerformanceTier::Medium,
                GpuTier::MediumLow => PerformanceTier::MediumLow,
                GpuTier::Low => PerformanceTier::Low,
            }
        }
    }
    
    fn get_system_ram_mb() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
                for line in content.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(kb_val) = kb.parse::<u64>() {
                                return kb_val / 1024;
                            }
                        }
                    }
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("sysctl")
                .arg("-n")
                .arg("hw.memsize")
                .output()
            {
                if let Ok(bytes_str) = String::from_utf8(output.stdout) {
                    if let Ok(bytes) = bytes_str.trim().parse::<u64>() {
                        return bytes / (1024 * 1024);
                    }
                }
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // Windows可以使用GlobalMemoryStatusEx
            // 这里简化处理
        }
        
        8192 // 默认8GB
    }
    
    fn recommend_settings(tier: PerformanceTier, is_mobile: bool) -> (f32, u32, u32, bool) {
        match tier {
            PerformanceTier::Flagship => (2.0, 4, 4, false),
            PerformanceTier::High => (1.5, 3, 4, false),
            PerformanceTier::MediumHigh => (1.0, 2, 3, !is_mobile),
            PerformanceTier::Medium => (1.0, 2, 2, true),
            PerformanceTier::MediumLow => (0.75, 1, 2, true),
            PerformanceTier::Low => (0.5, 0, 1, true),
        }
    }
    
    /// 是否支持高级特性
    pub fn supports_advanced_features(&self) -> bool {
        self.tier >= PerformanceTier::MediumHigh
    }
    
    /// 是否支持光线追踪
    pub fn can_use_raytracing(&self) -> bool {
        self.supports_raytracing && self.tier >= PerformanceTier::High
    }
    
    /// 是否应该使用NPU加速
    pub fn should_use_npu(&self) -> bool {
        self.has_npu && self.npu_tops > 5.0
    }
    
    /// 推荐的并行任务数
    pub fn recommended_parallel_tasks(&self) -> usize {
        match self.tier {
            PerformanceTier::Flagship => 16,
            PerformanceTier::High => 12,
            PerformanceTier::MediumHigh => 8,
            PerformanceTier::Medium => 6,
            PerformanceTier::MediumLow => 4,
            PerformanceTier::Low => 2,
        }
    }
    
    /// 推荐的批处理大小
    pub fn recommended_batch_size(&self) -> usize {
        match self.tier {
            PerformanceTier::Flagship => 2048,
            PerformanceTier::High => 1024,
            PerformanceTier::MediumHigh => 512,
            PerformanceTier::Medium => 256,
            PerformanceTier::MediumLow => 128,
            PerformanceTier::Low => 64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::hardware::{detect_gpu, detect_npu, detect_soc};

    #[test]
    fn test_capability_evaluation() {
        let gpu = detect_gpu();
        let npu = detect_npu();
        let soc = detect_soc();
        
        let capability = HardwareCapability::evaluate(&gpu, &npu, &soc);
        
        println!("Hardware Capability: {:#?}", capability);
        
        assert!(capability.system_ram_mb > 0);
        assert!(capability.gpu_vram_mb > 0);
    }

    #[test]
    fn test_recommendations() {
        let gpu = detect_gpu();
        let npu = detect_npu();
        let soc = detect_soc();
        
        let capability = HardwareCapability::evaluate(&gpu, &npu, &soc);
        
        println!("Performance Tier: {:?}", capability.tier);
        println!("Max Resolution Scale: {}", capability.max_resolution_scale);
        println!("Max Shadow Quality: {}", capability.max_shadow_quality);
        println!("Recommended Parallel Tasks: {}", capability.recommended_parallel_tasks());
        println!("Recommended Batch Size: {}", capability.recommended_batch_size());
    }
}
