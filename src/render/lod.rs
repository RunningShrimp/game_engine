//! LOD (Level of Detail) 系统
//!
//! 提供完整的多级细节管理，包括：
//! - 距离自动选择
//! - 平滑过渡 (Crossfade/Dithering)
//! - 屏幕覆盖率选择
//! - 性能预算控制
//!
//! # 示例
//!
//! ```ignore
//! // 创建 LOD 配置
//! let config = LodConfig::builder()
//!     .add_level(0.0, 20.0, LodQuality::High)
//!     .add_level(20.0, 50.0, LodQuality::Medium)
//!     .add_level(50.0, 100.0, LodQuality::Low)
//!     .with_transition(LodTransition::Crossfade { duration: 0.3 })
//!     .build();
//!
//! // 使用 LOD 选择器
//! let selector = LodSelector::new(config);
//! let lod_level = selector.select(distance, screen_size);
//! ```

use glam::{Mat4, Vec3};
use std::collections::HashMap;

/// LOD 质量等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LodQuality {
    /// 最高质量 (原始模型)
    #[default]
    High,
    /// 中等质量 (50% 面数)
    Medium,
    /// 低质量 (25% 面数)
    Low,
    /// 极低质量 (10% 面数/公告板)
    VeryLow,
    /// 剔除 (不渲染)
    Culled,
}

impl LodQuality {
    /// 获取建议的面数比例
    pub fn face_ratio(&self) -> f32 {
        match self {
            LodQuality::High => 1.0,
            LodQuality::Medium => 0.5,
            LodQuality::Low => 0.25,
            LodQuality::VeryLow => 0.1,
            LodQuality::Culled => 0.0,
        }
    }
    
    /// 转换为索引
    pub fn as_index(&self) -> usize {
        match self {
            LodQuality::High => 0,
            LodQuality::Medium => 1,
            LodQuality::Low => 2,
            LodQuality::VeryLow => 3,
            LodQuality::Culled => 4,
        }
    }
}

/// LOD 过渡方式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LodTransition {
    /// 立即切换 (无过渡)
    Instant,
    /// 交叉淡入淡出
    Crossfade {
        /// 过渡持续时间 (秒)
        duration: f32,
    },
    /// 抖动过渡 (基于屏幕空间噪声)
    Dithering {
        /// 过渡区域大小 (相对于距离)
        blend_range: f32,
    },
    /// 滞后切换 (防止频繁切换)
    Hysteresis {
        /// 滞后范围
        range: f32,
    },
}

impl Default for LodTransition {
    fn default() -> Self {
        LodTransition::Hysteresis { range: 0.1 }
    }
}

/// LOD 级别配置
#[derive(Debug, Clone)]
pub struct LodLevel {
    /// 最小距离
    pub min_distance: f32,
    /// 最大距离
    pub max_distance: f32,
    /// 质量等级
    pub quality: LodQuality,
    /// 网格资源标识 (可选)
    pub mesh_id: Option<String>,
    /// 该级别的顶点数量 (用于性能预算)
    pub vertex_count: u32,
    /// 该级别的三角形数量
    pub triangle_count: u32,
}

impl LodLevel {
    /// 创建新的 LOD 级别
    pub fn new(min_distance: f32, max_distance: f32, quality: LodQuality) -> Self {
        Self {
            min_distance,
            max_distance,
            quality,
            mesh_id: None,
            vertex_count: 0,
            triangle_count: 0,
        }
    }
    
    /// 设置网格信息
    pub fn with_mesh(mut self, mesh_id: &str, vertices: u32, triangles: u32) -> Self {
        self.mesh_id = Some(mesh_id.to_string());
        self.vertex_count = vertices;
        self.triangle_count = triangles;
        self
    }
    
    /// 检查距离是否在此级别范围内
    pub fn contains_distance(&self, distance: f32) -> bool {
        distance >= self.min_distance && distance < self.max_distance
    }
}

