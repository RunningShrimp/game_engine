//! 音频服务模块
//!
//! 提供音频播放、暂停、停止、音量控制等功能。
//! 底层使用rodio库实现跨平台音频播放。

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

// 性能监控集成
#[cfg(feature = "profiling")]
use crate::profiling::{
    ScopedTimer,
    record_counter,
    record_timing,
    prelude::*,
};

/// 音频服务
///
/// 管理音频流的播放、暂停、停止和音量控制。
///
/// # 使用示例
///
/// ```rust
/// use game_engine::services::audio::AudioService;
///
/// if let Some(mut audio) = AudioService::new() {
///     audio.play_sound("bgm", "assets/music.ogg", 0.8, true);
///     audio.set_volume("bgm", 0.5);
///     audio.pause_sound("bgm");
///     audio.resume_sound("bgm");
///     audio.stop_sound("bgm");
/// }
/// ```
pub struct AudioService {
    /// 音频输出流（保持生命周期）
    _stream: OutputStream,
    /// 音频接收器映射（名称 -> Sink）
    sinks: HashMap<String, Sink>,
}

impl AudioService {
    /// 创建新的音频服务
    ///
    /// # 返回
    ///
    /// 如果成功打开默认音频流，返回`Some(AudioService)`；否则返回`None`。
    ///
    /// # 错误
    ///
    /// 如果无法打开默认音频流（例如没有音频设备），返回`None`。
    pub fn new() -> Option<Self> {
        #[cfg(feature = "profiling")]
        let _timer = ScopedTimer::new("audio_service_init");
        
        #[cfg(feature = "profiling")]
        record_counter!(audio.service_init_attempts, 1);
        
        match OutputStreamBuilder::open_default_stream() {
            Ok(stream) => {
                #[cfg(feature = "profiling")]
                record_counter!(audio.service_init_success, 1);
                
                Some(Self {
                    _stream: stream,
                    sinks: HashMap::new(),
                })
            },
            Err(_) => {
                #[cfg(feature = "profiling")]
                record_counter!(audio.service_init_failures, 1);
                
                None
            }
        }
    }

