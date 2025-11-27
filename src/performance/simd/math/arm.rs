/// ARM NEON SIMD数学运算优化
/// 
/// 支持Apple M系列、华为麒麟、高通骁龙、联发科等ARM处理器
/// NEON是ARM的128位SIMD指令集，在aarch64上是强制支持的

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// 使用NEON的4维向量点积
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn dot_product_neon(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    let va = vld1q_f32(a.as_ptr());
    let vb = vld1q_f32(b.as_ptr());
    let mul = vmulq_f32(va, vb);
    
    // 使用vaddvq_f32进行水平加法（ARMv8）
    vaddvq_f32(mul)
}

/// 使用NEON的4维向量加法
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn add_vec4_neon(a: &[f32; 4], b: &[f32; 4], out: &mut [f32; 4]) {
    let va = vld1q_f32(a.as_ptr());
    let vb = vld1q_f32(b.as_ptr());
    let result = vaddq_f32(va, vb);
    vst1q_f32(out.as_mut_ptr(), result);
}

/// 使用NEON的4维向量减法
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn sub_vec4_neon(a: &[f32; 4], b: &[f32; 4], out: &mut [f32; 4]) {
    let va = vld1q_f32(a.as_ptr());
    let vb = vld1q_f32(b.as_ptr());
    let result = vsubq_f32(va, vb);
    vst1q_f32(out.as_mut_ptr(), result);
}

/// 使用NEON的4维向量乘法
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn mul_vec4_neon(a: &[f32; 4], b: &[f32; 4], out: &mut [f32; 4]) {
    let va = vld1q_f32(a.as_ptr());
    let vb = vld1q_f32(b.as_ptr());
    let result = vmulq_f32(va, vb);
    vst1q_f32(out.as_mut_ptr(), result);
}

/// 使用NEON的向量乘加运算 (a * b + c)
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn fma_vec4_neon(a: &[f32; 4], b: &[f32; 4], c: &[f32; 4], out: &mut [f32; 4]) {
    let va = vld1q_f32(a.as_ptr());
    let vb = vld1q_f32(b.as_ptr());
    let vc = vld1q_f32(c.as_ptr());
    // vfmaq_f32: fused multiply-add
    let result = vfmaq_f32(vc, va, vb);
    vst1q_f32(out.as_mut_ptr(), result);
}

/// 使用NEON的向量归一化
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn normalize_vec4_neon(v: &[f32; 4], out: &mut [f32; 4]) {
    let vec = vld1q_f32(v.as_ptr());
    let dot = dot_product_neon(v, v);
    let len = dot.sqrt();
    
    if len > 1e-6 {
        let inv_len = 1.0 / len;
        let scale = vdupq_n_f32(inv_len);
        let normalized = vmulq_f32(vec, scale);
        vst1q_f32(out.as_mut_ptr(), normalized);
    } else {
        vst1q_f32(out.as_mut_ptr(), vec);
    }
}

/// 批量向量归一化（NEON）
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn normalize_batch_neon(vectors: &mut [[f32; 4]]) {
    for vec in vectors.iter_mut() {
        normalize_vec4_neon(vec, vec);
    }
}

/// 3维向量叉积（NEON优化）
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn cross_product_neon(a: &[f32; 3], b: &[f32; 3], out: &mut [f32; 3]) {
    // 扩展到4维以使用NEON
    let a4 = [a[0], a[1], a[2], 0.0];
    let b4 = [b[0], b[1], b[2], 0.0];
    
    let va = vld1q_f32(a4.as_ptr());
    let vb = vld1q_f32(b4.as_ptr());
    
    // 叉积: (a.y*b.z - a.z*b.y, a.z*b.x - a.x*b.z, a.x*b.y - a.y*b.x)
    // 使用shuffle和multiply
    
    // a_yzx = [a.y, a.z, a.x, a.w]
    let a_yzx = vextq_f32(va, va, 1);
    // b_zxy = [b.z, b.x, b.y, b.w]
    let b_zxy = vextq_f32(vb, vb, 2);
    
    let mul1 = vmulq_f32(a_yzx, b_zxy);
    
    // a_zxy = [a.z, a.x, a.y, a.w]
    let a_zxy = vextq_f32(va, va, 2);
    // b_yzx = [b.y, b.z, b.x, b.w]
    let b_yzx = vextq_f32(vb, vb, 1);
    
    let mul2 = vmulq_f32(a_zxy, b_yzx);
    
    let result = vsubq_f32(mul1, mul2);
    
    // 提取前3个分量
    let mut temp = [0.0f32; 4];
    vst1q_f32(temp.as_mut_ptr(), result);
    out[0] = temp[0];
    out[1] = temp[1];
    out[2] = temp[2];
}

/// 4x4矩阵乘法（NEON优化）
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn mat4_mul_neon(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4], out: &mut [[f32; 4]; 4]) {
    // 加载矩阵B的列
    let b0 = vld1q_f32(b[0].as_ptr());
    let b1 = vld1q_f32(b[1].as_ptr());
    let b2 = vld1q_f32(b[2].as_ptr());
    let b3 = vld1q_f32(b[3].as_ptr());
    
    for i in 0..4 {
        let a_row = a[i];
        
        // 广播a的每个元素
        let a0 = vdupq_n_f32(a_row[0]);
        let a1 = vdupq_n_f32(a_row[1]);
        let a2 = vdupq_n_f32(a_row[2]);
        let a3 = vdupq_n_f32(a_row[3]);
        
        // 计算结果行: a[i][0]*b[0] + a[i][1]*b[1] + a[i][2]*b[2] + a[i][3]*b[3]
        let r0 = vmulq_f32(a0, b0);
        let r1 = vfmaq_f32(r0, a1, b1);
        let r2 = vfmaq_f32(r1, a2, b2);
        let result = vfmaq_f32(r2, a3, b3);
        
        vst1q_f32(out[i].as_mut_ptr(), result);
    }
}

