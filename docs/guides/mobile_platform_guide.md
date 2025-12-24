# 移动平台支持指南

本指南详细介绍如何在iOS和Android平台上使用游戏引擎。

## 概述

游戏引擎提供了完整的iOS和Android平台支持，包括：

- **iOS支持**: 原生iOS应用开发
- **Android支持**: 原生Android应用开发
- **移动平台优化**: 自适应性能、功耗优化、热节流检测
- **触摸输入**: 多点触控、手势识别
- **传感器支持**: 陀螺仪、加速度计

## iOS平台

### 前置要求

1. **macOS系统**: iOS开发只能在macOS上进行
2. **Xcode**: 安装最新版本的Xcode
3. **Rust工具链**: 安装Rust和rustup
4. **iOS目标**: 添加iOS编译目标

### 安装iOS目标

```bash
# 添加iOS设备目标（ARM64）
rustup target add aarch64-apple-ios

# 添加iOS模拟器目标（ARM64）
rustup target add aarch64-apple-ios-sim

# 添加iOS模拟器目标（x86_64，用于Intel Mac）
rustup target add x86_64-apple-ios
```

### 构建iOS库

使用提供的构建脚本：

```bash
# 调试版本
./scripts/build_ios.sh

# 发布版本
./scripts/build_ios.sh --release

# 指定目标架构
./scripts/build_ios.sh --target aarch64-apple-ios-sim
```

### 在Xcode项目中使用

1. **创建Xcode项目**:
   - 打开Xcode
   - 创建新的iOS项目
   - 选择Swift或Objective-C

2. **添加Rust库**:
   - 将构建的`.a`或`.rlib`文件添加到项目
   - 在Build Settings中设置Library Search Paths

3. **配置桥接头文件**:
   ```swift
   // Bridging-Header.h
   #import "game_engine.h"
   ```

4. **调用Rust代码**:
   ```swift
   import Foundation

   // 初始化引擎
   engine_init()
   
   // 运行游戏循环
   engine_run()
   ```

### iOS特定配置

```rust
use game_engine::platform::mobile::{MobileConfig, get_mobile_config};

// 获取iOS特定配置
if let Some(config) = get_mobile_config() {
    // 配置移动平台优化
    config.apply_to_graphics_config(&mut graphics_config);
}
```

## Android平台

### 前置要求

1. **Android NDK**: 安装Android NDK（推荐r21或更高版本）
2. **Rust工具链**: 安装Rust和rustup
3. **cargo-ndk**: 安装cargo-ndk工具
   ```bash
   cargo install cargo-ndk
   ```
4. **Android目标**: 添加Android编译目标

### 安装Android目标

```bash
# ARM64 (64位)
rustup target add aarch64-linux-android

# ARM (32位)
rustup target add armv7-linux-androideabi

# x86 (32位)
rustup target add i686-linux-android

# x86_64 (64位)
rustup target add x86_64-linux-android
```

### 配置Android NDK

设置环境变量：

```bash
# 方法1: 设置ANDROID_NDK_HOME
export ANDROID_NDK_HOME=/path/to/android-ndk

# 方法2: 设置ANDROID_NDK_ROOT
export ANDROID_NDK_ROOT=/path/to/android-ndk
```

### 构建Android库

使用提供的构建脚本：

```bash
# 调试版本
./scripts/build_android.sh

# 发布版本
./scripts/build_android.sh --release

# 指定目标架构和API级别
./scripts/build_android.sh --target aarch64-linux-android --api-level 30
```

### 在Android项目中使用

1. **创建Android项目**:
   - 使用Android Studio创建新项目
   - 选择Native C++模板

2. **配置CMakeLists.txt**:
   ```cmake
   # 添加Rust库
   add_library(game_engine STATIC IMPORTED)
   set_target_properties(game_engine PROPERTIES
       IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/libs/${ANDROID_ABI}/libgame_engine.a
   )
   
   # 链接库
   target_link_libraries(native-lib game_engine)
   ```

3. **在Java/Kotlin中调用**:
   ```kotlin
   // 加载本地库
   init {
       System.loadLibrary("game_engine")
   }
   
   // 声明本地方法
   external fun engineInit()
   external fun engineRun()
   ```

4. **在C++中实现JNI接口**:
   ```cpp
   #include <jni.h>
   #include "game_engine.h"

   extern "C" JNIEXPORT void JNICALL
   Java_com_example_MainActivity_engineInit(JNIEnv *env, jobject thiz) {
       engine_init();
   }
   ```

## 移动平台优化

### 自适应性能

游戏引擎提供自适应性能管理，根据设备性能自动调整设置：

```rust
use game_engine::platform::mobile::MobileAdaptivePerformance;

let mut adaptive = MobileAdaptivePerformance::new(config);

// 在游戏循环中更新
loop {
    let frame_time = measure_frame_time();
    adaptive.update(frame_time, current_timestamp());
    
    // 应用调整后的设置
    let resolution_scale = adaptive.resolution_scale();
    let target_fps = adaptive.target_fps();
    
    // 使用这些值更新渲染设置
}
```

