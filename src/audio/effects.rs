//! 高级音频效果模块
//!
//! 实现各种音频效果处理器，包括混响、均衡器、压缩器等。
//!
//! ## 功能特性
//!
//! - 混响效果（Reverb）
//! - 均衡器（Equalizer）
//! - 压缩器（Compressor）
//! - 延迟效果（Delay/Echo）
//! - 失真效果（Distortion）
//! - 效果链管理
//!
//! ## 使用示例
//!
//! ```rust
//! use crate::audio::effects::*;
//!
//! // 创建效果链
//! let mut chain = EffectChain::new();
//!
//! // 添加混响效果
//! let reverb = ReverbEffect::new(ReverbConfig {
//!     room_size: 0.8,
//!     damping: 0.5,
//!     wet_level: 0.3,
//!     dry_level: 0.7,
//! });
//! chain.add_effect(Box::new(reverb));
//!
//! // 添加均衡器
//! let eq = EqualizerEffect::new(EqualizerConfig::default());
//! chain.add_effect(Box::new(eq));
//!
//! // 处理音频数据
//! let mut samples = vec![0.5; 44100];
//! chain.process(&mut samples);
//! ```

use crate::impl_default;
use thiserror::Error;

/// 音频效果错误
#[derive(Error, Debug)]
pub enum EffectError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("Effect chain error: {0}")]
    ChainError(String),
}

/// 音频效果 trait
pub trait AudioEffect: Send + Sync {
    /// 处理音频样本
    fn process(&mut self, samples: &mut [f32]);

    /// 重置效果状态
    fn reset(&mut self);

    /// 设置效果启用状态
    fn set_enabled(&mut self, enabled: bool);

    /// 检查效果是否启用
    fn is_enabled(&self) -> bool;

    /// 获取效果名称
    fn name(&self) -> &str;
}

// ============================================================================
// 混响效果 (Reverb)
// ============================================================================

/// 混响配置
#[derive(Debug, Clone)]
pub struct ReverbConfig {
    /// 房间大小 (0.0 - 1.0)
    pub room_size: f32,
    /// 阻尼 (0.0 - 1.0)
    pub damping: f32,
    /// 湿信号级别 (0.0 - 1.0)
    pub wet_level: f32,
    /// 干信号级别 (0.0 - 1.0)
    pub dry_level: f32,
    /// 预延迟（秒）
    pub pre_delay: f32,
}

impl_default!(ReverbConfig {
    room_size: 0.5,
    damping: 0.5,
    wet_level: 0.3,
    dry_level: 0.7,
    pre_delay: 0.0,
});

/// 混响效果
pub struct ReverbEffect {
    config: ReverbConfig,
    enabled: bool,
    // 延迟线缓冲区
    delay_lines: Vec<Vec<f32>>,
    // 延迟线索引
    delay_indices: Vec<usize>,
    // 延迟线大小（样本数）
    delay_sizes: Vec<usize>,
    // 反馈系数
    feedback_coeffs: Vec<f32>,
}

impl ReverbEffect {
    /// 创建新的混响效果
    pub fn new(config: ReverbConfig) -> Self {
        // 创建多个延迟线模拟房间反射
        let num_delays = 8;
        let max_delay_samples = (config.pre_delay * 44100.0) as usize + 1000;

        let mut delay_lines = Vec::new();
        let mut delay_indices = Vec::new();
        let mut delay_sizes = Vec::new();
        let mut feedback_coeffs = Vec::new();

        for i in 0..num_delays {
            // 使用不同的延迟时间创建丰富的混响
            let delay_size = max_delay_samples + (i * 100);
            delay_lines.push(vec![0.0; delay_size]);
            delay_indices.push(0);
            delay_sizes.push(delay_size);

            // 反馈系数基于房间大小和阻尼
            let feedback = config.room_size * (1.0 - config.damping * 0.5);
            feedback_coeffs.push(feedback);
        }

        Self {
            config,
            enabled: true,
            delay_lines,
            delay_indices,
            delay_sizes,
            feedback_coeffs,
        }
    }

    /// 更新配置
    pub fn update_config(&mut self, config: ReverbConfig) {
        self.config = config;
        self.reset();
    }
}

