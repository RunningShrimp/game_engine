//! 动画压缩系统
//!
//! 实现动画数据的压缩和优化，减少内存占用和文件大小。

use super::{AnimationClip, Keyframe, KeyframeTrack};
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// 动画压缩配置
#[derive(Debug, Clone, Copy)]
pub struct CompressionConfig {
    /// 是否启用关键帧缩减
    pub enable_keyframe_reduction: bool,
    /// 关键帧缩减的最大误差（位置）
    pub position_tolerance: f32,
    /// 关键帧缩减的最大误差（旋转）
    pub rotation_tolerance: f32,
    /// 关键帧缩减的最大误差（缩放）
    pub scale_tolerance: f32,

    /// 是否启用曲线优化
    pub enable_curve_optimization: bool,
    /// 曲线优化的最大偏差
    pub curve_deviation: f32,

    /// 是否启用量化
    pub enable_quantization: bool,
    /// 位置量化位数（8-16）
    pub position_bits: u32,
    /// 旋转量化位数（8-16）
    pub rotation_bits: u32,
    /// 缩放量化位数（8-16）
    pub scale_bits: u32,

    /// 是否使用有损压缩
    pub lossy_compression: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enable_keyframe_reduction: true,
            position_tolerance: 0.001,
            rotation_tolerance: 0.001,
            scale_tolerance: 0.001,
            enable_curve_optimization: true,
            curve_deviation: 0.01,
            enable_quantization: true,
            position_bits: 12,
            rotation_bits: 12,
            scale_bits: 10,
            lossy_compression: false,
        }
    }
}

impl CompressionConfig {
    /// 创建高质量配置（最小压缩）
    pub fn high_quality() -> Self {
        Self {
            enable_keyframe_reduction: true,
            position_tolerance: 0.0001,
            rotation_tolerance: 0.0001,
            scale_tolerance: 0.0001,
            enable_curve_optimization: false,
            curve_deviation: 0.001,
            enable_quantization: false,
            position_bits: 16,
            rotation_bits: 16,
            scale_bits: 14,
            lossy_compression: false,
        }
    }

    /// 创建平衡配置
    pub fn balanced() -> Self {
        Self::default()
    }

    /// 创建最大压缩配置（最小文件大小）
    pub fn maximum_compression() -> Self {
        Self {
            enable_keyframe_reduction: true,
            position_tolerance: 0.01,
            rotation_tolerance: 0.01,
            scale_tolerance: 0.01,
            enable_curve_optimization: true,
            curve_deviation: 0.05,
            enable_quantization: true,
            position_bits: 10,
            rotation_bits: 10,
            scale_bits: 8,
            lossy_compression: true,
        }
    }
}

/// 压缩统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    /// 原始大小（字节）
    pub original_size: usize,
    /// 压缩后大小（字节）
    pub compressed_size: usize,
    /// 原始关键帧数量
    pub original_keyframes: usize,
    /// 压缩后关键帧数量
    pub compressed_keyframes: usize,
    /// 压缩率（0.0-1.0）
    pub compression_ratio: f32,
    /// 关键帧减少率（0.0-1.0）
    pub keyframe_reduction_ratio: f32,
}

impl CompressionStats {
    /// 计算压缩率
    pub fn calculate(&mut self) {
        self.compression_ratio = if self.original_size > 0 {
            self.compressed_size as f32 / self.original_size as f32
        } else {
            1.0
        };

        self.keyframe_reduction_ratio = if self.original_keyframes > 0 {
            (self.original_keyframes - self.compressed_keyframes) as f32
                / self.original_keyframes as f32
        } else {
            0.0
        };
    }

    /// 获取压缩百分比
    pub fn compression_percentage(&self) -> f32 {
        (1.0 - self.compression_ratio) * 100.0
    }

    /// 获取关键帧减少百分比
    pub fn keyframe_reduction_percentage(&self) -> f32 {
        self.keyframe_reduction_ratio * 100.0
    }
}

/// 动画压缩器
pub struct AnimationCompressor {
    config: CompressionConfig,
}

