# 鸿蒙系统 (HarmonyOS) 支持文档

## 概述

**任务**: P2-2.1 - 鸿蒙系统支持
**状态**: ✅ 已实现框架
**工期**: 4周
**文件位置**:
- `game_engine/src/platform/harmonyos.rs`
- `game_engine/src/platform/harmonyos_wgpu.rs`

---

## 什么是鸿蒙系统?

**HarmonyOS (鸿蒙)** 是华为开发的自主操作系统，基于微内核架构，支持多种设备类型：

- **智能手机和平板**
- **智能穿戴设备**
- **智慧屏**
- **车载系统**
- **IoT设备**

### 关键特性

1. **微内核架构**: 更小的内核，更高的安全性
2. **分布式能力**: 跨设备协同
3. **方舟开发框架 (Ark)**: 原生应用开发
4. **多设备适配**: 一次开发，多端部署

---

## 实现的功能

### 1. 平台检测

```rust
use game_engine::platform::harmonyos::{is_harmonyos, platform_info};

if is_harmonyos() {
    let info = platform_info();
    println!("HarmonyOS Version: {:?}", info.version);
    println!("Device Type: {:?}", info.device_type);
    println!("API Level: {}", info.api_level);
}
```

### 2. 窗口管理

```rust
use game_engine::platform::harmonyos::{HarmonyOSWindow, HarmonyOSWindowConfig};

let config = HarmonyOSWindowConfig {
    width: 1920,
    height: 1080,
    title: "My Game".to_string(),
    fullscreen: false,
    resizable: true,
    vsync: true,
};

let mut window = HarmonyOSWindow::new(config)?;
window.show();

// 全屏切换
window.set_fullscreen(true)?;

// 获取显示指标
let metrics = window.display_metrics();
println!("Display: {}x{}, DPI: {}", metrics.width, metrics.height, metrics.dpi);
```

### 3. 输入处理

```rust
use game_engine::platform::harmonyos::HarmonyOSInputManager;

let mut input_manager = HarmonyOSInputManager::new();

// 轮询触摸事件
let touch_events = input_manager.poll_touch_events();
for event in touch_events {
    println!("Touch: {:?} at ({}, {})", event.action, event.x, event.y);
}

// 处理原生触摸事件（从鸿蒙回调）
unsafe {
    input_manager.handle_native_touch_event(raw_event_ptr);
}
```

### 4. 图形上下文

```rust
use game_engine::platform::harmonyos::{HarmonyOSGraphicsContext, GraphicsBackend};

let graphics_context = HarmonyOSGraphicsContext::new(
    &window,
    GraphicsBackend::Vulkan,  // 或 GraphicsBackend::OpenGLES
)?;

println!("Graphics Backend: {:?}", graphics_context.backend());
```

### 5. WebGPU集成

```rust
use game_engine::platform::harmonyos_wgpu::{
    HarmonyOSWgpuInstance, HarmonyOSWgpuDevice, HarmonyOSWgpuContext,
    AdapterPreference, HarmonyOSWgpuConfig,
};

// 创建WebGPU实例
let instance = HarmonyOSWgpuInstance::new();

// 创建Surface
let surface_creator = HarmonyOSWgpuSurfaceCreator::new(&graphics_context);
let surface = instance.create_surface(&surface_creator)?;

// 请求高性能适配器
let adapter = instance.request_adapter(&surface, AdapterPreference::HighPerformance).await?;

// 创建设备
let device = HarmonyOSWgpuDevice::new(adapter).await?;

// 创建渲染上下文
let config = HarmonyOSWgpuConfig::default();
let context = HarmonyOSWgpuContext::new(
    surface,
    device,
    config,
    1920,
    1080,
).await?;

// 获取当前帧
let frame_view = context.get_current_frame()?;
```

### 6. 权限管理

```rust
use game_engine::platform::harmonyos::{PermissionManager, HarmonyOSPermission};

// 检查权限
if !PermissionManager::check_permission(HarmonyOSPermission::Storage) {
    // 请求权限
    let granted = PermissionManager::request_permission(
        HarmonyOSPermission::Storage
    ).await?;

    if granted {
        println!("Storage permission granted");
    }
}
```

### 7. 资源访问

```rust
use game_engine::platform::harmonyos::resolve_resource_path;

// 解析资源路径
let asset_path = resolve_resource_path("textures/player.png");
// 结果: /data/storage/el2/base/haps/entry/files/textures/player.png
```

---

## 数据结构

### HarmonyOSVersion

```rust
pub struct HarmonyOSVersion {
    pub major: u32,     // 主版本号 (3)
    pub minor: u32,     // 次版本号 (0)
    pub patch: u32,     // 补丁版本 (0)
    pub build: String,  // 构建信息 ("HarmonyOS 3.0")
}
```

### HarmonyOSPlatformInfo

```rust
pub struct HarmonyOSPlatformInfo {
    pub version: Option<HarmonyOSVersion>,
    pub device_type: DeviceType,
    pub api_level: u32,
}

pub enum DeviceType {
    Phone,
    Tablet,
    IoT,
    Car,
    TV,
    Unknown,
}
```