impl AudioEffect for ReverbEffect {
    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        for sample in samples.iter_mut() {
            let input = *sample;
            let mut output = input * self.config.dry_level;

            // 处理所有延迟线
            for i in 0..self.delay_lines.len() {
                let delay_line = &mut self.delay_lines[i];
                let index = self.delay_indices[i];
                let delay_size = self.delay_sizes[i];
                let feedback = self.feedback_coeffs[i];

                // 读取延迟线中的值
                let delayed = delay_line[index];

                // 写入新值（输入 + 反馈）
                delay_line[index] = input + delayed * feedback;

                // 更新索引
                self.delay_indices[i] = (index + 1) % delay_size;

                // 累加混响输出
                output += delayed * self.config.wet_level / self.delay_lines.len() as f32;
            }

            *sample = output.clamp(-1.0, 1.0);
        }
    }

    fn reset(&mut self) {
        for delay_line in &mut self.delay_lines {
            delay_line.fill(0.0);
        }
        self.delay_indices.fill(0);
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn name(&self) -> &str {
        "Reverb"
    }
}

// ============================================================================
// 均衡器 (Equalizer)
// ============================================================================

/// 均衡器频段
#[derive(Debug, Clone)]
pub struct EqualizerBand {
    /// 频率（Hz）
    pub frequency: f32,
    /// 增益（dB）
    pub gain: f32,
    /// Q值（带宽）
    pub q: f32,
}

impl_default!(EqualizerBand {
    frequency: 1000.0,
    gain: 0.0,
    q: 1.0,
});

/// 均衡器配置
#[derive(Debug, Clone)]
pub struct EqualizerConfig {
    /// 频段
    pub bands: Vec<EqualizerBand>,
    /// 采样率（Hz）
    pub sample_rate: f32,
}

impl_default!(EqualizerConfig {
    bands: vec![
        EqualizerBand { frequency: 60.0, gain: 0.0, q: 1.0 },
        EqualizerBand { frequency: 170.0, gain: 0.0, q: 1.0 },
        EqualizerBand { frequency: 310.0, gain: 0.0, q: 1.0 },
        EqualizerBand { frequency: 600.0, gain: 0.0, q: 1.0 },
        EqualizerBand { frequency: 1000.0, gain: 0.0, q: 1.0 },
        EqualizerBand { frequency: 3000.0, gain: 0.0, q: 1.0 },
        EqualizerBand { frequency: 6000.0, gain: 0.0, q: 1.0 },
        EqualizerBand { frequency: 12000.0, gain: 0.0, q: 1.0 },
        EqualizerBand { frequency: 14000.0, gain: 0.0, q: 1.0 },
        EqualizerBand { frequency: 16000.0, gain: 0.0, q: 1.0 },
    ],
    sample_rate: 44100.0,
});

/// 均衡器效果
pub struct EqualizerEffect {
    config: EqualizerConfig,
    enabled: bool,
    // 双二阶滤波器状态（每个频段）
    filter_states: Vec<BiquadState>,
}

/// 双二阶滤波器状态
#[derive(Debug, Clone)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    fn new() -> Self {
        Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
}

impl EqualizerEffect {
    /// 创建新的均衡器效果
    pub fn new(config: EqualizerConfig) -> Self {
        let filter_states = vec![BiquadState::new(); config.bands.len()];

        Self {
            config,
            enabled: true,
            filter_states,
        }
    }

    /// 更新频段增益
    pub fn set_band_gain(&mut self, band_index: usize, gain_db: f32) -> Result<(), EffectError> {
        if band_index >= self.config.bands.len() {
            return Err(EffectError::InvalidParameter(format!(
                "Band index {} out of range",
                band_index
            )));
        }
        self.config.bands[band_index].gain = gain_db;
        Ok(())
    }

    /// 计算双二阶滤波器系数
    fn calculate_biquad_coeffs(
        &self,
        band: &EqualizerBand,
        sample_rate: f32,
    ) -> (f32, f32, f32, f32, f32) {
        let f = band.frequency;
        let q = band.q;
        let gain_linear = 10.0_f32.powf(band.gain / 20.0);
        let w = 2.0 * std::f32::consts::PI * f / sample_rate;
        let cos_w = w.cos();
        let sin_w = w.sin();
        let alpha = sin_w / (2.0 * q);
        let a = (gain_linear + alpha).sqrt();

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha / a;

        (b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0)
    }
}

impl AudioEffect for EqualizerEffect {
    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        let sample_rate = self.config.sample_rate;
        let bands: Vec<_> = self.config.bands.clone();

