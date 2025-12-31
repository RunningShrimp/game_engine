//! GPU能力检测和优化建议
//!
//! 跨平台GPU能力检测，支持vendor-specific优化建议。

use serde::{Deserialize, Serialize};
use std::fmt;

/// GPU厂商
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuVendor {
    /// NVIDIA
    Nvidia,
    /// AMD
    Amd,
    /// Intel
    Intel,
    /// Apple (Apple Silicon)
    Apple,
    /// 未知
    Unknown,
}

impl fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuVendor::Nvidia => write!(f, "NVIDIA"),
            GpuVendor::Amd => write!(f, "AMD"),
            GpuVendor::Intel => write!(f, "Intel"),
            GpuVendor::Apple => write!(f, "Apple"),
            GpuVendor::Unknown => write!(f, "Unknown"),
        }
    }
}

/// GPU架构类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuArchitecture {
    /// NVIDIA Ampere (RTX 30系列)
    NvidiaAmpere,
    /// NVIDIA Turing (RTX 20系列)
    NvidiaTuring,
    /// NVIDIA Volta (GV100)
    NvidiaVolta,
    /// NVIDIA Pascal (GTX 10系列)
    NvidiaPascal,
    /// NVIDIA Maxwell
    NvidiaMaxwell,
    /// NVIDIA Kepler
    NvidiaKepler,
    /// AMD RDNA3 (RX 7000系列)
    AmdRdna3,
    /// AMD RDNA2 (RX 6000系列)
    AmdRdna2,
    /// AMD RDNA1
    AmdRdna1,
    /// AMD CDNA2 (Instinct MI200)
    AmdCdna2,
    /// AMD CDNA1 (Instinct MI100)
    AmdCdna1,
    /// AMD GCN (Graphics Core Next)
    AmdGcn,
    /// Intel Xe (Arc系列)
    IntelXe,
    /// Intel Gen12 (Iris Xe)
    IntelGen12,
    /// Intel Gen11
    IntelGen11,
    /// Apple M1/M2/M3
    AppleSilicon,
    /// 未知架构
    Unknown,
}

impl fmt::Display for GpuArchitecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuArchitecture::NvidiaAmpere => write!(f, "NVIDIA Ampere"),
            GpuArchitecture::NvidiaTuring => write!(f, "NVIDIA Turing"),
            GpuArchitecture::NvidiaVolta => write!(f, "NVIDIA Volta"),
            GpuArchitecture::NvidiaPascal => write!(f, "NVIDIA Pascal"),
            GpuArchitecture::NvidiaMaxwell => write!(f, "NVIDIA Maxwell"),
            GpuArchitecture::NvidiaKepler => write!(f, "NVIDIA Kepler"),
            GpuArchitecture::AmdRdna3 => write!(f, "AMD RDNA3"),
            GpuArchitecture::AmdRdna2 => write!(f, "AMD RDNA2"),
            GpuArchitecture::AmdRdna1 => write!(f, "AMD RDNA1"),
            GpuArchitecture::AmdCdna2 => write!(f, "AMD CDNA2"),
            GpuArchitecture::AmdCdna1 => write!(f, "AMD CDNA1"),
            GpuArchitecture::AmdGcn => write!(f, "AMD GCN"),
            GpuArchitecture::IntelXe => write!(f, "Intel Xe"),
            GpuArchitecture::IntelGen12 => write!(f, "Intel Gen12"),
            GpuArchitecture::IntelGen11 => write!(f, "Intel Gen11"),
            GpuArchitecture::AppleSilicon => write!(f, "Apple Silicon"),
            GpuArchitecture::Unknown => write!(f, "Unknown Architecture"),
        }
    }
}

/// GPU能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuCapabilities {
    /// GPU厂商
    pub vendor: GpuVendor,
    /// GPU架构
    pub architecture: GpuArchitecture,
    /// 设备名称
    pub device_name: String,
    /// 驱动版本
    pub driver_version: String,
    /// 计算能力（CUDA）/ 架构版本（其他）
    pub compute_capability: Option<(u32, u32)>,
    /// VRAM大小（字节）
    pub vram_size: u64,
    /// 最大工作组大小
    pub max_workgroup_size: u32,
    /// 最大缓冲区大小
    pub max_buffer_size: u64,
    /// 是否支持计算着色器
    pub supports_compute: bool,
    /// 是否支持原子操作
    pub supports_atomic: bool,
    /// 是否支持浮点原子
    pub supports_float_atomic: bool,
    /// 是否支持SPIR-V
    pub supports_spirv: bool,
    /// 是否支持DXC
    pub supports_dxc: bool,
    /// 推荐的工作组大小
    pub recommended_workgroup_size: u32,
    /// 物理模拟优化建议
    pub physics_optimizations: Vec<OptimizationHint>,
    /// 粒子系统优化建议
    pub particle_optimizations: Vec<OptimizationHint>,
}