/// LOD 配置
#[derive(Debug, Clone)]
pub struct LodConfig {
    /// LOD 级别列表 (按距离排序)
    pub levels: Vec<LodLevel>,
    /// 过渡方式
    pub transition: LodTransition,
    /// 距离偏移 (用于全局调整)
    pub distance_bias: f32,
    /// 屏幕尺寸因子 (启用屏幕覆盖率选择)
    pub use_screen_coverage: bool,
    /// 屏幕覆盖率阈值 (像素比)
    pub screen_coverage_thresholds: Vec<f32>,
    /// 是否强制使用最低 LOD
    pub force_low_quality: bool,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            levels: vec![
                LodLevel::new(0.0, 20.0, LodQuality::High),
                LodLevel::new(20.0, 50.0, LodQuality::Medium),
                LodLevel::new(50.0, 100.0, LodQuality::Low),
                LodLevel::new(100.0, f32::MAX, LodQuality::VeryLow),
            ],
            transition: LodTransition::default(),
            distance_bias: 0.0,
            use_screen_coverage: false,
            screen_coverage_thresholds: vec![0.1, 0.05, 0.01],
            force_low_quality: false,
        }
    }
}

impl LodConfig {
    /// 创建构建器
    pub fn builder() -> LodConfigBuilder {
        LodConfigBuilder::new()
    }
    
    /// 根据距离获取 LOD 级别
    pub fn get_level_for_distance(&self, distance: f32) -> Option<&LodLevel> {
        let adjusted_distance = distance + self.distance_bias;
        
        if self.force_low_quality {
            return self.levels.last();
        }
        
        self.levels.iter().find(|level| level.contains_distance(adjusted_distance))
    }
    
    /// 获取指定质量的级别
    pub fn get_level_by_quality(&self, quality: LodQuality) -> Option<&LodLevel> {
        self.levels.iter().find(|level| level.quality == quality)
    }
}

/// LOD 配置构建器
pub struct LodConfigBuilder {
    config: LodConfig,
}

impl LodConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: LodConfig {
                levels: Vec::new(),
                ..Default::default()
            },
        }
    }
    
    /// 添加 LOD 级别
    pub fn add_level(mut self, min_dist: f32, max_dist: f32, quality: LodQuality) -> Self {
        self.config.levels.push(LodLevel::new(min_dist, max_dist, quality));
        self
    }
    
    /// 添加带网格信息的 LOD 级别
    pub fn add_level_with_mesh(
        mut self,
        min_dist: f32,
        max_dist: f32,
        quality: LodQuality,
        mesh_id: &str,
        vertices: u32,
        triangles: u32,
    ) -> Self {
        self.config.levels.push(
            LodLevel::new(min_dist, max_dist, quality)
                .with_mesh(mesh_id, vertices, triangles)
        );
        self
    }
    
    /// 设置过渡方式
    pub fn with_transition(mut self, transition: LodTransition) -> Self {
        self.config.transition = transition;
        self
    }
    
    /// 设置距离偏移
    pub fn with_distance_bias(mut self, bias: f32) -> Self {
        self.config.distance_bias = bias;
        self
    }
    
    /// 启用屏幕覆盖率选择
    pub fn with_screen_coverage(mut self, thresholds: Vec<f32>) -> Self {
        self.config.use_screen_coverage = true;
        self.config.screen_coverage_thresholds = thresholds;
        self
    }
    
    /// 构建配置
    pub fn build(mut self) -> LodConfig {
        // 按距离排序
        self.config.levels.sort_by(|a, b| 
            a.min_distance.partial_cmp(&b.min_distance).unwrap()
        );
        self.config
    }
}

impl Default for LodConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// LOD 选择结果
#[derive(Debug, Clone)]
pub struct LodSelection {
    /// 当前 LOD 级别索引
    pub current_level: usize,
    /// 当前质量
    pub quality: LodQuality,
    /// 过渡因子 (0.0 = 完全当前级别, 1.0 = 完全下一级别)
    pub transition_factor: f32,
    /// 是否正在过渡
    pub is_transitioning: bool,
    /// 下一级别 (如果正在过渡)
    pub next_level: Option<usize>,
}

