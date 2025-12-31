# XMake Build Guide

本指南详细介绍如何使用XMake构建游戏引擎项目。

## 目录

- [简介](#简介)
- [安装XMake](#安装xmake)
- [快速开始](#快速开始)
- [配置选项](#配置选项)
- [平台特定配置](#平台特定配置)
- [交叉编译](#交叉编译)
- [高级功能](#高级功能)
- [故障排除](#故障排除)
- [最佳实践](#最佳实践)

---

## 简介

XMake是一个基于Lua的跨平台构建工具，支持C/C++、Rust、Go等多种语言。本项目使用XMake作为主要的构建系统，提供以下优势：

- **跨平台支持**: Windows、Linux、macOS、Android、WebAssembly
- **简洁配置**: 基于Lua的声明式配置
- **依赖管理**: 内置包管理器，支持远程依赖包
- **快速构建**: 支持增量编译和并行构建
- **灵活性**: 易于扩展和自定义

---

## 安装XMake

### Windows

```powershell
# 使用PowerShell一键安装
Invoke-Expression (Invoke-Webfile 'https://xmake.io/psget.txt')

# 或使用scoop安装
scoop install xmake

# 或使用choco安装
choco install xmake
```

### Linux/macOS

```bash
# 使用curl一键安装
curl -fsSL https://xmake.io/sh | sh

# 或使用homebrew安装（macOS）
brew install xmake

# 或从源码编译
git clone https://github.com/xmake-io/xmake.git
cd xmake
make build
```

### 验证安装

```bash
xmake --version
```

预期输出：
```
XMake v2.8.3+20231218
```

---

## 快速开始

### 基本构建

```bash
# 1. 克隆项目
git clone https://github.com/yourusername/game_engine.git
cd game_engine/game_engine

# 2. 配置项目（Debug模式）
xmake config -m debug

# 3. 构建
xmake

# 4. 运行
xmake run
```

### Release构建

```bash
# 配置为Release模式
xmake config -m release

# 构建
xmake

# 运行
xmake run
```

### 清理构建

```bash
# 清理构建产物
xmake clean

# 清理所有（包括配置）
xmake clean -a

# 使用自定义任务清理所有
xmake clean-all
```

---

## 配置选项

### 模式配置

XMake支持多种构建模式：

| 模式 | 优化级别 | 调试符号 | 说明 |
|------|---------|---------|------|
| `debug` | 无 | 完整 | 开发调试 |
| `release` | 最高 | 无 | 生产发布 |
| `asan` | 无 | 完整 | 地址消毒器（Address Sanitizer）|
| `tsan` | 无 | 完整 | 线程消毒器（Thread Sanitizer）|
| `lsan` | 无 | 完整 | 内存泄漏检测器 |
| `ubsan` | 无 | 完整 | 未定义行为检测器 |

```bash
# 配置调试模式
xmake config -m debug

# 配置发布模式
xmake config -m release

# 配置带ASAN的调试模式
xmake config -m asan
```

### 平台配置

```bash
# 配置目标平台
xmake config -p [platform]

# 支持的平台：
# - windows (Windows)
# - linux (Linux)
# - macosx (macOS)
# - android (Android)
# - wasm (WebAssembly)
```

### 架构配置

```bash
# 配置目标架构
xmake config -a [arch]

# 支持的架构：
# Linux/macOS:
#   - x86_64
#   - x64
#   - arm64
#   - aarch64
#
# Android:
#   - arm64-v8a
#   - armeabi-v7a
#   - x86_64
#   - x86
#
# WebAssembly:
#   - wasm32
```

---

## 平台特定配置

### Windows

#### MSVC编译器

```bash
# 使用MSVC编译
xmake config -p windows --toolchain=msvc

# 指定MSVC版本
xmake config -p windows --vs=2022

# 启用Unicode
xmake config -p windows --unicode=y
```

#### MinGW编译器

```bash
# 使用MinGW
xmake config -p windows --toolchain=mingw
```

#### Windows特定选项

```lua
-- 在xmake.lua中
if is_plat("windows") then
    -- 设置子系统
    add_ldflags("/SUBSYSTEM:CONSOLE", {force = true})

    -- 设置Windows版本
    add_defines("WINVER=0x0A00", "_WIN32_WINNT=0x0A00")

    -- 链接系统库
    add_syslinks("ws2_32", "userenv", "msvcrt")
end
```

### Linux

```bash
# 配置Linux平台
xmake config -p linux

# 指定编译器
xmake config -p linux --toolchain=gcc
xmake config -p linux --toolchain=clang

# 启用PIE（位置独立可执行文件）
xmake config -p linux --pie=y

# 链接静态库
xmake config -p linux --kind=static
```

#### Linux特定选项

```lua
-- 在xmake.lua中
if is_plat("linux") then
    -- 链接pthread
    add_ldflags("-pthread")

    -- 启用RLIMIT（资源限制）
    add_ldflags("-Wl,--no-as-needed")

    -- 链接系统库
    add_syslinks("pthread", "dl", "m")
end
```

### macOS

```bash
# 配置macOS平台
xmake config -p macosx

# 指定最低macOS版本
xmake config -p macosx --minver=11.0

-- 启用Universal Binary（支持x86_64和arm64）
xmake config -p macosx --appledev=y
```

#### macOS特定选项

```lua
-- 在xmake.lua中
if is_plat("macosx") then
    -- 链接框架
    add_frameworks("Cocoa", "Metal", "CoreVideo")

    -- 设置SDK路径
    add_sdkdirs()

    -- 启用ARC（自动引用计数）
    add_ldflags("-fobjc-arc")
end
```

### Android

#### 配置Android NDK

```bash
# 设置Android NDK路径
export ANDROID_NDK_HOME=/path/to/android-ndk

# 配置Android ARM64
xmake config -p android -a arm64-v8a -m release

# 配置Android ARMv7
xmake config -p android -a armeabi-v7a -m release

# 配置Android x86_64（模拟器）
xmake config -p android -a x86_64 -m debug
```

#### Android特定选项

```lua
-- Android工具链配置
toolchain("android-arm64")
    set_kind("standalone")
    set_sdkdir(os.getenv("ANDROID_NDK_HOME"))
    set_arch("arm64-v8a")

    -- 设置交叉编译器
    set_toolset("cc", "aarch64-linux-android-clang")
    set_toolset("cxx", "aarch64-linux-android-clang++")

    -- 添加编译标志
    add_cxxflags("-fPIC", "-DANDROID")
toolchain_end()
```

#### 生成APK

```bash
# 构建Android APK
xmake build-android

# 或使用gradle
cd android
./gradlew assembleDebug
```

### WebAssembly

#### 配置Emscripten

```bash
# 安装Emscripten SDK
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk
./emsdk install latest
./emsdk activate latest
source ./emsdk_env.sh

# 配置WebAssembly
export EMSCRIPTEN_ROOT=/path/to/emsdk/upstream/emscripten
xmake config -p wasm -m release
```

#### WebAssembly特定选项

```lua
-- WebAssembly工具链
toolchain("wasm")
    set_kind("standalone")
    set_sdkdir(os.getenv("EMSCRIPTEN_ROOT"))

    -- 设置Emscripten工具
    set_toolset("cc", "emcc")
    set_toolset("cxx", "em++")

    -- 添加WASM标志
    add_ldflags("-s WASM=1", "-s ALLOW_MEMORY_GROWTH=1")
toolchain_end()
```

#### 运行WebAssembly

```bash
# 构建WASM
xmake -p wasm

# 使用本地服务器运行
python -m http.server 8000

# 在浏览器中打开
# http://localhost:8000/build/wasm/release/index.html
```

---

## 交叉编译

### Linux到Windows

```bash
# 安装MinGW交叉编译工具
sudo apt-get install mingw-w64

# 配置交叉编译
xmake config -p windows --toolchain=gcc --cross=/usr/bin/x86_64-w64-mingw32-

# 构建
xmake
```

### Linux到Android

```bash
# 配置Android ARM64
export ANDROID_NDK_HOME=/path/to/ndk
xmake config -p android -a arm64-v8a -c

# 构建
xmake
```

### Linux到ARM64

```bash
# 安装ARM64交叉编译工具
sudo apt-get install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu

# 配置交叉编译
xmake config -p linux -a aarch64 --cross=aarch64-linux-gnu-

# 构建
xmake
```

### macOS到Universal Binary

```bash
# 构建Universal Binary（x86_64 + arm64）
xmake config -p macosx --appledev=y
xmake
```

---

## 高级功能

### 自定义任务

XMake支持自定义任务，在`xmake.lua`中定义：

```lua
-- 清理所有任务
task("clean-all")
    on_run(function ()
        os.exec("xmake clean")
        os.rm("build/**")
        print("Clean completed!")
    end)
task_end()

-- 运行任务
xmake clean-all
```

### 资源处理

```lua
-- 资源处理目标
target("game-resources")
    set_kind("phony")

    on_build(function (target)
        -- 复制资源文件
        os.cp("assets/**", "$(buildir)/assets")

        -- 压缩资源
        if is_mode("release") then
            os.exec("zip -r $(buildir)/assets.zip $(buildir)/assets")
        end
    end)
target_end()

-- 构建资源
xmake build game-resources
```

### 构建后脚本

```lua
target("game")
    -- ... 其他配置 ...

    -- 构建后执行
    after_build(function (target)
        -- 复制资源文件
        local assets_dir = path.absolute("assets")
        local target_dir = path.absolute(target:targetdir())

        if os.isdir(assets_dir) then
            os.cp(assets_dir, path.join(target_dir, "assets"))
        end

        -- 运行测试
        os.exec("$(targetdir)/game --test")
    end)
target_end()
```

### 安装和卸载

```bash
# 安装
xmake install

# 指定安装前缀
xmake install -o /usr/local

# 卸载
xmake uninstall
```

### 打包

```bash
# 创建发布包
xmake package

# 包含在xmake.lua中的打包任务
task("package")
    on_run(function ()
        -- 构建release版本
        os.exec("xmake config -m release")
        os.exec("xmake")

        -- 创建发布包
        local dist_dir = "dist"
        os.mkdir(dist_dir)

        -- 复制文件
        os.cp("build/release/game", path.join(dist_dir, "bin"))
        os.cp("assets", path.join(dist_dir, "share"))

        -- 打包
        os.exec("tar -czf game-engine-$(version).tar.gz " .. dist_dir)
    end)
task_end()
```

---

## 故障排除

### 常见问题

#### 1. 找不到Rust编译器

**问题**：
```
error: cannot run rustc
```

**解决方案**：
```bash
# 安装Rust工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 配置环境变量
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

#### 2. Android NDK未找到

**问题**：
```
error: android ndk not found!
```

**解决方案**：
```bash
# 设置ANDROID_NDK_HOME环境变量
export ANDROID_NDK_HOME=/path/to/android-ndk

# 添加到~/.bashrc或~/.zshrc
echo 'export ANDROID_NDK_HOME=/path/to/android-ndk' >> ~/.bashrc
```

#### 3. Emscripten未找到

**问题**：
```
error: emcc not found!
```

**解决方案**：
```bash
# 安装Emscripten SDK
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk
./emsdk install latest
./emsdk activate latest

# 激活环境
source ./emsdk_env.sh

# 或设置环境变量
export EMSCRIPTEN_ROOT=/path/to/emsdk/upstream/emscripten
```

#### 4. 权限错误（Linux/macOS）

**问题**：
```
error: permission denied
```

**解决方案**：
```bash
# 使用sudo安装xmake（如果需要）
sudo curl -fsSL https://xmake.io/sh | sh

# 或修复权限
sudo chown -R $USER ~/.xmake
```

#### 5. Windows路径错误

**问题**：
```
error: invalid path
```

**解决方案**：
```powershell
# 使用正斜杠或双反斜杠
# 正确：add_files("src/main.rs")
# 错误：add_files("src\main.rs")

# 或使用原始字符串
add_files([[src\main.rs]])
```

### 调试构建

```bash
# 启用详细输出
xmake -vD

# 显示详细配置信息
xmake f -v

# 查看构建日志
xmake --log=/tmp/xmake.log

# 追踪文件访问
xmake --trace
```

### 清理缓存

```bash
# 清理构建缓存
xmake clean

# 删除配置
rm -rf .xmake

# 删除构建目录
rm -rf build

# 清理所有
xmake clean-all
```

---

## 最佳实践

### 1. 使用配置文件

创建`xmake.conf`保存常用配置：

```bash
# xmake.conf
--mode=release
--platform=linux
--arch=x86_64
--verbose=y
```

使用配置文件：
```bash
xmake config -c xmake.conf
```

### 2. 分离Debug和Release构建

```bash
# 使用不同的构建目录
xmake config -m debug -o build_debug
xmake config -m release -o build_release

# 或使用全局配置
xmake g -m debug
xmake g -m release
```

### 3. 并行构建

```bash
# 使用所有CPU核心
xmake -j $(nproc)

# 或指定并行数
xmake -j 4
```

### 4. 增量编译

XMake自动支持增量编译，只重新编译修改的文件：

```bash
# 首次完整构建
xmake

# 修改源文件后，只重新编译修改的部分
xmake
```

### 5. 静态分析

```bash
# 运行Clippy
xmake lint

# 检查代码格式
xmake format -c

# 运行测试
xmake test
```

### 6. 持续集成

在CI/CD中使用XMake：

```yaml
# .github/workflows/build.yml
jobs:
  build:
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - name: Install XMake
        run: curl -fsSL https://xmake.io/sh | sh

      - name: Build
        run: |
          xmake config -m release
          xmake
          xmake test
```

### 7. 版本管理

```bash
# 在xmake.lua中设置版本
set_version("0.1.0", {build = "%Y%m%d%H%M"})

# 或使用Git标签
set_version("v0.1.0")
```

### 8. 依赖管理

```lua
-- 添加依赖包
add_requires("spdlog", "glfw")

-- 在target中使用
add_packages("spdlog", "glfw")

-- 指定版本
add_requires("spdlog ~>1.10", "glfw 3.3.x")
```

---

## CLI命令生成XMake配置

使用CLI工具生成XMake配置：

```bash
# 生成XMake配置文件
game-engine build-system --system xmake

# 指定输出目录
game-engine build-system --system xmake --output ./my-project

# 强制覆盖已存在的配置
game-engine build-system --system xmake --force
```

这将生成完整的`xmake.lua`配置文件，包含：
- 跨平台支持
- 资源处理
- 自定义任务
- 工具链配置

---

## 参考资源

- [XMake官方文档](https://xmake.io/#/)
- [XMake GitHub仓库](https://github.com/xmake-io/xmake)
- [Rust交叉编译指南](https://rust-lang.github.io/rustc/platform-support.html)
- [Emscripten文档](https://emscripten.org/docs/)
- [Android NDK指南](https://developer.android.com/ndk/guides)

---

## 获取帮助

```bash
# XMake帮助
xmake help

# 特定命令帮助
xmake help config
xmake help build

# 查看版本
xmake --version

# 报告问题
# https://github.com/xmake-io/xmake/issues
```

---

**文档版本**: v0.1.0
**最后更新**: 2025-12-31
**维护者**: Game Engine Team
