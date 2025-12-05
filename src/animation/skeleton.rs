//! 骨骼数据结构
//!
//! 定义骨骼层级和骨骼节点，支持复杂角色动画。

use bevy_ecs::prelude::*;
use glam::{Mat4, Quat, Vec3};

// ============================================================================
// 骨骼节点
// ============================================================================

/// 骨骼节点
#[derive(Clone, Debug)]
pub struct Bone {
    /// 骨骼名称
    pub name: String,
    /// 父骨骼索引（None 表示根骨骼）
    pub parent_index: Option<usize>,
    /// 子骨骼索引列表
    pub children_indices: Vec<usize>,
    /// 局部变换（相对于父骨骼）
    pub local_transform: BoneTransform,
    /// 逆绑定矩阵（将顶点从模型空间变换到骨骼空间）
    pub inverse_bind_matrix: Mat4,
}

/// 骨骼变换
#[derive(Clone, Copy, Debug, Default)]
pub struct BoneTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl BoneTransform {
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn identity() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// 转换为 4x4 矩阵
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// 从 4x4 矩阵分解
    pub fn from_matrix(matrix: Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// 线性插值
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

impl Bone {
    pub fn new(name: impl Into<String>, parent_index: Option<usize>) -> Self {
        Self {
            name: name.into(),
            parent_index,
            children_indices: Vec::new(),
            local_transform: BoneTransform::identity(),
            inverse_bind_matrix: Mat4::IDENTITY,
        }
    }
}

// ============================================================================
// 骨骼层级（Skeleton）
// ============================================================================

/// 骨骼层级组件
#[derive(Component)]
pub struct Skeleton {
    /// 所有骨骼
    pub bones: Vec<Bone>,
    /// 骨骼名称到索引的映射
    pub bone_name_to_index: std::collections::HashMap<String, usize>,
    /// 当前姿态的骨骼矩阵（世界空间）
    pub bone_matrices: Vec<Mat4>,
    /// 最终蒙皮矩阵（bone_matrix * inverse_bind_matrix）
    pub skin_matrices: Vec<Mat4>,
    /// GPU 骨骼矩阵缓冲区
    pub matrix_buffer: Option<wgpu::Buffer>,
    /// 是否需要更新 GPU 缓冲区
    pub dirty: bool,
}

impl Skeleton {
    /// 创建新的骨骼层级
    pub fn new(bones: Vec<Bone>) -> Self {
        let bone_count = bones.len();
        let bone_name_to_index = bones
            .iter()
            .enumerate()
            .map(|(i, b)| (b.name.clone(), i))
            .collect();

        Self {
            bones,
            bone_name_to_index,
            bone_matrices: vec![Mat4::IDENTITY; bone_count],
            skin_matrices: vec![Mat4::IDENTITY; bone_count],
            matrix_buffer: None,
            dirty: true,
        }
    }

    /// 获取骨骼数量
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    /// 通过名称获取骨骼索引
    pub fn get_bone_index(&self, name: &str) -> Option<usize> {
        self.bone_name_to_index.get(name).copied()
    }

    /// 获取骨骼
    pub fn get_bone(&self, index: usize) -> Option<&Bone> {
        self.bones.get(index)
    }

    /// 获取可变骨骼
    pub fn get_bone_mut(&mut self, index: usize) -> Option<&mut Bone> {
        self.dirty = true;
        self.bones.get_mut(index)
    }

    /// 设置骨骼局部变换
    pub fn set_bone_transform(&mut self, index: usize, transform: BoneTransform) {
        if let Some(bone) = self.bones.get_mut(index) {
            bone.local_transform = transform;
            self.dirty = true;
        }
    }

    /// 计算所有骨骼的世界空间矩阵
    pub fn compute_bone_matrices(&mut self) {
        for i in 0..self.bones.len() {
            let local_matrix = self.bones[i].local_transform.to_matrix();

            self.bone_matrices[i] = if let Some(parent_idx) = self.bones[i].parent_index {
                // 子骨骼：父骨骼矩阵 * 局部矩阵
                self.bone_matrices[parent_idx] * local_matrix
            } else {
                // 根骨骼：直接使用局部矩阵
                local_matrix
            };
        }
    }

    /// 计算最终蒙皮矩阵
    pub fn compute_skin_matrices(&mut self) {
        for i in 0..self.bones.len() {
            self.skin_matrices[i] = self.bone_matrices[i] * self.bones[i].inverse_bind_matrix;
        }
    }

    /// 更新骨骼姿态（计算世界矩阵和蒙皮矩阵）
    pub fn update_pose(&mut self) {
        self.compute_bone_matrices();
        self.compute_skin_matrices();
    }

    /// 更新 GPU 缓冲区
    pub fn update_gpu_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if !self.dirty {
            return;
        }

        let buffer_size =
            (self.skin_matrices.len() * std::mem::size_of::<Mat4>()) as wgpu::BufferAddress;

        // 创建或更新缓冲区
        if self.matrix_buffer.is_none() {
            self.matrix_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Skeleton Matrix Buffer"),
                size: buffer_size.max(256), // 最小 256 字节
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        // 写入蒙皮矩阵
        if let Some(buffer) = &self.matrix_buffer {
            let data: Vec<[[f32; 4]; 4]> = self
                .skin_matrices
                .iter()
                .map(|m| m.to_cols_array_2d())
                .collect();
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&data));
        }

        self.dirty = false;
    }
}

