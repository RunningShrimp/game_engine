//! 批次构建器模块
//!
//! 提供便捷的批次构建 API，简化实例化渲染的使用流程。
//!
//! ## 设计目标
//!
//! - 流式 API 构建批次
//! - 支持静态/动态批次
//! - 自动 LOD 选择
//! - 与 GPU Driven 剔除集成
//!
//! ## 使用示例
//!
//! ```ignore
//! let batch = BatchBuilder::new()
//!     .mesh(mesh_handle)
//!     .material(material_handle)
//!     .add_instance(transform1)
//!     .add_instance(transform2)
//!     .static_batch(true)
//!     .build(&mut batch_manager);
//! ```

use crate::impl_default;
use glam::{Mat4, Quat, Vec3};
use std::sync::Arc;

use super::instance_batch::{BatchKey, BatchManager};
use super::mesh::GpuMesh;
use super::pbr_renderer::Instance3D;

// ============================================================================
// 批次构建器
// ============================================================================

/// 批次构建器 - 流式 API 构建实例批次
pub struct BatchBuilder {
    /// 网格引用
    mesh: Option<Arc<GpuMesh>>,
    /// 材质绑定组
    material_bind_group: Option<Arc<wgpu::BindGroup>>,
    /// 网格 ID
    mesh_id: u64,
    /// 材质 ID
    material_id: u64,
    /// 待添加的实例数据
    instances: Vec<InstanceData>,
    /// 是否为静态批次
    is_static: bool,
    /// LOD 级别
    lod_level: u32,
    /// 是否启用剔除
    culling_enabled: bool,
    /// 包围球半径（用于剔除）
    bounding_radius: f32,
}

/// 实例数据（用于构建）
#[derive(Clone)]
pub struct InstanceData {
    /// 位置
    pub position: Vec3,
    /// 旋转
    pub rotation: Quat,
    /// 缩放
    pub scale: Vec3,
    /// 自定义数据
    pub custom_data: Option<[f32; 4]>,
}

impl_default!(InstanceData {
    position: Vec3::ZERO,
    rotation: Quat::IDENTITY,
    scale: Vec3::ONE,
    custom_data: None,
});

impl InstanceData {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
            custom_data: None,
        }
    }

    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    pub fn from_transform(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self::new(position, rotation, scale)
    }

    /// 转换为模型矩阵
    pub fn to_model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// 转换为 Instance3D
    pub fn to_instance3d(&self) -> Instance3D {
        Instance3D {
            model: self.to_model_matrix().to_cols_array_2d(),
        }
    }
}

impl_default!(BatchBuilder {
    mesh: None,
    material_bind_group: None,
    mesh_id: 0,
    material_id: 0,
    instances: Vec::new(),
    is_static: false,
    lod_level: 0,
    culling_enabled: true,
    bounding_radius: 1.0,
});

impl BatchBuilder {
    /// 创建新的批次构建器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置网格
    pub fn mesh(mut self, mesh: Arc<GpuMesh>, mesh_id: u64) -> Self {
        self.mesh = Some(mesh);
        self.mesh_id = mesh_id;
        self
    }

    /// 设置材质
    pub fn material(mut self, bind_group: Arc<wgpu::BindGroup>, material_id: u64) -> Self {
        self.material_bind_group = Some(bind_group);
        self.material_id = material_id;
        self
    }

    /// 添加实例（使用位置）
    pub fn add_instance_position(mut self, position: Vec3) -> Self {
        self.instances.push(InstanceData::from_position(position));
        self
    }

    /// 添加实例（使用完整变换）
    pub fn add_instance(mut self, position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        self.instances
            .push(InstanceData::new(position, rotation, scale));
        self
    }

    /// 添加实例数据
    pub fn add_instance_data(mut self, data: InstanceData) -> Self {
        self.instances.push(data);
        self
    }

    /// 批量添加实例（位置数组）
    pub fn add_instances_positions(mut self, positions: &[Vec3]) -> Self {
        for pos in positions {
            self.instances.push(InstanceData::from_position(*pos));
        }
        self
    }

    /// 批量添加实例数据
    pub fn add_instances(mut self, instances: Vec<InstanceData>) -> Self {
        self.instances.extend(instances);
        self
    }

    /// 设置为静态批次（不常更新）
    pub fn static_batch(mut self, is_static: bool) -> Self {
        self.is_static = is_static;
        self
    }

    /// 设置 LOD 级别
    pub fn lod_level(mut self, level: u32) -> Self {
        self.lod_level = level;
        self
    }

    /// 设置是否启用剔除
    pub fn culling(mut self, enabled: bool) -> Self {
        self.culling_enabled = enabled;
        self
    }

    /// 设置包围球半径
    pub fn bounding_radius(mut self, radius: f32) -> Self {
        self.bounding_radius = radius;
        self
    }

