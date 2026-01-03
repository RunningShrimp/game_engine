# Console Platform Integration Guide

本指南介绍如何在游戏引擎中集成游戏机平台功能（Nintendo Switch、PlayStation、Xbox）。

## 目录

1. [平台概览](#平台概览)
2. [Switch开发](#switch开发)
3. [PlayStation开发](#playstation开发)
4. [Xbox开发](#xbox开发)
5. [通用功能](#通用功能)
6. [平台认证](#平台认证)
7. [性能优化](#性能优化)
8. [最佳实践](#最佳实践)

## 平台概览

### 支持的平台

- **Nintendo Switch** - 任天堂Switch主机
- **PlayStation 5** - 索尼PS5主机
- **PlayStation 4** - 索尼PS4主机
- **Xbox Series X/S** - 微软Xbox Series主机
- **Xbox One** - 微软Xbox One主机

### 硬件规格对比

| 平台 | 内存 | GPU | 最大分辨率 | HDR |
|------|------|-----|-----------|-----|
| PS5 | 16GB GDDR6 | 10.28 TFLOPs | 4K | ✓ |
| PS4 | 8GB GDDR5 | 1.84 TFLOPs | 1080p | ✓ |
| Xbox Series X | 16GB GDDR6 | 12 TFLOPs | 4K | ✓ |
| Xbox Series S | 10GB GDDR6 | 4 TFLOPs | 1440p | ✓ |
| Xbox One | 8GB DDR3 | 1.31 TFLOPs | 1080p | ✓ |
| Switch | 4GB LPDDR4 | 0.4 TFLOPs (dock) | 1080p | ✗ |

## Switch开发

### 开发环境设置

1. 获取Nintendo Developer账号
2. 下载Nintendo SDK
3. 配置开发环境

### Switch特定功能

```rust
use game_engine::platform::console::switch;

fn setup_switch_platform() {
    let mut platform = switch::SwitchPlatform::new();
    platform.initialize().unwrap();

    // 获取内存信息
    let memory = switch::SwitchMemory::new();
    println!("Available memory: {} bytes", memory.get_available_memory());

    // 配置Joy-Con控制器
    let controller = switch::SwitchController::new();
    if controller.is_wireless() {
        println!("Using wireless Joy-Con");
    }
}
```

### Switch注意事项

- **内存限制**：Switch有4GB统一内存，需要谨慎管理
- **屏幕分辨率**：
  - 掌机模式：720p (1280x720)
  - 底座模式：最高1080p (1920x1080)
- **性能模式**：支持两种性能配置
- **存档限制**：存档大小限制为100MB

## PlayStation开发

### 开发环境设置

1. 注册为PlayStation合作伙伴
2. 访问 PlayStation Developer Portal
3. 下载PS5/PS4 SDK
4. 设置开发工具

### PS5特定功能

```rust
#[cfg(feature = "psn")]
use game_engine::platform::console::ps5;

fn setup_ps5_platform() {
    let mut platform = ps5::PS5Platform::new();
    platform.initialize().unwrap();

    // 获取硬件能力
    let capabilities = platform.get_hardware_capabilities();
    println!("GPU: {} TFLOPs", capabilities.gpu_teraflips);
    println!("Ray Tracing: {}", capabilities.supports_ray_tracing);

    // DualSense手柄支持
    let controller = ps5::DualSenseController::new();
    controller.set_haptic_feedback(0.5, 0.7);

    // PSN集成
    let mut psn = ps5::PSNIntegration::new();
    psn.initialize().unwrap();

    if let Some(player_id) = psn.get_player_id() {
        println!("Player ID: {}", player_id);
    }
}
```

### PlayStation奖杯系统

```rust
use game_engine::platform::console::{Achievement, AchievementSystem, TrophyType};

fn setup_playstation_trophies() {
    let mut trophy_system = AchievementSystem::new(game_engine::platform::console::ConsolePlatform::PlayStation5);

    // 注册奖杯
    let trophy = Achievement {
        id: "bronze_winner".to_string(),
        name: "First Win".to_string(),
        description: "Win your first match".to_string(),
        hidden: false,
        progress: 0.0,
        required_progress: 1.0,
        status: game_engine::platform::console::AchievementStatus::Locked,
        unlocked_at: None,
        gamerscore: 0, // PlayStation uses trophies, not gamerscore
        trophy_type: Some(TrophyType::Bronze),
    };

    trophy_system.register_achievement(trophy);

    // 解锁奖杯
    trophy_system.unlock_achievement("bronze_winner").unwrap();
}
```

### PlayStation注意事项

- **奖杯系统**：必须集成到游戏中
- **PSN要求**：在线功能需要PSN账号
- **内容指南**：遵守PlayStation内容指南
- **技术要求清单**：必须通过TRC（技术要求清单）

## Xbox开发

### 开发环境设置

1. 注册为Xbox开发者
2. 访问 Xbox Developer Portal
3. 下载Xbox Development Kit (XDK)
4. 设置Visual Studio

### Xbox特定功能

```rust
#[cfg(feature = "xbox")]
use game_engine::platform::console::xbox;

fn setup_xbox_platform() {
    let mut platform = xbox::XboxPlatform::new();
    platform.initialize().unwrap();

    // 检测主机类型
    let console_type = platform.get_console_type();
    match console_type {
        xbox::XboxConsoleType::SeriesX => {
            println!("Xbox Series X detected");
        }
        xbox::XboxConsoleType::SeriesS => {
            println!("Xbox Series S detected");
        }
    }

    // Xbox Live集成
    let mut xbox_live = xbox::XboxLiveIntegration::new();
    xbox_live.initialize().unwrap();

    if let Some(gamertag) = xbox_live.get_gamertag() {
        println!("Gamertag: {}", gamertag);
    }

    let gamerscore = xbox_live.get_gamerscore();
    println!("Gamerscore: {}", gamerscore);
}
```

### Xbox成就系统

```rust
use game_engine::platform::console::xbox;

fn setup_xbox_achievements() {
    let mut achievements = xbox::Achievements::new();

    // 解锁成就
    achievements.unlock_achievement("achievement_id").unwrap();
}
```

### Xbox注意事项

- **Gamerscore**：每个成就都有对应的Gamerscore值
- **Xbox Live**：在线功能需要Xbox Live账号
- **SmartGlass**：支持第二屏功能
- **认证要求**：必须通过Xbox认证流程

## 通用功能

### 成就系统

```rust
use game_engine::platform::console::{AchievementSystem, Achievement};

fn setup_cross_platform_achievements(platform: ConsolePlatform) {
    let mut achievement_system = AchievementSystem::new(platform);

    // 注册成就
    let achievement = Achievement {
        id: "first_blood".to_string(),
        name: "First Blood".to_string(),
        description: "Get your first elimination".to_string(),
        hidden: false,
        progress: 0.0,
        required_progress: 1.0,
        status: game_engine::platform::console::AchievementStatus::Locked,
        unlocked_at: None,
        gamerscore: 10, // Xbox
        trophy_type: Some(game_engine::platform::console::TrophyType::Bronze), // PlayStation
    };

    achievement_system.register_achievement(achievement);

    // 更新进度
    achievement_system.update_progress("first_blood", 0.5).unwrap();

    // 完整进度时自动解锁
    achievement_system.update_progress("first_blood", 1.0).unwrap();

    // 检查统计
    let stats = achievement_system.get_stats();
    println!("Unlocked: {}/{}", stats.unlocked_count, stats.total_count);
    println!("Completion: {:.1}%", stats.completion_percentage);
}
```

### 云存档

```rust
use game_engine::platform::console::{CloudSaveManager, SaveMetadata};

fn setup_cloud_saves(platform: ConsolePlatform) {
    let save_path = std::path::PathBuf::from("/path/to/saves");
    let mut save_manager = CloudSaveManager::new(platform, save_path);

    save_manager.initialize().unwrap();

    // 保存游戏
    let metadata = SaveMetadata {
        game_version: "1.0.0".to_string(),
        player_level: 10,
        current_chapter: "Chapter 2".to_string(),
        completion_percentage: 25.0,
        custom_data: {
            let mut map = std::collections::HashMap::new();
            map.insert("playtime".to_string(), "3600".to_string());
            map
        },
    };

    let save_data = b"player_state_data_here";
    save_manager.save_game(1, save_data, metadata).unwrap();

    // 加载游戏
    let loaded_data = save_manager.load_game(1).unwrap();
    println!("Loaded {} bytes", loaded_data.len());

    // 同步所有存档到云端
    save_manager.sync_all_to_cloud().unwrap();
}
```

### 手柄输入

```rust
use game_engine::platform::console::{ConsoleInputHandler, ControllerState, ButtonState};

fn handle_controller_input() {
    let mut input_handler = ConsoleInputHandler::new();

    // 更新控制器状态
    let state = ControllerState {
        id: 0,
        connected: true,
        left_stick: (0.5, 0.3),
        right_stick: (0.0, 0.0),
        left_trigger: 0.0,
        right_trigger: 0.0,
        buttons: ButtonState {
            a: true,
            b: false,
            x: false,
            y: false,
            left_bumper: false,
            right_bumper: false,
            left_stick_click: false,
            right_stick_click: false,
            dpad_up: false,
            dpad_down: false,
            dpad_left: false,
            dpad_right: false,
            menu: false,
            view: false,
        },
    };

    input_handler.update_controller(0, state);

    // 读取控制器输入
    if let Some(controller) = input_handler.get_controller(0) {
        if controller.buttons.a {
            println!("A button pressed");
        }

        if controller.left_stick.0 > 0.5 {
            println!("Moving right");
        }
    }
}
```

### 手柄扩展功能

```rust
use game_engine::platform::console::controller_extended::{
    ExtendedControllerManager, VibrationIntensity, LedColor
};

fn setup_extended_controller(platform: ConsolePlatform) {
    let manager = ExtendedControllerManager::new(platform);

    // 设置震动
    let vibration = VibrationIntensity::new(0.7, 0.5);
    manager.set_vibration(0, vibration).unwrap();

    // 设置LED颜色（PS4/PS5/Switch Pro）
    let color = LedColor::red();
    manager.set_led_color(0, color).unwrap();

    // 获取触摸板输入（PS4/PS5）
    if let Ok(touch_points) = manager.get_touch_input(0) {
        for point in &touch_points {
            if point.touching {
                println!("Touch at ({}, {})", point.x, point.y);
            }
        }
    }

    // 获取运动数据（PS4/PS5/Switch Joy-Con）
    if let Ok(motion) = manager.get_motion_data(0) {
        println!("Gyro: ({}, {}, {})", motion.gyro.0, motion.gyro.1, motion.gyro.2);
        println!("Accel: ({}, {}, {})", motion.accel.0, motion.accel.1, motion.accel.2);
    }
}
```

### 性能监控

```rust
use game_engine::platform::console::ConsolePerformanceMonitor;

fn monitor_performance() {
    let mut monitor = ConsolePerformanceMonitor::new();

    loop {
        // 更新帧时间
        let frame_time = 16.7; // ms (approximately 60 FPS)
        monitor.update_frame_time(frame_time);

        // 更新资源使用率
        monitor.update_gpu_usage(0.85);
        monitor.update_cpu_usage(0.65);

        // 检查性能
        let fps = monitor.current_fps();
        println!("FPS: {:.1}", fps);

        if monitor.check_performance_issues(60) {
            println!("Performance issues detected!");
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
```

## 平台认证

所有游戏机平台都有严格的认证要求。以下是认证检查清单：

### 通用要求

- [ ] 成就/奖杯系统已集成
- [ ] 云存档已集成
- [ ] 手柄震动正常工作
- [ ] 错误处理完善
- [ ] 加载屏幕显示进度
- [ ] 暂停菜单可访问
- [ ] 网络断开处理
- [ ] 存档损坏处理

### PlayStation特定要求

- [ ] 奖杯图标正确设置
- [ ] PSN在线功能正常
- [ ] DualSense触觉反馈
- [ ] 遵守PS UI指南

### Xbox特定要求

- [ ] 成就图标正确设置
- [ ] Xbox Live集成正常
- [ ] Gamerscore正确分配
- [ ] 遵守Xbox UI指南

### Nintendo Switch特定要求

- [ ] 存档管理符合要求
- [ ] Joy-Con正确配对
- [ ] 掌机/底座模式切换
- [ ] 截图功能正常

### 认证检查工具

```rust
use game_engine::platform::console::certification::CertificationChecker;

fn run_certification_check(platform: ConsolePlatform) {
    let checker = CertificationChecker::new(platform);
    let report = checker.check_all_requirements();

    if report.all_passed() {
        println!("All certification requirements passed!");
    } else {
        println!("Certification issues found:");
        println!("{}", report.generate_report());
    }
}
```

## 性能优化

### 帧率稳定

游戏机平台通常要求稳定的帧率：

```rust
use game_engine::platform::console::ConsoleConfig;

fn setup_performance_profile(platform: ConsolePlatform) {
    let mut config = ConsoleConfig::from_hardware(&hardware_info);

    // 启用性能模式
    config.performance_mode = true;
    config.quality_mode = false;

    // 应用到图形配置
    let mut graphics_config = GraphicsConfig::default();
    config.apply_to_graphics_config(&mut graphics_config);

    println!("Target FPS: {}", config.target_fps);
}
```

### 内存管理

游戏机平台内存有限，需要精心管理：

- 使用对象池减少分配
- 及时释放未使用的资源
- 监控内存使用
- 使用平台特定的内存分析工具

### 帧率优化技巧

1. **使用性能模式**：降低分辨率换取帧率
2. **动态分辨率**：根据负载调整分辨率
3. **Level of Detail (LOD)**：根据距离调整细节
4. **遮挡剔除**：不渲染看不见的对象
5. **批处理**：减少绘制调用

## 最佳实践

### 1. 开发流程

1. **早期原型**：先在PC上开发原型
2. **平台迁移**：尽早迁移到目标平台
3. **持续测试**：在真机上持续测试
4. **性能分析**：使用平台提供的分析工具

### 2. 平台差异处理

```rust
use game_engine::platform::console::ConsolePlatform;

fn handle_platform_differences(platform: ConsolePlatform) {
    match platform {
        ConsolePlatform::NintendoSwitch => {
            // Switch特定优化
            // 降低分辨率，减少特效
        }
        ConsolePlatform::PlayStation5 => {
            // PS5特定功能
            // 启用光线追踪，触觉反馈
        }
        ConsolePlatform::XboxSeries => {
            // Xbox特定功能
            // Quick Resume，SmartGlass
        }
        _ => {}
    }
}
```

### 3. 输入处理

- 支持所有标准按钮
- 处理手柄断开/重连
- 提供按键绑定选项
- 支持多个手柄同时使用

### 4. 存档管理

- 定期自动保存
- 提供多个存档槽位
- 云存档同步
- 处理存档损坏

### 5. 网络功能

- 优雅处理网络断开
- 提供离线模式
- 显示网络状态
- 支持跨平台联机（如果允许）

## 开发工具

### PlayStation工具

- **SN Systems**：性能分析和调试
- **Razör**：GPU分析工具
- **PS5 DevKit**：开发硬件

### Xbox工具

- **PIX**：性能分析工具
- **Xbox Assessment**：认证检查工具
- **Xbox Developer Portal**：在线管理

### Nintendo工具

- **Nintendo Developer Support**：官方支持
- **Switch DevKit**：开发硬件
- **Ware-Tools**：开发工具集

## 故障排除

### 常见问题

1. **内存不足**
   - 使用对象池
   - 减少资源大小
   - 优化纹理格式

2. **帧率不稳定**
   - 简化场景
   - 减少绘制调用
   - 优化物理模拟

3. **认证失败**
   - 检查TRC/TCR要求
   - 运行认证检查工具
   - 咨询平台支持

4. **手柄问题**
   - 正确处理连接/断开
   - 测试所有按钮
   - 验证震动功能

## 更多资源

- [PlayStation Partners](https://partners.playstation.com/)
- [Xbox Developers](https://developer.xbox.com/)
- [Nintendo Developers](https://developer.nintendo.com/)
- [Wwise音频中间件](https://www.audiokinetic.com/)
- [FMOD音频中间件](https://www.fmod.com/)

## 技术支持

每个平台都提供开发者技术支持：

- PlayStation: devrel@playstation.sony.com
- Xbox: xdd@microsoft.com
- Nintendo: wware-support@noa.nintendo.com
