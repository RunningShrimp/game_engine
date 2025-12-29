//! # Audio System
//!
//! 本模块提供完整的音频系统，支持3D空间音频、音频特效和流式加载。
//!
//! ## 功能特性
//!
//! - **3D空间音频** - 完整的位置音频和听者系统
//! - **音频特效链** - EQ、混响、延迟、压缩等
//! - **流式音频** - 支持大文件的流式播放
//! - **HRTF** - 头部相关传输函数，增强3D定位
//! - **异步处理** - 不阻塞主线程的音频解码
//!
//! ## 主要组件
//!
//! - [`SpatialAudioService`] - 空间音频服务
//! - [`SpatialAudioSource`] - 空间音频源
//! - [`AudioListener`] - 音频听者（摄像机）
//! - [`AudioStream`] - 流式音频加载器
//! - [`EffectChain`] - 音频特效链
//! - [`AudioDomainService`] - 富领域对象服务
//!
//! ## 空间音频特性
//!
//! ### 距离衰减模型
//! - **Linear** - 线性衰减
//! - **Inverse** - 反比衰减（真实物理）
//! - **Exponential** - 指数衰减
//!
//! ### 高级特性
//! - **声锥方向性** - 模拟定向扬声器
//! - **多普勒效应** - 移动物体的音调变化
//! - **立体声定位** - 基于位置的左右声道平衡

pub mod async_processing;
/// 音频特效系统 - 提供多种音频处理特效（EQ、混响、延迟、压缩）
pub mod effects;
/// HRTF (头部相关传输函数) - 用于3D音频定位
pub mod hrtf;
/// 空间音频 - 提供3D空间音频和听者定位支持
pub mod spatial;
/// 音频流 - 提供流式音频加载和播放支持
pub mod streaming;

pub use spatial::{
    AudioListener, DistanceModel, SoundCone, SpatialAudioParams, SpatialAudioService,
    SpatialAudioSource, SpatialAudioState,
};

pub use streaming::{
    AudioBuffer, AudioStream, AudioStreamLoader, StreamConfig, StreamId, StreamState,
    StreamingError,
};

pub use effects::{
    AudioEffect, CompressorConfig, CompressorEffect, DelayConfig, DelayEffect, EffectChain,
    EffectError, EqualizerBand, EqualizerConfig, EqualizerEffect, ReverbConfig, ReverbEffect,
};

// 重新导出新的富领域对象（推荐使用）
pub use crate::domain::audio::{
    AudioSource as RichAudioSource, AudioSourceId, AudioSourceState,
    SpatialAudioSource as RichSpatialAudioSource,
};

pub use crate::domain::services::AudioDomainService;

#[cfg(test)]
mod tests;
