# 配置系统

引擎提供全面的配置管理系统，支持TOML、JSON等多种格式，并允许运行时动态调整。

## 配置文件概述

### 默认配置查找顺序

引擎按以下顺序查找配置文件：

1. `./config.toml` - 当前目录的TOML文件
2. `./config.json` - 当前目录的JSON文件
3. `~/.config/game_engine/config.toml` - 用户主目录配置
4. 最后使用内置默认值

### 配置文件结构

所有配置都位于根级对象下，包含以下主要部分：

```toml
[graphics]          # 图形渲染配置
[performance]       # 性能优化配置
[audio]            # 音频系统配置
[input]            # 输入设备配置
[logging]          # 日志系统配置
```

## 图形配置

### 分辨率设置

```toml
[graphics.resolution]
width = 1920
height = 1080
```

### 显示选项

```toml
[graphics]
vsync = true
fullscreen = false
anti_aliasing = "TAA"      # None, FXAA, MSAA, TAA
shadow_quality = "High"    # Low, Medium, High, Ultra
texture_quality = "High"   # Low, Medium, High, Ultra
effects_quality = "High"   # Low, Medium, High, Ultra
```

### 光线追踪

```toml
[graphics.ray_tracing]
enabled = false
shadows = false
reflections = false
global_illumination = false
ambient_occlusion = false
```

### 超分辨率上采样

```toml
[graphics.upscaling]
enabled = true
technology = "Auto"       # Auto, DLSS, FSR, XeSS
quality = "Quality"       # UltraQuality, Quality, Balanced, Performance
render_scale = 0.67       # 渲染分辨率缩放比例 (0.5-1.0)
```

## 性能配置

### 基本性能设置

```toml
[performance]
target_fps = 60
auto_optimize = true
```

### SIMD优化

```toml
[performance.simd]
enabled = true
force_instruction_set = null    # "SSE2", "AVX", "AVX2", "AVX512", "NEON"
batch_size = 1000
```

### NPU硬件加速

```toml
[performance.npu]
enabled = true
backend = "Auto"         # Auto, OnnxRuntime, TensorRT, CoreML, DirectML
ai_upscaling = false
physics_prediction = false
```

### 多线程配置

```toml
[performance.threading]
worker_threads = 0       # 0 = 自动检测CPU核心数
render_threads = 1
physics_threads = 1
```

### 内存管理

```toml
[performance.memory]
texture_cache_mb = 512
model_cache_mb = 256
audio_cache_mb = 128
use_object_pools = true
```

## 音频配置

```toml
[audio]
# 音量控制 (0.0 - 1.0)
master_volume = 1.0
music_volume = 0.8
sfx_volume = 1.0
voice_volume = 1.0

# 音频输出设置
sample_rate = 48000
buffer_size = 1024
muted = false
```

## 输入配置

### 鼠标设置

```toml
[input]
mouse_sensitivity = 1.0
mouse_invert_y = false
```

### 手柄设置

```toml
[input]
gamepad_deadzone = 0.1    # 0.0-1.0 死区范围
gamepad_vibration = true
```

### 键盘映射

```toml
[input.key_bindings]
forward = "W"
backward = "S"
left = "A"
right = "D"
jump = "Space"
crouch = "C"
sprint = "Shift"
interact = "E"
```

## 日志配置

```toml
[logging]
level = "Info"               # Trace, Debug, Info, Warn, Error
log_to_file = false
log_file_path = "game_engine.log"
log_to_console = true
```

## 在代码中使用配置

### Rust API使用

```rust
use game_engine::config::{EngineConfig, GraphicsConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载配置文件
    let mut config = EngineConfig::load_or_default();

    // 修改运行时配置
    config.graphics.resolution.width = 2560;
    config.graphics.resolution.height = 1440;

    // 应用环境变量覆盖
    config.apply_env_overrides();

    // 验证配置
    config.validate()?;

    // 保存配置
    config.save_toml("my_config.toml")?;

    Ok(())
}
```

### 运行时配置热重载

引擎支持运行时配置热重载，但某些设置需要重启生效：

```rust
// 即时生效的设置
- 音量控制
- 鼠标灵敏度
- 某些渲染设置

// 需要重启的设置
- 分辨率
- 全屏模式
- VSync
- 深度缓冲格式
```

## 环境变量覆盖

可以通过环境变量覆盖配置文件设置：

```bash
# 图形设置
export ENGINE_GRAPHICS_WIDTH=2560
export ENGINE_GRAPHICS_HEIGHT=1440
export ENGINE_GRAPHICS_VSYNC=true

# 性能设置
export ENGINE_PERFORMANCE_TARGET_FPS=144
export ENGINE_PERFORMANCE_AUTO_OPTIMIZE=false

# 音频设置
export ENGINE_AUDIO_MASTER_VOLUME=0.8
```

## 配置验证

引擎会在启动时自动验证配置，如果发现无效设置会：

1. 记录警告信息
2. 使用合理的默认值
3. 继续启动（大多数情况）

严重的配置错误（如无效的分辨率）会阻止引擎启动。

## 最佳实践

### 开发环境配置
```toml
[logging]
level = "Debug"
log_to_file = true

[performance]
target_fps = 30
auto_optimize = false
```

### 生产环境配置
```toml
[logging]
level = "Warn"
log_to_file = true

[performance]
target_fps = 60
auto_optimize = true

[graphics]
vsync = true
anti_aliasing = "TAA"
shadow_quality = "High"
```

### 低端设备优化
```toml
[graphics]
anti_aliasing = "FXAA"
shadow_quality = "Low"
texture_quality = "Medium"
effects_quality = "Low"

[graphics.upscaling]
enabled = true
technology = "FSR"
quality = "Performance"
render_scale = 0.5

[performance]
target_fps = 30
use_object_pools = true

[performance.memory]
texture_cache_mb = 256
model_cache_mb = 128
audio_cache_mb = 64
```

## 故障排除

### 常见问题

**配置不生效**
- 确保文件格式正确（TOML语法）
- 检查文件权限
- 验证文件路径
- 重启应用程序

**性能设置无效**
- 某些硬件加速特性需要特定硬件支持
- 检查驱动版本和硬件兼容性
- 查看控制台日志以获取详细错误信息

**音频配置问题**
- 验证音频设备支持
- 检查采样率兼容性
- 确认缓冲区大小在合理范围内