/// 优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationHint {
    /// 建议类型
    pub hint_type: OptimizationType,
    /// 建议描述
    pub description: String,
    /// 预估性能提升（百分比）
    pub estimated_improvement: f32,
    /// 实施难度（1-10）
    pub difficulty: u32,
    /// 是否已应用
    pub applied: bool,
}

/// 优化类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationType {
    /// 增加工作组大小
    IncreaseWorkgroupSize,
    /// 减少工作组大小
    DecreaseWorkgroupSize,
    /// 使用共享内存
    UseSharedMemory,
    /// 使用原子操作
    UseAtomicOperations,
    /// 使用向量化指令
    UseVectorizedInstructions,
    /// 启用早期深度测试
    EnableEarlyDepthTest,
    /// 使用压缩纹理格式
    UseCompressedTextures,
    /// 减少Draw Calls
    ReduceDrawCalls,
    /// 使用实例化渲染
    UseInstancedRendering,
    /// 启用GPU剔除
    EnableGpuCulling,
    /// 自定义
    Custom,
}

impl fmt::Display for OptimizationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptimizationType::IncreaseWorkgroupSize => write!(f, "增加工作组大小"),
            OptimizationType::DecreaseWorkgroupSize => write!(f, "减少工作组大小"),
            OptimizationType::UseSharedMemory => write!(f, "使用共享内存"),
            OptimizationType::UseAtomicOperations => write!(f, "使用原子操作"),
            OptimizationType::UseVectorizedInstructions => write!(f, "使用向量化指令"),
            OptimizationType::EnableEarlyDepthTest => write!(f, "启用早期深度测试"),
            OptimizationType::UseCompressedTextures => write!(f, "使用压缩纹理格式"),
            OptimizationType::ReduceDrawCalls => write!(f, "减少Draw Calls"),
            OptimizationType::UseInstancedRendering => write!(f, "使用实例化渲染"),
            OptimizationType::EnableGpuCulling => write!(f, "启用GPU剔除"),
            OptimizationType::Custom => write!(f, "自定义优化"),
        }
    }
}

impl GpuCapabilities {
    /// 创建GPU能力探测器
    pub fn detect() -> Self {
        // 实际实现中，这里会使用wgpu适配器信息
        // 这里提供一个示例实现
        Self {
            vendor: GpuVendor::Unknown,
            architecture: GpuArchitecture::Unknown,
            device_name: "Unknown GPU".to_string(),
            driver_version: "0.0.0".to_string(),
            compute_capability: None,
            vram_size: 0,
            max_workgroup_size: 256,
            max_buffer_size: 256 * 1024 * 1024,
            supports_compute: true,
            supports_atomic: true,
            supports_float_atomic: false,
            supports_spirv: false,
            supports_dxc: false,
            recommended_workgroup_size: 64,
            physics_optimizations: vec![],
            particle_optimizations: vec![],
        }
    }

    /// 从设备信息创建（实际使用时通过wgpu获取）
    pub fn from_device_info(
        vendor: GpuVendor,
        architecture: GpuArchitecture,
        device_name: String,
        vram_size: u64,
    ) -> Self {
        let compute_capability = match architecture {
            GpuArchitecture::NvidiaAmpere => Some((8, 0)),
            GpuArchitecture::NvidiaTuring => Some((7, 5)),
            GpuArchitecture::NvidiaVolta => Some((7, 0)),
            GpuArchitecture::NvidiaPascal => Some((6, 1)),
            GpuArchitecture::NvidiaMaxwell => Some((5, 0)),
            GpuArchitecture::NvidiaKepler => Some((3, 0)),
            _ => None,
        };

        let (recommended_workgroup_size, physics_opts, particle_opts) =
            Self::generate_optimizations(vendor, architecture);

        Self {
            vendor,
            architecture,
            device_name,
            driver_version: "Unknown".to_string(),
            compute_capability,
            vram_size,
            max_workgroup_size: 1024, // 保守估计
            max_buffer_size: vram_size / 2,
            supports_compute: true,
            supports_atomic: true,
            supports_float_atomic: matches!(architecture, GpuArchitecture::NvidiaAmpere | GpuArchitecture::NvidiaTuring | GpuArchitecture::AmdRdna3 | GpuArchitecture::AmdRdna2),
            supports_spirv: matches!(vendor, GpuVendor::Amd | GpuVendor::Intel),
            supports_dxc: matches!(vendor, GpuVendor::Nvidia | GpuVendor::Intel | GpuVendor::Amd),
            recommended_workgroup_size,
            physics_optimizations: physics_opts,
            particle_optimizations: particle_opts,
        }
    }

