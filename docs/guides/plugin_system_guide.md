# 插件系统开发指南

## 概述

游戏引擎提供了灵活的插件系统，允许开发者扩展引擎功能而不修改核心代码。

## 插件架构

### 插件接口

```rust
use game_engine::plugins::Plugin;

pub trait Plugin {
    /// 插件名称
    fn name(&self) -> &str;
    
    /// 插件版本
    fn version(&self) -> &str;
    
    /// 初始化插件
    fn initialize(&mut self, engine: &mut Engine) -> Result<(), PluginError>;
    
    /// 更新插件（每帧调用）
    fn update(&mut self, engine: &mut Engine, delta_time: f32) -> Result<(), PluginError>;
    
    /// 清理插件
    fn cleanup(&mut self, engine: &mut Engine) -> Result<(), PluginError>;
}
```

### 创建插件

```rust
use game_engine::plugins::{Plugin, PluginError, Engine};

pub struct MyCustomPlugin {
    // 插件状态
}

impl Plugin for MyCustomPlugin {
    fn name(&self) -> &str {
        "MyCustomPlugin"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn initialize(&mut self, engine: &mut Engine) -> Result<(), PluginError> {
        // 注册系统、资源、事件处理器等
        engine.add_system(my_system);
        Ok(())
    }
    
    fn update(&mut self, engine: &mut Engine, delta_time: f32) -> Result<(), PluginError> {
        // 每帧更新逻辑
        Ok(())
    }
    
    fn cleanup(&mut self, engine: &mut Engine) -> Result<(), PluginError> {
        // 清理资源
        Ok(())
    }
}
```

## 插件注册

### 静态插件

```rust
use game_engine::plugins::PluginManager;

let mut plugin_manager = PluginManager::new();
plugin_manager.register(Box::new(MyCustomPlugin::new()));
plugin_manager.initialize_all(&mut engine)?;
```

### 动态插件（运行时加载）

```rust
use game_engine::plugins::DynamicPluginLoader;

let loader = DynamicPluginLoader::new();
let plugin = loader.load_plugin("path/to/plugin.so")?;
plugin_manager.register(plugin);
```

## 插件间通信

### 事件系统

```rust
// 插件A发布事件
event_bus.publish(MyEvent { data: 42 })?;

// 插件B订阅事件
event_bus.subscribe::<MyEvent>(|event| {
    println!("Received event: {:?}", event);
})?;
```

### 资源访问

```rust
// 访问共享资源
let resource = engine.world.get_resource::<MyResource>()?;
// 使用资源
```

## 最佳实践

1. **保持接口稳定**：避免频繁更改插件接口
2. **错误处理**：妥善处理插件错误，避免影响主引擎
3. **资源管理**：插件负责清理自己创建的资源
4. **版本兼容**：检查插件版本兼容性
5. **性能考虑**：避免插件阻塞主线程

## 相关文档

- [插件系统API文档](../../game_engine/src/plugins/mod.rs)

