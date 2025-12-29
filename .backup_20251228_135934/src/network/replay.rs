//! 网络回放系统
//!
//! 提供网络游戏状态的录制、回放和时间旅行调试功能：
//! - 状态录制
//! - 事件录制
//! - 回放控制
//! - 时间旅行调试
//! - 回放文件管理

use crate::network::synchronization::{EntityState, NetworkEvent};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// 回放配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayConfig {
    /// 是否启用录制
    pub enable_recording: bool,
    /// 是否启用回放
    pub enable_playback: bool,
    /// 快照间隔（tick数）
    pub snapshot_interval: u64,
    /// 最大快照数量
    pub max_snapshots: usize,
    /// 是否录制事件
    pub record_events: bool,
    /// 是否录制输入
    pub record_inputs: bool,
    /// 压缩级别（0-9，0=无压缩）
    pub compression_level: u32,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            enable_recording: true,
            enable_playback: false,
            snapshot_interval: 60, // 每60 tick一个快照
            max_snapshots: 1000,
            record_events: true,
            record_inputs: true,
            compression_level: 6,
        }
    }
}

/// 回放快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaySnapshot {
    /// 快照tick
    pub tick: u64,
    /// 时间戳
    pub timestamp: u64,
    /// 实体状态映射（实体ID -> 状态）
    pub entity_states: HashMap<u64, EntityState>,
    /// 事件列表
    pub events: Vec<NetworkEvent>,
}

/// 回放帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayFrame {
    /// 帧tick
    pub tick: u64,
    /// 时间戳
    pub timestamp: u64,
    /// 增量状态更新（实体ID -> 状态）
    pub delta_states: HashMap<u64, EntityState>,
    /// 事件列表
    pub events: Vec<NetworkEvent>,
    /// 输入数据（如果录制）
    pub inputs: Vec<u8>,
}

