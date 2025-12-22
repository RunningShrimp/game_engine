# 服务层使用指南

## 概述

服务层提供应用级别的服务，协调领域对象和基础设施层。服务层遵循以下原则：

1. **单一职责**：每个服务只负责一个业务领域
2. **无状态**：服务本身不持有业务状态，状态由领域对象管理
3. **协调者**：服务负责协调领域对象之间的交互
4. **依赖注入**：服务通过依赖注入获取所需资源

## 可用服务

### RenderService（渲染服务）

负责协调渲染相关的业务逻辑，包括：
- 场景构建和更新
- LOD配置和管理
- 视锥体剔除
- 渲染命令生成

**职责边界**：
- ✅ 协调RenderScene聚合根
- ✅ 管理LOD选择器配置
- ✅ 更新视锥体
- ❌ 不包含具体渲染逻辑（由领域对象处理）
- ❌ 不直接操作GPU资源（由基础设施层处理）

**使用示例**：
```rust
use game_engine::services::render::RenderService;
use bevy_ecs::prelude::*;

let mut render_service = RenderService::new();
render_service.use_default_lod();
render_service.update_frustum(view_proj);
render_service.build_domain_scene(&mut world)?;
render_service.update_scene(0.016, camera_pos)?;
```

### AudioService（音频服务）

负责音频播放管理，包括：
- 音频流管理
- 播放控制（播放、暂停、停止）
- 音量控制

**职责边界**：
- ✅ 管理音频流生命周期
- ✅ 控制播放状态
- ❌ 不包含音频解码逻辑（由基础设施层处理）
- ❌ 不包含音频资源管理（由AssetServer处理）

**使用示例**：
```rust
use game_engine::services::audio::AudioService;

if let Some(mut audio) = AudioService::new() {
    audio.play_sound("bgm", "assets/music.ogg", 0.8, true);
    audio.set_volume("bgm", 0.5);
    audio.pause_sound("bgm");
}
```

### ScriptingService（脚本服务）

负责JavaScript脚本执行，包括：
- 运行时管理
- API绑定
- 脚本执行

**职责边界**：
- ✅ 管理JavaScript运行时
- ✅ 提供API绑定
- ❌ 不包含脚本资源管理（由AssetServer处理）
- ❌ 不包含脚本编译逻辑（由QuickJS处理）

**使用示例**：
```rust
use game_engine::services::scripting::ScriptingService;

let service = ScriptingService::new();
service.bind_core_api();
service.execute("print('Hello from script!');");
```

## 服务设计原则

### 1. 单一职责原则

每个服务只负责一个业务领域。如果服务职责过多，应该拆分为多个服务。

**示例**：
- ✅ `RenderService` - 只负责渲染协调
- ✅ `AudioService` - 只负责音频播放
- ❌ `GameService` - 包含所有游戏逻辑（应该拆分）

### 2. 无状态原则

服务本身不持有业务状态，状态由领域对象管理。

**正确**：
```rust
// 状态在领域对象中
let mut scene = RenderScene::new();
scene.add_object(object);

// 服务只负责协调
service.update_scene(&mut scene)?;
```

**错误**：
```rust
// ❌ 服务不应该持有业务状态
struct RenderService {
    objects: Vec<RenderObject>, // 错误：状态应该在领域对象中
}
```

### 3. 依赖注入

服务通过构造函数或方法参数接收依赖，而不是直接创建。

**正确**：
```rust
impl RenderService {
    pub fn new() -> Self {
        Self {
            render_scene: RenderScene::new(), // 领域对象
            ..Default::default()
        }
    }
}
```

**错误**：
```rust
// ❌ 服务不应该直接创建基础设施资源
impl RenderService {
    pub fn new() -> Self {
        let device = wgpu::Device::new(); // 错误：应该通过依赖注入
        Self { device }
    }
}
```

## 扩展服务

### 添加新服务

1. 在 `game_engine/src/services/mod.rs` 中添加模块声明
2. 创建服务文件 `game_engine/src/services/your_service.rs`
3. 实现服务结构体和方法
4. 添加文档和使用示例
5. 添加单元测试

### 服务接口文档

每个服务应该包含：
- 服务职责说明
- 使用示例
- 依赖说明
- 错误处理说明

## 最佳实践

1. **保持服务轻量**：服务应该是薄薄的协调层，业务逻辑在领域对象中
2. **使用领域对象**：通过领域对象的方法执行业务逻辑
3. **错误处理**：使用Result类型返回错误，不要panic
4. **文档完善**：为每个公共方法添加文档注释
5. **测试覆盖**：为服务添加单元测试和集成测试

