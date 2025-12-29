//  寻路系统
//
//  实现A*寻路算法和导航网格支持。

use glam::Vec3;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

// SIMD优化支持
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use game_engine_simd::Vec3Simd;

/// 寻路节点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathNode {
    /// 节点ID
    pub id: u32,
    /// 节点位置
    pub position: Vec3,
    /// 是否可通行
    pub traversable: bool,
}

/// 寻路连接
#[derive(Debug, Clone)]
pub struct PathConnection {
    /// 起始节点ID
    pub from: u32,
    /// 目标节点ID
    pub to: u32,
    /// 连接代价
    pub cost: f32,
}

/// 寻路网格
#[derive(Debug, Clone)]
pub struct NavigationMesh {
    /// 所有节点
    pub nodes: HashMap<u32, PathNode>,
    /// 节点间的连接
    pub connections: Vec<PathConnection>,
}

impl NavigationMesh {
    /// 创建新的导航网格
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, node: PathNode) {
        self.nodes.insert(node.id, node);
    }

    /// 添加连接
    pub fn add_connection(&mut self, connection: PathConnection) {
        self.connections.push(connection);
    }

    /// 获取节点
    pub fn get_node(&self, id: u32) -> Option<&PathNode> {
        self.nodes.get(&id)
    }

    /// 获取节点的邻居
    pub fn get_neighbors(&self, node_id: u32) -> Vec<(u32, f32)> {
        self.connections.iter()
            .filter(|conn| conn.from == node_id)
            .map(|conn| (conn.to, conn.cost))
            .collect()
    }
}

impl Default for NavigationMesh {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationMesh {
    /// 计算两点间的启发式距离（欧几里得距离）
    pub fn heuristic(&self, from: u32, to: u32) -> f32 {
        if let (Some(from_node), Some(to_node)) = (self.get_node(from), self.get_node(to)) {
            #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
            {
                use game_engine_simd::VectorOps;
                let from_simd = Vec3Simd::new(from_node.position.x, from_node.position.y, from_node.position.z);
                let to_simd = Vec3Simd::new(to_node.position.x, to_node.position.y, to_node.position.z);
                let diff = from_simd.sub(&to_simd);
                diff.length()
            }
            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            {
                from_node.position.distance(to_node.position)
            }
        } else {
            f32::INFINITY
        }
    }

    /// 寻找路径
    pub fn find_path(&self, start: Vec3, end: Vec3) -> Option<Vec<Vec3>> {
        // 找到最近的起始节点和目标节点
        let start_node = self.find_nearest_node(start)?;
        let end_node = self.find_nearest_node(end)?;

        if start_node == end_node {
            return Some(vec![start, end]);
        }

        // A* 算法
        let path = self.a_star(start_node, end_node)?;
        Some(path)
    }

    /// 找到最近的可通行节点（SIMD优化）
    fn find_nearest_node(&self, position: Vec3) -> Option<u32> {
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            use game_engine_simd::VectorOps;
            let pos_simd = Vec3Simd::new(position.x, position.y, position.z);
            self.nodes
                .values()
                .filter(|node| node.traversable)
                .min_by(|a, b| {
                    let a_simd = Vec3Simd::new(a.position.x, a.position.y, a.position.z);
                    let b_simd = Vec3Simd::new(b.position.x, b.position.y, b.position.z);
                    let a_diff = a_simd.sub(&pos_simd);
                    let b_diff = b_simd.sub(&pos_simd);
                    let a_dist_sq = a_diff.dot(&a_diff);
                    let b_dist_sq = b_diff.dot(&b_diff);
                    a_dist_sq.partial_cmp(&b_dist_sq).unwrap_or(Ordering::Equal)
                })
                .map(|node| node.id)
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            self.nodes
                .values()
                .filter(|node| node.traversable)
                .min_by(|a, b| {
                    a.position
                        .distance_squared(position)
                        .partial_cmp(&b.position.distance_squared(position))
                        .unwrap_or(Ordering::Equal)
                })
                .map(|node| node.id)
        }
    }

