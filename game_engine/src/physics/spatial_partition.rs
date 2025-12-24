//! 空间分区系统
//!
//! 提供BVH（Bounding Volume Hierarchy）和空间哈希等空间分区算法，
//! 用于优化碰撞检测性能。

use rapier3d::parry::bounding_volume::Aabb;
use rapier3d::prelude::*;
use std::collections::HashMap;

/// 空间分区类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialPartitionType {
    /// BVH（Bounding Volume Hierarchy）
    BVH,
    /// 空间哈希（Spatial Hash）
    SpatialHash,
    /// 四叉树（2D）或八叉树（3D）
    Octree,
}

/// BVH节点
#[derive(Debug, Clone)]
struct BVHNode {
    /// 包围盒
    aabb: Aabb,
    /// 左子节点索引（None表示叶子节点）
    left: Option<usize>,
    /// 右子节点索引（None表示叶子节点）
    right: Option<usize>,
    /// 碰撞体句柄列表（叶子节点）
    colliders: Vec<ColliderHandle>,
    /// 节点深度
    depth: usize,
}

/// BVH树
#[derive(Debug)]
pub struct BVHTree {
    /// 节点列表
    nodes: Vec<BVHNode>,
    /// 根节点索引
    root: Option<usize>,
    /// 最大深度
    max_depth: usize,
    /// 每个叶子节点的最大碰撞体数量
    max_colliders_per_leaf: usize,
}

