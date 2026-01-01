# xmake 构建系统完整指南

**版本**: 0.1.0
**更新日期**: 2025-01-01
**状态**: ✅ 完整文档

---

## 目录

1. [快速开始](#快速开始)
2. [安装xmake](#安装xmake)
3. [基础命令](#基础命令)
4. [平台配置](#平台配置)
5. [构建模式](#构建模式)
6. [常见问题](#常见问题)
7. [高级用法](#高级用法)
8. [交互式初始化](#交互式初始化)
9. [CI/CD集成](#cicd集成)

---

## 快速开始

### 最简流程

```bash
# 1. 安装xmake（见下文）
# 2. 克隆项目
git clone https://github.com/your-org/game-engine.git
cd game-engine

# 3. 初始化项目（交互式）
python scripts/init_xmake.py

# 4. 构建
xmake

# 5. 运行
xmake run
```

### 5分钟快速构建

```bash
# macOS/Linux
curl -fsSL https://xmake.io/shget | bash
xmake
```

```powershell
# Windows (PowerShell)
Invoke-Expression (Invoke-WebRequest https://xmake.io/shget.ps1 -UseBasicParsing).Content
xmake
```

---

## 安装xmake

### macOS

```bash
# 使用Homebrew
brew install xmake

# 或使用安装脚本
curl -fsSL https://xmake.io/shget | bash
```

### Linux

```bash
# Ubuntu/Debian
bash <(curl -fsSL https://xmake.io/shget)

# Fedora
sudo dnf install xmake

# Arch Linux
yay -S xmake
```

### Windows

```powershell
# 使用PowerShell脚本
Invoke-Expression (Invoke-WebRequest https://xmake.io/shget.ps1 -UseBasicParsing).Content

# 或使用Scoop
scoop install xmake

# 或使用Chocolatey
choco install xmake
```

### 验证安装

```bash
xmake --version
# 应显示: xmake v2.8.3+ (或更新版本)
```

---

## 基础命令

### 项目配置

```bash
# 显示配置菜单
xmake f

# 配置平台
xmake f -p windows
xmake f -p linux
xmake f -p macosx
xmake f -p android
xmake f -p wasm

# 配置架构
xmake f -a x86_64
xmake f -a arm64
xmake f -a armv7

# 配置模式
xmake f -m debug
xmake f -m release

# 组合配置
xmake f -p android -a arm64 -m release
```

### 构建命令

```bash
# 构建项目（默认模式）
xmake

# 构建并运行
xmake run

# 重新构建
xmake -r

# 监视文件变化并自动构建
xmake watch -w

# 并行构建（使用所有CPU核心）
xmake -j $(nproc)
```

### 清理命令

```bash
# 清理构建产物
xmake clean

# 深度清理（包括配置）
xmake clean --all

# 清理特定目标
xmake clean --target game-engine-core
```

### 包管理

```bash
# 安装依赖包
xmake require

# 更新依赖包
xmake require --upgrade

# 搜索包
xmake require --search opengl
```

---

## 平台配置

### Windows

```bash
# 配置Visual Studio
xmake f -p windows -c vsxmake

# 配置MinGW
xmake f -p windows -c mingw

# 构建命令
xmake
xmake run
```

**要求**:
- Visual Studio 2019+ 或 MinGW-w64
- Windows 10+

**常见问题**:
```bash
# 如果遇到MSVC错误
xmake f -c vsxmake --vs=2022

# 如果需要指定SDK版本
xmake f --windows_sdk=10.0.22621.0
```

### Linux

```bash
# 标准配置
xmake f -p linux

# 安装依赖
sudo apt install build-essential cmake libx11-dev libgl1-mesa-dev

# 构建
xmake
```

**要求**:
- GCC 9+ 或 Clang 10+
- POSIX线程库
- X11开发库（用于窗口系统）

**发行版特定**:
```bash
# Ubuntu/Debian
sudo apt install build-essential libx11-dev libgl1-mesa-dev

# Fedora
sudo dnf install gcc-c++ libX11-devel mesa-libGL-devel

# Arch Linux
sudo pacman -S base-devel libx11 mesa
```

### macOS

```bash
# 配置
xmake f -p macosx

# 构建
xmake

# 代码签名（如需要）
codesign -s --force --deep build/macosx/x86_64/release/*.app
```

**要求**:
- Xcode 13+ 或 Xcode Command Line Tools
- macOS 11+ (Big Sur)

**常见问题**:
```bash
# 如果遇到Xcode路径问题
sudo xcode-select -s /Applications/Xcode.app

# 如果遇到代码签名问题
xmake f --appledev=entitlements
```

### Android

```bash
# 配置Android
xmake f -p android -a arm64

# 设置NDK路径（如果xmake找不到）
xmake f --ndk=/path/to/android-ndk

# 构建
xmake

# 生成APK
xmake package
```

**要求**:
- Android NDK r21+
- Android SDK r30+
- JDK 11+

**环境变量**:
```bash
export ANDROID_NDK_HOME=/path/to/android-ndk
export ANDROID_SDK_HOME=/path/to/android-sdk
```

### WebAssembly (Wasm)

```bash
# 配置Wasm
xmake f -p wasm

# 构建
xmake

# 启动本地服务器测试
python -m http.server 8000 -d build/wasm
```

**要求**:
- Emscripten 3.0+
- Python 3.8+

**安装Emscripten**:
```bash
# 获取emsdk
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk

# 安装最新版本
./emsdk install latest
./emsdk activate latest

# 配置环境
source ./emsdk_env.sh
```

---

## 构建模式

### Debug模式

```bash
# 配置debug模式
xmake f -m debug

# 或简写
xmake f -d

# 特性:
# - 完整符号信息
# - 无优化
# - 启用断言
# - 额外运行时检查
```

### Release模式

```bash
# 配置release模式
xmake f -m release

# 或简写
xmake f -r

# 特性:
# - 最高优化级别 (-O3)
# - 去除符号
# - 禁用断言
# - 最小二进制大小
```

### Sanitizer模式

```bash
# AddressSanitizer (检测内存错误)
xmake f -m asan

# ThreadSanitizer (检测数据竞争)
xmake f -m tsan

# LeakSanitizer (检测内存泄漏)
xmake f -m lsan

# UndefinedBehaviorSanitizer
xmake f -m ubsan
```

**使用示例**:
```bash
# 检测内存错误
xmake f -m asan
xmake run
# 如果检测到错误，会显示详细报告
```

---

## 常见问题

### 编译错误

**问题**: 找不到头文件
```bash
# 解决方案1: 安装开发包
sudo apt install libxxx-dev

# 解决方案2: 指定包含路径
xmake f --includedirs=/path/to/headers
```

**问题**: 链接错误
```bash
# 解决方案1: 安装库文件
sudo apt install libxxx

# 解决方案2: 指定库路径
xmake f --linkdirs=/path/to/libs

# 解决方案3: 添加链接标志
xmake f --ldflags="-lxxx"
```

### Rust相关问题

**问题**: Cargo编译失败
```bash
# 清理Cargo缓存
cd game_engine
cargo clean

# 重新构建
xmake -r
```

**问题**: 找不到Rust工具链
```bash
# 设置Rust路径
xmake f --rustc=/path/to/rustc

# 或使用环境变量
export RUSTC=/path/to/rustc
export CARGO=/path/to/cargo
```

### 性能问题

**问题**: 构建速度慢
```bash
# 使用更多并行任务
xmake -j 8

# 使用ccache加速
xmake f --ccache=y

# 启用增量编译
xmake f --incremental=y
```

---

## 高级用法

### 自定义配置选项

```bash
# 查看所有配置选项
xmake f --help

# 设置特定选项
xmake f --xxx=y

# 常用选项:
xmake f --enable_tests=n       # 禁用测试
xmake f --enable_examples=n    # 禁用示例
xmake f --strict_warnings=y    # 严格警告
```

### 模块化构建

```bash
# 只构建特定目标
xmake build game-engine-core

# 构建多个目标
xmake build game-engine-core game-engine-tools

# 排除目标
xmake build --exclude=*test*
```

### 交叉编译

```bash
# Linux到Windows交叉编译
xmake f -p windows -a x86_64 --cross=/usr/bin/x86_64-w64-mingw32-

# Linux到ARM交叉编译
xmake f -p linux -a arm64 --cross=aarch64-linux-gnu-

# macOS到iOS交叉编译
xmake f -p iphoneos -a arm64
```

### 生成IDE项目

```bash
# 生成VSCode项目
xmake project -k vscode

# 生成Visual Studio项目
xmake project -k vsxmake -m "vs2022"

# 生成Xcode项目
xmake project -k xcode

# 生成CMake项目
xmake project -k cmake
```

---

## 交互式初始化

### 使用初始化脚本

```bash
# 运行交互式初始化
python scripts/init_xmake.py

# 或使用xmake配置向导
xmake f --menu
```

**初始化步骤**:
1. 检测操作系统和平台
2. 检测编译器（GCC, Clang, MSVC）
3. 检测依赖库（SDL2, OpenGL, Vulkan等）
4. 生成最优配置
5. 提供项目模板选择

**示例输出**:
```
[Game Engine xmake初始化向导]
=========================================

检测到平台: macOS (arm64)
检测到编译器: Clang 14.0.0
检测到Xcode: 14.2

建议配置:
- 平台: macosx
- 架构: arm64
- 模式: release
- 优化: fastest

是否接受此配置? [Y/n]: Y

配置已保存到 .xmake/config.cache
现在可以运行: xmake
```

---

## CI/CD集成

### GitHub Actions

```yaml
name: Build with xmake

on: [push, pull_request]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        mode: [debug, release]

    steps:
    - uses: actions/checkout@v2

    - name: Install xmake
      run: |
        curl -fsSL https://xmake.io/shget | bash
        echo "$HOME/.xmake/bin" >> $GITHUB_PATH

    - name: Configure
      run: xmake f -m ${{ matrix.mode }}

    - name: Build
      run: xmake -j $(nproc)

    - name: Test
      run: xmake test
```

### GitLab CI

```yaml
build:
  script:
    - curl -fsSL https://xmake.io/shget | bash
    - xmake f -m release
    - xmake
    - xmake test
  artifacts:
    paths:
      - build/
```

### Docker

```dockerfile
FROM ubuntu:22.04

# 安装xmake
RUN curl -fsSL https://xmake.io/shget | bash
ENV PATH=/root/.xmake/bin:$PATH

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    build-essential \
    libx11-dev \
    libgl1-mesa-dev

# 复制项目
COPY . /app
WORKDIR /app

# 构建项目
RUN xmake f -m release && xmake
```

---

## 附录

### 速查表

| 任务 | 命令 |
|------|------|
| 构建项目 | `xmake` |
| 运行项目 | `xmake run` |
| 清理 | `xmake clean` |
| 配置 | `xmake f` |
| 帮助 | `xmake help` |
| 版本 | `xmake --version` |
| 配置菜单 | `xmake f --menu` |

### 配置文件位置

```
项目根目录/
├── xmake.lua              # 主配置文件
├── .xmake/                # xmake配置目录
│   ├── config.cache       # 配置缓存
│   └── build/             # 构建缓存
└── build/                 # 构建输出
    ├── linux/
    ├── macosx/
    ├── windows/
    └── wasm/
```

### 环境变量

```bash
# xmake相关
export XMAKE_ROOT=/path/to/xmake
export XMAKE_GLOBAL_DIR=/path/to/global/config

# 构建相关
export CFLAGS="-Wall -Wextra"
export CXXFLAGS="-std=c++20"
export LDFLAGS="-lpthread"

# Rust相关
export CARGO_HOME=/path/to/cargo
export RUSTUP_HOME=/path/to/rustup
```

---

## 支持和反馈

- **文档**: https://xmake.io/#/docs/home
- **GitHub**: https://github.com/xmake-io/xmake
- **论坛**: https://github.com/xmake-io/xmake/discussions
- **问题报告**: https://github.com/xmake-io/xmake/issues

---

**文档版本**: 1.0
**最后更新**: 2025-01-01
**维护者**: Game Engine Development Team
