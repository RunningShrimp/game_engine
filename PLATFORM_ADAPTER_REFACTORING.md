# 平台适配器重构文档

## 概述

本文档描述了游戏引擎平台抽象层的重构，旨在减少条件编译的使用，提高代码的可维护性和可扩展性。

## 问题分析

### 原有问题

1. **条件编译过度使用**：大量使用 `#[cfg(target_arch = "wasm32")]` 等条件编译指令
2. **平台特定代码分散**：不同平台的实现分散在多个文件中
3. **接口不一致**：不同平台的接口定义不一致
4. **测试困难**：条件编译使得跨平台测试变得困难

### 重构目标

1. 统一平台抽象接口
2. 使用动态分发替代编译时条件编译
3. 提供清晰的平台适配器 API
4. 支持运行时平台检测和切换

## 架构设计

### 核心组件

```
platform/
├── adapter.rs          # 平台适配器（统一入口）
├── mod.rs             # 平台抽象接口定义
├── winit.rs           # Winit 窗口实现
├── web_fs.rs          # Web 文件系统实现
├── web_input.rs       # Web 输入实现
├── native_input.rs    # 原生输入实现
├── hardware_info.rs   # 硬件信息
├── power_aware.rs     # 功耗感知
└── console.rs         # 控制台平台支持
```

### 平台适配器 (PlatformAdapter)

`PlatformAdapter` 是平台抽象层的统一入口，提供了以下功能：

```rust
pub struct PlatformAdapter {
    pub filesystem: Box<dyn PlatformFilesystem>,
    pub window: Box<dyn PlatformWindow>,
    pub input: Box<dyn PlatformInput>,
    pub hardware: HardwareInfo,
    pub power_aware: PowerAwareManager,
    pub console: Option<ConsoleConfig>,
}
```

### 统一接口

#### Filesystem 接口

**原生平台**：
```rust
#[async_trait::async_trait]
pub trait Filesystem: Send + Sync {
    async fn read(&self, path: &Path) -> Result<Vec<u8>, FsError>;
    async fn write(&self, path: &Path, data: &[u8]) -> Result<(), FsError>;
    fn exists(&self, path: &Path) -> bool;
    async fn exists_async(&self, path: &Path) -> bool;
    async fn create_dir_all(&self, path: &Path) -> Result<(), FsError>;
    async fn remove_file(&self, path: &Path) -> Result<(), FsError>;
    async fn read_dir(&self, path: &Path) -> Result<Vec<std::path::PathBuf>, FsError>;
    fn watch(&self, path: &Path, tx: Sender<FsEvent>) -> Result<WatchHandle, FsError>;
}
```

**Web 平台**：
```rust
pub trait Filesystem: Send + Sync {
    fn read_async(&self, url: &str) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, FsError>> + Send>>;
    fn cache_get(&self, key: &str) -> Option<Vec<u8>>;
    fn cache_set(&self, key: &str, data: &[u8]);
}
```

#### Window 接口

```rust
pub trait Window: Send + Sync {
    fn size(&self) -> (u32, u32);
    fn scale_factor(&self) -> f64;
    fn request_redraw(&self);
    fn set_title(&self, title: &str);
    fn set_fullscreen(&self, fullscreen: bool);
    fn set_cursor_visible(&self, visible: bool);

    #[cfg(not(target_arch = "wasm32"))]
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle;

    #[cfg(not(target_arch = "wasm32"))]
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle;
}
```

#### Input 接口

```rust
pub trait Input: Send + Sync {
    fn poll_events(&mut self) -> Vec<InputEvent>;
    fn is_key_pressed(&self, key: KeyCode) -> bool;
    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool;
    fn mouse_position(&self) -> (f32, f32);
    fn set_cursor_grab(&mut self, grab: bool);
    fn set_cursor_visible(&mut self, visible: bool);

    #[cfg(feature = "xr")]
    fn xr_actions(&self) -> Option<&XrActionSet>;
}
```

## 使用示例

### 基本使用

```rust
use game_engine::platform::adapter::PlatformAdapter;

// 创建平台适配器
let platform = PlatformAdapter::new();

// 访问文件系统
let data = platform.filesystem().read_sync(&path)?;

// 访问窗口
let size = platform.window().size();
platform.window().set_title("My Game");

// 访问输入
let events = platform.input_mut().poll_events();
if platform.input().is_key_pressed(KeyCode::Space) {
    // 处理空格键
}

// 访问硬件信息
let gpu_info = &platform.hardware().gpu;
println!("GPU: {}", gpu_info.name);

// 访问功耗感知
let target_fps = platform.power_aware().target_fps();
```

