/// 骨骼蒙皮批量处理优化
/// 
/// 用于角色动画的顶点蒙皮计算

use super::{BatchConfig, BatchStats};
use std::time::Instant;

/// 骨骼影响数据
#[derive(Debug, Clone, Copy)]
pub struct BoneInfluence {
    /// 骨骼索引
    pub bone_indices: [u32; 4],
    /// 骨骼权重
    pub bone_weights: [f32; 4],
}

impl Default for BoneInfluence {
    fn default() -> Self {
        Self {
            bone_indices: [0; 4],
            bone_weights: [0.0; 4],
        }
    }
}

/// 批量蒙皮处理器
pub struct BatchSkinning {
    config: BatchConfig,
}

impl BatchSkinning {
    pub fn new(config: BatchConfig) -> Self {
        Self { config }
    }
    
    /// 批量线性混合蒙皮（LBS）
    pub fn linear_blend_skinning(
        &self,
        vertices: &[[f32; 3]],
        normals: &[[f32; 3]],
        influences: &[BoneInfluence],
        bone_matrices: &[[[f32; 4]; 4]],
        output_vertices: &mut [[f32; 3]],
        output_normals: &mut [[f32; 3]],
    ) -> BatchStats {
        assert_eq!(vertices.len(), normals.len());
        assert_eq!(vertices.len(), influences.len());
        assert_eq!(vertices.len(), output_vertices.len());
        assert_eq!(vertices.len(), output_normals.len());
        
        let start = Instant::now();
        let count = vertices.len();
        
        for i in 0..count {
            let v = &vertices[i];
            let n = &normals[i];
            let inf = &influences[i];
            
            // 初始化输出
            let mut out_v = [0.0f32; 3];
            let mut out_n = [0.0f32; 3];
            
            // 对每个骨骼影响进行累加
            for j in 0..4 {
                let weight = inf.bone_weights[j];
                if weight > 0.0001 {
                    let bone_idx = inf.bone_indices[j] as usize;
                    if bone_idx < bone_matrices.len() {
                        let matrix = &bone_matrices[bone_idx];
                        
                        // 变换顶点
                        let transformed_v = [
                            matrix[0][0] * v[0] + matrix[0][1] * v[1] + matrix[0][2] * v[2] + matrix[0][3],
                            matrix[1][0] * v[0] + matrix[1][1] * v[1] + matrix[1][2] * v[2] + matrix[1][3],
                            matrix[2][0] * v[0] + matrix[2][1] * v[1] + matrix[2][2] * v[2] + matrix[2][3],
                        ];
                        
                        // 变换法线（忽略平移）
                        let transformed_n = [
                            matrix[0][0] * n[0] + matrix[0][1] * n[1] + matrix[0][2] * n[2],
                            matrix[1][0] * n[0] + matrix[1][1] * n[1] + matrix[1][2] * n[2],
                            matrix[2][0] * n[0] + matrix[2][1] * n[1] + matrix[2][2] * n[2],
                        ];
                        
                        // 加权累加
                        out_v[0] += transformed_v[0] * weight;
                        out_v[1] += transformed_v[1] * weight;
                        out_v[2] += transformed_v[2] * weight;
                        
                        out_n[0] += transformed_n[0] * weight;
                        out_n[1] += transformed_n[1] * weight;
                        out_n[2] += transformed_n[2] * weight;
                    }
                }
            }
            
            // 归一化法线
            let n_len = (out_n[0] * out_n[0] + out_n[1] * out_n[1] + out_n[2] * out_n[2]).sqrt();
            if n_len > 1e-6 {
                let inv_len = 1.0 / n_len;
                out_n[0] *= inv_len;
                out_n[1] *= inv_len;
                out_n[2] *= inv_len;
            }
            
            output_vertices[i] = out_v;
            output_normals[i] = out_n;
        }
        
        BatchStats {
            elements_processed: count,
            processing_time_us: start.elapsed().as_micros() as u64,
            backend_used: Some(self.config.backend),
        }
    }
    
