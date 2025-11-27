/// 运行时动态分发系统
/// 
/// 根据CPU特性自动选择最优的SIMD实现

use super::scalar::*;
use super::VectorOps;
use crate::performance::simd::SimdBackend;

#[cfg(target_arch = "x86_64")]
use super::x86::*;

#[cfg(target_arch = "aarch64")]
use super::arm::*;

/// 4维向量（自动SIMD优化）
#[derive(Debug, Clone, Copy)]
pub struct Vec4Simd {
    pub data: [f32; 4],
}

impl Vec4Simd {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { data: [x, y, z, w] }
    }
    
    pub fn zero() -> Self {
        Self { data: [0.0; 4] }
    }
    
    pub fn x(&self) -> f32 { self.data[0] }
    pub fn y(&self) -> f32 { self.data[1] }
    pub fn z(&self) -> f32 { self.data[2] }
    pub fn w(&self) -> f32 { self.data[3] }
}

impl VectorOps for Vec4Simd {
    fn dot(&self, other: &Self) -> f32 {
        let backend = SimdBackend::best_available();
        
        #[cfg(target_arch = "x86_64")]
        {
            match backend {
                SimdBackend::Avx512 | SimdBackend::Avx2 | SimdBackend::Avx => {
                    unsafe {
                        if is_x86_feature_detected!("sse4.1") {
                            return dot_product_sse41(&self.data, &other.data);
                        }
                    }
                }
                SimdBackend::Sse41 => {
                    unsafe {
                        if is_x86_feature_detected!("sse4.1") {
                            return dot_product_sse41(&self.data, &other.data);
                        }
                    }
                }
                SimdBackend::Sse2 => {
                    unsafe {
                        if is_x86_feature_detected!("sse2") {
                            return dot_product_sse2(&self.data, &other.data);
                        }
                    }
                }
                _ => {}
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            if backend == SimdBackend::Neon || backend == SimdBackend::Sve {
                unsafe {
                    return dot_product_neon(&self.data, &other.data);
                }
            }
        }
        
        // 标量回退
        dot_product_scalar(&self.data, &other.data)
    }
    
    fn add(&self, other: &Self) -> Self {
        let mut result = Self::zero();
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("sse2") {
                unsafe {
                    use std::arch::x86_64::*;
                    let va = _mm_loadu_ps(self.data.as_ptr());
                    let vb = _mm_loadu_ps(other.data.as_ptr());
                    let vr = _mm_add_ps(va, vb);
                    _mm_storeu_ps(result.data.as_mut_ptr(), vr);
                    return result;
                }
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            unsafe {
                add_vec4_neon(&self.data, &other.data, &mut result.data);
                return result;
            }
        }
        
        add_vec4_scalar(&self.data, &other.data, &mut result.data);
        result
    }
    
    fn sub(&self, other: &Self) -> Self {
        let mut result = Self::zero();
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("sse2") {
                unsafe {
                    use std::arch::x86_64::*;
                    let va = _mm_loadu_ps(self.data.as_ptr());
                    let vb = _mm_loadu_ps(other.data.as_ptr());
                    let vr = _mm_sub_ps(va, vb);
                    _mm_storeu_ps(result.data.as_mut_ptr(), vr);
                    return result;
                }
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            unsafe {
                sub_vec4_neon(&self.data, &other.data, &mut result.data);
                return result;
            }
        }
        
        sub_vec4_scalar(&self.data, &other.data, &mut result.data);
        result
    }
    
    fn mul(&self, scalar: f32) -> Self {
        let mut result = Self::zero();
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("sse2") {
                unsafe {
                    use std::arch::x86_64::*;
                    let va = _mm_loadu_ps(self.data.as_ptr());
                    let vs = _mm_set1_ps(scalar);
                    let vr = _mm_mul_ps(va, vs);
                    _mm_storeu_ps(result.data.as_mut_ptr(), vr);
                    return result;
                }
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            unsafe {
                use std::arch::aarch64::*;
                let va = vld1q_f32(self.data.as_ptr());
                let vs = vdupq_n_f32(scalar);
                let vr = vmulq_f32(va, vs);
                vst1q_f32(result.data.as_mut_ptr(), vr);
                return result;
            }
        }
        
        mul_vec4_scalar(&self.data, scalar, &mut result.data);
        result
    }
    
    fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }
    
    fn normalize(&self) -> Self {
        let len = self.length();
        if len > 1e-6 {
            self.mul(1.0 / len)
        } else {
            *self
        }
    }
}

/// 3维向量（自动SIMD优化）
#[derive(Debug, Clone, Copy)]
pub struct Vec3Simd {
    pub data: [f32; 3],
}

