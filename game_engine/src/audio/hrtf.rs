//! HRTF (头部相关传输函数) 滤波器模块
//!
//! 提供基于HRTF的3D空间音频定位，支持：
//! - 基于方位角和仰角的HRTF滤波
//! - ITD (双耳时间差) 和 ILD (双耳强度差) 计算
//! - 多普勒效应计算
//! - 可配置的HRTF数据集
//!
//! ## HRTF原理
//!
//! HRTF描述了声音从空间中的某个点到达双耳时的频率响应差异。
//! 这些差异包括：
//! - **ITD (Interaural Time Difference)**: 声音到达左右耳的时间差
//! - **ILD (Interaural Level Difference)**: 左右耳的强度差异
//! - **频谱变化**: 头部、耳廓和躯干对声音的滤波效应
//!
//! ## 使用示例
//!
//! ```ignore
//! use game_engine::audio::hrtf::{HrtfFilter, HrtfConfig};
//!
//! // 创建HRTF滤波器
//! let mut hrtf = HrtfFilter::new(HrtfConfig::default());
//!
//! // 更新声源位置（相对于监听器）
//! hrtf.update_source_position(azimuth, elevation, distance);
//!
//! // 处理音频样本
//! let (left_samples, right_samples) = hrtf.process_mono(&mono_samples);
//! ```

use crate::core::validation::validators;
use crate::core::validation::{Validate, ValidationError};
use glam::Vec3;
use std::f32::consts::PI;

/// HRTF配置
#[derive(Debug, Clone)]
pub struct HrtfConfig {
    /// 采样率 (Hz)
    pub sample_rate: f32,
    /// 头部半径 (米) - 用于ITD计算
    pub head_radius: f32,
    /// 声速 (米/秒)
    pub speed_of_sound: f32,
    /// 启用ITD (双耳时间差)
    pub enable_itd: bool,
    /// 启用ILD (双耳强度差)
    pub enable_ild: bool,
    /// 启用频谱滤波
    pub enable_spectral_filtering: bool,
    /// 最大ITD延迟 (秒)
    pub max_itd_delay: f32,
    /// 低通滤波截止频率 (Hz) - 用于模拟头部阴影
    pub shadow_filter_cutoff: f32,
}

impl Default for HrtfConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100.0,
            head_radius: 0.0875, // 平均成人头部半径
            speed_of_sound: 343.0,
            enable_itd: true,
            enable_ild: true,
            enable_spectral_filtering: true,
            max_itd_delay: 0.0007, // 约0.7ms，对应90度方位角
            shadow_filter_cutoff: 2000.0,
        }
    }
}

impl Validate for HrtfConfig {
    type Error = ValidationError;

    fn validate(&self) -> Result<(), Self::Error> {
        // 验证采样率（常见音频采样率：8000 - 192000 Hz）
        validators::validate_range(self.sample_rate, 8000.0, 192000.0)?;

        // 验证头部半径（0.01m - 0.15m）
        validators::validate_range(self.head_radius, 0.01, 0.15)?;

        // 验证声速（300 - 400 m/s，考虑不同温度和海拔）
        validators::validate_range(self.speed_of_sound, 300.0, 400.0)?;

        // 验证最大ITD延迟（0 - 0.001秒，即1ms）
        validators::validate_range(self.max_itd_delay, 0.0, 0.001)?;

        // 验证低通滤波截止频率（20 - 20000 Hz，人耳听力范围）
        validators::validate_range(self.shadow_filter_cutoff, 20.0, 20000.0)?;

        Ok(())
    }
}

/// HRTF滤波器
pub struct HrtfFilter {
    config: HrtfConfig,
    /// 当前方位角 (弧度, -π 到 π)
    azimuth: f32,
    /// 当前仰角 (弧度, -π/2 到 π/2)
    elevation: f32,
    /// 当前距离 (米)
    distance: f32,
    /// ITD延迟缓冲区 (左耳)
    left_delay_buffer: Vec<f32>,
    /// ITD延迟缓冲区 (右耳)
    right_delay_buffer: Vec<f32>,
    /// 延迟缓冲区写入索引
    delay_write_index: usize,
    /// 低通滤波器状态 (左耳)
    left_lowpass_state: f32,
    /// 低通滤波器状态 (右耳)
    right_lowpass_state: f32,
}