/// LOD 选择器
pub struct LodSelector {
    config: LodConfig,
    /// 每个实体的当前状态 (用于滞后/过渡)
    entity_states: HashMap<u64, LodEntityState>,
}

/// 实体 LOD 状态
struct LodEntityState {
    current_level: usize,
    last_distance: f32,
    transition_progress: f32,
    target_level: Option<usize>,
}

impl LodSelector {
    /// 创建 LOD 选择器
    pub fn new(config: LodConfig) -> Self {
        Self {
            config,
            entity_states: HashMap::new(),
        }
    }
    
    /// 选择 LOD 级别 (无状态)
    pub fn select_stateless(&self, distance: f32, bounding_radius: f32, view_proj: &Mat4) -> LodSelection {
        let effective_distance = if self.config.use_screen_coverage {
            self.distance_from_screen_coverage(distance, bounding_radius, view_proj)
        } else {
            distance + self.config.distance_bias
        };
        
        // 查找匹配的级别
        let level_index = self.find_level_index(effective_distance);
        let quality = self.config.levels.get(level_index)
            .map(|l| l.quality)
            .unwrap_or(LodQuality::Culled);
        
        LodSelection {
            current_level: level_index,
            quality,
            transition_factor: 0.0,
            is_transitioning: false,
            next_level: None,
        }
    }
    
    /// 选择 LOD 级别 (带状态追踪)
    pub fn select(&mut self, entity_id: u64, distance: f32, delta_time: f32) -> LodSelection {
        let effective_distance = distance + self.config.distance_bias;
        let target_level = self.find_level_index(effective_distance);
        
        // 获取或创建实体状态
        let state = self.entity_states.entry(entity_id).or_insert(LodEntityState {
            current_level: target_level,
            last_distance: distance,
            transition_progress: 0.0,
            target_level: None,
        });
        
        // 应用过渡逻辑
        let selection = match self.config.transition {
            LodTransition::Instant => {
                state.current_level = target_level;
                LodSelection {
                    current_level: target_level,
                    quality: self.config.levels.get(target_level)
                        .map(|l| l.quality)
                        .unwrap_or(LodQuality::Culled),
                    transition_factor: 0.0,
                    is_transitioning: false,
                    next_level: None,
                }
            }
            
            LodTransition::Hysteresis { range } => {
                Self::apply_hysteresis_cfg(&self.config, state, target_level, effective_distance, range)
            }
            
            LodTransition::Crossfade { duration } => {
                Self::apply_crossfade_cfg(&self.config, state, target_level, delta_time, duration)
            }
            
            LodTransition::Dithering { blend_range } => {
                Self::apply_dithering_cfg(&self.config, state, target_level, effective_distance, blend_range)
            }
        };
        
        state.last_distance = distance;
        selection
    }
    
    /// 清除实体状态
    pub fn remove_entity(&mut self, entity_id: u64) {
        self.entity_states.remove(&entity_id);
    }
    
    /// 清除所有状态
    pub fn clear(&mut self) {
        self.entity_states.clear();
    }
    
    // 内部辅助方法
    
    fn find_level_index(&self, distance: f32) -> usize {
        for (i, level) in self.config.levels.iter().enumerate() {
            if level.contains_distance(distance) {
                return i;
            }
        }
        self.config.levels.len().saturating_sub(1)
    }
    
    fn distance_from_screen_coverage(
        &self,
        distance: f32,
        bounding_radius: f32,
        view_proj: &Mat4,
    ) -> f32 {
        // 计算屏幕覆盖率
        let screen_coverage = self.calculate_screen_coverage(distance, bounding_radius, view_proj);
        
        // 根据屏幕覆盖率阈值选择虚拟距离
        for (i, &threshold) in self.config.screen_coverage_thresholds.iter().enumerate() {
            if screen_coverage > threshold {
                // 返回对应级别的中间距离
                if let Some(level) = self.config.levels.get(i) {
                    return (level.min_distance + level.max_distance) / 2.0;
                }
            }
        }
        
        // 默认使用最远距离
        distance
    }
    