### HarmonyOSWindowConfig

```rust
pub struct HarmonyOSWindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub fullscreen: bool,
    pub resizable: bool,
    pub vsync: bool,
}
```

### GraphicsBackend

```rust
pub enum GraphicsBackend {
    Vulkan,     // Vulkan渲染API
    OpenGLES,   // OpenGL ES渲染API
}
```

---

## WebGPU特性

### 适配器选择

```rust
pub enum AdapterPreference {
    HighPerformance,  // 高性能GPU（独显）
    LowPower,         // 低功耗（集显）
    Any,              // 任何适配器
}
```

### GPU信息

```rust
pub struct HarmonyOSGpuInfo {
    pub name: String,         // GPU名称
    pub vendor: String,       // 供应商
    pub driver: String,       // 驱动版本
    pub driver_info: String,  // 驱动信息
    pub backend: wgpu::Backend,  // 后端类型
}
```

### 性能提示

```rust
pub enum PerformanceHint {
    LowPower,         // 低功耗模式
    Balanced,         // 平衡模式
    HighPerformance,  // 高性能模式
}

pub fn set_performance_hint(hint: PerformanceHint);
```

---

## 支持的图形API

### Vulkan

**检查支持**:
```rust
use game_engine::platform::harmonyos_wgpu::is_vulkan_supported;

if is_vulkan_supported() {
    println!("Vulkan is supported");
}
```

**扩展**:
- VK_KHR_surface
- VK_KHR_swapchain
- VK_EXT_hdr_metadata

### OpenGL ES

**检查支持**:
```rust
use game_engine::platform::harmonyos_wgpu::is_opengles_supported;

if is_opengles_supported() {
    println!("OpenGL ES is supported");
}
```

**扩展**:
- GL_OES_EGL_image
- GL_EXT_texture_rg
- GL_OES_texture_float

---

## 权限类型

```rust
pub enum HarmonyOSPermission {
    Internet,    // 网络访问
    Storage,     // 存储读写
    Camera,      // 相机
    Microphone,  // 麦克风
    Location,    // 位置信息
    Vibrate,     // 震动
}
```

---

## 使用示例

### 完整游戏循环初始化

```rust
use game_engine::platform::harmonyos::*;
use game_engine::platform::harmonyos_wgpu::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 检查平台
    if !is_harmonyos() {
        return Err("Not running on HarmonyOS".into());
    }

    // 2. 获取平台信息
    let info = platform_info();
    println!("Platform: HarmonyOS {:?}", info.version);

    // 3. 创建窗口
    let window_config = HarmonyOSWindowConfig::default();
    let window = HarmonyOSWindow::new(window_config)?;
    window.show();

    // 4. 创建输入管理器
    let mut input_manager = HarmonyOSInputManager::new();

    // 5. 创建图形上下文
    let graphics_context = HarmonyOSGraphicsContext::new(
        &window,
        GraphicsBackend::Vulkan,
    )?;

    // 6. 创建WebGPU实例
    let wgpu_instance = HarmonyOSWgpuInstance::new();
    let surface_creator = HarmonyOSWgpuSurfaceCreator::new(&graphics_context);
    let surface = wgpu_instance.create_surface(&surface_creator)?;

    // 7. 请求高性能适配器
    let adapter = wgpu_instance.request_adapter(
        &surface,
        AdapterPreference::HighPerformance,
    ).await?;

    // 8. 创建设备
    let wgpu_device = HarmonyOSWgpuDevice::new(adapter).await?;

    // 9. 创建渲染上下文
    let wgpu_config = HarmonyOSWgpuConfig::default();
    let render_context = HarmonyOSWgpuContext::new(
        surface,
        wgpu_device,
        wgpu_config,
        1920,
        1080,
    ).await?;

    // 10. 游戏循环
    loop {
        // 处理输入
        let touch_events = input_manager.poll_touch_events();
        for event in touch_events {
            // 处理触摸事件
        }

        // 渲染
        let frame_view = render_context.get_current_frame()?;
        // ... 渲染逻辑 ...

        // 呈现
        render_context.present();
    }
}
```

---

## 编译和部署

### Feature Flag

```toml
# Cargo.toml
[features]
harmonyos = []
```

### 编译命令

```bash
# 启用鸿蒙支持编译
cargo build --features harmonyos

# 发布版本
cargo build --release --features harmonyos
```

### 鸿蒙NDK

**要求**:
- 鸿蒙NDK r3+
- Rust工具链
- Native API绑定

**当前状态**:
- ✅ 框架实现完成
- ⚠️ 需要完整的鸿蒙API绑定
- ⚠️ 需要实际设备测试

---

## 架构设计

### 模块结构

