//  GPU检测模块
//
//  检测并识别主流GPU，包括独立显卡和集成显卡
//
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
    pub driver_info: String,
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
            driver_info: "Unknown".to_string(),
            supports_raytracing: false,
            supports_mesh_shaders: false,
            supports_variable_rate_shading: false,
            compute_units: 0,
        }
    }
}

//  检测GPU信息
pub fn detect_gpu() -> GpuInfo {
    #[cfg(feature = "wgpu")]
    if let Some(wgpu_info) = detect_gpu_wgpu() {
        return wgpu_info;
    }

    #[cfg(not(feature = "wgpu"))]
    {
        tracing::warn!("GPU detection requires 'wgpu' feature. Returning default values.");
        return GpuInfo::default();
    }

    #[cfg(feature = "wgpu")]
    {
        tracing::warn!("GPU detection returned None. Using default values.");
    }

    #[cfg(feature = "wgpu")]
    GpuInfo::default()
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
    use wgpu::{Backends, DeviceType as WgpuDeviceType, Instance};

    let instance = Instance::new(wgpu::InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });

    let mut info = None;

    for adapter in instance.enumerate_adapters(wgpu::Backends::all()) {
        let adapter_info = adapter.get_info();

        let vendor = match adapter_info.vendor.to_lowercase() {
            s if s.contains("nvidia") => GpuVendor::Nvidia,
            s if s.contains("amd") || s.contains("radeon") || s.contains("advanced micro devices") => GpuVendor::Amd,
            s if s.contains("intel") => GpuVendor::Intel,
            s if s.contains("apple") => GpuVendor::Apple,
            s if s.contains("qualcomm") => GpuVendor::Qualcomm,
            s if s.contains("arm") || s.contains("mali") => GpuVendor::Mali,
            s if s.contains("powervr") => GpuVendor::PowerVR,
            _ => GpuVendor::Unknown,
        };

        let tier = classify_gpu_tier(&adapter_info, adapter.get_limits());

        let device_type = match adapter_info.device_type {
            WgpuDeviceType::DiscreteGpu => DeviceType::DiscreteGpu,
            WgpuDeviceType::IntegratedGpu => DeviceType::IntegratedGpu,
            WgpuDeviceType::VirtualGpu => DeviceType::VirtualGpu,
            WgpuDeviceType::Cpu => DeviceType::Cpu,
            WgpuDeviceType::Other => DeviceType::Other,
        };

        info = Some(GpuInfo {
            vendor,
            name: adapter_info.name,
            tier,
            device_type,
            vram_mb: adapter_info.memory as u64 / (1024 * 1024),
            driver_version: adapter_info.driver,
            driver_info: adapter_info.driver_info.unwrap_or_default().description,
            supports_raytracing: supports_raytracing(&adapter_info),
            supports_mesh_shaders: true,
            supports_variable_rate_shading: true,
            compute_units: compute_units_count(adapter.get_limits()),
        });

        break;
    }

    info
}

#[cfg(feature = "wgpu")]
fn classify_gpu_tier(adapter_info: &wgpu::AdapterInfo, limits: &wgpu::Limits) -> GpuTier {
    let vram = adapter_info.memory as u64;
    let max_texture_2d = limits.max_texture_dimension_2d;

    match (vram, max_texture_2d) {
        (_, t) if t >= 16384 => GpuTier::Flagship,
        (_, t) if t >= 8192 => GpuTier::High,
        (_, t) if t >= 4096 => GpuTier::MediumHigh,
        (v, _) if v >= 4 * 1024 * 1024 * 1024 => GpuTier::MediumHigh,
        (v, _) if v >= 2 * 1024 * 1024 * 1024 => GpuTier::Medium,
        (v, _) if v >= 1 * 1024 * 1024 * 1024 => GpuTier::MediumLow,
        _ => GpuTier::Low,
    }
}

#[cfg(feature = "wgpu")]
fn supports_raytracing(adapter_info: &wgpu::AdapterInfo) -> bool {
    false
}

#[cfg(feature = "wgpu")]
fn compute_units_count(limits: &wgpu::Limits) -> u32 {
    let max_compute_workgroup_storage_size = limits.max_compute_workgroup_storage_size;

    match max_compute_workgroup_storage_size {
        _ if max_compute_workgroup_storage_size >= 16384 => 4,
        _ if max_compute_workgroup_storage_size >= 8192 => 3,
        _ if max_compute_workgroup_storage_size >= 4096 => 2,
        _ => 1,
    }
}
