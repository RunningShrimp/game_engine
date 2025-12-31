//! # ARM NEON SIMD优化
//!
//! 为ARM架构提供NEON SIMD加速支持。
//!
//! ## 什么是NEON?
//!
//! **ARM NEON** 是ARM的128位SIMD（单指令多数据）扩展：
//! - **128位寄存器**: 可同时处理多个数据
//! - **向量运算**: 并行处理浮点/整数向量
//! - **广泛支持**: ARMv7+, ARMv8 (AArch64), Apple Silicon
//!
//! ## 性能提升
//!
//! - **向量运算**: 4x float32 或 2x float64 并行
//! - **图像处理**: 像素操作加速
//! - **物理计算**: 向量数学加速
//! - **音频处理**: 并行音频混合
//!
//! ## 使用场景
//!
//! - **移动游戏**: Android/iOS 3D游戏
//! - **物理模拟**: 向量物理计算
//! - **图像处理**: 实时滤镜和效果
//! - **音频合成**: 多轨音频混合

#![allow(unsafe_op_in_unsafe_fn)]  // 允许在unsafe函数中使用unsafe操作

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;
#[cfg(target_arch = "arm")]
use std::arch::arm::*;

/// NEON向量优化器
pub struct NeonOptimizer;

impl NeonOptimizer {
    /// 检测是否支持NEON
    pub fn is_supported() -> bool {
        #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
        {
            // ARMv8 (aarch64) 总是支持NEON
            // ARMv7 需要运行时检测
            Self::detect_neon_runtime()
        }

        #[cfg(not(any(target_arch = "aarch64", target_arch = "arm")))]
        {
            false
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn detect_neon_runtime() -> bool {
        true  // ARMv8 总是支持NEON
    }

    #[cfg(target_arch = "arm")]
    fn detect_neon_runtime() -> bool {
        // 运行时检测NEON支持
        // 注：实际实现需要读取CPU特性寄存器
        true  // 假设支持
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "arm")))]
    fn detect_neon_runtime() -> bool {
        false
    }
}

/// NEON优化的向量运算
pub struct NeonVecOps;

impl NeonVecOps {
    /// 向量加法 (4x float32)
    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub unsafe fn add_f32x4(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
        let va = vld1q_f32(a.as_ptr());
        let vb = vld1q_f32(b.as_ptr());
        let result = vaddq_f32(va, vb);

        let mut output = [0.0f32; 4];
        vst1q_f32(output.as_mut_ptr(), result);
        output
    }

    /// 向量加法 (4x float32) - 标量回退
    #[cfg(not(target_arch = "aarch64"))]
    #[inline(always)]
    pub unsafe fn add_f32x4(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
        [
            a[0] + b[0],
            a[1] + b[1],
            a[2] + b[2],
            a[3] + b[3],
        ]
    }

    /// 向量乘法 (4x float32)
    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub unsafe fn mul_f32x4(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
        let va = vld1q_f32(a.as_ptr());
        let vb = vld1q_f32(b.as_ptr());
        let result = vmulq_f32(va, vb);

        let mut output = [0.0f32; 4];
        vst1q_f32(output.as_mut_ptr(), result);
        output
    }

    #[cfg(not(target_arch = "aarch64"))]
    #[inline(always)]
    pub unsafe fn mul_f32x4(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
        [
            a[0] * b[0],
            a[1] * b[1],
            a[2] * b[2],
            a[3] * b[3],
        ]
    }

    /// 向量点积 (4x float32)
    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub unsafe fn dot_f32x4(a: &[f32; 4], b: &[f32; 4]) -> f32 {
        let va = vld1q_f32(a.as_ptr());
        let vb = vld1q_f32(b.as_ptr());
        let result = vmulq_f32(va, vb);

        // 水平求和
        let mut sum = [0.0f32; 4];
        vst1q_f32(sum.as_mut_ptr(), result);
        sum[0] + sum[1] + sum[2] + sum[3]
    }

