/// 批量变换优化
/// 
/// 用于批量处理顶点变换、法线变换等

use super::{BatchConfig, BatchStats};
use std::time::Instant;

#[cfg(target_arch = "x86_64")]
#[cfg(target_arch = "x86_64")]
use crate::math::x86::*;

#[cfg(target_arch = "aarch64")]
#[cfg(target_arch = "aarch64")]
use crate::math::arm::*;

/// 批量顶点变换
pub struct BatchTransform {
    config: BatchConfig,
}

impl BatchTransform {
    pub fn new(config: BatchConfig) -> Self {
        Self { config }
    }
    
    /// 批量变换顶点（矩阵 * 向量）
    pub fn transform_vertices(
        &self,
        matrix: &[[f32; 4]; 4],
        vertices: &[[f32; 4]],
        output: &mut [[f32; 4]],
    ) -> BatchStats {
        assert_eq!(vertices.len(), output.len());
        
        let start = Instant::now();
        let count = vertices.len();
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("sse2") {
                unsafe {
                    transform_vectors_sse2(matrix, vertices, output);
                }
                return BatchStats {
                    elements_processed: count,
                    processing_time_us: start.elapsed().as_micros() as u64,
                    backend_used: Some(self.config.backend),
                };
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            unsafe {
                transform_vectors_neon(matrix, vertices, output);
            }
            return BatchStats {
                elements_processed: count,
                processing_time_us: start.elapsed().as_micros() as u64,
                backend_used: Some(self.config.backend),
            };
        }
        
        // 标量回退
        self.transform_vertices_scalar(matrix, vertices, output);
        
        BatchStats {
            elements_processed: count,
            processing_time_us: start.elapsed().as_micros() as u64,
            backend_used: None,
        }
    }
    
    fn transform_vertices_scalar(
        &self,
        matrix: &[[f32; 4]; 4],
        vertices: &[[f32; 4]],
        output: &mut [[f32; 4]],
    ) {
        for (v, out) in vertices.iter().zip(output.iter_mut()) {
            for i in 0..4 {
                out[i] = matrix[i][0] * v[0]
                       + matrix[i][1] * v[1]
                       + matrix[i][2] * v[2]
                       + matrix[i][3] * v[3];
            }
        }
    }
    
    /// 批量变换法线（3x3矩阵的逆转置 * 法线）
    pub fn transform_normals(
        &self,
        matrix: &[[f32; 3]; 3],
        normals: &[[f32; 3]],
        output: &mut [[f32; 3]],
    ) -> BatchStats {
        assert_eq!(normals.len(), output.len());
        
        let start = Instant::now();
        let count = normals.len();
        
        // 简化：直接使用矩阵变换（假设已经是逆转置）
        for (n, out) in normals.iter().zip(output.iter_mut()) {
            for i in 0..3 {
                out[i] = matrix[i][0] * n[0]
                       + matrix[i][1] * n[1]
                       + matrix[i][2] * n[2];
            }
            
            // 归一化
            let len = (out[0] * out[0] + out[1] * out[1] + out[2] * out[2]).sqrt();
            if len > 1e-6 {
                let inv_len = 1.0 / len;
                out[0] *= inv_len;
                out[1] *= inv_len;
                out[2] *= inv_len;
            }
        }
        
        BatchStats {
            elements_processed: count,
            processing_time_us: start.elapsed().as_micros() as u64,
            backend_used: Some(self.config.backend),
        }
    }
    
    /// 批量计算包围盒
    pub fn compute_bounding_boxes(
        &self,
        vertices: &[[f32; 3]],
    ) -> ([f32; 3], [f32; 3]) {
        if vertices.is_empty() {
            return ([0.0; 3], [0.0; 3]);
        }
        
        let mut min = vertices[0];
        let mut max = vertices[0];
        
        for v in vertices.iter().skip(1) {
            for i in 0..3 {
                min[i] = min[i].min(v[i]);
                max[i] = max[i].max(v[i]);
            }
        }
        
        (min, max)
    }
}

/// 批量插值处理
pub struct BatchInterpolation {
    config: BatchConfig,
}

impl BatchInterpolation {
    pub fn new(config: BatchConfig) -> Self {
        Self { config }
    }
    
