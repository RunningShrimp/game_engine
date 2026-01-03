# CLI API 文档
# Command Line Interface API Reference

**版本**: v0.3.0
**最后更新**: 2026-01-03

---

## 📋 目录

1. [概述](#概述)
2. [全局命令](#全局命令)
3. [项目管理命令](#项目管理命令)
4. [构建命令](#构建命令)
5. [运行命令](#运行命令)
6. [资源管理](#资源管理)
7. [配置选项](#配置选项)
8. [插件系统](#插件系统)

---

## 概述

游戏引擎CLI是一个强大的命令行工具，用于创建、构建和管理游戏项目。

### 安装

```bash
# 通过Cargo安装
cargo install game-engine-cli

# 或使用预编译二进制
curl -sSf https://install.game-engine.dev/cli | sh
```

### 基本使用

```bash
# 查看帮助
game-engine --help

# 查看版本
game-engine --version

# 查看特定命令帮助
game-engine new --help
```

---

## 全局命令

### --verbose, -v

增加输出详细程度。

```bash
game-engine -v build
game-engine -vv build  # 更详细
```

### --quiet, -q

减少输出，只显示错误。

```bash
game-engine -q build
```

### --color

控制彩色输出。

```bash
game-engine --color=always build
game-engine --color=never build
game-engine --color=auto build  # 默认
```

### --config, -c

指定配置文件。

```bash
game-engine -c custom-config.toml build
```

---

## 项目管理命令

### new

创建新项目。

**语法**:
```bash
game-engine new [OPTIONS] <PROJECT_NAME>
```

**选项**:
- `--template <TEMPLATE>` - 项目模板
  - `3d-game` - 3D游戏（默认）
  - `2d-platformer` - 2D平台游戏
  - `vr-app` - VR应用
  - `ar-app` - AR应用
  - `empty` - 空项目
- `--path <PATH>` - 项目路径（默认：当前目录）
- `--name <NAME>` - 项目名称（默认：使用目录名）

**示例**:
```bash
# 创建3D游戏项目
game-engine new my-game --template 3d-game

# 创建2D平台游戏
game-engine new platformer --template 2d-platformer

# 在指定路径创建项目
game-engine new my-game --path ~/projects

# 创建空项目
game-engine new custom --template empty
```

**生成的项目结构**:
```
my-game/
├── src/
│   ├── main.rs
│   └── lib.rs
├── assets/
│   ├── models/
│   ├── textures/
│   └── audio/
├── Cargo.toml
├── README.md
└── .gitignore
```

### init

在现有目录中初始化项目。

**语法**:
```bash
game-engine init [OPTIONS]
```

**选项**:
- `--template <TEMPLATE>` - 项目模板

**示例**:
```bash
cd existing-project
game-engine init --template 3d-game
```

### clean

清理构建输出。

**语法**:
```bash
game-engine clean [OPTIONS]
```

**选项**:
- `--all` - 清理所有构建产物
- `--deps` - 只清理依赖
- `--target <TARGET>` - 清理特定目标

**示例**:
```bash
game-engine clean
game-engine clean --all
```

---

## 构建命令

### build

构建项目。

**语法**:
```bash
game-engine build [OPTIONS] [PATH]
```

**选项**:
- `--release` - 发布模式构建
- `--debug` - 调试模式构建（默认）
- `--target <TRIPLE>` - 目标平台
  - `x86_64-unknown-linux-gnu` - Linux
  - `x86_64-pc-windows-msvc` - Windows
  - `x86_64-apple-darwin` - macOS
  - `aarch64-linux-android` - Android
  - `aarch64-apple-ios` - iOS
- `--features <FEATURES>` - 启用功能
- `--no-default-features` - 禁用默认功能
- `--profile <PROFILE>` - 构建配置文件
- `--timings` - 显示构建时间

**示例**:
```bash
# 调试模式构建
game-engine build

# 发布模式构建
game-engine build --release

# 跨平台构建
game-engine build --target x86_64-pc-windows-msvc

# 启用特定功能
game-engine build --features "network,physics"

# 显示构建时间
game-engine build --timings
```

**输出示例**:
```
   Compiling game-engine v0.3.0
   Compiling my-game v0.1.0 (/path/to/my-game)
    Finished dev [unoptimized + debuginfo] target(s) in 12.45s
```

### check

快速检查代码（不生成二进制）。

**语法**:
```bash
game-engine check [OPTIONS]
```

**示例**:
```bash
game-engine check
```

### test

运行测试。

**语法**:
```bash
game-engine test [OPTIONS]
```

**选项**:
- `--release` - 发布模式测试
- `--no-run` - 编译但不运行
- `--nocapture` - 不捕获输出
- `--test-threads <NUM>` - 测试线程数

**示例**:
```bash
game-engine test
game-engine test --release
game-engine test -- --ignored  # 运行被忽略的测试
```

---

## 运行命令

### run

运行项目。

**语法**:
```bash
game-engine run [OPTIONS] [--] [ARGS]...
```

**选项**:
- `--release` - 发布模式运行
- `--example <NAME>` - 运行示例
- `--bin <NAME>` - 运行特定二进制
- `--profile <PROFILE>` - 使用配置文件

**示例**:
```bash
# 调试模式运行
game-engine run

# 发布模式运行
game-engine run --release

# 运行示例
game-engine run --example hello-world

# 传递参数
game-engine run -- --fullscreen --vsync
```

### benchmark

运行性能基准测试。

**语法**:
```bash
game-engine benchmark [OPTIONS]
```

**选项**:
- `--filter <FILTER>` - 过滤基准测试
- `--save-baseline <NAME>` - 保存基线
- `--baseline <NAME>` - 比较基线

**示例**:
```bash
game-engine benchmark
game-engine benchmark --filter "rendering"
```

---

## 资源管理

### asset

管理项目资源。

**语法**:
```bash
game-engine asset <SUBCOMMAND> [OPTIONS]
```

#### asset import

导入资源。

**语法**:
```bash
game-engine asset import <SOURCE> [DESTINATION]
```

**示例**:
```bash
# 导入模型
game-engine asset import ~/models/player.fbx assets/models/

# 导入纹理
game-engine asset import ~/textures/player.png assets/textures/
```

#### asset optimize

优化资源。

**语法**:
```bash
game-engine asset optimize <ASSET> [OPTIONS]
```

**选项**:
- `--quality <LEVEL>` - 质量级别 (1-10)
- `--format <FORMAT>` - 输出格式

**示例**:
```bash
game-engine asset optimize assets/textures/player.png --quality 8
```

#### asset bundle

打包资源。

**语法**:
```bash
game-engine asset bundle [OPTIONS]
```

**选项**:
- `--output <PATH>` - 输出路径
- `--compression <LEVEL>` - 压缩级别

**示例**:
```bash
game-engine asset bundle --output assets.bundle
```

---

## 配置选项

### 配置文件 (game-engine.toml)

**项目根目录**:
```toml
[project]
name = "My Game"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]

[build]
target = "x86_64-unknown-linux-gnu"
opt-level = 3
lto = true

[profile.dev]
opt-level = 0
debug = true

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[features]
default = ["rendering", "physics"]
network = []
physics = ["dep:physics-engine"]
rendering = []

[assets]
source = "assets"
target = "target/assets"
compression = "zstd"
```

**用户配置** (~/.config/game-engine/config.toml):
```toml
[build]
jobs = 4
target-dir = "~/.cache/game-engine"

[editor]
font-family = "Monaco"
font-size = 14
theme = "dark"

[lsp]
enabled = true
port = 9000
```

---

## 插件系统

### plugin

管理CLI插件。

**语法**:
```bash
game-engine plugin <SUBCOMMAND> [OPTIONS]
```

#### plugin install

安装插件。

**语法**:
```bash
game-engine plugin install <NAME> [VERSION]
```

**示例**:
```bash
# 从仓库安装
game-engine plugin install game-engine-plugin-vs

# 指定版本
game-engine plugin install game-engine-plugin-vs --version 1.0.0

# 从本地路径安装
game-engine plugin install ./my-plugin
```

#### plugin list

列出已安装的插件。

**语法**:
```bash
game-engine plugin list
```

**输出示例**:
```
NAME                     VERSION    STATUS
game-engine-plugin-vs    1.0.0      ✅ Enabled
game-engine-plugin-blender 0.5.0   ✅ Enabled
```

#### plugin remove

卸载插件。

**语法**:
```bash
game-engine plugin remove <NAME>
```

**示例**:
```bash
game-engine plugin remove game-engine-plugin-vs
```

---

## 高级功能

### watch

监视文件变化并自动重新构建。

**语法**:
```bash
game-engine watch [OPTIONS]
```

**示例**:
```bash
game-engine watch
```

### doctor

诊断项目问题。

**语法**:
```bash
game-engine doctor [OPTIONS]
```

**示例**:
```bash
game-engine doctor
```

**输出示例**:
```
✅ Rust toolchain: 1.70.0
✅ .NET SDK: 8.0.100
✅ Project structure: Valid
⚠️  Unused dependencies: 2
   - unused-crate
   - another-unused-crate
```

### info

显示项目信息。

**语法**:
```bash
game-engine info [OPTIONS]
```

**示例**:
```bash
game-engine info
```

**输出示例**:
```
Project: My Game v0.1.0
Engine: v0.3.0
Template: 3d-game
Build: debug
Targets: native, web
```

---

## 环境变量

### GAME_ENGINE_HOME

引擎安装路径。

```bash
export GAME_ENGINE_HOME=/opt/game-engine
```

### GAME_ENGINE_LOG

日志级别。

```bash
export GAME_ENGINE_LOG=debug
```

### GAME_ENGINE_CACHE

缓存目录。

```bash
export GAME_ENGINE_CACHE=~/.cache/game-engine
```

---

## 故障排除

### 常见问题

**Q: 构建失败，提示找不到依赖**
- A: 运行 `game-engine fetch` 更新依赖索引

**Q: 跨平台构建失败**
- A: 安装相应的交叉编译工具链

**Q: 资源导入失败**
- A: 检查资源文件格式和路径

### 调试模式

启用调试日志：
```bash
RUST_LOG=debug game-engine build
```

---

## 参考资料

- [Cargo文档](https://doc.rust-lang.org/cargo/)
- [Rust工具链](https://rust-lang.org/tools)

---

**文档版本**: v1.0
**最后更新**: 2026-01-03