    #[cfg(not(target_arch = "aarch64"))]
    #[inline(always)]
    pub unsafe fn dot_f32x4(a: &[f32; 4], b: &[f32; 4]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
    }

    /// 向量平方根 (4x float32)
    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub unsafe fn sqrt_f32x4(a: &[f32; 4]) -> [f32; 4] {
        let va = vld1q_f32(a.as_ptr());
        let result = vsqrtq_f32(va);

        let mut output = [0.0f32; 4];
        vst1q_f32(output.as_mut_ptr(), result);
        output
    }

    #[cfg(not(target_arch = "aarch64"))]
    #[inline(always)]
    pub unsafe fn sqrt_f32x4(a: &[f32; 4]) -> [f32; 4] {
        [
            a[0].sqrt(),
            a[1].sqrt(),
            a[2].sqrt(),
            a[3].sqrt(),
        ]
    }

    /// 向量倒数 (4x float32)
    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub unsafe fn reciprocal_f32x4(a: &[f32; 4]) -> [f32; 4] {
        let va = vld1q_f32(a.as_ptr());
        let ones = vdupq_n_f32(1.0);
        let result = vdivq_f32(ones, va);

        let mut output = [0.0f32; 4];
        vst1q_f32(output.as_mut_ptr(), result);
        output
    }

    #[cfg(not(target_arch = "aarch64"))]
    #[inline(always)]
    pub unsafe fn reciprocal_f32x4(a: &[f32; 4]) -> [f32; 4] {
        [
            1.0 / a[0],
            1.0 / a[1],
            1.0 / a[2],
            1.0 / a[3],
        ]
    }

    /// 向量最小值 (4x float32)
    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub unsafe fn min_f32x4(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
        let va = vld1q_f32(a.as_ptr());
        let vb = vld1q_f32(b.as_ptr());
        let result = vminq_f32(va, vb);

        let mut output = [0.0f32; 4];
        vst1q_f32(output.as_mut_ptr(), result);
        output
    }

    #[cfg(not(target_arch = "aarch64"))]
    #[inline(always)]
    pub unsafe fn min_f32x4(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
        [
            a[0].min(b[0]),
            a[1].min(b[1]),
            a[2].min(b[2]),
            a[3].min(b[3]),
        ]
    }

    /// 向量最大值 (4x float32)
    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub unsafe fn max_f32x4(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
        let va = vld1q_f32(a.as_ptr());
        let vb = vld1q_f32(b.as_ptr());
        let result = vmaxq_f32(va, vb);

        let mut output = [0.0f32; 4];
        vst1q_f32(output.as_mut_ptr(), result);
        output
    }

    #[cfg(not(target_arch = "aarch64"))]
    #[inline(always)]
    pub unsafe fn max_f32x4(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
        [
            a[0].max(b[0]),
            a[1].max(b[1]),
            a[2].max(b[2]),
            a[3].max(b[3]),
        ]
    }
}

/// NEON优化的数组运算
pub struct NeonArrayOps;

