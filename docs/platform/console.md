# 控制台平台支持

## 支持的平台

- PlayStation 5
- PlayStation 4
- Xbox Series X/S
- Xbox One
- Nintendo Switch

## 平台检测

引擎会自动检测当前运行的控制台平台：

```rust
use game_engine::platform::console::{is_console_platform, get_console_config};

if is_console_platform() {
    if let Some(config) = get_console_config() {
        println!("Running on: {:?}", config.platform);
    }
}
```

## 配置

### 性能模式 vs 质量模式

控制台平台支持两种模式：

- **性能模式**: 降低分辨率，提高帧率
- **质量模式**: 提高分辨率，可能降低帧率

```rust
use game_engine::platform::console::ConsoleConfig;

let mut config = ConsoleConfig::default();
config.performance_mode = true; // 启用性能模式
config.quality_mode = false;
```

## 输入处理

### 控制器支持

```rust
use game_engine::platform::console::ConsoleInputHandler;

let mut input_handler = ConsoleInputHandler::new();

// 更新控制器状态
input_handler.update_controller(0, controller_state);

// 获取控制器状态
if let Some(controller) = input_handler.get_controller(0) {
    let left_stick = controller.left_stick;
    let a_pressed = controller.buttons.a;
}
```

## 性能监控

```rust
use game_engine::platform::console::ConsolePerformanceMonitor;

let mut monitor = ConsolePerformanceMonitor::new();
monitor.update_frame_time(16.67); // 60 FPS
monitor.update_gpu_usage(0.75);
monitor.update_cpu_usage(0.60);

if monitor.check_performance_issues(60) {
    // 性能问题，需要调整设置
}
```

## 平台特定优化

### PlayStation 5

- 支持光线追踪
- 支持HDR
- 最大分辨率：4K (3840x2160)
- 目标帧率：60 FPS

### Xbox Series X/S

- 支持光线追踪
- 支持HDR
- 最大分辨率：4K (3840x2160)
- 目标帧率：60 FPS

### Nintendo Switch

- 不支持光线追踪
- 不支持HDR
- 最大分辨率：1080p (1920x1080)
- 目标帧率：30 FPS（手持模式）或 60 FPS（底座模式）

## 注意事项

1. 控制台平台通常强制启用VSync
2. 帧率通常锁定在30或60 FPS
3. 需要平台特定的SDK进行实际部署
4. 不同平台的内存和性能限制不同

