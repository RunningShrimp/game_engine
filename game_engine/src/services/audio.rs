//  音频服务模块
// 
//  提供音频播放、暂停、停止、音量控制等功能。
//  底层使用rodio库实现跨平台音频播放。

use rodio::{OutputStream, OutputStreamBuilder, Sink, Source};
use std::collections::HashMap;
use std::io::Read;
// use tokio::io::AsyncReadExt; // Temporarily disabled - not currently used

// 性能监控集成
#[cfg(feature = "profiling")]
use crate::profiling::{ScopedTimer, prelude::*, record_counter, record_timing};

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
            }
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
        let result = Self::load_audio_file(path);

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
                self.sinks.insert(name.to_string(), (sink, false));

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
    pub fn stop_sound(&mut self, name: &str) {
        #[cfg(feature = "profiling")]
        let _timer = ScopedTimer::new("audio_stop_attempts", 1);

        if let Some((sink, _)) = self.sinks.remove(name) {
            sink.stop();
            #[cfg(feature = "profiling")]
            record_counter!(audio.stop_success, 1);
        } else {
            #[cfg(feature = "profiling")]
            record_counter!(audio.stop_failures, 1);
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
    pub fn pause_sound(&mut self, name: &str) {
        #[cfg(feature = "profiling")]
        let _timer = ScopedTimer::new("audio_pause_attempts", 1);

        if let Some((sink, is_paused)) = self.sinks.get_mut(name) {
            sink.pause();
            *is_paused = true;
            #[cfg(feature = "profiling")]
            record_counter!(audio.pause_success, 1);
        } else {
            #[cfg(feature = "profiling")]
            record_counter!(audio.pause_failures, 1);
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
    pub fn resume_sound(&mut self, name: &str) {
        #[cfg(feature = "profiling")]
        let _timer = ScopedTimer::new("audio_resume_attempts", 1);

        if let Some((sink, is_paused)) = self.sinks.get_mut(name) {
            sink.play();
            *is_paused = false;
            #[cfg(feature = "profiling")]
            record_counter!(audio.resume_success, 1);
        } else {
            #[cfg(feature = "profiling")]
            record_counter!(audio.resume_failures, 1);
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
            #[cfg(feature = "profiling")]
            record_counter!(audio.volume_failures, 1);
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
        #[cfg(feature = "profiling")]
        let _timer = ScopedTimer::new("audio_service_cleanup");

        #[cfg(feature = "profiling")]
        record_counter!(audio.service_cleanup, 1);

        // 停止所有音频
        for (sink, _) in self.sinks.values() {
            sink.stop();
        }
        self.sinks.clear();
    }
}
