/// 音频配置

use serde::{Deserialize, Serialize};
use super::{ConfigResult, ConfigError};

/// 音频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// 主音量 (0.0 - 1.0)
    pub master_volume: f32,
    
    /// 音乐音量 (0.0 - 1.0)
    pub music_volume: f32,
    
    /// 音效音量 (0.0 - 1.0)
    pub sfx_volume: f32,
    
    /// 语音音量 (0.0 - 1.0)
    pub voice_volume: f32,
    
    /// 采样率
    pub sample_rate: u32,
    
    /// 缓冲区大小
    pub buffer_size: usize,
    
    /// 是否静音
    pub muted: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 1.0,
            voice_volume: 1.0,
            sample_rate: 48000,
            buffer_size: 1024,
            muted: false,
        }
    }
}

impl AudioConfig {
    /// 验证配置
    pub fn validate(&self) -> ConfigResult<()> {
        if !(0.0..=1.0).contains(&self.master_volume) {
            return Err(ConfigError::ValidationError("Invalid master volume".to_string()));
        }
        if !(0.0..=1.0).contains(&self.music_volume) {
            return Err(ConfigError::ValidationError("Invalid music volume".to_string()));
        }
        if !(0.0..=1.0).contains(&self.sfx_volume) {
            return Err(ConfigError::ValidationError("Invalid SFX volume".to_string()));
        }
        if !(0.0..=1.0).contains(&self.voice_volume) {
            return Err(ConfigError::ValidationError("Invalid voice volume".to_string()));
        }
        Ok(())
    }
}
