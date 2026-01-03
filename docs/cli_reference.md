# Game Engine CLI 参考文档

**版本**: v0.2.0
**更新日期**: 2026-01-03
**状态**: ✅ 基础功能完成

---

## 📖 目录

1. [概述](#概述)
2. [安装](#安装)
3. [命令参考](#命令参考)
4. [项目模板](#项目模板)
5. [配置](#配置)
6. [最佳实践](#最佳实践)
7. [故障排除](#故障排除)

---

## 概述

Game Engine CLI (`game-engine`) 是一个命令行工具，用于创建、构建和管理游戏引擎项目。

### 主要功能

- 🚀 **项目创建**: 快速创建新项目
- 📦 **模板管理**: 使用和创建项目模板
- 🔧 **依赖管理**: 智能依赖分析和优化
- 🏗️ **构建系统**: 生成 CMake、xmake 构建文件
- 📊 **信息查询**: 查看引擎版本、依赖等信息
- ⚙️ **配置管理**: 项目和全局配置

### 命令结构

```bash
game-engine [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]
```

---

## 安装

### 从源码安装

```bash
cd game_engine
cargo install --path game_engine/src/tools/cli
```

### 验证安装

```bash
game-engine --version
game-engine --help
```

### 自动补全

#### Bash

```bash
# 临时启用
source <(game-engine completions bash)

# 永久启用
game-engine completions bash > ~/.local/share/bash-completion/completions/game-engine
```

#### Zsh

```bash
# 临时启用
source <(game-engine completions zsh)

# 永久启用
game-engine completions zsh > /usr/local/share/zsh/site-functions/_game-engine
```

#### Fish

```bash
# 永久启用
game-engine completions fish > ~/.config/fish/completions/game-engine.fish
```

---

## 命令参考

### 全局选项

适用于所有命令的选项：

| 选项 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--help` | `-h` | 显示帮助信息 | - |
| `--version` | `-V` | 显示版本号 | - |
| `--verbose` | `-v` | 详细输出 | false |
| `--quiet` | `-q` | 静默模式 | false |
| `--color` | | 彩色输出 `auto|always|never` | auto |
| `--config` | `-c` | 配置文件路径 | `./game-engine.toml` |

---

### 1. `game-engine new` - 创建新项目

创建一个新的游戏引擎项目。

#### 用法

```bash
game-engine new [OPTIONS] <PROJECT_NAME>
```

#### 参数

| 参数 | 说明 |
|------|------|
| `PROJECT_NAME` | 项目名称（必须，kebab-case） |

#### 选项

| 选项 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--template` | `-t` | 项目模板 | `basic` |
| `--path` | `-p` | 项目路径 | 当前目录 |
| `--git` | | 初始化 Git 仓库 | true |
| `--author` | | 作者名称 | 从 Git 配置读取 |
| `--license` | | 许可证类型 | `MIT` |
| `--description` | | 项目描述 | - |

#### 模板列表

| 模板 | 说明 | 适用场景 |
|------|------|----------|
| `basic` | 基础项目模板 | 学习、小型项目 |
| `2d-platformer` | 2D平台游戏 | 2D横版游戏 |
| `3d-fps` | 3D第一人称射击 | 3D FPS游戏 |
| `rts` | 即时战略游戏 | RTS游戏 |
| `rpg` | 角色扮演游戏 | RPG游戏 |
| `networked` | 网络游戏 | 多人在线游戏 |

#### 示例

```bash
# 创建基础项目
game-engine new my-game

# 创建2D平台游戏
game-engine new platformer --template 2d-platformer

# 创建3D FPS游戏，指定路径
game-engine new fps-game --template 3d-fps --path ~/projects/

# 创建项目但不初始化Git
game-engine new test-game --no-git

# 创建项目并指定作者
game-engine new my-game --author "Your Name" --description "My awesome game"
```

#### 生成的项目结构

```
my-game/
├── Cargo.toml          # 项目配置
├── .gitignore          # Git忽略规则
├── assets/             # 资源文件夹
│   ├── images/
│   ├── sounds/
│   └── models/
├── src/
│   └── main.rs         # 入口文件
└── README.md           # 项目说明
```

---

### 2. `game-engine init` - 初始化现有项目

在现有目录中初始化游戏引擎项目。

#### 用法

```bash
game-engine init [OPTIONS]
```

#### 选项

| 选项 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--force` | `-f` | 覆盖现有文件 | false |
| `--template` | `-t` | 要添加的模板文件 | `basic` |

#### 示例

```bash
# 在当前目录初始化
cd existing-project
game-engine init

# 强制覆盖配置文件
game-engine init --force
```

---

### 3. `game-engine template` - 模板管理

管理项目模板。

#### 3.1 `template list` - 列出模板

```bash
game-engine template list [OPTIONS]
```

**选项**:
| 选项 | 说明 |
|------|------|
| `--search` | 搜索关键词 |
| `--category` | 按类别筛选 |

**示例**:
```bash
# 列出所有模板
game-engine template list

# 搜索特定模板
game-engine template list --search platformer

# 按类别筛选
game-engine template list --category 2d
```

**输出示例**:
```
可用的项目模板:

  basic           - 基础项目模板
                   类别:入门

  2d-platformer   - 2D平台游戏模板
                   类别:2D
                   特性: 碰撞检测、物理模拟、动画系统

  3d-fps          - 3D第一人称射击模板
                   类别:3D
                   特性: 第一人称控制器、武器系统、AI导航

使用 game-engine new <PROJECT> --template <TEMPLATE> 创建项目
```

#### 3.2 `template info` - 查看模板详情

```bash
game-engine template info <TEMPLATE_NAME>
```

**示例**:
```bash
game-engine template info 2d-platformer
```

**输出示例**:
```
模板: 2d-platformer
描述: 2D平台游戏项目模板
类别: 2D游戏
版本: 0.2.0

特性:
  - 碰撞检测系统
  - 物理模拟
  - 动画系统
  - 关卡编辑器集成

依赖:
  - game_engine (latest)
  - physx-rs (for physics)

示例场景:
  - 经典平台跳跃游戏
  - 横版动作游戏
```

#### 3.3 `template create` - 创建自定义模板

```bash
game-engine template create [OPTIONS] <TEMPLATE_NAME>
```

**选项**:
| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--from` | 基于现有项目 | - |
| `--description` | 模板描述 | - |
| `--category` | 模板类别 | custom |

**示例**:
```bash
# 从当前项目创建模板
game-engine template create my-template --from . --description "我的自定义模板"
```

---

### 4. `game-engine build-system` - 构建系统生成

生成构建系统配置文件。

#### 用法

```bash
game-engine build-system [OPTIONS] --system <SYSTEM>
```

#### 选项

| 选项 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--system` | `-s` | 构建系统类型 | - |
| `--output` | `-o` | 输出目录 | 当前目录 |
| `--profile` | | 配置类型 `debug|release` | debug |

#### 支持的构建系统

| 系统 | 说明 | 文件 |
|------|------|------|
| `cmake` | CMake 构建系统 | CMakeLists.txt |
| `xmake` | xmake 构建系统 | xmake.lua |
| `meson` | Meson 构建系统 | meson.build |
| `premake` | Premake 构建系统 | premake5.lua |

#### 示例

```bash
# 生成 CMake 配置
game-engine build-system --system cmake

# 生成 xmake 配置到指定目录
game-engine build-system --system xmake --output ./build/

# 生成 Release 配置
game-engine build-system --system cmake --profile release
```

#### 生成的 CMakeLists.txt 示例

```cmake
cmake_minimum_required(VERSION 3.15)
project(my-game VERSION 0.1.0 LANGUAGES Rust)

# 查找 Rust 编译器
find_package(Rust REQUIRED)

# 添加可执行文件
add_executable(my-game
    src/main.rs
    src/game.rs
    src/entities.rs
)

# 链接游戏引擎库
target_link_libraries(my-game game_engine)

# 设置 Rust 特性
set_target_properties(my-game PROPERTIES
    CARGO_FEATURES "audio;physics;networking"
)

# 复制资源文件
add_custom_command(TARGET my-game POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_directory
    ${CMAKE_SOURCE_DIR}/assets
    $<TARGET_FILE_DIR:my-game>/assets
)
```

---

### 5. `game-engine check` - 依赖检查

检查项目依赖的健康状况。

#### 用法

```bash
game-engine check [OPTIONS]
```

#### 选项

| 选项 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--fix` | | 自动修复问题 | false |
| `--warnings-as-errors` | `-W` | 将警告视为错误 | false |
| `--offline` | | 离线模式（不联网） | false |

#### 示例

```bash
# 检查依赖
game-engine check

# 自动修复可修复的问题
game-engine check --fix

# 将警告视为错误
game-engine check --warnings-as-errors
```

#### 输出示例

```
正在检查依赖...

✅ 版本冲突: 未发现
⚠️  未使用依赖: 2个
   - rand (0.8.5) - 40KB
   - log (0.4.17) - 15KB
💡 优化建议: 3个
   - serde_json -> simd-json (2-4x 更快)
   - tokio -> async-std (更简单)
   - chrono -> time (更小)

状态: 健康 (3个建议)
```

---

### 6. `game-engine upgrade` - 升级依赖

升级项目依赖到最新兼容版本。

#### 用法

```bash
game-engine upgrade [OPTIONS] [DEPENDENCY]
```

#### 参数

| 参数 | 说明 |
|------|------|
| `DEPENDENCY` | 要升级的依赖（可选） |

#### 选项

| 选项 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--dry-run` | | 预览升级，不实际执行 | false |
| `--compatible` | | 仅升级到兼容版本 | true |
| `--breakage` | | 允许破坏性升级 | false |
| `--interactive` | `-i` | 交互式选择版本 | false |

#### 示例

```bash
# 升级所有依赖（兼容版本）
game-engine upgrade

# 预览升级
game-engine upgrade --dry-run

# 升级特定依赖
game-engine upgrade serde

# 允许破坏性升级
game-engine upgrade --breakage

# 交互式升级
game-engine upgrade --interactive
```

#### 交互式示例

```
? 选择要升级的依赖:
❯ ◯ serde (1.0.152 → 1.0.180)
  ◯ tokio (1.23.0 → 1.24.0)
  ◯ rand (0.8.5 → 0.8.6)

? 升级 serde 到 1.0.180?
  ◯ 是
  ◯ 否
  ◯ 查看变更日志

正在升级...
✅ serde: 1.0.152 → 1.0.180
✅ Cargo.lock 已更新
✅ 运行 cargo check 通过
```

---

### 7. `game-engine add` - 添加依赖

添加新的依赖到项目。

#### 用法

```bash
game-engine add <DEPENDENCY> [OPTIONS]
```

#### 选项

| 选项 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--version` | `-v` | 指定版本 | latest |
| `--dev` | `-d` | 添加为开发依赖 | false |
| `--build` | `-b` | 添加为构建依赖 | false |
| `--feature` | `-F` | 启用的特性 | - |
| `--git` | | 从 Git 仓库添加 | - |
| `--branch` | | Git 分支 | main |

#### 示例

```bash
# 添加最新版本
game-engine add serde

# 指定版本
game-engine add serde --version 1.0.150

# 添加开发依赖
game-engine add criterion --dev

# 启用特性
game-engine add tokio --feature full

# 从 Git 添加
game-engine add my-lib --git https://github.com/user/repo
```

---

### 8. `game-engine remove` - 移除依赖

从项目中移除依赖。

#### 用法

```bash
game-engine remove <DEPENDENCY> [OPTIONS]
```

#### 选项

| 选项 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--purge` | | 同时从 Cargo.lock 移除 | false |
| `--dry-run` | | 预览移除，不实际执行 | false |

#### 示例

```bash
# 移除依赖
game-engine remove rand

# 预览移除
game-engine remove rand --dry-run
```

---

### 9. `game-engine dependency` - 依赖管理

高级依赖管理功能。

#### 9.1 `dependency graph` - 显示依赖图

```bash
game-engine dependency graph [OPTIONS]
```

**选项**:
| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--format` | 输出格式 `text|dot|json` | text |
| `--output` | 输出到文件 | - |

**示例**:
```bash
# 文本格式显示
game-engine dependency graph

# 生成 Graphviz DOT 文件
game-engine dependency graph --format dot --output deps.dot

# JSON 格式（用于脚本处理）
game-engine dependency graph --format json > deps.json
```

#### 9.2 `dependency unused` - 检测未使用依赖

```bash
game-engine dependency unused [OPTIONS]
```

**选项**:
| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--remove` | 自动移除未使用依赖 | false |
| `--include-dev` | 包含开发依赖 | true |

**示例**:
```bash
# 检测未使用依赖
game-engine dependency unused

# 自动移除
game-engine dependency unused --remove
```

#### 9.3 `dependency optimize` - 依赖优化

```bash
game-engine dependency optimize [OPTIONS]
```

**选项**:
| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--apply` | 自动应用优化 | false |
| `--aggressive` | 激进优化模式 | false |

**示例**:
```bash
# 查看优化建议
game-engine dependency optimize

# 自动应用优化
game-engine dependency optimize --apply
```

---

### 10. `game-engine info` - 显示信息

显示引擎或项目信息。

#### 用法

```bash
game-engine info [OPTIONS]
```

#### 选项

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--json` | JSON 格式输出 | false |
| `--dependencies` | 显示依赖信息 | false |
| `--features` | 显示编译特性 | false |

#### 示例

```bash
# 显示基本信息
game-engine info

# 显示完整信息
game-engine info --dependencies --features

# JSON 格式
game-engine info --json
```

#### 输出示例

```
Game Engine CLI
版本: 0.2.0
编译器: rustc 1.70.0
目标: x86_64-unknown-linux-gnu

特性:
  ✅ audio - 音频系统
  ✅ physics - 物理引擎
  ✅ networking - 网络支持
  ❌ vulkan - Vulkan渲染（未启用）

依赖: 123个
  - serde (1.0.152)
  - tokio (1.23.0)
  ...
```

---

### 11. `game-engine config` - 配置管理

管理 CLI 和项目配置。

#### 11.1 `config show` - 显示配置

```bash
game-engine config show [OPTIONS]
```

**选项**:
| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--global` | 显示全局配置 | false |
| `--local` | 显示本地配置 | true |

**示例**:
```bash
# 显示项目配置
game-engine config show

# 显示全局配置
game-engine config show --global
```

#### 11.2 `config set` - 设置配置

```bash
game-engine config set <KEY> <VALUE> [OPTIONS]
```

**选项**:
| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--global` | 设置全局配置 | false |

**示例**:
```bash
# 设置本地配置
game-engine config set author.name "Your Name"

# 设置全局配置
game-engine config set default-template basic --global
```

#### 11.3 `config unset` - 删除配置

```bash
game-engine config unset <KEY> [OPTIONS]
```

**示例**:
```bash
game-engine config unset author.name
```

---

## 项目模板

### Basic 模板

最简单的游戏项目，适合学习和快速原型。

**特性**:
- 基础窗口创建
- 简单渲染循环
- 输入处理
- 音频播放（可选）

**适用场景**:
- 学习游戏引擎
- 简单的2D游戏
- 快速验证想法

### 2D Platformer 模板

完整的2D平台游戏框架。

**特性**:
- 碰撞检测系统
- 物理模拟（重力、速度）
- 动画系统
- 关卡编辑器集成
- 敌人AI基础

**包含组件**:
- Player: 玩家控制器
- Platform: 平台碰撞体
- Collectible: 收集物品
- Enemy: 基础敌人AI

**适用场景**:
- 平台跳跃游戏
- 横版动作游戏
- 2D冒险游戏

### 3D FPS 模板

第一人称射击游戏框架。

**特性**:
- 第一人称控制器
- 武器系统
- 敌人AI导航
- 3D物理集成
- 网络支持（可选）

**包含系统**:
- PlayerController: FPS控制器
- WeaponSystem: 武器管理
- HealthSystem: 生命值系统
- DamageSystem: 伤害计算

**适用场景**:
- FPS游戏
- TPS游戏
- 3D动作游戏

---

## 配置

### 项目配置文件

`game-engine.toml`:

```toml
[project]
name = "my-game"
version = "0.1.0"
description = "My awesome game"

[engine]
version = "0.2.0"
features = ["audio", "physics", "networking"]

[build]
target = ["x86_64-unknown-linux-gnu", "wasm32-unknown-unknown"]
opt-level = 3

[dependencies]
auto-update = true
check-unused = true

[author]
name = "Your Name"
email = "your.email@example.com"

[template]
name = "basic"
custom-templates-path = "./templates"
```

### 全局配置文件

`~/.config/game-engine/config.toml`:

```toml
[general]
default-template = "basic"
auto-completion = true
color-output = true

[build]
default-target = "release"
parallel-jobs = 4

[dependencies]
auto-check = true
check-frequency = "weekly"

[editor]
preferred = "vscode"
```

---

## 最佳实践

### 1. 项目结构

```
my-game/
├── src/              # 源代码
│   ├── main.rs       # 入口
│   ├── game.rs       # 游戏逻辑
│   ├── entities/     # 实体定义
│   ├── systems/      # 系统实现
│   └── resources/    # 资源管理
├── assets/           # 游戏资源
│   ├── images/
│   ├── sounds/
│   └── models/
├── tests/            # 测试
├── examples/         # 示例
├── benches/          # 性能测试
├── Cargo.toml        # 项目配置
├── game-engine.toml  # 引擎配置
└── README.md         # 项目说明
```

### 2. 依赖管理

```bash
# 定期检查依赖
game-engine check

# 定期升级（每月）
game-engine upgrade --dry-run  # 先预览
game-engine upgrade            # 确认后升级

# 移除未使用依赖
game-engine dependency unused --remove

# 优化依赖
game-engine dependency optimize
```

### 3. 版本控制

```bash
# 创建 .gitignore
echo "/target/" >> .gitignore
echo "*.lock" >> .gitignore
echo ".idea/" >> .gitignore

# 使用 Git 管理配置
git add game-engine.toml
git commit -m "Add engine config"
```

### 4. CI/CD 集成

```yaml
# .github/workflows/build.yml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Check dependencies
        run: game-engine check

      - name: Build
        run: cargo build --release

      - name: Test
        run: cargo test
```

---

## 故障排除

### 常见问题

#### 1. 命令未找到

**错误**: `bash: game-engine: command not found`

**解决**:
```bash
# 检查安装
which game-engine

# 重新安装
cargo install --path game_engine/src/tools/cli --force

# 添加到 PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

#### 2. 权限错误

**错误**: `Permission denied`

**解决**:
```bash
# 检查文件权限
ls -la $(which game-engine)

# 修复权限
chmod +x ~/.cargo/bin/game-engine
```

#### 3. 网络问题

**错误**: `Failed to fetch registry`

**解决**:
```bash
# 使用镜像
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# 或离线模式
game-engine check --offline
```

#### 4. 编译失败

**错误**: `Compilation failed`

**解决**:
```bash
# 清理缓存
cargo clean
game-engine clean

# 更新 Rust
rustup update stable

# 检查版本
rustc --version
game-engine --version
```

---

## 相关资源

- **项目仓库**: [GitHub]
- **文档**: `/docs/`
- **API 参考**: `/docs/api_reference.md`
- **示例**: `/examples/`
- **问题反馈**: GitHub Issues

---

## 更新日志

### v0.2.0 (2026-01-03)

- ✅ 项目创建和模板系统
- ✅ 依赖管理功能
- ✅ 构建系统生成
- ✅ 配置管理
- ✅ 自动补全脚本

### 未来计划

- 🚧 交互式配置向导
- 🚧 插件系统
- 🚧 云端模板库
- 🚧 性能分析工具

---

**祝你开发愉快！** 🎮