    /// 播放音频
    ///
    /// # 参数
    ///
    /// * `name` - 音频名称（用于后续控制）
    /// * `path` - 音频文件路径
    /// * `volume` - 音量（0.0-1.0）
    /// * `looped` - 是否循环播放
    ///
    /// # 注意
    ///
    /// 如果同名音频已在播放，此方法不会做任何操作。
    pub fn play_sound(&mut self, name: &str, path: &str, volume: f32, looped: bool) {
        #[cfg(feature = "profiling")]
        let _timer = ScopedTimer::new("audio_play_sound");
        
        #[cfg(feature = "profiling")]
        record_counter!(audio.play_attempts, 1);
        
        if self.sinks.contains_key(name) {
            #[cfg(feature = "profiling")]
            record_counter!(audio.play_duplicates_skipped, 1);
            return;
        }

        #[cfg(feature = "profiling")]
        let start_time = Instant::now();

        // 使用tokio::task::block_in_place来同步调用异步文件读取
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                Self::load_audio_file(path).await
            })
        });

        match result {
            Ok(source) => {
                let sink = Sink::connect_new(self._stream.mixer());
                sink.set_volume(volume);
                if looped {
                    sink.append(source.repeat_infinite());
                    #[cfg(feature = "profiling")]
                    record_counter!(audio.looped_sounds_started, 1);
                } else {
                    sink.append(source);
                    #[cfg(feature = "profiling")]
                    record_counter!(audio.one_shot_sounds_started, 1);
                }
                self.sinks.insert(name.to_string(), sink);
                
                #[cfg(feature = "profiling")]
                {
                    record_counter!(audio.active_sounds, self.sinks.len() as u64);
                    record_counter!(audio.play_success, 1);
                    
                    if let Ok(duration) = start_time.elapsed() {
                        record_timing!(audio.play_latency_ms, duration.as_millis() as f64);
                    }
                }
            }
            Err(e) => {
                #[cfg(feature = "profiling")]
                record_counter!(audio.decode_failures, 1);
                tracing::error!(target: "audio", "Failed to load audio {}: {}", path, e);
            }
        }
    }

    /// 异步加载音频文件
    async fn load_audio_file(path: &str) -> Result<Box<dyn Source + Send>, String> {
        let file = tokio::fs::File::open(path).await
            .map_err(|e| format!("Failed to open audio file: {}", e))?;
        
        let reader = tokio::io::BufReader::new(file);
        
        // 将异步文件读取转换为同步解码器所需的同步读取
        let file_sync = tokio::task::spawn_blocking(move || {
            let mut buffer = Vec::new();
            let mut reader_sync = std::io::BufReader::new(reader.into_inner());
            reader_sync.read_to_end(&mut buffer)
                .map(|_| buffer)
                .map_err(|e| format!("Failed to read audio file: {}", e))
        }).await.map_err(|e| format!("Failed to read audio file: {}", e))?;
        
        let cursor = std::io::Cursor::new(file_sync);
        Decoder::new(cursor)
            .map_err(|e| format!("Failed to decode audio: {}", e))
    }

    /// 停止音频播放
    ///
    /// # 参数
    ///
    /// * `name` - 音频名称
    ///
    /// # 注意
    ///
    /// 如果音频不存在，此方法不会做任何操作。
    pub fn stop_sound(&mut self, name: &str) {
        #[cfg(feature = "profiling")]
        record_counter!(audio.stop_attempts, 1);
        
        if let Some(sink) = self.sinks.remove(name) {
            sink.stop();
            #[cfg(feature = "profiling")]
            {
                record_counter!(audio.stop_success, 1);
                record_counter!(audio.active_sounds, self.sinks.len() as u64);
            }
        } else {
            #[cfg(feature = "profiling")]
            record_counter!(audio.stop_not_found, 1);
        }
    }

    /// 暂停指定音频
    pub fn pause_sound(&mut self, name: &str) {
        #[cfg(feature = "profiling")]
        record_counter!(audio.pause_attempts, 1);
        
        if let Some(sink) = self.sinks.get(name) {
            sink.pause();
            #[cfg(feature = "profiling")]
            record_counter!(audio.pause_success, 1);
        } else {
            #[cfg(feature = "profiling")]
            record_counter!(audio.pause_not_found, 1);
        }
    }

    /// 恢复指定音频
    pub fn resume_sound(&mut self, name: &str) {
        #[cfg(feature = "profiling")]
        record_counter!(audio.resume_attempts, 1);
        
        if let Some(sink) = self.sinks.get(name) {
            sink.play();
            #[cfg(feature = "profiling")]
            record_counter!(audio.resume_success, 1);
        } else {
            #[cfg(feature = "profiling")]
            record_counter!(audio.resume_not_found, 1);
        }
    }

    /// 检查音频是否正在播放
    pub fn is_playing(&self, name: &str) -> bool {
        self.sinks
            .get(name)
            .map(|s| !s.is_paused() && !s.empty())
            .unwrap_or(false)
    }

    /// 检查音频是否暂停
    pub fn is_paused(&self, name: &str) -> bool {
        self.sinks.get(name).map(|s| s.is_paused()).unwrap_or(false)
    }

    /// 设置音频音量
    ///
    /// # 参数
    ///
    /// * `name` - 音频名称
    /// * `volume` - 音量（0.0-1.0）
    ///
    /// # 注意
    ///
    /// 如果音频不存在，此方法不会做任何操作。
    pub fn set_volume(&mut self, name: &str, volume: f32) {
        #[cfg(feature = "profiling")]
        record_counter!(audio.volume_set_attempts, 1);
        
        if let Some(sink) = self.sinks.get(name) {
            sink.set_volume(volume);
            #[cfg(feature = "profiling")]
            record_counter!(audio.volume_set_success, 1);
        } else {
            #[cfg(feature = "profiling")]
            record_counter!(audio.volume_set_not_found, 1);
        }
    }

    /// 清理已完成的音频
    ///
    /// 移除所有已播放完成的音频接收器，释放资源。
    pub fn cleanup(&mut self) {
        #[cfg(feature = "profiling")]
        let _timer = ScopedTimer::new("audio_cleanup");
        
        let before_count = self.sinks.len();
        self.sinks.retain(|_, sink| !sink.empty());
        let after_count = self.sinks.len();
        let cleaned_count = before_count - after_count;
        
        #[cfg(feature = "profiling")]
        {
            record_counter!(audio.cleanup_operations, 1);
            record_counter!(audio.sounds_cleaned, cleaned_count as u64);
            record_counter!(audio.active_sounds, after_count as u64);
        }
    }
}

