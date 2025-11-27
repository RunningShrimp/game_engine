/// 标量回退实现
/// 
/// 当SIMD不可用时的纯标量实现

/// 4维向量点积（标量）
pub fn dot_product_scalar(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

/// 4维向量加法（标量）
pub fn add_vec4_scalar(a: &[f32; 4], b: &[f32; 4], out: &mut [f32; 4]) {
    out[0] = a[0] + b[0];
    out[1] = a[1] + b[1];
    out[2] = a[2] + b[2];
    out[3] = a[3] + b[3];
}

/// 4维向量减法（标量）
pub fn sub_vec4_scalar(a: &[f32; 4], b: &[f32; 4], out: &mut [f32; 4]) {
    out[0] = a[0] - b[0];
    out[1] = a[1] - b[1];
    out[2] = a[2] - b[2];
    out[3] = a[3] - b[3];
}

/// 4维向量标量乘法
pub fn mul_vec4_scalar(a: &[f32; 4], scalar: f32, out: &mut [f32; 4]) {
    out[0] = a[0] * scalar;
    out[1] = a[1] * scalar;
    out[2] = a[2] * scalar;
    out[3] = a[3] * scalar;
}

/// 4x4矩阵乘法（标量）
pub fn mat4_mul_scalar(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4], out: &mut [[f32; 4]; 4]) {
    for i in 0..4 {
        for j in 0..4 {
            out[i][j] = a[i][0] * b[0][j]
                      + a[i][1] * b[1][j]
                      + a[i][2] * b[2][j]
                      + a[i][3] * b[3][j];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_ops() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [5.0, 6.0, 7.0, 8.0];
        
        let dot = dot_product_scalar(&a, &b);
        assert_eq!(dot, 70.0);
        
        let mut sum = [0.0; 4];
        add_vec4_scalar(&a, &b, &mut sum);
        assert_eq!(sum, [6.0, 8.0, 10.0, 12.0]);
    }
}
