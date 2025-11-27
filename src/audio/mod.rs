//! 音频系统模块
//! 
//! 提供音频播放、暂停、停止、音量控制等功能。
//! 底层使用 rodio 库实现跨平台音频播放。
//! 
//! ## 架构设计（贫血模型）
//! 
//! 遵循 DDD 贫血模型设计原则，将数据与行为分离：
//! - `AudioState` (Resource): 纯数据结构，存储音频系统状态
//! - `AudioService`: 业务逻辑封装，提供音频操作方法
//! - `audio_*_system`: ECS 系统，负责调度编排
//! 
//! 由于 rodio 的 OutputStream 不是 Send 安全的，本模块使用通道模式，
//! 在专用后台线程中运行音频播放。

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::collections::{HashMap, HashSet};
use std::io::BufReader;
use bevy_ecs::prelude::*;
use crossbeam_channel::{Sender, Receiver, unbounded};
use rodio::{OutputStream, OutputStreamBuilder, Sink, Decoder, Source};

/// 音频资源 - 存储音频文件数据
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

/// 音频源组件 - 附加到需要播放声音的实体（纯数据结构）
#[derive(Component, Clone, Debug)]
pub struct AudioSource {
    /// 音频标识名称
    pub name: String,
    /// 音频文件路径
    pub path: String,
    /// 音量 (0.0 - 1.0)
    pub volume: f32,
    /// 是否循环播放
    pub loop_sound: bool,
    /// 播放状态
    pub is_playing: bool,
    /// 是否暂停
    pub is_paused: bool,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            path: String::new(),
            volume: 1.0,
            loop_sound: false,
            is_playing: false,
            is_paused: false,
        }
    }
}

/// 内部音频命令
#[derive(Clone)]
enum AudioCommand {
    PlayFile { entity_id: u64, path: String, volume: f32, looped: bool },
    Stop { entity_id: u64 },
    Pause { entity_id: u64 },
    Resume { entity_id: u64 },
    SetVolume { entity_id: u64, volume: f32 },
    SetMasterVolume { volume: f32 },
    Cleanup,
    QueryStatus { entity_id: u64 },
}

/// 音频状态响应
#[derive(Clone, Copy, Default)]
struct AudioStatus {
    is_playing: bool,
    is_paused: bool,
}

// ============================================================================
// 贫血模型：AudioState（纯数据 Resource）
// ============================================================================

/// 音频系统状态 - 纯数据结构 (Resource)
/// 
/// 遵循贫血模型设计，仅包含状态数据，不包含业务逻辑。
/// 业务逻辑由 `AudioService` 提供。
#[derive(Resource)]
pub struct AudioState {
    /// 命令发送通道
    pub(crate) command_tx: Sender<AudioCommand>,
    /// 正在播放的实体集合 (用于快速查询)
    pub(crate) playing_entities: Arc<Mutex<HashSet<u64>>>,
    /// 暂停的实体集合
    pub(crate) paused_entities: Arc<Mutex<HashSet<u64>>>,
    /// 系统是否可用
    pub(crate) available: Arc<AtomicBool>,
    /// 主音量 (0.0 - 2.0)
    pub master_volume: f32,
}

impl Default for AudioState {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioState {
    /// 创建新的音频状态并启动后台线程
    pub fn new() -> Self {
        let (command_tx, command_rx) = unbounded::<AudioCommand>();
        let playing_entities = Arc::new(Mutex::new(HashSet::new()));
        let paused_entities = Arc::new(Mutex::new(HashSet::new()));
        let available = Arc::new(AtomicBool::new(false));
        
        let playing_clone = playing_entities.clone();
        let paused_clone = paused_entities.clone();
        let available_clone = available.clone();
        
        // 在后台线程运行音频系统
        std::thread::spawn(move || {
            AudioBackendRunner::run(command_rx, playing_clone, paused_clone, available_clone);
        });
        
        Self {
            command_tx,
            playing_entities,
            paused_entities,
            available,
            master_volume: 1.0,
        }
    }
}

// ============================================================================
// 贫血模型：AudioService（业务逻辑）
// ============================================================================

/// 音频服务 - 封装音频业务逻辑
/// 
/// 遵循贫血模型设计原则：
/// - `AudioState` (Resource): 纯数据结构
/// - `AudioService` (Service): 封装业务逻辑
/// - `audio_*_system` (System): 调度编排
/// 
/// # 示例
/// 
/// ```ignore
/// fn play_sound(mut state: ResMut<AudioState>, entity: Entity) {
///     AudioService::play_file(&mut state, entity, "sound.mp3", 1.0, false);
/// }
/// ```
pub struct AudioService;

impl AudioService {
    /// 检查音频系统是否可用
    pub fn is_available(state: &AudioState) -> bool {
        state.available.load(Ordering::SeqCst)
    }
    