```
game_engine/src/platform/
├── harmonyos.rs           # 鸿蒙平台核心功能
│   ├── 平台检测
│   ├── 窗口管理
│   ├── 输入处理
│   ├── 权限管理
│   └── 资源访问
│
└── harmonyos_wgpu.rs      # WebGPU集成
    ├── Surface创建
    ├── 适配器选择
    ├── 设备初始化
    └── 渲染上下文
```

### 集成点

1. **平台抽象层**: 与其他平台（Android, iOS, Desktop）统一的API
2. **图形后端**: 通过WebGPU实现跨平台图形
3. **输入系统**: 统一的输入事件抽象
4. **文件系统**: 统一的资源访问接口

---

## 限制和注意事项

### 当前限制

1. **API绑定不完整**: 需要完整的鸿蒙Native API FFI绑定
2. **无实际设备测试**: 框架实现，需要真实设备验证
3. **性能未优化**: 未针对鸿蒙特性优化

### 已知问题

1. **raw-window-handle集成**: 需要实现鸿蒙窗口句柄适配
2. **Vulkan/OpenGL ES支持**: 需要实际测试
3. **权限系统**: 可能需要额外配置

### 改进方向

1. **完整API绑定**: 使用bindgen生成FFI绑定
2. **性能优化**: 针对鸿蒙微内核优化
3. **分布式特性**: 利用鸿蒙分布式能力
4. **方舟编译器**: 支持方舟编译器优化

---

## 与其他平台对比

| 特性 | HarmonyOS | Android | iOS |
|------|-----------|---------|-----|
| 微内核 | ✅ | ❌ | ❌ (混合) |
| 分布式 | ✅ | ❌ | ❌ |
| Vulkan | ✅ | ✅ | ✅ (部分) |
| OpenGL ES | ✅ | ✅ | ✅ |
| WebGPU | ✅ | ✅ | ⚠️ (实验) |
| 方舟编译器 | ✅ | ❌ | ❌ |

---

## 参考资源

### 官方文档

- [HarmonyOS开发者官网](https://developer.huawei.com/consumer/cn/)
- [HarmonyOS应用开发文档](https://developer.huawei.com/consumer/cn/doc/harmonyos-guides-V5/)
- [Native API参考](https://developer.huawei.com/consumer/cn/doc/harmonyos-references-V5/)

### 技术规范

- 鸿蒙系统架构规范
- Native API接口规范
- Vulkan/OpenGL ES实现指南

### 社区资源

- 鸿蒙开发者论坛
- GitHub鸿蒙开源项目
- Rust for HarmonyOS工作组

---

## 测试

### 单元测试

```bash
# 运行鸿蒙相关测试
cargo test --features harmonyos harmonyos
```

### 测试覆盖

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_harmonyos_detection() {
        // 在非鸿蒙平台应返回false
        #[cfg(not(feature = "harmonyos"))]
        assert!(!is_harmonyos());

        // 在鸿蒙平台应返回true
        #[cfg(feature = "harmonyos")]
        assert!(is_harmonyos());
    }

    #[test]
    fn test_version_string() {
        let version = HarmonyOSVersion {
            major: 3,
            minor: 0,
            patch: 0,
            build: "HarmonyOS 3.0".to_string(),
        };

        assert_eq!(version.to_string(), "3.0.0-HarmonyOS 3.0");
    }
}
```

---

## 故障排除

### 问题1: 编译失败

**错误**: `cannot find harmonyos in this scope`

**解决方案**:
```bash
# 确保启用了harmonyos feature
cargo build --features harmonyos
```

### 问题2: 运行时崩溃

**原因**: 鸿蒙API调用失败

**解决方案**:
1. 确保在鸿蒙系统上运行
2. 检查Native API可用性
3. 查看日志错误信息

### 问题3: 窗口无法创建

**原因**: 鸿蒙NativeWindow API未正确绑定

**解决方案**:
1. 实现完整的OH_NativeWindow FFI绑定
2. 检查窗口参数
3. 验证图形后端支持

---

## 下一步

### P2-2.2: 集成显卡优化

为鸿蒙设备的集成GPU优化渲染策略。

### P2-2.3: 移动端Tile-based优化

针对移动GPU的Tile-based渲染架构优化。

### P2-2.4: ARM NEON优化

为ARM架构启用NEON SIMD加速。

---

## 总结

P2-2.1任务已完成鸿蒙系统支持的框架实现：

✅ **平台检测** - 自动识别鸿蒙系统
✅ **窗口管理** - 鸿蒙原生窗口抽象
✅ **输入处理** - 触摸事件处理框架
✅ **图形集成** - WebGPU/Vulkan/OpenGL ES支持
✅ **权限管理** - 权限请求和检查
✅ **资源访问** - 鸿蒙文件系统路径解析
✅ **文档** - 完整的API文档和使用指南

**状态**: 框架实现完成，需要完整的鸿蒙API绑定和实际设备测试。

**下一步**: P2-2.2 - 集成显卡优化

---

**文档版本**: v1.0
**完成日期**: 2025-12-31
**作者**: Claude Code
**状态**: ✅ P2-2.1框架实现完成
