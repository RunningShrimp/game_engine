//! 硬件检测和优化模块
//!
//! 提供GPU、NPU、SoC等硬件的自动检测和优化建议。

pub mod gpu;
pub mod npu;
pub mod soc;
pub mod capability;
pub mod config;
pub mod upscaling;
pub mod adaptive;
pub mod utils;
pub mod error;

// Re-export public API
pub use gpu::{GpuInfo, GpuVendor, GpuTier, detect_gpu};
pub use npu::{NpuInfo, NpuVendor, detect_npu};
pub use soc::{SocInfo, SocVendor, detect_soc};
pub use capability::{HardwareCapability, PerformanceTier};
pub use config::{AutoConfig, QualityPreset};
pub use error::{HardwareError, HardwareResult};
pub use npu::sdk::extended::{OpenVINOEngine, ROCmEngine, AscendEngine, SNPEEngine, NeuroPilotEngine};

use std::sync::OnceLock;

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

/// 全局硬件信息缓存
static HARDWARE_INFO: OnceLock<HardwareInfo> = OnceLock::new();

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
        assert!(!info.gpu.name.is_empty());
    }
}