// ============================================================================
// 骨骼姿态（Pose）
// ============================================================================

/// 骨骼姿态 - 存储所有骨骼的变换状态
#[derive(Clone, Debug)]
pub struct SkeletonPose {
    /// 每个骨骼的局部变换
    pub bone_transforms: Vec<BoneTransform>,
}

impl SkeletonPose {
    /// 从骨骼创建默认姿态
    pub fn from_skeleton(skeleton: &Skeleton) -> Self {
        Self {
            bone_transforms: skeleton.bones.iter().map(|b| b.local_transform).collect(),
        }
    }

    /// 创建指定大小的空姿态
    pub fn with_capacity(bone_count: usize) -> Self {
        Self {
            bone_transforms: vec![BoneTransform::identity(); bone_count],
        }
    }

    /// 线性插值两个姿态
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        assert_eq!(self.bone_transforms.len(), other.bone_transforms.len());

        Self {
            bone_transforms: self
                .bone_transforms
                .iter()
                .zip(other.bone_transforms.iter())
                .map(|(a, b)| a.lerp(b, t))
                .collect(),
        }
    }

    /// 应用姿态到骨骼
    pub fn apply_to_skeleton(&self, skeleton: &mut Skeleton) {
        for (i, transform) in self.bone_transforms.iter().enumerate() {
            if i < skeleton.bones.len() {
                skeleton.bones[i].local_transform = *transform;
            }
        }
        skeleton.dirty = true;
    }
}

// ============================================================================
// GLTF 导入辅助
// ============================================================================

/// 从 GLTF Skin 构建骨骼
///
/// # 参数
/// - `gltf_skin`: GLTF skin 数据
/// - `buffers`: GLTF 缓冲区数据
///
/// # 示例
/// ```ignore
/// let (document, buffers, _) = gltf::import("model.gltf")?;
/// if let Some(skin) = document.skins().next() {
///     let skeleton = build_skeleton_from_gltf(&skin, &buffers);
/// }
/// ```
#[cfg(feature = "gltf")]
pub fn build_skeleton_from_gltf(
    gltf_skin: &gltf::Skin,
    buffers: &[gltf::buffer::Data],
) -> Skeleton {
    let mut bones = Vec::new();
    let joints: Vec<_> = gltf_skin.joints().collect();

    // 读取逆绑定矩阵
    let inverse_bind_matrices: Vec<Mat4> = if let Some(accessor) = gltf_skin.inverse_bind_matrices()
    {
        read_inverse_bind_matrices(&accessor, buffers)
    } else {
        vec![Mat4::IDENTITY; joints.len()]
    };

    // 构建骨骼节点索引映射 (gltf node index -> skeleton bone index)
    let joint_to_index: std::collections::HashMap<usize, usize> = joints
        .iter()
        .enumerate()
        .map(|(i, joint)| (joint.index(), i))
        .collect();

    // 构建父子关系映射 (通过遍历节点的 children)
    let mut parent_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for joint in &joints {
        for child in joint.children() {
            // 如果 child 也是骨骼节点，记录父子关系
            if joint_to_index.contains_key(&child.index()) {
                parent_map.insert(child.index(), joint.index());
            }
        }
    }

    // 创建骨骼
    for (i, joint) in joints.iter().enumerate() {
        let (translation, rotation, scale) = joint.transform().decomposed();

        // 查找父骨骼索引
        let parent_index = parent_map
            .get(&joint.index())
            .and_then(|parent_node_idx| joint_to_index.get(parent_node_idx).copied());

        let name = joint
            .name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("bone_{}", i));

        let mut bone = Bone::new(name, parent_index);

        bone.local_transform = BoneTransform::new(
            Vec3::from(translation),
            Quat::from_array(rotation),
            Vec3::from(scale),
        );
        bone.inverse_bind_matrix = inverse_bind_matrices
            .get(i)
            .copied()
            .unwrap_or(Mat4::IDENTITY);

        bones.push(bone);
    }

    // 建立子骨骼索引
    for i in 0..bones.len() {
        if let Some(parent_idx) = bones[i].parent_index {
            let child_idx = i;
            if parent_idx < bones.len() {
                bones[parent_idx].children_indices.push(child_idx);
            }
        }
    }

    Skeleton::new(bones)
}