/// 音频后端trait
///
/// 定义音频后端的通用接口，允许不同的音频实现。
pub trait AudioBackend {
    /// 播放音频
    fn play(&mut self, name: &str, path: &str, volume: f32, looped: bool);
    /// 停止音频
    fn stop(&mut self, name: &str);
    /// 暂停音频
    fn pause(&mut self, name: &str);
    /// 恢复音频
    fn resume(&mut self, name: &str);
    /// 设置音量
    fn set_volume(&mut self, name: &str, volume: f32);
    /// 检查是否正在播放
    fn is_playing(&self, name: &str) -> bool;
    /// 检查是否暂停
    fn is_paused(&self, name: &str) -> bool;
    /// 清理资源
    fn cleanup(&mut self);
}

impl AudioBackend for AudioService {
    fn play(&mut self, name: &str, path: &str, volume: f32, looped: bool) {
        self.play_sound(name, path, volume, looped);
    }
    fn stop(&mut self, name: &str) {
        self.stop_sound(name);
    }
    fn pause(&mut self, name: &str) {
        self.pause_sound(name);
    }
    fn resume(&mut self, name: &str) {
        self.resume_sound(name);
    }
    fn set_volume(&mut self, name: &str, volume: f32) {
        AudioService::set_volume(self, name, volume);
    }
    fn is_playing(&self, name: &str) -> bool {
        AudioService::is_playing(self, name)
    }
    fn is_paused(&self, name: &str) -> bool {
        AudioService::is_paused(self, name)
    }
    fn cleanup(&mut self) {
        AudioService::cleanup(self);
    }
}

/// 创建新的音频后端
///
/// # 返回
///
/// 如果成功创建音频服务，返回`Some(Box<dyn AudioBackend>)`；否则返回`None`。
pub fn new_backend() -> Option<Box<dyn AudioBackend>> {
    AudioService::new().map(|s| Box::new(s) as Box<dyn AudioBackend>)
}

/// 音频命令枚举
///
/// 用于在音频驱动线程和主线程之间传递命令。
#[derive(Clone)]
pub enum AudioCommand {
    /// 播放音频命令
    Play {
        /// 音频名称
        name: String,
        /// 音频文件路径
        path: String,
        /// 音量（0.0-1.0）
        volume: f32,
        /// 是否循环播放
        looped: bool,
    },
    /// 停止音频命令
    Stop {
        /// 音频名称
        name: String,
    },
    /// 暂停音频命令
    Pause {
        /// 音频名称
        name: String,
    },
    /// 恢复音频命令
    Resume {
        /// 音频名称
        name: String,
    },
    /// 设置音量命令
    SetVolume {
        /// 音频名称
        name: String,
        /// 音量（0.0-1.0）
        volume: f32,
    },
    /// 清理资源命令
    Cleanup,
}

/// 音频队列资源
///
/// ECS资源，用于向音频驱动线程发送命令。
#[derive(bevy_ecs::system::Resource, Clone)]
pub struct AudioQueueResource(pub crossbeam_channel::Sender<AudioCommand>);