    /// 构建批次并添加到管理器
    pub fn build(self, batch_manager: &mut BatchManager) -> Option<BatchKey> {
        let mesh = self.mesh?;
        let material_bind_group = self.material_bind_group?;

        let key = BatchKey {
            mesh_id: self.mesh_id,
            material_id: self.material_id,
        };

        // 获取或创建批次
        let batch = batch_manager.get_or_create_batch(key, mesh, material_bind_group);
        batch.is_static = self.is_static;

        // 添加所有实例
        for instance_data in self.instances {
            batch.add_instance(instance_data.to_instance3d());
        }

        // 标记为可见
        batch_manager.mark_visible(key);

        Some(key)
    }
}

// ============================================================================
// 网格批次生成器（用于程序化生成大量实例）
// ============================================================================

/// 网格批次生成器 - 程序化生成大量实例
pub struct MeshBatchGenerator {
    /// 随机种子
    seed: u64,
    /// 位置范围
    position_range: (Vec3, Vec3),
    /// 缩放范围
    scale_range: (f32, f32),
    /// 是否随机旋转
    random_rotation: bool,
}

impl_default!(MeshBatchGenerator {
    seed: 12345,
    position_range: (Vec3::splat(-100.0), Vec3::splat(100.0)),
    scale_range: (0.5, 2.0),
    random_rotation: true,
});

impl MeshBatchGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置随机种子
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// 设置位置范围
    pub fn position_range(mut self, min: Vec3, max: Vec3) -> Self {
        self.position_range = (min, max);
        self
    }

    /// 设置缩放范围
    pub fn scale_range(mut self, min: f32, max: f32) -> Self {
        self.scale_range = (min, max);
        self
    }

    /// 设置是否随机旋转
    pub fn random_rotation(mut self, enabled: bool) -> Self {
        self.random_rotation = enabled;
        self
    }

    /// 生成实例数据
    pub fn generate(&self, count: usize) -> Vec<InstanceData> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut instances = Vec::with_capacity(count);
        let (pos_min, pos_max) = self.position_range;
        let (scale_min, scale_max) = self.scale_range;

        for i in 0..count {
            // 简单的伪随机数生成
            let mut hasher = DefaultHasher::new();
            (self.seed, i as u64).hash(&mut hasher);
            let hash = hasher.finish();

            // 从哈希值提取随机数
            let r1 = ((hash >> 0) & 0xFFFF) as f32 / 65535.0;
            let r2 = ((hash >> 16) & 0xFFFF) as f32 / 65535.0;
            let r3 = ((hash >> 32) & 0xFFFF) as f32 / 65535.0;
            let r4 = ((hash >> 48) & 0xFFFF) as f32 / 65535.0;

            // 计算位置
            let position = Vec3::new(
                pos_min.x + (pos_max.x - pos_min.x) * r1,
                pos_min.y + (pos_max.y - pos_min.y) * r2,
                pos_min.z + (pos_max.z - pos_min.z) * r3,
            );

            // 计算缩放
            let scale_factor = scale_min + (scale_max - scale_min) * r4;
            let scale = Vec3::splat(scale_factor);

            // 计算旋转
            let rotation = if self.random_rotation {
                // 使用额外的哈希值生成旋转
                let mut hasher2 = DefaultHasher::new();
                (self.seed + 1, i as u64).hash(&mut hasher2);
                let hash2 = hasher2.finish();

                let angle = (hash2 & 0xFFFF) as f32 / 65535.0 * std::f32::consts::TAU;
                let axis_x = ((hash2 >> 16) & 0xFFFF) as f32 / 65535.0 * 2.0 - 1.0;
                let axis_y = ((hash2 >> 32) & 0xFFFF) as f32 / 65535.0 * 2.0 - 1.0;
                let axis_z = ((hash2 >> 48) & 0xFFFF) as f32 / 65535.0 * 2.0 - 1.0;

                let axis = Vec3::new(axis_x, axis_y, axis_z).normalize_or_zero();
                if axis.length_squared() > 0.001 {
                    Quat::from_axis_angle(axis, angle)
                } else {
                    Quat::IDENTITY
                }
            } else {
                Quat::IDENTITY
            };

            instances.push(InstanceData::new(position, rotation, scale));
        }

        instances
    }

    /// 生成网格排列的实例数据
    pub fn generate_grid(
        &self,
        grid_size: (usize, usize, usize),
        spacing: f32,
    ) -> Vec<InstanceData> {
        let (gx, gy, gz) = grid_size;
        let mut instances = Vec::with_capacity(gx * gy * gz);

        let offset = Vec3::new(
            (gx as f32 - 1.0) * spacing * 0.5,
            (gy as f32 - 1.0) * spacing * 0.5,
            (gz as f32 - 1.0) * spacing * 0.5,
        );

        for x in 0..gx {
            for y in 0..gy {
                for z in 0..gz {
                    let position = Vec3::new(
                        x as f32 * spacing - offset.x,
                        y as f32 * spacing - offset.y,
                        z as f32 * spacing - offset.z,
                    );
                    instances.push(InstanceData::from_position(position));
                }
            }
        }

        instances
    }
}