impl HrtfFilter {
    /// 创建新的HRTF滤波器
    pub fn new(config: HrtfConfig) -> Self {
        let max_delay_samples = (config.max_itd_delay * config.sample_rate).ceil() as usize + 1;

        Self {
            config,
            azimuth: 0.0,
            elevation: 0.0,
            distance: 1.0,
            left_delay_buffer: vec![0.0; max_delay_samples],
            right_delay_buffer: vec![0.0; max_delay_samples],
            delay_write_index: 0,
            left_lowpass_state: 0.0,
            right_lowpass_state: 0.0,
        }
    }

    /// 更新声源位置
    ///
    /// # Arguments
    /// * `azimuth` - 方位角 (弧度, -π 到 π, 0 = 正前方, π/2 = 右侧)
    /// * `elevation` - 仰角 (弧度, -π/2 到 π/2, 0 = 水平, π/2 = 上方)
    /// * `distance` - 距离 (米)
    pub fn update_source_position(&mut self, azimuth: f32, elevation: f32, distance: f32) {
        self.azimuth = azimuth.clamp(-PI, PI);
        self.elevation = elevation.clamp(-PI / 2.0, PI / 2.0);
        self.distance = distance.max(0.01);
    }

    /// 从相对位置更新声源位置
    ///
    /// # Arguments
    /// * `relative_position` - 声源相对于监听器的位置
    /// * `listener_forward` - 监听器前方向
    /// * `listener_up` - 监听器上方向
    pub fn update_from_relative_position(
        &mut self,
        relative_position: Vec3,
        listener_forward: Vec3,
        listener_up: Vec3,
    ) {
        let distance = relative_position.length();
        if distance < 0.01 {
            self.azimuth = 0.0;
            self.elevation = 0.0;
            self.distance = 0.01;
            return;
        }

        let direction = relative_position / distance;
        let listener_right = listener_forward.cross(listener_up).normalize();

        // 计算方位角 (水平面内与前方的夹角)
        let forward_component = direction.dot(listener_forward);
        let right_component = direction.dot(listener_right);
        self.azimuth = right_component.atan2(forward_component);

        // 计算仰角
        let up_component = direction.dot(listener_up);
        self.elevation = up_component.asin();

        self.distance = distance;
    }

    /// 计算ITD (双耳时间差)
    ///
    /// 返回 (左耳延迟, 右耳延迟) 以秒为单位
    fn calculate_itd(&self) -> (f32, f32) {
        if !self.config.enable_itd {
            return (0.0, 0.0);
        }

        // 使用Woodworth模型计算ITD
        // ITD = (head_radius / speed_of_sound) * (azimuth + sin(azimuth))
        let head_radius = self.config.head_radius;
        let speed = self.config.speed_of_sound;

        // 考虑仰角的影响（简化模型）
        let effective_azimuth = self.azimuth * self.elevation.cos();

        let itd_base = (head_radius / speed) * (effective_azimuth + effective_azimuth.sin());
        let itd = itd_base.clamp(-self.config.max_itd_delay, self.config.max_itd_delay);

        // 如果声源在右侧，右耳先听到（ITD为负）
        if self.azimuth > 0.0 {
            (itd.abs(), -itd.abs())
        } else {
            (-itd.abs(), itd.abs())
        }
    }

    /// 计算ILD (双耳强度差)
    ///
    /// 返回 (左耳增益, 右耳增益)
    pub fn calculate_ild(&self) -> (f32, f32) {
        if !self.config.enable_ild {
            return (1.0, 1.0);
        }

        // 简化的ILD模型：基于方位角的余弦函数
        // 当声源在右侧时，右耳增益高，左耳增益低（头部阴影效应）
        let azimuth_normalized = self.azimuth / PI; // -1 到 1

        // 使用余弦函数模拟头部阴影
        let shadow_factor = (azimuth_normalized * PI / 2.0).cos();

        // 左耳增益：声源在右侧时降低
        let left_gain = if self.azimuth > 0.0 {
            0.5 + 0.5 * shadow_factor
        } else {
            1.0
        };

        // 右耳增益：声源在左侧时降低
        let right_gain = if self.azimuth < 0.0 {
            0.5 + 0.5 * shadow_factor
        } else {
            1.0
        };

        // 考虑仰角的影响（简化）
        let elevation_factor = 1.0 - (self.elevation.abs() / (PI / 2.0)) * 0.2;

        (left_gain * elevation_factor, right_gain * elevation_factor)
    }

