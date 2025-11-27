/// SIMD优化模块
/// 
/// 为主流芯片提供SIMD和特有指令集优化，包括：
/// - Intel/AMD: SSE2, SSE3, SSE4.1, SSE4.2, AVX, AVX2, AVX-512
/// - Apple M系列: ARM NEON, SVE
/// - 华为麒麟: ARM NEON
/// - 高通骁龙: ARM NEON, Hexagon DSP
/// - 联发科: ARM NEON

pub mod cpu_detect;
pub mod math;
pub mod batch;

// 重新导出主要类型
pub use cpu_detect::{CpuFeatures, detect_cpu_features, print_cpu_info};
pub use math::{Vec3Simd, Vec4Simd, Mat4Simd, QuatSimd};

/// SIMD向量宽度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdWidth {
    /// 128位 (SSE, NEON)
    W128,
    /// 256位 (AVX, AVX2)
    W256,
    /// 512位 (AVX-512)
    W512,
}

/// SIMD后端类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdBackend {
    /// 标量回退实现（无SIMD）
    Scalar,
    /// SSE2 (Intel/AMD)
    Sse2,
    /// SSE4.1 (Intel/AMD)
    Sse41,
    /// AVX (Intel/AMD)
    Avx,
    /// AVX2 (Intel/AMD)
    Avx2,
    /// AVX-512 (Intel/AMD高端)
    Avx512,
    /// ARM NEON (Apple M系列, 麒麟, 高通, 联发科)
    Neon,
    /// ARM SVE (Apple M系列, ARM v9)
    Sve,
}

impl SimdBackend {
    /// 获取当前平台最优的SIMD后端
    pub fn best_available() -> Self {
        let features = detect_cpu_features();
        
        #[cfg(target_arch = "x86_64")]
        {
            if features.avx512f {
                return Self::Avx512;
            }
            if features.avx2 {
                return Self::Avx2;
            }
            if features.avx {
                return Self::Avx;
            }
            if features.sse41 {
                return Self::Sse41;
            }
            if features.sse2 {
                return Self::Sse2;
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            if features.sve {
                return Self::Sve;
            }
            if features.neon {
                return Self::Neon;
            }
        }
        
        Self::Scalar
    }
    
    /// 获取SIMD向量宽度
    pub fn width(&self) -> SimdWidth {
        match self {
            Self::Scalar | Self::Sse2 | Self::Sse41 | Self::Neon => SimdWidth::W128,
            Self::Avx | Self::Avx2 => SimdWidth::W256,
            Self::Avx512 | Self::Sve => SimdWidth::W512,
        }
    }
    
    /// 获取可以并行处理的f32数量
    pub fn f32_lanes(&self) -> usize {
        match self.width() {
            SimdWidth::W128 => 4,
            SimdWidth::W256 => 8,
            SimdWidth::W512 => 16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_detection() {
        let backend = SimdBackend::best_available();
        println!("Detected SIMD backend: {:?}", backend);
        assert_ne!(backend, SimdBackend::Scalar); // 现代CPU应该至少支持SSE2或NEON
    }

    #[test]
    fn test_backend_lanes() {
        assert_eq!(SimdBackend::Sse2.f32_lanes(), 4);
        assert_eq!(SimdBackend::Avx2.f32_lanes(), 8);
        assert_eq!(SimdBackend::Avx512.f32_lanes(), 16);
        assert_eq!(SimdBackend::Neon.f32_lanes(), 4);
    }
}
