//  GPU检测模块
// 
//  检测并识别主流GPU，包括独立显卡和集成显卡

//  GPU厂商
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Qualcomm,
    Mali,
    PowerVR,
    Unknown,
}

//  GPU性能等级
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum GpuTier {
    /// 低端（入门级集显）
    Low,
    /// 中低端（主流集显）
    MediumLow,
    /// 中端（高端集显/入门独显）
    Medium,
    /// 中高端（主流独显）
    MediumHigh,
    /// 高端（高性能独显）
    High,
    /// 旗舰（顶级独显）
    Flagship,
}

//  GPU信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpuInfo {
    pub vendor: GpuVendor,
    pub name: String,
    pub tier: GpuTier,
    pub device_type: DeviceType,
    pub vram_mb: u64,
    pub driver_version: String,
    pub supports_raytracing: bool,
    pub supports_mesh_shaders: bool,
    pub supports_variable_rate_shading: bool,
    pub compute_units: u32,
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self {
            vendor: GpuVendor::Unknown,
            name: "Unknown GPU".to_string(),
            tier: GpuTier::Medium,
            device_type: DeviceType::IntegratedGpu,
            vram_mb: 2048,
            driver_version: "Unknown".to_string(),
            supports_raytracing: false,
            supports_mesh_shaders: false,
            supports_variable_rate_shading: false,
            compute_units: 0,
        }
    }
}

//  检测GPU信息
pub fn detect_gpu() -> GpuInfo {
    // TODO: Implement GPU detection
    // Temporarily return default values
    let info = GpuInfo::default();
    
    #[cfg(feature = "wgpu")]
    if let Some(wgpu_info) = detect_gpu_wgpu() {
        return wgpu_info;
    }
    
    info
}

//  设备类型（本地定义，避免依赖wgpu）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeviceType {
    DiscreteGpu,
    IntegratedGpu,
    VirtualGpu,
    Cpu,
    Other,
}

//  通过wgpu检测GPU（可选功能）
#[cfg(feature = "wgpu")]
fn detect_gpu_wgpu() -> Option<GpuInfo> {
    // use wgpu::{Backends, DeviceType as WgpuDeviceType, Instance}; // TODO: Uncomment when implementing GPU detection

    // TODO: Implement GPU detection with proper async handling
    // Temporarily return None to allow compilation
    None
}

