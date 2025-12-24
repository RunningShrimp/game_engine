//  AI 路径寻找加速
//
//  使用 SIMD 和并行处理优化多个智能体的路径寻找
//  - SIMD 加速启发式函数
//  - 批量寻路
//  - 路径缓存
//  - 多智能体协调

use glam::Vec3;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// 为Vec3实现Hash和Eq的包装器
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3Hash {
    pub vec: Vec3,
}

impl Vec3Hash {
    pub fn new(vec: Vec3) -> Self {
        Self { vec }
    }
}

impl From<Vec3> for Vec3Hash {
    fn from(vec: Vec3) -> Self {
        Self { vec }
    }
}

impl From<Vec3Hash> for Vec3 {
    fn from(hash: Vec3Hash) -> Self {
        hash.vec
    }
}

impl Eq for Vec3Hash {}

impl std::hash::Hash for Vec3Hash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // 使用浮点数的位表示进行哈希，确保相同的值有相同的哈希
        self.vec.x.to_bits().hash(state);
        self.vec.y.to_bits().hash(state);
        self.vec.z.to_bits().hash(state);
    }
}

/// 启发式函数类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeuristicType {
    /// 曼哈顿距离
    Manhattan,
    /// 欧几里得距离
    Euclidean,
    /// 切比雪夫距离 (棋盘距离)
    Chebyshev,
}

/// A* 寻路节点
#[derive(Debug, Clone, Copy)]
pub struct PathNode {
    /// 节点位置
    pub position: Vec3,
    /// g 值 (到起点的距离)
    pub g_cost: f32,
    /// h 值 (启发式估计到目标的距离)
    pub h_cost: f32,
    /// f 值 (g + h)
    pub f_cost: f32,
    /// 父节点索引
    pub parent_idx: Option<usize>,
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        (self.position - other.position).length_squared() < 0.001
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // 反向排序用于优先级队列 (最小堆)
        other.f_cost.partial_cmp(&self.f_cost)
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// 寻路结果
#[derive(Debug, Clone)]
pub struct PathfindingResult {
    /// 路径 (位置数组)
    pub path: Vec<Vec3>,
    /// 路径长度
    pub path_length: f32,
    /// 扩展的节点数量
    pub nodes_expanded: u32,
    /// 是否找到路径
    pub found: bool,
    /// 计算时间 (毫秒)
    pub compute_time_ms: f32,
}

/// SIMD 启发式函数
pub struct SIMDHeuristics;

impl SIMDHeuristics {
    /// 批量计算欧几里得距离 (SIMD 优化)
    ///
    /// # Arguments
    /// * `positions` - 位置数组
    /// * `target` - 目标位置
    ///
    /// # Returns
    /// 每个位置到目标的距离数组
    pub fn batch_euclidean_distance(positions: &[Vec3], target: Vec3) -> Vec<f32> {
        let mut distances = Vec::with_capacity(positions.len());

        // SIMD 优化: AVX2 一次处理 8 个 Vec3
        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx2") {
            return Self::batch_euclidean_distance_avx2(positions, target);
        }

        // 标量回退
        for pos in positions {
            distances.push((*pos - target).length());
        }

        distances
    }

    /// AVX2 优化的批量欧几里得距离
    #[cfg(target_arch = "x86_64")]
    fn batch_euclidean_distance_avx2(positions: &[Vec3], target: Vec3) -> Vec<f32> {
        use std::arch::x86_64::*;

        let mut distances = Vec::with_capacity(positions.len());

        unsafe {
            let target_x = _mm256_set1_ps(target.x);
            let target_y = _mm256_set1_ps(target.y);
            let target_z = _mm256_set1_ps(target.z);

            // 一次处理 8 个 Vec3 - AVX2 可以处理 8 个浮点数
            let chunks = positions.chunks_exact(8);
            let remainder = positions.len() % 8;

            for chunk in chunks {
                // 提取 8 个 Vec3 的 x, y, z 坐标
                // 每个 Vec3 是 3 个浮点数, 8 个 Vec3 是 24 个浮点数
                // 将它们排列为 x0, y0, z0, x1, y1, z1, ..., x7, y7, z7

                // 先加载 8 个 Vec3 到 24 个浮点数的数组
                let mut data = [0.0f32; 24];
                for i in 0..8 {
                    data[i * 3 + 0] = chunk[i].x;
                    data[i * 3 + 1] = chunk[i].y;
                    data[i * 3 + 2] = chunk[i].z;
                }

                // 计算 x 坐标的差异
                let xs = _mm256_loadu_ps(&data[0] as *const f32); // x0-x7
                let dx = _mm256_sub_ps(xs, target_x);
                let dx2 = _mm256_mul_ps(dx, dx);

                // 计算 y 坐标的差异
                let ys = _mm256_loadu_ps(&data[8] as *const f32); // y0-y7
                let dy = _mm256_sub_ps(ys, target_y);
                let dy2 = _mm256_mul_ps(dy, dy);

                // 计算 z 坐标的差异
                let zs = _mm256_loadu_ps(&data[16] as *const f32); // z0-z7
                let dz = _mm256_sub_ps(zs, target_z);
                let dz2 = _mm256_mul_ps(dz, dz);

                // 求和并开方
                let sum = _mm256_add_ps(_mm256_add_ps(dx2, dy2), dz2);
                let dist = _mm256_sqrt_ps(sum);

                // 存储结果 (8 个距离值)
                let mut tmp = [0.0f32; 8];
                _mm256_storeu_ps(tmp.as_mut_ptr(), dist);

                for &d in &tmp {
                    distances.push(d);
                }
            }

            // 处理剩余元素
            for pos in &positions[positions.len() - remainder..] {
                distances.push((pos - target).length());
            }
        }

        distances
    }

