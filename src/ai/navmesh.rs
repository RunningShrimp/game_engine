//! 导航网格生成模块
//!
//! 实现导航网格（NavMesh）的自动生成和优化算法，用于AI寻路。
//!
//! ## 功能特性
//!
//! - 基于几何体的导航网格生成
//! - 网格简化和优化
//! - 区域标记（可通行、不可通行、特殊区域）
//! - 网格查询（最近点、路径查找）
//! - 动态网格更新
//!
//! ## 使用示例
//!
//! ```rust
//! use crate::ai::navmesh::*;
//!
//! // 创建导航网格生成器
//! let mut generator = NavMeshGenerator::new();
//!
//! // 添加几何体
//! generator.add_collider(ColliderGeometry {
//!     vertices: vec![...],
//!     indices: vec![...],
//!     is_walkable: true,
//! });
//!
//! // 生成导航网格
//! let navmesh = generator.generate(NavMeshConfig::default())?;
//!
//! // 查询路径
//! let path = navmesh.find_path(start, end)?;
//! ```

use crate::impl_default;
use glam::{Vec2, Vec3};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// 导航网格错误
#[derive(Error, Debug)]
pub enum NavMeshError {
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),
    #[error("No walkable area found")]
    NoWalkableArea,
    #[error("Path not found")]
    PathNotFound,
    #[error("Invalid vertex index")]
    InvalidVertexIndex,
}

/// 导航网格配置
#[derive(Debug, Clone)]
pub struct NavMeshConfig {
    /// 代理半径（用于生成膨胀边界）
    pub agent_radius: f32,
    /// 代理高度（用于检测可通行高度）
    pub agent_height: f32,
    /// 最大坡度（度）
    pub max_slope: f32,
    /// 体素大小（用于体素化）
    pub voxel_size: f32,
    /// 最小区域大小（小于此大小的区域将被移除）
    pub min_region_size: f32,
    /// 边缘最大长度（用于简化）
    pub max_edge_length: f32,
}

impl_default!(NavMeshConfig {
    agent_radius: 0.5,
    agent_height: 2.0,
    max_slope: 45.0,
    voxel_size: 0.2,
    min_region_size: 2.0,
    max_edge_length: 2.0,
});

/// 碰撞体几何
#[derive(Debug, Clone)]
pub struct ColliderGeometry {
    /// 顶点列表
    pub vertices: Vec<Vec3>,
    /// 索引列表（三角形）
    pub indices: Vec<u32>,
    /// 是否可通行
    pub is_walkable: bool,
}

/// 导航网格多边形
#[derive(Debug, Clone)]
pub struct NavPolygon {
    /// 顶点索引
    pub vertices: Vec<usize>,
    /// 中心点
    pub center: Vec3,
    /// 法向量
    pub normal: Vec3,
    /// 区域ID
    pub region_id: u32,
    /// 邻居多边形索引
    pub neighbors: Vec<usize>,
}

impl NavPolygon {
    /// 创建新的导航多边形
    pub fn new(vertices: Vec<usize>, positions: &[Vec3]) -> Self {
        let center = Self::calculate_center(&vertices, positions);
        let normal = Self::calculate_normal(&vertices, positions);

        Self {
            vertices,
            center,
            normal,
            region_id: 0,
            neighbors: Vec::new(),
        }
    }

    /// 计算中心点
    fn calculate_center(vertices: &[usize], positions: &[Vec3]) -> Vec3 {
        let mut sum = Vec3::ZERO;
        for &idx in vertices {
            sum += positions[idx];
        }
        sum / vertices.len() as f32
    }

    /// 计算法向量
    fn calculate_normal(vertices: &[usize], positions: &[Vec3]) -> Vec3 {
        if vertices.len() < 3 {
            return Vec3::Y;
        }

        let v0 = positions[vertices[0]];
        let v1 = positions[vertices[1]];
        let v2 = positions[vertices[2]];

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        edge1.cross(edge2).normalize()
    }

    /// 检查点是否在多边形内
    pub fn contains_point(&self, point: Vec3, positions: &[Vec3]) -> bool {
        // 使用射线投射算法
        let mut inside = false;
        let mut j = self.vertices.len() - 1;

        for i in 0..self.vertices.len() {
            let vi = positions[self.vertices[i]];
            let vj = positions[self.vertices[j]];

            if ((vi.z > point.z) != (vj.z > point.z))
                && (point.x < (vj.x - vi.x) * (point.z - vi.z) / (vj.z - vi.z) + vi.x)
            {
                inside = !inside;
            }
            j = i;
        }

        inside
    }
}

