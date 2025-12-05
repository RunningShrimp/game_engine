//! 音频流式加载模块
//!
//! 实现音频流式解码和缓冲区管理，支持大音频文件的分块加载和播放。
//!
//! ## 功能特性
//!
//! - 流式音频解码
//! - 双缓冲或三缓冲管理
//! - 自动预加载
//! - 内存使用优化
//! - 支持多种音频格式（WAV, MP3, OGG等）
//!
//! ## 使用示例
//!
//! ```rust
//! use crate::audio::streaming::*;
//!
//! // 创建流式音频加载器
//! let mut stream_loader = AudioStreamLoader::new();
//!
//! // 开始流式加载音频
//! let stream_id = stream_loader.start_streaming("assets/music.mp3", StreamConfig::default())?;
//!
//! // 获取音频流
//! if let Some(stream) = stream_loader.get_stream(stream_id) {
//!     // 播放流式音频
//!     stream.play()?;
//! }
//! ```

use crate::impl_default;
use crate::core::utils::current_timestamp_ms;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// 音频流式加载错误
#[derive(Error, Debug)]
pub enum StreamingError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Decode error: {0}")]
    DecodeError(String),
    #[error("Stream not found: {0}")]
    StreamNotFound(u64),
    #[error("Stream already exists: {0}")]
    StreamAlreadyExists(u64),
    #[error("Buffer overflow")]
    BufferOverflow,
    #[error("Stream ended")]
    StreamEnded,
}

/// 音频流ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamId(pub u64);

impl StreamId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 流式配置
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// 缓冲区大小（样本数）
    pub buffer_size: usize,
    /// 预加载缓冲区数量
    pub preload_buffers: usize,
    /// 是否循环播放
    pub looped: bool,
    /// 采样率（Hz）
    pub sample_rate: Option<u32>,
    /// 声道数
    pub channels: Option<u16>,
}

impl_default!(StreamConfig {
    buffer_size: 44100,
    preload_buffers: 2,
    looped: false,
    sample_rate: None,
    channels: None,
});

/// 音频缓冲区
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    /// 音频数据（交错格式：LRLRLR...）
    pub data: Vec<f32>,
    /// 采样率（Hz）
    pub sample_rate: u32,
    /// 声道数
    pub channels: u16,
    /// 是否已填充
    pub filled: bool,
    /// 时间戳（用于排序）
    pub timestamp: u64,
}

impl AudioBuffer {
    /// 创建新的音频缓冲区
    pub fn new(sample_rate: u32, channels: u16, size: usize) -> Self {
        Self {
            data: vec![0.0; size * channels as usize],
            sample_rate,
            channels,
            filled: false,
            timestamp: current_timestamp_ms(),
        }
    }

    /// 获取样本数
    pub fn sample_count(&self) -> usize {
        self.data.len() / self.channels as usize
    }

    /// 获取时长（秒）
    pub fn duration(&self) -> f32 {
        self.sample_count() as f32 / self.sample_rate as f32
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.data.fill(0.0);
        self.filled = false;
    }

    /// 填充数据
    pub fn fill(&mut self, data: &[f32]) -> Result<(), StreamingError> {
        if data.len() > self.data.len() {
            return Err(StreamingError::BufferOverflow);
        }

        self.data[..data.len()].copy_from_slice(data);
        self.filled = true;
        self.timestamp = current_timestamp_ms();
        Ok(())
    }
}

/// 音频流状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamState {
    /// 初始化中
    Initializing,
    /// 加载中
    Loading,
    /// 就绪（可以播放）
    Ready,
    /// 播放中
    Playing,
    /// 暂停
    Paused,
    /// 停止
    Stopped,
    /// 结束
    Ended,
    /// 错误
    Error(String),
}

/// 音频流
pub struct AudioStream {
    /// 流ID
    pub id: StreamId,
    /// 文件路径
    pub path: PathBuf,
    /// 当前状态
    pub state: StreamState,
    /// 配置
    pub config: StreamConfig,
    /// 缓冲区队列
    buffers: Vec<AudioBuffer>,
    /// 当前播放的缓冲区索引
    current_buffer_index: usize,
    /// 当前缓冲区中的样本位置
    current_sample_position: usize,
    /// 采样率
    sample_rate: u32,
    /// 声道数
    channels: u16,
    /// 总时长（秒，如果已知）
    total_duration: Option<f32>,
    /// 已播放时长（秒）
    played_duration: f32,
    /// 解码器句柄（占位，实际应使用rodio或其他解码器）
    decoder_handle: Option<()>,
}