    /// 从文件路径播放音频
    pub fn play_file(
        state: &AudioState, 
        entity: Entity, 
        path: &str, 
        volume: f32, 
        looped: bool
    ) -> Result<(), String> {
        let _ = state.command_tx.send(AudioCommand::PlayFile {
            entity_id: entity.to_bits(),
            path: path.to_string(),
            volume,
            looped,
        });
        Ok(())
    }
    
    /// 停止指定实体的音频播放
    pub fn stop(state: &AudioState, entity: Entity) {
        let _ = state.command_tx.send(AudioCommand::Stop {
            entity_id: entity.to_bits(),
        });
    }
    
    /// 暂停指定实体的音频
    pub fn pause(state: &AudioState, entity: Entity) {
        let _ = state.command_tx.send(AudioCommand::Pause {
            entity_id: entity.to_bits(),
        });
    }
    
    /// 恢复指定实体的音频播放
    pub fn resume(state: &AudioState, entity: Entity) {
        let _ = state.command_tx.send(AudioCommand::Resume {
            entity_id: entity.to_bits(),
        });
    }
    
    /// 设置指定实体的音频音量
    pub fn set_volume(state: &AudioState, entity: Entity, volume: f32) {
        let _ = state.command_tx.send(AudioCommand::SetVolume {
            entity_id: entity.to_bits(),
            volume,
        });
    }
    
    /// 设置主音量 (影响所有音频)
    pub fn set_master_volume(state: &mut AudioState, volume: f32) {
        state.master_volume = volume.clamp(0.0, 2.0);
        let _ = state.command_tx.send(AudioCommand::SetMasterVolume {
            volume: state.master_volume,
        });
    }
    
    /// 检查指定实体是否正在播放音频
    pub fn is_playing(state: &AudioState, entity: Entity) -> bool {
        let entity_id = entity.to_bits();
        let playing = state.playing_entities.lock().map(|p| p.contains(&entity_id)).unwrap_or(false);
        let paused = state.paused_entities.lock().map(|p| p.contains(&entity_id)).unwrap_or(false);
        playing && !paused
    }
    
    /// 检查指定实体的音频是否暂停
    pub fn is_paused(state: &AudioState, entity: Entity) -> bool {
        state.paused_entities.lock().map(|p| p.contains(&entity.to_bits())).unwrap_or(false)
    }
    
    /// 清理已完成播放的音频实例
    pub fn cleanup(state: &AudioState) {
        let _ = state.command_tx.send(AudioCommand::Cleanup);
    }
    
    /// 停止所有音频
    pub fn stop_all(state: &AudioState) {
        let entities: Vec<u64> = state.playing_entities.lock()
            .map(|p| p.iter().copied().collect())
            .unwrap_or_default();
        for entity_id in entities {
            let _ = state.command_tx.send(AudioCommand::Stop { entity_id });
        }
    }
    