    /// 生成特定GPU的优化建议
    fn generate_optimizations(
        vendor: GpuVendor,
        architecture: GpuArchitecture,
    ) -> (u32, Vec<OptimizationHint>, Vec<OptimizationHint>) {
        let mut physics_opts = vec![];
        let mut particle_opts = vec![];
        let recommended_wg = match vendor {
            GpuVendor::Nvidia => {
                // NVIDIA GPU优化
                match architecture {
                    GpuArchitecture::NvidiaAmpere | GpuArchitecture::NvidiaTuring => {
                        physics_opts.push(OptimizationHint {
                            hint_type: OptimizationType::IncreaseWorkgroupSize,
                            description: "使用128或256的工作组大小以充分利用Tensor Cores".to_string(),
                            estimated_improvement: 15.0,
                            difficulty: 2,
                            applied: false,
                        });
                        particle_opts.push(OptimizationHint {
                            hint_type: OptimizationType::UseVectorizedInstructions,
                            description: "使用向量化内存访问以提高带宽利用率".to_string(),
                            estimated_improvement: 20.0,
                            difficulty: 4,
                            applied: false,
                        });
                        128
                    }
                    _ => {
                        physics_opts.push(OptimizationHint {
                            hint_type: OptimizationType::UseSharedMemory,
                            description: "频繁访问的数据使用共享内存".to_string(),
                            estimated_improvement: 25.0,
                            difficulty: 6,
                            applied: false,
                        });
                        64
                    }
                }
            }
            GpuVendor::Amd => {
                // AMD GPU优化
                match architecture {
                    GpuArchitecture::AmdRdna3 | GpuArchitecture::AmdRdna2 => {
                        physics_opts.push(OptimizationHint {
                            hint_type: OptimizationType::UseAtomicOperations,
                            description: "RDNA2/3对原子操作有良好支持".to_string(),
                            estimated_improvement: 18.0,
                            difficulty: 3,
                            applied: false,
                        });
                        particle_opts.push(OptimizationHint {
                            hint_type: OptimizationType::IncreaseWorkgroupSize,
                            description: "RDNA架构在64-128工作组大小时效率最高".to_string(),
                            estimated_improvement: 12.0,
                            difficulty: 2,
                            applied: false,
                        });
                        64
                    }
                    _ => {
                        physics_opts.push(OptimizationHint {
                            hint_type: OptimizationType::UseVectorizedInstructions,
                            description: "GCN架构使用wavefront-wide操作".to_string(),
                            estimated_improvement: 20.0,
                            difficulty: 5,
                            applied: false,
                        });
                        64
                    }
                }
            }
            GpuVendor::Apple => {
                // Apple Silicon优化
                physics_opts.push(OptimizationHint {
                    hint_type: OptimizationType::DecreaseWorkgroupSize,
                    description: "Apple Silicon在较小工作组（32-64）时效率更高".to_string(),
                    estimated_improvement: 10.0,
                    difficulty: 2,
                    applied: false,
                });
                particle_opts.push(OptimizationHint {
                    hint_type: OptimizationType::UseAtomicOperations,
                    description: "利用Apple Silicon的高效原子操作".to_string(),
                    estimated_improvement: 15.0,
                    difficulty: 3,
                    applied: false,
                });
                32
            }
            GpuVendor::Intel => {
                // Intel GPU优化
                physics_opts.push(OptimizationHint {
                    hint_type: OptimizationType::IncreaseWorkgroupSize,
                    description: "Intel Xe GPU在较大工作组（64-128）时性能更好".to_string(),
                    estimated_improvement: 12.0,
                    difficulty: 2,
                    applied: false,
                });
                particle_opts.push(OptimizationHint {
                    hint_type: OptimizationType::UseCompressedTextures,
                    description: "使用压缩纹理减少带宽压力".to_string(),
                    estimated_improvement: 25.0,
                    difficulty: 3,
                    applied: false,
                });
                64
            }
            _ => {
                // 未知GPU，使用保守设置
                64
            }
        };

        (recommended_wg, physics_opts, particle_opts)
    }