    fn calculate_screen_coverage(
        &self,
        distance: f32,
        bounding_radius: f32,
        view_proj: &Mat4,
    ) -> f32 {
        // 简化的屏幕覆盖率计算
        // 实际应使用投影后的包围球大小
        let projected_size = bounding_radius / distance.max(0.001);
        
        // 获取投影矩阵的缩放因子
        let scale_x = view_proj.col(0).length();
        let scale_y = view_proj.col(1).length();
        let avg_scale = (scale_x + scale_y) / 2.0;
        
        projected_size * avg_scale
    }
    
    fn apply_hysteresis_cfg(
        cfg: &LodConfig,
        state: &mut LodEntityState,
        target_level: usize,
        distance: f32,
        range: f32,
    ) -> LodSelection {
        let current = state.current_level;
        
        // 只有当距离变化足够大时才切换
        if target_level != current {
            if let Some(current_level_config) = cfg.levels.get(current) {
                let hysteresis_distance = if target_level > current {
                    // 切换到更低质量：需要超过阈值
                    current_level_config.max_distance * (1.0 + range)
                } else {
                    // 切换到更高质量：需要低于阈值
                    current_level_config.min_distance * (1.0 - range)
                };
                
                let should_switch = if target_level > current {
                    distance > hysteresis_distance
                } else {
                    distance < hysteresis_distance
                };
                
                if should_switch {
                    state.current_level = target_level;
                }
            }
        }
        
        LodSelection {
            current_level: state.current_level,
            quality: cfg.levels.get(state.current_level)
                .map(|l| l.quality)
                .unwrap_or(LodQuality::Culled),
            transition_factor: 0.0,
            is_transitioning: false,
            next_level: None,
        }
    }
    
    fn apply_crossfade_cfg(
        cfg: &LodConfig,
        state: &mut LodEntityState,
        target_level: usize,
        delta_time: f32,
        duration: f32,
    ) -> LodSelection {
        // 检测是否需要开始新的过渡
        if target_level != state.current_level && state.target_level.is_none() {
            state.target_level = Some(target_level);
            state.transition_progress = 0.0;
        }
        
        // 更新过渡进度
        if let Some(target) = state.target_level {
            state.transition_progress += delta_time / duration;
            
            if state.transition_progress >= 1.0 {
                // 过渡完成
                state.current_level = target;
                state.target_level = None;
                state.transition_progress = 0.0;
            }
        }
        
        LodSelection {
            current_level: state.current_level,
            quality: cfg.levels.get(state.current_level)
                .map(|l| l.quality)
                .unwrap_or(LodQuality::Culled),
            transition_factor: state.transition_progress,
            is_transitioning: state.target_level.is_some(),
            next_level: state.target_level,
        }
    }
    
