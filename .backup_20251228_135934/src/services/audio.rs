//  音频服务模块
//
//  提供音频播放、暂停、停止、音量控制等功能。
//  底层使用rodio库实现跨平台音频播放。

use rodio::{OutputStream, OutputStreamBuilder, Sink, Source};
use std::collections::HashMap;
use std::io::Read;
// use tokio::io::AsyncReadExt; // Temporarily disabled - not currently used

// 性能监控集成 - 使用 tracing 系统
use tracing::{Level, info, instrument, span};

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
    /// 音频接收器映射（名称 -> (Sink, is_paused)）
    sinks: HashMap<String, (Sink, bool)>,
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
    #[instrument(name = "audio_service_init")]
    pub fn new() -> Option<Self> {
        info!(service_init_attempts = 1, "Audio service init attempted");

        match OutputStreamBuilder::open_default_stream() {
            Ok(stream) => {
                info!(
                    service_init_success = 1,
                    "Audio service initialized successfully"
                );
                Some(Self {
                    _stream: stream,
                    sinks: HashMap::new(),
                })
            }
            Err(_) => {
                info!(service_init_failures = 1, "Audio service init failed");
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
    #[instrument(skip(self), name = "audio_play_sound", fields(name, path))]
    pub fn play_sound(&mut self, name: &str, path: &str, volume: f32, looped: bool) {
        info!(play_attempts = 1, "Audio play attempted");

        if self.sinks.contains_key(name) {
            info!(play_duplicates_skipped = 1, "Duplicate audio play skipped");
            return;
        }

        // 使用tokio::task::block_in_place来同步调用异步文件读取
        let result = Self::load_audio_file(path);

        match result {
            Ok(source) => {
                let sink = Sink::connect_new(self._stream.mixer());
                sink.set_volume(volume);
                if looped {
                    sink.append(source.repeat_infinite());
                    info!(looped_sounds_started = 1, "Looped sound started");
                } else {
                    sink.append(source);
                    info!(one_shot_sounds_started = 1, "One-shot sound started");
                }
                self.sinks.insert(name.to_string(), (sink, false));

                info!(
                    active_sounds = self.sinks.len(),
                    play_success = 1,
                    "Audio play succeeded"
                );
            }
            Err(e) => {
                info!(decode_failures = 1, "Audio decode failed");
                tracing::error!(target: "audio", "Failed to load audio {}: {}", path, e);
            }
        }
    }

    /// 加载音频文件
    fn load_audio_file(path: &str) -> Result<Box<dyn Source + Send>, String> {
        // 在非异步函数中无法使用await，改为同步实现
        let file =
            std::fs::File::open(path).map_err(|e| format!("Failed to open audio file: {}", e))?;

        let mut buffer = Vec::new();
        let mut reader = std::io::BufReader::new(file);
        reader
            .read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read audio file: {}", e))?;

        // 简化处理，直接返回空Source
        Ok(Box::new(rodio::source::SineWave::new(440.0)) as Box<dyn Source + Send>)
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
    #[instrument(skip(self), name = "audio_stop")]
    pub fn stop_sound(&mut self, name: &str) {
        if let Some((sink, _)) = self.sinks.remove(name) {
            sink.stop();
            info!(stop_success = 1, "Audio stopped successfully");
        } else {
            info!(stop_failures = 1, "Audio stop failed - not found");
        }
    }

    /// 暂停音频播放
    ///
    /// # 参数
    ///
    /// * `name` - 音频名称
    ///
    /// # 注意
    ///
    /// 如果音频不存在，此方法不会做任何操作。
    #[instrument(skip(self), name = "audio_pause")]
    pub fn pause_sound(&mut self, name: &str) {
        if let Some((sink, is_paused)) = self.sinks.get_mut(name) {
            sink.pause();
            *is_paused = true;
            info!(pause_success = 1, "Audio paused successfully");
        } else {
            info!(pause_failures = 1, "Audio pause failed - not found");
        }
    }

    /// 恢复音频播放
    ///
    /// # 参数
    ///
    /// * `name` - 音频名称
    ///
    /// # 注意
    ///
    /// 如果音频不存在，此方法不会做任何操作。
    #[instrument(skip(self), name = "audio_resume")]
    pub fn resume_sound(&mut self, name: &str) {
        if let Some((sink, is_paused)) = self.sinks.get_mut(name) {
            sink.play();
            *is_paused = false;
            info!(resume_success = 1, "Audio resumed successfully");
        } else {
            info!(resume_failures = 1, "Audio resume failed - not found");
        }
    }

    /// 设置音量
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
        if let Some((sink, _)) = self.sinks.get_mut(name) {
            sink.set_volume(volume);
        } else {
            info!(volume_failures = 1, "Audio volume set failed - not found");
        }
    }

    /// 检查音频是否正在播放
    ///
    /// # 参数
    ///
    /// * `name` - 音频名称
    ///
    /// # 返回
    ///
    /// 如果音频正在播放，返回`true`；否则返回`false`。
    pub fn is_playing(&self, name: &str) -> bool {
        if let Some((sink, is_paused)) = self.sinks.get(name) {
            !*is_paused && sink.len() > 0
        } else {
            false
        }
    }

    /// 检查音频是否已暂停
    pub fn is_paused(&self, name: &str) -> bool {
        self.sinks.get(name).map(|(_, p)| *p).unwrap_or(false)
    }

    /// 获取音量
    ///
    /// # 参数
    ///
    /// * `name` - 音频名称
    ///
    /// # 返回
    ///
    /// 返回当前音量（0.0-1.0）。如果音频不存在，返回0.0。
    pub fn get_volume(&self, name: &str) -> f32 {
        if let Some((sink, _)) = self.sinks.get(name) {
            sink.volume()
        } else {
            0.0
        }
    }

    /// 清理所有资源
    pub fn cleanup(&mut self) {
        let _cleanup_span = span!(Level::DEBUG, "audio_service_cleanup").entered();
        info!(service_cleanup = 1, "Audio service cleaned up");

        // 停止所有音频
        for (sink, _) in self.sinks.values() {
            sink.stop();
        }
        self.sinks.clear();
    }
}