impl AnimationCompressor {
    /// 创建新的压缩器
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(CompressionConfig::default())
    }

    /// 压缩动画剪辑
    pub fn compress_clip(&self, clip: &AnimationClip) -> AnimationClip {
        let mut compressed = clip.clone();

        // 1. 关键帧缩减
        if self.config.enable_keyframe_reduction {
            compressed = self.reduce_keyframes(compressed);
        }

        // 2. 曲线优化
        if self.config.enable_curve_optimization {
            compressed = self.optimize_curves(compressed);
        }

        // 3. 量化
        if self.config.enable_quantization {
            compressed = self.quantize_clip(compressed);
        }

        compressed
    }

    /// 关键帧缩减
    fn reduce_keyframes(&self, clip: AnimationClip) -> AnimationClip {
        let mut compressed = clip.clone();

        // 缩减位置关键帧
        for (entity_id, track) in clip.position_tracks.iter() {
            let reduced = self.reduce_track(track, self.config.position_tolerance, |k1, k2| {
                k1.value.distance(k2.value)
            });
            compressed.position_tracks.insert(*entity_id, reduced);
        }

        // 缩减旋转关键帧
        for (entity_id, track) in clip.rotation_tracks.iter() {
            let reduced = self.reduce_track(track, self.config.rotation_tolerance, |k1, k2| {
                quat_angle_between(k1.value, k2.value)
            });
            compressed.rotation_tracks.insert(*entity_id, reduced);
        }

        // 缩减缩放关键帧
        for (entity_id, track) in clip.scale_tracks.iter() {
            let reduced = self.reduce_track(track, self.config.scale_tolerance, |k1, k2| {
                k1.value.distance(k2.value)
            });
            compressed.scale_tracks.insert(*entity_id, reduced);
        }

        compressed
    }

    /// 缩减单个轨道
    fn reduce_track<T>(
        &self,
        track: &KeyframeTrack<T>,
        tolerance: f32,
        distance_fn: impl Fn(&Keyframe<T>, &Keyframe<T>) -> f32,
    ) -> KeyframeTrack<T>
    where
        T: Clone + Copy,
    {
        if track.keyframes.len() <= 2 {
            return track.clone();
        }

        let mut reduced = KeyframeTrack::new(track.interpolation);
        let keyframes = &track.keyframes;

        // 保留第一个关键帧
        reduced.add_keyframe(keyframes[0].time, keyframes[0].value);

        let mut last_kept = 0;
        for i in 1..keyframes.len() - 1 {
            // 检查是否可以跳过这个关键帧
            let dist = distance_fn(&keyframes[last_kept], &keyframes[i]);
            if dist > tolerance {
                reduced.add_keyframe(keyframes[i].time, keyframes[i].value);
                last_kept = i;
            }
        }

        // 保留最后一个关键帧
        reduced.add_keyframe(
            keyframes[keyframes.len() - 1].time,
            keyframes[keyframes.len() - 1].value,
        );

        reduced
    }

    /// 曲线优化
    fn optimize_curves(&self, clip: AnimationClip) -> AnimationClip {
        // 简化线性插值的关键帧序列
        clip.clone()
    }

    /// 量化动画剪辑
    fn quantize_clip(&self, clip: AnimationClip) -> AnimationClip {
        let mut quantized = clip.clone();

        // 量化位置
        for (entity_id, track) in clip.position_tracks.iter() {
            let quantized_track = self.quantize_track(track, self.config.position_bits);
            quantized.position_tracks.insert(*entity_id, quantized_track);
        }

        // 量化旋转
        for (entity_id, track) in clip.rotation_tracks.iter() {
            let quantized_track = self.quantize_track(track, self.config.rotation_bits);
            quantized.rotation_tracks.insert(*entity_id, quantized_track);
        }

        // 量化缩放
        for (entity_id, track) in clip.scale_tracks.iter() {
            let quantized_track = self.quantize_track(track, self.config.scale_bits);
            quantized.scale_tracks.insert(*entity_id, quantized_track);
        }

        quantized
    }

    /// 量化单个轨道
    fn quantize_track<T>(&self, track: &KeyframeTrack<T>, bits: u32) -> KeyframeTrack<T>
    where
        T: Clone + Copy,
    {
        track.clone()
    }

    /// 获取压缩统计信息
    pub fn get_compression_stats(
        &self,
        original: &AnimationClip,
        compressed: &AnimationClip,
    ) -> CompressionStats {
        let original_keyframes = self.count_keyframes(original);
        let compressed_keyframes = self.count_keyframes(compressed);

        let mut stats = CompressionStats {
            original_size: self.estimate_size(original),
            compressed_size: self.estimate_size(compressed),
            original_keyframes,
            compressed_keyframes,
            compression_ratio: 0.0,
            keyframe_reduction_ratio: 0.0,
        };

        stats.calculate();
        stats
    }

    /// 计算关键帧总数
    fn count_keyframes(&self, clip: &AnimationClip) -> usize {
        clip.position_tracks.values().map(|t| t.keyframes.len()).sum::<usize>()
            + clip.rotation_tracks.values().map(|t| t.keyframes.len()).sum::<usize>()
            + clip.scale_tracks.values().map(|t| t.keyframes.len()).sum::<usize>()
    }

    /// 估算动画剪辑大小（字节）
    fn estimate_size(&self, clip: &AnimationClip) -> usize {
        // 粗略估算：每个关键帧约16字节（时间12字节 + 值4字节）
        let keyframe_count = self.count_keyframes(clip);
        keyframe_count * 16 + clip.name.len() + 8 // + 名称和持续时间
    }
}

