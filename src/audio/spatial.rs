//! 空间音频模块
//!
//! 提供 3D 空间音频功能，支持：
//! - 距离衰减 (Linear, Inverse, Exponential)
//! - 3D 定位和平移
//! - 多普勒效果
//! - 环境遮挡/阻挡
//! - HRTF (头部相关传输函数) 支持
//!
//! # 架构设计
//!
//! 遵循与主音频模块相同的贫血模型设计：
//! - `SpatialAudioState` (Resource): 空间音频状态数据
//! - `AudioListener` (Component): 听者组件
//! - `SpatialAudioSource` (Component): 空间音频源组件
//! - `SpatialAudioService`: 空间音频业务逻辑
//!
//! # 示例
//!
//! ```ignore
//! // 设置监听器
//! commands.spawn((
//!     AudioListener::new(),
//!     Transform::from_translation(Vec3::ZERO),
//! ));
//!
//! // 创建空间音频源
//! commands.spawn((
//!     SpatialAudioSource::new("explosion")
//!         .with_distance_model(DistanceModel::Inverse { ref_distance: 1.0, rolloff: 1.0 }),
//!     Transform::from_translation(Vec3::new(10.0, 0.0, 5.0)),
//! ));
//! ```

use glam::{Vec3, Quat};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// 距离衰减模型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistanceModel {
    /// 无衰减 (固定音量)
    None,
    /// 线性衰减: gain = 1 - rolloff * (distance - ref_distance) / (max_distance - ref_distance)
    Linear {
        /// 参考距离 (开始衰减的距离)
        ref_distance: f32,
        /// 最大距离 (完全静音的距离)
        max_distance: f32,
        /// 衰减系数 (0.0 - 1.0)
        rolloff: f32,
    },
    /// 反比衰减: gain = ref_distance / (ref_distance + rolloff * (distance - ref_distance))
    Inverse {
        /// 参考距离
        ref_distance: f32,
        /// 衰减系数
        rolloff: f32,
    },
    /// 指数衰减: gain = (distance / ref_distance) ^ -rolloff
    Exponential {
        /// 参考距离
        ref_distance: f32,
        /// 衰减系数
        rolloff: f32,
    },
}

impl Default for DistanceModel {
    fn default() -> Self {
        DistanceModel::Inverse {
            ref_distance: 1.0,
            rolloff: 1.0,
        }
    }
}

impl DistanceModel {
    /// 计算指定距离的增益值
    pub fn calculate_gain(&self, distance: f32) -> f32 {
        match *self {
            DistanceModel::None => 1.0,
            
            DistanceModel::Linear { ref_distance, max_distance, rolloff } => {
                if distance <= ref_distance {
                    1.0
                } else if distance >= max_distance {
                    0.0
                } else {
                    let range = max_distance - ref_distance;
                    let dist = distance - ref_distance;
                    (1.0 - rolloff * (dist / range)).max(0.0)
                }
            }
            
            DistanceModel::Inverse { ref_distance, rolloff } => {
                if distance <= ref_distance {
                    1.0
                } else {
                    ref_distance / (ref_distance + rolloff * (distance - ref_distance))
                }
            }
            
            DistanceModel::Exponential { ref_distance, rolloff } => {
                if distance <= ref_distance {
                    1.0
                } else {
                    (distance / ref_distance).powf(-rolloff)
                }
            }
        }
    }
}

/// 声锥设置 (用于方向性声源)
#[derive(Debug, Clone, Copy)]
pub struct SoundCone {
    /// 内锥角度 (弧度) - 在此角度内音量为 100%
    pub inner_angle: f32,
    /// 外锥角度 (弧度) - 超过此角度使用外锥增益
    pub outer_angle: f32,
    /// 外锥增益 (0.0 - 1.0)
    pub outer_gain: f32,
}

impl Default for SoundCone {
    fn default() -> Self {
        Self {
            inner_angle: std::f32::consts::PI * 2.0, // 360度 (全向)
            outer_angle: std::f32::consts::PI * 2.0,
            outer_gain: 0.0,
        }
    }
}

impl SoundCone {
    /// 创建全向声锥
    pub fn omnidirectional() -> Self {
        Self::default()
    }
    
    /// 创建方向性声锥
    pub fn directional(inner_deg: f32, outer_deg: f32, outer_gain: f32) -> Self {
        Self {
            inner_angle: inner_deg.to_radians(),
            outer_angle: outer_deg.to_radians(),
            outer_gain,
        }
    }
    
