//  SoA (Struct of Arrays) 布局优化
//
//  将传统结构体数组转换为数组结构体，提高缓存局部性和SIMD性能

use crate::ecs::Transform;
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
    pub fn from_world(world: &mut World) -> Self {
        let mut storage = Self::new();

        // 遍历所有实体，查找有Transform组件的
        let mut query = world.query::<(Entity, &Transform)>();
        for (entity, transform) in query.iter(world) {
            let index = storage.positions.len();
            storage.positions.push(transform.pos);
            storage.rotations.push(transform.rot);
            storage.scales.push(transform.scale);
            storage.entity_to_index.insert(entity, index);
            storage.index_to_entity.push(entity);
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

    /// 批量更新位置
    pub fn update_positions_batch<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec3),
    {
        for pos in &mut self.positions {
            f(pos);
        }
    }

    /// 批量更新旋转
    pub fn update_rotations_batch<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Quat),
    {
        for rot in &mut self.rotations {
            f(rot);
        }
    }

    /// 批量更新缩放
    pub fn update_scales_batch<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec3),
    {
        for scale in &mut self.scales {
            f(scale);
        }
    }

    /// 获取实体的变换
    pub fn get_transform(&self, entity: Entity) -> Option<Transform> {
        self.entity_to_index.get(&entity).map(|&index| Transform {
            pos: self.positions[index],
            rot: self.rotations[index],
            scale: self.scales[index],
        })
    }

    /// 设置实体的变换
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
#[derive(Default)]
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

impl SoAVelocityStorage {
    /// 创建新的SoA存储
    pub fn new() -> Self {
        Self::default()
    }

    /// 从ECS查询构建SoA布局
    pub fn from_world(_world: &World) -> Self {
        

        // 这里需要Velocity组件的定义
        // 暂时留空
        Self::new()
    }

    /// 获取实体数量
    pub fn len(&self) -> usize {
        self.linear_velocities.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.linear_velocities.is_empty()
    }
}


/// SoA布局管理器
pub struct SoALayoutManager {
    transforms: SoATransformStorage,
    velocities: SoAVelocityStorage,
    enabled: bool,
}

impl SoALayoutManager {
    /// 创建新的SoA布局管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 启用SoA布局
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// 禁用SoA布局
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
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

impl Default for SoALayoutManager {
    fn default() -> Self {
        Self {
            transforms: SoATransformStorage::new(),
            velocities: SoAVelocityStorage::new(),
            enabled: false,
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

    #[test]
    fn test_soa_transform_storage() {
        let mut storage = SoATransformStorage::new();

        // 创建测试实体
        let entity = Entity::from_raw_u32(1).unwrap();

        let transform = Transform {
            pos: Vec3::new(1.0, 2.0, 3.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        };

        storage.add_entity(entity, transform);

        assert_eq!(storage.len(), 1);
        assert!(!storage.is_empty());

        // 测试获取变换
        let retrieved = storage.get_transform(entity).unwrap();
        assert_eq!(retrieved.pos, transform.pos);

        // 测试设置变换
        let new_transform = Transform {
            pos: Vec3::new(4.0, 5.0, 6.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        assert!(storage.set_transform(entity, new_transform));

        let retrieved = storage.get_transform(entity).unwrap();
        assert_eq!(retrieved.pos, new_transform.pos);

        // 测试移除实体
        assert!(storage.remove_entity(entity));
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
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