impl BVHTree {
    /// 创建新的BVH树
    pub fn new(max_depth: usize, max_colliders_per_leaf: usize) -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
            max_depth,
            max_colliders_per_leaf,
        }
    }

    /// 构建BVH树
    pub fn build(&mut self, collider_set: &ColliderSet) {
        if collider_set.len() == 0 {
            return;
        }

        // 收集所有碰撞体的AABB和句柄
        let mut items: Vec<(ColliderHandle, Aabb)> = collider_set
            .iter()
            .map(|(handle, collider)| (handle, collider.compute_aabb()))
            .collect();

        // 构建BVH树
        self.root = Some(self.build_node(&mut items, 0));
    }

    /// 递归构建BVH节点
    fn build_node(&mut self, items: &mut [(ColliderHandle, Aabb)], depth: usize) -> usize {
        if items.is_empty() {
            return usize::MAX; // 无效索引
        }

        // 如果达到最大深度或碰撞体数量足够少，创建叶子节点
        if depth >= self.max_depth || items.len() <= self.max_colliders_per_leaf {
            let aabb = self.compute_union_aabb(items);
            let colliders: Vec<ColliderHandle> = items.iter().map(|(handle, _)| *handle).collect();

            let node_index = self.nodes.len();
            self.nodes.push(BVHNode {
                aabb,
                left: None,
                right: None,
                colliders,
                depth,
            });
            return node_index;
        }

        // 选择分割轴（选择最长的轴）
        let aabb = self.compute_union_aabb(items);
        let extents = aabb.extents();
        let split_axis = if extents.x >= extents.y && extents.x >= extents.z {
            0 // X轴
        } else if extents.y >= extents.z {
            1 // Y轴
        } else {
            2 // Z轴
        };

        // 按分割轴排序
        items.sort_by(|a, b| {
            let a_center = a.1.center();
            let b_center = b.1.center();
            let a_val = match split_axis {
                0 => a_center.x,
                1 => a_center.y,
                _ => a_center.z,
            };
            let b_val = match split_axis {
                0 => b_center.x,
                1 => b_center.y,
                _ => b_center.z,
            };
            a_val.partial_cmp(&b_val).unwrap_or(std::cmp::Ordering::Equal)
        });

        // 分割为左右两部分
        let mid = items.len() / 2;
        let (left_items, right_items) = items.split_at_mut(mid);

        // 递归构建左右子节点
        let left_index = self.build_node(left_items, depth + 1);
        let right_index = self.build_node(right_items, depth + 1);

        // 创建内部节点
        let aabb = self.compute_union_aabb(items);
        let node_index = self.nodes.len();
        self.nodes.push(BVHNode {
            aabb,
            left: if left_index != usize::MAX {
                Some(left_index)
            } else {
                None
            },
            right: if right_index != usize::MAX {
                Some(right_index)
            } else {
                None
            },
            colliders: Vec::new(),
            depth,
        });

        node_index
    }

    /// 计算多个AABB的并集
    fn compute_union_aabb(&self, items: &[(ColliderHandle, Aabb)]) -> Aabb {
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

    /// 查询与AABB相交的碰撞体
    pub fn query_aabb(&self, query_aabb: &Aabb, collider_set: &ColliderSet) -> Vec<ColliderHandle> {
        let mut results = Vec::new();

        if let Some(root_index) = self.root {
            self.query_node(root_index, query_aabb, collider_set, &mut results);
        }

        results
    }

    /// 递归查询节点
    fn query_node(
        &self,
        node_index: usize,
        query_aabb: &Aabb,
        collider_set: &ColliderSet,
        results: &mut Vec<ColliderHandle>,
    ) {
        if node_index >= self.nodes.len() {
            return;
        }

        let node = &self.nodes[node_index];

        // 检查节点AABB是否与查询AABB相交
        if !node.aabb.intersects(query_aabb) {
            return;
        }

        // 如果是叶子节点，检查所有碰撞体
        if node.left.is_none() && node.right.is_none() {
            for &handle in &node.colliders {
                if let Some(collider) = collider_set.get(handle) {
                    let collider_aabb = collider.compute_aabb();
                    if query_aabb.intersects(&collider_aabb) {
                        results.push(handle);
                    }
                }
            }
        } else {
            // 递归查询子节点
            if let Some(left_index) = node.left {
                self.query_node(left_index, query_aabb, collider_set, results);
            }
            if let Some(right_index) = node.right {
                self.query_node(right_index, query_aabb, collider_set, results);
            }
        }
    }

    /// 射线查询
    pub fn raycast(
        &self,
        ray: &Ray,
        max_toi: f32,
        collider_set: &ColliderSet,
    ) -> Option<(ColliderHandle, f32)> {
        if let Some(root_index) = self.root {
            self.raycast_node(root_index, ray, max_toi, collider_set)
        } else {
            None
        }
    }

    /// 递归射线查询节点
    fn raycast_node(
        &self,
        node_index: usize,
        ray: &Ray,
        max_toi: f32,
        collider_set: &ColliderSet,
    ) -> Option<(ColliderHandle, f32)> {
        if node_index >= self.nodes.len() {
            return None;
        }

        let node = &self.nodes[node_index];

        // 简化的AABB射线相交测试
        let ray_dir = ray.dir;
        let ray_origin = ray.origin;
        let aabb_center = node.aabb.center();
        let aabb_extents = node.aabb.extents();

        let mut tmin = 0.0f32;
        let mut tmax = max_toi;
        let mut intersects = true;

        for i in 0..3 {
            let axis = match i {
                0 => ray_dir.x,
                1 => ray_dir.y,
                2 => ray_dir.z,
                _ => unreachable!(),
            };
            let origin_component = match i {
                0 => ray_origin.x,
                1 => ray_origin.y,
                2 => ray_origin.z,
                _ => unreachable!(),
            };
            let center_component = match i {
                0 => aabb_center.x,
                1 => aabb_center.y,
                2 => aabb_center.z,
                _ => unreachable!(),
            };
            let extent_component = match i {
                0 => aabb_extents.x,
                1 => aabb_extents.y,
                2 => aabb_extents.z,
                _ => unreachable!(),
            };

            if axis.abs() < 1e-6 {
                if origin_component < center_component - extent_component
                    || origin_component > center_component + extent_component
                {
                    intersects = false;
                    break;
                }
            } else {
                let inv_dir = 1.0 / axis;
                let t1 = (center_component - extent_component - origin_component) * inv_dir;
                let t2 = (center_component + extent_component - origin_component) * inv_dir;
                let (t1, t2) = if t1 > t2 { (t2, t1) } else { (t1, t2) };
                tmin = tmin.max(t1);
                tmax = tmax.min(t2);
                if tmin > tmax {
                    intersects = false;
                    break;
                }
            }
        }

        if !intersects {
            return None;
        }

        // 如果是叶子节点，检查所有碰撞体
        if node.left.is_none() && node.right.is_none() {
            let mut closest: Option<(ColliderHandle, f32)> = None;
            let mut closest_toi = max_toi;

            for &handle in &node.colliders {
                if let Some(collider) = collider_set.get(handle) {
                    // 使用AABB进行简化的射线测试
                    let collider_aabb = collider.compute_aabb();
                    let mut tmin_collider = 0.0f32;
                    let mut tmax_collider = max_toi;
                    let mut collider_intersects = true;

                    for i in 0..3 {
                        let axis = match i {
                            0 => ray_dir.x,
                            1 => ray_dir.y,
                            2 => ray_dir.z,
                            _ => unreachable!(),
                        };
                        let origin_component = match i {
                            0 => ray_origin.x,
                            1 => ray_origin.y,
                            2 => ray_origin.z,
                            _ => unreachable!(),
                        };
                        let min_component = match i {
                            0 => collider_aabb.mins.x,
                            1 => collider_aabb.mins.y,
                            2 => collider_aabb.mins.z,
                            _ => unreachable!(),
                        };
                        let max_component = match i {
                            0 => collider_aabb.maxs.x,
                            1 => collider_aabb.maxs.y,
                            2 => collider_aabb.maxs.z,
                            _ => unreachable!(),
                        };

                        if axis.abs() < 1e-6 {
                            if origin_component < min_component || origin_component > max_component
                            {
                                collider_intersects = false;
                                break;
                            }
                        } else {
                            let inv_dir = 1.0 / axis;
                            let t1 = (min_component - origin_component) * inv_dir;
                            let t2 = (max_component - origin_component) * inv_dir;
                            let (t1, t2) = if t1 > t2 { (t2, t1) } else { (t1, t2) };
                            tmin_collider = tmin_collider.max(t1);
                            tmax_collider = tmax_collider.min(t2);
                            if tmin_collider > tmax_collider {
                                collider_intersects = false;
                                break;
                            }
                        }
                    }

                    if collider_intersects && tmin_collider < closest_toi {
                        closest = Some((handle, tmin_collider));
                        closest_toi = tmin_collider;
                    }
                }
            }

            return closest;
        }

        // 递归查询子节点
        let mut closest: Option<(ColliderHandle, f32)> = None;
        let mut _closest_toi = max_toi;

        if let Some(left_index) = node.left {
            if let Some(result) = self.raycast_node(left_index, ray, max_toi, collider_set) {
                if result.1 < _closest_toi {
                    closest = Some(result);
                    _closest_toi = result.1;
                }
            }
        }

        if let Some(right_index) = node.right {
            if let Some(result) = self.raycast_node(right_index, ray, max_toi, collider_set) {
                if result.1 < _closest_toi {
                    closest = Some(result);
                    _closest_toi = result.1;
                }
            }
        }

        closest
    }

    /// 清除BVH树
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root = None;
    }

    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