/// 4x4矩阵转置（NEON优化）
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn mat4_transpose_neon(m: &[[f32; 4]; 4], out: &mut [[f32; 4]; 4]) {
    let r0 = vld1q_f32(m[0].as_ptr());
    let r1 = vld1q_f32(m[1].as_ptr());
    let r2 = vld1q_f32(m[2].as_ptr());
    let r3 = vld1q_f32(m[3].as_ptr());
    
    // 使用vtrn和vzip进行转置
    // 这是一个简化版本，实际可以更优化
    let mut temp = [[0.0f32; 4]; 4];
    vst1q_f32(temp[0].as_mut_ptr(), r0);
    vst1q_f32(temp[1].as_mut_ptr(), r1);
    vst1q_f32(temp[2].as_mut_ptr(), r2);
    vst1q_f32(temp[3].as_mut_ptr(), r3);
    
    for i in 0..4 {
        for j in 0..4 {
            out[i][j] = temp[j][i];
        }
    }
}

/// 批量向量变换（矩阵 * 向量）
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn transform_vectors_neon(
    matrix: &[[f32; 4]; 4],
    vectors: &[[f32; 4]],
    out: &mut [[f32; 4]]
) {
    assert_eq!(vectors.len(), out.len());
    
    // 加载矩阵的行
    let m0 = vld1q_f32(matrix[0].as_ptr());
    let m1 = vld1q_f32(matrix[1].as_ptr());
    let m2 = vld1q_f32(matrix[2].as_ptr());
    let m3 = vld1q_f32(matrix[3].as_ptr());
    
    for (vec, out_vec) in vectors.iter().zip(out.iter_mut()) {
        let v = vld1q_f32(vec.as_ptr());
        
        // 提取向量的每个分量
        let vx = vdupq_laneq_f32(v, 0);
        let vy = vdupq_laneq_f32(v, 1);
        let vz = vdupq_laneq_f32(v, 2);
        let vw = vdupq_laneq_f32(v, 3);
        
        // 计算 M * v
        let r0 = vmulq_f32(m0, vx);
        let r1 = vfmaq_f32(r0, m1, vy);
        let r2 = vfmaq_f32(r1, m2, vz);
        let result = vfmaq_f32(r2, m3, vw);
        
        vst1q_f32(out_vec.as_mut_ptr(), result);
    }
}

/// 四元数乘法（NEON优化）
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn quat_mul_neon(a: &[f32; 4], b: &[f32; 4], out: &mut [f32; 4]) {
    // 四元数乘法: (w1*w2 - x1*x2 - y1*y2 - z1*z2,
    //              w1*x2 + x1*w2 + y1*z2 - z1*y2,
    //              w1*y2 - x1*z2 + y1*w2 + z1*x2,
    //              w1*z2 + x1*y2 - y1*x2 + z1*w2)
    
    let qa = vld1q_f32(a.as_ptr());
    let qb = vld1q_f32(b.as_ptr());
    
    // 这里使用简化实现，实际可以更优化
    let mut result = [0.0f32; 4];
    
    // 标量实现（NEON版本较复杂）
    result[0] = a[0]*b[0] - a[1]*b[1] - a[2]*b[2] - a[3]*b[3];
    result[1] = a[0]*b[1] + a[1]*b[0] + a[2]*b[3] - a[3]*b[2];
    result[2] = a[0]*b[2] - a[1]*b[3] + a[2]*b[0] + a[3]*b[1];
    result[3] = a[0]*b[3] + a[1]*b[2] - a[2]*b[1] + a[3]*b[0];
    
    let vresult = vld1q_f32(result.as_ptr());
    vst1q_f32(out.as_mut_ptr(), vresult);
}

/// 批量线性插值（NEON）
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn lerp_batch_neon(a: &[[f32; 4]], b: &[[f32; 4]], t: f32, out: &mut [[f32; 4]]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());
    
    let vt = vdupq_n_f32(t);
    let v_one_minus_t = vdupq_n_f32(1.0 - t);
    
    for i in 0..a.len() {
        let va = vld1q_f32(a[i].as_ptr());
        let vb = vld1q_f32(b[i].as_ptr());
        
        // lerp = a * (1 - t) + b * t
        let scaled_a = vmulq_f32(va, v_one_minus_t);
        let result = vfmaq_f32(scaled_a, vb, vt);
        
        vst1q_f32(out[i].as_mut_ptr(), result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_dot_product_neon() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [5.0, 6.0, 7.0, 8.0];
        let expected = 1.0 * 5.0 + 2.0 * 6.0 + 3.0 * 7.0 + 4.0 * 8.0;
        
        unsafe {
            let result = dot_product_neon(&a, &b);
            assert!((result - expected).abs() < 1e-5);
        }
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_vec4_operations() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [5.0, 6.0, 7.0, 8.0];
        let mut out = [0.0; 4];
        
        unsafe {
            add_vec4_neon(&a, &b, &mut out);
            assert_eq!(out, [6.0, 8.0, 10.0, 12.0]);
            
            mul_vec4_neon(&a, &b, &mut out);
            assert_eq!(out, [5.0, 12.0, 21.0, 32.0]);
        }
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_mat4_mul_neon() {
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let scale = [
            [2.0, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 2.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let mut out = [[0.0; 4]; 4];
        
        unsafe {
            mat4_mul_neon(&identity, &scale, &mut out);
            assert_eq!(out[0][0], 2.0);
            assert_eq!(out[1][1], 2.0);
            assert_eq!(out[2][2], 2.0);
            assert_eq!(out[3][3], 1.0);
        }
    }
}