/// 启动音频驱动线程
///
/// 创建一个独立的线程来处理音频命令，避免阻塞主线程。
///
/// # 返回
///
/// 如果成功创建音频后端和驱动线程，返回`Some(AudioQueueResource)`；否则返回`None`。
pub fn start_audio_driver() -> Option<AudioQueueResource> {
    #[cfg(feature = "profiling")]
    let _timer = ScopedTimer::new("audio_driver_start");
    
    #[cfg(feature = "profiling")]
    record_counter!(audio.driver_start_attempts, 1);
    
    let (tx, rx) = crossbeam_channel::unbounded::<AudioCommand>();
    std::thread::spawn(move || {
        #[cfg(feature = "profiling")]
        record_counter!(audio.driver_thread_started, 1);
        
        if let Some(mut backend) = new_backend() {
            #[cfg(feature = "profiling")]
            record_counter!(audio.driver_backend_created, 1);
            
            loop {
                #[cfg(feature = "profiling")]
                let command_start = std::time::Instant::now();
                
                match rx.recv() {
                    Ok(AudioCommand::Play {
                        name,
                        path,
                        volume,
                        looped,
                    }) => {
                        #[cfg(feature = "profiling")]
                        record_counter!(audio.driver_play_commands, 1);
                        backend.play(&name, &path, volume, looped);
                    },
                    Ok(AudioCommand::Stop { name }) => {
                        #[cfg(feature = "profiling")]
                        record_counter!(audio.driver_stop_commands, 1);
                        backend.stop(&name);
                    },
                    Ok(AudioCommand::Pause { name }) => {
                        #[cfg(feature = "profiling")]
                        record_counter!(audio.driver_pause_commands, 1);
                        backend.pause(&name);
                    },
                    Ok(AudioCommand::Resume { name }) => {
                        #[cfg(feature = "profiling")]
                        record_counter!(audio.driver_resume_commands, 1);
                        backend.resume(&name);
                    },
                    Ok(AudioCommand::SetVolume { name, volume }) => {
                        #[cfg(feature = "profiling")]
                        record_counter!(audio.driver_volume_commands, 1);
                        backend.set_volume(&name, volume)
                    }
                    Ok(AudioCommand::Cleanup) => {
                        #[cfg(feature = "profiling")]
                        record_counter!(audio.driver_cleanup_commands, 1);
                        backend.cleanup();
                    },
                    Err(_) => {
                        #[cfg(feature = "profiling")]
                        record_counter!(audio.driver_thread_exited, 1);
                        break;
                    },
                }
                
                #[cfg(feature = "profiling")]
                {
                    if let Ok(duration) = command_start.elapsed() {
                        record_timing!(audio.driver_command_processing_ms, duration.as_millis() as f64);
                    }
                }
            }
        } else {
            #[cfg(feature = "profiling")]
            record_counter!(audio.driver_backend_creation_failed, 1);
        }
    });
    
    #[cfg(feature = "profiling")]
    record_counter!(audio.driver_start_success, 1);
    
    Some(AudioQueueResource(tx))
}

/// 播放音频（便捷函数）
///
/// # 参数
///
/// * `q` - 音频队列资源
/// * `name` - 音频名称
/// * `path` - 音频文件路径
/// * `volume` - 音量（0.0-1.0）
/// * `looped` - 是否循环播放
pub fn audio_play(q: &AudioQueueResource, name: &str, path: &str, volume: f32, looped: bool) {
    #[cfg(feature = "profiling")]
    record_counter!(audio.queue_play_commands, 1);
    
    let _ = q.0.send(AudioCommand::Play {
        name: name.to_string(),
        path: path.to_string(),
        volume,
        looped,
    });
}

/// 停止音频（便捷函数）
///
/// # 参数
///
/// * `q` - 音频队列资源
/// * `name` - 音频名称
pub fn audio_stop(q: &AudioQueueResource, name: &str) {
    #[cfg(feature = "profiling")]
    record_counter!(audio.queue_stop_commands, 1);
    
    let _ = q.0.send(AudioCommand::Stop {
        name: name.to_string(),
    });
}

/// 暂停音频（便捷函数）
///
/// # 参数
///
/// * `q` - 音频队列资源
/// * `name` - 音频名称
pub fn audio_pause(q: &AudioQueueResource, name: &str) {
    #[cfg(feature = "profiling")]
    record_counter!(audio.queue_pause_commands, 1);
    
    let _ = q.0.send(AudioCommand::Pause {
        name: name.to_string(),
    });
}

/// 恢复音频（便捷函数）
///
/// # 参数
///
/// * `q` - 音频队列资源
/// * `name` - 音频名称
pub fn audio_resume(q: &AudioQueueResource, name: &str) {
    #[cfg(feature = "profiling")]
    record_counter!(audio.queue_resume_commands, 1);
    
    let _ = q.0.send(AudioCommand::Resume {
        name: name.to_string(),
    });
}

/// 设置音频音量（便捷函数）
///
/// # 参数
///
/// * `q` - 音频队列资源
/// * `name` - 音频名称
/// * `volume` - 音量（0.0-1.0）
pub fn audio_set_volume(q: &AudioQueueResource, name: &str, volume: f32) {
    #[cfg(feature = "profiling")]
    record_counter!(audio.queue_volume_commands, 1);
    
    let _ = q.0.send(AudioCommand::SetVolume {
        name: name.to_string(),
        volume,
    });
}

/// 清理音频资源（便捷函数）
///
/// # 参数
///
/// * `q` - 音频队列资源
pub fn audio_cleanup(q: &AudioQueueResource) {
    #[cfg(feature = "profiling")]
    record_counter!(audio.queue_cleanup_commands, 1);
    
    let _ = q.0.send(AudioCommand::Cleanup);
}
