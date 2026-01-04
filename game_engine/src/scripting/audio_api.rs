// 音频系统脚本API
//
// 提供完整的音频系统脚本接口，支持2D/3D音效、背景音乐、3D空间音频

use crate::scripting::{ScriptResult, api::ScriptApi, system::ScriptValue};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 音频系统脚本API
pub struct AudioScriptApi {
    /// 音频资源存储（模拟）
    audio_clips: Arc<Mutex<HashMap<String, AudioClipInfo>>>,
    /// 活跃的音频源
    active_sources: Arc<Mutex<HashMap<String, AudioSourceInfo>>>,
}

/// 音频片段信息
#[derive(Debug, Clone)]
struct AudioClipInfo {
    name: String,
    duration: f32,
    channels: u8,
    sample_rate: u32,
}

/// 音频源信息
#[derive(Debug, Clone)]
struct AudioSourceInfo {
    clip_name: String,
    is_playing: bool,
    is_looping: bool,
    volume: f32,
    pitch: f32,
    spatial: bool,
    position: Option<(f32, f32, f32)>,
}

impl AudioScriptApi {
    /// 创建新的音频脚本API
    pub fn new() -> Self {
        Self {
            audio_clips: Arc::new(Mutex::new(HashMap::new())),
            active_sources: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 注册所有音频API到脚本系统
    pub fn register_api(&self, api: &mut ScriptApi) {
        // ========== 2D音频 ==========
        self.register_2d_audio_api(api);

        // ========== 3D音频 ==========
        self.register_3d_audio_api(api);

        // ========== 音乐控制 ==========
        self.register_music_api(api);

        // ========== 音频资源 ==========
        self.register_resource_api(api);
    }

    /// 注册2D音频API
    fn register_2d_audio_api(&self, api: &mut ScriptApi) {
        let sources = self.active_sources.clone();

        // 播放2D音效
        api.register_function("audio_play_2d", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("audio_play_2d() requires clip_name".to_string());
            }

            let clip_name = match &args[0] {
                ScriptValue::String(name) => name.clone(),
                _ => return ScriptResult::Error("clip_name must be a string".to_string()),
            };

            let volume = if args.len() > 1 {
                args[1].as_number().unwrap_or(1.0)
            } else {
                1.0
            };

            let pitch = if args.len() > 2 {
                args[2].as_number().unwrap_or(1.0)
            } else {
                1.0
            };

            let loop_param = if args.len() > 3 {
                args[3].as_boolean().unwrap_or(false)
            } else {
                false
            };

            // 创建音频源
            let source_id = format!(
                "source_{}_{}",
                clip_name,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );

            if let Ok(mut sources_guard) = sources.try_lock() {
                sources_guard.insert(
                    source_id.clone(),
                    AudioSourceInfo {
                        clip_name: clip_name.clone(),
                        is_playing: true,
                        is_looping: loop_param,
                        volume: volume as f32,
                        pitch: pitch as f32,
                        spatial: false,
                        position: None,
                    },
                );
            }

            ScriptResult::Success(ScriptValue::String(format!(
                "Playing 2D audio: {clip_name}, volume={volume}, pitch={pitch}, loop={loop_param}, source_id={source_id}"
            )))
        });

        // 停止音频
        let sources = self.active_sources.clone();
        api.register_function("audio_stop", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("audio_stop() requires source_id".to_string());
            }

            let source_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("source_id must be a string".to_string()),
            };

            if let Ok(mut sources_guard) = sources.try_lock() {
                if let Some(source) = sources_guard.get_mut(&source_id) {
                    source.is_playing = false;
                    return ScriptResult::Success(ScriptValue::String(format!(
                        "Stopped audio: {source_id}"
                    )));
                }
            }

