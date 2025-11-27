# 动画系统

引擎提供完整的动画系统，支持骨骼动画、逐帧动画、程序动画等。

## 动画架构

### 核心组件

```rust
#[derive(Component)]
pub struct Flipbook {
    pub frames: Vec<FlipFrame>,
    pub speed: f32,
    pub looping: bool,
    pub elapsed: f32,
    pub current: usize,
}

#[derive(Clone, Debug)]
pub struct FlipFrame {
    pub uv_off: [f32; 2],
    pub uv_scale: [f32; 2],
    pub duration: f32,
}
```

### 动画系统

```rust
pub fn flipbook_system(mut query: Query<(&mut Sprite, &mut Flipbook)>, time: Res<Time>) {
    for (mut sprite, mut fb) in query.iter_mut() {
        if fb.frames.is_empty() { continue; }
        fb.elapsed += time.delta_seconds * fb.speed;
        let mut t = fb.elapsed;
        let mut idx = fb.current;
        while t > fb.frames[idx].duration {
            t -= fb.frames[idx].duration;
            idx += 1;
            if idx >= fb.frames.len() {
                if fb.looping { idx = 0; } else { idx = fb.frames.len() - 1; break; }
            }
        }
        fb.current = idx;
        let fr = &fb.frames[idx];
        sprite.uv_off = fr.uv_off;
        sprite.uv_scale = fr.uv_scale;
    }
}
```

## 基本使用

### 创建逐帧动画

```rust
// 创建动画帧
let frames = vec![
    FlipFrame {
        uv_off: [0.0, 0.0],
        uv_scale: [0.5, 0.25],
        duration: 0.1,
    },
    FlipFrame {
        uv_off: [0.5, 0.0],
        uv_scale: [0.5, 0.25],
        duration: 0.1,
    },
    // 更多帧...
];

commands.spawn((
    Transform::default(),
    Sprite {
        tex_index: 0,
        layer: 0.0,
        ..default()
    },
    Flipbook {
        frames,
        speed: 1.0,
        looping: true,
        elapsed: 0.0,
        current: 0,
    },
));
```

## 高级动画

### 骨骼动画支持

引擎预留了骨骼动画接口，计划支持：

- 骨骼层次结构
- 蒙皮网格
- 动画混合
- IK解算

### 程序动画

通过修改Transform组件实现：

```rust
fn program_animation(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<ProgramAnimation>>,
) {
    for mut transform in query.iter_mut() {
        let t = time.elapsed_seconds as f32;

        // 旋转动画
        transform.rotation = Quat::from_rotation_z(t.sin() * 0.5);

        // 缩放动画
        let scale = 1.0 + (t * 2.0).sin() * 0.2;
        transform.scale = Vec3::splat(scale);
    }
}
```

## 性能优化

### 动画池化

```rust
struct AnimationPool {
    animations: HashMap<String, Flipbook>,
}

impl AnimationPool {
    pub fn get(&self, name: &str) -> Option<&Flipbook> {
        self.animations.get(name)
    }
}
```

### 批处理渲染

相同动画的精灵会自动批处理渲染，减少Draw Call数量。

## 最佳实践

1. **预加载动画**: 在游戏开始时加载所有动画资源
2. **复用动画**: 相同动画可以被多个实体共享
3. **LOD动画**: 根据距离调整动画复杂度
4. **内存优化**: 压缩动画纹理数据

## 调试和诊断

### 动画调试工具

- 动画状态检查器
- 帧时间可视化
- 纹理坐标调试

### 性能监控

```rust
struct AnimationStats {
    active_animations: usize,
    total_frames: usize,
    skipped_frames: usize,
    memory_usage: usize,
}
```

## 未来计划

- 骨骼动画系统
- 动画混合树
- 程序化动画生成
- 物理动画混合
- 面部动画系统