    /// 批量线性插值
    pub fn lerp(
        &self,
        a: &[[f32; 4]],
        b: &[[f32; 4]],
        t: f32,
        output: &mut [[f32; 4]],
    ) -> BatchStats {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), output.len());
        
        let start = Instant::now();
        let count = a.len();
        
        #[cfg(target_arch = "aarch64")]
        {
            unsafe {
                lerp_batch_neon(a, b, t, output);
            }
            return BatchStats {
                elements_processed: count,
                processing_time_us: start.elapsed().as_micros() as u64,
                backend_used: Some(self.config.backend),
            };
        }
        
        // 标量实现
        for i in 0..count {
            for j in 0..4 {
                output[i][j] = a[i][j] * (1.0 - t) + b[i][j] * t;
            }
        }
        
        BatchStats {
            elements_processed: count,
            processing_time_us: start.elapsed().as_micros() as u64,
            backend_used: None,
        }
    }
    
    /// 批量球面线性插值（四元数）
    pub fn slerp(
        &self,
        a: &[[f32; 4]],
        b: &[[f32; 4]],
        t: f32,
        output: &mut [[f32; 4]],
    ) -> BatchStats {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), output.len());
        
        let start = Instant::now();
        let count = a.len();
        
        // SLERP标量实现
        for i in 0..count {
            let qa = &a[i];
            let qb = &b[i];
            
            // 计算点积
            let mut dot = qa[0] * qb[0] + qa[1] * qb[1] + qa[2] * qb[2] + qa[3] * qb[3];
            
            // 如果点积为负，反转一个四元数
            let (qb0, qb1, qb2, qb3) = if dot < 0.0 {
                dot = -dot;
                (-qb[0], -qb[1], -qb[2], -qb[3])
            } else {
                (qb[0], qb[1], qb[2], qb[3])
            };
            
            // 如果四元数非常接近，使用线性插值
            if dot > 0.9995 {
                output[i][0] = qa[0] + t * (qb0 - qa[0]);
                output[i][1] = qa[1] + t * (qb1 - qa[1]);
                output[i][2] = qa[2] + t * (qb2 - qa[2]);
                output[i][3] = qa[3] + t * (qb3 - qa[3]);
            } else {
                let theta = dot.acos();
                let sin_theta = theta.sin();
                let a_coeff = ((1.0 - t) * theta).sin() / sin_theta;
                let b_coeff = (t * theta).sin() / sin_theta;
                
                output[i][0] = a_coeff * qa[0] + b_coeff * qb0;
                output[i][1] = a_coeff * qa[1] + b_coeff * qb1;
                output[i][2] = a_coeff * qa[2] + b_coeff * qb2;
                output[i][3] = a_coeff * qa[3] + b_coeff * qb3;
            }
            
            // 归一化
            let len = (output[i][0] * output[i][0] + output[i][1] * output[i][1] 
                     + output[i][2] * output[i][2] + output[i][3] * output[i][3]).sqrt();
            if len > 1e-6 {
                let inv_len = 1.0 / len;
                output[i][0] *= inv_len;
                output[i][1] *= inv_len;
                output[i][2] *= inv_len;
                output[i][3] *= inv_len;
            }
        }
        
        BatchStats {
            elements_processed: count,
            processing_time_us: start.elapsed().as_micros() as u64,
            backend_used: Some(self.config.backend),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_transform() {
        let config = BatchConfig::default();
        let transformer = BatchTransform::new(config);
        
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        
        let vertices = vec![
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
        ];
        
        let mut output = vec![[0.0; 4]; 3];
        
        let stats = transformer.transform_vertices(&identity, &vertices, &mut output);
        assert_eq!(stats.elements_processed, 3);
        assert_eq!(output[0], vertices[0]);
    }

    #[test]
    fn test_batch_lerp() {
        let config = BatchConfig::default();
        let interpolator = BatchInterpolation::new(config);
        
        let a = vec![[0.0, 0.0, 0.0, 1.0]];
        let b = vec![[1.0, 1.0, 1.0, 1.0]];
        let mut output = vec![[0.0; 4]; 1];
        
        let stats = interpolator.lerp(&a, &b, 0.5, &mut output);
        assert_eq!(stats.elements_processed, 1);
        assert!((output[0][0] - 0.5).abs() < 1e-5);
    }
}
