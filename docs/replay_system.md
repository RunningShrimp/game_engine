# 网络回放系统指南

## 概述

本文档介绍游戏引擎的网络回放系统，提供游戏状态的录制、回放和时间旅行调试功能。

## 功能特性

- **状态录制**: 录制游戏状态和事件
- **事件录制**: 录制网络事件
- **回放控制**: 播放、暂停、快进、快退
- **时间旅行调试**: 在时间线上前后移动，设置断点
- **回放文件管理**: 保存和加载回放文件

## 录制系统

### 使用方法

```rust
use game_engine::network::{ReplayRecorder, ReplayConfig};
use game_engine::network::synchronization::{EntityState, NetworkEvent};
use glam::{Quat, Vec3};
use std::collections::HashMap;
use std::path::PathBuf;

// 创建配置
let config = ReplayConfig {
    enable_recording: true,
    enable_playback: false,
    snapshot_interval: 60, // 每60 tick一个快照
    max_snapshots: 1000,
    record_events: true,
    record_inputs: true,
    compression_level: 6,
};

// 创建录制器
let mut recorder = ReplayRecorder::new(config);

// 开始录制
recorder.start_recording();

// 在游戏循环中录制帧
let mut delta_states = HashMap::new();
delta_states.insert(
    entity_id,
    EntityState::new(
        Vec3::new(1.0, 0.0, 0.0),
        Quat::IDENTITY,
        Vec3::ONE,
        Vec3::ZERO,
    ),
);

let events = vec![/* ... */];
let inputs = Some(input_data);

recorder.record_frame(current_tick, delta_states, events, inputs);

// 停止录制
recorder.stop_recording();

// 保存回放文件
let path = PathBuf::from("replay.json");
recorder.save_to_file(&path)?;
```

### 配置选项

- `enable_recording`: 是否启用录制
- `snapshot_interval`: 快照间隔（tick数）
- `max_snapshots`: 最大快照数量
- `record_events`: 是否录制事件
- `record_inputs`: 是否录制输入
- `compression_level`: 压缩级别（0-9）

## 回放系统

### 使用方法

```rust
use game_engine::network::ReplayPlayer;
use std::path::PathBuf;

// 从文件加载回放
let path = PathBuf::from("replay.json");
let mut player = ReplayPlayer::load_from_file(&path)?;

// 开始播放
player.start_playback();

// 在游戏循环中更新
loop {
    let delta_time = get_delta_time();
    player.update(delta_time);

    // 获取当前状态
    if let Some(state) = player.get_current_state() {
        // 应用状态到游戏世界
        apply_state_to_world(&state);
    }

    // 检查是否播放完成
    if !player.is_playing() {
        break;
    }
}

// 控制播放
player.pause_playback();
player.resume_playback();
player.set_playback_speed(2.0); // 2倍速
player.set_loop(true); // 循环播放

// 跳转
player.seek_to_tick(1000)?;
player.seek_to_time(5000)?; // 跳转到5秒
```

### 播放控制

- `start_playback()`: 开始播放
- `stop_playback()`: 停止播放
- `pause_playback()`: 暂停播放
- `resume_playback()`: 恢复播放
- `seek_to_tick(tick)`: 跳转到指定tick
- `seek_to_time(time_ms)`: 跳转到指定时间
- `set_playback_speed(speed)`: 设置播放速度
- `set_loop(loop)`: 设置循环播放

## 时间旅行调试

### 使用方法

```rust
use game_engine::network::{ReplayPlayer, TimeTravelDebugger};
use std::path::PathBuf;

// 加载回放
let path = PathBuf::from("replay.json");
let player = ReplayPlayer::load_from_file(&path)?;

// 创建调试器
let mut debugger = TimeTravelDebugger::new(player);

// 开始调试
debugger.start_debugging();

// 添加断点
debugger.add_breakpoint(1000);
debugger.add_breakpoint(2000);

// 在游戏循环中更新
loop {
    let delta_time = get_delta_time();
    debugger.update(delta_time);

    // 检查是否在断点处暂停
    if !debugger.player().is_playing() {
        // 在断点处，可以检查状态
        if let Some(state) = debugger.get_current_state() {
            inspect_state(&state);
        }

        // 单步执行
        debugger.step();
        // 或继续执行
        debugger.continue_execution();
    }
}

// 单步操作
debugger.step(); // 前进一步
debugger.step_back(); // 后退一步

// 跳转
debugger.seek_to_tick(500)?;
```

### 调试功能

- **断点**: 在指定tick处暂停
- **单步执行**: 一次执行一个tick
- **单步后退**: 一次后退一个tick
- **继续执行**: 从当前位置继续播放
- **跳转**: 跳转到任意tick

## 回放文件格式

回放文件使用JSON格式，包含：

1. **文件头**: 元数据和统计信息
2. **快照**: 完整状态快照
3. **帧**: 增量状态更新

### 文件结构

```json
{
  "version": 1,
  "start_time": 1234567890,
  "end_time": 1234567890,
  "total_ticks": 3600,
  "snapshot_count": 60,
  "frame_count": 3600,
  "metadata": {
    "game_version": "1.0.0",
    "map": "test_map"
  }
}
SNAPSHOT
{
  "tick": 0,
  "timestamp": 1234567890,
  "entity_states": {...},
  "events": [...]
}
FRAME
{
  "tick": 1,
  "timestamp": 1234567891,
  "delta_states": {...},
  "events": [...],
  "inputs": [...]
}
...
```

## 性能优化建议

### 录制优化

1. **快照间隔**: 较大的间隔减少文件大小但降低精度
2. **最大快照数**: 限制快照数量以控制内存使用
3. **压缩级别**: 较高的压缩级别减少文件大小但增加CPU使用
4. **选择性录制**: 只录制必要的事件和状态

### 回放优化

1. **预加载**: 预加载回放文件到内存
2. **快照查找**: 使用二分查找快速定位快照
3. **增量应用**: 只应用从快照到目标tick的增量更新
4. **缓存**: 缓存最近访问的状态

## 硬件要求

### 录制

- **内存**: 每个快照约几KB到几MB（取决于实体数量）
- **存储**: 回放文件通常几MB到几GB（取决于录制时长）
- **CPU**: 录制开销很小（主要是序列化）

### 回放

- **内存**: 需要加载整个回放文件到内存
- **CPU**: 回放开销很小（主要是反序列化和状态应用）
- **存储**: 需要足够的存储空间保存回放文件

## 限制和注意事项

1. **文件大小**: 长时间录制会产生大文件
2. **内存使用**: 回放需要将整个文件加载到内存
3. **版本兼容性**: 回放文件格式可能随版本变化
4. **确定性**: 回放需要确定性的游戏逻辑

## 使用场景

1. **调试**: 重现和调试网络问题
2. **测试**: 自动化测试和回归测试
3. **分析**: 分析游戏性能和平衡性
4. **演示**: 录制和回放游戏演示
5. **回放系统**: 玩家可以观看游戏回放

## 未来计划

- [ ] 压缩回放文件
- [ ] 增量回放文件格式
- [ ] 回放文件版本管理
- [ ] 可视化回放编辑器
- [ ] 网络回放同步（多人回放）

## 更多信息

- [网络同步系统](./network_sync.md)
- [客户端预测](./client_prediction.md)
- [网络API参考](../api_reference.md)

