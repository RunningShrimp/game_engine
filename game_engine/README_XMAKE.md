# XMake Build System Support

本项目已完全集成XMake构建系统，提供跨平台构建能力。

## 快速开始

### 1. 生成XMake配置

使用CLI工具生成XMake配置文件：

```bash
cargo run --bin game-engine -- build-system --system xmake
```

选项：
- `--system`: 构建系统类型（xmake, cmake）
- `--output`: 输出目录（默认当前目录）
- `--force`: 强制覆盖已存在的配置

### 2. 基本构建

```bash
# 配置为Debug模式
xmake config -m debug

# 构建
xmake

# 运行
xmake run
```

### 3. Release构建

```bash
# 配置为Release模式
xmake config -m release

# 构建
xmake

# 运行
xmake run
```

## 支持的平台

- **Windows** (MSVC, MinGW)
- **Linux** (GCC, Clang)
- **macOS** (Clang)
- **Android** (NDK)
- **WebAssembly** (Emscripten)

## 跨平台编译

### Android

```bash
# 配置Android环境
export ANDROID_NDK_HOME=/path/to/android-ndk

# 配置ARM64
xmake config -p android -a arm64-v8a -m release

# 构建
xmake
```

### WebAssembly

```bash
# 配置Emscripten环境
export EMSCRIPTEN_ROOT=/path/to/emscripten

# 配置WASM
xmake config -p wasm -m release

# 构建
xmake
```

## 自定义任务

```bash
# 清理所有
xmake clean-all

# 格式化代码
xmake format

# 运行linter
xmake lint

# 运行测试
xmake test

# 生成文档
xmake docs
```

## 文件结构

```
game_engine/
├── xmake.lua                    # 主配置文件
├── templates/
│   └── xmake/
│       ├── xmake.lua.hbs        # Handlebars模板
│       └── metadata.json        # 模板元数据
├── .github/
│   └── workflows/
│       └── xmake.yml            # CI/CD配置
└── docs/
    └── xmake_build_guide.md     # 详细文档
```

## 完整文档

查看 [`docs/xmake_build_guide.md`](docs/xmake_build_guide.md) 获取完整的构建指南，包括：

- 详细配置选项
- 平台特定配置
- 交叉编译指南
- 故障排除
- 最佳实践

## CLI集成

XMake已完全集成到CLI工具中：

```bash
# 显示帮助
game-engine build-system --help

# 生成配置
game-engine build-system --system xmake

# 强制覆盖
game-engine build-system --system xmake --force
```

## 贡献

欢迎贡献！请提交问题或拉取请求。

## 许可证

MIT OR Apache-2.0