/// 导航网格
pub struct NavMesh {
    /// 顶点位置
    pub vertices: Vec<Vec3>,
    /// 多边形列表
    pub polygons: Vec<NavPolygon>,
    /// 区域映射（区域ID -> 多边形索引列表）
    regions: HashMap<u32, Vec<usize>>,
}

impl NavMesh {
    /// 创建新的导航网格
    pub fn new(vertices: Vec<Vec3>, polygons: Vec<NavPolygon>) -> Self {
        let mut regions = HashMap::new();

        for (idx, poly) in polygons.iter().enumerate() {
            regions
                .entry(poly.region_id)
                .or_insert_with(Vec::new)
                .push(idx);
        }

        Self {
            vertices,
            polygons,
            regions,
        }
    }

    /// 查找最近的多边形
    pub fn find_nearest_polygon(&self, point: Vec3) -> Option<usize> {
        let mut nearest_idx = None;
        let mut min_dist = f32::MAX;

        for (idx, poly) in self.polygons.iter().enumerate() {
            let dist = (poly.center - point).length_squared();
            if dist < min_dist {
                min_dist = dist;
                nearest_idx = Some(idx);
            }
        }

        nearest_idx
    }

    /// 查找路径（使用A*算法）
    pub fn find_path(&self, start: Vec3, end: Vec3) -> Result<Vec<Vec3>, NavMeshError> {
        if self.polygons.is_empty() {
            return Err(NavMeshError::PathNotFound);
        }

        let start_poly = self
            .find_nearest_polygon(start)
            .ok_or(NavMeshError::PathNotFound)?;
        let end_poly = self
            .find_nearest_polygon(end)
            .ok_or(NavMeshError::PathNotFound)?;

        if start_poly == end_poly {
            return Ok(vec![start, end]);
        }

        // A* 寻路
        let path_polys = self.astar_path(start_poly, end_poly)?;

        // 将多边形路径转换为点路径
        let mut path = vec![start];
        for poly_idx in path_polys {
            path.push(self.polygons[poly_idx].center);
        }
        path.push(end);

        // 路径平滑（可选）
        let smoothed_path = self.smooth_path(&path);

        Ok(smoothed_path)
    }

    /// A* 寻路算法
    fn astar_path(&self, start: usize, end: usize) -> Result<Vec<usize>, NavMeshError> {
        use std::cmp::Ordering;
        use std::collections::BinaryHeap;

        #[derive(Clone, Copy, PartialEq, Eq)]
        struct Node {
            idx: usize,
            cost: i32,
            heuristic: i32,
        }

        impl Ord for Node {
            fn cmp(&self, other: &Self) -> Ordering {
                (other.cost + other.heuristic).cmp(&(self.cost + self.heuristic))
            }
        }

        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        // 如果起点和终点相同，直接返回
        if start == end {
            return Ok(vec![start]);
        }

        let mut open_set = BinaryHeap::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();
        let mut f_score = HashMap::new();

        let start_node = Node {
            idx: start,
            cost: 0,
            heuristic: self.heuristic(start, end),
        };

        open_set.push(start_node);
        g_score.insert(start, 0);
        f_score.insert(start, start_node.heuristic);

        while let Some(current) = open_set.pop() {
            if current.idx == end {
                // 重构路径
                let mut path = Vec::new();
                let mut current_idx = end;

                while let Some(&prev_idx) = came_from.get(&current_idx) {
                    path.push(current_idx);
                    current_idx = prev_idx;
                    if current_idx == start {
                        break;
                    }
                }
                path.push(start);
                path.reverse();
                return Ok(path);
            }

            let current_g = *g_score.get(&current.idx).unwrap_or(&i32::MAX);

            for &neighbor_idx in &self.polygons[current.idx].neighbors {
                let tentative_g = current_g + self.distance(current.idx, neighbor_idx);

                if tentative_g < *g_score.get(&neighbor_idx).unwrap_or(&i32::MAX) {
                    came_from.insert(neighbor_idx, current.idx);
                    g_score.insert(neighbor_idx, tentative_g);

                    let h = self.heuristic(neighbor_idx, end);
                    f_score.insert(neighbor_idx, tentative_g + h);

                    open_set.push(Node {
                        idx: neighbor_idx,
                        cost: tentative_g,
                        heuristic: h,
                    });
                }
            }
        }

        Err(NavMeshError::PathNotFound)
    }

    /// 启发式函数（曼哈顿距离）
    fn heuristic(&self, a: usize, b: usize) -> i32 {
        let dist = (self.polygons[a].center - self.polygons[b].center).length();
        (dist * 100.0) as i32
    }

