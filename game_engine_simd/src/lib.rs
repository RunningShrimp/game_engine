//! # game_engine_simd
//!
//! 高性能SIMD优化库，为游戏引擎提供跨平台的向量化数学运算和CPU特性检测。
//!
//! ## 特性
//!
//! - **跨平台支持**: x86_64 (SSE2/SSE4.1/AVX/AVX2/AVX-512) 和 aarch64 (NEON/SVE)
//! - **自动检测**: 运行时检测CPU特性，选择最优SIMD后端
//! - **批量处理**: 优化的批量变换、蒙皮、粒子系统处理
//! - **零成本抽象**: 提供高级API，自动选择最优实现
//!
//! ## 快速开始
//!
//! ```rust
//! use game_engine_simd::{detect_cpu_features, Vec4Simd, SimdBackend};
//!
//! // 检测CPU特性
//! let features = detect_cpu_features();
//! println!("AVX2支持: {}", features.avx2);
//!
//! // 使用SIMD向量运算
//! let a = Vec4Simd::new(1.0, 2.0, 3.0, 4.0);
//! let b = Vec4Simd::new(5.0, 6.0, 7.0, 8.0);
//! let dot = a.dot(&b);
//!
//! // 获取最优后端
//! let backend = SimdBackend::best_available();
//! println!("使用后端: {:?}", backend);
//! ```
//!
//! ## 模块
//!
//! - [`cpu_detect`]: CPU特性检测
//! - [`math`]: SIMD数学运算（Vec3/Vec4/Mat4/Quat）
//! - [`batch`]: 批量处理优化（变换、蒙皮、粒子）
//!
//! ## 性能
//!
//! 相比标量实现，典型性能提升：
//! - 向量运算: 2-4x
//! - 矩阵运算: 3-6x
//! - 批量变换: 4-8x
//!
//! ## 示例
//!
//! ### CPU特性检测
//!
//! ```rust
//! use game_engine_simd::{detect_cpu_features, print_cpu_info};
//!
//! // 检测CPU特性
//! let features = detect_cpu_features();
//! println!("CPU厂商: {:?}", features.vendor);
//! println!("AVX2支持: {}", features.avx2);
//!
//! // 打印详细信息
//! print_cpu_info();
//! ```
//!
//! ### SIMD向量运算
//!
//! ```rust
//! use game_engine_simd::Vec4Simd;
//!
//! let a = Vec4Simd::new(1.0, 2.0, 3.0, 4.0);
//! let b = Vec4Simd::new(5.0, 6.0, 7.0, 8.0);
//!
//! // 点积
//! let dot = a.dot(&b);
//!
//! // 向量加法
//! let sum = a.add(&b);
//!
//! // 归一化
//! let normalized = a.normalize();
//! ```
//!
//! ### 批量处理
//!
//! ```rust
//! use game_engine_simd::{BatchConfig, batch::BatchTransform};
//!
//! let config = BatchConfig::default();
//! let mut batch_transform = BatchTransform::new(config);
//!
//! // 批量变换顶点
//! let mut vertices = vec![[0.0f32; 3]; 1000];
//! let matrices = vec![[[1.0f32; 4]; 4]; 1000];
//! let stats = batch_transform.transform_vertices(&mut vertices, &matrices);
//!
//! println!("处理了 {} 个顶点", stats.elements_processed);
//! println!("吞吐量: {:.2} 顶点/秒", stats.throughput());
//! ```

pub mod cpu_detect;
pub mod math;
pub mod batch;
pub mod audio;

// 重新导出主要类型
pub use cpu_detect::{CpuFeatures, CpuVendor, detect_cpu_features, print_cpu_info};
pub use math::{Vec3Simd, Vec4Simd, Mat4Simd, QuatSimd, MatrixBatchOps, VectorBatchOps, GeometryOps, TransformOps, PerformanceTest, VectorBatchResult};
pub use batch::{BatchConfig, BatchStats};
pub use audio::{AudioSpatialOps, AudioDSPOps, AudioSpatialResult, AudioDSPResult, DistanceModel};

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
///
/// 表示可用的SIMD指令集后端，按性能从低到高排序。
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
    ///
    /// 根据运行时检测的CPU特性，返回当前平台支持的最高性能SIMD后端。
    ///
    /// # 返回
    ///
    /// 最优的SIMD后端，如果平台不支持任何SIMD指令集则返回`Scalar`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine_simd::SimdBackend;
    ///
    /// let backend = SimdBackend::best_available();
    /// println!("使用后端: {:?}", backend);
    /// ```
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
    ///
    /// # 返回
    ///
    /// SIMD向量宽度枚举值。
    pub fn width(&self) -> SimdWidth {
        match self {
            Self::Scalar | Self::Sse2 | Self::Sse41 | Self::Neon => SimdWidth::W128,
            Self::Avx | Self::Avx2 => SimdWidth::W256,
            Self::Avx512 | Self::Sve => SimdWidth::W512,
        }
    }
    
    /// 获取可以并行处理的f32数量
    ///
    /// # 返回
    ///
    /// 可以并行处理的f32数量（通道数）。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine_simd::SimdBackend;
    ///
    /// assert_eq!(SimdBackend::Sse2.f32_lanes(), 4);
    /// assert_eq!(SimdBackend::Avx2.f32_lanes(), 8);
    /// assert_eq!(SimdBackend::Avx512.f32_lanes(), 16);
    /// ```
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
        // 现代CPU应该至少支持SSE2或NEON
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        assert_ne!(backend, SimdBackend::Scalar);
    }

    #[test]
    fn test_backend_lanes() {
        assert_eq!(SimdBackend::Sse2.f32_lanes(), 4);
        assert_eq!(SimdBackend::Avx2.f32_lanes(), 8);
        assert_eq!(SimdBackend::Avx512.f32_lanes(), 16);
        assert_eq!(SimdBackend::Neon.f32_lanes(), 4);
    }
}