impl AudioStream {
    /// 创建新的音频流
    pub fn new(id: StreamId, path: PathBuf, config: StreamConfig) -> Self {
        Self {
            id,
            path,
            state: StreamState::Initializing,
            config,
            buffers: Vec::new(),
            current_buffer_index: 0,
            current_sample_position: 0,
            sample_rate: 44100,
            channels: 2,
            total_duration: None,
            played_duration: 0.0,
            decoder_handle: None,
        }
    }

    /// 初始化解码器
    pub fn initialize_decoder(&mut self) -> Result<(), StreamingError> {
        // NOTE: 实际实现中需要：
        // 1. 打开音频文件
        // 2. 创建解码器（使用rodio或其他库）
        // 3. 读取音频格式信息（采样率、声道数）
        // 4. 预加载初始缓冲区

        // 占位实现
        self.sample_rate = self.config.sample_rate.unwrap_or(44100);
        self.channels = self.config.channels.unwrap_or(2);

        // 创建预加载缓冲区
        for _ in 0..self.config.preload_buffers {
            let buffer = AudioBuffer::new(self.sample_rate, self.channels, self.config.buffer_size);
            self.buffers.push(buffer);
        }

        self.state = StreamState::Loading;
        Ok(())
    }

    /// 更新流（填充缓冲区）
    pub fn update(&mut self) -> Result<(), StreamingError> {
        if matches!(&self.state, StreamState::Ended) {
            return Ok(());
        }

        // 检查是否需要填充缓冲区
        let buffers_to_fill = self.buffers.iter().filter(|b| !b.filled).count();

        if buffers_to_fill < self.config.preload_buffers {
            // 填充空缓冲区
            for i in 0..self.buffers.len() {
                if !self.buffers[i].filled {
                    self.fill_buffer(i)?;
                    break;
                }
            }
        }

        // 检查是否就绪
        if matches!(&self.state, StreamState::Loading) {
            let ready_buffers = self.buffers.iter().filter(|b| b.filled).count();

            if ready_buffers >= self.config.preload_buffers {
                self.state = StreamState::Ready;
            }
        }

        Ok(())
    }

    /// 填充单个缓冲区
    fn fill_buffer(&mut self, buffer_index: usize) -> Result<(), StreamingError> {
        // NOTE: 实际实现中需要：
        // 1. 从解码器读取样本
        // 2. 转换为f32格式
        // 3. 填充到缓冲区

        if buffer_index >= self.buffers.len() {
            return Err(StreamingError::BufferOverflow);
        }

        let buffer = &mut self.buffers[buffer_index];

        // 占位实现：填充零数据
        let sample_count = buffer.sample_count();
        let data_size = sample_count * buffer.channels as usize;
        let zero_data = vec![0.0; data_size];
        buffer.fill(&zero_data)?;

        Ok(())
    }

    /// 获取当前播放的样本数据
    pub fn get_samples(&mut self, count: usize) -> Result<Vec<f32>, StreamingError> {
        if !matches!(self.state, StreamState::Playing) {
            return Err(StreamingError::StreamEnded);
        }

        let mut samples = Vec::with_capacity(count * self.channels as usize);
        let mut remaining = count;

        while remaining > 0 && self.current_buffer_index < self.buffers.len() {
            // 检查缓冲区是否已填充
            let buffer_filled = self.buffers[self.current_buffer_index].filled;

            if !buffer_filled {
                // 缓冲区未填充，尝试更新
                self.update()?;
                // 重新检查
                if !self.buffers[self.current_buffer_index].filled {
                    break; // 无法获取更多数据
                }
            }

            let buffer = &self.buffers[self.current_buffer_index];
            let available = buffer.sample_count() - self.current_sample_position;
            let to_take = remaining.min(available);

            let start_idx = self.current_sample_position * buffer.channels as usize;
            let end_idx = start_idx + to_take * buffer.channels as usize;
            samples.extend_from_slice(&buffer.data[start_idx..end_idx]);

            self.current_sample_position += to_take;
            remaining -= to_take;

            // 检查是否到达缓冲区末尾
            if self.current_sample_position >= buffer.sample_count() {
                // 标记缓冲区为空，可以重新填充
                self.buffers[self.current_buffer_index].filled = false;
                self.current_buffer_index = (self.current_buffer_index + 1) % self.buffers.len();
                self.current_sample_position = 0;

                // 检查是否循环
                if !self.config.looped && self.current_buffer_index == 0 {
                    self.state = StreamState::Ended;
                    break;
                }
            }
        }

        // 更新播放时长
        let samples_played = (count - remaining) as f32;
        self.played_duration += samples_played / self.sample_rate as f32;

        Ok(samples)
    }