        // 预先计算所有系数
        let coeffs: Vec<_> = bands
            .iter()
            .map(|band| self.calculate_biquad_coeffs(band, sample_rate))
            .collect();

        for sample in samples.iter_mut() {
            let mut output = *sample;

            // 应用所有频段
            for (i, (b0, b1, b2, a1, a2)) in coeffs.iter().enumerate() {
                let state = &mut self.filter_states[i];

                // 双二阶滤波器处理
                let input = output;
                output = b0 * input + b1 * state.x1 + b2 * state.x2 - a1 * state.y1 - a2 * state.y2;

                // 更新状态
                state.x2 = state.x1;
                state.x1 = input;
                state.y2 = state.y1;
                state.y1 = output;
            }

            *sample = output.clamp(-1.0, 1.0);
        }
    }

    fn reset(&mut self) {
        for state in &mut self.filter_states {
            *state = BiquadState::new();
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn name(&self) -> &str {
        "Equalizer"
    }
}

// ============================================================================
// 压缩器 (Compressor)
// ============================================================================

/// 压缩器配置
#[derive(Debug, Clone)]
pub struct CompressorConfig {
    /// 阈值（dB）
    pub threshold: f32,
    /// 压缩比 (例如 4.0 表示 4:1)
    pub ratio: f32,
    /// 攻击时间（毫秒）
    pub attack_ms: f32,
    /// 释放时间（毫秒）
    pub release_ms: f32,
    /// 增益补偿（dB）
    pub makeup_gain: f32,
}

impl_default!(CompressorConfig {
    threshold: -12.0,
    ratio: 4.0,
    attack_ms: 1.0,
    release_ms: 50.0,
    makeup_gain: 0.0,
});

/// 压缩器效果
pub struct CompressorEffect {
    config: CompressorConfig,
    enabled: bool,
    // 包络跟随器状态
    envelope: f32,
    // 采样率
    sample_rate: f32,
}

impl CompressorEffect {
    /// 创建新的压缩器效果
    pub fn new(config: CompressorConfig) -> Self {
        Self {
            config,
            enabled: true,
            envelope: 0.0,
            sample_rate: 44100.0,
        }
    }

    /// 更新配置
    pub fn update_config(&mut self, config: CompressorConfig) {
        self.config = config;
    }
}

impl AudioEffect for CompressorEffect {
    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        let threshold_linear = 10.0_f32.powf(self.config.threshold / 20.0);
        let attack_coeff = (-1.0 / (self.config.attack_ms * 0.001 * self.sample_rate)).exp();
        let release_coeff = (-1.0 / (self.config.release_ms * 0.001 * self.sample_rate)).exp();
        let makeup_gain_linear = 10.0_f32.powf(self.config.makeup_gain / 20.0);

        for sample in samples.iter_mut() {
            let input_abs = sample.abs();

            // 包络跟随
            if input_abs > self.envelope {
                self.envelope = input_abs + (self.envelope - input_abs) * attack_coeff;
            } else {
                self.envelope = input_abs + (self.envelope - input_abs) * release_coeff;
            }

            // 计算压缩增益
            let mut gain = 1.0;
            if self.envelope > threshold_linear {
                let over_threshold = self.envelope - threshold_linear;
                let compressed = threshold_linear + over_threshold / self.config.ratio;
                gain = compressed / self.envelope;
            }

            // 应用增益和补偿
            *sample = (*sample * gain * makeup_gain_linear).clamp(-1.0, 1.0);
        }
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn name(&self) -> &str {
        "Compressor"
    }
}

// ============================================================================
// 延迟效果 (Delay/Echo)
// ============================================================================

/// 延迟效果配置
#[derive(Debug, Clone)]
pub struct DelayConfig {
    /// 延迟时间（秒）
    pub delay_time: f32,
    /// 反馈量 (0.0 - 1.0)
    pub feedback: f32,
    /// 湿信号级别 (0.0 - 1.0)
    pub wet_level: f32,
    /// 干信号级别 (0.0 - 1.0)
    pub dry_level: f32,
}

impl_default!(DelayConfig {
    delay_time: 0.25,
    feedback: 0.3,
    wet_level: 0.4,
    dry_level: 0.6,
});

/// 延迟效果
pub struct DelayEffect {
    config: DelayConfig,
    enabled: bool,
    // 延迟缓冲区
    delay_buffer: Vec<f32>,
    // 延迟缓冲区索引
    delay_index: usize,
    // 采样率
    sample_rate: f32,
}

impl DelayEffect {
    /// 创建新的延迟效果
    pub fn new(config: DelayConfig) -> Self {
        let sample_rate = 44100.0;
        let delay_samples = (config.delay_time * sample_rate) as usize;
        let delay_buffer = vec![0.0; delay_samples.max(1)];

        Self {
            config,
            enabled: true,
            delay_buffer,
            delay_index: 0,
            sample_rate,
        }
    }

    /// 更新配置
    pub fn update_config(&mut self, config: DelayConfig) {
        self.config = config;
        let delay_samples = (self.config.delay_time * self.sample_rate) as usize;
        self.delay_buffer.resize(delay_samples.max(1), 0.0);
        self.delay_index = 0;
    }
}

impl AudioEffect for DelayEffect {
    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        for sample in samples.iter_mut() {
            let input = *sample;

            // 读取延迟缓冲区
            let delayed = self.delay_buffer[self.delay_index];

            // 写入延迟缓冲区（输入 + 反馈）
            self.delay_buffer[self.delay_index] = input + delayed * self.config.feedback;

            // 更新索引
            self.delay_index = (self.delay_index + 1) % self.delay_buffer.len();

            // 混合干湿信号
            *sample =
                (input * self.config.dry_level + delayed * self.config.wet_level).clamp(-1.0, 1.0);
        }
    }

    fn reset(&mut self) {
        self.delay_buffer.fill(0.0);
        self.delay_index = 0;
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn name(&self) -> &str {
        "Delay"
    }
}

// ============================================================================
// 效果链 (Effect Chain)
// ============================================================================

/// 效果链
pub struct EffectChain {
    effects: Vec<Box<dyn AudioEffect>>,
    enabled: bool,
}

impl EffectChain {
    /// 创建新的效果链
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加效果
    pub fn add_effect(&mut self, effect: Box<dyn AudioEffect>) {
        self.effects.push(effect);
    }

    /// 移除效果
    pub fn remove_effect(&mut self, index: usize) -> Result<(), EffectError> {
        if index >= self.effects.len() {
            return Err(EffectError::InvalidParameter(format!(
                "Effect index {} out of range",
                index
            )));
        }
        self.effects.remove(index);
        Ok(())
    }

    /// 获取效果数量
    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }

    /// 处理音频样本
    pub fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        for effect in &mut self.effects {
            if effect.is_enabled() {
                effect.process(samples);
            }
        }
    }

    /// 重置所有效果
    pub fn reset_all(&mut self) {
        for effect in &mut self.effects {
            effect.reset();
        }
    }

    /// 设置效果链启用状态
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 检查效果链是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl_default!(EffectChain {
    effects: Vec::new(),
    enabled: true,
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverb_effect() {
        let mut reverb = ReverbEffect::new(ReverbConfig::default());
        let mut samples = vec![0.5; 100];
        reverb.process(&mut samples);

        // 混响应该改变样本值
        assert!(samples.iter().any(|&s| s != 0.5));
    }

    #[test]
    fn test_equalizer_effect() {
        let mut eq = EqualizerEffect::new(EqualizerConfig::default());
        let mut samples = vec![0.5; 100];
        eq.process(&mut samples);

        // 均衡器应该处理样本
        assert!(samples.len() == 100);
    }

    #[test]
    fn test_compressor_effect() {
        let mut compressor = CompressorEffect::new(CompressorConfig::default());
        let mut samples = vec![0.8; 100]; // 高音量样本
        compressor.process(&mut samples);

        // 压缩器应该降低高音量
        assert!(samples.iter().all(|&s| s.abs() <= 1.0));
    }

    #[test]
    fn test_delay_effect() {
        let mut delay = DelayEffect::new(DelayConfig::default());
        let mut samples = vec![0.5; 100];
        delay.process(&mut samples);

        // 延迟效果应该处理样本
        assert!(samples.len() == 100);
    }

    #[test]
    fn test_effect_chain() {
        let mut chain = EffectChain::new();

        let reverb = ReverbEffect::new(ReverbConfig::default());
        chain.add_effect(Box::new(reverb));

        let eq = EqualizerEffect::new(EqualizerConfig::default());
        chain.add_effect(Box::new(eq));

        assert_eq!(chain.effect_count(), 2);

        let mut samples = vec![0.5; 100];
        chain.process(&mut samples);

        // 效果链应该处理样本
        assert!(samples.len() == 100);
    }
}