    /// 处理单声道音频，输出立体声
    ///
    /// # Arguments
    /// * `mono_samples` - 输入单声道音频样本
    ///
    /// # Returns
    /// (左声道样本, 右声道样本)
    pub fn process_mono(&mut self, mono_samples: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let (itd_left, itd_right) = self.calculate_itd();
        let (ild_left, ild_right) = self.calculate_ild();

        let delay_samples_left = (itd_left * self.config.sample_rate).round() as i32;
        let delay_samples_right = (itd_right * self.config.sample_rate).round() as i32;

        let buffer_size = self.left_delay_buffer.len();
        let mut left_output = Vec::with_capacity(mono_samples.len());
        let mut right_output = Vec::with_capacity(mono_samples.len());

        for &sample in mono_samples {
            // 写入延迟缓冲区
            self.left_delay_buffer[self.delay_write_index] = sample;
            self.right_delay_buffer[self.delay_write_index] = sample;

            // 读取延迟后的样本
            let left_delay_index = (self.delay_write_index as i32 - delay_samples_left
                + buffer_size as i32) as usize
                % buffer_size;
            let right_delay_index = (self.delay_write_index as i32 - delay_samples_right
                + buffer_size as i32) as usize
                % buffer_size;

            let mut left_sample = self.left_delay_buffer[left_delay_index];
            let mut right_sample = self.right_delay_buffer[right_delay_index];

            // 应用ILD增益
            left_sample *= ild_left;
            right_sample *= ild_right;

            // 应用头部阴影滤波（低通滤波）
            if self.config.enable_spectral_filtering {
                left_sample = self.apply_shadow_filter(left_sample, true);
                right_sample = self.apply_shadow_filter(right_sample, false);
            }

            left_output.push(left_sample);
            right_output.push(right_sample);

            // 更新写入索引
            self.delay_write_index = (self.delay_write_index + 1) % buffer_size;
        }

        (left_output, right_output)
    }

    /// 应用头部阴影滤波（低通滤波）
    ///
    /// 当声源在头部另一侧时，高频被衰减
    fn apply_shadow_filter(&mut self, sample: f32, is_left: bool) -> f32 {
        // 判断是否需要应用阴影滤波
        let needs_shadow = if is_left {
            self.azimuth > 0.0 // 声源在右侧，左耳需要阴影
        } else {
            self.azimuth < 0.0 // 声源在左侧，右耳需要阴影
        };

        if !needs_shadow {
            return sample;
        }

        // 简单的一阶低通滤波器
        let cutoff = self.config.shadow_filter_cutoff;
        let sample_rate = self.config.sample_rate;
        let rc = 1.0 / (2.0 * PI * cutoff);
        let dt = 1.0 / sample_rate;
        let alpha = dt / (rc + dt);

        let state = if is_left {
            &mut self.left_lowpass_state
        } else {
            &mut self.right_lowpass_state
        };

        *state = *state + alpha * (sample - *state);
        *state
    }

    /// 获取当前配置
    pub fn config(&self) -> &HrtfConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: HrtfConfig) {
        // 如果延迟缓冲区大小改变，需要重新分配
        let max_delay_samples = (config.max_itd_delay * config.sample_rate).ceil() as usize + 1;
        if max_delay_samples > self.left_delay_buffer.len() {
            self.left_delay_buffer.resize(max_delay_samples, 0.0);
            self.right_delay_buffer.resize(max_delay_samples, 0.0);
        }
        self.config = config;
    }
}

// ============================================================================
// 多普勒效应计算
// ============================================================================

/// 多普勒效应计算器
pub struct DopplerCalculator {
    /// 声速 (米/秒)
    speed_of_sound: f32,
    /// 多普勒因子 (0.0 = 无多普勒, 1.0 = 完全多普勒)
    doppler_factor: f32,
}

impl DopplerCalculator {
    /// 创建新的多普勒计算器
    pub fn new(speed_of_sound: f32, doppler_factor: f32) -> Self {
        Self {
            speed_of_sound,
            doppler_factor: doppler_factor.clamp(0.0, 1.0),
        }
    }

