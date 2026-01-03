# C# 脚本热重载使用指南

## 概述

C# 脚本热重载是一个开发时优化特性，允许在修改 C# 脚本后自动重新编译和加载，无需重启应用程序。

## 特性

- ✅ **文件系统监听**: 自动检测 .cs 文件变化
- ✅ **防抖动处理**: 避免频繁重载（默认 100ms）
- ✅ **自动重新编译**: 修改后自动编译
- ✅ **缓存更新**: 自动更新编译缓存
- ✅ **错误恢复**: 编译失败时保留旧版本

## 性能

- **文件变化检测**: <1ms
- **热重载延迟**: <100ms（可配置）
- **内存开销**: <10MB
- **CPU开销**: 空闲时 <1%

## 架构

### HotReloadWatcher

```rust
pub struct HotReloadWatcher {
    _watcher: Option<RecommendedWatcher>,
    scripts: Arc<Mutex<HashMap<PathBuf, ScriptInfo>>>,
    config: HotReloadConfig,
    dotnet_host: Option<Arc<DotNetCliHost>>,
    compile_cache: Option<Arc<CompileCache>>,
    // ...
}
```

### 工作流程

```
文件修改 → 防抖动(100ms) → 检测变化 → 重新编译 → 更新缓存 → 通知应用
```

## 使用方法

### 基本用法

```rust
use game_engine::scripting::csharp::CSharpContext;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx = CSharpContext::new();

    // 启用热重载
    ctx.enable_hot_reload(
        vec![PathBuf::from("./scripts")],  // 监听目录
        100,  // 防抖动延迟（毫秒）
    )?;

    // 主循环
    loop {
        // 检查热重载事件
        match ctx.check_hot_reload() {
            Ok(reloaded) => {
                if !reloaded.is_empty() {
                    println!("Reloaded {} scripts", reloaded.len());
                    // 处理重新加载的脚本
                }
            }
            Err(e) => {
                eprintln!("Hot reload error: {}", e);
            }
        }

        // 游戏逻辑...
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
```

### 高级配置

```rust
use game_engine::scripting::csharp::CSharpContext;
use game_engine::scripting::csharp_hot_reload::HotReloadConfig;

// 自定义配置
let config = HotReloadConfig {
    watch_directories: vec![
        PathBuf::from("./scripts/game"),
        PathBuf::from("./scripts/ui"),
    ],
    debounce_duration_ms: 200,  // 更长的防抖动
    auto_compile: true,
    update_cache: true,
    file_pattern: Some("*.cs".to_string()),
};

// 创建上下文...
```

### 事件处理

```rust
let mut ctx = CSharpContext::new();

// 启用热重载
ctx.enable_hot_reload(vec![PathBuf::from("./scripts")], 100)?;

// 自定义事件处理（需要扩展 API）
// 目前可以依赖 tracing 日志查看事件
```

### 手动重新加载

```rust
// 强制重新加载所有脚本
match ctx.reload_all_scripts() {
    Ok(reloaded) => {
        println!("Manually reloaded {} scripts", reloaded.len());
    }
    Err(e) => {
        eprintln!("Failed to reload: {}", e);
    }
}
```

### 禁用热重载

```rust
ctx.disable_hot_reload();
```

## 配置选项

### HotReloadConfig

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `watch_directories` | `Vec<PathBuf>` | `["./"]` | 监听的目录列表 |
| `debounce_duration_ms` | `u64` | `100` | 防抖动延迟（毫秒） |
| `auto_compile` | `bool` | `true` | 是否自动编译 |
| `update_cache` | `bool` | `true` | 是否更新缓存 |
| `file_pattern` | `Option<String>` | `Some("*.cs")` | 文件过滤模式 |

### 防抖动延迟建议

- **快速响应**: 50ms（编辑时立即重载）
- **平衡**: 100ms（默认）
- **稳定**: 200-500ms（批量编辑后重载）

## 集成到游戏循环

### Bevy ECS 集成