### 性能监控

监控移动设备性能：

```rust
use game_engine::platform::mobile::MobilePerformanceMonitor;

let mut monitor = MobilePerformanceMonitor::new();

// 更新帧时间
monitor.update_frame_time(frame_time_ms);

// 检查性能问题
let issue = monitor.check_performance_issues(target_fps);
match issue {
    PerformanceIssue::LowFps { current, target } => {
        // 降低质量
    }
    PerformanceIssue::ThermalThrottling => {
        // 启用省电模式
    }
    PerformanceIssue::LowBattery { level } => {
        // 降低帧率
    }
    PerformanceIssue::None => {
        // 性能良好
    }
}
```

### 触摸输入

处理触摸输入：

```rust
use game_engine::platform::mobile::MobileInputHandler;

let mut input_handler = MobileInputHandler::new();

// 处理触摸事件
match event {
    InputEvent::TouchStart { id, x, y } => {
        input_handler.handle_touch_start(id, x, y, 1.0, timestamp);
    }
    InputEvent::TouchMove { id, x, y } => {
        input_handler.handle_touch_move(id, x, y, 1.0, timestamp);
    }
    InputEvent::TouchEnd { id, .. } => {
        input_handler.handle_touch_end(id, timestamp);
    }
    _ => {}
}

// 获取触摸点
for touch in input_handler.get_touches() {
    println!("Touch {} at ({}, {})", touch.id, touch.position.0, touch.position.1);
}
```

### 陀螺仪支持

使用陀螺仪数据：

```rust
use game_engine::platform::mobile::MobileInputHandler;

let mut input_handler = MobileInputHandler::new();

// 更新陀螺仪数据
input_handler.update_gyroscope(
    (rotation_x, rotation_y, rotation_z),
    (accel_x, accel_y, accel_z),
    timestamp,
);

// 获取陀螺仪数据
if let Some(gyro) = input_handler.get_gyroscope() {
    let rotation_rate = gyro.rotation_rate;
    let acceleration = gyro.acceleration;
    
    // 使用陀螺仪数据
}
```

## 移动平台配置

### 移动平台配置选项

```rust
use game_engine::platform::mobile::MobileConfig;

let config = MobileConfig {
    target_fps: 60,                    // 目标帧率
    adaptive_fps: true,                // 启用自适应帧率
    power_saving: true,                // 启用省电模式
    thermal_throttling_detection: true, // 启用热节流检测
    max_resolution_scale: 1.0,         // 最大分辨率缩放
    dynamic_resolution: true,          // 启用动态分辨率
    touch_sensitivity: 1.0,            // 触摸灵敏度
    gyroscope_enabled: false,          // 是否启用陀螺仪
};
```

### 从硬件信息创建配置

```rust
use game_engine::platform::mobile::MobileConfig;
use game_engine_hardware::HardwareInfo;

let hardware = HardwareInfo::detect();
let config = MobileConfig::from_hardware(&hardware);

// 应用配置到图形设置
config.apply_to_graphics_config(&mut graphics_config);
```

## 最佳实践

### 1. 性能优化

- **使用纹理压缩**: 启用平台特定的纹理压缩格式
- **降低MSAA**: 移动平台使用较低的MSAA级别（2x或4x）
- **动态分辨率**: 启用动态分辨率以保持帧率
- **LOD系统**: 使用细节层次系统减少渲染负载

### 2. 功耗优化

- **自适应帧率**: 根据设备性能调整帧率
- **省电模式**: 低电量时降低质量
- **热节流检测**: 检测并响应热节流

### 3. 输入处理

- **触摸优化**: 优化触摸输入响应
- **手势识别**: 实现常见手势（捏合、滑动等）
- **虚拟摇杆**: 为游戏实现虚拟摇杆

### 4. 内存管理

- **内存池**: 使用内存池减少分配
- **资源压缩**: 压缩纹理和音频资源
- **流式加载**: 使用流式加载减少内存占用

## 常见问题

### Q: iOS构建失败怎么办？

**A**: 检查以下几点：
1. 确保在macOS上构建
2. 检查Xcode是否正确安装
3. 验证iOS目标是否正确添加
4. 检查代码签名设置

### Q: Android构建失败怎么办？

**A**: 检查以下几点：
1. 确保Android NDK已安装并配置
2. 检查cargo-ndk是否正确安装
3. 验证Android目标是否正确添加
4. 检查API级别是否兼容

### Q: 如何在移动平台上调试？

**A**: 
- **iOS**: 使用Xcode的调试器
- **Android**: 使用Android Studio的调试器或adb logcat
- **远程调试**: 使用远程调试工具

### Q: 如何处理不同屏幕尺寸？

**A**: 
- 使用动态分辨率缩放
- 实现响应式UI布局
- 测试不同设备尺寸

## 相关文档

- [平台适配器](../architecture.md#平台适配器)
- [性能优化指南](./performance_optimization.md)
- [输入系统](../platform/mod.rs)

