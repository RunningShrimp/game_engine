/// x86/x64 SIMD数学运算优化
/// 
/// 支持SSE2, SSE4.1, AVX, AVX2, AVX-512指令集

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// 使用SSE2的4维向量点积
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
pub unsafe fn dot_product_sse2(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    let va = _mm_loadu_ps(a.as_ptr());
    let vb = _mm_loadu_ps(b.as_ptr());
    let mul = _mm_mul_ps(va, vb);
    
    // 水平加法
    let shuf = _mm_shuffle_ps(mul, mul, 0b_11_10_11_10);
    let sums = _mm_add_ps(mul, shuf);
    let shuf2 = _mm_movehl_ps(sums, sums);
    let result = _mm_add_ss(sums, shuf2);
    
    _mm_cvtss_f32(result)
}

/// 使用SSE4.1的4维向量点积（更高效）
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub unsafe fn dot_product_sse41(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    let va = _mm_loadu_ps(a.as_ptr());
    let vb = _mm_loadu_ps(b.as_ptr());
    // _mm_dp_ps: 点积指令，0xFF表示使用所有4个分量
    let result = _mm_dp_ps(va, vb, 0xFF);
    _mm_cvtss_f32(result)
}

/// 使用AVX的8维向量点积
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
pub unsafe fn dot_product_avx(a: &[f32; 8], b: &[f32; 8]) -> f32 {
    let va = _mm256_loadu_ps(a.as_ptr());
    let vb = _mm256_loadu_ps(b.as_ptr());
    let mul = _mm256_mul_ps(va, vb);
    
    // 水平加法
    let sum = horizontal_add_avx(mul);
    sum
}

/// AVX水平加法辅助函数
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
unsafe fn horizontal_add_avx(v: __m256) -> f32 {
    // 提取高128位和低128位
    let hi = _mm256_extractf128_ps(v, 1);
    let lo = _mm256_castps256_ps128(v);
    let sum128 = _mm_add_ps(hi, lo);
    
    // 128位水平加法
    let shuf = _mm_shuffle_ps(sum128, sum128, 0b_11_10_11_10);
    let sums = _mm_add_ps(sum128, shuf);
    let shuf2 = _mm_movehl_ps(sums, sums);
    let result = _mm_add_ss(sums, shuf2);
    
    _mm_cvtss_f32(result)
}

/// 使用FMA的向量乘加运算 (a * b + c)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "fma")]
pub unsafe fn fma_vec4(a: &[f32; 4], b: &[f32; 4], c: &[f32; 4], out: &mut [f32; 4]) {
    let va = _mm_loadu_ps(a.as_ptr());
    let vb = _mm_loadu_ps(b.as_ptr());
    let vc = _mm_loadu_ps(c.as_ptr());
    let result = _mm_fmadd_ps(va, vb, vc);
    _mm_storeu_ps(out.as_mut_ptr(), result);
}

/// 使用AVX2的8个f32向量加法
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn add_vec8_avx2(a: &[f32; 8], b: &[f32; 8], out: &mut [f32; 8]) {
    let va = _mm256_loadu_ps(a.as_ptr());
    let vb = _mm256_loadu_ps(b.as_ptr());
    let result = _mm256_add_ps(va, vb);
    _mm256_storeu_ps(out.as_mut_ptr(), result);
}

/// 使用AVX2的8个f32向量乘法
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn mul_vec8_avx2(a: &[f32; 8], b: &[f32; 8], out: &mut [f32; 8]) {
    let va = _mm256_loadu_ps(a.as_ptr());
    let vb = _mm256_loadu_ps(b.as_ptr());
    let result = _mm256_mul_ps(va, vb);
    _mm256_storeu_ps(out.as_mut_ptr(), result);
}

/// 使用AVX-512的16个f32向量点积
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub unsafe fn dot_product_avx512(a: &[f32; 16], b: &[f32; 16]) -> f32 {
    let va = _mm512_loadu_ps(a.as_ptr());
    let vb = _mm512_loadu_ps(b.as_ptr());
    let mul = _mm512_mul_ps(va, vb);
    
    // AVX-512的reduce_add指令
    _mm512_reduce_add_ps(mul)
}

/// 批量向量归一化（SSE2）
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
pub unsafe fn normalize_batch_sse2(vectors: &mut [[f32; 4]]) {
    for vec in vectors.iter_mut() {
        let v = _mm_loadu_ps(vec.as_ptr());
        let dot = dot_product_sse2(vec, vec);
        let len = dot.sqrt();
        
        if len > 1e-6 {
            let inv_len = _mm_set1_ps(1.0 / len);
            let normalized = _mm_mul_ps(v, inv_len);
            _mm_storeu_ps(vec.as_mut_ptr(), normalized);
        }
    }
}

