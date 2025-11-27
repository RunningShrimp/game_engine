/// GPU检测模块
/// 
/// 检测并识别主流GPU，包括独立显卡和集成显卡

use std::collections::HashMap;

/// GPU厂商
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// GPU性能等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

/// GPU信息
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub vendor: GpuVendor,
    pub name: String,
    pub tier: GpuTier,
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
            vram_mb: 2048,
            driver_version: "Unknown".to_string(),
            supports_raytracing: false,
            supports_mesh_shaders: false,
            supports_variable_rate_shading: false,
            compute_units: 0,
        }
    }
}

/// 检测GPU信息
pub fn detect_gpu() -> GpuInfo {
    // 尝试通过wgpu获取GPU信息
    if let Some(info) = detect_gpu_wgpu() {
        return info;
    }
    
    // 回退到系统信息检测
    detect_gpu_system()
}

/// 通过wgpu检测GPU
fn detect_gpu_wgpu() -> Option<GpuInfo> {
    use wgpu::{Instance, Backends, DeviceType};
    
    let instance = Instance::new(wgpu::InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });
    
    let adapters: Vec<_> = instance.enumerate_adapters(Backends::all());
    
    if adapters.is_empty() {
        return None;
    }
    
    // 优先选择独立显卡
    let adapter = adapters.iter()
        .find(|a| a.get_info().device_type == DeviceType::DiscreteGpu)
        .or_else(|| adapters.iter()
            .find(|a| a.get_info().device_type == DeviceType::IntegratedGpu))
        .or_else(|| adapters.first())?;
    
    let info = adapter.get_info();
    let features = adapter.features();
    let limits = adapter.limits();
    
    let vendor = match info.vendor {
        0x10DE => GpuVendor::Nvidia,
        0x1002 | 0x1022 => GpuVendor::Amd,
        0x8086 => GpuVendor::Intel,
        0x106B => GpuVendor::Apple,
        _ => {
            let name_lower = info.name.to_lowercase();
            if name_lower.contains("nvidia") || name_lower.contains("geforce") || name_lower.contains("rtx") {
                GpuVendor::Nvidia
            } else if name_lower.contains("amd") || name_lower.contains("radeon") {
                GpuVendor::Amd
            } else if name_lower.contains("intel") || name_lower.contains("uhd") || name_lower.contains("iris") {
                GpuVendor::Intel
            } else if name_lower.contains("apple") || name_lower.contains("m1") || name_lower.contains("m2") || name_lower.contains("m3") {
                GpuVendor::Apple
            } else if name_lower.contains("adreno") {
                GpuVendor::Qualcomm
            } else if name_lower.contains("mali") {
                GpuVendor::Mali
            } else if name_lower.contains("powervr") {
                GpuVendor::PowerVR
            } else {
                GpuVendor::Unknown
            }
        }
    };
    
    let tier = classify_gpu_tier(vendor, &info.name, info.device_type);
    
    // 估算显存（wgpu不直接提供）
    let vram_mb = estimate_vram(&info.name, info.device_type);
    
    Some(GpuInfo {
        vendor,
        name: info.name.clone(),
        tier,
        vram_mb,
        driver_version: format!("{}", info.driver_info),
        supports_raytracing: features.contains(wgpu::Features::RAY_TRACING_ACCELERATION_STRUCTURE),
        supports_mesh_shaders: false, // wgpu暂不支持mesh shader检测
        supports_variable_rate_shading: false, // wgpu暂不支持VRS检测
        compute_units: 0, // wgpu不提供
    })
}