impl Vec3Simd {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { data: [x, y, z] }
    }
    
    pub fn zero() -> Self {
        Self { data: [0.0; 3] }
    }
    
    pub fn cross(&self, other: &Self) -> Self {
        let mut result = Self::zero();
        
        #[cfg(target_arch = "aarch64")]
        {
            unsafe {
                cross_product_neon(&self.data, &other.data, &mut result.data);
                return result;
            }
        }
        
        // 标量实现
        result.data[0] = self.data[1] * other.data[2] - self.data[2] * other.data[1];
        result.data[1] = self.data[2] * other.data[0] - self.data[0] * other.data[2];
        result.data[2] = self.data[0] * other.data[1] - self.data[1] * other.data[0];
        result
    }
    
    pub fn x(&self) -> f32 { self.data[0] }
    pub fn y(&self) -> f32 { self.data[1] }
    pub fn z(&self) -> f32 { self.data[2] }
}

/// 4x4矩阵（自动SIMD优化）
#[derive(Debug, Clone, Copy)]
pub struct Mat4Simd {
    pub data: [[f32; 4]; 4],
}

impl Mat4Simd {
    pub fn identity() -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    
    pub fn zero() -> Self {
        Self {
            data: [[0.0; 4]; 4],
        }
    }
    
    pub fn mul(&self, other: &Self) -> Self {
        let mut result = Self::zero();
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx") {
                unsafe {
                    mat4_mul_avx(&self.data, &other.data, &mut result.data);
                    return result;
                }
            }
            if is_x86_feature_detected!("sse2") {
                unsafe {
                    mat4_mul_sse2(&self.data, &other.data, &mut result.data);
                    return result;
                }
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            unsafe {
                mat4_mul_neon(&self.data, &other.data, &mut result.data);
                return result;
            }
        }
        
        mat4_mul_scalar(&self.data, &other.data, &mut result.data);
        result
    }
    
    pub fn transform(&self, vec: &Vec4Simd) -> Vec4Simd {
        let mut result = Vec4Simd::zero();
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("sse2") {
                unsafe {
                    transform_vectors_sse2(&self.data, &[vec.data], &mut [result.data]);
                    return result;
                }
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            unsafe {
                transform_vectors_neon(&self.data, &[vec.data], &mut [result.data]);
                return result;
            }
        }
        
        // 标量实现
        for i in 0..4 {
            result.data[i] = self.data[i][0] * vec.data[0]
                           + self.data[i][1] * vec.data[1]
                           + self.data[i][2] * vec.data[2]
                           + self.data[i][3] * vec.data[3];
        }
        result
    }
}

/// 四元数（自动SIMD优化）
#[derive(Debug, Clone, Copy)]
pub struct QuatSimd {
    pub data: [f32; 4], // [w, x, y, z]
}

impl QuatSimd {
    pub fn identity() -> Self {
        Self { data: [1.0, 0.0, 0.0, 0.0] }
    }
    
    pub fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { data: [w, x, y, z] }
    }
    
    pub fn mul(&self, other: &Self) -> Self {
        let mut result = Self::identity();
        
        #[cfg(target_arch = "aarch64")]
        {
            unsafe {
                quat_mul_neon(&self.data, &other.data, &mut result.data);
                return result;
            }
        }
        
        // 标量实现
        result.data[0] = self.data[0]*other.data[0] - self.data[1]*other.data[1] 
                       - self.data[2]*other.data[2] - self.data[3]*other.data[3];
        result.data[1] = self.data[0]*other.data[1] + self.data[1]*other.data[0] 
                       + self.data[2]*other.data[3] - self.data[3]*other.data[2];
        result.data[2] = self.data[0]*other.data[2] - self.data[1]*other.data[3] 
                       + self.data[2]*other.data[0] + self.data[3]*other.data[1];
        result.data[3] = self.data[0]*other.data[3] + self.data[1]*other.data[2] 
                       - self.data[2]*other.data[1] + self.data[3]*other.data[0];
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec4_dispatch() {
        let a = Vec4Simd::new(1.0, 2.0, 3.0, 4.0);
        let b = Vec4Simd::new(5.0, 6.0, 7.0, 8.0);
        
        let dot = a.dot(&b);
        assert!((dot - 70.0).abs() < 1e-5);
        
        let sum = a.add(&b);
        assert!((sum.x() - 6.0).abs() < 1e-5);
        assert!((sum.y() - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_mat4_dispatch() {
        let m1 = Mat4Simd::identity();
        let m2 = Mat4Simd::identity();
        let result = m1.mul(&m2);
        
        assert!((result.data[0][0] - 1.0).abs() < 1e-5);
        assert!((result.data[1][1] - 1.0).abs() < 1e-5);
    }
}