    /// 是否为高性能GPU
    pub fn is_high_performance(&self) -> bool {
        matches!(
            self.architecture,
            GpuArchitecture::NvidiaAmpere
                | GpuArchitecture::NvidiaTuring
                | GpuArchitecture::NvidiaVolta
                | GpuArchitecture::AmdRdna3
                | GpuArchitecture::AmdRdna2
                | GpuArchitecture::AppleSilicon
        )
    }

    /// 是否支持CUDA特定优化
    pub fn supports_cuda_optimizations(&self) -> bool {
        matches!(self.vendor, GpuVendor::Nvidia)
            && self.compute_capability.is_some()
            && self.compute_capability.unwrap() >= (7, 0)
    }

    /// 是否支持ROCm特定优化
    pub fn supports_rocm_optimizations(&self) -> bool {
        matches!(self.vendor, GpuVendor::Amd)
            && matches!(
                self.architecture,
                GpuArchitecture::AmdRdna3
                    | GpuArchitecture::AmdRdna2
                    | GpuArchitecture::AmdCdna2
                    | GpuArchitecture::AmdCdna1
            )
    }

    /// 获取总体性能评分（0-100）
    pub fn get_performance_score(&self) -> u32 {
        let mut score = 50u32;

        // 基于架构评分
        score += match self.architecture {
            GpuArchitecture::NvidiaAmpere | GpuArchitecture::AmdRdna3 => 30,
            GpuArchitecture::NvidiaTuring | GpuArchitecture::AmdRdna2 => 25,
            GpuArchitecture::NvidiaVolta | GpuArchitecture::AmdCdna2 => 20,
            GpuArchitecture::NvidiaPascal | GpuArchitecture::AmdRdna1 | GpuArchitecture::AppleSilicon => 15,
            GpuArchitecture::NvidiaMaxwell | GpuArchitecture::AmdCdna1 => 10,
            GpuArchitecture::IntelXe | GpuArchitecture::IntelGen12 => 12,
            _ => 0,
        };

        // 基于VRAM评分
        if self.vram_size >= 8 * 1024 * 1024 * 1024 {
            score += 15;
        } else if self.vram_size >= 4 * 1024 * 1024 * 1024 {
            score += 10;
        } else if self.vram_size >= 2 * 1024 * 1024 * 1024 {
            score += 5;
        }

        // 基于特性支持评分
        if self.supports_float_atomic {
            score += 5;
        }
        if self.supports_compute {
            score += 5;
        }

        score.min(100)
    }

    /// 生成能力报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("═════════════════════════════════════════════════════\n");
        report.push_str("                    GPU能力报告\n");
        report.push_str("═════════════════════════════════════════════════════\n\n");

        report.push_str(&format!("📊 GPU信息:\n"));
        report.push_str(&format!("  厂商: {}\n", self.vendor));
        report.push_str(&format!("  架构: {}\n", self.architecture));
        report.push_str(&format!("  设备: {}\n", self.device_name));
        report.push_str(&format!("  驱动: {}\n", self.driver_version));
        report.push_str(&format!("  VRAM: {:.1} GB\n", self.vram_size as f64 / (1024.0 * 1024.0 * 1024.0)));

        if let Some((major, minor)) = self.compute_capability {
            report.push_str(&format!("  计算能力: {}.{}\n", major, minor));
        }

        report.push_str(&format!("\n🔧 特性支持:\n"));
        report.push_str(&format!("  计算着色器: {}\n", if self.supports_compute { "✅" } else { "❌" }));
        report.push_str(&format!("  原子操作: {}\n", if self.supports_atomic { "✅" } else { "❌" }));
        report.push_str(&format!("  浮点原子: {}\n", if self.supports_float_atomic { "✅" } else { "❌" }));
        report.push_str(&format!("  SPIR-V: {}\n", if self.supports_spirv { "✅" } else { "❌" }));
        report.push_str(&format!("  DXC: {}\n", if self.supports_dxc { "✅" } else { "❌" }));

        report.push_str(&format!("\n⚙️  性能参数:\n"));
        report.push_str(&format!("  最大工作组: {}\n", self.max_workgroup_size));
        report.push_str(&format!("  推荐工作组: {}\n", self.recommended_workgroup_size));
        report.push_str(&format!("  最大缓冲区: {:.1} MB\n", self.max_buffer_size as f64 / (1024.0 * 1024.0)));

        report.push_str(&format!("\n📈 性能评分: {}/100\n", self.get_performance_score()));

        if self.is_high_performance() {
            report.push_str("  状态: ✅ 高性能GPU\n");
        } else {
            report.push_str("  状态: ⚠️  中等性能GPU\n");
        }