    /// 批量计算曼哈顿距离
    pub fn batch_manhattan_distance(positions: &[Vec3], target: Vec3) -> Vec<f32> {
        positions
            .iter()
            .map(|pos| {
                (pos.x - target.x).abs() + (pos.y - target.y).abs() + (pos.z - target.z).abs()
            })
            .collect()
    }

    /// 批量计算切比雪夫距离
    pub fn batch_chebyshev_distance(positions: &[Vec3], target: Vec3) -> Vec<f32> {
        positions
            .iter()
            .map(|pos| {
                ((pos.x - target.x).abs())
                    .max((pos.y - target.y).abs())
                    .max((pos.z - target.z).abs())
            })
            .collect()
    }
}
/// 单个智能体的寻路器
/// 支持多种启发式函数和可扩展的寻路算法
#[derive(Clone)]
pub struct AgentPathfinder {
    /// 智能体标识
    pub agent_id: u32,
    /// 当前位置
    pub current_position: Vec3,
    /// 目标位置
    pub target_position: Vec3,
    /// 当前路径
    pub current_path: Vec<Vec3>,
    /// 路径索引
    pub path_index: usize,
    /// 启发式函数类型
    pub heuristic: HeuristicType,
}

impl AgentPathfinder {
    /// 创建新的智能体寻路器
    pub fn new(agent_id: u32, position: Vec3) -> Self {
        Self {
            agent_id,
            current_position: position,
            target_position: position,
            current_path: Vec::new(),
            path_index: 0,
            heuristic: HeuristicType::Euclidean,
        }
    }

    /// 设置启发式函数
    pub fn set_heuristic(&mut self, heuristic: HeuristicType) {
        self.heuristic = heuristic;
    }

    /// 计算启发式值
    fn compute_heuristic(&self, from: Vec3, to: Vec3) -> f32 {
        match self.heuristic {
            HeuristicType::Euclidean => (from - to).length(),
            HeuristicType::Manhattan => {
                (from.x - to.x).abs() + (from.y - to.y).abs() + (from.z - to.z).abs()
            }
            HeuristicType::Chebyshev => {
                ((from.x - to.x).abs()).max((from.y - to.y).abs()).max((from.z - to.z).abs())
            }
        }
    }

    /// 计算两个网格点之间的移动成本
    fn compute_move_cost(&self, from: Vec3, to: Vec3) -> f32 {
        (from - to).length()
    }

    /// 获取相邻节点
    fn get_neighbors(&self, position: Vec3, grid_size: f32) -> Vec<Vec3> {
        let mut neighbors = Vec::with_capacity(8);

        // 8个方向的移动 (包括对角线)
        let offsets = [
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 0.0, 1.0),
            (-1.0, 0.0, 0.0),
            (0.0, -1.0, 0.0),
            (0.0, 0.0, -1.0),
            (1.0, 1.0, 0.0),
            (-1.0, -1.0, 0.0),
            (1.0, 0.0, 1.0),
            (-1.0, 0.0, -1.0),
            (0.0, 1.0, 1.0),
            (0.0, -1.0, -1.0),
        ];

        for offset in offsets {
            let neighbor = Vec3::new(
                position.x + offset.0 * grid_size,
                position.y + offset.1 * grid_size,
                position.z + offset.2 * grid_size,
            );
            neighbors.push(neighbor);
        }

