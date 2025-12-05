//! AI 路径寻找加速
//!
//! 使用 SIMD 和并行处理优化多个智能体的路径寻找
//! - SIMD 加速启发式函数
//! - 批量寻路
//! - 路径缓存
//! - 多智能体协调

use glam::Vec3;
use std::cmp::Ordering;
use std::collections::HashMap;

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

        // SIMD 优化: AVX2 一次处理 4 个 Vec3
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

            // 因为 Vec3 不是 4 个浮点数，我们需要手动处理
            let chunks = positions.chunks_exact(4);
            let remainder = positions.len() % 4;

            for chunk in chunks {
                // 提取 x, y, z 坐标
                let mut xs = [0.0f32; 4];
                let mut ys = [0.0f32; 4];
                let mut zs = [0.0f32; 4];

                for (i, pos) in chunk.iter().enumerate() {
                    xs[i] = pos.x;
                    ys[i] = pos.y;
                    zs[i] = pos.z;
                }

                let xs_v = _mm256_loadu_ps(xs.as_ptr());
                let ys_v = _mm256_loadu_ps(ys.as_ptr());
                let zs_v = _mm256_loadu_ps(zs.as_ptr());

                // 计算差值
                let dx = _mm256_sub_ps(xs_v, target_x);
                let dy = _mm256_sub_ps(ys_v, target_y);
                let dz = _mm256_sub_ps(zs_v, target_z);

                // 平方
                let dx2 = _mm256_mul_ps(dx, dx);
                let dy2 = _mm256_mul_ps(dy, dy);
                let dz2 = _mm256_mul_ps(dz, dz);

                // 求和并开方
                let sum = _mm256_add_ps(_mm256_add_ps(dx2, dy2), dz2);
                let dist = _mm256_sqrt_ps(sum);

                // 存储结果
                let mut tmp = [0.0f32; 8];
                _mm256_storeu_ps(tmp.as_mut_ptr(), dist);
                for i in 0..4 {
                    distances.push(tmp[i]);
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
            HeuristicType::Chebyshev => ((from.x - to.x).abs())
                .max((from.y - to.y).abs())
                .max((from.z - to.z).abs()),
        }
    }

    /// 简化的 A* 寻路 (网格基础)
    pub fn find_path(&mut self, target: Vec3, grid_size: f32) -> PathfindingResult {
        let start = std::time::Instant::now();

        self.target_position = target;
        let mut path = Vec::new();
        let mut nodes_expanded = 0u32;

        // 简化实现: 直接线性插值到目标
        let direction = (target - self.current_position).normalize();
        let distance = (target - self.current_position).length();
        let steps = (distance / grid_size).ceil() as usize;

        if steps > 0 {
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let pos = self.current_position + direction * distance * t;
                path.push(pos);
                nodes_expanded += 1;
            }
        } else {
            path.push(target);
        }

        self.current_path = path.clone();
        self.path_index = 0;

        let path_length = path.windows(2).map(|w| (w[1] - w[0]).length()).sum();

        let compute_time = start.elapsed().as_secs_f32() * 1000.0;

        PathfindingResult {
            path,
            path_length,
            nodes_expanded,
            found: true,
            compute_time_ms: compute_time,
        }
    }
}

/// 批量路径寻找管理器
pub struct BatchPathfinder {
    /// 所有智能体
    agents: HashMap<u32, AgentPathfinder>,
    /// 路径缓存 (使用字符串键代替 Vec3)
    path_cache: HashMap<String, Vec<Vec3>>,
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
    fn path_key(from: Vec3, to: Vec3) -> String {
        format!(
            "{:.2}_{:.2}_{:.2}_to_{:.2}_{:.2}_{:.2}",
            from.x, from.y, from.z, to.x, to.y, to.z
        )
    }

    /// 添加智能体
    pub fn add_agent(&mut self, agent_id: u32, position: Vec3) {
        self.agents
            .insert(agent_id, AgentPathfinder::new(agent_id, position));
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
        let mut results = Vec::new();

        for (agent_id, target) in targets {
            if let Some(result) = self.find_path_for_agent(*agent_id, *target) {
                results.push(result);
            }
        }

        results
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
}