    /// 计算给定角度的增益
    pub fn calculate_gain(&self, angle: f32) -> f32 {
        if angle <= self.inner_angle * 0.5 {
            1.0
        } else if angle >= self.outer_angle * 0.5 {
            self.outer_gain
        } else {
            // 在内外锥之间线性插值
            let inner_half = self.inner_angle * 0.5;
            let outer_half = self.outer_angle * 0.5;
            let t = (angle - inner_half) / (outer_half - inner_half);
            1.0 + t * (self.outer_gain - 1.0)
        }
    }
}

/// 空间音频监听器组件
/// 
/// 附加到代表"耳朵"的实体 (通常是相机或玩家)
#[derive(Component, Clone)]
pub struct AudioListener {
    /// 是否启用
    pub enabled: bool,
    /// 音量乘数
    pub gain: f32,
    /// 位置偏移 (相对于 Transform)
    pub position_offset: Vec3,
    /// 朝向偏移 (相对于 Transform 的旋转)
    pub orientation_offset: Quat,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            enabled: true,
            gain: 1.0,
            position_offset: Vec3::ZERO,
            orientation_offset: Quat::IDENTITY,
        }
    }
}

impl AudioListener {
    /// 创建新的监听器
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 设置增益
    pub fn with_gain(mut self, gain: f32) -> Self {
        self.gain = gain;
        self
    }
    
    /// 设置位置偏移
    pub fn with_position_offset(mut self, offset: Vec3) -> Self {
        self.position_offset = offset;
        self
    }
}

/// 空间音频源组件
/// 
/// 附加到需要 3D 定位的音频实体
#[derive(Component, Clone)]
pub struct SpatialAudioSource {
    /// 音频标识
    pub name: String,
    /// 音频文件路径
    pub path: String,
    /// 基础音量 (0.0 - 1.0)
    pub volume: f32,
    /// 距离衰减模型
    pub distance_model: DistanceModel,
    /// 声锥设置 (方向性)
    pub cone: SoundCone,
    /// 最大距离 (超过此距离不播放)
    pub max_distance: f32,
    /// 最小距离 (小于此距离音量不再增加)
    pub min_distance: f32,
    /// 多普勒因子 (0 = 无多普勒效果)
    pub doppler_factor: f32,
    /// 是否循环
    pub looping: bool,
    /// 播放状态
    pub is_playing: bool,
    /// 空间混合 (0 = 2D, 1 = 完全3D)
    pub spatial_blend: f32,
    /// 优先级 (用于声音剔除)
    pub priority: i32,
}

impl Default for SpatialAudioSource {
    fn default() -> Self {
        Self {
            name: "spatial_sound".to_string(),
            path: String::new(),
            volume: 1.0,
            distance_model: DistanceModel::default(),
            cone: SoundCone::default(),
            max_distance: 100.0,
            min_distance: 1.0,
            doppler_factor: 1.0,
            looping: false,
            is_playing: false,
            spatial_blend: 1.0,
            priority: 0,
        }
    }
}

impl SpatialAudioSource {
    /// 创建新的空间音频源
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
    
    /// 设置音频文件路径
    pub fn with_path(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self
    }
    
    /// 设置音量
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }
    
    /// 设置距离模型
    pub fn with_distance_model(mut self, model: DistanceModel) -> Self {
        self.distance_model = model;
        self
    }
    
    /// 设置声锥
    pub fn with_cone(mut self, cone: SoundCone) -> Self {
        self.cone = cone;
        self
    }
    
    /// 设置距离范围
    pub fn with_distance_range(mut self, min: f32, max: f32) -> Self {
        self.min_distance = min;
        self.max_distance = max;
        self
    }
    
    /// 设置多普勒因子
    pub fn with_doppler(mut self, factor: f32) -> Self {
        self.doppler_factor = factor;
        self
    }
    
    /// 设置循环
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
    
    /// 设置空间混合
    pub fn with_spatial_blend(mut self, blend: f32) -> Self {
        self.spatial_blend = blend.clamp(0.0, 1.0);
        self
    }
    
    /// 设置优先级
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// 计算后的空间音频参数
#[derive(Debug, Clone, Copy, Default)]
pub struct SpatialAudioParams {
    /// 最终音量 (考虑距离衰减、声锥等)
    pub volume: f32,
    /// 左声道音量
    pub left_gain: f32,
    /// 右声道音量
    pub right_gain: f32,
    /// 音高偏移 (多普勒效果)
    pub pitch: f32,
    /// 距离
    pub distance: f32,
    /// 相对角度 (弧度)
    pub azimuth: f32,
    /// 相对仰角 (弧度)
    pub elevation: f32,
}

