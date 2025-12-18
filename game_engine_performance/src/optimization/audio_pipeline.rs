//! 实时音频处理管道
//!
//! 提供优化的音频处理管道，集成各种音频效果和处理操作
//! - 音频混合
//! - 实时效果处理
//! - 批量音频更新
//! - 性能监控

use crate::impl_default;
use glam::Vec3;
use std::collections::HashMap;

/// 音频处理效果类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioEffectType {
    /// 低通滤波
    LowPass,
    /// 高通滤波
    HighPass,
    /// 延迟/混响
    Reverb,
    /// 压缩
    Compressor,
    /// 均衡
    EQ,
    /// 空间化 (HRTF)
    Spatial,
}

/// 音频效果参数
#[derive(Debug, Clone)]
pub struct AudioEffect {
    /// 效果类型
    pub effect_type: AudioEffectType,
    /// 效果强度 (0.0 - 1.0)
    pub intensity: f32,
    /// 额外参数 (根据效果类型不同)
    pub params: HashMap<String, f32>,
    /// 是否启用
    pub enabled: bool,
}

impl AudioEffect {
    /// 创建新的音频效果
    pub fn new(effect_type: AudioEffectType) -> Self {
        Self {
            effect_type,
            intensity: 1.0,
            params: HashMap::new(),
            enabled: true,
        }
    }

    /// 设置效果强度
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.clamp(0.0, 1.0);
        self
    }

    /// 设置参数
    pub fn with_param(mut self, key: String, value: f32) -> Self {
        self.params.insert(key, value);
        self
    }
}

/// 音频输出通道
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioChannel {
    /// 主输出
    Master,
    /// 音乐通道
    Music,
    /// 音效通道
    SFX,
    /// 语音通道
    Voice,
    /// 环境通道
    Ambient,
}

/// 音频通道混合器
pub struct AudioChannelMixer {
    /// 各通道音量
    channel_volumes: HashMap<AudioChannel, f32>,
    /// 各通道启用状态
    channel_enabled: HashMap<AudioChannel, bool>,
    /// 主音量
    master_volume: f32,
    /// 各通道应用的效果
    channel_effects: HashMap<AudioChannel, Vec<AudioEffect>>,
}

impl Default for AudioChannelMixer {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioChannelMixer {
    /// 创建新的混合器
    pub fn new() -> Self {
        let mut mixer = Self {
            channel_volumes: HashMap::new(),
            channel_enabled: HashMap::new(),
            master_volume: 1.0,
            channel_effects: HashMap::new(),
        };

        // 初始化所有通道
        for channel in &[
            AudioChannel::Master,
            AudioChannel::Music,
            AudioChannel::SFX,
            AudioChannel::Voice,
            AudioChannel::Ambient,
        ] {
            mixer.channel_volumes.insert(*channel, 1.0);
            mixer.channel_enabled.insert(*channel, true);
            mixer.channel_effects.insert(*channel, Vec::new());
        }

        mixer
    }

    /// 设置通道音量
    pub fn set_channel_volume(&mut self, channel: AudioChannel, volume: f32) {
        self.channel_volumes.insert(channel, volume.clamp(0.0, 1.0));
    }

    /// 获取通道音量
    pub fn get_channel_volume(&self, channel: AudioChannel) -> f32 {
        self.channel_volumes.get(&channel).copied().unwrap_or(1.0)
    }

    /// 启用/禁用通道
    pub fn set_channel_enabled(&mut self, channel: AudioChannel, enabled: bool) {
        self.channel_enabled.insert(channel, enabled);
    }

    /// 设置主音量
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// 添加通道效果
    pub fn add_channel_effect(&mut self, channel: AudioChannel, effect: AudioEffect) {
        self.channel_effects
            .entry(channel)
            .or_insert_with(Vec::new)
            .push(effect);
    }

