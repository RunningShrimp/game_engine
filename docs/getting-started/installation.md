# 安装指南

本页引导您从源码开始构建和安装游戏引擎。

## 系统要求

### 最低要求
- Rust 1.70+ `rustup install stable`
- Windows 10+ / macOS 10.15+ / Ubuntu 18.04+
- CUDA 11.0+（可选，用于GPU计算）
- 4GB RAM
- Vulkan兼容显卡（或通过软件渲染）

### 推荐配置
- Rust 1.75+
- Windows 11 / macOS 13+ / Ubuntu 22.04+
- NVIDIA RTX系列显卡 + CUDA 12.0+
- 16GB+ RAM
- SSD存储

## 从源码构建

### 1. 克隆仓库

```bash
git clone https://github.com/your-username/game_engine.git
cd game_engine
```

### 2. 安装依赖

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install build-essential cmake pkg-config libxcb-shape0-dev libxcb-xfixes0-dev libasound2-dev
```

#### macOS
```bash
brew install cmake pkg-config
```

#### Windows
通过以下方式安装依赖：
- 安装 [Build Tools for Visual Studio](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
- 或者安装 [MSYS2](https://www.msys2.org/) 配合 MinGW

### 3. 构建引擎

```bash
# 调试版本（推荐用于开发）
cargo build

# 发布版本（推荐用于生产）
cargo build --release

# 可选：运行测试
cargo test
```

### 4. 运行示例

```bash
# 演示应用
cargo run

# 特定示例
cargo run --example physics_demo
cargo run --example hardware_optimization
```

## 高级安装选项

### GPU加速支持

#### CUDA集成
```bash
# 安装CUDA工具包
# 然后启用特性
cargo build --release --features cuda_acceleration
```

#### ROCm集成（AMD）
```bash
# 安装ROCm
# 然后启用特性
cargo build --release --features rocm_acceleration
```

#### CoreML集成（Apple Silicon）
```bash
cargo build --release --features coreml_acceleration
```

### 性能优化构建

```bash
# SIMD优化（默认启用）
cargo build --release

# 神经网络上采样
cargo build --release --features neural_upscaling

# 高级硬件检测
cargo build --release --features hardware_detection
```

## 故障排除

### 编译错误

#### Vulkan SDK 问题
```bash
# Linux: 安装Vulkan开发包
sudo apt install vulkan-tools libvulkan-dev vulkan-validationlayers-dev spirv-tools

# macOS: Vulkan SDK 需要手动安装
# 参考 https://vulkan.lunarg.com/sdk/home
```

#### 依赖问题
```bash
# 清理缓存后重新构建
cargo clean
cargo update
cargo build
```

### 运行时错误

#### 显卡驱动问题
- NVIDIA: 确保安装最新的显卡驱动
- AMD: 更新到最新 Mesa 版本
- Intel: 使用最新的内核驱动

#### 权限问题
```bash
# Linux: 添加用户到video组
sudo usermod -a -G video $USER
```

### 寻求帮助

如果遇到问题，请：
1. 查看[问题追踪](https://github.com/your-username/game_engine/issues)
2. 运行 `cargo build --verbose` 获取详细错误信息
3. 在论坛或Discord寻求社区帮助