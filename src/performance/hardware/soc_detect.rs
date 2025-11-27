/// SoC（System on Chip）检测模块
/// 
/// 检测移动和嵌入式平台的SoC信息

/// SoC厂商
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocVendor {
    Apple,
    Qualcomm,
    MediaTek,
    Samsung,
    HiSilicon,
    Nvidia,
    Unknown,
}

/// SoC信息
#[derive(Debug, Clone)]
pub struct SocInfo {
    pub vendor: SocVendor,
    pub name: String,
    pub cpu_cores: u32,
    pub gpu_cores: u32,
    pub max_cpu_freq_mhz: u32,
    pub process_node_nm: u32,
    pub thermal_design_power_w: f32,
}

impl Default for SocInfo {
    fn default() -> Self {
        Self {
            vendor: SocVendor::Unknown,
            name: "Unknown SoC".to_string(),
            cpu_cores: 4,
            gpu_cores: 0,
            max_cpu_freq_mhz: 2000,
            process_node_nm: 7,
            thermal_design_power_w: 5.0,
        }
    }
}

/// 检测SoC信息
pub fn detect_soc() -> Option<SocInfo> {
    #[cfg(target_os = "macos")]
    {
        if let Some(info) = detect_apple_soc() {
            return Some(info);
        }
    }
    
    #[cfg(target_os = "android")]
    {
        if let Some(info) = detect_android_soc() {
            return Some(info);
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Some(info) = detect_linux_soc() {
            return Some(info);
        }
    }
    
    None
}

#[cfg(target_os = "macos")]
fn detect_apple_soc() -> Option<SocInfo> {
    use std::process::Command;
    
    // 检测Apple芯片
    if let Ok(output) = Command::new("sysctl")
        .arg("-n")
        .arg("machdep.cpu.brand_string")
        .output()
    {
        if let Ok(brand) = String::from_utf8(output.stdout) {
            let brand_lower = brand.to_lowercase();
            
            if brand_lower.contains("m3 max") {
                return Some(SocInfo {
                    vendor: SocVendor::Apple,
                    name: "Apple M3 Max".to_string(),
                    cpu_cores: 16,
                    gpu_cores: 40,
                    max_cpu_freq_mhz: 4050,
                    process_node_nm: 3,
                    thermal_design_power_w: 30.0,
                });
            } else if brand_lower.contains("m3 pro") {
                return Some(SocInfo {
                    vendor: SocVendor::Apple,
                    name: "Apple M3 Pro".to_string(),
                    cpu_cores: 12,
                    gpu_cores: 18,
                    max_cpu_freq_mhz: 4050,
                    process_node_nm: 3,
                    thermal_design_power_w: 20.0,
                });
            } else if brand_lower.contains("m3") {
                return Some(SocInfo {
                    vendor: SocVendor::Apple,
                    name: "Apple M3".to_string(),
                    cpu_cores: 8,
                    gpu_cores: 10,
                    max_cpu_freq_mhz: 4050,
                    process_node_nm: 3,
                    thermal_design_power_w: 15.0,
                });
            } else if brand_lower.contains("m2") {
                return Some(SocInfo {
                    vendor: SocVendor::Apple,
                    name: "Apple M2".to_string(),
                    cpu_cores: 8,
                    gpu_cores: 10,
                    max_cpu_freq_mhz: 3500,
                    process_node_nm: 5,
                    thermal_design_power_w: 15.0,
                });
            } else if brand_lower.contains("m1") {
                return Some(SocInfo {
                    vendor: SocVendor::Apple,
                    name: "Apple M1".to_string(),
                    cpu_cores: 8,
                    gpu_cores: 8,
                    max_cpu_freq_mhz: 3200,
                    process_node_nm: 5,
                    thermal_design_power_w: 15.0,
                });
            }
        }
    }
    
    None
}

#[cfg(target_os = "android")]
fn detect_android_soc() -> Option<SocInfo> {
    use std::fs;
    
    // 读取/proc/cpuinfo
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        let content_lower = content.to_lowercase();
        
        // 高通骁龙
        if content_lower.contains("qualcomm") {
            if content_lower.contains("8 gen 3") || content_lower.contains("8gen3") {
                return Some(SocInfo {
                    vendor: SocVendor::Qualcomm,
                    name: "Snapdragon 8 Gen 3".to_string(),
                    cpu_cores: 8,
                    gpu_cores: 0,
                    max_cpu_freq_mhz: 3300,
                    process_node_nm: 4,
                    thermal_design_power_w: 10.0,
                });
            } else if content_lower.contains("8 gen 2") {
                return Some(SocInfo {
                    vendor: SocVendor::Qualcomm,
                    name: "Snapdragon 8 Gen 2".to_string(),
                    cpu_cores: 8,
                    gpu_cores: 0,
                    max_cpu_freq_mhz: 3200,
                    process_node_nm: 4,
                    thermal_design_power_w: 9.0,
                });
            }
        }
        
        // 联发科天玑
        if content_lower.contains("mediatek") || content_lower.contains("dimensity") {
            if content_lower.contains("9300") {
                return Some(SocInfo {
                    vendor: SocVendor::MediaTek,
                    name: "Dimensity 9300".to_string(),
                    cpu_cores: 8,
                    gpu_cores: 0,
                    max_cpu_freq_mhz: 3250,
                    process_node_nm: 4,
                    thermal_design_power_w: 10.0,
                });
            }
        }
        
        // 三星Exynos
        if content_lower.contains("exynos") {
            return Some(SocInfo {
                vendor: SocVendor::Samsung,
                name: "Samsung Exynos".to_string(),
                cpu_cores: 8,
                gpu_cores: 0,
                max_cpu_freq_mhz: 2900,
                process_node_nm: 5,
                thermal_design_power_w: 8.0,
            });
        }
        
        // 华为麒麟
        if content_lower.contains("kirin") || content_lower.contains("hisilicon") {
            return Some(SocInfo {
                vendor: SocVendor::HiSilicon,
                name: "HiSilicon Kirin".to_string(),
                cpu_cores: 8,
                gpu_cores: 0,
                max_cpu_freq_mhz: 2860,
                process_node_nm: 5,
                thermal_design_power_w: 8.0,
            });
        }
    }
    
    // 尝试读取系统属性
    if let Ok(output) = std::process::Command::new("getprop")
        .arg("ro.product.board")
        .output()
    {
        if let Ok(board) = String::from_utf8(output.stdout) {
            let board_lower = board.to_lowercase();
            
            if board_lower.contains("sm8") {
                return Some(SocInfo {
                    vendor: SocVendor::Qualcomm,
                    name: "Qualcomm Snapdragon 8 Series".to_string(),
                    cpu_cores: 8,
                    gpu_cores: 0,
                    max_cpu_freq_mhz: 3000,
                    process_node_nm: 5,
                    thermal_design_power_w: 9.0,
                });
            }
        }
    }
    
    None
}

