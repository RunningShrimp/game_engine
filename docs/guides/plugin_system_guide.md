# 插件系统使用指南

## 概述

游戏引擎提供强大的插件系统，允许开发者通过插件扩展引擎功能。插件系统支持：

- **模块化架构**：按需加载功能模块
- **依赖管理**：自动处理插件依赖关系
- **热重载**：支持运行时动态加载和重载插件
- **生命周期管理**：完整的插件生命周期支持

## 插件接口

### EnginePlugin Trait

所有插件必须实现 `EnginePlugin` trait：

```rust
use game_engine::plugins::{EnginePlugin, PluginVersion, PluginDependency};
use bevy_ecs::prelude::*;

pub struct MyPlugin;

impl EnginePlugin for MyPlugin {
    fn name(&self) -> &'static str {
        "my_plugin"
    }

    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "我的自定义插件"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![
            PluginDependency {
                name: "render".to_string(),
                version_requirement: ">=1.0.0".to_string(),
            }
        ]
    }

    fn build(&self, app: &mut App) {
        // 注册资源和系统
        app.add_system(my_system);
        app.insert_resource(MyResource::default());
    }

    fn startup(&self, world: &mut World) {
        // 初始化运行时状态
        tracing::info!("MyPlugin started");
    }

    fn update(&self, world: &mut World) {
        // 每帧调用
    }

    fn shutdown(&self, world: &mut World) {
        // 清理资源
        tracing::info!("MyPlugin shutdown");
    }
}
```

## 扩展点

### 1. 系统注册

在 `build` 方法中注册ECS系统：

```rust
fn build(&self, app: &mut App) {
    app.add_system(my_update_system)
        .add_system(my_render_system);
}
```

### 2. 资源注册

在 `build` 方法中注册ECS资源：

```rust
fn build(&self, app: &mut App) {
    app.insert_resource(MyConfig::default())
        .insert_resource(MyState::new());
}
```

### 3. 组件注册

组件自动注册，无需显式注册：

```rust
#[derive(Component)]
pub struct MyComponent {
    pub value: f32,
}
```

### 4. 事件注册

注册自定义事件：

```rust
#[derive(Event)]
pub struct MyEvent {
    pub data: String,
}

fn build(&self, app: &mut App) {
    app.add_event::<MyEvent>();
}
```

## 使用插件

### 基本使用

```rust
use game_engine::plugins::{App, EnginePlugin};

let mut app = App::new();
app.add_plugin(MyPlugin);
app.build_plugins();
app.run_startup();

// 主循环
loop {
    app.update();
}
```

### 依赖管理

插件系统自动处理依赖关系：

```rust
// 插件A依赖插件B
struct PluginA;
impl EnginePlugin for PluginA {
    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![PluginDependency {
            name: "plugin_b".to_string(),
            version_requirement: ">=1.0.0".to_string(),
        }]
    }
    // ...
}

// 插件B
struct PluginB;
impl EnginePlugin for PluginB {
    fn name(&self) -> &'static str {
        "plugin_b"
    }
    // ...
}

// 使用：插件系统会自动按依赖顺序加载
let mut app = App::new();
app.add_plugin(PluginA); // 会自动先加载PluginB
app.add_plugin(PluginB);
app.build_plugins(); // 按依赖顺序构建
```

## 内置插件

引擎提供以下内置插件：

- **RenderPlugin**：渲染系统
- **PhysicsPlugin**：物理系统
- **AudioPlugin**：音频系统
- **UiPlugin**：UI系统
- **ScriptingPlugin**：脚本系统
- **XrPlugin**：XR支持
- **ScenePlugin**：场景管理
- **ResourcePlugin**：资源管理

### 使用内置插件

```rust
use game_engine::plugins::builtin::*;

let mut app = App::new();
app.add_plugin(RenderPlugin)
    .add_plugin(PhysicsPlugin)
    .add_plugin(AudioPlugin);
app.build_plugins();
```

## 热重载

插件系统支持运行时热重载：

```rust
use game_engine::plugins::hot_reload::HotReloadManager;

let mut hot_reload = HotReloadManager::new();
hot_reload.load_plugin("plugins/my_plugin.so")?;
hot_reload.reload_plugin("my_plugin")?;
```

### 热重载限制