    /// 批量双四元数蒙皮（DQS）
    /// 
    /// 相比LBS，DQS可以避免"糖果包装"效果
    pub fn dual_quaternion_skinning(
        &self,
        vertices: &[[f32; 3]],
        normals: &[[f32; 3]],
        influences: &[BoneInfluence],
        bone_dual_quats: &[([f32; 4], [f32; 4])], // (real, dual)
        output_vertices: &mut [[f32; 3]],
        output_normals: &mut [[f32; 3]],
    ) -> BatchStats {
        assert_eq!(vertices.len(), normals.len());
        assert_eq!(vertices.len(), influences.len());
        assert_eq!(vertices.len(), output_vertices.len());
        assert_eq!(vertices.len(), output_normals.len());
        
        let start = Instant::now();
        let count = vertices.len();
        
        for i in 0..count {
            let v = &vertices[i];
            let n = &normals[i];
            let inf = &influences[i];
            
            // 混合双四元数
            let mut blended_real = [0.0f32; 4];
            let mut blended_dual = [0.0f32; 4];
            
            // 确保所有四元数朝向同一方向
            let first_bone_idx = inf.bone_indices[0] as usize;
            if first_bone_idx >= bone_dual_quats.len() {
                continue;
            }
            let reference_quat = bone_dual_quats[first_bone_idx].0;
            
            for j in 0..4 {
                let weight = inf.bone_weights[j];
                if weight > 0.0001 {
                    let bone_idx = inf.bone_indices[j] as usize;
                    if bone_idx < bone_dual_quats.len() {
                        let (mut real, dual) = bone_dual_quats[bone_idx];
                        
                        // 检查点积，确保朝向一致
                        let dot = real[0] * reference_quat[0] + real[1] * reference_quat[1]
                                + real[2] * reference_quat[2] + real[3] * reference_quat[3];
                        
                        let sign = if dot < 0.0 { -1.0 } else { 1.0 };
                        let adjusted_weight = weight * sign;
                        
                        for k in 0..4 {
                            blended_real[k] += real[k] * adjusted_weight;
                            blended_dual[k] += dual[k] * adjusted_weight;
                        }
                    }
                }
            }
            
            // 归一化实部四元数
            let real_len = (blended_real[0] * blended_real[0] + blended_real[1] * blended_real[1]
                          + blended_real[2] * blended_real[2] + blended_real[3] * blended_real[3]).sqrt();
            
            if real_len > 1e-6 {
                let inv_len = 1.0 / real_len;
                for k in 0..4 {
                    blended_real[k] *= inv_len;
                    blended_dual[k] *= inv_len;
                }
                
                // 从双四元数提取平移
                let translation = [
                    2.0 * (-blended_dual[0] * blended_real[1] + blended_dual[1] * blended_real[0]
                          - blended_dual[2] * blended_real[3] + blended_dual[3] * blended_real[2]),
                    2.0 * (-blended_dual[0] * blended_real[2] + blended_dual[1] * blended_real[3]
                          + blended_dual[2] * blended_real[0] - blended_dual[3] * blended_real[1]),
                    2.0 * (-blended_dual[0] * blended_real[3] - blended_dual[1] * blended_real[2]
                          + blended_dual[2] * blended_real[1] + blended_dual[3] * blended_real[0]),
                ];
                
                // 使用四元数旋转顶点和法线
                let rotated_v = self.rotate_by_quaternion(v, &blended_real);
                let rotated_n = self.rotate_by_quaternion(n, &blended_real);
                
                output_vertices[i] = [
                    rotated_v[0] + translation[0],
                    rotated_v[1] + translation[1],
                    rotated_v[2] + translation[2],
                ];
                output_normals[i] = rotated_n;
            } else {
                output_vertices[i] = *v;
                output_normals[i] = *n;
            }
        }
        
        BatchStats {
            elements_processed: count,
            processing_time_us: start.elapsed().as_micros() as u64,
            backend_used: Some(self.config.backend),
        }
    }
    
    /// 使用四元数旋转向量
    fn rotate_by_quaternion(&self, v: &[f32; 3], q: &[f32; 4]) -> [f32; 3] {
        // q = [w, x, y, z]
        let w = q[0];
        let qx = q[1];
        let qy = q[2];
        let qz = q[3];
        
        // v' = v + 2 * cross(q.xyz, cross(q.xyz, v) + w * v)
        let cross1 = [
            qy * v[2] - qz * v[1],
            qz * v[0] - qx * v[2],
            qx * v[1] - qy * v[0],
        ];
        
        let cross1_plus_wv = [
            cross1[0] + w * v[0],
            cross1[1] + w * v[1],
            cross1[2] + w * v[2],
        ];
        
        let cross2 = [
            qy * cross1_plus_wv[2] - qz * cross1_plus_wv[1],
            qz * cross1_plus_wv[0] - qx * cross1_plus_wv[2],
            qx * cross1_plus_wv[1] - qy * cross1_plus_wv[0],
        ];
        
        [
            v[0] + 2.0 * cross2[0],
            v[1] + 2.0 * cross2[1],
            v[2] + 2.0 * cross2[2],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bone_influence() {
        let influence = BoneInfluence::default();
        assert_eq!(influence.bone_indices[0], 0);
        assert_eq!(influence.bone_weights[0], 0.0);
    }

    #[test]
    fn test_linear_blend_skinning() {
        let config = BatchConfig::default();
        let skinning = BatchSkinning::new(config);
        
        let vertices = vec![[1.0, 0.0, 0.0]];
        let normals = vec![[0.0, 1.0, 0.0]];
        let influences = vec![BoneInfluence {
            bone_indices: [0, 0, 0, 0],
            bone_weights: [1.0, 0.0, 0.0, 0.0],
        }];
        
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let bone_matrices = vec![identity];
        
        let mut output_vertices = vec![[0.0; 3]];
        let mut output_normals = vec![[0.0; 3]];
        
        let stats = skinning.linear_blend_skinning(
            &vertices,
            &normals,
            &influences,
            &bone_matrices,
            &mut output_vertices,
            &mut output_normals,
        );
        
        assert_eq!(stats.elements_processed, 1);
        assert!((output_vertices[0][0] - 1.0).abs() < 1e-5);
    }
}