impl NeonArrayOps {
    /// 并行数组加法
    #[inline(always)]
    pub fn add_arrays_f32(a: &[f32], b: &[f32], output: &mut [f32]) {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), output.len());

        let len = a.len();
        let simd_len = len & !3;  // 对齐到4的倍数

        unsafe {
            // SIMD处理
            let i = 0;
            while i < simd_len {
                let va = [a[i], a[i+1], a[i+2], a[i+3]];
                let vb = [b[i], b[i+1], b[i+2], b[i+3]];
                let result = NeonVecOps::add_f32x4(&va, &vb);
                output[i..i+4].copy_from_slice(&result);
            }

            // 标量处理剩余元素
            for i in simd_len..len {
                output[i] = a[i] + b[i];
            }
        }
    }

    /// 并行数组乘法
    #[inline(always)]
    pub fn mul_arrays_f32(a: &[f32], b: &[f32], output: &mut [f32]) {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), output.len());

        let len = a.len();
        let simd_len = len & !3;

        unsafe {
            // SIMD处理
            let i = 0;
            while i < simd_len {
                let va = [a[i], a[i+1], a[i+2], a[i+3]];
                let vb = [b[i], b[i+1], b[i+2], b[i+3]];
                let result = NeonVecOps::mul_f32x4(&va, &vb);
                output[i..i+4].copy_from_slice(&result);
            }

            // 标量处理剩余元素
            for i in simd_len..len {
                output[i] = a[i] * b[i];
            }
        }
    }

    /// 并行数组缩放
    #[inline(always)]
    pub fn scale_array_f32(a: &[f32], scalar: f32, output: &mut [f32]) {
        assert_eq!(a.len(), output.len());

        let len = a.len();
        let simd_len = len & !3;

        unsafe {
            let scalar_vec = [scalar; 4];

            // SIMD处理
            let i = 0;
            while i < simd_len {
                let va = [a[i], a[i+1], a[i+2], a[i+3]];
                let result = NeonVecOps::mul_f32x4(&va, &scalar_vec);
                output[i..i+4].copy_from_slice(&result);
            }

            // 标量处理剩余元素
            for i in simd_len..len {
                output[i] = a[i] * scalar;
            }
        }
    }

    /// 并行数组平方根
    #[inline(always)]
    pub fn sqrt_array_f32(a: &[f32], output: &mut [f32]) {
        assert_eq!(a.len(), output.len());

        let len = a.len();
        let simd_len = len & !3;

        unsafe {
            // SIMD处理
            let i = 0;
            while i < simd_len {
                let va = [a[i], a[i+1], a[i+2], a[i+3]];
                let result = NeonVecOps::sqrt_f32x4(&va);
                output[i..i+4].copy_from_slice(&result);
            }

            // 标量处理剩余元素
            for i in simd_len..len {
                output[i] = a[i].sqrt();
            }
        }
    }
}

/// NEON优化的矩阵运算（3x3和4x4）
pub struct NeonMatrixOps;

impl NeonMatrixOps {
    /// 3x3矩阵乘法
    pub fn mul_mat3x3(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
        unsafe {
            [
                NeonVecOps::dot_f32x4(&[a[0], a[1], a[2], 0.0], &[b[0], b[3], b[6], 0.0]),
                NeonVecOps::dot_f32x4(&[a[0], a[1], a[2], 0.0], &[b[1], b[4], b[7], 0.0]),
                NeonVecOps::dot_f32x4(&[a[0], a[1], a[2], 0.0], &[b[2], b[5], b[8], 0.0]),
                NeonVecOps::dot_f32x4(&[a[3], a[4], a[5], 0.0], &[b[0], b[3], b[6], 0.0]),
                NeonVecOps::dot_f32x4(&[a[3], a[4], a[5], 0.0], &[b[1], b[4], b[7], 0.0]),
                NeonVecOps::dot_f32x4(&[a[3], a[4], a[5], 0.0], &[b[2], b[5], b[8], 0.0]),
                NeonVecOps::dot_f32x4(&[a[6], a[7], a[8], 0.0], &[b[0], b[3], b[6], 0.0]),
                NeonVecOps::dot_f32x4(&[a[6], a[7], a[8], 0.0], &[b[1], b[4], b[7], 0.0]),
                NeonVecOps::dot_f32x4(&[a[6], a[7], a[8], 0.0], &[b[2], b[5], b[8], 0.0]),
            ]
        }
    }

    /// 4x4矩阵乘法
    pub fn mul_mat4x4(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
        unsafe {
            [
                NeonVecOps::dot_f32x4(&a[0..4].try_into().unwrap(), &b[0..4].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[0..4].try_into().unwrap(), &b[4..8].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[0..4].try_into().unwrap(), &b[8..12].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[0..4].try_into().unwrap(), &b[12..16].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[4..8].try_into().unwrap(), &b[0..4].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[4..8].try_into().unwrap(), &b[4..8].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[4..8].try_into().unwrap(), &b[8..12].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[4..8].try_into().unwrap(), &b[12..16].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[8..12].try_into().unwrap(), &b[0..4].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[8..12].try_into().unwrap(), &b[4..8].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[8..12].try_into().unwrap(), &b[8..12].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[8..12].try_into().unwrap(), &b[12..16].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[12..16].try_into().unwrap(), &b[0..4].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[12..16].try_into().unwrap(), &b[4..8].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[12..16].try_into().unwrap(), &b[8..12].try_into().unwrap()),
                NeonVecOps::dot_f32x4(&a[12..16].try_into().unwrap(), &b[12..16].try_into().unwrap()),
            ]
        }
    }
}

