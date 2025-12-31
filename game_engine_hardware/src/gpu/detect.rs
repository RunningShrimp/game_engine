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
#[cfg(feature = "wgpu")]
pub fn detect_gpu() -> GpuInfo {
    if let Some(wgpu_info) = detect_gpu_wgpu() {
        wgpu_info
    } else {
        tracing::warn!("GPU detection returned None. Using default values.");
        GpuInfo::default()
    }
}

//  检测GPU信息
#[cfg(not(feature = "wgpu"))]
pub fn detect_gpu() -> GpuInfo {
    tracing::warn!("GPU detection requires 'wgpu' feature. Returning default values.");
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

    let instance = Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });

    let mut info = None;

    let adapters = instance.enumerate_adapters(wgpu::Backends::all());
    if let Some(adapter) = adapters.first() {
        let adapter_info = adapter.get_info();

        // `AdapterInfo.vendor` is a numeric vendor ID; use the adapter name string for text matching
        let name_lc = adapter_info.name.to_lowercase();
        let vendor = match name_lc.as_str() {
            s if s.contains("nvidia") || s.contains("nv") => GpuVendor::Nvidia,
            s if s.contains("amd") || s.contains("radeon") => GpuVendor::Amd,
            s if s.contains("intel") => GpuVendor::Intel,
            s if s.contains("apple") => GpuVendor::Apple,
            s if s.contains("qualcomm") => GpuVendor::Qualcomm,
            s if s.contains("arm") || s.contains("mali") => GpuVendor::Mali,
            s if s.contains("powervr") => GpuVendor::PowerVR,
            _ => GpuVendor::Unknown,
        };

        // adapt to current wgpu API: use `limits()` method
        let tier = classify_gpu_tier(&adapter_info, &adapter.limits());

        let device_type = match adapter_info.device_type {
            WgpuDeviceType::DiscreteGpu => DeviceType::DiscreteGpu,
            WgpuDeviceType::IntegratedGpu => DeviceType::IntegratedGpu,
            WgpuDeviceType::VirtualGpu => DeviceType::VirtualGpu,
            WgpuDeviceType::Cpu => DeviceType::Cpu,
            WgpuDeviceType::Other => DeviceType::Other,
        };

        // AdapterInfo does not always expose VRAM in a cross-platform way; default to 0 when unknown
        info = Some(GpuInfo {
            vendor,
            name: adapter_info.name.clone(),
            tier,
            device_type,
            vram_mb: 0,
            driver_version: adapter_info.driver.clone(),
            // `driver_info` is a String on recent wgpu versions
            driver_info: adapter_info.driver_info.clone(),
            supports_raytracing: supports_raytracing(&adapter_info),
            supports_mesh_shaders: true,
            supports_variable_rate_shading: true,
            compute_units: compute_units_count(&adapter.limits()),
        });
    }

    info
}

#[cfg(feature = "wgpu")]
fn classify_gpu_tier(_adapter_info: &wgpu::AdapterInfo, limits: &wgpu::Limits) -> GpuTier {
    // AdapterInfo often doesn't provide VRAM; classify primarily by max texture size
    let max_texture_2d = limits.max_texture_dimension_2d;

    match max_texture_2d {
        t if t >= 16384 => GpuTier::Flagship,
        t if t >= 8192 => GpuTier::High,
        t if t >= 4096 => GpuTier::MediumHigh,
        t if t >= 2048 => GpuTier::Medium,
        t if t >= 1024 => GpuTier::MediumLow,
        _ => GpuTier::Low,
    }
}

#[cfg(feature = "wgpu")]
fn supports_raytracing(adapter_info: &wgpu::AdapterInfo) -> bool {
    // Basic heuristic: detect common model names that indicate hardware RT support
    let name = adapter_info.name.to_lowercase();
    if name.contains("rtx") || name.contains("ray") {
        return true;
    }

    // Check driver string for keywords
    let driver = adapter_info.driver.to_lowercase();
    if driver.contains("nvidia") || driver.contains("amd") {
        // conservatively assume modern discrete drivers support ray tracing
        return true;
    }

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
