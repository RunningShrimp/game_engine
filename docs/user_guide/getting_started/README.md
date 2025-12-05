# 快速开始

本指南将帮助您安装引擎并创建第一个项目。

## 安装

### 系统要求

- Rust 1.70+ (stable)
- 支持 Vulkan/Metal/DX12 的 GPU
- Windows 10+, macOS 10.15+, 或 Linux

### 安装 Rust

如果您还没有安装 Rust，请访问 [rustup.rs](https://rustup.rs/) 安装。

### 添加依赖

在您的 `Cargo.toml` 中添加：

```toml
[dependencies]
game_engine = { git = "https://github.com/username/game_engine" }
```

或使用本地路径：

```toml
[dependencies]
game_engine = { path = "../game_engine" }
```

## 第一个项目

### 创建新项目

```bash
cargo new my_game
cd my_game
```

### 基本代码

编辑 `src/main.rs`：

```rust
use game_engine::core::Engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Engine::run()?;
    Ok(())
}
```

### 运行

```bash
cargo run
```

## 下一步

- 查看 [核心概念](../core_concepts/README.md) 了解引擎架构
- 运行 [示例项目](../../../examples/README.md) 学习用法
- 阅读 [教程](../tutorials/README.md) 创建您的第一个游戏

