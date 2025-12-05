# 动画系统 API 参考

## AnimationClip

动画剪辑，包含多个关键帧轨道。

### 示例

```rust
use game_engine::animation::{AnimationClip, KeyframeTrack, InterpolationMode};
use glam::{Vec3, Quat};

let mut clip = AnimationClip::new("walk_animation".to_string(), 5.0);

// 添加位置轨道
let mut position_track = KeyframeTrack::new(InterpolationMode::Linear);
position_track.add_keyframe(0.0, Vec3::ZERO);
position_track.add_keyframe(2.5, Vec3::new(5.0, 0.0, 0.0));
position_track.add_keyframe(5.0, Vec3::new(10.0, 0.0, 0.0));
clip.add_track("position".to_string(), position_track);

// 添加旋转轨道
let mut rotation_track = KeyframeTrack::new(InterpolationMode::Slerp);
rotation_track.add_keyframe(0.0, Quat::IDENTITY);
rotation_track.add_keyframe(5.0, Quat::from_rotation_y(std::f32::consts::PI));
clip.add_track("rotation".to_string(), rotation_track);
```

## AnimationPlayer

动画播放器，用于播放动画剪辑。

### 示例

```rust
use game_engine::animation::{AnimationPlayer, AnimationClip};

let clip = AnimationClip::new("idle".to_string(), 2.0);
let mut player = AnimationPlayer::new(clip);

// 更新动画
player.update(delta_time);

// 获取当前值
let position = player.get_value::<Vec3>("position")?;
```

## 插值模式

- `Linear` - 线性插值
- `Slerp` - 球面线性插值（用于旋转）
- `Step` - 步进插值（无插值）
- `Cubic` - 三次样条插值

