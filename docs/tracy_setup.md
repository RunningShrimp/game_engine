# Tracy Profiler 设置指南

## 概述

Tracy Profiler是一个高性能的实时性能分析工具。本指南介绍如何设置和使用Tracy进行性能分析。

## 安装步骤

### 1. 安装Tracy客户端库

在`game_engine/Cargo.toml`中添加依赖：

```toml
[dependencies]
tracy-client = { version = "0.22", optional = true }

[features]
tracy = ["dep:tracy-client"]
```

### 2. 安装Tracy Profiler应用程序

从以下位置下载并安装Tracy Profiler：

- **GitHub**: https://github.com/wolfpld/tracy/releases
- **官方网站**: https://github.com/wolfpld/tracy

### 3. 编译启用Tracy的版本

```bash
cargo build --features tracy
```

或运行示例：

```bash
cargo run --example tracy_profiling --features tracy
```

## 使用说明

### 基本用法

```rust
use game_engine::profiling::tracy::TracyScope;

{
    let _scope = TracyScope::new("my_function");
    // 你的代码
}
```

### 便捷宏

```rust
use game_engine::{tracy_scope, tracy_message, tracy_frame};

tracy_scope!("render_frame");
tracy_message!("Important event");
tracy_frame!();
```

## 连接Tracy Profiler

1. 启动你的应用程序（使用`--features tracy`编译）
2. 打开Tracy Profiler应用程序
3. 点击"Connect"连接到应用程序
4. 开始查看实时性能数据

## 注意事项

- Tracy只在开发和调试时使用
- 生产环境应禁用Tracy特性
- 确保防火墙允许Tracy连接

## 更多信息

参见 [Tracy Profiling指南](./tracy_profiling_guide.md) 了解详细使用方法。