            ScriptResult::Error("Audio source not found".to_string())
        });

        // 暂停音频
        let sources = self.active_sources.clone();
        api.register_function("audio_pause", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("audio_pause() requires source_id".to_string());
            }

            let source_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("source_id must be a string".to_string()),
            };

            if let Ok(mut sources_guard) = sources.try_lock() {
                if let Some(source) = sources_guard.get_mut(&source_id) {
                    source.is_playing = false;
                    return ScriptResult::Success(ScriptValue::String(format!(
                        "Paused audio: {source_id}"
                    )));
                }
            }

            ScriptResult::Error("Audio source not found".to_string())
        });

        // 恢复音频
        let sources = self.active_sources.clone();
        api.register_function("audio_resume", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("audio_resume() requires source_id".to_string());
            }

            let source_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("source_id must be a string".to_string()),
            };

            if let Ok(mut sources_guard) = sources.try_lock() {
                if let Some(source) = sources_guard.get_mut(&source_id) {
                    source.is_playing = true;
                    return ScriptResult::Success(ScriptValue::String(format!(
                        "Resumed audio: {source_id}"
                    )));
                }
            }

            ScriptResult::Error("Audio source not found".to_string())
        });

        // 设置音量
        let sources = self.active_sources.clone();
        api.register_function("audio_set_volume", move |args| {
            if args.len() < 2 {
                return ScriptResult::Error(
                    "audio_set_volume() requires source_id, volume".to_string(),
                );
            }

            let source_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("source_id must be a string".to_string()),
            };

            let volume = args[1].as_number().unwrap_or(1.0);

            if let Ok(mut sources_guard) = sources.try_lock() {
                if let Some(source) = sources_guard.get_mut(&source_id) {
                    source.volume = (volume.clamp(0.0, 1.0)) as f32;
                    return ScriptResult::Success(ScriptValue::String(format!(
                        "Volume set: {source_id}={volume}"
                    )));
                }
            }

            ScriptResult::Error("Audio source not found".to_string())
        });

        // 设置音高
        let sources = self.active_sources.clone();
        api.register_function("audio_set_pitch", move |args| {
            if args.len() < 2 {
                return ScriptResult::Error(
                    "audio_set_pitch() requires source_id, pitch".to_string(),
                );
            }

            let source_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("source_id must be a string".to_string()),
            };

            let pitch = args[1].as_number().unwrap_or(1.0);

            if let Ok(mut sources_guard) = sources.try_lock() {
                if let Some(source) = sources_guard.get_mut(&source_id) {
                    source.pitch = (pitch.clamp(0.1, 2.0)) as f32;
                    return ScriptResult::Success(ScriptValue::String(format!(
                        "Pitch set: {source_id}={pitch}"
                    )));
                }
            }

            ScriptResult::Error("Audio source not found".to_string())
        });
    }

    /// 注册3D音频API
    fn register_3d_audio_api(&self, api: &mut ScriptApi) {
        let sources = self.active_sources.clone();

        // 播放3D音效
        api.register_function("audio_play_3d", move |args| {
            if args.len() < 4 {
                return ScriptResult::Error(
                    "audio_play_3d() requires clip_name, x, y, z".to_string(),
                );
            }

            let clip_name = match &args[0] {
                ScriptValue::String(name) => name.clone(),
                _ => return ScriptResult::Error("clip_name must be a string".to_string()),
            };

            let position = (
                args[1].as_number().unwrap_or(0.0),
                args[2].as_number().unwrap_or(0.0),
                args[3].as_number().unwrap_or(0.0),
            );

            let volume = if args.len() > 4 {
                args[4].as_number().unwrap_or(1.0)
            } else {
                1.0
            };

            // 创建3D音频源
            let source_id = format!(
                "3d_source_{}_{}",
                clip_name,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );

            if let Ok(mut sources_guard) = sources.try_lock() {
                sources_guard.insert(
                    source_id.clone(),
                    AudioSourceInfo {
                        clip_name: clip_name.clone(),
                        is_playing: true,
                        is_looping: false,
                        volume: volume as f32,
                        pitch: 1.0_f32,
                        spatial: true,
                        position: Some((position.0 as f32, position.1 as f32, position.2 as f32)),
                    },
                );
            }

            ScriptResult::Success(ScriptValue::String(format!(
                "Playing 3D audio: {}, position=({},{},{}), volume={}, source_id={}",
                clip_name, position.0, position.1, position.2, volume, source_id
            )))
        });

        // 更新3D音频位置
        let sources = self.active_sources.clone();
        api.register_function("audio_update_3d_position", move |args| {
            if args.len() < 4 {
                return ScriptResult::Error(
                    "audio_update_3d_position() requires source_id, x, y, z".to_string(),
                );
            }

            let source_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("source_id must be a string".to_string()),
            };

            let position = (
                args[1].as_number().unwrap_or(0.0),
                args[2].as_number().unwrap_or(0.0),
                args[3].as_number().unwrap_or(0.0),
            );

            if let Ok(mut sources_guard) = sources.try_lock() {
                if let Some(source) = sources_guard.get_mut(&source_id) {
                    source.position =
                        Some((position.0 as f32, position.1 as f32, position.2 as f32));
                    return ScriptResult::Success(ScriptValue::String(format!(
                        "Position updated: {}=({},{},{})",
                        source_id, position.0, position.1, position.2
                    )));
                }
            }

            ScriptResult::Error("Audio source not found".to_string())
        });

        // 设置3D音频衰减距离
        api.register_function("audio_set_3d_attenuation", move |args| {
            if args.len() < 3 {
                return ScriptResult::Error(
                    "audio_set_3d_attenuation() requires source_id, min_distance, max_distance"
                        .to_string(),
                );
            }

            let source_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("source_id must be a string".to_string()),
            };

            let min_distance = args[1].as_number().unwrap_or(1.0);
            let max_distance = args[2].as_number().unwrap_or(100.0);

            ScriptResult::Success(ScriptValue::String(format!(
                "Attenuation set: {source_id}=[{min_distance}, {max_distance}]"
            )))
        });

        // 设置多普勒效应
        api.register_function("audio_enable_doppler", move |args| {
            if args.len() < 2 {
                return ScriptResult::Error(
                    "audio_enable_doppler() requires source_id, enabled".to_string(),
                );
            }

            let source_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("source_id must be a string".to_string()),
            };

            let enabled = args[1].as_boolean().unwrap_or(false);

            ScriptResult::Success(ScriptValue::String(format!(
                "Doppler effect: {source_id}={enabled}"
            )))
        });
    }

    /// 注册音乐控制API
    fn register_music_api(&self, api: &mut ScriptApi) {
        // 播放背景音乐
        api.register_function("audio_play_music", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("audio_play_music() requires music_name".to_string());
            }

            let music_name = match &args[0] {
                ScriptValue::String(name) => name.clone(),
                _ => return ScriptResult::Error("music_name must be a string".to_string()),
            };

            let volume = if args.len() > 1 {
                args[1].as_number().unwrap_or(0.7)
            } else {
                0.7
            };

            let loop_param = if args.len() > 2 {
                args[2].as_boolean().unwrap_or(true)
            } else {
                true
            };

            let fade_duration = if args.len() > 3 {
                args[3].as_number().unwrap_or(1.0)
            } else {
                1.0
            };

            ScriptResult::Success(ScriptValue::String(format!(
                "Playing music: {music_name}, volume={volume}, loop={loop_param}, fade={fade_duration}s"
            )))
        });

        // 停止音乐
        api.register_function("audio_stop_music", move |args| {
            let fade_duration = if !args.is_empty() {
                args[0].as_number().unwrap_or(1.0)
            } else {
                1.0
            };

            ScriptResult::Success(ScriptValue::String(format!(
                "Music stopped (fade={fade_duration}s)"
            )))
        });

        // 设置音乐音量
        api.register_function("audio_set_music_volume", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("audio_set_music_volume() requires volume".to_string());
            }

            let volume = args[0].as_number().unwrap_or(0.7);

            ScriptResult::Success(ScriptValue::String(format!("Music volume set to {volume}")))
        });

        // 淡入音乐
        api.register_function("audio_fade_in_music", move |args| {
            let duration = if !args.is_empty() {
                args[0].as_number().unwrap_or(2.0)
            } else {
                2.0
            };

            let target_volume = if args.len() > 1 {
                args[1].as_number().unwrap_or(0.7)
            } else {
                0.7
            };

            ScriptResult::Success(ScriptValue::String(format!(
                "Music fade in: duration={duration}s, target_volume={target_volume}"
            )))
        });

        // 淡出音乐
        api.register_function("audio_fade_out_music", move |args| {
            let duration = if !args.is_empty() {
                args[0].as_number().unwrap_or(2.0)
            } else {
                2.0
            };

            ScriptResult::Success(ScriptValue::String(format!(
                "Music fade out: duration={duration}s"
            )))
        });
    }

    /// 注册音频资源API
    fn register_resource_api(&self, api: &mut ScriptApi) {
        let clips = self.audio_clips.clone();

        // 加载音频资源
        api.register_function("audio_load_clip", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("audio_load_clip() requires clip_path".to_string());
            }

            let clip_path = match &args[0] {
                ScriptValue::String(path) => path.clone(),
                _ => return ScriptResult::Error("clip_path must be a string".to_string()),
            };

            let clip_name = if args.len() > 1 {
                match &args[1] {
                    ScriptValue::String(name) => Some(name.clone()),
                    _ => None,
                }
            } else {
                None
            };

            let name = clip_name.unwrap_or_else(|| {
                // 从路径提取文件名
                clip_path.split('/').next_back().unwrap_or(&clip_path).to_string()
            });

            // 模拟加载音频片段
            if let Ok(mut clips_guard) = clips.try_lock() {
                clips_guard.insert(
                    name.clone(),
                    AudioClipInfo {
                        name: name.clone(),
                        duration: 5.0, // 模拟5秒时长
                        channels: 2,
                        sample_rate: 44100,
                    },
                );
            }

            ScriptResult::Success(ScriptValue::String(format!("Audio clip loaded: {name}")))
        });

        // 卸载音频资源
        let clips = self.audio_clips.clone();
        api.register_function("audio_unload_clip", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("audio_unload_clip() requires clip_name".to_string());
            }

            let clip_name = match &args[0] {
                ScriptValue::String(name) => name.clone(),
                _ => return ScriptResult::Error("clip_name must be a string".to_string()),
            };

            if let Ok(mut clips_guard) = clips.try_lock() {
                if clips_guard.remove(&clip_name).is_some() {
                    return ScriptResult::Success(ScriptValue::String(format!(
                        "Audio clip unloaded: {clip_name}"
                    )));
                }
            }

            ScriptResult::Error("Audio clip not found".to_string())
        });

        // 获取音频信息
        let clips = self.audio_clips.clone();
        api.register_function("audio_get_clip_info", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("audio_get_clip_info() requires clip_name".to_string());
            }

            let clip_name = match &args[0] {
                ScriptValue::String(name) => name.clone(),
                _ => return ScriptResult::Error("clip_name must be a string".to_string()),
            };

            if let Ok(clips_guard) = clips.try_lock() {
                if let Some(clip) = clips_guard.get(&clip_name) {
                    return ScriptResult::Success(ScriptValue::String(format!(
                        "Clip info: name={}, duration={}s, channels={}, sample_rate={}Hz",
                        clip.name, clip.duration, clip.channels, clip.sample_rate
                    )));
                }
            }

            ScriptResult::Error("Audio clip not found".to_string())
        });
    }
}

impl Default for AudioScriptApi {
    fn default() -> Self {
        Self::new()
    }
}