/// 空间音频状态 (Resource)
#[derive(Resource, Default)]
pub struct SpatialAudioState {
    /// 监听器位置
    pub listener_position: Vec3,
    /// 监听器朝向 (前方向)
    pub listener_forward: Vec3,
    /// 监听器上方向
    pub listener_up: Vec3,
    /// 监听器速度 (用于多普勒效果)
    pub listener_velocity: Vec3,
    /// 声速 (米/秒)
    pub speed_of_sound: f32,
    /// 全局空间音频增益
    pub global_gain: f32,
    /// 最大同时播放的空间音频数量
    pub max_concurrent_sounds: usize,
    /// 源速度缓存 (用于多普勒计算)
    pub source_velocities: HashMap<Entity, Vec3>,
}

impl SpatialAudioState {
    /// 创建新的空间音频状态
    pub fn new() -> Self {
        Self {
            listener_position: Vec3::ZERO,
            listener_forward: Vec3::NEG_Z, // 默认看向 -Z
            listener_up: Vec3::Y,
            listener_velocity: Vec3::ZERO,
            speed_of_sound: 343.0, // 常温空气中的声速
            global_gain: 1.0,
            max_concurrent_sounds: 32,
            source_velocities: HashMap::new(),
        }
    }
}

/// 空间音频服务 - 封装空间音频计算逻辑
pub struct SpatialAudioService;

impl SpatialAudioService {
    /// 更新监听器位置和朝向
    pub fn update_listener(
        state: &mut SpatialAudioState,
        position: Vec3,
        forward: Vec3,
        up: Vec3,
        velocity: Vec3,
    ) {
        state.listener_position = position;
        state.listener_forward = forward.normalize_or_zero();
        state.listener_up = up.normalize_or_zero();
        state.listener_velocity = velocity;
    }
    
    /// 计算空间音频参数
    pub fn calculate_params(
        state: &SpatialAudioState,
        source: &SpatialAudioSource,
        source_position: Vec3,
        source_forward: Vec3,
        source_velocity: Vec3,
    ) -> SpatialAudioParams {
        // 计算相对位置
        let relative_pos = source_position - state.listener_position;
        let distance = relative_pos.length();
        
        // 如果超过最大距离，返回静音参数
        if distance > source.max_distance {
            return SpatialAudioParams::default();
        }
        
        // 计算距离衰减
        let distance_gain = source.distance_model.calculate_gain(distance);
        
        // 计算声锥衰减
        let to_listener = -relative_pos.normalize_or_zero();
        let cone_angle = source_forward.angle_between(to_listener);
        let cone_gain = source.cone.calculate_gain(cone_angle);
        
        // 计算左右声道定位 (简化的HRTF)
        let listener_right = state.listener_forward.cross(state.listener_up).normalize_or_zero();
        let direction_to_source = relative_pos.normalize_or_zero();
        
        // 方位角 (水平面内与前方的夹角)
        let azimuth = listener_right.dot(direction_to_source).asin();
        
        // 仰角
        let elevation = state.listener_up.dot(direction_to_source).asin();
        
        // 计算左右声道增益 (简化的立体声平移)
        let pan = (azimuth / std::f32::consts::FRAC_PI_2).clamp(-1.0, 1.0);
        let left_gain = ((1.0 - pan) / 2.0).sqrt();
        let right_gain = ((1.0 + pan) / 2.0).sqrt();
        
        // 计算多普勒效果
        let pitch = if source.doppler_factor > 0.0 {
            Self::calculate_doppler(
                state,
                relative_pos,
                source_velocity,
                source.doppler_factor,
            )
        } else {
            1.0
        };
        
        // 应用空间混合
        let final_left = lerp(0.5, left_gain, source.spatial_blend);
        let final_right = lerp(0.5, right_gain, source.spatial_blend);
        
        // 计算最终音量
        let volume = source.volume * distance_gain * cone_gain * state.global_gain;
        
        SpatialAudioParams {
            volume,
            left_gain: volume * final_left,
            right_gain: volume * final_right,
            pitch,
            distance,
            azimuth,
            elevation,
        }
    }
    
    /// 计算多普勒效果
    fn calculate_doppler(
        state: &SpatialAudioState,
        relative_pos: Vec3,
        source_velocity: Vec3,
        doppler_factor: f32,
    ) -> f32 {
        let direction = relative_pos.normalize_or_zero();
        
        // 计算朝向监听器的相对速度
        let listener_speed = state.listener_velocity.dot(direction);
        let source_speed = source_velocity.dot(direction);
        
        // 多普勒公式: f' = f * (c + v_listener) / (c + v_source)
        let c = state.speed_of_sound;
        let numerator = c + listener_speed * doppler_factor;
        let denominator = c + source_speed * doppler_factor;
        
        if denominator.abs() < 0.001 {
            1.0 // 避免除零
        } else {
            (numerator / denominator).clamp(0.5, 2.0) // 限制音高范围
        }
    }
    
