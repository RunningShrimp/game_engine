//! SoA (Structure of Arrays) 布局实现
//!
//! 将组件数据从AoS (Array of Structures) 布局转换为SoA布局，
//! 提升缓存命中率和SIMD友好性

use crate::impl_default;
use super::{Transform, Velocity};
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};

/// SoA布局的Transform组件存储
#[derive(Default)]
pub struct SoATransformStorage {
    /// 位置数组
    pub positions: Vec<Vec3>,
    /// 旋转数组
    pub rotations: Vec<Quat>,
    /// 缩放数组
    pub scales: Vec<Vec3>,
    /// 实体ID到索引的映射
    pub entity_to_index: std::collections::HashMap<Entity, usize>,
    /// 索引到实体ID的映射
    pub index_to_entity: Vec<Entity>,
}

impl SoATransformStorage {
    /// 创建新的SoA存储
    pub fn new() -> Self {
        Self::default()
    }

    /// 从ECS查询构建SoA布局
    pub fn from_world(world: &World) -> Self {
        let mut storage = Self::new();

        // 遍历所有实体，查找有Transform组件的
        for entity_ref in world.iter_entities() {
            let entity = entity_ref.id();
            if let Some(transform) = world.get::<Transform>(entity) {
                let index = storage.positions.len();
                storage.positions.push(transform.pos);
                storage.rotations.push(transform.rot);
                storage.scales.push(transform.scale);
                storage.entity_to_index.insert(entity, index);
                storage.index_to_entity.push(entity);
            }
        }

        storage
    }

    /// 添加实体
    pub fn add_entity(&mut self, entity: Entity, transform: Transform) {
        let index = self.positions.len();
        self.positions.push(transform.pos);
        self.rotations.push(transform.rot);
        self.scales.push(transform.scale);
        self.entity_to_index.insert(entity, index);
        self.index_to_entity.push(entity);
    }

    /// 移除实体
    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        if let Some(&index) = self.entity_to_index.get(&entity) {
            // 使用swap-remove保持数组紧凑
            let last_index = self.positions.len() - 1;

            if index != last_index {
                // 交换最后一个元素到当前位置
                self.positions.swap(index, last_index);
                self.rotations.swap(index, last_index);
                self.scales.swap(index, last_index);

                // 更新最后一个实体的索引映射
                let last_entity = self.index_to_entity[last_index];
                self.entity_to_index.insert(last_entity, index);
                self.index_to_entity.swap(index, last_index);
            }

            // 移除最后一个元素
            self.positions.pop();
            self.rotations.pop();
            self.scales.pop();
            self.index_to_entity.pop();
            self.entity_to_index.remove(&entity);

            true
        } else {
            false
        }
    }

    /// 获取实体数量
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// 批量更新位置（SIMD友好）
    pub fn update_positions_batch<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec3),
    {
        for pos in &mut self.positions {
            f(pos);
        }
    }

    /// 批量更新旋转（SIMD友好）
    pub fn update_rotations_batch<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Quat),
    {
        for rot in &mut self.rotations {
            f(rot);
        }
    }

    /// 批量更新缩放（SIMD友好）
    pub fn update_scales_batch<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec3),
    {
        for scale in &mut self.scales {
            f(scale);
        }
    }

    /// 获取实体的Transform
    pub fn get_transform(&self, entity: Entity) -> Option<Transform> {
        self.entity_to_index.get(&entity).map(|&index| Transform {
            pos: self.positions[index],
            rot: self.rotations[index],
            scale: self.scales[index],
        })
    }

    /// 设置实体的Transform
    pub fn set_transform(&mut self, entity: Entity, transform: Transform) -> bool {
        if let Some(&index) = self.entity_to_index.get(&entity) {
            self.positions[index] = transform.pos;
            self.rotations[index] = transform.rot;
            self.scales[index] = transform.scale;
            true
        } else {
            false
        }
    }

    /// 转换为ECS组件（用于同步回ECS）
    pub fn sync_to_ecs(&self, mut commands: Commands) {
        for (entity, &index) in &self.entity_to_index {
            commands.entity(*entity).insert(Transform {
                pos: self.positions[index],
                rot: self.rotations[index],
                scale: self.scales[index],
            });
        }
    }
}

/// SoA布局的Velocity组件存储
pub struct SoAVelocityStorage {
    /// 线性速度数组
    pub linear_velocities: Vec<Vec3>,
    /// 角速度数组
    pub angular_velocities: Vec<Vec3>,
    /// 实体ID到索引的映射
    pub entity_to_index: std::collections::HashMap<Entity, usize>,
    /// 索引到实体ID的映射
    pub index_to_entity: Vec<Entity>,
}

impl_default!(SoAVelocityStorage {
    linear_velocities: Vec::new(),
    angular_velocities: Vec::new(),
    entity_to_index: std::collections::HashMap::new(),
    index_to_entity: Vec::new(),
});

impl SoAVelocityStorage {
    /// 创建新的SoA存储
    pub fn new() -> Self {
        Self::default()
    }