    /// 获取当前活动的音频数量
    pub fn active_count(state: &AudioState) -> usize {
        state.playing_entities.lock().map(|p| p.len()).unwrap_or(0)
    }
}

// ============================================================================
// 后台音频运行器
// ============================================================================

/// 后台音频运行器 - 在专用线程中处理音频命令
struct AudioBackendRunner;

impl AudioBackendRunner {
    fn run(
        rx: Receiver<AudioCommand>,
        playing: Arc<Mutex<HashSet<u64>>>,
        paused: Arc<Mutex<HashSet<u64>>>,
        available: Arc<AtomicBool>,
    ) {
        let stream = match OutputStreamBuilder::open_default_stream() {
            Ok(s) => {
                available.store(true, Ordering::SeqCst);
                s
            }
            Err(e) => {
                eprintln!("Warning: Failed to initialize audio output: {}", e);
                return;
            }
        };
        
        let mut sinks: HashMap<u64, Sink> = HashMap::new();
        let mut volumes: HashMap<u64, f32> = HashMap::new();
        let mut master_volume = 1.0f32;
        
        loop {
            match rx.recv() {
                Ok(AudioCommand::PlayFile { entity_id, path, volume, looped }) => {
                    // 停止已存在的音频
                    if let Some(old_sink) = sinks.remove(&entity_id) {
                        old_sink.stop();
                    }
                    
                    if let Ok(file) = std::fs::File::open(&path) {
                        if let Ok(source) = Decoder::new(BufReader::new(file)) {
                            let sink = Sink::connect_new(stream.mixer());
                            sink.set_volume(volume * master_volume);
                            
                            if looped {
                                sink.append(source.repeat_infinite());
                            } else {
                                sink.append(source);
                            }
                            
                            sinks.insert(entity_id, sink);
                            volumes.insert(entity_id, volume);
                            if let Ok(mut p) = playing.lock() { p.insert(entity_id); }
                            if let Ok(mut p) = paused.lock() { p.remove(&entity_id); }
                        } else {
                            eprintln!("Failed to decode audio: {}", path);
                        }
                    } else {
                        eprintln!("Failed to open audio file: {}", path);
                    }
                }
                Ok(AudioCommand::Stop { entity_id }) => {
                    if let Some(sink) = sinks.remove(&entity_id) {
                        sink.stop();
                    }
                    volumes.remove(&entity_id);
                    if let Ok(mut p) = playing.lock() { p.remove(&entity_id); }
                    if let Ok(mut p) = paused.lock() { p.remove(&entity_id); }
                }
                Ok(AudioCommand::Pause { entity_id }) => {
                    if let Some(sink) = sinks.get(&entity_id) {
                        sink.pause();
                        if let Ok(mut p) = paused.lock() { p.insert(entity_id); }
                    }
                }
                Ok(AudioCommand::Resume { entity_id }) => {
                    if let Some(sink) = sinks.get(&entity_id) {
                        sink.play();
                        if let Ok(mut p) = paused.lock() { p.remove(&entity_id); }
                    }
                }
                Ok(AudioCommand::SetVolume { entity_id, volume }) => {
                    if let Some(sink) = sinks.get(&entity_id) {
                        sink.set_volume(volume * master_volume);
                        volumes.insert(entity_id, volume);
                    }
                }
                Ok(AudioCommand::SetMasterVolume { volume }) => {
                    master_volume = volume;
                    for (entity_id, sink) in &sinks {
                        if let Some(&vol) = volumes.get(entity_id) {
                            sink.set_volume(vol * master_volume);
                        }
                    }
                }
                Ok(AudioCommand::Cleanup) => {
                    let finished: Vec<u64> = sinks.iter()
                        .filter(|(_, sink)| sink.empty() && !sink.is_paused())
                        .map(|(&id, _)| id)
                        .collect();
                    
                    for id in finished {
                        sinks.remove(&id);
                        volumes.remove(&id);
                        if let Ok(mut p) = playing.lock() { p.remove(&id); }
                        if let Ok(mut p) = paused.lock() { p.remove(&id); }
                    }
                }
                Ok(AudioCommand::QueryStatus { .. }) => {
                    // 状态查询通过共享状态处理
                }
                Err(_) => {
                    // 通道关闭，退出线程
                    break;
                }
            }
        }
    }
}

// ============================================================================
// 兼容层：AudioSystem（保持向后兼容）
// ============================================================================

/// 音频系统资源 - 管理所有音频播放 (线程安全)
/// 
/// **注意**: 此类型为兼容层，推荐使用 `AudioState` + `AudioService` 模式。
#[derive(Resource)]
pub struct AudioSystem {
    /// 命令发送通道
    command_tx: Sender<AudioCommand>,
    /// 正在播放的实体集合 (用于快速查询)
    playing_entities: Arc<Mutex<HashSet<u64>>>,
    /// 暂停的实体集合
    paused_entities: Arc<Mutex<HashSet<u64>>>,
    /// 系统是否可用
    available: Arc<AtomicBool>,
    /// 主音量
    pub master_volume: f32,
}

impl AudioSystem {
    /// 创建新的音频系统
    /// 
    /// **注意**: 推荐使用 `AudioState::new()` + `AudioService` 模式。
    pub fn new() -> Self {
        let state = AudioState::new();
        Self {
            command_tx: state.command_tx,
            playing_entities: state.playing_entities,
            paused_entities: state.paused_entities,
            available: state.available,
            master_volume: state.master_volume,
        }
    }
    
