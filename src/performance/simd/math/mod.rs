/// SIMD优化的数学运算模块

#[cfg(target_arch = "x86_64")]
pub mod x86;

#[cfg(target_arch = "aarch64")]
pub mod arm;

mod scalar;
mod dispatch;

pub use dispatch::{Vec3Simd, Vec4Simd, Mat4Simd, QuatSimd};

/// 向量运算trait
pub trait VectorOps {
    fn dot(&self, other: &Self) -> f32;
    fn add(&self, other: &Self) -> Self;
    fn sub(&self, other: &Self) -> Self;
    fn mul(&self, scalar: f32) -> Self;
    fn length(&self) -> f32;
    fn normalize(&self) -> Self;
}

/// 矩阵运算trait
pub trait MatrixOps {
    fn mul(&self, other: &Self) -> Self;
    fn transpose(&self) -> Self;
    fn transform_vec4(&self, vec: &[f32; 4]) -> [f32; 4];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec4_operations() {
        let a = Vec4Simd::new(1.0, 2.0, 3.0, 4.0);
        let b = Vec4Simd::new(5.0, 6.0, 7.0, 8.0);
        
        let dot = a.dot(&b);
        assert!((dot - 70.0).abs() < 1e-5);
        
        let sum = a.add(&b);
        assert!((sum.data[0] - 6.0).abs() < 1e-5);
    }
}
