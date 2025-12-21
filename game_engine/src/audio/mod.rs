//  音频系统模块
// 
//  提供音频播放、暂停、停止、音量控制等功能。
//  底层使用 rodio 库实现跨平台音频播放。
// 
//  ## 空间音频
// 
//  支持 3D 空间音频，详见 `spatial` 子模块：
//  - 距离衰减 (Linear, Inverse, Exponential)
//  - 声锥方向性
//  - 多普勒效果
//  - 立体声定位

/// 音频特效系统 - 提供多种音频处理特效（EQ、混响、延迟、压缩）
pub mod effects;
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
