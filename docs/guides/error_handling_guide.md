# 错误处理指南

本文档说明游戏引擎中错误处理的最佳实践和统一模式。

## 概述

游戏引擎使用统一的错误处理系统，基于`thiserror`和`EngineError`类型，提供：
- 类型安全的错误传播
- 错误恢复策略
- 错误监控和统计
- 结构化错误报告

## 错误类型层次结构

### 核心错误类型

```rust
EngineError
├── Render(RenderError)
├── Physics(PhysicsError)
├── Audio(AudioError)
├── Resource(ResourceError)
├── Input(InputError)
├── System(SystemError)
├── General { message, source, severity, location, backtrace }
├── Multiple { count, errors, primary }
└── Chain { context, source, metadata }
```

### 模块级错误类型

每个子系统都有专门的错误类型：

- **RenderError**: 渲染系统错误（GPU操作、着色器编译、纹理创建等）
- **PhysicsError**: 物理系统错误（碰撞检测、物理模拟等）
- **AudioError**: 音频系统错误（音频加载、播放等）
- **ResourceError**: 资源管理错误（文件加载、资源解析等）
- **InputError**: 输入系统错误（输入设备、输入映射等）
- **SystemError**: 系统级错误（内存分配、线程操作等）

## 错误处理模式

### 1. 使用Result类型

**✅ 正确**：
```rust
pub fn load_texture(path: &str) -> Result<Texture, RenderError> {
    // 可能失败的操作
    let data = std::fs::read(path)
        .map_err(|e| RenderError::TextureCreation {
            texture: path.to_string(),
            message: format!("Failed to read file: {}", e),
            severity: ErrorSeverity::Error,
        })?;
    
    // 处理数据...
    Ok(texture)
}
```

**❌ 错误**：
```rust
pub fn load_texture(path: &str) -> Texture {
    let data = std::fs::read(path).unwrap(); // 不要使用unwrap
    // ...
}
```

### 2. 错误传播

使用`?`操作符传播错误：

```rust
pub fn load_scene(path: &str) -> Result<Scene, EngineError> {
    let data = load_file(path)?;  // 自动转换为EngineError
    let scene = parse_scene(&data)?;
    Ok(scene)
}
```

### 3. 错误上下文

使用`context`方法添加上下文信息：

```rust
use crate::error::EngineError;

pub fn process_entity(entity_id: u64) -> Result<(), EngineError> {
    let entity = find_entity(entity_id)
        .ok_or_else(|| EngineError::General {
            message: format!("Entity {} not found", entity_id),
            source: None,
            severity: ErrorSeverity::Error,
            location: Some(file!().to_string()),
            backtrace: None,
        })?;
    
    // 处理实体...
    Ok(())
}
```

### 4. 错误恢复策略

使用`RecoveryStrategy`定义错误恢复行为：

```rust
use crate::error::{RecoveryStrategy, RecoveryManager};

let recovery_manager = RecoveryManager::new();
recovery_manager.register_strategy(
    ErrorCategory::Resource,
    RecoveryStrategy::Retry {
        max_attempts: 3,
        base_delay_ms: 100,
        backoff_multiplier: 2.0,
        max_delay_ms: 1000,
    }
);
```

## 错误严重级别

使用`ErrorSeverity`枚举表示错误的严重程度：

- **Info**: 信息性消息，不影响系统运行
- **Warning**: 警告，可能影响性能或用户体验
- **Error**: 错误，影响部分功能但系统可继续运行
- **Critical**: 严重错误，影响核心功能
- **Fatal**: 致命错误，系统无法继续运行

## 错误监控

使用`ErrorMonitor`监控错误：

```rust
use crate::error::{ErrorMonitor, MonitorConfig};

let monitor = ErrorMonitor::new(MonitorConfig::default());
monitor.record_error(&error);
let stats = monitor.get_stats();
```

## 最佳实践

### ✅ 应该做的

1. **使用Result类型返回错误**
   ```rust
   pub fn operation() -> Result<(), EngineError>
   ```

2. **提供清晰的错误消息**
   ```rust
   Err(EngineError::General {
       message: "Failed to load texture: file not found".to_string(),
       // ...
   })
   ```

3. **使用错误恢复策略**
   ```rust
   let result = retry_executor.execute_with_recovery(|| {
       load_resource(path)
   });
   ```

4. **记录错误上下文**
   ```rust
   error.with_context("loading scene", "scene.json")
   ```

### ❌ 不应该做的

1. **不要使用unwrap()**
   ```rust
   // ❌ 错误
   let value = option.unwrap();
   
   // ✅ 正确
   let value = option.ok_or_else(|| EngineError::General {
       message: "Expected value".to_string(),
       // ...
   })?;
   ```

2. **不要忽略错误**
   ```rust
   // ❌ 错误
   let _ = operation();
   
   // ✅ 正确
   operation()?;
   ```

3. **不要使用panic**
   ```rust
   // ❌ 错误
   panic!("Something went wrong");
   
   // ✅ 正确
   return Err(EngineError::General {
       message: "Something went wrong".to_string(),
       severity: ErrorSeverity::Error,
       // ...
   });
   ```

## 错误恢复策略

### 重试策略

```rust
use crate::error::retry::{RetryConfig, RetryExecutor};

let config = RetryConfig {
    max_attempts: 3,
    base_delay: Duration::from_millis(100),
    backoff_multiplier: 2.0,
    max_delay: Duration::from_secs(1),
};

let executor = RetryExecutor::new(config);
let result = executor.execute(|| {
    network_request()
});
```

### 使用默认值

```rust
use crate::error::RecoveryStrategy;

let strategy = RecoveryStrategy::UseDefault {
    default_description: "Using default texture".to_string(),
    log_warning: true,
};
```

### 优雅降级

```rust
let strategy = RecoveryStrategy::GracefulDegradation {
    degradation_level: 1,
    description: "Reducing texture quality".to_string(),
    fallback: "Using lower quality texture".to_string(),
};
```

## 错误处理示例

### 资源加载错误处理

```rust
pub async fn load_texture_async(path: &str) -> Result<Texture, EngineError> {
    // 尝试加载
    match tokio::fs::read(path).await {
        Ok(data) => {
            Texture::from_bytes(&data)
                .map_err(|e| RenderError::TextureCreation {
                    texture: path.to_string(),
                    message: format!("Failed to decode texture: {}", e),
                    severity: ErrorSeverity::Error,
                }.into())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // 文件不存在，使用默认纹理
            tracing::warn!("Texture not found: {}, using default", path);
            Ok(Texture::default())
        }
        Err(e) => {
            Err(ResourceError::FileRead {
                path: path.to_string(),
                message: e.to_string(),
                severity: ErrorSeverity::Error,
            }.into())
        }
    }
}
```

### 网络错误处理

```rust
pub fn send_message(msg: NetworkMessage) -> Result<(), EngineError> {
    self.connection.send(msg)
        .map_err(|e| SystemError::Network {
            message: format!("Failed to send message: {}", e),
            severity: ErrorSeverity::Error,
        }.into())
}
```

## 相关资源

- [thiserror文档](https://docs.rs/thiserror/)
- [Rust错误处理最佳实践](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [错误处理模块文档](../error/mod.rs)

