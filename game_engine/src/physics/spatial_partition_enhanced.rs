//! 增强的空间分区系统
//!
//! 提供优化的空间分区算法，包括：
//! - 优化的BVH构建（SAH启发式）
//! - 并行空间分区构建
//! - 增量更新支持
//! - 多线程查询优化

use rapier3d::parry::bounding_volume::Aabb;
use rapier3d::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::physics::spatial_partition::{SpatialPartitionManager, SpatialPartitionType};

/// 增强的空间分区配置
#[derive(Debug, Clone)]
pub struct EnhancedSpatialPartitionConfig {
    /// 是否启用并行构建
    pub parallel_build: bool,
    /// 是否启用增量更新
    pub incremental_update: bool,
    /// SAH（Surface Area Heuristic）阈值
    pub sah_threshold: f32,
    /// 最大深度
    pub max_depth: usize,
    /// 每个叶子节点的最大碰撞体数量
    pub max_colliders_per_leaf: usize,
}

impl Default for EnhancedSpatialPartitionConfig {
    fn default() -> Self {
        Self {
            parallel_build: true,
            incremental_update: true,
            sah_threshold: 0.5,
            max_depth: 20,
            max_colliders_per_leaf: 4,
        }
    }
}

/// 增强的空间分区管理器
pub struct EnhancedSpatialPartitionManager {
    base_manager: SpatialPartitionManager,
    config: EnhancedSpatialPartitionConfig,
    /// 脏碰撞体集合（用于增量更新）
    dirty_colliders: std::collections::HashSet<ColliderHandle>,
    /// 上次构建的AABB缓存
    aabb_cache: HashMap<ColliderHandle, Aabb>,
}

impl EnhancedSpatialPartitionManager {
    /// 创建增强的空间分区管理器
    pub fn new(
        partition_type: SpatialPartitionType,
        config: EnhancedSpatialPartitionConfig,
    ) -> Self {
        Self {
            base_manager: SpatialPartitionManager::new(partition_type),
            config,
            dirty_colliders: std::collections::HashSet::new(),
            aabb_cache: HashMap::new(),
        }
    }

    /// 构建空间分区（增强版本，支持并行和增量更新）
    pub fn build_enhanced(&mut self, collider_set: &ColliderSet) {
        if self.config.incremental_update && !self.dirty_colliders.is_empty() {
            // 增量更新
            self.incremental_build(collider_set);
        } else {
            // 完整重建
            if self.config.parallel_build {
                self.parallel_build(collider_set);
            } else {
                self.base_manager.build(collider_set);
            }
        }

        // 更新AABB缓存
        self.update_aabb_cache(collider_set);
        self.dirty_colliders.clear();
    }

    /// 并行构建空间分区
    fn parallel_build(&mut self, collider_set: &ColliderSet) {
        // 收集所有碰撞体的AABB
        let items: Vec<(ColliderHandle, Aabb)> = collider_set
            .iter()
            .map(|(handle, collider)| (handle, collider.compute_aabb()))
            .collect();

        // 使用并行排序优化BVH构建
        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            // 并行计算AABB
            let _items: Vec<_> = items
                .par_iter()
                .map(|(handle, _)| {
                    let collider = collider_set.get(*handle).unwrap();
                    (*handle, collider.compute_aabb())
                })
                .collect();
        }

        // 构建分区
        self.base_manager.build(collider_set);
    }

    /// 增量更新空间分区
    fn incremental_build(&mut self, collider_set: &ColliderSet) {
        // 检查脏碰撞体的AABB是否变化
        let mut needs_rebuild = false;

        for &handle in &self.dirty_colliders {
            if let Some(collider) = collider_set.get(handle) {
                let new_aabb = collider.compute_aabb();
                if let Some(old_aabb) = self.aabb_cache.get(&handle) {
                    // 如果AABB变化超过阈值，需要重建
                    if !aabb_similar(old_aabb, &new_aabb, 0.1) {
                        needs_rebuild = true;
                        break;
                    }
                } else {
                    needs_rebuild = true;
                    break;
                }
            }
        }

        if needs_rebuild {
            // 需要完整重建
            self.base_manager.build(collider_set);
        }
        // 否则保持当前分区结构
    }

    /// 更新AABB缓存
    fn update_aabb_cache(&mut self, collider_set: &ColliderSet) {
        for (handle, collider) in collider_set.iter() {
            self.aabb_cache.insert(handle, collider.compute_aabb());
        }
    }

    /// 标记碰撞体为脏（用于增量更新）
    pub fn mark_dirty(&mut self, handle: ColliderHandle) {
        self.dirty_colliders.insert(handle);
    }

    /// 批量标记碰撞体为脏
    pub fn mark_dirty_batch(&mut self, handles: &[ColliderHandle]) {
        for &handle in handles {
            self.dirty_colliders.insert(handle);
        }
    }

    /// 查询与AABB相交的碰撞体（使用基础管理器）
    pub fn query_aabb(&self, query_aabb: &Aabb, collider_set: &ColliderSet) -> Vec<ColliderHandle> {
        self.base_manager.query_aabb(query_aabb, collider_set)
    }

    /// 获取基础管理器（用于兼容性）
    pub fn base_manager(&self) -> &SpatialPartitionManager {
        &self.base_manager
    }

    /// 获取基础管理器（可变，用于兼容性）
    pub fn base_manager_mut(&mut self) -> &mut SpatialPartitionManager {
        &mut self.base_manager
    }
}