    /// 从 AudioState 创建 AudioSystem（兼容层）
    pub fn from_state(state: &AudioState) -> Self {
        Self {
            command_tx: state.command_tx.clone(),
            playing_entities: state.playing_entities.clone(),
            paused_entities: state.paused_entities.clone(),
            available: state.available.clone(),
            master_volume: state.master_volume,
        }
    }
    
    /// 转换为 AudioState
    pub fn to_state(&self) -> AudioState {
        AudioState {
            command_tx: self.command_tx.clone(),
            playing_entities: self.playing_entities.clone(),
            paused_entities: self.paused_entities.clone(),
            available: self.available.clone(),
            master_volume: self.master_volume,
        }
    }
    
    /// 检查音频系统是否可用
    #[deprecated(since = "0.2.0", note = "请使用 AudioService::is_available(&state) 代替")]
    pub fn is_available(&self) -> bool {
        self.available.load(Ordering::SeqCst)
    }
    
    /// 从文件路径播放音频
    #[deprecated(since = "0.2.0", note = "请使用 AudioService::play_file(&state, ...) 代替")]
    pub fn play_file(&self, entity: Entity, path: &str, volume: f32, looped: bool) -> Result<(), String> {
        let _ = self.command_tx.send(AudioCommand::PlayFile {
            entity_id: entity.to_bits(),
            path: path.to_string(),
            volume,
            looped,
        });
        Ok(())
    }
    
    /// 停止指定实体的音频播放
    #[deprecated(since = "0.2.0", note = "请使用 AudioService::stop(&state, entity) 代替")]
    pub fn stop(&self, entity: Entity) {
        let _ = self.command_tx.send(AudioCommand::Stop {
            entity_id: entity.to_bits(),
        });
    }
    
    /// 暂停指定实体的音频
    #[deprecated(since = "0.2.0", note = "请使用 AudioService::pause(&state, entity) 代替")]
    pub fn pause(&self, entity: Entity) {
        let _ = self.command_tx.send(AudioCommand::Pause {
            entity_id: entity.to_bits(),
        });
    }
    
    /// 恢复指定实体的音频播放
    #[deprecated(since = "0.2.0", note = "请使用 AudioService::resume(&state, entity) 代替")]
    pub fn resume(&self, entity: Entity) {
        let _ = self.command_tx.send(AudioCommand::Resume {
            entity_id: entity.to_bits(),
        });
    }
    
    /// 设置指定实体的音频音量
    #[deprecated(since = "0.2.0", note = "请使用 AudioService::set_volume(&state, ...) 代替")]
    pub fn set_volume(&self, entity: Entity, volume: f32) {
        let _ = self.command_tx.send(AudioCommand::SetVolume {
            entity_id: entity.to_bits(),
            volume,
        });
    }
    