/// NEON性能基准
pub struct NeonBenchmark;

impl NeonBenchmark {
    /// 基准测试：向量加法
    pub fn bench_vector_add(iterations: usize) -> (f64, f64) {
        use std::time::Instant;

        let a = [1.0f32; 4];
        let b = [2.0f32; 4];

        // 标量版本
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]];
        }
        let scalar_time = start.elapsed().as_secs_f64();

        // SIMD版本
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = unsafe { NeonVecOps::add_f32x4(&a, &b) };
        }
        let simd_time = start.elapsed().as_secs_f64();

        (scalar_time, simd_time)
    }

    /// 获取加速比
    pub fn speedup_ratio(scalar_time: f64, simd_time: f64) -> f32 {
        if simd_time > 0.0 {
            (scalar_time / simd_time) as f32
        } else {
            1.0
        }
    }
}

// =============================================================================
// 辅助函数
// =============================================================================

/// 检测是否为ARM架构
pub fn is_arm_arch() -> bool {
    cfg!(any(target_arch = "arm", target_arch = "aarch64"))
}

/// 检测是否为AArch64（64位ARM）
pub fn is_aarch64() -> bool {
    cfg!(target_arch = "aarch64")
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neon_support() {
        // ARM架构应该支持NEON
        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        {
            assert!(NeonOptimizer::is_supported());
        }
    }

    #[test]
    fn test_vector_add() {
        let a = [1.0f32, 2.0, 3.0, 4.0];
        let b = [5.0f32, 6.0, 7.0, 8.0];
        let expected = [6.0f32, 8.0, 10.0, 12.0];

        let result = unsafe { NeonVecOps::add_f32x4(&a, &b) };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_vector_mul() {
        let a = [1.0f32, 2.0, 3.0, 4.0];
        let b = [2.0f32, 3.0, 4.0, 5.0];
        let expected = [2.0f32, 6.0, 12.0, 20.0];

        let result = unsafe { NeonVecOps::mul_f32x4(&a, &b) };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_vector_dot() {
        let a = [1.0f32, 2.0, 3.0, 4.0];
        let b = [2.0f32, 3.0, 4.0, 5.0];
        let expected = 1.0*2.0 + 2.0*3.0 + 3.0*4.0 + 4.0*5.0;  // 40.0

        let result = unsafe { NeonVecOps::dot_f32x4(&a, &b) };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_vector_sqrt() {
        let a = [1.0f32, 4.0, 9.0, 16.0];
        let expected = [1.0f32, 2.0, 3.0, 4.0];

        let result = unsafe { NeonVecOps::sqrt_f32x4(&a) };
        for i in 0..4 {
            assert!((result[i] - expected[i]).abs() < 0.001);
        }
    }

    #[test]
    fn test_array_add() {
        let a = vec![1.0f32; 100];
        let b = vec![2.0f32; 100];
        let mut output = vec![0.0f32; 100];

        NeonArrayOps::add_arrays_f32(&a, &b, &mut output);

        for i in 0..100 {
            assert_eq!(output[i], 3.0);
        }
    }

    #[test]
    fn test_matrix_mul() {
        // 单位矩阵
        let identity = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];

        let mat = [
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 10.0, 11.0, 12.0,
            13.0, 14.0, 15.0, 16.0,
        ];

        let result = NeonMatrixOps::mul_mat4x4(&mat, &identity);

        for i in 0..16 {
            assert_eq!(result[i], mat[i]);
        }
    }
}