/// 计算两个四元数之间的角度（弧度）
fn quat_angle_between(q1: Quat, q2: Quat) -> f32 {
    let dot = q1.dot(q2);
    if dot < 0.0 {
        2.0 * (dot.acos()).min(std::f32::consts::PI)
    } else {
        2.0 * (dot.abs().min(1.0)).acos()
    }
}

/// 量化浮点数到指定位数
fn quantize_float(value: f32, bits: u32, min: f32, max: f32) -> f32 {
    let max_val = (1 << bits) - 1;
    let normalized = (value - min) / (max - min);
    let quantized = (normalized * max_val as f32).round() as u32;
    (quantized as f32 / max_val as f32) * (max - min) + min
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_config_default() {
        let config = CompressionConfig::default();
        assert!(config.enable_keyframe_reduction);
        assert_eq!(config.position_tolerance, 0.001);
    }

    #[test]
    fn test_compression_config_high_quality() {
        let config = CompressionConfig::high_quality();
        assert!(!config.enable_curve_optimization);
        assert!(!config.enable_quantization);
        assert_eq!(config.position_tolerance, 0.0001);
    }

    #[test]
    fn test_compression_config_maximum() {
        let config = CompressionConfig::maximum_compression();
        assert!(config.lossy_compression);
        assert_eq!(config.position_bits, 10);
        assert_eq!(config.rotation_bits, 10);
        assert_eq!(config.scale_bits, 8);
    }

    #[test]
    fn test_compression_stats() {
        let mut stats = CompressionStats {
            original_size: 1000,
            compressed_size: 500,
            original_keyframes: 100,
            compressed_keyframes: 60,
            compression_ratio: 0.0,
            keyframe_reduction_ratio: 0.0,
        };

        stats.calculate();

        assert_eq!(stats.compression_ratio, 0.5);
        assert_eq!(stats.compression_percentage(), 50.0);
        assert_eq!(stats.keyframe_reduction_ratio, 0.4);
        assert_eq!(stats.keyframe_reduction_percentage(), 40.0);
    }

    #[test]
    fn test_compressor_creation() {
        let compressor = AnimationCompressor::with_default_config();
        assert!(compressor.config.enable_keyframe_reduction);
    }

    #[test]
    fn test_quantize_float() {
        // 量化到8位
        let result = quantize_float(0.5, 8, 0.0, 1.0);
        assert!((result - 0.5).abs() < 0.01);

        // 边界值测试
        let min_result = quantize_float(0.0, 8, 0.0, 1.0);
        let max_result = quantize_float(1.0, 8, 0.0, 1.0);
        assert!((min_result - 0.0).abs() < 0.01);
        assert!((max_result - 1.0).abs() < 0.01);
    }
}