/// 通过系统信息检测GPU
fn detect_gpu_system() -> GpuInfo {
    #[cfg(target_os = "linux")]
    {
        if let Some(info) = detect_gpu_linux() {
            return info;
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Some(info) = detect_gpu_windows() {
            return info;
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if let Some(info) = detect_gpu_macos() {
            return info;
        }
    }
    
    GpuInfo::default()
}

#[cfg(target_os = "linux")]
fn detect_gpu_linux() -> Option<GpuInfo> {
    use std::process::Command;
    
    // 尝试使用lspci
    if let Ok(output) = Command::new("lspci").output() {
        if let Ok(text) = String::from_utf8(output.stdout) {
            for line in text.lines() {
                if line.contains("VGA") || line.contains("3D") {
                    let name = line.split(':').nth(2)?.trim().to_string();
                    let vendor = detect_vendor_from_name(&name);
                    let tier = classify_gpu_tier(vendor, &name, wgpu::DeviceType::DiscreteGpu);
                    
                    return Some(GpuInfo {
                        vendor,
                        name,
                        tier,
                        vram_mb: 4096,
                        driver_version: "Unknown".to_string(),
                        supports_raytracing: false,
                        supports_mesh_shaders: false,
                        supports_variable_rate_shading: false,
                        compute_units: 0,
                    });
                }
            }
        }
    }
    
    None
}

#[cfg(target_os = "windows")]
fn detect_gpu_windows() -> Option<GpuInfo> {
    // Windows可以使用WMI或DirectX API
    // 这里简化处理
    None
}

#[cfg(target_os = "macos")]
fn detect_gpu_macos() -> Option<GpuInfo> {
    use std::process::Command;
    
    // 使用system_profiler
    if let Ok(output) = Command::new("system_profiler")
        .arg("SPDisplaysDataType")
        .output()
    {
        if let Ok(text) = String::from_utf8(output.stdout) {
            for line in text.lines() {
                if line.contains("Chipset Model:") {
                    let name = line.split(':').nth(1)?.trim().to_string();
                    let vendor = detect_vendor_from_name(&name);
                    let tier = classify_gpu_tier(vendor, &name, wgpu::DeviceType::IntegratedGpu);
                    
                    return Some(GpuInfo {
                        vendor,
                        name,
                        tier,
                        vram_mb: 8192,
                        driver_version: "Unknown".to_string(),
                        supports_raytracing: vendor == GpuVendor::Apple,
                        supports_mesh_shaders: vendor == GpuVendor::Apple,
                        supports_variable_rate_shading: false,
                        compute_units: 0,
                    });
                }
            }
        }
    }
    
    None
}

/// 从名称检测厂商
fn detect_vendor_from_name(name: &str) -> GpuVendor {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("nvidia") || name_lower.contains("geforce") || name_lower.contains("rtx") || name_lower.contains("gtx") {
        GpuVendor::Nvidia
    } else if name_lower.contains("amd") || name_lower.contains("radeon") {
        GpuVendor::Amd
    } else if name_lower.contains("intel") || name_lower.contains("uhd") || name_lower.contains("iris") || name_lower.contains("arc") {
        GpuVendor::Intel
    } else if name_lower.contains("apple") || name_lower.contains("m1") || name_lower.contains("m2") || name_lower.contains("m3") {
        GpuVendor::Apple
    } else if name_lower.contains("adreno") {
        GpuVendor::Qualcomm
    } else if name_lower.contains("mali") {
        GpuVendor::Mali
    } else if name_lower.contains("powervr") {
        GpuVendor::PowerVR
    } else {
        GpuVendor::Unknown
    }
}

/// 分类GPU性能等级
fn classify_gpu_tier(vendor: GpuVendor, name: &str, device_type: wgpu::DeviceType) -> GpuTier {
    let name_lower = name.to_lowercase();
    
    match vendor {
        GpuVendor::Nvidia => {
            if name_lower.contains("rtx 4090") || name_lower.contains("rtx 4080") {
                GpuTier::Flagship
            } else if name_lower.contains("rtx 40") || name_lower.contains("rtx 3090") || name_lower.contains("rtx 3080") {
                GpuTier::High
            } else if name_lower.contains("rtx 30") || name_lower.contains("rtx 20") {
                GpuTier::MediumHigh
            } else if name_lower.contains("gtx 16") || name_lower.contains("gtx 10") {
                GpuTier::Medium
            } else if name_lower.contains("mx") {
                GpuTier::MediumLow
            } else {
                GpuTier::Medium
            }
        }
        GpuVendor::Amd => {
            if name_lower.contains("7900") || name_lower.contains("7800") {
                GpuTier::Flagship
            } else if name_lower.contains("79") || name_lower.contains("78") || name_lower.contains("6900") {
                GpuTier::High
            } else if name_lower.contains("69") || name_lower.contains("68") || name_lower.contains("59") {
                GpuTier::MediumHigh
            } else if name_lower.contains("58") || name_lower.contains("57") {
                GpuTier::Medium
            } else {
                GpuTier::MediumLow
            }
        }
        GpuVendor::Intel => {
            if name_lower.contains("arc a7") {
                GpuTier::MediumHigh
            } else if name_lower.contains("arc a5") {
                GpuTier::Medium
            } else if name_lower.contains("arc a3") {
                GpuTier::MediumLow
            } else if name_lower.contains("iris xe") {
                GpuTier::MediumLow
            } else if name_lower.contains("uhd") {
                GpuTier::Low
            } else {
                GpuTier::Low
            }
        }
        GpuVendor::Apple => {
            if name_lower.contains("m3 max") || name_lower.contains("m2 ultra") {
                GpuTier::High
            } else if name_lower.contains("m3 pro") || name_lower.contains("m2 max") {
                GpuTier::MediumHigh
            } else if name_lower.contains("m3") || name_lower.contains("m2 pro") {
                GpuTier::Medium
            } else if name_lower.contains("m2") || name_lower.contains("m1 pro") || name_lower.contains("m1 max") {
                GpuTier::MediumLow
            } else {
                GpuTier::Low
            }
        }
        GpuVendor::Qualcomm => {
            if name_lower.contains("adreno 7") {
                GpuTier::MediumHigh
            } else if name_lower.contains("adreno 6") {
                GpuTier::Medium
            } else {
                GpuTier::MediumLow
            }
        }
        GpuVendor::Mali => {
            if name_lower.contains("g7") {
                GpuTier::Medium
            } else {
                GpuTier::MediumLow
            }
        }
        _ => {
            match device_type {
                wgpu::DeviceType::DiscreteGpu => GpuTier::Medium,
                wgpu::DeviceType::IntegratedGpu => GpuTier::MediumLow,
                _ => GpuTier::Low,
            }
        }
    }
}

/// 估算显存大小
fn estimate_vram(name: &str, device_type: wgpu::DeviceType) -> u64 {
    let name_lower = name.to_lowercase();
    
    // 根据GPU型号估算
    if name_lower.contains("4090") {
        24576
    } else if name_lower.contains("4080") || name_lower.contains("3090") {
        16384
    } else if name_lower.contains("4070") || name_lower.contains("3080") {
        12288
    } else if name_lower.contains("4060") || name_lower.contains("3070") {
        8192
    } else if name_lower.contains("3060") || name_lower.contains("2060") {
        6144
    } else if device_type == wgpu::DeviceType::DiscreteGpu {
        4096
    } else {
        2048 // 集成显卡
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_detection() {
        let gpu = detect_gpu();
        println!("Detected GPU: {:#?}", gpu);
        assert!(!gpu.name.is_empty());
    }

    #[test]
    fn test_vendor_detection() {
        assert_eq!(detect_vendor_from_name("NVIDIA GeForce RTX 4090"), GpuVendor::Nvidia);
        assert_eq!(detect_vendor_from_name("AMD Radeon RX 7900 XTX"), GpuVendor::Amd);
        assert_eq!(detect_vendor_from_name("Intel Arc A770"), GpuVendor::Intel);
        assert_eq!(detect_vendor_from_name("Apple M2 Max"), GpuVendor::Apple);
    }
}