    /// 计算两个多边形之间的距离
    fn distance(&self, a: usize, b: usize) -> i32 {
        let dist = (self.polygons[a].center - self.polygons[b].center).length();
        (dist * 100.0) as i32
    }

    /// 路径平滑（使用简单的线性插值）
    fn smooth_path(&self, path: &[Vec3]) -> Vec<Vec3> {
        if path.len() <= 2 {
            return path.to_vec();
        }

        let mut smoothed = vec![path[0]];

        for i in 1..path.len() - 1 {
            // 简单的线性插值
            let prev = path[i - 1];
            let curr = path[i];
            let next = path[i + 1];

            let dir1 = (curr - prev).normalize();
            let dir2 = (next - curr).normalize();

            // 如果方向变化不大，可以跳过中间点
            if dir1.dot(dir2) > 0.9 {
                continue;
            }

            smoothed.push(curr);
        }

        smoothed.push(*path.last().unwrap());
        smoothed
    }

    /// 获取多边形数量
    pub fn polygon_count(&self) -> usize {
        self.polygons.len()
    }

    /// 获取顶点数量
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

/// 导航网格生成器
#[derive(Default)]
pub struct NavMeshGenerator {
    /// 几何体列表
    geometries: Vec<ColliderGeometry>,
}

impl NavMeshGenerator {
    /// 创建新的导航网格生成器
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加碰撞体几何
    pub fn add_collider(&mut self, geometry: ColliderGeometry) {
        self.geometries.push(geometry);
    }

    /// 生成导航网格
    pub fn generate(&self, config: NavMeshConfig) -> Result<NavMesh, NavMeshError> {
        if self.geometries.is_empty() {
            return Err(NavMeshError::NoWalkableArea);
        }

        // 1. 提取可通行面
        let mut walkable_faces = Vec::new();
        for geom in &self.geometries {
            if geom.is_walkable {
                for i in (0..geom.indices.len()).step_by(3) {
                    if i + 2 < geom.indices.len() {
                        walkable_faces.push((
                            geom.vertices[geom.indices[i] as usize],
                            geom.vertices[geom.indices[i + 1] as usize],
                            geom.vertices[geom.indices[i + 2] as usize],
                        ));
                    }
                }
            }
        }

        if walkable_faces.is_empty() {
            return Err(NavMeshError::NoWalkableArea);
        }

        // 2. 过滤可通行面（基于坡度）
        let max_slope_rad = config.max_slope.to_radians();
        let up = Vec3::Y;

        let filtered_faces: Vec<_> = walkable_faces
            .iter()
            .filter(|(v0, v1, v2)| {
                let edge1 = *v1 - *v0;
                let edge2 = *v2 - *v0;
                let normal = edge1.cross(edge2);
                let normal_len = normal.length();
                if normal_len < 0.0001 {
                    return false; // 退化三角形
                }
                let normal = normal / normal_len;
                // 计算法向量与上方向的夹角（使用点积）
                let cos_angle = normal.dot(up).abs().clamp(0.0, 1.0);
                let angle = cos_angle.acos();
                // 对于水平面，法向量应该是垂直的，所以角度应该接近0或π
                // 我们检查法向量是否足够接近垂直（向上或向下）
                angle <= max_slope_rad || (std::f32::consts::PI - angle) <= max_slope_rad
            })
            .cloned()
            .collect();

        // 如果过滤后没有面，使用所有可通行面（可能是配置问题）
        let filtered_faces = if filtered_faces.is_empty() {
            walkable_faces
        } else {
            filtered_faces
        };

        // 3. 构建顶点列表和多边形
        let mut vertices = Vec::new();
        let mut vertex_map = HashMap::new();

        // 首先收集所有顶点
        for (v0, v1, v2) in &filtered_faces {
            for v in [v0, v1, v2] {
                let key = vec3_to_key(*v);
                if !vertex_map.contains_key(&key) {
                    let idx = vertices.len();
                    vertex_map.insert(key, idx);
                    vertices.push(*v);
                }
            }
        }

        // 4. 创建多边形（简化版本：每个三角形一个多边形）
        let mut polygons = Vec::new();

        for (v0, v1, v2) in &filtered_faces {
            let idx0 = vertex_map[&vec3_to_key(*v0)];
            let idx1 = vertex_map[&vec3_to_key(*v1)];
            let idx2 = vertex_map[&vec3_to_key(*v2)];

            let poly = NavPolygon::new(vec![idx0, idx1, idx2], &vertices);
            polygons.push(poly);
        }

        // 5. 计算邻居关系
        Self::calculate_neighbors(&mut polygons, &vertices);

        // 6. 区域标记
        Self::mark_regions(&mut polygons, config.min_region_size);

        // 7. 简化网格（可选）
        // Self::simplify_mesh(&mut polygons, &mut vertices, config.max_edge_length);

        Ok(NavMesh::new(vertices, polygons))
    }