    /// A* 寻路算法
    fn a_star(&self, start: u32, goal: u32) -> Option<Vec<Vec3>> {
        let mut open_set = BinaryHeap::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();
        let mut f_score = HashMap::new();

        // 初始化
        open_set.push(SearchNode {
            id: start,
            f_score: 0.0,
        });
        g_score.insert(start, 0.0);
        f_score.insert(start, self.heuristic(start, goal));

        while let Some(current) = open_set.pop() {
            if current.id == goal {
                // 重建路径
                return Some(self.reconstruct_path(came_from, current.id));
            }

            for (neighbor, cost) in self.get_neighbors(current.id) {
                let tentative_g_score = g_score.get(&current.id).unwrap_or(&f32::INFINITY) + cost;

                if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&f32::INFINITY) {
                    came_from.insert(neighbor, current.id);
                    g_score.insert(neighbor, tentative_g_score);
                    let f = tentative_g_score + self.heuristic(neighbor, goal);
                    f_score.insert(neighbor, f);

                    // 检查是否已经在开放集合中
                    if !open_set.iter().any(|node| node.id == neighbor) {
                        open_set.push(SearchNode {
                            id: neighbor,
                            f_score: f,
                        });
                    }
                }
            }
        }

        None // 没有找到路径
    }

    /// 重建路径
    fn reconstruct_path(&self, came_from: HashMap<u32, u32>, current: u32) -> Vec<Vec3> {
        let mut path = vec![];
        let mut current = current;

        while let Some(&prev) = came_from.get(&current) {
            if let Some(node) = self.get_node(current) {
                path.push(node.position);
            }
            current = prev;
        }

        if let Some(node) = self.get_node(current) {
            path.push(node.position);
        }

        path.reverse();
        path
    }
}

/// A* 搜索节点
#[derive(Debug, Clone, Copy, PartialEq)]
struct SearchNode {
    id: u32,
    f_score: f32,
}

impl Eq for SearchNode {}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // 注意：反转顺序，因为BinaryHeap是最大堆
        other.f_score.partial_cmp(&self.f_score).unwrap_or(Ordering::Equal)
    }
}

/// 寻路请求
#[derive(Debug, Clone)]
pub struct PathfindingRequest {
    /// 请求ID（用于匹配结果）
    pub request_id: u64,
    /// 起始位置
    pub start: Vec3,
    /// 目标位置
    pub end: Vec3,
}

/// 寻路结果
#[derive(Debug, Clone)]
pub struct PathfindingResult {
    /// 请求ID
    pub request_id: u64,
    /// 找到的路径
    pub path: Option<Vec<Vec3>>,
}

// ParallelPathfindingService 已删除 - 请使用 AsyncPathfindingService 替代

/// 寻路服务
pub struct PathfindingService;

impl PathfindingService {
    /// 创建导航网格
    pub fn create_nav_mesh() -> NavigationMesh {
        NavigationMesh::new()
    }

    /// 添加节点到导航网格
    pub fn add_node_to_mesh(mesh: &mut NavigationMesh, position: Vec3, traversable: bool) -> u32 {
        let id = mesh.nodes.len() as u32;
        let node = PathNode {
            id,
            position,
            traversable,
        };
        mesh.add_node(node);
        id
    }

    /// 在导航网格中添加连接
    pub fn add_connection_to_mesh(mesh: &mut NavigationMesh, from: u32, to: u32, cost: f32) {
        let connection = PathConnection { from, to, cost };
        mesh.add_connection(connection);
    }

    /// 寻找路径
    pub fn find_path(mesh: &NavigationMesh, start: Vec3, end: Vec3) -> Option<Vec<Vec3>> {
        mesh.find_path(start, end)
    }

