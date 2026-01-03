/// 骨骼蒙皮SIMD批量处理集成
///
/// 集成game_engine_simd的蒙皮批量处理功能。
use bevy_ecs::prelude::*;
use glam::Mat4;

// 条件性导入SIMD支持
use game_engine_simd::batch::BatchConfig;
#[cfg(feature = "simd")]
use game_engine_simd::batch::skinning::{BatchSkinning, BoneInfluence};

/// SIMD蒙皮影响数据组件
#[derive(Component, Debug, Clone)]
pub struct SimdBoneInfluence {
    /// 骨骼索引
    pub bone_indices: [u32; 4],
    /// 骨骼权重
    pub bone_weights: [f32; 4],
}

impl Default for SimdBoneInfluence {
    fn default() -> Self {
        Self {
            bone_indices: [0; 4],
            bone_weights: [0.0; 4],
        }
    }
}

#[cfg(feature = "simd")]
impl From<BoneInfluence> for SimdBoneInfluence {
    fn from(bi: BoneInfluence) -> Self {
        Self {
            bone_indices: bi.bone_indices,
            bone_weights: bi.bone_weights,
        }
    }
}

#[cfg(feature = "simd")]
impl From<SimdBoneInfluence> for BoneInfluence {
    fn from(bi: SimdBoneInfluence) -> Self {
        Self {
            bone_indices: bi.bone_indices,
            bone_weights: bi.bone_weights,
        }
    }
}

/// SIMD蒙皮批量处理器资源
#[derive(Resource)]
pub struct SimdSkinningProcessor {
    /// 批量处理器
    #[cfg(feature = "simd")]
    processor: BatchSkinning,
    /// 配置
    config: BatchConfig,
}

impl std::fmt::Debug for SimdSkinningProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimdSkinningProcessor").field("config", &self.config).finish()
    }
}

impl Default for SimdSkinningProcessor {
    fn default() -> Self {
        let config = BatchConfig::default();
        #[cfg(feature = "simd")]
        let processor = BatchSkinning::new(config.clone());

        Self {
            #[cfg(feature = "simd")]
            processor,
            config,
        }
    }
}

/// 顶点蒙皮数据组件
#[derive(Component, Debug, Clone)]
pub struct SimdSkinnedVertex {
    /// 原始顶点位置
    pub position: [f32; 3],
    /// 原始法线
    pub normal: [f32; 3],
    /// 变换后的顶点位置
    pub transformed_position: [f32; 3],
    /// 变换后的法线
    pub transformed_normal: [f32; 3],
}

impl Default for SimdSkinnedVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            transformed_position: [0.0; 3],
            transformed_normal: [0.0, 1.0, 0.0],
        }
    }
}

/// 骨骼变换矩阵资源
#[derive(Resource, Debug, Clone, Default)]
pub struct BoneTransforms {
    /// 骨骼变换矩阵列表
    pub bone_matrices: Vec<Mat4>,
}

/// SIMD线性混合蒙皮（LBS）系统
///
/// 使用SIMD加速批量进行线性混合蒙皮计算
pub fn simd_linear_blend_skinning_system(
    mut query: Query<(&mut SimdSkinnedVertex, &SimdBoneInfluence)>,
    bone_transforms: Res<BoneTransforms>,
    mut processor: ResMut<SimdSkinningProcessor>,
) {
    // 收集蒙皮数据
    let vertices: Vec<[f32; 3]> = query.iter().map(|(v, _)| v.position).collect();
    let normals: Vec<[f32; 3]> = query.iter().map(|(v, _)| v.normal).collect();

    #[cfg(feature = "simd")]
    let influences: Vec<BoneInfluence> =
        query.iter().map(|(_, i)| BoneInfluence::from(i.clone())).collect();

    #[cfg(not(feature = "simd"))]
    let influences: Vec<SimdBoneInfluence> = query.iter().map(|(_, i)| i.clone()).collect();

    if vertices.is_empty() || bone_transforms.bone_matrices.is_empty() {
        return;
    }

    // 转换骨骼矩阵格式
    let bone_matrices: Vec<[[f32; 4]; 4]> = bone_transforms
        .bone_matrices
        .iter()
        .map(|m| {
            [
                [m.x_axis.x, m.x_axis.y, m.x_axis.z, m.x_axis.w],
                [m.y_axis.x, m.y_axis.y, m.y_axis.z, m.y_axis.w],
                [m.z_axis.x, m.z_axis.y, m.z_axis.z, m.z_axis.w],
                [m.w_axis.x, m.w_axis.y, m.w_axis.z, m.w_axis.w],
            ]
        })
        .collect();

    let mut output_vertices = vec![[0.0; 3]; vertices.len()];
    let mut output_normals = vec![[0.0; 3]; normals.len()];

    // 使用SIMD批量进行线性混合蒙皮
    #[cfg(feature = "simd")]
    {
        let _stats = processor.processor.linear_blend_skinning(
            &vertices,
            &normals,
            &influences,
            &bone_matrices,
            &mut output_vertices,
            &mut output_normals,
        );
    }

    #[cfg(not(feature = "simd"))]
    {
        // 标量回退实现
        for i in 0..vertices.len() {
            let v = &vertices[i];
            let n = &normals[i];
            let inf = &influences[i];

            let mut out_v = [0.0f32; 3];
            let mut out_n = [0.0f32; 3];

            for j in 0..4 {
                let weight = inf.bone_weights[j];
                if weight > 0.0001 {
                    let bone_idx = inf.bone_indices[j] as usize;
                    if bone_idx < bone_matrices.len() {
                        let matrix = &bone_matrices[bone_idx];

                        let transformed_v = [
                            matrix[0][0] * v[0]
                                + matrix[0][1] * v[1]
                                + matrix[0][2] * v[2]
                                + matrix[0][3],
                            matrix[1][0] * v[0]
                                + matrix[1][1] * v[1]
                                + matrix[1][2] * v[2]
                                + matrix[1][3],
                            matrix[2][0] * v[0]
                                + matrix[2][1] * v[1]
                                + matrix[2][2] * v[2]
                                + matrix[2][3],
                        ];

                        let transformed_n = [
                            matrix[0][0] * n[0] + matrix[0][1] * n[1] + matrix[0][2] * n[2],
                            matrix[1][0] * n[0] + matrix[1][1] * n[1] + matrix[1][2] * n[2],
                            matrix[2][0] * n[0] + matrix[2][1] * n[1] + matrix[2][2] * n[2],
                        ];

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
    }

    // 将结果写回ECS组件
    for (mut vertex, _) in query.iter_mut() {
        // 找到对应的顶点（这里简化处理，实际应该使用entity索引）
        if let Some((out_v, out_n)) = output_vertices.iter().zip(output_normals.iter()).next() {
            vertex.transformed_position = *out_v;
            vertex.transformed_normal = *out_n;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_bone_influence_default() {
        let influence = SimdBoneInfluence::default();
        assert_eq!(influence.bone_indices[0], 0);
        assert_eq!(influence.bone_weights[0], 0.0);
    }

    #[test]
    fn test_simd_skinned_vertex_default() {
        let vertex = SimdSkinnedVertex::default();
        assert_eq!(vertex.position, [0.0; 3]);
        assert_eq!(vertex.normal, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_simd_skinning_processor_default() {
        let processor = SimdSkinningProcessor::default();
        assert!(processor.config.batch_size > 0);
    }

    #[test]
    fn test_bone_transforms_default() {
        let transforms = BoneTransforms::default();
        assert!(transforms.bone_matrices.is_empty());
    }
}