/// 回放文件头
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayHeader {
    /// 版本号
    pub version: u32,
    /// 录制开始时间
    pub start_time: u64,
    /// 录制结束时间
    pub end_time: u64,
    /// 总tick数
    pub total_ticks: u64,
    /// 快照数量
    pub snapshot_count: usize,
    /// 帧数量
    pub frame_count: usize,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 回放录制器
pub struct ReplayRecorder {
    config: ReplayConfig,
    /// 当前tick
    current_tick: u64,
    /// 快照列表
    snapshots: VecDeque<ReplaySnapshot>,
    /// 帧列表
    frames: VecDeque<ReplayFrame>,
    /// 开始时间
    start_time: SystemTime,
    /// 元数据
    metadata: HashMap<String, String>,
}

impl ReplayRecorder {
    /// 创建新的回放录制器
    pub fn new(config: ReplayConfig) -> Self {
        Self {
            config,
            current_tick: 0,
            snapshots: VecDeque::new(),
            frames: VecDeque::new(),
            start_time: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    /// 开始录制
    pub fn start_recording(&mut self) {
        self.current_tick = 0;
        self.snapshots.clear();
        self.frames.clear();
        self.start_time = SystemTime::now();
        self.metadata.clear();
    }

    /// 停止录制
    pub fn stop_recording(&mut self) {
        // 清理旧数据
        while self.snapshots.len() > self.config.max_snapshots {
            self.snapshots.pop_front();
        }
    }

    /// 录制帧
    pub fn record_frame(
        &mut self,
        tick: u64,
        delta_states: HashMap<u64, EntityState>,
        events: Vec<NetworkEvent>,
        inputs: Option<Vec<u8>>,
    ) {
        if !self.config.enable_recording {
            return;
        }

        self.current_tick = tick;

        // 创建帧
        let frame = ReplayFrame {
            tick,
            timestamp: current_timestamp_ms(),
            delta_states,
            events: if self.config.record_events {
                events
            } else {
                Vec::new()
            },
            inputs: if self.config.record_inputs {
                inputs.unwrap_or_default()
            } else {
                Vec::new()
            },
        };

        self.frames.push_back(frame);

        // 检查是否需要创建快照
        if tick.is_multiple_of(self.config.snapshot_interval) {
            self.create_snapshot(tick);
        }
    }

    /// 创建快照
    fn create_snapshot(&mut self, tick: u64) {
        // 从最近的帧重建完整状态
        let mut entity_states = HashMap::new();
        let mut events = Vec::new();

        // 从最后一个快照开始应用所有帧
        let last_snapshot_tick = self
            .snapshots
            .back()
            .map(|s| s.tick)
            .unwrap_or(0);

        for frame in self.frames.iter() {
            if frame.tick > last_snapshot_tick && frame.tick <= tick {
                // 应用增量状态
                for (entity_id, state) in &frame.delta_states {
                    entity_states.insert(*entity_id, state.clone());
                }

                // 收集事件
                events.extend(frame.events.iter().cloned());
            }
        }

        let snapshot = ReplaySnapshot {
            tick,
            timestamp: current_timestamp_ms(),
            entity_states,
            events,
        };

        self.snapshots.push_back(snapshot);

        // 限制快照数量
        while self.snapshots.len() > self.config.max_snapshots {
            self.snapshots.pop_front();
        }
    }

    /// 保存回放到文件
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), ReplayError> {
        let header = ReplayHeader {
            version: 1,
            start_time: self
                .start_time
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            end_time: current_timestamp_ms(),
            total_ticks: self.current_tick,
            snapshot_count: self.snapshots.len(),
            frame_count: self.frames.len(),
            metadata: self.metadata.clone(),
        };

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // 写入文件头
        let header_json = serde_json::to_string(&header)?;
        writer.write_all(header_json.as_bytes())?;
        writer.write_all(b"\n")?;

        // 写入快照
        for snapshot in &self.snapshots {
            let snapshot_json = serde_json::to_string(snapshot)?;
            writer.write_all(b"SNAPSHOT\n")?;
            writer.write_all(snapshot_json.as_bytes())?;
            writer.write_all(b"\n")?;
        }

        // 写入帧
        for frame in &self.frames {
            let frame_json = serde_json::to_string(frame)?;
            writer.write_all(b"FRAME\n")?;
            writer.write_all(frame_json.as_bytes())?;
            writer.write_all(b"\n")?;
        }

        writer.flush()?;
        Ok(())
    }

    /// 获取当前tick
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// 获取快照数量
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// 获取帧数量
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

/// 回放播放器
pub struct ReplayPlayer {
    config: ReplayConfig,
    /// 回放头
    header: ReplayHeader,
    /// 快照列表
    snapshots: Vec<ReplaySnapshot>,
    /// 帧列表
    frames: Vec<ReplayFrame>,
    /// 当前播放位置（tick）
    current_tick: u64,
    /// 播放速度（1.0 = 正常速度）
    playback_speed: f32,
    /// 是否正在播放
    is_playing: bool,
    /// 是否循环播放
    loop_playback: bool,
}

impl ReplayPlayer {
    /// 从文件加载回放
    pub fn load_from_file(path: &PathBuf) -> Result<Self, ReplayError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // 读取文件头
        let mut header_line = String::new();
        reader.read_line(&mut header_line)?;
        let header: ReplayHeader = serde_json::from_str(header_line.trim())?;

        let mut snapshots = Vec::new();
        let mut frames = Vec::new();

        // 读取快照和帧
        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            let trimmed = line.trim();
            if trimmed == "SNAPSHOT" {
                let mut snapshot_line = String::new();
                reader.read_line(&mut snapshot_line)?;
                let snapshot: ReplaySnapshot = serde_json::from_str(snapshot_line.trim())?;
                snapshots.push(snapshot);
            } else if trimmed == "FRAME" {
                let mut frame_line = String::new();
                reader.read_line(&mut frame_line)?;
                let frame: ReplayFrame = serde_json::from_str(frame_line.trim())?;
                frames.push(frame);
            }
            line.clear();
        }

        Ok(Self {
            config: ReplayConfig::default(),
            header,
            snapshots,
            frames,
            current_tick: 0,
            playback_speed: 1.0,
            is_playing: false,
            loop_playback: false,
        })
    }

    /// 获取回放配置
    pub fn get_config(&self) -> &ReplayConfig {
        &self.config
    }

    /// 设置回放配置
    pub fn set_config(&mut self, config: ReplayConfig) {
        self.config = config;
    }

    /// 开始播放
    pub fn start_playback(&mut self) {
        self.is_playing = true;
        self.current_tick = 0;
    }

    /// 停止播放
    pub fn stop_playback(&mut self) {
        self.is_playing = false;
    }

    /// 暂停播放
    pub fn pause_playback(&mut self) {
        self.is_playing = false;
    }

    /// 恢复播放
    pub fn resume_playback(&mut self) {
        self.is_playing = true;
    }

    /// 跳转到指定tick
    pub fn seek_to_tick(&mut self, tick: u64) -> Result<(), ReplayError> {
        if tick > self.header.total_ticks {
            return Err(ReplayError::InvalidTick(tick));
        }

        self.current_tick = tick;
        Ok(())
    }

    /// 跳转到指定时间（毫秒）
    pub fn seek_to_time(&mut self, time_ms: u64) -> Result<(), ReplayError> {
        // 估算tick（假设60 tick/秒）
        let estimated_tick = (time_ms as f64 / 1000.0 * 60.0) as u64;
        self.seek_to_tick(estimated_tick.min(self.header.total_ticks))
    }

    /// 获取当前帧的状态
    pub fn get_current_state(&self) -> Option<ReplayState> {
        // 找到最近的快照
        let snapshot = self
            .snapshots
            .iter()
            .filter(|s| s.tick <= self.current_tick)
            .max_by_key(|s| s.tick)?;

        // 应用从快照到当前tick的所有帧
        let mut entity_states = snapshot.entity_states.clone();
        let mut events = snapshot.events.clone();

        for frame in &self.frames {
            if frame.tick > snapshot.tick && frame.tick <= self.current_tick {
                // 应用增量状态
                for (entity_id, state) in &frame.delta_states {
                    entity_states.insert(*entity_id, state.clone());
                }

                // 收集事件
                events.extend(frame.events.iter().cloned());
            }
        }

        Some(ReplayState {
            tick: self.current_tick,
            entity_states,
            events,
        })
    }

    /// 更新播放（应该在游戏循环中调用）
    pub fn update(&mut self, delta_time: f32) {
        if !self.is_playing {
            return;
        }

        // 根据播放速度更新tick
        let tick_delta = (delta_time * 60.0 * self.playback_speed) as u64;
        self.current_tick += tick_delta;

        // 检查是否到达结尾
        if self.current_tick >= self.header.total_ticks {
            if self.loop_playback {
                self.current_tick = 0;
            } else {
                self.is_playing = false;
            }
        }
    }

    /// 设置播放速度
    pub fn set_playback_speed(&mut self, speed: f32) {
        self.playback_speed = speed.clamp(0.0, 10.0);
    }

    /// 设置循环播放
    pub fn set_loop(&mut self, loop_playback: bool) {
        self.loop_playback = loop_playback;
    }

    /// 获取当前tick
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// 获取总tick数
    pub fn total_ticks(&self) -> u64 {
        self.header.total_ticks
    }

    /// 获取播放进度（0.0 - 1.0）
    pub fn playback_progress(&self) -> f32 {
        if self.header.total_ticks == 0 {
            return 0.0;
        }
        (self.current_tick as f32 / self.header.total_ticks as f32).min(1.0)
    }

    /// 是否正在播放
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
}

/// 回放状态
#[derive(Debug, Clone)]
pub struct ReplayState {
    /// 当前tick
    pub tick: u64,
    /// 实体状态映射
    pub entity_states: HashMap<u64, EntityState>,
    /// 事件列表
    pub events: Vec<NetworkEvent>,
}

/// 时间旅行调试器
pub struct TimeTravelDebugger {
    /// 回放播放器
    player: ReplayPlayer,
    /// 断点列表（tick）
    breakpoints: Vec<u64>,
    /// 是否在调试模式
    is_debugging: bool,
    /// 单步执行模式
    step_mode: bool,
}

impl TimeTravelDebugger {
    /// 创建新的时间旅行调试器
    pub fn new(player: ReplayPlayer) -> Self {
        Self {
            player,
            breakpoints: Vec::new(),
            is_debugging: false,
            step_mode: false,
        }
    }

    /// 开始调试
    pub fn start_debugging(&mut self) {
        self.is_debugging = true;
        self.player.start_playback();
    }

    /// 停止调试
    pub fn stop_debugging(&mut self) {
        self.is_debugging = false;
        self.player.stop_playback();
    }

    /// 添加断点
    pub fn add_breakpoint(&mut self, tick: u64) {
        if !self.breakpoints.contains(&tick) {
            self.breakpoints.push(tick);
        }
    }

    /// 移除断点
    pub fn remove_breakpoint(&mut self, tick: u64) {
        self.breakpoints.retain(|&t| t != tick);
    }

    /// 单步执行
    pub fn step(&mut self) {
        self.step_mode = true;
        self.player.seek_to_tick(self.player.current_tick() + 1)
            .unwrap_or_default();
    }

    /// 单步后退
    pub fn step_back(&mut self) {
        if self.player.current_tick() > 0 {
            self.player.seek_to_tick(self.player.current_tick() - 1)
                .unwrap_or_default();
        }
    }

    /// 继续执行
    pub fn continue_execution(&mut self) {
        self.step_mode = false;
        self.player.resume_playback();
    }

    /// 更新调试器
    pub fn update(&mut self, delta_time: f32) {
        if !self.is_debugging {
            return;
        }

        // 检查断点
        if self.breakpoints.contains(&self.player.current_tick()) {
            self.player.pause_playback();
            return;
        }

        // 单步模式
        if self.step_mode {
            self.player.pause_playback();
            return;
        }

        // 正常更新
        self.player.update(delta_time);
    }

    /// 获取当前状态
    pub fn get_current_state(&self) -> Option<ReplayState> {
        self.player.get_current_state()
    }

    /// 跳转到tick
    pub fn seek_to_tick(&mut self, tick: u64) -> Result<(), ReplayError> {
        self.player.seek_to_tick(tick)
    }

    /// 获取播放器
    pub fn player(&self) -> &ReplayPlayer {
        &self.player
    }
}

/// 回放错误
#[derive(Debug)]
pub enum ReplayError {
    IoError(std::io::Error),
    SerializationError(String),
    InvalidTick(u64),
    InvalidFile(String),
}

impl From<std::io::Error> for ReplayError {
    fn from(err: std::io::Error) -> Self {
        ReplayError::IoError(err)
    }
}

impl From<serde_json::Error> for ReplayError {
    fn from(err: serde_json::Error) -> Self {
        ReplayError::SerializationError(err.to_string())
    }
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplayError::IoError(e) => write!(f, "IO error: {}", e),
            ReplayError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            ReplayError::InvalidTick(tick) => write!(f, "Invalid tick: {}", tick),
            ReplayError::InvalidFile(e) => write!(f, "Invalid file: {}", e),
        }
    }
}

impl std::error::Error for ReplayError {}

// 辅助函数
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_recorder() {
        use glam::{Quat, Vec3};
        let config = ReplayConfig::default();
        let mut recorder = ReplayRecorder::new(config);
        recorder.start_recording();

        let mut states = HashMap::new();
        let entity_state = EntityState::new(
            Vec3::new(0.0, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
            Vec3::ZERO,
        );
        states.insert(1, entity_state);

        recorder.record_frame(1, states, Vec::new(), None);
        assert_eq!(recorder.current_tick(), 1);
    }

    #[test]
    fn test_replay_player() {
        // 创建临时文件进行测试
        let temp_path = PathBuf::from("/tmp/test_replay.json");
        let config = ReplayConfig::default();
        let mut recorder = ReplayRecorder::new(config);
        recorder.start_recording();
        recorder.record_frame(1, HashMap::new(), Vec::new(), None);
        recorder.stop_recording();
        recorder.save_to_file(&temp_path).unwrap();

        let player = ReplayPlayer::load_from_file(&temp_path).unwrap();
        assert_eq!(player.total_ticks(), 1);
    }
}

