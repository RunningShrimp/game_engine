/// 硬件检测和能力评估系统
/// 
/// 自动检测GPU、NPU、SoC等硬件特性，并提供优化建议

pub mod gpu_detect;
pub mod npu_detect;
pub mod soc_detect;
pub mod capability;
pub mod auto_config;
pub mod gpu_optimization;
pub mod npu_acceleration;
pub mod soc_power;
pub mod upscaling;
pub mod adaptive_performance;

pub use gpu_detect::{GpuInfo, GpuVendor, GpuTier, detect_gpu};
pub use npu_detect::{NpuInfo, NpuVendor, detect_npu};
pub use soc_detect::{SocInfo, SocVendor, detect_soc};
pub use capability::{HardwareCapability, PerformanceTier};
pub use auto_config::{AutoConfig, QualityPreset};
pub use gpu_optimization::{GpuOptimization, PipelineMode};
pub use npu_acceleration::{NpuAccelerator, PhysicsPrediction, BehaviorDecision};
pub use soc_power::{PowerManager, PowerMode, ThermalState};
pub use upscaling::{UpscalingManager, UpscalingTech, UpscalingQuality};
pub use adaptive_performance::{AdaptivePerformance, AdaptiveStats};

use std::sync::OnceLock;

/// 全局硬件信息
static HARDWARE_INFO: OnceLock<HardwareInfo> = OnceLock::new();

/// 完整的硬件信息
#[derive(Debug, Clone)]
pub struct HardwareInfo {
    pub gpu: GpuInfo,
    pub npu: Option<NpuInfo>,
    pub soc: Option<SocInfo>,
    pub capability: HardwareCapability,
    pub recommended_config: AutoConfig,
}

impl HardwareInfo {
    /// 检测硬件信息
    pub fn detect() -> Self {
        let gpu = detect_gpu();
        let npu = detect_npu();
        let soc = detect_soc();
        
        let capability = HardwareCapability::evaluate(&gpu, &npu, &soc);
        let recommended_config = AutoConfig::from_capability(&capability);
        
        Self {
            gpu,
            npu,
            soc,
            capability,
            recommended_config,
        }
    }
    
    /// 打印硬件信息
    pub fn print(&self) {
        println!("=== 硬件信息 ===");
        println!();
        
        println!("GPU:");
        println!("  厂商: {:?}", self.gpu.vendor);
        println!("  型号: {}", self.gpu.name);
        println!("  等级: {:?}", self.gpu.tier);
        println!("  显存: {} MB", self.gpu.vram_mb);
        println!("  驱动版本: {}", self.gpu.driver_version);
        println!();
        
        if let Some(npu) = &self.npu {
            println!("NPU:");
            println!("  厂商: {:?}", npu.vendor);
            println!("  型号: {}", npu.name);
            println!("  算力: {:.2} TOPS", npu.tops);
            println!();
        }
        
        if let Some(soc) = &self.soc {
            println!("SoC:");
            println!("  厂商: {:?}", soc.vendor);
            println!("  型号: {}", soc.name);
            println!("  CPU核心数: {}", soc.cpu_cores);
            println!("  GPU核心数: {}", soc.gpu_cores);
            println!();
        }
        
        println!("性能等级: {:?}", self.capability.tier);
        println!("推荐配置: {:?}", self.recommended_config.quality_preset);
        println!();
    }
}

/// 获取全局硬件信息（缓存）
pub fn get_hardware_info() -> &'static HardwareInfo {
    HARDWARE_INFO.get_or_init(HardwareInfo::detect)
}

/// 打印硬件信息
pub fn print_hardware_info() {
    get_hardware_info().print();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detection() {
        let info = get_hardware_info();
        println!("{:#?}", info);
        assert!(!info.gpu.name.is_empty());
    }

    #[test]
    fn test_print_info() {
        print_hardware_info();
    }
}