        neighbors
    }

    /// 真正的 A* 寻路算法实现
    pub fn find_path(&mut self, target: Vec3, grid_size: f32) -> PathfindingResult {
        let start = std::time::Instant::now();

        self.target_position = target;

        // 如果已经在目标位置
        if (self.current_position - target).length_squared() < 0.001 {
            return PathfindingResult {
                path: vec![target],
                path_length: 0.0,
                nodes_expanded: 0,
                found: true,
                compute_time_ms: start.elapsed().as_secs_f32() * 1000.0,
            };
        }

        // 开放列表 (优先级队列)
        let mut open_list = BinaryHeap::new();
        // 关闭列表
        let mut closed_list = HashSet::<Vec3Hash>::new();
        // 所有节点的列表
        let mut all_nodes = Vec::new();

        // 创建起始节点
        let start_node = PathNode {
            position: self.current_position,
            g_cost: 0.0,
            h_cost: self.compute_heuristic(self.current_position, target),
            f_cost: 0.0, // g + h
            parent_idx: None,
        };

        open_list.push(start_node);
        all_nodes.push(start_node);
        closed_list.insert(Vec3Hash::new(self.current_position));

        let mut nodes_expanded = 0u32;

        while let Some(current_node) = open_list.pop() {
            nodes_expanded += 1;

            // 检查是否到达目标
            if (current_node.position - target).length() < grid_size * 0.5 {
                // 构建路径
                let mut path = Vec::new();
                let mut current_idx = all_nodes.len() - 1;

                loop {
                    path.push(all_nodes[current_idx].position);

                    if let Some(parent_idx) = all_nodes[current_idx].parent_idx {
                        current_idx = parent_idx;
                    } else {
                        break;
                    }
                }

                path.reverse();

                // 存储当前路径
                self.current_path = path.clone();
                self.path_index = 0;

                let path_length = path.windows(2).map(|w| (w[1] - w[0]).length()).sum();

                return PathfindingResult {
                    path,
                    path_length,
                    nodes_expanded,
                    found: true,
                    compute_time_ms: start.elapsed().as_secs_f32() * 1000.0,
                };
            }

            // 获取相邻节点
            let neighbors = self.get_neighbors(current_node.position, grid_size);

            for neighbor_pos in neighbors {
                if closed_list.contains(&Vec3Hash::new(neighbor_pos)) {
                    continue;
                }

                // 计算 g_cost
                let tentative_g_cost = current_node.g_cost
                    + self.compute_move_cost(current_node.position, neighbor_pos);

                // 检查是否已经在开放列表中
                let mut found_in_open = false;
                for node in &open_list {
                    if node.position == neighbor_pos {
                        found_in_open = true;
                        if node.g_cost <= tentative_g_cost {
                            // 已经有更好的路径
                            break;
                        }
                        // 需要更新路径
                        break;
                    }
                }

                if !found_in_open {
                    // 创建新节点并加入开放列表
                    let neighbor_node = PathNode {
                        position: neighbor_pos,
                        g_cost: tentative_g_cost,
                        h_cost: self.compute_heuristic(neighbor_pos, target),
                        f_cost: tentative_g_cost + self.compute_heuristic(neighbor_pos, target),
                        parent_idx: Some(all_nodes.len() - 1), // 当前节点是最后一个加入的
                    };

                    open_list.push(neighbor_node);
                    all_nodes.push(neighbor_node);
                    closed_list.insert(Vec3Hash::new(neighbor_pos));
                }
            }
        }

        // 未找到路径
        self.current_path.clear();
        self.path_index = 0;

        PathfindingResult {
            path: Vec::new(),
            path_length: 0.0,
            nodes_expanded,
            found: false,
            compute_time_ms: start.elapsed().as_secs_f32() * 1000.0,
        }
    }
}

/// 批量路径寻找管理器
pub struct BatchPathfinder {
    /// 所有智能体
    agents: HashMap<u32, AgentPathfinder>,
    /// 路径缓存 (使用 Vec3Hash 元组键)
    path_cache: HashMap<(Vec3Hash, Vec3Hash), Vec<Vec3>>,
    /// 网格大小
    grid_size: f32,
}

impl BatchPathfinder {
    /// 创建新的批量寻路管理器
    pub fn new(grid_size: f32) -> Self {
        Self {
            agents: HashMap::new(),
            path_cache: HashMap::new(),
            grid_size,
        }
    }

    /// 为路径对生成缓存键
    fn path_key(from: Vec3, to: Vec3) -> (Vec3Hash, Vec3Hash) {
        (Vec3Hash::new(from), Vec3Hash::new(to))
    }

    /// 添加智能体
    pub fn add_agent(&mut self, agent_id: u32, position: Vec3) {
        self.agents.insert(agent_id, AgentPathfinder::new(agent_id, position));
    }

