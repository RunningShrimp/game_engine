# 音频系统 API 参考

## AudioSource

音频源，表示一个音频资源。

### 示例

```rust
use game_engine::domain::audio::{AudioSource, AudioSourceId};

let source = AudioSource::from_file(
    AudioSourceId(1),
    "sound.mp3"
)?;

source.play()?;
source.set_volume(0.5)?;
source.pause()?;
source.stop()?;
```

## AudioDomainService

音频领域服务，管理音频播放。

### 示例

```rust
use game_engine::domain::audio::AudioDomainService;

let mut service = AudioDomainService::new();

// 加载音频
let source_id = service.load_audio("music.mp3")?;

// 播放音频
service.play(source_id)?;

// 设置音量
service.set_volume(source_id, 0.7)?;
```

## 空间音频

### SpatialAudio

空间音频支持，基于3D位置播放音频。

### 示例

```rust
use game_engine::audio::spatial::SpatialAudio;
use glam::Vec3;

let spatial_audio = SpatialAudio::new(
    source_id,
    Vec3::new(0.0, 0.0, 0.0), // 位置
    10.0, // 最大距离
)?;

spatial_audio.update_listener_position(Vec3::new(5.0, 0.0, 0.0))?;
```