// ============================================================================
// LOD 批次构建器
// ============================================================================

/// LOD 网格配置
#[derive(Clone)]
pub struct LodMeshConfig {
    /// LOD 级别
    pub level: u32,
    /// 网格
    pub mesh: Arc<GpuMesh>,
    /// 网格 ID
    pub mesh_id: u64,
    /// 最大距离（超过此距离使用下一级 LOD）
    pub max_distance: f32,
}

/// LOD 批次构建器
pub struct LodBatchBuilder {
    /// LOD 网格配置列表（按距离排序）
    lod_meshes: Vec<LodMeshConfig>,
    /// 材质绑定组
    material_bind_group: Option<Arc<wgpu::BindGroup>>,
    /// 材质 ID
    material_id: u64,
    /// 实例数据
    instances: Vec<(InstanceData, f32)>, // (数据, 到相机距离)
}

impl Default for LodBatchBuilder {
    fn default() -> Self {
        Self {
            lod_meshes: Vec::new(),
            material_bind_group: None,
            material_id: 0,
            instances: Vec::new(),
        }
    }
}

impl LodBatchBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加 LOD 级别
    pub fn add_lod(
        mut self,
        level: u32,
        mesh: Arc<GpuMesh>,
        mesh_id: u64,
        max_distance: f32,
    ) -> Self {
        self.lod_meshes.push(LodMeshConfig {
            level,
            mesh,
            mesh_id,
            max_distance,
        });
        // 按距离排序
        self.lod_meshes
            .sort_by(|a, b| a.max_distance.partial_cmp(&b.max_distance).unwrap());
        self
    }

    /// 设置材质
    pub fn material(mut self, bind_group: Arc<wgpu::BindGroup>, material_id: u64) -> Self {
        self.material_bind_group = Some(bind_group);
        self.material_id = material_id;
        self
    }

    /// 添加实例（带距离信息）
    pub fn add_instance(mut self, data: InstanceData, camera_distance: f32) -> Self {
        self.instances.push((data, camera_distance));
        self
    }

    /// 根据相机位置计算距离并添加实例
    pub fn add_instance_with_camera(mut self, data: InstanceData, camera_pos: Vec3) -> Self {
        let distance = data.position.distance(camera_pos);
        self.instances.push((data, distance));
        self
    }

    /// 选择合适的 LOD 级别
    fn select_lod(&self, distance: f32) -> Option<&LodMeshConfig> {
        for lod in &self.lod_meshes {
            if distance <= lod.max_distance {
                return Some(lod);
            }
        }
        // 返回最后一个 LOD（最低质量）
        self.lod_meshes.last()
    }

    /// 构建所有 LOD 批次
    pub fn build(self, batch_manager: &mut BatchManager) -> Vec<BatchKey> {
        let material_bind_group = match self.material_bind_group {
            Some(ref bg) => bg.clone(),
            None => return Vec::new(),
        };

        let mut keys = Vec::new();

        // 按 LOD 分组实例
        let mut lod_instances: std::collections::HashMap<u64, Vec<InstanceData>> =
            std::collections::HashMap::new();

        for (data, distance) in &self.instances {
            if let Some(lod) = self.select_lod(*distance) {
                lod_instances
                    .entry(lod.mesh_id)
                    .or_default()
                    .push(data.clone());
            }
        }

        // 为每个 LOD 创建批次
        for lod in &self.lod_meshes {
            if let Some(instances) = lod_instances.get(&lod.mesh_id) {
                if instances.is_empty() {
                    continue;
                }

                let key = BatchKey {
                    mesh_id: lod.mesh_id,
                    material_id: self.material_id,
                };

                let batch = batch_manager.get_or_create_batch(
                    key,
                    lod.mesh.clone(),
                    material_bind_group.clone(),
                );

                for instance_data in instances {
                    batch.add_instance(instance_data.to_instance3d());
                }

                batch_manager.mark_visible(key);
                keys.push(key);
            }
        }

        keys
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_data_to_matrix() {
        let data = InstanceData::new(Vec3::new(1.0, 2.0, 3.0), Quat::IDENTITY, Vec3::ONE);
        let matrix = data.to_model_matrix();
        assert_eq!(matrix.w_axis.truncate(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_mesh_batch_generator_grid() {
        let generator = MeshBatchGenerator::new();
        let instances = generator.generate_grid((3, 3, 3), 2.0);
        assert_eq!(instances.len(), 27);
    }

    #[test]
    fn test_mesh_batch_generator_random() {
        let generator = MeshBatchGenerator::new()
            .seed(42)
            .position_range(Vec3::ZERO, Vec3::splat(10.0));

        let instances = generator.generate(100);
        assert_eq!(instances.len(), 100);

        // 验证位置在范围内
        for inst in &instances {
            assert!(inst.position.x >= 0.0 && inst.position.x <= 10.0);
            assert!(inst.position.y >= 0.0 && inst.position.y <= 10.0);
            assert!(inst.position.z >= 0.0 && inst.position.z <= 10.0);
        }
    }
}