/// 4x4矩阵乘法（SSE优化）
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
pub unsafe fn mat4_mul_sse2(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4], out: &mut [[f32; 4]; 4]) {
    // 加载矩阵B的列
    let b0 = _mm_loadu_ps(b[0].as_ptr());
    let b1 = _mm_loadu_ps(b[1].as_ptr());
    let b2 = _mm_loadu_ps(b[2].as_ptr());
    let b3 = _mm_loadu_ps(b[3].as_ptr());
    
    for i in 0..4 {
        let a_row = a[i];
        
        // 广播a的每个元素
        let a0 = _mm_set1_ps(a_row[0]);
        let a1 = _mm_set1_ps(a_row[1]);
        let a2 = _mm_set1_ps(a_row[2]);
        let a3 = _mm_set1_ps(a_row[3]);
        
        // 计算结果行
        let r0 = _mm_mul_ps(a0, b0);
        let r1 = _mm_mul_ps(a1, b1);
        let r2 = _mm_mul_ps(a2, b2);
        let r3 = _mm_mul_ps(a3, b3);
        
        let result = _mm_add_ps(_mm_add_ps(r0, r1), _mm_add_ps(r2, r3));
        _mm_storeu_ps(out[i].as_mut_ptr(), result);
    }
}

/// 4x4矩阵乘法（AVX优化，处理两个矩阵）
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
pub unsafe fn mat4_mul_avx(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4], out: &mut [[f32; 4]; 4]) {
    // AVX可以一次处理两行，但这里简化为回退到SSE
    // 实际应用中可以实现更高效的AVX版本
    mat4_mul_sse2(a, b, out);
}

/// 批量向量变换（矩阵 * 向量）
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
pub unsafe fn transform_vectors_sse2(
    matrix: &[[f32; 4]; 4],
    vectors: &[[f32; 4]],
    out: &mut [[f32; 4]]
) {
    assert_eq!(vectors.len(), out.len());
    
    // 加载矩阵的行
    let m0 = _mm_loadu_ps(matrix[0].as_ptr());
    let m1 = _mm_loadu_ps(matrix[1].as_ptr());
    let m2 = _mm_loadu_ps(matrix[2].as_ptr());
    let m3 = _mm_loadu_ps(matrix[3].as_ptr());
    
    for (vec, out_vec) in vectors.iter().zip(out.iter_mut()) {
        let v = _mm_loadu_ps(vec.as_ptr());
        
        // 广播向量的每个分量
        let vx = _mm_shuffle_ps(v, v, 0b_00_00_00_00);
        let vy = _mm_shuffle_ps(v, v, 0b_01_01_01_01);
        let vz = _mm_shuffle_ps(v, v, 0b_10_10_10_10);
        let vw = _mm_shuffle_ps(v, v, 0b_11_11_11_11);
        
        // 计算 M * v
        let r0 = _mm_mul_ps(m0, vx);
        let r1 = _mm_mul_ps(m1, vy);
        let r2 = _mm_mul_ps(m2, vz);
        let r3 = _mm_mul_ps(m3, vw);
        
        let result = _mm_add_ps(_mm_add_ps(r0, r1), _mm_add_ps(r2, r3));
        _mm_storeu_ps(out_vec.as_mut_ptr(), result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_dot_product() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [5.0, 6.0, 7.0, 8.0];
        let expected = 1.0 * 5.0 + 2.0 * 6.0 + 3.0 * 7.0 + 4.0 * 8.0;
        
        unsafe {
            if is_x86_feature_detected!("sse2") {
                let result = dot_product_sse2(&a, &b);
                assert!((result - expected).abs() < 1e-5);
            }
            
            if is_x86_feature_detected!("sse4.1") {
                let result = dot_product_sse41(&a, &b);
                assert!((result - expected).abs() < 1e-5);
            }
        }
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_mat4_mul() {
        let a = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [1.0, 2.0, 3.0, 1.0],
        ];
        let b = [
            [2.0, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 2.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let mut out = [[0.0; 4]; 4];
        
        unsafe {
            if is_x86_feature_detected!("sse2") {
                mat4_mul_sse2(&a, &b, &mut out);
                assert_eq!(out[0][0], 2.0);
                assert_eq!(out[1][1], 2.0);
                assert_eq!(out[2][2], 2.0);
            }
        }
    }
}