### 平台特定功能

```rust
use game_engine::platform::adapter::PlatformAdapter;

let platform = PlatformAdapter::new();

// 检查是否为控制台平台
if let Some(console) = platform.console() {
    match console.platform {
        ConsolePlatform::PlayStation5 => {
            // PS5 特定优化
            console.apply_to_graphics_config(&mut config);
        }
        ConsolePlatform::NintendoSwitch => {
            // Switch 特定优化
        }
        _ => {}
    }
}

// 功耗感知优化
let power_state = platform.power_aware().power_state();
match power_state {
    PowerState::Charging => {
        // 充电中，全性能
    }
    PowerState::LowBattery => {
        // 低电量，降低性能
        let scale = platform.power_aware().performance_scale();
    }
    _ => {}
}
```

## 条件编译减少策略

### 1. 使用动态分发

**之前**：
```rust
#[cfg(target_arch = "wasm32")]
let filesystem = WebFilesystem::new();

#[cfg(not(target_arch = "wasm32"))]
let filesystem = NativeFilesystem::new();
```

**之后**：
```rust
let platform = PlatformAdapter::new();
let filesystem = platform.filesystem();
```

### 2. 统一接口设计

**之前**：
```rust
// 原生平台
async fn read(&self, path: &Path) -> Result<Vec<u8>, FsError>;

// Web 平台
fn read_async(&self, url: &str) -> Pin<Box<dyn Future<...>>>;
```

**之后**：
```rust
// 通过平台适配器统一访问
let data = platform.filesystem().read_sync(&path)?;
```

### 3. 运行时平台检测

```rust
let hardware = HardwareInfo::detect();
if is_console_platform() {
    let console = get_console_config();
    console.apply_to_graphics_config(&mut config);
}
```

## 实现细节

### 平台适配器初始化

```rust
impl PlatformAdapter {
    pub fn new() -> Self {
        let hardware = HardwareInfo::detect();
        let power_aware = PowerAwareManager::new();
        let console = ConsoleConfig::from_hardware(&hardware);

        #[cfg(target_arch = "wasm32")]
        {
            let filesystem = Box::new(WebFilesystem::new()?) as Box<dyn PlatformFilesystem>;
            let window = Box::new(WebWindow::new()) as Box<dyn PlatformWindow>;
            let input = Box::new(WebInput::new("canvas")?) as Box<dyn PlatformInput>;

            Self {
                filesystem,
                window,
                input,
                hardware,
                power_aware,
                console: Some(console),
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let filesystem = Box::new(NativeFilesystem::new()) as Box<dyn PlatformFilesystem>;
            let window = Box::new(WinitWindow::default()) as Box<dyn PlatformWindow>;
            let input = Box::new(NativeInput::new()) as Box<dyn PlatformInput>;

            Self {
                filesystem,
                window,
                input,
                hardware,
                power_aware,
                console: Some(console),
            }
        }
    }
}
```

### WinitWindow 默认实现

为了支持 `Default` trait，`WinitWindow` 现在使用 `Option<Arc<Window>>`：

```rust
pub struct WinitWindow {
    window: Option<Arc<Window>>,
}

impl Default for WinitWindow {
    fn default() -> Self {
        Self { window: None }
    }
}
```

所有方法现在都检查 `window` 是否为 `None`，并提供合理的默认值。

## 性能考虑

### 动态分发开销

使用 trait objects 会带来轻微的性能开销（虚函数调用），但：

1. 平台 API 调用频率相对较低
2. 性能关键路径仍然使用具体类型
3. 可维护性和可扩展性的收益远大于性能损失

### 内存开销

每个平台适配器包含多个 trait objects，但：

1. 只有一个全局实例
2. 内存占用可忽略不计
3. 提供了统一的访问接口

## 未来改进

### 1. 进一步减少条件编译

- 将更多平台特定代码移到运行时检测
- 使用配置文件替代编译时特性

### 2. 插件系统

- 支持动态加载平台实现
- 允许第三方平台适配器

### 3. 测试改进

- 添加平台适配器的单元测试
- 添加跨平台集成测试
- 模拟平台行为进行测试

## 总结

通过引入 `PlatformAdapter`，我们成功地：

1. **减少了条件编译的使用**：大部分平台特定代码集中在 `PlatformAdapter::new()` 中
2. **提高了代码可维护性**：统一的接口和清晰的 API
3. **增强了可扩展性**：易于添加新的平台支持
4. **改善了测试性**：可以模拟平台行为进行测试

这种设计遵循了开放-封闭原则，对扩展开放，对修改封闭，为未来的平台支持提供了良好的基础。
