//! 覆盖图系统（Influence Map）
//!
//! 实现用于AI战术决策的空间影响力分析系统。
//!
//! ## 功能特性
//!
//! - **2D网格覆盖图** - 基于网格的空间影响力表示
//! - **迭代传播算法** - 高效的影响力扩散计算
//! - **战术分析** - 领土控制、危险区域、机会点分析
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::ai::influence_map::{InfluenceGrid, TacticalInfluenceMap};
//!
//! // 创建覆盖图
//! let mut grid = InfluenceGrid::new(100, 100, 1.0);
//!
//! // 添加影响力源
//! grid.add_source(50, 50, 100.0);
//!
//! // 传播影响力
//! grid.propagate(0.5, 5);
//! ```

use std::f32::consts::PI;

/// 2D影响力网格
///
/// 用于表示和计算空间中的影响力分布。
#[derive(Debug, Clone)]
pub struct InfluenceGrid {
    /// 网格宽度
    width: usize,
    /// 网格高度
    height: usize,
    /// 单元格大小（世界坐标单位）
    cell_size: f32,
    /// 影响力值网格
    values: Vec<f32>,
}

impl InfluenceGrid {
    /// 创建新的影响力网格
    ///
    /// # 参数
    /// - `width`: 网格宽度（单元格数）
    /// - `height`: 网格高度（单元格数）
    /// - `cell_size`: 每个单元格的世界坐标大小
    pub fn new(width: usize, height: usize, cell_size: f32) -> Self {
        assert!(width > 0 && height > 0, "Grid dimensions must be positive");
        assert!(cell_size > 0.0, "Cell size must be positive");

        Self {
            width,
            height,
            cell_size,
            values: vec![0.0; width * height],
        }
    }

    /// 获取网格宽度
    pub fn width(&self) -> usize {
        self.width
    }

    /// 获取网格高度
    pub fn height(&self) -> usize {
        self.height
    }

    /// 获取单元格大小
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// 获取指定位置的影响力值
    ///
    /// # 参数
    /// - `x`: X坐标（单元格索引）
    /// - `y`: Y坐标（单元格索引）
    ///
    /// # 返回
    /// 该位置的影响力值，如果坐标越界返回0.0
    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        self.values[y * self.width + x]
    }

    /// 设置指定位置的影响力值
    ///
    /// # 参数
    /// - `x`: X坐标（单元格索引）
    /// - `y`: Y坐标（单元格索引）
    /// - `value`: 新的影响力值
    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.values[y * self.width + x] = value;
    }

    /// 添加影响力源
    ///
    /// # 参数
    /// - `x`: X坐标（单元格索引）
    /// - `y`: Y坐标（单元格索引）
    /// - `strength`: 影响力强度
    pub fn add_source(&mut self, x: usize, y: usize, strength: f32) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.values[y * self.width + x] += strength;
    }

    /// 传播影响力（迭代扩散算法）
    ///
    /// # 参数
    /// - `decay`: 衰减系数（0.0-1.0），每传播一次衰减的比例
    /// - `iterations`: 迭代次数
    ///
    /// # 算法说明
    /// 每次迭代，每个单元格将其影响力传播到相邻的4个单元格（上下左右），
    /// 传播的强度会根据距离衰减。
    pub fn propagate(&mut self, decay: f32, iterations: usize) {
        let decay = decay.clamp(0.0, 1.0);

        for _ in 0..iterations {
            let mut new_values = self.values.clone();

            for y in 0..self.height {
                for x in 0..self.width {
                    let idx = y * self.width + x;
                    let value = self.values[idx];

                    if value.abs() < 0.001 {
                        continue;
                    }

                    // 向四个方向传播
                    let neighbors = [
                        (x.wrapping_sub(1), y),     // 左
                        (x + 1, y),                 // 右
                        (x, y.wrapping_sub(1)),     // 上
                        (x, y + 1),                 // 下
                    ];

                    for (nx, ny) in neighbors {
                        if nx < self.width && ny < self.height {
                            let nidx = ny * self.width + nx;
                            new_values[nidx] += value * decay * 0.25;
                        }
                    }
                }
            }

            self.values = new_values;
        }
    }

    /// 应用高斯模糊平滑影响力分布
    ///
    /// # 参数
    /// - `sigma`: 高斯核的标准差
    /// - `radius`: 高斯核的半径
    pub fn gaussian_smooth(&mut self, sigma: f32, radius: usize) {
        let mut new_values = self.values.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                let mut sum = 0.0;
                let mut weight_sum = 0.0;

                // 应用高斯核
                for dy in -(radius as isize)..=(radius as isize) {
                    for dx in -(radius as isize)..=(radius as isize) {
                        let nx = x as isize + dx;
                        let ny = y as isize + dy;

                        if nx >= 0 && nx < self.width as isize
                            && ny >= 0 && ny < self.height as isize
                        {
                            let distance = ((dx * dx + dy * dy) as f32).sqrt();
                            let weight = (-distance * distance / (2.0 * sigma * sigma)).exp();
                            sum += self.get(nx as usize, ny as usize) * weight;
                            weight_sum += weight;
                        }
                    }
                }

                new_values[y * self.width + x] = if weight_sum > 0.0 {
                    sum / weight_sum
                } else {
                    self.get(x, y)
                };
            }
        }

        self.values = new_values;
    }

    /// 归一化影响力值到指定范围
    ///
    /// # 参数
    /// - `min`: 最小值
    /// - `max`: 最大值
    pub fn normalize(&mut self, min: f32, max: f32) {
        let current_min = self.values.iter().cloned().fold(f32::INFINITY, f32::min);
        let current_max = self.values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let range = current_max - current_min;

        if range < 0.001 {
            // 所有值相同，设为范围中点
            let mid = (min + max) / 2.0;
            for value in &mut self.values {
                *value = mid;
            }
        } else {
            let target_range = max - min;
            for value in &mut self.values {
                *value = min + (*value - current_min) / range * target_range;
            }
        }
    }

    /// 获取所有影响力值
    pub fn values(&self) -> &[f32] {
        &self.values
    }

    /// 获取可变的所有影响力值
    pub fn values_mut(&mut self) -> &mut [f32] {
        &mut self.values
    }

    /// 清空所有影响力值
    pub fn clear(&mut self) {
        for value in &mut self.values {
            *value = 0.0;
        }
    }

    /// 查找最大影响力位置
    ///
    /// # 返回
    /// (x, y, value) - 最大影响力的坐标和值
    pub fn find_max(&self) -> (usize, usize, f32) {
        let mut max_x = 0;
        let mut max_y = 0;
        let mut max_value = f32::NEG_INFINITY;

        for y in 0..self.height {
            for x in 0..self.width {
                let value = self.get(x, y);
                if value > max_value {
                    max_value = value;
                    max_x = x;
                    max_y = y;
                }
            }
        }

        (max_x, max_y, max_value)
    }

    /// 查找最小影响力位置
    ///
    /// # 返回
    /// (x, y, value) - 最小影响力的坐标和值
    pub fn find_min(&self) -> (usize, usize, f32) {
        let mut min_x = 0;
        let mut min_y = 0;
        let mut min_value = f32::INFINITY;

        for y in 0..self.height {
            for x in 0..self.width {
                let value = self.get(x, y);
                if value < min_value {
                    min_value = value;
                    min_x = x;
                    min_y = y;
                }
            }
        }

        (min_x, min_y, min_value)
    }
}