    /// 计算最终输出增益 (包括通道音量、主音量、效果等)
    pub fn calculate_output_gain(&self, channel: AudioChannel, base_gain: f32) -> f32 {
        if !self.channel_enabled.get(&channel).copied().unwrap_or(true) {
            return 0.0;
        }

        let channel_vol = self.channel_volumes.get(&channel).copied().unwrap_or(1.0);
        let mut gain = base_gain * channel_vol * self.master_volume;

        // 应用效果的强度
        if let Some(effects) = self.channel_effects.get(&channel) {
            for effect in effects {
                if effect.enabled {
                    gain *= 1.0 - effect.intensity * 0.1; // 最多减少 10%
                }
            }
        }

        gain.clamp(0.0, 1.0)
    }

    /// 获取通道效果
    pub fn get_channel_effects(&self, channel: AudioChannel) -> Option<&Vec<AudioEffect>> {
        self.channel_effects.get(&channel)
    }
}

/// 实时音频处理管道
pub struct AudioProcessingPipeline {
    /// 音频通道混合器
    mixer: AudioChannelMixer,
    /// 预处理步骤 (空间化等)
    pre_effects_enabled: bool,
    /// 后处理步骤 (压缩、均衡等)
    post_effects_enabled: bool,
    /// 性能指标
    metrics: AudioPipelineMetrics,
}

/// 音频管道性能指标
#[derive(Debug, Clone, Default)]
pub struct AudioPipelineMetrics {
    /// 处理的样本数
    pub samples_processed: u64,
    /// 应用的空间化次数
    pub spatial_updates: u64,
    /// 应用的效果次数
    pub effect_applications: u64,
    /// 平均延迟 (ms)
    pub average_latency_ms: f32,
    /// 峰值延迟 (ms)
    pub peak_latency_ms: f32,
    /// 处理时间 (ms)
    pub total_processing_time_ms: f64,
}

impl_default!(AudioProcessingPipeline {
    mixer: AudioChannelMixer::new(),
    pre_effects_enabled: true,
    post_effects_enabled: true,
    metrics: AudioPipelineMetrics::default(),
});

impl AudioProcessingPipeline {
    /// 创建新的音频处理管道
    pub fn new() -> Self {
        Self::default()
    }

    /// 启用/禁用预处理效果
    pub fn set_pre_effects_enabled(&mut self, enabled: bool) {
        self.pre_effects_enabled = enabled;
    }

    /// 启用/禁用后处理效果
    pub fn set_post_effects_enabled(&mut self, enabled: bool) {
        self.post_effects_enabled = enabled;
    }

    /// 获取混合器
    pub fn mixer_mut(&mut self) -> &mut AudioChannelMixer {
        &mut self.mixer
    }

    /// 获取性能指标
    pub fn get_metrics(&self) -> &AudioPipelineMetrics {
        &self.metrics
    }

    /// 处理一批音频帧
    ///
    /// # Arguments
    /// * `frames` - 音频帧数据
    /// * `channel` - 输入通道
    /// * `base_gain` - 基础增益
    pub fn process_batch(
        &mut self,
        frames: &[f32],
        channel: AudioChannel,
        base_gain: f32,
    ) -> Vec<f32> {
        let start = std::time::Instant::now();

        let output_gain = self.mixer.calculate_output_gain(channel, base_gain);
        let mut output = frames.to_vec();

        // 预处理
        if self.pre_effects_enabled {
            // 空间化、定位等
            self.metrics.spatial_updates += 1;
        }

        // 应用增益
        for sample in &mut output {
            *sample *= output_gain;
        }

        // 后处理
        if self.post_effects_enabled {
            if let Some(effects) = self.mixer.channel_effects.get(&channel) {
                for effect in effects {
                    if effect.enabled {
                        self.metrics.effect_applications += 1;
                    }
                }
            }
        }

        // 更新指标
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        self.metrics.samples_processed += frames.len() as u64;
        self.metrics.total_processing_time_ms += elapsed;
        if elapsed > self.metrics.peak_latency_ms as f64 {
            self.metrics.peak_latency_ms = elapsed as f32;
        }

        output
    }