#[cfg(target_os = "linux")]
fn detect_linux_soc() -> Option<SocInfo> {
    use std::fs;
    
    // 检测Jetson等嵌入式平台
    if let Ok(content) = fs::read_to_string("/proc/device-tree/model") {
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("jetson") {
            if content_lower.contains("orin") {
                return Some(SocInfo {
                    vendor: SocVendor::Nvidia,
                    name: "NVIDIA Jetson Orin".to_string(),
                    cpu_cores: 12,
                    gpu_cores: 0,
                    max_cpu_freq_mhz: 2200,
                    process_node_nm: 8,
                    thermal_design_power_w: 60.0,
                });
            } else if content_lower.contains("xavier") {
                return Some(SocInfo {
                    vendor: SocVendor::Nvidia,
                    name: "NVIDIA Jetson Xavier".to_string(),
                    cpu_cores: 8,
                    gpu_cores: 0,
                    max_cpu_freq_mhz: 2260,
                    process_node_nm: 12,
                    thermal_design_power_w: 30.0,
                });
            }
        }
    }
    
    None
}

/// 是否为移动/嵌入式平台
pub fn is_mobile_platform() -> bool {
    detect_soc().is_some()
}

/// 获取CPU核心数
pub fn get_cpu_cores() -> u32 {
    detect_soc()
        .map(|soc| soc.cpu_cores)
        .unwrap_or_else(|| num_cpus::get() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soc_detection() {
        if let Some(soc) = detect_soc() {
            println!("Detected SoC: {:#?}", soc);
            assert!(soc.cpu_cores > 0);
        } else {
            println!("No SoC detected (likely desktop platform)");
        }
    }

    #[test]
    fn test_mobile_platform() {
        let is_mobile = is_mobile_platform();
        println!("Is mobile platform: {}", is_mobile);
    }
}