/// 读取逆绑定矩阵
#[cfg(feature = "gltf")]
fn read_inverse_bind_matrices(
    accessor: &gltf::Accessor,
    buffers: &[gltf::buffer::Data],
) -> Vec<Mat4> {
    let view = match accessor.view() {
        Some(v) => v,
        None => return vec![Mat4::IDENTITY; accessor.count()],
    };

    let buffer_index = view.buffer().index();
    if buffer_index >= buffers.len() {
        return vec![Mat4::IDENTITY; accessor.count()];
    }

    let buffer = &buffers[buffer_index];
    let offset = view.offset() + accessor.offset();
    let byte_size = accessor.size() * accessor.count();

    if offset + byte_size > buffer.len() {
        return vec![Mat4::IDENTITY; accessor.count()];
    }

    let data = &buffer[offset..offset + byte_size];

    // 逆绑定矩阵是 MAT4 类型 (16 个 f32)
    if data.len() % 64 != 0 {
        return vec![Mat4::IDENTITY; accessor.count()];
    }

    data.chunks_exact(64)
        .map(|chunk| {
            let arr: [[f32; 4]; 4] = [
                [
                    f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
                    f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]),
                    f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]),
                    f32::from_le_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]),
                ],
                [
                    f32::from_le_bytes([chunk[16], chunk[17], chunk[18], chunk[19]]),
                    f32::from_le_bytes([chunk[20], chunk[21], chunk[22], chunk[23]]),
                    f32::from_le_bytes([chunk[24], chunk[25], chunk[26], chunk[27]]),
                    f32::from_le_bytes([chunk[28], chunk[29], chunk[30], chunk[31]]),
                ],
                [
                    f32::from_le_bytes([chunk[32], chunk[33], chunk[34], chunk[35]]),
                    f32::from_le_bytes([chunk[36], chunk[37], chunk[38], chunk[39]]),
                    f32::from_le_bytes([chunk[40], chunk[41], chunk[42], chunk[43]]),
                    f32::from_le_bytes([chunk[44], chunk[45], chunk[46], chunk[47]]),
                ],
                [
                    f32::from_le_bytes([chunk[48], chunk[49], chunk[50], chunk[51]]),
                    f32::from_le_bytes([chunk[52], chunk[53], chunk[54], chunk[55]]),
                    f32::from_le_bytes([chunk[56], chunk[57], chunk[58], chunk[59]]),
                    f32::from_le_bytes([chunk[60], chunk[61], chunk[62], chunk[63]]),
                ],
            ];
            Mat4::from_cols_array_2d(&arr)
        })
        .collect()
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bone_transform_identity() {
        let t = BoneTransform::identity();
        assert_eq!(t.translation, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_bone_transform_to_matrix() {
        let t = BoneTransform::new(Vec3::new(1.0, 2.0, 3.0), Quat::IDENTITY, Vec3::ONE);
        let m = t.to_matrix();
        assert_eq!(m.w_axis.truncate(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_skeleton_bone_hierarchy() {
        let bones = vec![
            Bone::new("root", None),
            Bone::new("spine", Some(0)),
            Bone::new("head", Some(1)),
        ];

        let skeleton = Skeleton::new(bones);

        assert_eq!(skeleton.bone_count(), 3);
        assert_eq!(skeleton.get_bone_index("root"), Some(0));
        assert_eq!(skeleton.get_bone_index("spine"), Some(1));
        assert_eq!(skeleton.get_bone_index("head"), Some(2));
    }

    #[test]
    fn test_skeleton_pose_lerp() {
        let pose1 = SkeletonPose {
            bone_transforms: vec![BoneTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE)],
        };

        let pose2 = SkeletonPose {
            bone_transforms: vec![BoneTransform::new(
                Vec3::new(10.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            )],
        };

        let lerped = pose1.lerp(&pose2, 0.5);
        assert!((lerped.bone_transforms[0].translation.x - 5.0).abs() < 0.001);
    }
}