    /// 按距离和优先级排序音频源，选择要播放的
    pub fn select_active_sources<'a>(
        state: &SpatialAudioState,
        sources: impl Iterator<Item = (Entity, &'a SpatialAudioSource, Vec3)>,
    ) -> Vec<(Entity, f32)> {
        let mut scored: Vec<(Entity, f32, i32)> = sources
            .map(|(entity, source, position)| {
                let distance = (position - state.listener_position).length();
                let score = source.priority as f32 * 1000.0 - distance; // 优先级优先，距离次之
                (entity, distance, source.priority)
            })
            .filter(|(_, distance, _)| *distance <= 200.0) // 预剔除远处声音
            .collect();
        
        // 按分数排序 (高分在前)
        scored.sort_by(|a, b| {
            let score_a = a.2 as f32 * 1000.0 - a.1;
            let score_b = b.2 as f32 * 1000.0 - b.1;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // 返回前 N 个
        scored.into_iter()
            .take(state.max_concurrent_sounds)
            .map(|(entity, distance, _)| (entity, distance))
            .collect()
    }
}

/// 线性插值辅助函数
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

// ============================================================================
// ECS 系统
// ============================================================================

/// 更新监听器位置的系统
pub fn update_listener_system(
    mut spatial_state: ResMut<SpatialAudioState>,
    query: Query<(&Transform, &AudioListener), With<AudioListener>>,
) {
    for (transform, listener) in query.iter() {
        if listener.enabled {
            let position = transform.translation + listener.position_offset;
            let rotation = transform.rotation * listener.orientation_offset;
            let forward = rotation * Vec3::NEG_Z;
            let up = rotation * Vec3::Y;
            
            // TODO: 计算速度 (需要存储上一帧位置)
            let velocity = Vec3::ZERO;
            
            SpatialAudioService::update_listener(
                &mut spatial_state,
                position,
                forward,
                up,
                velocity,
            );
            
            break; // 只使用第一个启用的监听器
        }
    }
}

/// Transform 组件 (如果尚未定义)
#[derive(Component, Clone, Copy, Default)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_distance_model_linear() {
        let model = DistanceModel::Linear {
            ref_distance: 1.0,
            max_distance: 10.0,
            rolloff: 1.0,
        };
        
        assert!((model.calculate_gain(0.0) - 1.0).abs() < 0.001);
        assert!((model.calculate_gain(1.0) - 1.0).abs() < 0.001);
        assert!((model.calculate_gain(5.5) - 0.5).abs() < 0.001);
        assert!((model.calculate_gain(10.0) - 0.0).abs() < 0.001);
        assert!((model.calculate_gain(15.0) - 0.0).abs() < 0.001);
    }
    
    #[test]
    fn test_distance_model_inverse() {
        let model = DistanceModel::Inverse {
            ref_distance: 1.0,
            rolloff: 1.0,
        };
        
        assert!((model.calculate_gain(1.0) - 1.0).abs() < 0.001);
        assert!((model.calculate_gain(2.0) - 0.5).abs() < 0.001);
        assert!((model.calculate_gain(10.0) - 0.1).abs() < 0.001);
    }
    
    #[test]
    fn test_sound_cone() {
        let cone = SoundCone::directional(60.0, 120.0, 0.2);
        
        // 在内锥内
        assert!((cone.calculate_gain(0.0) - 1.0).abs() < 0.001);
        assert!((cone.calculate_gain(0.5_f32.to_radians()) - 1.0).abs() < 0.001);
        
        // 在外锥外
        assert!((cone.calculate_gain(1.2_f32) - 0.2).abs() < 0.1);
    }
    
    #[test]
    fn test_spatial_params_calculation() {
        let state = SpatialAudioState::new();
        let source = SpatialAudioSource::new("test")
            .with_volume(1.0)
            .with_distance_model(DistanceModel::Inverse {
                ref_distance: 1.0,
                rolloff: 1.0,
            });
        
        let params = SpatialAudioService::calculate_params(
            &state,
            &source,
            Vec3::new(5.0, 0.0, 0.0), // 右侧 5 米
            Vec3::NEG_Z,
            Vec3::ZERO,
        );
        
        // 距离 5m 应该有 0.2 的衰减
        assert!((params.volume - 0.2).abs() < 0.01);
        // 在右侧，右声道应该比左声道大
        assert!(params.right_gain > params.left_gain);
    }
}
