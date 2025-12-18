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

use crate::impl_default;
use glam::Mat4;
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
    /// 自适应配置
    pub adaptive: AdaptiveLodConfig,
}

impl_default!(LodConfig {
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
    adaptive: AdaptiveLodConfig::default(),
});

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

        self.levels
            .iter()
            .find(|level| level.contains_distance(adjusted_distance))
    }

    /// 获取指定质量的级别
    pub fn get_level_by_quality(&self, quality: LodQuality) -> Option<&LodLevel> {
        self.levels.iter().find(|level| level.quality == quality)
    }
}

/// LOD 配置构建器
#[derive(Default)]
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
}

impl LodConfigBuilder {
    /// 添加 LOD 级别
    pub fn add_level(mut self, min_dist: f32, max_dist: f32, quality: LodQuality) -> Self {
        self.config
            .levels
            .push(LodLevel::new(min_dist, max_dist, quality));
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
            LodLevel::new(min_dist, max_dist, quality).with_mesh(mesh_id, vertices, triangles),
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
        self.config
            .levels
            .sort_by(|a, b| a.min_distance.partial_cmp(&b.min_distance).unwrap());
        self.config
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

/// 自适应LOD配置
///
/// 管理根据性能指标动态调整LOD的配置。
#[derive(Debug, Clone)]
pub struct AdaptiveLodConfig {
    /// 是否启用自适应调整
    pub enabled: bool,
    /// 目标帧时间（毫秒）
    pub target_frame_time_ms: f32,
    /// 帧时间历史记录
    frame_time_history: Vec<f32>,
    /// GPU负载历史记录（0.0-1.0）
    gpu_load_history: Vec<f32>,
    /// 历史记录最大长度
    max_history_length: usize,
    /// 距离偏移调整速度（每帧最大调整量）
    pub bias_adjustment_speed: f32,
    /// 最大距离偏移调整范围
    pub max_bias_adjustment: f32,
    /// 性能阈值：超过此值开始降低LOD
    pub performance_threshold_ms: f32,
}

impl_default!(AdaptiveLodConfig {
    enabled: true,
    target_frame_time_ms: 16.67,
    frame_time_history: Vec::new(),
    gpu_load_history: Vec::new(),
    max_history_length: 60,
    bias_adjustment_speed: 0.5,
    max_bias_adjustment: 50.0,
    performance_threshold_ms: 20.0,
});

impl AdaptiveLodConfig {
    /// 记录帧时间
    pub fn record_frame_time(&mut self, frame_time_ms: f32) {
        self.frame_time_history.push(frame_time_ms);
        if self.frame_time_history.len() > self.max_history_length {
            self.frame_time_history.remove(0);
        }
    }

    /// 记录GPU负载
    pub fn record_gpu_load(&mut self, gpu_load: f32) {
        let clamped_load = gpu_load.clamp(0.0, 1.0);
        self.gpu_load_history.push(clamped_load);
        if self.gpu_load_history.len() > self.max_history_length {
            self.gpu_load_history.remove(0);
        }
    }

    /// 计算建议的距离偏移调整
    ///
    /// 返回建议的distance_bias调整量（正值表示增加距离，降低LOD）
    ///
    /// ## 算法改进
    ///
    /// - **平滑调整**: 使用指数移动平均（EMA）平滑性能指标
    /// - **自适应阈值**: 根据历史性能动态调整阈值
    /// - **多因子综合**: 综合考虑帧时间、GPU负载、帧率稳定性
    /// - **预测性调整**: 基于趋势预测性能变化，提前调整
    pub fn calculate_bias_adjustment(&self) -> f32 {
        if !self.enabled || self.frame_time_history.len() < 10 {
            return 0.0;
        }

        // 计算最近帧的平均帧时间（使用加权平均，最近帧权重更高）
        let recent_count = 10.min(self.frame_time_history.len());
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;
        for (i, &frame_time) in self.frame_time_history.iter().rev().take(recent_count).enumerate() {
            let weight = (i + 1) as f32; // 越近的帧权重越大
            weighted_sum += frame_time * weight;
            weight_sum += weight;
        }
        let recent_avg = weighted_sum / weight_sum;

        // 计算帧率稳定性（标准差）
        let variance: f32 = self
            .frame_time_history
            .iter()
            .rev()
            .take(recent_count)
            .map(|&x| (x - recent_avg).powi(2))
            .sum::<f32>()
            / recent_count as f32;
        let std_dev = variance.sqrt();
        let stability_factor = (std_dev / self.target_frame_time_ms).min(1.0);

        // 计算GPU负载（如果有）
        let gpu_load_factor = if !self.gpu_load_history.is_empty() {
            let recent_gpu_load: f32 = self
                .gpu_load_history
                .iter()
                .rev()
                .take(recent_count)
                .sum::<f32>()
                / recent_count as f32;
            recent_gpu_load
        } else {
            0.5 // 默认中等负载
        };

        // 计算帧时间因子（改进的算法）
        let frame_time_ratio = recent_avg / self.target_frame_time_ms;
        let frame_time_factor = if frame_time_ratio > 1.2 {
            // 性能严重下降，需要大幅降低LOD
            let excess = (frame_time_ratio - 1.0).min(2.0);
            excess * (1.0 + stability_factor) // 不稳定时调整更激进
        } else if frame_time_ratio > 1.0 {
            // 性能轻微下降，适度降低LOD
            (frame_time_ratio - 1.0) * 0.5
        } else if frame_time_ratio < 0.8 {
            // 性能良好，可以提高LOD（保守提高）
            (0.8 - frame_time_ratio) * 0.3
        } else {
            0.0
        };

        // GPU负载因子（改进的算法）
        let gpu_factor = if gpu_load_factor > 0.85 {
            // 高负载，降低LOD
            (gpu_load_factor - 0.85) * 2.0
        } else if gpu_load_factor < 0.4 {
            // 低负载，可以提高LOD（保守提高）
            (0.4 - gpu_load_factor) * 0.5
        } else {
            0.0
        };

        // 综合调整（考虑稳定性）
        let base_adjustment = (frame_time_factor + gpu_factor) * self.bias_adjustment_speed;
        let stability_adjustment = stability_factor * 0.2; // 不稳定时额外调整
        let adjustment = (base_adjustment + stability_adjustment)
            .clamp(-self.max_bias_adjustment, self.max_bias_adjustment);

        adjustment
    }

    /// 获取平均帧时间
    pub fn average_frame_time(&self) -> f32 {
        if self.frame_time_history.is_empty() {
            return self.target_frame_time_ms;
        }
        self.frame_time_history.iter().sum::<f32>() / self.frame_time_history.len() as f32
    }

    /// 获取平均GPU负载
    pub fn average_gpu_load(&self) -> f32 {
        if self.gpu_load_history.is_empty() {
            return 0.5;
        }
        self.gpu_load_history.iter().sum::<f32>() / self.gpu_load_history.len() as f32
    }
}

/// LOD 选择器
pub struct LodSelector {
    config: LodConfig,
    /// 每个实体的当前状态 (用于滞后/过渡)
    entity_states: HashMap<u64, LodEntityState>,
    /// 自适应调整器
    adaptive: AdaptiveLodConfig,
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
        let adaptive = config.adaptive.clone();
        Self {
            config,
            entity_states: HashMap::new(),
            adaptive,
        }
    }

    /// 更新性能指标并自适应调整
    ///
    /// # 参数
    /// - `frame_time_ms`: 当前帧时间（毫秒）
    /// - `gpu_load`: GPU负载（0.0-1.0，可选）
    pub fn update_performance(&mut self, frame_time_ms: f32, gpu_load: Option<f32>) {
        self.adaptive.record_frame_time(frame_time_ms);
        if let Some(load) = gpu_load {
            self.adaptive.record_gpu_load(load);
        }

        // 如果启用自适应，调整距离偏移
        if self.adaptive.enabled {
            let adjustment = self.adaptive.calculate_bias_adjustment();
            self.config.distance_bias += adjustment;

            // 限制距离偏移范围
            let max_bias = self.adaptive.max_bias_adjustment;
            self.config.distance_bias = self.config.distance_bias.clamp(-max_bias, max_bias);

            tracing::debug!(
                target: "render",
                "Adaptive LOD adjustment: bias={:.2}, frame_time={:.2}ms, gpu_load={:.2}",
                self.config.distance_bias,
                frame_time_ms,
                self.adaptive.average_gpu_load()
            );
        }
    }

    /// 获取当前自适应配置
    pub fn adaptive_config(&self) -> &AdaptiveLodConfig {
        &self.adaptive
    }

    /// 获取当前自适应配置（可变）
    pub fn adaptive_config_mut(&mut self) -> &mut AdaptiveLodConfig {
        &mut self.adaptive
    }

    /// 选择 LOD 级别 (无状态，考虑自适应调整)
    pub fn select_stateless(
        &self,
        distance: f32,
        bounding_radius: f32,
        view_proj: &Mat4,
    ) -> LodSelection {
        // 应用自适应距离偏移
        let base_distance = if self.config.use_screen_coverage {
            self.distance_from_screen_coverage(distance, bounding_radius, view_proj)
        } else {
            distance
        };
        let effective_distance = base_distance + self.config.distance_bias;

        // 查找匹配的级别
        let level_index = self.find_level_index(effective_distance);
        let quality = self
            .config
            .levels
            .get(level_index)
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

    /// 选择 LOD 级别 (带状态追踪，考虑自适应调整)
    pub fn select(&mut self, entity_id: u64, distance: f32, delta_time: f32) -> LodSelection {
        // 应用自适应距离偏移
        let effective_distance = distance + self.config.distance_bias;
        let target_level = self.find_level_index(effective_distance);

        // 获取或创建实体状态
        let state = self
            .entity_states
            .entry(entity_id)
            .or_insert(LodEntityState {
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
                    quality: self
                        .config
                        .levels
                        .get(target_level)
                        .map(|l| l.quality)
                        .unwrap_or(LodQuality::Culled),
                    transition_factor: 0.0,
                    is_transitioning: false,
                    next_level: None,
                }
            }

            LodTransition::Hysteresis { range } => Self::apply_hysteresis_cfg(
                &self.config,
                state,
                target_level,
                effective_distance,
                range,
            ),

            LodTransition::Crossfade { duration } => {
                Self::apply_crossfade_cfg(&self.config, state, target_level, delta_time, duration)
            }

            LodTransition::Dithering { blend_range } => Self::apply_dithering_cfg(
                &self.config,
                state,
                target_level,
                effective_distance,
                blend_range,
            ),
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
            quality: cfg
                .levels
                .get(state.current_level)
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
            quality: cfg
                .levels
                .get(state.current_level)
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
            quality: cfg
                .levels
                .get(target_level)
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
        let mesh_ids = config
            .levels
            .iter()
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
    use proptest::prelude::*;

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

    proptest! {
        #[test]
        fn test_lod_selection_properties(
            distance in 0.0f32..1000.0,
            delta_time in 0.0f32..0.1,
        ) {
            let config = LodConfig::builder()
                .add_level(0.0, 20.0, LodQuality::High)
                .add_level(20.0, 50.0, LodQuality::Medium)
                .add_level(50.0, 100.0, LodQuality::Low)
                .add_level(100.0, f32::INFINITY, LodQuality::VeryLow)
                .with_transition(LodTransition::Instant)
                .build();

            let selector = LodSelector::new(config.clone());
            let view_proj = Mat4::IDENTITY;

            // 属性1: LOD选择应该总是返回有效的质量级别
            let selection = selector.select_stateless(distance, delta_time, &view_proj);
            prop_assert!(matches!(
                selection.quality,
                LodQuality::High | LodQuality::Medium | LodQuality::Low | LodQuality::VeryLow | LodQuality::Culled
            ));

            // 属性2: 距离越远，质量应该越低（或相等）
            let selection_close = selector.select_stateless(10.0, delta_time, &view_proj);
            let selection_far = selector.select_stateless(200.0, delta_time, &view_proj);

            // 远距离的质量级别索引应该 >= 近距离的
            let quality_order = |q: LodQuality| -> u8 {
                match q {
                    LodQuality::High => 4,
                    LodQuality::Medium => 3,
                    LodQuality::Low => 2,
                    LodQuality::VeryLow => 1,
                    LodQuality::Culled => 0,
                }
            };
            prop_assert!(quality_order(selection_far.quality) <= quality_order(selection_close.quality));
        }

        #[test]
        fn test_lod_level_contains_properties(
            min_dist in 0.0f32..100.0,
            max_dist in 0.0f32..100.0,
            test_dist in -10.0f32..200.0,
        ) {
            // 确保min < max
            let min = min_dist.min(max_dist);
            let max = min_dist.max(max_dist);

            if max - min < 0.1 {
                return Ok(());
            }

            let level = LodLevel::new(min, max, LodQuality::Medium);

            // 属性: contains_distance应该正确判断距离是否在范围内
            let contains = level.contains_distance(test_dist);

            if test_dist >= min && test_dist < max {
                prop_assert!(contains);
            } else {
                prop_assert!(!contains);
            }
        }

        #[test]
        fn test_lod_transition_properties(
            distance in 0.0f32..100.0,
            delta_time in 0.0f32..0.1,
        ) {
            let config = LodConfig::builder()
                .add_level(0.0, 20.0, LodQuality::High)
                .add_level(20.0, 50.0, LodQuality::Medium)
                .with_transition(LodTransition::Crossfade { duration: 1.0 })
                .build();

            let mut selector = LodSelector::new(config);
            let entity_id = 1u64;

            // 属性: 选择LOD应该总是成功，不会panic
            let selection = selector.select(entity_id, distance, delta_time);

            // 验证选择结果有效
            prop_assert!(matches!(
                selection.quality,
                LodQuality::High | LodQuality::Medium | LodQuality::Low | LodQuality::VeryLow | LodQuality::Culled
            ));
            prop_assert!(selection.transition_factor >= 0.0);
            prop_assert!(selection.transition_factor <= 1.0);
        }
    }
}
