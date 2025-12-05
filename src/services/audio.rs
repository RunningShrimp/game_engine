//! 音频服务模块
//!
//! 提供音频播放、暂停、停止、音量控制等功能。
//! 底层使用rodio库实现跨平台音频播放。

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

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
        match OutputStreamBuilder::open_default_stream() {
            Ok(stream) => Some(Self {
                _stream: stream,
                sinks: HashMap::new(),
            }),
            Err(_) => None,
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
        if self.sinks.contains_key(name) {
            return;
        }

        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            if let Ok(source) = Decoder::new(reader) {
                let sink = Sink::connect_new(self._stream.mixer());
                sink.set_volume(volume);
                if looped {
                    sink.append(source.repeat_infinite());
                } else {
                    sink.append(source);
                }
                self.sinks.insert(name.to_string(), sink);
            } else {
                tracing::error!(target: "audio", "Failed to decode audio: {}", path);
            }
        } else {
            tracing::error!(target: "audio", "Failed to open audio file: {}", path);
        }
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
        if let Some(sink) = self.sinks.remove(name) {
            sink.stop();
        }
    }

    /// 暂停指定音频
    pub fn pause_sound(&mut self, name: &str) {
        if let Some(sink) = self.sinks.get(name) {
            sink.pause();
        }
    }

    /// 恢复指定音频
    pub fn resume_sound(&mut self, name: &str) {
        if let Some(sink) = self.sinks.get(name) {
            sink.play();
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
        if let Some(sink) = self.sinks.get(name) {
            sink.set_volume(volume);
        }
    }

    /// 清理已完成的音频
    ///
    /// 移除所有已播放完成的音频接收器，释放资源。
    pub fn cleanup(&mut self) {
        self.sinks.retain(|_, sink| !sink.empty());
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
    let (tx, rx) = crossbeam_channel::unbounded::<AudioCommand>();
    std::thread::spawn(move || {
        if let Some(mut backend) = new_backend() {
            loop {
                match rx.recv() {
                    Ok(AudioCommand::Play {
                        name,
                        path,
                        volume,
                        looped,
                    }) => backend.play(&name, &path, volume, looped),
                    Ok(AudioCommand::Stop { name }) => backend.stop(&name),
                    Ok(AudioCommand::Pause { name }) => backend.pause(&name),
                    Ok(AudioCommand::Resume { name }) => backend.resume(&name),
                    Ok(AudioCommand::SetVolume { name, volume }) => {
                        backend.set_volume(&name, volume)
                    }
                    Ok(AudioCommand::Cleanup) => backend.cleanup(),
                    Err(_) => break,
                }
            }
        }
    });
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
    let _ = q.0.send(AudioCommand::Cleanup);
}