    /// 设置主音量 (影响所有音频)
    #[deprecated(since = "0.2.0", note = "请使用 AudioService::set_master_volume(&mut state, volume) 代替")]
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 2.0);
        let _ = self.command_tx.send(AudioCommand::SetMasterVolume {
            volume: self.master_volume,
        });
    }
    
    /// 检查指定实体是否正在播放音频
    #[deprecated(since = "0.2.0", note = "请使用 AudioService::is_playing(&state, entity) 代替")]
    pub fn is_playing(&self, entity: Entity) -> bool {
        let entity_id = entity.to_bits();
        let playing = self.playing_entities.lock().map(|p| p.contains(&entity_id)).unwrap_or(false);
        let paused = self.paused_entities.lock().map(|p| p.contains(&entity_id)).unwrap_or(false);
        playing && !paused
    }
    
    /// 检查指定实体的音频是否暂停
    #[deprecated(since = "0.2.0", note = "请使用 AudioService::is_paused(&state, entity) 代替")]
    pub fn is_paused(&self, entity: Entity) -> bool {
        self.paused_entities.lock().map(|p| p.contains(&entity.to_bits())).unwrap_or(false)
    }
    
    /// 清理已完成播放的音频实例
    #[deprecated(since = "0.2.0", note = "请使用 AudioService::cleanup(&state) 代替")]
    pub fn cleanup(&self) {
        let _ = self.command_tx.send(AudioCommand::Cleanup);
    }
    
    /// 停止所有音频
    #[deprecated(since = "0.2.0", note = "请使用 AudioService::stop_all(&state) 代替")]
    pub fn stop_all(&self) {
        let entities: Vec<u64> = self.playing_entities.lock()
            .map(|p| p.iter().copied().collect())
            .unwrap_or_default();
        for entity_id in entities {
            let _ = self.command_tx.send(AudioCommand::Stop { entity_id });
        }
    }
    
    /// 获取当前活动的音频数量
    #[deprecated(since = "0.2.0", note = "请使用 AudioService::active_count(&state) 代替")]
    pub fn active_count(&self) -> usize {
        self.playing_entities.lock().map(|p| p.len()).unwrap_or(0)
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ECS 系统函数（使用新的 Service 模式）
// ============================================================================

/// 音频播放系统 - 根据 AudioSource 组件状态自动管理音频播放
/// 
/// 使用新的 `AudioState` + `AudioService` 模式
pub fn audio_playback_system_v2(
    audio_state: Res<AudioState>,
    mut query: Query<(Entity, &mut AudioSource)>,
) {
    for (entity, mut source) in query.iter_mut() {
        let currently_playing = AudioService::is_playing(&audio_state, entity);
        let currently_paused = AudioService::is_paused(&audio_state, entity);
        
        if source.is_playing && !source.is_paused {
            if !currently_playing && !currently_paused {
                if !source.path.is_empty() {
                    if let Err(e) = AudioService::play_file(
                        &audio_state,
                        entity,
                        &source.path,
                        source.volume,
                        source.loop_sound,
                    ) {
                        eprintln!("Failed to play audio '{}': {}", source.path, e);
                        source.is_playing = false;
                    }
                }
            } else if currently_paused {
                AudioService::resume(&audio_state, entity);
            }
        } else if source.is_paused && currently_playing {
            AudioService::pause(&audio_state, entity);
        } else if !source.is_playing {
            if currently_playing || currently_paused {
                AudioService::stop(&audio_state, entity);
            }
        }
    }
}

/// 音频清理系统 - 在实体移除时停止其音频（v2 版本）
pub fn audio_cleanup_system_v2(
    audio_state: Res<AudioState>,
    mut removed: RemovedComponents<AudioSource>,
) {
    for entity in removed.read() {
        AudioService::stop(&audio_state, entity);
    }
}

/// 定期清理系统 - 移除已完成的音频实例（v2 版本）
pub fn audio_gc_system_v2(audio_state: Res<AudioState>) {
    AudioService::cleanup(&audio_state);
}

// ============================================================================
// 兼容层：旧版系统函数
// ============================================================================

/// 音频播放系统 - 根据 AudioSource 组件状态自动管理音频播放
/// 
/// **注意**: 推荐使用 `audio_playback_system_v2` 配合 `AudioState`
#[allow(deprecated)]
pub fn audio_playback_system(
    audio_system: Res<AudioSystem>,
    mut query: Query<(Entity, &mut AudioSource)>,
) {
    for (entity, mut source) in query.iter_mut() {
        let currently_playing = audio_system.is_playing(entity);
        let currently_paused = audio_system.is_paused(entity);
        
        if source.is_playing && !source.is_paused {
            // 需要播放
            if !currently_playing && !currently_paused {
                // 开始新的播放
                if !source.path.is_empty() {
                    if let Err(e) = audio_system.play_file(
                        entity,
                        &source.path,
                        source.volume,
                        source.loop_sound,
                    ) {
                        eprintln!("Failed to play audio '{}': {}", source.path, e);
                        source.is_playing = false;
                    }
                }
            } else if currently_paused {
                // 恢复播放
                audio_system.resume(entity);
            }
        } else if source.is_paused && currently_playing {
            // 需要暂停
            audio_system.pause(entity);
        } else if !source.is_playing {
            // 需要停止
            if currently_playing || currently_paused {
                audio_system.stop(entity);
            }
        }
    }
}

/// 音频清理系统 - 在实体移除时停止其音频
/// 
/// **注意**: 推荐使用 `audio_cleanup_system_v2` 配合 `AudioState`
#[allow(deprecated)]
pub fn audio_cleanup_system(
    audio_system: Res<AudioSystem>,
    mut removed: RemovedComponents<AudioSource>,
) {
    for entity in removed.read() {
        audio_system.stop(entity);
    }
}

/// 定期清理系统 - 移除已完成的音频实例
/// 
/// **注意**: 推荐使用 `audio_gc_system_v2` 配合 `AudioState`
#[allow(deprecated)]
pub fn audio_gc_system(audio_system: Res<AudioSystem>) {
    audio_system.cleanup();
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
            is_paused: false,
        };
        
        assert_eq!(source.name, "test.mp3");
        assert!(source.loop_sound);
        assert_eq!(source.volume, 0.5);
    }
    
    // ========================================
    // 新 Service 模式测试
    // ========================================
    
    #[test]
    fn test_audio_state_creation() {
        let state = AudioState::new();
        assert_eq!(state.master_volume, 1.0);
    }
    
    #[test]
    fn test_audio_service_master_volume() {
        let mut state = AudioState::new();
        
        AudioService::set_master_volume(&mut state, 0.5);
        assert_eq!(state.master_volume, 0.5);
        
        // 测试边界值
        AudioService::set_master_volume(&mut state, -1.0);
        assert_eq!(state.master_volume, 0.0);
        
        AudioService::set_master_volume(&mut state, 10.0);
        assert_eq!(state.master_volume, 2.0);
    }
    
    #[test]
    fn test_audio_service_stop_all() {
        let state = AudioState::new();
        // 测试 stop_all 不会 panic
        AudioService::stop_all(&state);
        assert_eq!(AudioService::active_count(&state), 0);
    }
    
    #[test]
    fn test_audio_service_cleanup() {
        let state = AudioState::new();
        // 测试 cleanup 不会 panic
        AudioService::cleanup(&state);
    }
    
    // ========================================
    // 兼容层测试（使用 #[allow(deprecated)]）
    // ========================================
    
    #[test]
    #[allow(deprecated)]
    fn test_audio_system_creation() {
        let system = AudioSystem::new();
        // 音频系统应该能够创建 (即使没有音频设备也不会panic)
        assert_eq!(system.master_volume, 1.0);
    }
    
    #[test]
    #[allow(deprecated)]
    fn test_audio_system_master_volume() {
        let mut system = AudioSystem::new();
        system.set_master_volume(0.5);
        assert_eq!(system.master_volume, 0.5);
        
        // 测试边界值
        system.set_master_volume(-1.0);
        assert_eq!(system.master_volume, 0.0);
        
        system.set_master_volume(10.0);
        assert_eq!(system.master_volume, 2.0);
    }
    
    #[test]
    #[allow(deprecated)]
    fn test_audio_system_stop_all() {
        let system = AudioSystem::new();
        // 测试stop_all不会panic
        system.stop_all();
        assert_eq!(system.active_count(), 0);
    }
    
    #[test]
    fn test_audio_system_to_state_conversion() {
        let system = AudioSystem::new();
        let state = system.to_state();
        assert_eq!(state.master_volume, 1.0);
    }
}
