# 游戏引擎教程文档索引

**版本**: v1.0
**日期**: 2025-12-31
**目标**: 为开发者提供全面的学习资源

---

## 教程概览

| 教程 | 主题 | 预计时间 | 难度 |
|------|------|---------|------|
| [快速开始](#1-快速开始) | 创建第一个游戏 | 30分钟 | 初级 |
| [ECS系统](#2-ecs系统) | 实体组件系统 | 1小时 | 中级 |
| [渲染系统](#3-渲染系统) | PBR渲染和材质 | 2小时 | 中级 |
| [脚本系统](#4-脚本系统) | JavaScript/Python集成 | 1.5小时 | 中级 |
| [资源管理](#5-资源管理) | 加载和管理资源 | 1小时 | 初级 |
| [物理系统](#6-物理系统) | 碰撞检测和物理模拟 | 2小时 | 高级 |
| [音频系统](#7-音频系统) | 音效和音乐 | 1小时 | 初级 |
| [性能优化](#8-性能优化) | 性能分析和优化 | 2小时 | 高级 |
| [编辑器使用](#9-编辑器使用) | 可视化编辑器 | 1.5小时 | 中级 |
| [发布游戏](#10-发布游戏) | 打包和发布 | 1小时 | 中级 |

---

## 1. 快速开始

### 目标
在30分钟内创建一个简单的3D游戏

### 涵盖内容
- 引擎初始化
- 创建3D场景
- 添加玩家控制
- 基础碰撞检测
- 简单UI

### 代码示例
```rust
use game_engine::core::Engine;
use game_engine::render::Mesh3D;
use game_engine::platform::InputEvent;

fn main() {
    let mut engine = Engine::new();

    // 创建3D立方体
    let cube = engine.create_mesh(Mesh3D::cube(1.0));
    engine.spawn_entity(cube);

    // 运行游戏循环
    engine.run(|state| {
        // 游戏逻辑
    });
}
```

**详细文档**: [TUTORIAL_01_QUICKSTART.md](TUTORIAL_01_QUICKSTART.md)

---

## 2. ECS系统

### 目标
深入理解实体组件系统

### 涵盖内容
- Entity（实体）概念
- Component（组件）设计
- System（系统）实现
- Query（查询）使用
- 事件系统

### 代码示例
```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

#[derive(Component)]
struct Player {
    speed: f32,
}

fn health_system(query: Query<&Health>) {
    for health in query.iter() {
        println!("Health: {}/{}", health.current, health.max);
    }
}
```

**详细文档**: [TUTORIAL_02_ECS.md](TUTORIAL_02_ECS.md)

---

## 3. 渲染系统

### 目标
掌握PBR渲染和材质系统

### 涵盖内容
- PBR材质参数
- 纹理系统
- 光照设置
- Shadow映射
- 后处理效果

### 代码示例
```rust
use game_engine::render::{PbrMaterial, PbrTextures};

fn create_pbr_material() -> PbrMaterial {
    PbrMaterial {
        base_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
        metallic: 0.0,
        roughness: 0.5,
        ambient_occlusion: 1.0,
        ..Default::default()
    }
}
```

**详细文档**: [TUTORIAL_03_RENDERING.md](TUTORIAL_03_RENDERING.md)

---

## 4. 脚本系统

### 目标
集成JavaScript和Python脚本

### 涵盖内容
- JavaScript API
- Python API
- 脚本热重载
- 脚本调试
- 性能考虑

### 代码示例
```javascript
// JavaScript示例
const entity = Entity.create('Player', 0, 0, 0);
Entity.setPosition(entity, 10, 0, 5);
const light = Light.create('POINT', 'MainLight');
Light.setColor(light, 1.0, 1.0, 1.0);
```

```python
# Python示例
entity = Entity.create('Player', 0, 0, 0)
Entity.set_position(entity, 10, 0, 5)
light = Light.create('POINT', 'MainLight')
Light.set_color(light, 1.0, 1.0, 1.0)
```

**详细文档**: [TUTORIAL_04_SCRIPTING.md](TUTORIAL_04_SCRIPTING.md)

---

## 5. 资源管理

### 目标
高效加载和管理游戏资源

### 涵盖内容
- 资源加载
- 资源缓存
- 异步加载
- 资源热重载
- 资源压缩

### 代码示例
```rust
use game_engine::resources::ResourceManager;

async fn load_scene(path: &str) -> Result<Scene> {
    let manager = ResourceManager::new();

    // 异步加载GLTF模型
    let model = manager.load_gltf(path).await?;

    Ok(model)
}
```

**详细文档**: [TUTORIAL_05_RESOURCES.md](TUTORIAL_05_RESOURCES.md)

---

## 6. 物理系统

### 目标
实现真实的物理模拟

### 涵盖内容
- 刚体动力学
- 碰撞检测
- 物理材质
- 约束和关节
- 物理性能优化

### 代码示例
```rust
use game_engine::physics::{RigidBody, Collider};

fn create_physics_entity() {
    let body = RigidBody::dynamic()
        .with_mass(1.0)
        .with_position(Vec3::new(0, 10, 0));

    let collider = Collider::box_shape(Vec3::ONE);

    entity.insert(body);
    entity.insert(collider);
}
```

**详细文档**: [TUTORIAL_06_PHYSICS.md](TUTORIAL_06_PHYSICS.md)

---

## 7. 音频系统

### 目标
添加音效和音乐

### 涵盖内容
- 音频加载
- 3D空间音频
- 音频效果
- 音乐播放
- 音频优化

### 代码示例
```rust
use game_engine::audio::{AudioSource, Sound};

fn play_footstep_sound() {
    let sound = Sound::load("sounds/footstep.ogg");
    let source = AudioSource::new();
    source.play(sound);
}
```

**详细文档**: [TUTORIAL_07_AUDIO.md](TUTORIAL_07_AUDIO.md)

---

## 8. 性能优化

### 目标
优化游戏性能

### 涵盖内容
- 性能分析工具
- LOD系统
- 批处理优化
- 内存优化
- 渲染优化

### 代码示例
```rust
use game_engine::performance::{PerformanceProfiler, OptimizationSuggestion};

fn profile_game() {
    let profiler = PerformanceProfiler::new();
    profiler.start_recording();

    // 运行游戏...

    let bottlenecks = profiler.stop_recording();
    for bottleneck in bottlenecks {
        println!("{}: {}", bottleneck.category, bottleneck.description);
    }
}
```

**详细文档**: [TUTORIAL_08_PERFORMANCE.md](TUTORIAL_08_PERFORMANCE.md)

---

## 9. 编辑器使用

### 目标
使用可视化编辑器开发游戏

### 涵盖内容
- 编辑器界面
- 场景编辑
- 材质编辑器
- 动画工具
- 性能面板

### 代码示例
```rust
use game_engine::editor::Editor;

fn launch_editor() {
    let mut editor = Editor::new();

    // 加载项目
    editor.load_project("examples/platformer");

    // 运行编辑器
    editor.run();
}
```

**详细文档**: [TUTORIAL_09_EDITOR.md](TUTORIAL_09_EDITOR.md)

---

## 10. 发布游戏

### 目标
打包和发布游戏

### 涵盖内容
- 桌面平台打包
- Web平台构建
- 移动平台打包
- 资源优化
- 发布流程

### 代码示例
```bash
# 桌面平台
cargo build --release

# Web平台
cargo build --release --target wasm32-unknown-unknown

# Android
cargo apk build --release
```

**详细文档**: [TUTORIAL_10_PUBLISHING.md](TUTORIAL_10_PUBLISHING.md)

---

## 学习路径建议

### 初学者路径
1. 快速开始 → 2. ECS系统 → 5. 资源管理 → 9. 编辑器使用

### 中级开发者路径
1. 渲染系统 → 4. 脚本系统 → 7. 音频系统 → 8. 性能优化

### 高级开发者路径
1. 物理系统 → 8. 性能优化（高级） → 10. 发布游戏

---

## 实战项目

### 项目1: 3D平台跳跃游戏
**难度**: ⭐⭐
**时间**: 4-6小时
**涉及教程**: 1, 2, 5, 9

### 项目2: 第一人称射击游戏
**难度**: ⭐⭐⭐
**时间**: 8-12小时
**涉及教程**: 1, 2, 3, 5, 6, 7

### 项目3: 开放世界RPG
**难度**: ⭐⭐⭐⭐⭐
**时间**: 20-30小时
**涉及教程**: 全部

---

## 社区资源

### 官方资源
- GitHub仓库: [game-engine](https://github.com/your-org/game-engine)
- 文档网站: [docs.gameengine.dev](https://docs.gameengine.dev)
- API文档: [api.gameengine.dev](https://api.gameengine.dev)

### 社区教程
- 社区贡献的教程列表
- YouTube视频教程
- 示例项目集合

---

## 贡献指南

欢迎社区贡献新教程！请参考：
- 贡献指南: [CONTRIBUTING.md](CONTRIBUTING.md)
- 教程模板: [TUTORIAL_TEMPLATE.md](TUTORIAL_TEMPLATE.md)

---

**文档维护**: 游戏引擎文档团队
**最后更新**: 2025-12-31
**版本**: v1.0