/// 战术覆盖图系统
///
/// 组合多个覆盖图用于战术分析。
#[derive(Debug, Clone)]
pub struct TacticalInfluenceMap {
    /// 领土控制覆盖图
    pub territory: InfluenceGrid,
    /// 危险区域覆盖图
    pub danger: InfluenceGrid,
    /// 机会点覆盖图
    pub opportunity: InfluenceGrid,
}

impl TacticalInfluenceMap {
    /// 创建新的战术覆盖图
    ///
    /// # 参数
    /// - `width`: 网格宽度
    /// - `height`: 网格高度
    /// - `cell_size`: 单元格大小
    pub fn new(width: usize, height: usize, cell_size: f32) -> Self {
        Self {
            territory: InfluenceGrid::new(width, height, cell_size),
            danger: InfluenceGrid::new(width, height, cell_size),
            opportunity: InfluenceGrid::new(width, height, cell_size),
        }
    }

    /// 更新所有覆盖图
    ///
    /// # 参数
    /// - `decay`: 衰减系数
    /// - `iterations`: 传播迭代次数
    pub fn update(&mut self, decay: f32, iterations: usize) {
        self.territory.propagate(decay, iterations);
        self.danger.propagate(decay, iterations);
        self.opportunity.propagate(decay, iterations);
    }

    /// 分析战术位置
    ///
    /// # 参数
    /// - `x`: X坐标
    /// - `y`: Y坐标
    ///
    /// # 返回
    /// 该位置的战术评分（综合考虑领土、危险和机会）
    pub fn analyze_position(&self, x: usize, y: usize) -> f32 {
        let territory = self.territory.get(x, y);
        let danger = self.danger.get(x, y);
        let opportunity = self.opportunity.get(x, y);

        // 战术评分 = 领土控制 + 机会 - 危险
        territory + opportunity - danger.abs()
    }