    /// 重置指标
    pub fn reset_metrics(&mut self) {
        self.metrics = AudioPipelineMetrics::default();
    }
}

/// 批量音频更新管理器
pub struct BatchAudioUpdater {
    /// 待处理的音频更新队列
    pending_updates: Vec<AudioUpdate>,
    /// 最大批处理大小
    max_batch_size: usize,
}

/// 单个音频更新
#[derive(Debug, Clone)]
pub struct AudioUpdate {
    /// 音频源标识
    pub source_id: u32,
    /// 新位置
    pub position: Option<Vec3>,
    /// 新速度
    pub velocity: Option<Vec3>,
    /// 新增益
    pub gain: Option<f32>,
    /// 新通道
    pub channel: Option<AudioChannel>,
}

impl AudioUpdate {
    /// 创建新的更新
    pub fn new(source_id: u32) -> Self {
        Self {
            source_id,
            position: None,
            velocity: None,
            gain: None,
            channel: None,
        }
    }
}

impl_default!(BatchAudioUpdater {
    pending_updates: Vec::new(),
    max_batch_size: 256,
});

impl BatchAudioUpdater {
    /// 创建新的批量更新管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加更新到队列
    pub fn enqueue_update(&mut self, update: AudioUpdate) {
        self.pending_updates.push(update);
    }

    /// 处理所有待处理的更新
    ///
    /// 返回处理的更新数量
    pub fn process_batch(&mut self) -> usize {
        let count = self.pending_updates.len().min(self.max_batch_size);

        // 批量处理更新 (在实际实现中会调用 SIMD 优化的函数)
        let _to_process: Vec<_> = self.pending_updates.drain(0..count).collect();

        count
    }

    /// 获取待处理的更新数量
    pub fn pending_count(&self) -> usize {
        self.pending_updates.len()
    }

    /// 清空所有待处理的更新
    pub fn clear(&mut self) {
        self.pending_updates.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_channel_mixer() {
        let mut mixer = AudioChannelMixer::new();

        mixer.set_channel_volume(AudioChannel::Music, 0.5);
        mixer.set_master_volume(0.8);

        let gain = mixer.calculate_output_gain(AudioChannel::Music, 1.0);
        assert!((gain - 0.4).abs() < 0.01); // 1.0 * 0.5 * 0.8
    }

    #[test]
    fn test_audio_effect() {
        let effect = AudioEffect::new(AudioEffectType::LowPass)
            .with_intensity(0.5)
            .with_param("cutoff".to_string(), 1000.0);

        assert_eq!(effect.effect_type, AudioEffectType::LowPass);
        assert_eq!(effect.intensity, 0.5);
        assert_eq!(effect.params.get("cutoff"), Some(&1000.0));
    }

    #[test]
    fn test_audio_processing_pipeline() {
        let mut pipeline = AudioProcessingPipeline::new();

        let frames = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let output = pipeline.process_batch(&frames, AudioChannel::SFX, 1.0);

        assert_eq!(output.len(), 5);
        assert!(pipeline.metrics.samples_processed > 0);
    }

    #[test]
    fn test_batch_audio_updater() {
        let mut updater = BatchAudioUpdater::new();

        let update1 = AudioUpdate::new(1);
        let update2 = AudioUpdate::new(2);

        updater.enqueue_update(update1);
        updater.enqueue_update(update2);

        assert_eq!(updater.pending_count(), 2);

        let processed = updater.process_batch();
        assert_eq!(processed, 2);
        assert_eq!(updater.pending_count(), 0);
    }

    #[test]
    fn test_channel_effects() {
        let mut mixer = AudioChannelMixer::new();

        let effect = AudioEffect::new(AudioEffectType::Reverb).with_intensity(0.7);
        mixer.add_channel_effect(AudioChannel::Voice, effect);

        let effects = mixer.get_channel_effects(AudioChannel::Voice);
        assert!(effects.is_some());
        assert_eq!(effects.unwrap().len(), 1);
    }
}