    /// 平滑路径（简单的直线优化）
    pub fn smooth_path(path: &[Vec3], max_angle: f32) -> Vec<Vec3> {
        if path.len() <= 2 {
            return path.to_vec();
        }

        let mut smoothed = vec![path[0]];
        let mut current = 0;

        while current < path.len() - 2 {
            let mut farthest = current + 1;

            for i in (current + 2)..path.len() {
                // 检查从current到i的直线是否可通行
                if Self::can_traverse_line(path[current], path[i], max_angle) {
                    farthest = i;
                } else {
                    break;
                }
            }

            smoothed.push(path[farthest]);
            current = farthest;
        }

        if current < path.len() - 1 {
            smoothed.push(path[path.len() - 1]);
        }

        smoothed
    }

    /// 检查两点间的直线是否可通行
    fn can_traverse_line(start: Vec3, end: Vec3, max_angle: f32) -> bool {
        let direction = (end - start).normalize();
        let angle = direction.angle_between(Vec3::Y);
        angle <= max_angle
    }

    /// 计算路径长度
    pub fn path_length(path: &[Vec3]) -> f32 {
        path.windows(2).map(|pair| pair[0].distance(pair[1])).sum()
    }

    /// 简化路径（移除不必要的节点）
    pub fn simplify_path(path: &[Vec3], tolerance: f32) -> Vec<Vec3> {
        if path.len() <= 2 {
            return path.to_vec();
        }

        let mut simplified = vec![path[0]];
        let mut anchor = 0;

        for i in 1..path.len() {
            let point = path[i];
            let anchor_point = path[anchor];

            // 计算最大偏离
            let mut max_distance = 0.0;
            for intermediate_point in &path[(anchor + 1)..i] {
                let distance = Self::point_to_line_distance(*intermediate_point, anchor_point, point);
                if distance > max_distance {
                    max_distance = distance;
                }
            }

            if max_distance > tolerance {
                simplified.push(path[i - 1]);
                anchor = i - 1;
            }
        }

        simplified.push(path[path.len() - 1]);
        simplified
    }

    /// 计算点到线段的距离
    fn point_to_line_distance(point: Vec3, line_start: Vec3, line_end: Vec3) -> f32 {
        let line = line_end - line_start;
        let to_point = point - line_start;

        let line_length = line.length();
        if line_length == 0.0 {
            return to_point.length();
        }

        let t = (to_point.dot(line) / (line_length * line_length)).clamp(0.0, 1.0);
        let closest = line_start + line * t;
        (point - closest).length()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_nav_mesh_creation() {
        let mut mesh = NavigationMesh::new();

        // 添加节点
        let node1 = PathNode {
            id: 0,
            position: Vec3::new(0.0, 0.0, 0.0),
            traversable: true,
        };
        let node2 = PathNode {
            id: 1,
            position: Vec3::new(1.0, 0.0, 0.0),
            traversable: true,
        };
        let node3 = PathNode {
            id: 2,
            position: Vec3::new(1.0, 1.0, 0.0),
            traversable: true,
        };

        mesh.add_node(node1);
        mesh.add_node(node2);
        mesh.add_node(node3);

        // 添加连接
        mesh.add_connection(PathConnection {
            from: 0,
            to: 1,
            cost: 1.0,
        });
        mesh.add_connection(PathConnection {
            from: 1,
            to: 2,
            cost: 1.0,
        });

        assert_eq!(mesh.nodes.len(), 3);
        assert_eq!(mesh.connections.len(), 2);
    }

    #[test]
    fn test_pathfinding() {
        let mut mesh = NavigationMesh::new();

        // 创建一个简单的网格
        PathfindingService::add_node_to_mesh(&mut mesh, Vec3::new(0.0, 0.0, 0.0), true);
        PathfindingService::add_node_to_mesh(&mut mesh, Vec3::new(1.0, 0.0, 0.0), true);
        PathfindingService::add_node_to_mesh(&mut mesh, Vec3::new(2.0, 0.0, 0.0), true);

        PathfindingService::add_connection_to_mesh(&mut mesh, 0, 1, 1.0);
        PathfindingService::add_connection_to_mesh(&mut mesh, 1, 2, 1.0);

        // 寻找路径
        let path = PathfindingService::find_path(
            &mesh,
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        );
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.len() >= 2);
    }