    /// 查找最佳战术位置
    ///
    /// # 返回
    /// (x, y, score) - 最佳位置的坐标和评分
    pub fn find_best_position(&self) -> (usize, usize, f32) {
        let mut best_x = 0;
        let mut best_y = 0;
        let mut best_score = f32::NEG_INFINITY;

        for y in 0..self.territory.height() {
            for x in 0..self.territory.width() {
                let score = self.analyze_position(x, y);
                if score > best_score {
                    best_score = score;
                    best_x = x;
                    best_y = y;
                }
            }
        }

        (best_x, best_y, best_score)
    }
}

/// 影响力图系统
///
/// 提供高级的覆盖图管理功能。
#[derive(Debug)]
pub struct InfluenceMapSystem {
    /// 地图集合
    maps: Vec<InfluenceGrid>,
}

impl InfluenceMapSystem {
    /// 创建新的影响力图系统
    pub fn new() -> Self {
        Self {
            maps: Vec::new(),
        }
    }

    /// 添加新的影响力图
    pub fn add_map(&mut self, map: InfluenceGrid) {
        self.maps.push(map);
    }

    /// 获取指定索引的影响力图
    pub fn get_map(&self, index: usize) -> Option<&InfluenceGrid> {
        self.maps.get(index)
    }

    /// 获取指定索引的可变影响力图
    pub fn get_map_mut(&mut self, index: usize) -> Option<&mut InfluenceGrid> {
        self.maps.get_mut(index)
    }

    /// 更新所有影响力图
    pub fn update_all(&mut self, decay: f32, iterations: usize) {
        for map in &mut self.maps {
            map.propagate(decay, iterations);
        }
    }

    /// 清空所有影响力图
    pub fn clear_all(&mut self) {
        for map in &mut self.maps {
            map.clear();
        }
    }
}

impl Default for InfluenceMapSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_influence_grid_creation() {
        let grid = InfluenceGrid::new(10, 10, 1.0);
        assert_eq!(grid.width(), 10);
        assert_eq!(grid.height(), 10);
        assert_eq!(grid.cell_size(), 1.0);
    }

    #[test]
    fn test_influence_grid_set_get() {
        let mut grid = InfluenceGrid::new(10, 10, 1.0);
        grid.set(5, 5, 100.0);
        assert_eq!(grid.get(5, 5), 100.0);
    }

    #[test]
    fn test_influence_grid_add_source() {
        let mut grid = InfluenceGrid::new(10, 10, 1.0);
        grid.add_source(5, 5, 50.0);
        grid.add_source(5, 5, 30.0);
        assert_eq!(grid.get(5, 5), 80.0);
    }

    #[test]
    fn test_influence_grid_propagate() {
        let mut grid = InfluenceGrid::new(10, 10, 1.0);
        grid.add_source(5, 5, 100.0);
        grid.propagate(0.5, 1);

        // 中心值应该仍然很高
        assert!(grid.get(5, 5) > 50.0);

        // 相邻单元格应该有传播过来的值
        assert!(grid.get(4, 5) > 0.0);
        assert!(grid.get(6, 5) > 0.0);
        assert!(grid.get(5, 4) > 0.0);
        assert!(grid.get(5, 6) > 0.0);
    }

    #[test]
    fn test_influence_grid_find_max() {
        let mut grid = InfluenceGrid::new(10, 10, 1.0);
        grid.add_source(3, 7, 100.0);
        grid.add_source(5, 5, 50.0);

        let (x, y, value) = grid.find_max();
        assert_eq!(x, 3);
        assert_eq!(y, 7);
        assert_eq!(value, 100.0);
    }

    #[test]
    fn test_influence_grid_clear() {
        let mut grid = InfluenceGrid::new(10, 10, 1.0);
        grid.add_source(5, 5, 100.0);
        grid.clear();

        assert_eq!(grid.get(5, 5), 0.0);
    }

    #[test]
    fn test_tactical_influence_map() {
        let mut tactical = TacticalInfluenceMap::new(20, 20, 1.0);

        tactical.territory.add_source(10, 10, 100.0);
        tactical.danger.add_source(5, 5, -50.0);
        tactical.opportunity.add_source(15, 15, 30.0);

        tactical.update(0.3, 3);

        let score = tactical.analyze_position(10, 10);
        assert!(score > 0.0);
    }

    #[test]
    fn test_influence_map_system() {
        let mut system = InfluenceMapSystem::new();

        let map1 = InfluenceGrid::new(10, 10, 1.0);
        let map2 = InfluenceGrid::new(15, 15, 2.0);

        system.add_map(map1);
        system.add_map(map2);

        assert_eq!(system.get_map(0).unwrap().width(), 10);
        assert_eq!(system.get_map(1).unwrap().width(), 15);
    }
}