    /// 播放流
    pub fn play(&mut self) -> Result<(), StreamingError> {
        match &self.state {
            StreamState::Ready | StreamState::Paused | StreamState::Stopped => {
                self.state = StreamState::Playing;
                Ok(())
            }
            StreamState::Ended => {
                if self.config.looped {
                    self.current_buffer_index = 0;
                    self.current_sample_position = 0;
                    self.played_duration = 0.0;
                    self.state = StreamState::Playing;
                    Ok(())
                } else {
                    Err(StreamingError::StreamEnded)
                }
            }
            _ => Err(StreamingError::DecodeError("Stream not ready".to_string())),
        }
    }

    /// 暂停流
    pub fn pause(&mut self) -> Result<(), StreamingError> {
        if matches!(self.state, StreamState::Playing) {
            self.state = StreamState::Paused;
            Ok(())
        } else {
            Err(StreamingError::DecodeError(
                "Stream not playing".to_string(),
            ))
        }
    }

    /// 停止流
    pub fn stop(&mut self) -> Result<(), StreamingError> {
        self.state = StreamState::Stopped;
        self.current_buffer_index = 0;
        self.current_sample_position = 0;
        self.played_duration = 0.0;

        // 清空所有缓冲区
        for buffer in &mut self.buffers {
            buffer.clear();
        }

        Ok(())
    }

    /// 获取当前状态
    pub fn state(&self) -> &StreamState {
        &self.state
    }

    /// 获取采样率
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// 获取声道数
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// 获取已播放时长
    pub fn played_duration(&self) -> f32 {
        self.played_duration
    }

    /// 获取总时长（如果已知）
    pub fn total_duration(&self) -> Option<f32> {
        self.total_duration
    }
}

/// 音频流式加载器
#[derive(Default)]
pub struct AudioStreamLoader {
    /// 流映射
    streams: HashMap<StreamId, Arc<Mutex<AudioStream>>>,
    /// 下一个流ID
    next_stream_id: u64,
}

impl AudioStreamLoader {
    /// 创建新的音频流式加载器
    pub fn new() -> Self {
        Self {
            next_stream_id: 1,
            ..Default::default()
        }
    }

    /// 开始流式加载音频
    pub fn start_streaming(
        &mut self,
        path: impl AsRef<Path>,
        config: StreamConfig,
    ) -> Result<StreamId, StreamingError> {
        let path = path.as_ref();

        // 检查文件是否存在
        if !path.exists() {
            return Err(StreamingError::FileNotFound(path.display().to_string()));
        }

        let id = StreamId::new(self.next_stream_id);
        self.next_stream_id += 1;

        let mut stream = AudioStream::new(id, path.to_path_buf(), config);
        stream.initialize_decoder()?;

        self.streams.insert(id, Arc::new(Mutex::new(stream)));

        Ok(id)
    }

    /// 获取音频流
    pub fn get_stream(&self, id: StreamId) -> Option<Arc<Mutex<AudioStream>>> {
        self.streams.get(&id).cloned()
    }

    /// 移除音频流
    pub fn remove_stream(&mut self, id: StreamId) -> Result<(), StreamingError> {
        self.streams
            .remove(&id)
            .ok_or(StreamingError::StreamNotFound(id.0))?;
        Ok(())
    }

    /// 更新所有流
    pub fn update_all(&self) -> Result<(), StreamingError> {
        for stream in self.streams.values() {
            if let Ok(mut s) = stream.lock() {
                s.update()?;
            }
        }
        Ok(())
    }

    /// 获取流数量
    pub fn stream_count(&self) -> usize {
        self.streams.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_buffer() {
        let mut buffer = AudioBuffer::new(44100, 2, 1000);

        assert_eq!(buffer.sample_count(), 1000);
        assert_eq!(buffer.channels, 2);
        assert!(!buffer.filled);

        let test_data = vec![0.5; 2000];
        buffer.fill(&test_data).unwrap();

        assert!(buffer.filled);
        assert_eq!(buffer.data.len(), 2000);
    }

    #[test]
    fn test_audio_stream() {
        let mut stream = AudioStream::new(
            StreamId::new(1),
            PathBuf::from("test.wav"),
            StreamConfig::default(),
        );

        assert_eq!(*stream.state(), StreamState::Initializing);

        stream.initialize_decoder().unwrap();
        assert!(matches!(*stream.state(), StreamState::Loading));

        stream.update().unwrap();
        // 更新后应该变为Ready状态（如果缓冲区已填充）
    }

    #[test]
    fn test_stream_loader() {
        let mut loader = AudioStreamLoader::new();

        assert_eq!(loader.stream_count(), 0);

        // 注意：这个测试需要实际文件，所以可能会失败
        // 在实际使用中应该创建测试文件或使用mock
    }
}
