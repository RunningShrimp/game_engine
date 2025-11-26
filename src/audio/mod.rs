use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use bevy_ecs::prelude::*;

/// 音频资源
pub struct AudioAsset {
    /// 音频数据
    pub data: Vec<u8>,
    /// 音频名称
    pub name: String,
}

impl AudioAsset {
    /// 创建新的音频资源
    pub fn new(name: String, data: Vec<u8>) -> Self {
        Self { name, data }
    }
    
    /// 从文件加载音频
    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
        let data = std::fs::read(path)?;
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        Ok(Self::new(name, data))
    }
}

/// 音频源组件
#[derive(Component, Clone, Debug)]
pub struct AudioSource {
    /// 音频名称
    pub name: String,
    /// 音频路径
    pub path: String,
    /// 音量 (0.0 - 1.0)
    pub volume: f32,
    /// 是否循环播放
    pub loop_sound: bool,
    /// 播放状态
    pub is_playing: bool,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            path: "".to_string(),
            volume: 1.0,
            loop_sound: false,
            is_playing: false,
        }
    }
}

/// 音频系统资源(简化版本)
#[derive(Resource)]
pub struct AudioSystem {
    /// 音频资源
    assets: HashMap<String, AudioAsset>,
    /// 活动的音频实体
    playing: Arc<Mutex<HashMap<Entity, bool>>>,
    /// 主音量
    pub master_volume: f32,
}

impl AudioSystem {
    /// 创建新的音频系统
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            playing: Arc::new(Mutex::new(HashMap::new())),
            master_volume: 1.0,
        }
    }
    
    /// 加载音频资源
    pub fn load_asset(&mut self, asset: AudioAsset) {
        self.assets.insert(asset.name.clone(), asset);
    }
    
    /// 从文件加载音频资源
    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let asset = AudioAsset::from_file(path)
            .map_err(|e| format!("Failed to load audio file: {}", e))?;
        self.load_asset(asset);
        Ok(())
    }
    
    /// 播放音频(简化版本,实际播放由外部系统处理)
    pub fn play(&self, entity: Entity, _asset_name: &str, _volume: f32, _looping: bool) -> Result<(), String> {
        if let Ok(mut playing) = self.playing.lock() {
            playing.insert(entity, true);
        }
        Ok(())
    }
    
    /// 停止音频
    pub fn stop(&self, entity: Entity) {
        if let Ok(mut playing) = self.playing.lock() {
            playing.remove(&entity);
        }
    }
    
    /// 暂停音频
    pub fn pause(&self, _entity: Entity) {
        // TODO: 实现暂停逻辑
    }
    
    /// 恢复音频
    pub fn resume(&self, _entity: Entity) {
        // TODO: 实现恢复逻辑
    }
    
    /// 设置音量
    pub fn set_volume(&self, _entity: Entity, _volume: f32) {
        // TODO: 实现音量设置逻辑
    }
    
    /// 设置主音量
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }
    
    /// 检查是否正在播放
    pub fn is_playing(&self, entity: Entity) -> bool {
        if let Ok(playing) = self.playing.lock() {
            return playing.get(&entity).copied().unwrap_or(false);
        }
        false
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// 保留旧的AudioState以保持兼容性
#[derive(Resource, Default)]
pub struct AudioState {
    pub master_volume: f32,
}

/// 音频播放系统
pub fn audio_playback_system(
    audio_system: Res<AudioSystem>,
    mut query: Query<(Entity, &mut AudioSource)>,
) {
    for (entity, mut source) in query.iter_mut() {
        if source.is_playing {
            // 检查是否需要开始播放
            if !audio_system.is_playing(entity) {
                if let Err(e) = audio_system.play(
                    entity,
                    &source.name,
                    source.volume,
                    source.loop_sound,
                ) {
                    eprintln!("Failed to play audio: {}", e);
                    source.is_playing = false;
                }
            }
        } else {
            // 停止播放
            if audio_system.is_playing(entity) {
                audio_system.stop(entity);
            }
        }
    }
}

/// 音频清理系统
pub fn audio_cleanup_system(
    audio_system: Res<AudioSystem>,
    mut removed: RemovedComponents<AudioSource>,
) {
    for entity in removed.read() {
        audio_system.stop(entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_asset() {
        let asset = AudioAsset::new("test.mp3".to_string(), vec![0, 1, 2, 3]);
        assert_eq!(asset.name, "test.mp3");
        assert_eq!(asset.data.len(), 4);
    }
    
    #[test]
    fn test_audio_source() {
        let source = AudioSource {
            name: "test.mp3".to_string(),
            path: "test.mp3".to_string(),
            volume: 0.5,
            loop_sound: true,
            is_playing: false,
        };
        
        assert_eq!(source.name, "test.mp3");
        assert!(source.loop_sound);
        assert_eq!(source.volume, 0.5);
    }
    
    #[test]
    fn test_audio_system() {
        let mut system = AudioSystem::new();
        
        // 加载音频资源
        let asset = AudioAsset::new("test.mp3".to_string(), vec![0, 1, 2, 3]);
        system.load_asset(asset);
        
        // 测试播放
        let entity = Entity::from_raw(0);
        assert!(system.play(entity, "test.mp3", 1.0, false).is_ok());
        assert!(system.is_playing(entity));
        
        // 测试停止
        system.stop(entity);
        assert!(!system.is_playing(entity));
    }
}