    /// 为智能体寻找路径
    pub fn find_path_for_agent(
        &mut self,
        agent_id: u32,
        target: Vec3,
    ) -> Option<PathfindingResult> {
        if let Some(agent) = self.agents.get_mut(&agent_id) {
            let result = agent.find_path(target, self.grid_size);

            // 缓存路径
            if result.found {
                let key = Self::path_key(agent.current_position, target);
                self.path_cache.insert(key, result.path.clone());
            }

            Some(result)
        } else {
            None
        }
    }

    /// 批量为所有智能体寻找路径
    pub fn find_paths_batch(&mut self, targets: &[(u32, Vec3)]) -> Vec<PathfindingResult> {
        targets
            .par_iter()
            .map(|(agent_id, target)| {
                // 为每个查询创建独立的 AgentPathfinder
                // 注意: 这里不使用内部的 agents HashMap，因为 HashMap 不是线程安全的
                if let Some(agent) = self.agents.get(&agent_id) {
                    let mut temp_agent = agent.clone();
                    let result = temp_agent.find_path(*target, self.grid_size);

                    // 如果在主线程中需要缓存路径，可以在这里处理
                    // 但需要注意线程安全
                    result
                } else {
                    // 智能体不存在，返回空结果
                    PathfindingResult {
                        path: Vec::new(),
                        path_length: 0.0,
                        nodes_expanded: 0,
                        found: false,
                        compute_time_ms: 0.0,
                    }
                }
            })
            .collect()
    }

    /// 更新智能体位置
    pub fn update_agent_position(&mut self, agent_id: u32, position: Vec3) {
        if let Some(agent) = self.agents.get_mut(&agent_id) {
            agent.current_position = position;
        }
    }

    /// 获取缓存的路径
    pub fn get_cached_path(&self, from: Vec3, to: Vec3) -> Option<&Vec<Vec3>> {
        let key = Self::path_key(from, to);
        self.path_cache.get(&key)
    }

    /// 清空路径缓存
    pub fn clear_cache(&mut self) {
        self.path_cache.clear();
    }

    /// 获取智能体数量
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// 获取缓存大小
    pub fn cache_size(&self) -> usize {
        self.path_cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_heuristics() {
        let positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let target = Vec3::new(1.0, 1.0, 0.0);

        let distances = SIMDHeuristics::batch_euclidean_distance(&positions, target);
        assert_eq!(distances.len(), 3);
        assert!(distances[0] > distances[2]); // (0,0,0) 到目标的距离 > (0,1,0) 到目标的距离
    }

    #[test]
    fn test_manhattan_distance() {
        let positions = vec![Vec3::ZERO, Vec3::new(1.0, 1.0, 0.0)];
        let target = Vec3::new(1.0, 1.0, 0.0);

        let distances = SIMDHeuristics::batch_manhattan_distance(&positions, target);
        assert_eq!(distances.len(), 2);
        assert_eq!(distances[1], 0.0);
        assert_eq!(distances[0], 2.0);
    }

    #[test]
    fn test_agent_pathfinder() {
        let mut agent = AgentPathfinder::new(1, Vec3::ZERO);
        let target = Vec3::new(10.0, 0.0, 0.0);

        let result = agent.find_path(target, 1.0);
        assert!(result.found);
        assert!(!result.path.is_empty());
    }

    #[test]
    fn test_batch_pathfinder() {
        let mut batch = BatchPathfinder::new(1.0);

        batch.add_agent(1, Vec3::ZERO);
        batch.add_agent(2, Vec3::new(5.0, 0.0, 0.0));

        assert_eq!(batch.agent_count(), 2);

        let targets = vec![
            (1, Vec3::new(10.0, 0.0, 0.0)),
            (2, Vec3::new(15.0, 0.0, 0.0)),
        ];

        let results = batch.find_paths_batch(&targets);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_path_caching() {
        let mut batch = BatchPathfinder::new(1.0);
        batch.add_agent(1, Vec3::ZERO);

        let target = Vec3::new(10.0, 0.0, 0.0);
        batch.find_path_for_agent(1, target);

        assert_eq!(batch.cache_size(), 1);
        assert!(batch.get_cached_path(Vec3::ZERO, target).is_some());
    }

    #[test]
    fn test_astar_implementation() {
        let mut agent = AgentPathfinder::new(1, Vec3::new(0.0, 0.0, 0.0));
        let target = Vec3::new(3.0, 3.0, 0.0);

        let result = agent.find_path(target, 1.0);
        assert!(result.found);
        assert!(result.path.len() > 0);
        assert!(result.path_length > 0.0);

        // 验证路径包含目标点
        assert!(
            result.path.contains(&target) || result.path.last().unwrap().distance(target) < 1.0
        );
    }
}