    fn apply_dithering_cfg(
        cfg: &LodConfig,
        state: &mut LodEntityState,
        target_level: usize,
        distance: f32,
        blend_range: f32,
    ) -> LodSelection {
        state.current_level = target_level;
        
        // 计算与级别边界的距离来确定混合因子
        let transition_factor = if let Some(level) = cfg.levels.get(target_level) {
            let level_range = level.max_distance - level.min_distance;
            let transition_zone = level_range * blend_range;
            
            // 在级别边界附近计算混合因子
            if distance > level.max_distance - transition_zone {
                (distance - (level.max_distance - transition_zone)) / transition_zone
            } else if distance < level.min_distance + transition_zone {
                (level.min_distance + transition_zone - distance) / transition_zone
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        LodSelection {
            current_level: target_level,
            quality: cfg.levels.get(target_level)
                .map(|l| l.quality)
                .unwrap_or(LodQuality::Culled),
            transition_factor: transition_factor.clamp(0.0, 1.0),
            is_transitioning: transition_factor > 0.0,
            next_level: if transition_factor > 0.0 {
                Some((target_level + 1).min(cfg.levels.len() - 1))
            } else {
                None
            },
        }
    }
}

/// LOD 组 - 管理一组网格的 LOD
pub struct LodGroup {
    /// 配置
    pub config: LodConfig,
    /// 各级别的网格 ID
    pub mesh_ids: Vec<String>,
}

impl LodGroup {
    /// 创建 LOD 组
    pub fn new(config: LodConfig) -> Self {
        let mesh_ids = config.levels.iter()
            .filter_map(|l| l.mesh_id.clone())
            .collect();
        
        Self { config, mesh_ids }
    }
    
    /// 获取当前应使用的网格 ID
    pub fn get_mesh_id(&self, level_index: usize) -> Option<&str> {
        self.mesh_ids.get(level_index).map(|s| s.as_str())
    }
}

/// LOD 统计信息
#[derive(Debug, Default)]
pub struct LodStats {
    /// 各级别的对象数量
    pub objects_per_level: [u32; 5],
    /// 总顶点数
    pub total_vertices: u64,
    /// 总三角形数
    pub total_triangles: u64,
    /// 正在过渡的对象数
    pub transitioning_count: u32,
}

impl LodStats {
    /// 记录一个 LOD 选择
    pub fn record(&mut self, selection: &LodSelection, vertex_count: u32, triangle_count: u32) {
        let level_idx = selection.quality.as_index().min(4);
        self.objects_per_level[level_idx] += 1;
        self.total_vertices += vertex_count as u64;
        self.total_triangles += triangle_count as u64;
        
        if selection.is_transitioning {
            self.transitioning_count += 1;
        }
    }
    
    /// 重置统计
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lod_config_builder() {
        let config = LodConfig::builder()
            .add_level(0.0, 10.0, LodQuality::High)
            .add_level(10.0, 30.0, LodQuality::Medium)
            .add_level(30.0, 100.0, LodQuality::Low)
            .with_transition(LodTransition::Instant)
            .build();
        
        assert_eq!(config.levels.len(), 3);
        assert_eq!(config.levels[0].quality, LodQuality::High);
    }
    
    #[test]
    fn test_lod_level_distance() {
        let level = LodLevel::new(10.0, 30.0, LodQuality::Medium);
        
        assert!(!level.contains_distance(5.0));
        assert!(level.contains_distance(15.0));
        assert!(level.contains_distance(25.0));
        assert!(!level.contains_distance(35.0));
    }
    
    #[test]
    fn test_lod_selector_instant() {
        let config = LodConfig::builder()
            .add_level(0.0, 10.0, LodQuality::High)
            .add_level(10.0, 30.0, LodQuality::Medium)
            .add_level(30.0, 100.0, LodQuality::Low)
            .with_transition(LodTransition::Instant)
            .build();
        
        let selector = LodSelector::new(config);
        
        let sel1 = selector.select_stateless(5.0, 1.0, &Mat4::IDENTITY);
        assert_eq!(sel1.quality, LodQuality::High);
        
        let sel2 = selector.select_stateless(20.0, 1.0, &Mat4::IDENTITY);
        assert_eq!(sel2.quality, LodQuality::Medium);
        
        let sel3 = selector.select_stateless(50.0, 1.0, &Mat4::IDENTITY);
        assert_eq!(sel3.quality, LodQuality::Low);
    }
    
    #[test]
    fn test_lod_selector_hysteresis() {
        let config = LodConfig::builder()
            .add_level(0.0, 10.0, LodQuality::High)
            .add_level(10.0, 30.0, LodQuality::Medium)
            .with_transition(LodTransition::Hysteresis { range: 0.2 })
            .build();
        
        let mut selector = LodSelector::new(config);
        
        // 开始在高质量
        let sel1 = selector.select(1, 5.0, 0.016);
        assert_eq!(sel1.quality, LodQuality::High);
        
        // 接近边界但未超过滞后范围
        let sel2 = selector.select(1, 11.0, 0.016);
        assert_eq!(sel2.quality, LodQuality::High); // 应保持高质量
        
        // 超过滞后范围
        let sel3 = selector.select(1, 13.0, 0.016);
        assert_eq!(sel3.quality, LodQuality::Medium); // 切换到中质量
    }
}