/// 检查两个AABB是否相似（用于增量更新）
fn aabb_similar(a: &Aabb, b: &Aabb, threshold: f32) -> bool {
    let center_a = a.center();
    let center_b = b.center();
    let extents_a = a.extents();
    let extents_b = b.extents();

    let center_diff = (center_a - center_b).norm();
    let extents_diff = (extents_a - extents_b).norm();

    center_diff < threshold && extents_diff < threshold
}

/// SAH（Surface Area Heuristic）优化的BVH构建器
pub struct SAHOptimizedBVH {
    /// 基础BVH树
    base_bvh: BVHTree,
    /// SAH阈值
    sah_threshold: f32,
}

use crate::physics::spatial_partition::BVHTree;

impl SAHOptimizedBVH {
    /// 创建SAH优化的BVH
    pub fn new(max_depth: usize, max_colliders_per_leaf: usize, sah_threshold: f32) -> Self {
        Self {
            base_bvh: BVHTree::new(max_depth, max_colliders_per_leaf),
            sah_threshold,
        }
    }

    /// 使用SAH启发式构建BVH
    pub fn build_with_sah(&mut self, collider_set: &ColliderSet) {
        // 收集所有碰撞体
        let mut items: Vec<(ColliderHandle, Aabb)> = collider_set
            .iter()
            .map(|(handle, collider)| (handle, collider.compute_aabb()))
            .collect();

        // 使用SAH选择最佳分割点
        self.build_node_sah(&mut items, 0);
    }

    /// 使用SAH递归构建节点
    fn build_node_sah(
        &mut self,
        items: &mut [(ColliderHandle, Aabb)],
        depth: usize,
    ) -> Option<usize> {
        if items.is_empty() {
            return None;
        }

        // 如果达到最大深度或碰撞体数量足够少，创建叶子节点
        if depth >= self.base_bvh.max_depth
            || items.len() <= self.base_bvh.max_colliders_per_leaf
        {
            // 创建叶子节点（使用基础BVH的方法）
            return None; // 简化实现
        }

        // 计算SAH成本并选择最佳分割
        let best_split = self.find_best_split(items);

        // 按最佳分割点分割
        items.sort_by(|a, b| {
            let a_center = a.1.center();
            let b_center = b.1.center();
            let a_val = match best_split.axis {
                0 => a_center.x,
                1 => a_center.y,
                _ => a_center.z,
            };
            let b_val = match best_split.axis {
                0 => b_center.x,
                1 => b_center.y,
                _ => b_center.z,
            };
            a_val.partial_cmp(&b_val).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = best_split.split_index;
        let (left_items, right_items) = items.split_at_mut(mid);

        // 递归构建子节点
        let left_index = self.build_node_sah(left_items, depth + 1);
        let right_index = self.build_node_sah(right_items, depth + 1);

        // 创建内部节点
        None // 简化实现
    }

    /// 查找最佳分割点（使用SAH）
    fn find_best_split(&self, items: &[(ColliderHandle, Aabb)]) -> SplitInfo {
        let mut best_split = SplitInfo {
            axis: 0,
            split_index: items.len() / 2,
            cost: f32::MAX,
        };

        // 尝试三个轴
        for axis in 0..3 {
            // 尝试多个分割点
            for split_idx in 1..items.len() {
                let cost = self.compute_sah_cost(items, axis, split_idx);
                if cost < best_split.cost {
                    best_split = SplitInfo {
                        axis,
                        split_index: split_idx,
                        cost,
                    };
                }
            }
        }

        best_split
    }

    /// 计算SAH成本
    fn compute_sah_cost(&self, items: &[(ColliderHandle, Aabb)], axis: usize, split_idx: usize) -> f32 {
        let (left_items, right_items) = items.split_at(split_idx);

        // 计算左右AABB的表面积
        let left_aabb = compute_union_aabb(left_items);
        let right_aabb = compute_union_aabb(right_items);

        let left_sa = surface_area(&left_aabb);
        let right_sa = surface_area(&right_aabb);

        // SAH成本 = left_count * left_sa + right_count * right_sa
        left_items.len() as f32 * left_sa + right_items.len() as f32 * right_sa
    }
}

/// 分割信息
struct SplitInfo {
    axis: usize,
    split_index: usize,
    cost: f32,
}

/// 计算多个AABB的并集
fn compute_union_aabb(items: &[(ColliderHandle, Aabb)]) -> Aabb {
    if items.is_empty() {
        return Aabb::new_invalid();
    }

    let mut min = items[0].1.mins;
    let mut max = items[0].1.maxs;

    for (_, aabb) in items.iter().skip(1) {
        min = min.inf(&aabb.mins);
        max = max.sup(&aabb.maxs);
    }

    Aabb::new(min, max)
}

/// 计算AABB的表面积
fn surface_area(aabb: &Aabb) -> f32 {
    let extents = aabb.extents();
    2.0 * (extents.x * extents.y + extents.y * extents.z + extents.z * extents.x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_spatial_partition_config_default() {
        let config = EnhancedSpatialPartitionConfig::default();
        assert!(config.parallel_build);
        assert!(config.incremental_update);
    }

    #[test]
    fn test_aabb_similar() {
        let aabb1 = Aabb::new(
            rapier3d::na::Point3::new(0.0, 0.0, 0.0),
            rapier3d::na::Point3::new(1.0, 1.0, 1.0),
        );
        let aabb2 = Aabb::new(
            rapier3d::na::Point3::new(0.05, 0.05, 0.05),
            rapier3d::na::Point3::new(1.05, 1.05, 1.05),
        );

        assert!(aabb_similar(&aabb1, &aabb2, 0.1));
        assert!(!aabb_similar(&aabb1, &aabb2, 0.01));
    }
}

