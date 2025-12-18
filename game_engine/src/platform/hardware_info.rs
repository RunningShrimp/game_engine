//! 硬件信息模块
//!
//! 提供硬件检测功能的简化实现

/// 硬件信息
#[derive(Debug, Clone)]
pub struct HardwareInfo {
    /// CPU核心数
    pub cpu_cores: usize,
    /// 系统内存大小（字节）
    pub total_memory: usize,
    /// GPU名称
    pub gpu_name: String,
    /// 支持的特性
    pub features: HardwareFeatures,
}

/// 硬件特性
#[derive(Debug, Clone)]
pub struct HardwareFeatures {
    /// 是否支持光线追踪
    pub ray_tracing: bool,
    /// 是否支持HDR
    pub hdr: bool,
    /// 是否支持VRS（可变速率着色）
    pub vrs: bool,
}

impl HardwareInfo {
    /// 检测硬件信息
    pub fn detect() -> Self {
        Self {
            cpu_cores: num_cpus::get(),
            total_memory: Self::get_system_memory(),
            gpu_name: "Unknown GPU".to_string(),
            features: HardwareFeatures {
                ray_tracing: false,
                hdr: false,
                vrs: false,
            },
        }
    }

    /// 获取系统内存大小
    fn get_system_memory() -> usize {
        // 简化实现，返回一个合理的默认值
        8 * 1024 * 1024 * 1024 // 8GB
    }
}