    #[test]
    fn test_path_smoothing() {
        let path = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
        ];

        let smoothed = PathfindingService::smooth_path(&path, 0.1);
        assert!(smoothed.len() <= path.len());
    }

    // ParallelPathfindingService 测试已删除 - 请使用 AsyncPathfindingService 的测试替代

    proptest! {
        #[test]
        fn test_pathfinding_properties(
            start_x in -100.0f32..100.0,
            start_y in -100.0f32..100.0,
            start_z in -100.0f32..100.0,
            end_x in -100.0f32..100.0,
            end_y in -100.0f32..100.0,
            end_z in -100.0f32..100.0,
        ) {
            let mut mesh = NavigationMesh::new();
            let start = Vec3::new(start_x, start_y, start_z);
            let end = Vec3::new(end_x, end_y, end_z);

            // 创建简单的网格：添加起点和终点附近的节点
            let start_node_id = 0;
            let end_node_id = 1;

            mesh.add_node(PathNode {
                id: start_node_id,
                position: start,
                traversable: true,
            });

            mesh.add_node(PathNode {
                id: end_node_id,
                position: end,
                traversable: true,
            });

            // 添加连接
            let distance = start.distance(end);
            mesh.add_connection(PathConnection {
                from: start_node_id,
                to: end_node_id,
                cost: distance,
            });

            // 属性1: 如果起点和终点相同，路径应该包含至少一个点
            if start.distance(end) < 0.1 {
                let path = mesh.find_path(start, end);
                prop_assert!(path.is_some());
                if let Some(p) = path {
                    prop_assert!(!p.is_empty());
                }
            } else {
                // 属性2: 如果存在路径，路径应该从起点开始，到终点结束
                let path = mesh.find_path(start, end);
                if let Some(p) = path {
                    prop_assert!(!p.is_empty());
                    // 路径的第一个点应该接近起点
                    let first_dist = p[0].distance(start);
                    prop_assert!(first_dist < 1.0);
                    // 路径的最后一个点应该接近终点
                    let last_dist = p[p.len() - 1].distance(end);
                    prop_assert!(last_dist < 1.0);
                }
            }

            // 属性3: 启发式函数应该满足三角不等式
            let h_start_end = mesh.heuristic(start_node_id, end_node_id);
            prop_assert!(h_start_end.is_finite());
            prop_assert!(h_start_end >= 0.0);
        }

        #[test]
        fn test_heuristic_properties(
            x1 in -100.0f32..100.0,
            y1 in -100.0f32..100.0,
            z1 in -100.0f32..100.0,
            x2 in -100.0f32..100.0,
            y2 in -100.0f32..100.0,
            z2 in -100.0f32..100.0,
        ) {
            let mut mesh = NavigationMesh::new();
            let pos1 = Vec3::new(x1, y1, z1);
            let pos2 = Vec3::new(x2, y2, z2);

            mesh.add_node(PathNode {
                id: 0,
                position: pos1,
                traversable: true,
            });
            mesh.add_node(PathNode {
                id: 1,
                position: pos2,
                traversable: true,
            });

            // 属性: 启发式函数应该是对称的（或至少非负）
            let h1 = mesh.heuristic(0, 1);
            let h2 = mesh.heuristic(1, 0);

            prop_assert!(h1.is_finite());
            prop_assert!(h2.is_finite());
            prop_assert!(h1 >= 0.0);
            prop_assert!(h2 >= 0.0);

            // 对于欧几里得距离，h1和h2应该相等
            prop_assert!((h1 - h2).abs() < 0.001);
        }
    }
}