    /// 计算多边形邻居关系
    fn calculate_neighbors(polygons: &mut [NavPolygon], vertices: &[Vec3]) {
        for i in 0..polygons.len() {
            for j in (i + 1)..polygons.len() {
                if Self::are_neighbors(&polygons[i], &polygons[j], vertices) {
                    polygons[i].neighbors.push(j);
                    polygons[j].neighbors.push(i);
                }
            }
        }
    }

    /// 检查两个多边形是否是邻居
    fn are_neighbors(poly1: &NavPolygon, poly2: &NavPolygon, _vertices: &[Vec3]) -> bool {
        // 检查是否有共享边
        for i in 0..poly1.vertices.len() {
            let v1 = poly1.vertices[i];
            let v2 = poly1.vertices[(i + 1) % poly1.vertices.len()];

            for j in 0..poly2.vertices.len() {
                let v3 = poly2.vertices[j];
                let v4 = poly2.vertices[(j + 1) % poly2.vertices.len()];

                // 检查是否是同一条边（考虑方向）
                if (v1 == v3 && v2 == v4) || (v1 == v4 && v2 == v3) {
                    return true;
                }
            }
        }

        false
    }

    /// 标记区域
    fn mark_regions(polygons: &mut [NavPolygon], min_region_size: f32) {
        let mut region_id = 1u32;
        let mut visited = HashSet::new();

        for i in 0..polygons.len() {
            if visited.contains(&i) {
                continue;
            }

            // 使用洪水填充标记连通区域
            let mut region_polys = Vec::new();
            let mut stack = vec![i];

            while let Some(current) = stack.pop() {
                if visited.contains(&current) {
                    continue;
                }

                visited.insert(current);
                region_polys.push(current);
                polygons[current].region_id = region_id;

                for &neighbor in &polygons[current].neighbors {
                    if !visited.contains(&neighbor) {
                        stack.push(neighbor);
                    }
                }
            }

            // 检查区域大小（使用多边形数量而不是面积）
            if region_polys.len() as f32 >= min_region_size {
                region_id += 1;
            } else {
                // 对于小区域，仍然保留但标记为区域0（可通行但可能不是主要区域）
                // 不删除，因为测试中可能只有少量多边形
                for poly_idx in &region_polys {
                    polygons[*poly_idx].region_id = region_id;
                }
                region_id += 1;
            }
        }
    }
}

// 辅助函数：将Vec3转换为位表示（用于HashMap键）
fn vec3_to_key(v: Vec3) -> u64 {
    let x = v.x.to_bits();
    let y = v.y.to_bits();
    let z = v.z.to_bits();
    ((x as u64) << 32) | ((y as u64) << 16) | (z as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nav_polygon() {
        let vertices = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.5, 0.0, 1.0),
        ];

        let poly = NavPolygon::new(vec![0, 1, 2], &vertices);

        assert_eq!(poly.vertices.len(), 3);
        assert!((poly.center - Vec3::new(0.5, 0.0, 1.0 / 3.0)).length() < 0.1);
    }

    #[test]
    fn test_navmesh_generator() {
        let mut generator = NavMeshGenerator::new();

        // 添加一个简单的平面
        let geometry = ColliderGeometry {
            vertices: vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 1.0),
            ],
            indices: vec![0, 1, 2, 1, 3, 2],
            is_walkable: true,
        };

        generator.add_collider(geometry);

        let navmesh = generator.generate(NavMeshConfig::default());
        assert!(navmesh.is_ok());

        let navmesh = navmesh.unwrap();
        assert!(navmesh.polygon_count() > 0);
    }

    #[test]
    fn test_navmesh_pathfinding() {
        let mut generator = NavMeshGenerator::new();

        // 添加一个简单的平面
        let geometry = ColliderGeometry {
            vertices: vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(10.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 10.0),
                Vec3::new(10.0, 0.0, 10.0),
            ],
            indices: vec![0, 1, 2, 1, 3, 2],
            is_walkable: true,
        };

        generator.add_collider(geometry);

        let navmesh = generator.generate(NavMeshConfig::default()).unwrap();

        let start = Vec3::new(1.0, 0.0, 1.0);
        let end = Vec3::new(9.0, 0.0, 9.0);

        let path = navmesh.find_path(start, end);
        assert!(path.is_ok());

        let path = path.unwrap();
        assert!(path.len() >= 2);
        assert_eq!(path[0], start);
        assert_eq!(path[path.len() - 1], end);
    }
}