/// 空间哈希表
#[derive(Debug)]
pub struct SpatialHash {
    /// 单元格大小
    cell_size: f32,
    /// 哈希表：网格坐标 -> 碰撞体句柄列表
    grid: HashMap<(i32, i32, i32), Vec<ColliderHandle>>,
}

impl SpatialHash {
    /// 创建新的空间哈希表
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
        }
    }

    /// 构建空间哈希表
    pub fn build(&mut self, collider_set: &ColliderSet) {
        self.grid.clear();

        for (handle, collider) in collider_set.iter() {
            let aabb = collider.compute_aabb();

            // 计算碰撞体覆盖的网格范围
            let min_cell = self.world_to_cell(aabb.mins);
            let max_cell = self.world_to_cell(aabb.maxs);

            // 将碰撞体添加到所有覆盖的单元格
            for x in min_cell.0..=max_cell.0 {
                for y in min_cell.1..=max_cell.1 {
                    for z in min_cell.2..=max_cell.2 {
                        self.grid.entry((x, y, z)).or_insert_with(Vec::new).push(handle);
                    }
                }
            }
        }
    }

    /// 世界坐标转换为网格坐标
    fn world_to_cell(&self, point: Point<Real>) -> (i32, i32, i32) {
        (
            (point.x / self.cell_size).floor() as i32,
            (point.y / self.cell_size).floor() as i32,
            (point.z / self.cell_size).floor() as i32,
        )
    }

    /// 查询与AABB相交的碰撞体
    pub fn query_aabb(&self, query_aabb: &Aabb, collider_set: &ColliderSet) -> Vec<ColliderHandle> {
        let mut results = Vec::new();
        let mut visited = std::collections::HashSet::new();

        let min_cell = self.world_to_cell(query_aabb.mins);
        let max_cell = self.world_to_cell(query_aabb.maxs);

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    if let Some(handles) = self.grid.get(&(x, y, z)) {
                        for &handle in handles {
                            if visited.insert(handle) {
                                if let Some(collider) = collider_set.get(handle) {
                                    let collider_aabb = collider.compute_aabb();
                                    if query_aabb.intersects(&collider_aabb) {
                                        results.push(handle);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        results
    }

    /// 清除空间哈希表
    pub fn clear(&mut self) {
        self.grid.clear();
    }
}

/// 八叉树节点
#[derive(Debug, Clone)]
struct OctreeNode {
    /// 包围盒
    aabb: Aabb,
    /// 子节点（8个，按顺序：前上左、前上右、前下左、前下右、后上左、后上右、后下左、后下右）
    children: Option<[usize; 8]>,
    /// 碰撞体句柄列表（叶子节点）
    colliders: Vec<ColliderHandle>,
    /// 节点深度
    depth: usize,
}

/// 八叉树
#[derive(Debug)]
pub struct Octree {
    /// 节点列表
    nodes: Vec<OctreeNode>,
    /// 根节点索引
    root: Option<usize>,
    /// 最大深度
    max_depth: usize,
    /// 每个叶子节点的最大碰撞体数量
    max_colliders_per_leaf: usize,
    /// 根节点包围盒
    root_aabb: Aabb,
}

impl Octree {
    /// 创建新的八叉树
    pub fn new(root_aabb: Aabb, max_depth: usize, max_colliders_per_leaf: usize) -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
            max_depth,
            max_colliders_per_leaf,
            root_aabb,
        }
    }

    /// 构建八叉树
    pub fn build(&mut self, collider_set: &ColliderSet) {
        if collider_set.len() == 0 {
            return;
        }

        // 收集所有碰撞体的AABB和句柄
        let items: Vec<(ColliderHandle, Aabb)> = collider_set
            .iter()
            .map(|(handle, collider)| (handle, collider.compute_aabb()))
            .collect();

        // 构建根节点
        self.root = Some(self.build_node(&items, self.root_aabb, 0));
    }

    /// 递归构建八叉树节点
    fn build_node(&mut self, items: &[(ColliderHandle, Aabb)], aabb: Aabb, depth: usize) -> usize {
        if items.is_empty() {
            return usize::MAX;
        }

        // 过滤出在当前AABB内的碰撞体
        let filtered_items: Vec<_> = items
            .iter()
            .filter(|(_, item_aabb)| aabb.intersects(item_aabb))
            .copied()
            .collect();

        if filtered_items.is_empty() {
            return usize::MAX;
        }

        // 如果达到最大深度或碰撞体数量足够少，创建叶子节点
        if depth >= self.max_depth || filtered_items.len() <= self.max_colliders_per_leaf {
            let colliders: Vec<ColliderHandle> =
                filtered_items.iter().map(|(handle, _)| *handle).collect();

            let node_index = self.nodes.len();
            self.nodes.push(OctreeNode {
                aabb,
                children: None,
                colliders,
                depth,
            });
            return node_index;
        }

        // 分割为8个子节点
        let center = aabb.center();
        let extents = aabb.extents();
        let _half_extents = extents * 0.5;

        let mut children_indices = [usize::MAX; 8];
        let child_aabbs = [
            // 前上左
            Aabb::new(
                Point::new(aabb.mins.x, center.y, aabb.mins.z),
                Point::new(center.x, aabb.maxs.y, center.z),
            ),
            // 前上右
            Aabb::new(
                Point::new(center.x, center.y, aabb.mins.z),
                Point::new(aabb.maxs.x, aabb.maxs.y, center.z),
            ),
            // 前下左
            Aabb::new(
                Point::new(aabb.mins.x, aabb.mins.y, aabb.mins.z),
                Point::new(center.x, center.y, center.z),
            ),
            // 前下右
            Aabb::new(
                Point::new(center.x, aabb.mins.y, aabb.mins.z),
                Point::new(aabb.maxs.x, center.y, center.z),
            ),
            // 后上左
            Aabb::new(
                Point::new(aabb.mins.x, center.y, center.z),
                Point::new(center.x, aabb.maxs.y, aabb.maxs.z),
            ),
            // 后上右
            Aabb::new(
                Point::new(center.x, center.y, center.z),
                Point::new(aabb.maxs.x, aabb.maxs.y, aabb.maxs.z),
            ),
            // 后下左
            Aabb::new(
                Point::new(aabb.mins.x, aabb.mins.y, center.z),
                Point::new(center.x, center.y, aabb.maxs.z),
            ),
            // 后下右
            Aabb::new(
                Point::new(center.x, aabb.mins.y, center.z),
                Point::new(aabb.maxs.x, center.y, aabb.maxs.z),
            ),
        ];

        for (i, child_aabb) in child_aabbs.iter().enumerate() {
            children_indices[i] = self.build_node(&filtered_items, *child_aabb, depth + 1);
        }

        // 创建内部节点
        let node_index = self.nodes.len();
        self.nodes.push(OctreeNode {
            aabb,
            children: Some(children_indices),
            colliders: Vec::new(),
            depth,
        });

        node_index
    }

    /// 查询与AABB相交的碰撞体
    pub fn query_aabb(&self, query_aabb: &Aabb, collider_set: &ColliderSet) -> Vec<ColliderHandle> {
        let mut results = Vec::new();

        if let Some(root_index) = self.root {
            self.query_node(root_index, query_aabb, collider_set, &mut results);
        }

        results
    }

    /// 递归查询节点
    fn query_node(
        &self,
        node_index: usize,
        query_aabb: &Aabb,
        collider_set: &ColliderSet,
        results: &mut Vec<ColliderHandle>,
    ) {
        if node_index >= self.nodes.len() || node_index == usize::MAX {
            return;
        }

        let node = &self.nodes[node_index];

        // 检查节点AABB是否与查询AABB相交
        if !node.aabb.intersects(query_aabb) {
            return;
        }

        // 如果是叶子节点，检查所有碰撞体
        if let None = node.children {
            for &handle in &node.colliders {
                if let Some(collider) = collider_set.get(handle) {
                    let collider_aabb = collider.compute_aabb();
                    if query_aabb.intersects(&collider_aabb) {
                        results.push(handle);
                    }
                }
            }
        } else if let Some(ref children) = node.children {
            // 递归查询子节点
            for &child_index in children.iter() {
                if child_index != usize::MAX {
                    self.query_node(child_index, query_aabb, collider_set, results);
                }
            }
        }
    }

    /// 清除八叉树
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root = None;
    }

    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

/// 空间分区管理器
#[derive(Debug)]
pub struct SpatialPartitionManager {
    /// 分区类型
    partition_type: SpatialPartitionType,
    /// BVH树（如果使用BVH）
    bvh: Option<BVHTree>,
    /// 空间哈希表（如果使用空间哈希）
    spatial_hash: Option<SpatialHash>,
    /// 八叉树（如果使用八叉树）
    octree: Option<Octree>,
    /// 是否需要重建
    needs_rebuild: bool,
    /// 性能统计：查询次数
    query_count: u64,
    /// 性能统计：平均查询时间（微秒）
    average_query_time_us: f64,
}

impl SpatialPartitionManager {
    /// 创建新的空间分区管理器
    pub fn new(partition_type: SpatialPartitionType) -> Self {
        match partition_type {
            SpatialPartitionType::BVH => Self {
                partition_type,
                bvh: Some(BVHTree::new(10, 4)),
                spatial_hash: None,
                octree: None,
                needs_rebuild: true,
                query_count: 0,
                average_query_time_us: 0.0,
            },
            SpatialPartitionType::SpatialHash => Self {
                partition_type,
                bvh: None,
                spatial_hash: Some(SpatialHash::new(2.0)), // 默认单元格大小2.0
                octree: None,
                needs_rebuild: true,
                query_count: 0,
                average_query_time_us: 0.0,
            },
            SpatialPartitionType::Octree => {
                // 创建默认根AABB（可以根据场景调整）
                let root_aabb = Aabb::new(
                    Point::new(-1000.0, -1000.0, -1000.0),
                    Point::new(1000.0, 1000.0, 1000.0),
                );
                Self {
                    partition_type,
                    bvh: None,
                    spatial_hash: None,
                    octree: Some(Octree::new(root_aabb, 10, 4)),
                    needs_rebuild: true,
                    query_count: 0,
                    average_query_time_us: 0.0,
                }
            }
        }
    }

    /// 构建空间分区
    pub fn build(&mut self, collider_set: &ColliderSet) {
        let start = std::time::Instant::now();

        match self.partition_type {
            SpatialPartitionType::BVH => {
                if let Some(ref mut bvh) = self.bvh {
                    bvh.build(collider_set);
                }
            }
            SpatialPartitionType::Octree => {
                if let Some(ref mut octree) = self.octree {
                    octree.build(collider_set);
                }
            }
            SpatialPartitionType::SpatialHash => {
                if let Some(ref mut spatial_hash) = self.spatial_hash {
                    spatial_hash.build(collider_set);
                }
            }
        }

        let elapsed = start.elapsed();
        tracing::debug!(
            target: "physics",
            "Spatial partition built in {:.2}ms (type: {:?})",
            elapsed.as_secs_f64() * 1000.0,
            self.partition_type
        );

        self.needs_rebuild = false;
    }

    /// 查询与AABB相交的碰撞体（带性能监控）
    pub fn query_aabb(&self, query_aabb: &Aabb, collider_set: &ColliderSet) -> Vec<ColliderHandle> {
        let start = std::time::Instant::now();

        let results = match self.partition_type {
            SpatialPartitionType::BVH => {
                if let Some(ref bvh) = self.bvh {
                    bvh.query_aabb(query_aabb, collider_set)
                } else {
                    Vec::new()
                }
            }
            SpatialPartitionType::Octree => {
                if let Some(ref octree) = self.octree {
                    octree.query_aabb(query_aabb, collider_set)
                } else {
                    Vec::new()
                }
            }
            SpatialPartitionType::SpatialHash => {
                if let Some(ref spatial_hash) = self.spatial_hash {
                    spatial_hash.query_aabb(query_aabb, collider_set)
                } else {
                    Vec::new()
                }
            }
        };

        let elapsed = start.elapsed();
        let _elapsed_us = elapsed.as_micros() as f64;

        // 更新性能统计（注意：这里需要&mut self，但为了保持API不变，我们使用内部可变性）
        // 实际实现中可以使用Arc<Mutex<>>或AtomicU64来记录统计

        results
    }

    /// 获取性能统计
    pub fn get_performance_stats(&self) -> SpatialPartitionStats {
        SpatialPartitionStats {
            query_count: self.query_count,
            average_query_time_us: self.average_query_time_us,
            partition_type: self.partition_type,
        }
    }

    /// 更新性能统计（内部使用）
    fn record_query(&mut self, query_time_us: f64) {
        self.query_count += 1;
        let alpha = 0.1; // 指数移动平均
        self.average_query_time_us =
            alpha * query_time_us + (1.0 - alpha) * self.average_query_time_us;
    }

    /// 获取树的深度信息（用于性能分析和调试）
    pub fn get_tree_depth_info(&self) -> (Option<usize>, Option<usize>) {
        let bvh_depth = self
            .bvh
            .as_ref()
            .and_then(|bvh| bvh.root.and_then(|root| bvh.nodes.get(root).map(|node| node.depth)));

        let octree_depth = self.octree.as_ref().and_then(|octree| {
            octree.root.and_then(|root| octree.nodes.get(root).map(|node| node.depth))
        });

        (bvh_depth, octree_depth)
    }

    /// 射线查询
    pub fn raycast(
        &self,
        ray: &Ray,
        max_toi: f32,
        collider_set: &ColliderSet,
    ) -> Option<(ColliderHandle, f32)> {
        match self.partition_type {
            SpatialPartitionType::BVH | SpatialPartitionType::Octree => {
                if let Some(ref bvh) = self.bvh {
                    bvh.raycast(ray, max_toi, collider_set)
                } else {
                    None
                }
            }
            SpatialPartitionType::SpatialHash => {
                // 空间哈希不支持射线查询，需要遍历所有碰撞体
                // 这里可以优化为只查询射线经过的单元格
                None
            }
        }
    }

    /// 标记需要重建
    pub fn mark_needs_rebuild(&mut self) {
        self.needs_rebuild = true;
    }

    /// 检查是否需要重建
    pub fn needs_rebuild(&self) -> bool {
        self.needs_rebuild
    }

    /// 动态调整分区（根据场景大小和对象分布）
    pub fn adjust_for_scene(&mut self, scene_aabb: &Aabb, object_count: usize) {
        match self.partition_type {
            SpatialPartitionType::Octree => {
                if let Some(ref mut octree) = self.octree {
                    // 根据场景大小调整根AABB
                    octree.root_aabb = *scene_aabb;
                    // 根据对象数量调整最大深度
                    let optimal_depth = (object_count as f32).log2().ceil() as usize;
                    octree.max_depth = optimal_depth.min(15).max(5);
                    self.needs_rebuild = true;
                }
            }
            SpatialPartitionType::SpatialHash => {
                if let Some(ref mut spatial_hash) = self.spatial_hash {
                    // 根据场景大小调整单元格大小
                    let scene_size = scene_aabb.extents();
                    let avg_size = (scene_size.x + scene_size.y + scene_size.z) / 3.0;
                    // 单元格大小应该是平均对象大小的2-4倍
                    let optimal_cell_size =
                        (avg_size / (object_count as f32).cbrt()).max(0.5).min(10.0);
                    *spatial_hash = SpatialHash::new(optimal_cell_size);
                    self.needs_rebuild = true;
                }
            }
            _ => {
                // BVH不需要动态调整
            }
        }
    }
}

/// 空间分区性能统计
#[derive(Debug, Clone, Copy)]
pub struct SpatialPartitionStats {
    /// 查询次数
    pub query_count: u64,
    /// 平均查询时间（微秒）
    pub average_query_time_us: f64,
    /// 分区类型
    pub partition_type: SpatialPartitionType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rapier3d::na::Point3 as NaPoint3;

    #[test]
    fn test_bvh_build() {
        let mut collider_set = ColliderSet::new();

        // 创建几个碰撞体
        for i in 0..10 {
            let shape = SharedShape::ball(0.5);
            let collider = ColliderBuilder::new(shape)
                .translation(vector![i as f32 * 2.0, 0.0, 0.0])
                .build();
            collider_set.insert(collider);
        }

        let mut bvh = BVHTree::new(10, 4);
        bvh.build(&collider_set);

        assert!(bvh.node_count() > 0);
    }

    #[test]
    fn test_bvh_query() {
        let mut collider_set = ColliderSet::new();

        // 创建碰撞体
        for i in 0..10 {
            let shape = SharedShape::ball(0.5);
            let collider = ColliderBuilder::new(shape)
                .translation(vector![i as f32 * 2.0, 0.0, 0.0])
                .build();
            collider_set.insert(collider);
        }

        let mut bvh = BVHTree::new(10, 4);
        bvh.build(&collider_set);

        // 查询AABB
        let query_aabb = rapier3d::parry::bounding_volume::Aabb::new(
            NaPoint3::new(0.0, -1.0, -1.0),
            NaPoint3::new(5.0, 1.0, 1.0),
        );
        let results = bvh.query_aabb(&query_aabb, &collider_set);

        assert!(!results.is_empty());
    }

    #[test]
    fn test_spatial_hash_build() {
        let mut collider_set = ColliderSet::new();

        // 创建几个碰撞体
        for i in 0..10 {
            let shape = SharedShape::ball(0.5);
            let collider = ColliderBuilder::new(shape)
                .translation(vector![i as f32 * 2.0, 0.0, 0.0])
                .build();
            collider_set.insert(collider);
        }

        let mut spatial_hash = SpatialHash::new(2.0);
        spatial_hash.build(&collider_set);

        assert!(!spatial_hash.grid.is_empty());
    }

    #[test]
    fn test_spatial_hash_query() {
        let mut collider_set = ColliderSet::new();

        // 创建碰撞体
        for i in 0..10 {
            let shape = SharedShape::ball(0.5);
            let collider = ColliderBuilder::new(shape)
                .translation(vector![i as f32 * 2.0, 0.0, 0.0])
                .build();
            collider_set.insert(collider);
        }

        let mut spatial_hash = SpatialHash::new(2.0);
        spatial_hash.build(&collider_set);

        // 查询AABB
        let query_aabb = rapier3d::parry::bounding_volume::Aabb::new(
            NaPoint3::new(0.0, -1.0, -1.0),
            NaPoint3::new(5.0, 1.0, 1.0),
        );
        let results = spatial_hash.query_aabb(&query_aabb, &collider_set);

        assert!(!results.is_empty());
    }
}