    /// 从ECS查询构建SoA布局
    pub fn from_world(world: &World) -> Self {
        let mut storage = Self::new();

        // 遍历所有实体，查找有Velocity组件的
        for entity_ref in world.iter_entities() {
            let entity = entity_ref.id();
            if let Some(velocity) = world.get::<Velocity>(entity) {
                let index = storage.linear_velocities.len();
                storage.linear_velocities.push(velocity.lin);
                storage.angular_velocities.push(velocity.ang);
                storage.entity_to_index.insert(entity, index);
                storage.index_to_entity.push(entity);
            }
        }

        storage
    }

    /// 批量更新速度（SIMD友好）
    pub fn update_velocities_batch<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec3, &mut Vec3),
    {
        for (lin, ang) in self
            .linear_velocities
            .iter_mut()
            .zip(self.angular_velocities.iter_mut())
        {
            f(lin, ang);
        }
    }

    /// 获取实体数量
    pub fn len(&self) -> usize {
        self.linear_velocities.len()
    }
}

/// SoA布局管理器
pub struct SoALayoutManager {
    transforms: SoATransformStorage,
    velocities: SoAVelocityStorage,
    enabled: bool,
}

impl_default!(SoALayoutManager {
    transforms: SoATransformStorage::new(),
    velocities: SoAVelocityStorage::new(),
    enabled: false,
});

impl SoALayoutManager {
    /// 创建新的SoA布局管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 启用SoA布局
    pub fn enable(&mut self) {
        self.enabled = true;
        tracing::info!(target: "soa_layout", "SoA layout enabled");
    }

    /// 禁用SoA布局
    pub fn disable(&mut self) {
        self.enabled = false;
        tracing::info!(target: "soa_layout", "SoA layout disabled");
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 从ECS世界构建SoA布局
    pub fn build_from_world(&mut self, world: &World) {
        if !self.enabled {
            return;
        }

        // 构建Transform的SoA布局
        self.transforms = SoATransformStorage::from_world(world);

        // 构建Velocity的SoA布局
        self.velocities = SoAVelocityStorage::from_world(world);

        tracing::info!(target: "soa_layout", 
            "Built SoA layout: {} transforms, {} velocities",
            self.transforms.len(),
            self.velocities.len());
    }

    /// 获取Transform存储的可变引用
    pub fn transforms_mut(&mut self) -> &mut SoATransformStorage {
        &mut self.transforms
    }

    /// 获取Velocity存储的可变引用
    pub fn velocities_mut(&mut self) -> &mut SoAVelocityStorage {
        &mut self.velocities
    }

    /// 同步SoA布局回ECS
    pub fn sync_to_ecs(&self, mut commands: Commands) {
        if !self.enabled {
            return;
        }

        self.transforms.sync_to_ecs(commands);
    }

    /// 获取统计信息
    pub fn stats(&self) -> SoAStats {
        SoAStats {
            transform_count: self.transforms.len(),
            velocity_count: self.velocities.len(),
            enabled: self.enabled,
        }
    }
}

/// SoA布局统计信息
#[derive(Debug, Clone)]
pub struct SoAStats {
    pub transform_count: usize,
    pub velocity_count: usize,
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_soa_transform_storage() {
        let mut storage = SoATransformStorage::new();

        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        storage.add_entity(
            entity1,
            Transform {
                pos: Vec3::new(1.0, 2.0, 3.0),
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
        );

        storage.add_entity(
            entity2,
            Transform {
                pos: Vec3::new(4.0, 5.0, 6.0),
                rot: Quat::IDENTITY,
                scale: Vec3::splat(2.0),
            },
        );

        assert_eq!(storage.len(), 2);

        let transform1 = storage.get_transform(entity1).unwrap();
        assert_eq!(transform1.pos, Vec3::new(1.0, 2.0, 3.0));

        // 测试批量更新
        storage.update_positions_batch(|pos| {
            pos.x += 1.0;
        });

        let transform1_updated = storage.get_transform(entity1).unwrap();
        assert_eq!(transform1_updated.pos.x, 2.0);

        // 测试移除
        assert!(storage.remove_entity(entity1));
        assert_eq!(storage.len(), 1);
        assert!(storage.get_transform(entity1).is_none());
    }

    #[test]
    fn test_soa_from_world() {
        let mut world = World::new();

        let entity1 = world
            .spawn(Transform {
                pos: Vec3::new(1.0, 2.0, 3.0),
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            })
            .id();

        let entity2 = world
            .spawn(Transform {
                pos: Vec3::new(4.0, 5.0, 6.0),
                rot: Quat::IDENTITY,
                scale: Vec3::splat(2.0),
            })
            .id();

        let storage = SoATransformStorage::from_world(&world);
        assert_eq!(storage.len(), 2);

        let transform1 = storage.get_transform(entity1).unwrap();
        assert_eq!(transform1.pos, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_soa_layout_manager() {
        let mut manager = SoALayoutManager::new();
        assert!(!manager.is_enabled());

        manager.enable();
        assert!(manager.is_enabled());

        let stats = manager.stats();
        assert_eq!(stats.transform_count, 0);
        assert_eq!(stats.velocity_count, 0);
    }
}