- 插件必须编译为动态库（.so/.dll/.dylib）
- 某些资源（如GPU资源）无法热重载
- 需要重新初始化状态

## 插件配置

### 配置文件

插件可以通过配置文件管理：

```toml
# plugins.toml
[plugins.my_plugin]
enabled = true
priority = 100
path = "plugins/my_plugin.so"

[plugins.my_plugin.parameters]
key1 = "value1"
key2 = "value2"
```

### 使用配置管理器

```rust
use game_engine::plugins::config::PluginConfigManager;

let mut config_manager = PluginConfigManager::new("plugins.toml");
config_manager.load()?;

if config_manager.is_enabled("my_plugin") {
    // 加载插件
}
```

## 最佳实践

### 1. 单一职责

每个插件应该只负责一个功能领域：

```rust
// ✅ 好：职责单一
struct PhysicsPlugin;
struct RenderPlugin;

// ❌ 差：职责过多
struct GamePlugin; // 包含所有功能
```

### 2. 最小依赖

尽量减少插件依赖：

```rust
// ✅ 好：最小依赖
fn dependencies(&self) -> Vec<PluginDependency> {
    vec![] // 无依赖
}

// ❌ 差：过多依赖
fn dependencies(&self) -> Vec<PluginDependency> {
    vec![
        PluginDependency { name: "plugin1".to_string(), ... },
        PluginDependency { name: "plugin2".to_string(), ... },
        // ... 太多依赖
    ]
}
```

### 3. 资源清理

在 `shutdown` 方法中清理资源：

```rust
fn shutdown(&self, world: &mut World) {
    // 清理资源
    world.remove_resource::<MyResource>();
    
    // 清理实体
    let mut query = world.query::<Entity, With<MyComponent>>();
    for entity in query.iter(world) {
        world.despawn(entity);
    }
}
```

### 4. 错误处理

使用 `Result` 类型处理错误：

```rust
fn startup(&self, world: &mut World) -> Result<(), PluginError> {
    // 初始化逻辑
    Ok(())
}
```

## 扩展示例

### 示例1：自定义渲染插件

```rust
pub struct CustomRenderPlugin;

impl EnginePlugin for CustomRenderPlugin {
    fn name(&self) -> &'static str {
        "custom_render"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![PluginDependency {
            name: "render".to_string(),
            version_requirement: ">=1.0.0".to_string(),
        }]
    }

    fn build(&self, app: &mut App) {
        app.add_system(custom_render_system);
    }
}

fn custom_render_system(query: Query<&CustomRenderComponent>) {
    // 自定义渲染逻辑
}
```

### 示例2：AI行为插件

```rust
pub struct AIBehaviorPlugin;

impl EnginePlugin for AIBehaviorPlugin {
    fn name(&self) -> &'static str {
        "ai_behavior"
    }

    fn build(&self, app: &mut App) {
        app.add_system(ai_update_system)
            .insert_resource(AIState::default());
    }

    fn startup(&self, world: &mut World) {
        // 初始化AI系统
        tracing::info!("AI Behavior Plugin started");
    }
}
```

### 示例3：网络多人插件

```rust
pub struct MultiplayerPlugin;

impl EnginePlugin for MultiplayerPlugin {
    fn name(&self) -> &'static str {
        "multiplayer"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![PluginDependency {
            name: "network".to_string(),
            version_requirement: ">=1.0.0".to_string(),
        }]
    }

    fn build(&self, app: &mut App) {
        app.add_system(network_sync_system)
            .add_event::<PlayerJoinEvent>()
            .add_event::<PlayerLeaveEvent>();
    }
}
```

## 故障排除

### 插件加载失败

- 检查插件依赖是否满足
- 检查插件版本兼容性
- 查看日志输出

### 热重载失败

- 确保插件编译为动态库
- 检查文件权限
- 确保没有资源泄漏

### 性能问题

- 减少插件数量
- 优化插件更新逻辑
- 使用条件编译禁用不需要的插件

## 总结

插件系统提供了强大的扩展能力，允许开发者：

1. **模块化开发**：将功能拆分为独立插件
2. **按需加载**：只加载需要的功能
3. **热重载**：快速迭代开发
4. **依赖管理**：自动处理依赖关系

通过遵循最佳实践，可以构建可维护、可扩展的游戏引擎应用。