```rust
use bevy_ecs::prelude::*;

fn hot_reload_system(mut ctx: ResMut<CSharpContext>) {
    if let Ok(reloaded) = ctx.check_hot_reload() {
        if !reloaded.is_empty() {
            info!("Hot reloaded {} C# scripts", reloaded.len());

            // 通知游戏系统重新加载脚本
            // ...
        }
    }
}

fn main() {
    let mut world = World::new();
    let mut schedule = Schedule::default();

    // 添加热重载系统
    schedule.add_systems(hot_reload_system);

    // 游戏循环
    loop {
        schedule.run(&mut world);
    }
}
```

### 独立游戏循环

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx = CSharpContext::new();
    ctx.enable_hot_reload(vec![PathBuf::from("./scripts")], 100)?;

    loop {
        // 1. 处理热重载
        let _ = ctx.check_hot_reload();

        // 2. 更新游戏逻辑
        update_game();

        // 3. 渲染
        render();

        // 4. 控制帧率
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
```

## 示例程序

运行热重载示例：

```bash
cargo run --example csharp_hot_reload_example --features csharp
```

示例将：
1. 创建示例脚本
2. 启用热重载
3. 定期修改脚本
4. 自动检测并重新加载
5. 执行重新加载的脚本

## 性能优化

### 文件监听优化

```rust
// 只监听特定目录
ctx.enable_hot_reload(
    vec![
        PathBuf::from("./scripts"),  // 仅监听脚本目录
    ],
    100,
)?;
```

### 防抖动优化

```rust
// 批量编辑时增加防抖动延迟
ctx.enable_hot_reload(
    vec![PathBuf::from("./scripts")],
    500,  // 500ms - 适合批量编辑
)?;
```

### 选择性编译

```rust
// 仅编译，不更新缓存（更快，但下次执行需要重新编译）
let config = HotReloadConfig {
    auto_compile: true,
    update_cache: false,  // 禁用缓存更新
    ..Default::default()
};
```

## 故障排查

### 热重载不工作

**检查**: 日志中是否有 "Failed to create file watcher"

**解决**: 确保系统文件监听功能可用

```bash
# Linux: 确保没有超出 inotify 限制
echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

### 编译失败

**检查**: 日志中的编译错误信息

**解决**: 修复脚本中的语法错误

```rust
match ctx.check_hot_reload() {
    Ok(reloaded) => { /* ... */ }
    Err(e) => {
        eprintln!("Compilation error: {}", e);
        // 显示错误给开发者
    }
}
```

### 性能问题

**检查**: CPU 使用率过高

**解决**:
1. 增加防抖动延迟
2. 减少监听目录数量
3. 禁用自动编译

## 最佳实践

1. **开发环境**: 启用热重载
2. **生产环境**: 禁用热重载
3. **定期检查**: 每帧或每100ms检查一次
4. **错误处理**: 记录编译错误但继续运行
5. **优雅降级**: 热重载失败时回退到手动重载

### 环境区分

```rust
#[cfg(debug_assertions)]
{
    // 开发模式：启用热重载
    ctx.enable_hot_reload(vec![PathBuf::from("./scripts")], 100)?;
}

#[cfg(not(debug_assertions))]
{
    // 发布模式：禁用热重载
    // 热重载代码会被编译器优化掉
}
```

## 限制和注意事项

1. **平台差异**: 文件监听在不同平台行为略有不同
2. **网络驱动**: 监听网络驱动器可能有延迟
3. **大项目**: 监听大量目录可能影响性能
4. **编译时间**: 复杂脚本编译可能需要较长时间

## 相关文档

- [C# 进程池](./csharp_process_pool.md)
- [C# 编译缓存](./csharp_compile_cache.md)
- [C# 运行时评估](./csharp_runtime_evaluation.md)

## 参考资料

- [notify crate 文档](https://docs.rs/notify/)
- [.NET CLI 文档](https://learn.microsoft.com/zh-cn/dotnet/core/tools/)