    /// 计算多普勒音高偏移
    ///
    /// # Arguments
    /// * `relative_position` - 声源相对于监听器的位置
    /// * `source_velocity` - 声源速度向量
    /// * `listener_velocity` - 监听器速度向量
    ///
    /// # Returns
    /// 音高乘数 (1.0 = 无变化, >1.0 = 音调升高, <1.0 = 音调降低)
    pub fn calculate_pitch_shift(
        &self,
        relative_position: Vec3,
        source_velocity: Vec3,
        listener_velocity: Vec3,
    ) -> f32 {
        if self.doppler_factor <= 0.0 {
            return 1.0;
        }

        let distance = relative_position.length();
        if distance < 0.01 {
            return 1.0;
        }

        let direction = relative_position / distance;
        let c = self.speed_of_sound;

        // 计算朝向监听器的相对速度分量
        let listener_speed = listener_velocity.dot(direction);
        let source_speed = source_velocity.dot(-direction); // 负号因为方向是从源到监听器

        // 多普勒公式: f' = f * (c + v_listener) / (c + v_source)
        // 其中 v_listener 和 v_source 是朝向彼此的速度分量
        let numerator = c + listener_speed * self.doppler_factor;
        let denominator = c + source_speed * self.doppler_factor;

        if denominator.abs() < 0.001 {
            1.0 // 避免除零
        } else {
            (numerator / denominator).clamp(0.5, 2.0) // 限制音高范围
        }
    }

    /// 批量计算多普勒音高偏移
    pub fn calculate_pitch_shift_batch(
        &self,
        relative_positions: &[Vec3],
        source_velocities: &[Vec3],
        listener_velocity: Vec3,
    ) -> Vec<f32> {
        relative_positions
            .iter()
            .zip(source_velocities.iter())
            .map(|(pos, vel)| self.calculate_pitch_shift(*pos, *vel, listener_velocity))
            .collect()
    }

    /// 更新声速
    pub fn set_speed_of_sound(&mut self, speed: f32) {
        self.speed_of_sound = speed.max(1.0);
    }

    /// 更新多普勒因子
    pub fn set_doppler_factor(&mut self, factor: f32) {
        self.doppler_factor = factor.clamp(0.0, 1.0);
    }
}

impl Default for DopplerCalculator {
    fn default() -> Self {
        Self::new(343.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hrtf_filter_creation() {
        let config = HrtfConfig::default();
        let filter = HrtfFilter::new(config);
        assert_eq!(filter.azimuth, 0.0);
        assert_eq!(filter.elevation, 0.0);
    }

    #[test]
    fn test_hrtf_position_update() {
        let config = HrtfConfig::default();
        let mut filter = HrtfFilter::new(config);

        filter.update_source_position(PI / 4.0, 0.0, 5.0);
        assert!((filter.azimuth - PI / 4.0).abs() < 0.001);
        assert_eq!(filter.distance, 5.0);
    }

    #[test]
    fn test_hrtf_itd_calculation() {
        let config = HrtfConfig::default();
        let mut filter = HrtfFilter::new(config);

        // 声源在右侧
        filter.update_source_position(PI / 2.0, 0.0, 1.0);
        let (itd_left, itd_right) = filter.calculate_itd();

        // 右耳应该先听到（ITD为负）
        assert!(itd_right < 0.0);
        assert!(itd_left > 0.0);
    }

    #[test]
    fn test_hrtf_ild_calculation() {
        let config = HrtfConfig::default();
        let mut filter = HrtfFilter::new(config);

        // 声源在右侧
        filter.update_source_position(PI / 2.0, 0.0, 1.0);
        let (ild_left, ild_right) = filter.calculate_ild();

        // 右耳增益应该大于左耳
        assert!(ild_right > ild_left);
    }

    #[test]
    fn test_hrtf_process_mono() {
        let config = HrtfConfig::default();
        let mut filter = HrtfFilter::new(config);

        filter.update_source_position(PI / 4.0, 0.0, 1.0);

        let mono_samples = vec![0.5, 0.3, -0.2, -0.4, 0.1];
        let (left, right) = filter.process_mono(&mono_samples);

        assert_eq!(left.len(), mono_samples.len());
        assert_eq!(right.len(), mono_samples.len());
    }

    #[test]
    fn test_doppler_calculator() {
        let calculator = DopplerCalculator::default();

        // 声源远离监听器
        let relative_pos = Vec3::new(10.0, 0.0, 0.0);
        let source_vel = Vec3::new(10.0, 0.0, 0.0); // 远离
        let listener_vel = Vec3::ZERO;

        let pitch = calculator.calculate_pitch_shift(relative_pos, source_vel, listener_vel);
        assert!(pitch < 1.0); // 音调应该降低
    }

    #[test]
    fn test_doppler_approaching() {
        let calculator = DopplerCalculator::default();

        // 声源接近监听器
        let relative_pos = Vec3::new(10.0, 0.0, 0.0);
        let source_vel = Vec3::new(-10.0, 0.0, 0.0); // 接近
        let listener_vel = Vec3::ZERO;

        let pitch = calculator.calculate_pitch_shift(relative_pos, source_vel, listener_vel);
        assert!(pitch > 1.0); // 音调应该升高
    }
}
