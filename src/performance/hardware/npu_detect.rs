/// NPU（神经网络处理器）检测模块
/// 
/// 检测并识别主流NPU，用于AI加速

/// NPU厂商
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NpuVendor {
    /// 华为昇腾
    HuaweiAscend,
    /// 苹果Neural Engine
    AppleNeuralEngine,
    /// 高通Hexagon
    QualcommHexagon,
    /// 联发科APU
    MediaTekApu,
    /// Intel Movidius
    IntelMovidius,
    /// Google Edge TPU
    GoogleEdgeTpu,
    /// NVIDIA Tensor Core
    NvidiaTensorCore,
    /// AMD Matrix Core
    AmdMatrixCore,
    Unknown,
}

/// NPU信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NpuInfo {
    pub vendor: NpuVendor,
    pub name: String,
    /// 算力（TOPS - Tera Operations Per Second）
    pub tops: f32,
    pub supports_int8: bool,
    pub supports_fp16: bool,
    pub supports_bf16: bool,
}

impl Default for NpuInfo {
    fn default() -> Self {
        Self {
            vendor: NpuVendor::Unknown,
            name: "No NPU".to_string(),
            tops: 0.0,
            supports_int8: false,
            supports_fp16: false,
            supports_bf16: false,
        }
    }
}

/// 检测NPU信息
pub fn detect_npu() -> Option<NpuInfo> {
    // 检测Apple Neural Engine
    #[cfg(target_os = "macos")]
    {
        if let Some(info) = detect_apple_neural_engine() {
            return Some(info);
        }
    }
    
    // 检测Android NPU
    #[cfg(target_os = "android")]
    {
        if let Some(info) = detect_android_npu() {
            return Some(info);
        }
    }
    
    // 检测华为昇腾（Linux）
    #[cfg(target_os = "linux")]
    {
        if let Some(info) = detect_huawei_ascend() {
            return Some(info);
        }
    }
    
    // 检测NVIDIA Tensor Core
    if let Some(info) = detect_nvidia_tensor_core() {
        return Some(info);
    }
    
    // 检测AMD Matrix Core
    if let Some(info) = detect_amd_matrix_core() {
        return Some(info);
    }
    
    None
}

#[cfg(target_os = "macos")]
fn detect_apple_neural_engine() -> Option<NpuInfo> {
    use std::process::Command;
    
    // 检测Apple芯片
    if let Ok(output) = Command::new("sysctl")
        .arg("-n")
        .arg("machdep.cpu.brand_string")
        .output()
    {
        if let Ok(brand) = String::from_utf8(output.stdout) {
            let brand_lower = brand.to_lowercase();
            
            if brand_lower.contains("m3") {
                return Some(NpuInfo {
                    vendor: NpuVendor::AppleNeuralEngine,
                    name: "Apple Neural Engine (M3)".to_string(),
                    tops: 18.0, // M3的Neural Engine约18 TOPS
                    supports_int8: true,
                    supports_fp16: true,
                    supports_bf16: false,
                });
            } else if brand_lower.contains("m2") {
                return Some(NpuInfo {
                    vendor: NpuVendor::AppleNeuralEngine,
                    name: "Apple Neural Engine (M2)".to_string(),
                    tops: 15.8,
                    supports_int8: true,
                    supports_fp16: true,
                    supports_bf16: false,
                });
            } else if brand_lower.contains("m1") {
                return Some(NpuInfo {
                    vendor: NpuVendor::AppleNeuralEngine,
                    name: "Apple Neural Engine (M1)".to_string(),
                    tops: 11.0,
                    supports_int8: true,
                    supports_fp16: true,
                    supports_bf16: false,
                });
            }
        }
    }
    
    None
}

#[cfg(target_os = "android")]
fn detect_android_npu() -> Option<NpuInfo> {
    use std::fs;
    
    // 检测高通Hexagon DSP
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        if content.to_lowercase().contains("qualcomm") {
            // 简化检测，实际需要更详细的API
            return Some(NpuInfo {
                vendor: NpuVendor::QualcommHexagon,
                name: "Qualcomm Hexagon DSP".to_string(),
                tops: 15.0, // 估算值
                supports_int8: true,
                supports_fp16: true,
                supports_bf16: false,
            });
        }
        
        if content.to_lowercase().contains("mediatek") {
            return Some(NpuInfo {
                vendor: NpuVendor::MediaTekApu,
                name: "MediaTek APU".to_string(),
                tops: 10.0,
                supports_int8: true,
                supports_fp16: true,
                supports_bf16: false,
            });
        }
    }
    
    None
}

#[cfg(target_os = "linux")]
fn detect_huawei_ascend() -> Option<NpuInfo> {
    use std::path::Path;
    
    // 检查是否安装了昇腾驱动
    if Path::new("/usr/local/Ascend").exists() {
        return Some(NpuInfo {
            vendor: NpuVendor::HuaweiAscend,
            name: "Huawei Ascend NPU".to_string(),
            tops: 256.0, // 昇腾910约256 TOPS
            supports_int8: true,
            supports_fp16: true,
            supports_bf16: true,
        });
    }
    
    None
}

fn detect_nvidia_tensor_core() -> Option<NpuInfo> {
    // 通过GPU信息推断Tensor Core
    use crate::performance::hardware::gpu_detect::{detect_gpu, GpuVendor};
    
    let gpu = detect_gpu();
    if gpu.vendor == GpuVendor::Nvidia {
        let name_lower = gpu.name.to_lowercase();
        
        if name_lower.contains("rtx") || name_lower.contains("a100") || name_lower.contains("h100") {
            let tops = if name_lower.contains("4090") {
                1321.0 // RTX 4090 Tensor性能
            } else if name_lower.contains("4080") {
                780.0
            } else if name_lower.contains("3090") {
                285.0
            } else if name_lower.contains("h100") {
                2000.0
            } else if name_lower.contains("a100") {
                1248.0
            } else {
                100.0
            };
            
            return Some(NpuInfo {
                vendor: NpuVendor::NvidiaTensorCore,
                name: format!("{} Tensor Cores", gpu.name),
                tops,
                supports_int8: true,
                supports_fp16: true,
                supports_bf16: true,
            });
        }
    }
    
    None
}

fn detect_amd_matrix_core() -> Option<NpuInfo> {
    use crate::performance::hardware::gpu_detect::{detect_gpu, GpuVendor};
    
    let gpu = detect_gpu();
    if gpu.vendor == GpuVendor::Amd {
        let name_lower = gpu.name.to_lowercase();
        
        // RDNA3架构支持AI加速
        if name_lower.contains("7900") || name_lower.contains("7800") {
            return Some(NpuInfo {
                vendor: NpuVendor::AmdMatrixCore,
                name: format!("{} AI Accelerators", gpu.name),
                tops: 123.0, // RX 7900 XTX约123 TOPS
                supports_int8: true,
                supports_fp16: true,
                supports_bf16: false,
            });
        }
    }
    
    None
}

/// NPU是否可用
pub fn is_npu_available() -> bool {
    detect_npu().is_some()
}

/// 获取NPU算力（TOPS）
pub fn get_npu_tops() -> f32 {
    detect_npu().map(|npu| npu.tops).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npu_detection() {
        if let Some(npu) = detect_npu() {
            println!("Detected NPU: {:#?}", npu);
            assert!(npu.tops > 0.0);
        } else {
            println!("No NPU detected");
        }
    }

    #[test]
    fn test_npu_availability() {
        let available = is_npu_available();
        println!("NPU available: {}", available);
    }
}