        if self.supports_cuda_optimizations() {
            report.push_str("  CUDA优化: ✅ 支持\n");
        }
        if self.supports_rocm_optimizations() {
            report.push_str("  ROCm优化: ✅ 支持\n");
        }

        if !self.physics_optimizations.is_empty() {
            report.push_str(&format!("\n💡 物理模拟优化建议:\n"));
            for (i, hint) in self.physics_optimizations.iter().enumerate() {
                report.push_str(&format!("  {}. {} (预估提升: {:.1}%)\n", i + 1, hint.description, hint.estimated_improvement));
            }
        }

        if !self.particle_optimizations.is_empty() {
            report.push_str(&format!("\n✨ 粒子系统优化建议:\n"));
            for (i, hint) in self.particle_optimizations.iter().enumerate() {
                report.push_str(&format!("  {}. {} (预估提升: {:.1}%)\n", i + 1, hint.description, hint.estimated_improvement));
            }
        }

        report.push_str("\n═════════════════════════════════════════════════════\n");

        report
    }
}

impl fmt::Display for GpuCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.generate_report())
    }
}

impl Default for GpuCapabilities {
    fn default() -> Self {
        Self::detect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_capabilities_detection() {
        let caps = GpuCapabilities::detect();
        assert_eq!(caps.vendor, GpuVendor::Unknown);
        assert_eq!(caps.architecture, GpuArchitecture::Unknown);
    }

    #[test]
    fn test_nvidia_optimizations() {
        let caps = GpuCapabilities::from_device_info(
            GpuVendor::Nvidia,
            GpuArchitecture::NvidiaAmpere,
            "NVIDIA RTX 3080".to_string(),
            10 * 1024 * 1024 * 1024,
        );

        assert_eq!(caps.vendor, GpuVendor::Nvidia);
        assert_eq!(caps.architecture, GpuArchitecture::NvidiaAmpere);
        assert_eq!(caps.compute_capability, Some((8, 0)));
        assert!(caps.supports_cuda_optimizations());
        assert!(!caps.physics_optimizations.is_empty());
        assert!(caps.is_high_performance());
        assert!(caps.get_performance_score() > 80);
    }

    #[test]
    fn test_amd_optimizations() {
        let caps = GpuCapabilities::from_device_info(
            GpuVendor::Amd,
            GpuArchitecture::AmdRdna2,
            "AMD RX 6800 XT".to_string(),
            16 * 1024 * 1024 * 1024,
        );

        assert_eq!(caps.vendor, GpuVendor::Amd);
        assert_eq!(caps.architecture, GpuArchitecture::AmdRdna2);
        assert!(caps.supports_rocm_optimizations());
        assert!(!caps.physics_optimizations.is_empty());
        assert!(caps.is_high_performance());
    }

    #[test]
    fn test_apple_silicon() {
        let caps = GpuCapabilities::from_device_info(
            GpuVendor::Apple,
            GpuArchitecture::AppleSilicon,
            "Apple M2 GPU".to_string(),
            8 * 1024 * 1024 * 1024,
        );

        assert_eq!(caps.vendor, GpuVendor::Apple);
        assert_eq!(caps.recommended_workgroup_size, 32);
        assert!(caps.is_high_performance());
    }

    #[test]
    fn test_performance_score() {
        let ampere = GpuCapabilities::from_device_info(
            GpuVendor::Nvidia,
            GpuArchitecture::NvidiaAmpere,
            "RTX 3080".to_string(),
            10 * 1024 * 1024 * 1024,
        );
        assert!(ampere.get_performance_score() > 80);

        let pascal = GpuCapabilities::from_device_info(
            GpuVendor::Nvidia,
            GpuArchitecture::NvidiaPascal,
            "GTX 1080".to_string(),
            8 * 1024 * 1024 * 1024,
        );
        assert!(pascal.get_performance_score() > 60);
        assert!(pascal.get_performance_score() < ampere.get_performance_score());
    }

    #[test]
    fn test_optimization_generation() {
        let caps = GpuCapabilities::from_device_info(
            GpuVendor::Nvidia,
            GpuArchitecture::NvidiaAmpere,
            "RTX 3080".to_string(),
            10 * 1024 * 1024 * 1024,
        );

        assert!(!caps.physics_optimizations.is_empty());
        assert!(!caps.particle_optimizations.is_empty());

        for hint in &caps.physics_optimizations {
            assert!(!hint.description.is_empty());
            assert!(hint.estimated_improvement > 0.0);
            assert!(hint.difficulty > 0 && hint.difficulty <= 10);
        }
    }
}
