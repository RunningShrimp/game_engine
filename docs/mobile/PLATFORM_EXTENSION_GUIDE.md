# Platform Extension Documentation

本指南介绍游戏引擎的平台扩展功能，包括移动平台、游戏机平台和跨平台API。

## 目录

1. [支持的平台](#支持的平台)
2. [移动平台](#移动平台)
3. [游戏机平台](#游戏机平台)
4. [跨平台API](#跨平台api)
5. [平台特性检测](#平台特性检测)
6. [使用示例](#使用示例)
7. [最佳实践](#最佳实践)

## 支持的平台

### 移动平台
- **iOS** - iPhone/iPad (iOS 12+)
- **Android** - Android 5.0+ (API Level 21+)
- **HarmonyOS** - 华为鸿蒙系统 (feature flag: `harmonyos`)

### 游戏机平台
- **Nintendo Switch**
- **PlayStation 5**
- **PlayStation 4**
- **Xbox Series X/S**
- **Xbox One**

### 桌面平台
- **Windows** - Windows 10+
- **macOS** - macOS 10.14+
- **Linux** - Ubuntu 18.04+, CentOS 7+

### Web平台
- **WebAssembly** - 现代浏览器 (Chrome, Firefox, Safari, Edge)

## 移动平台

### iOS平台特性

#### Game Center集成
```rust
use game_engine::platform::mobile::services::GameCenter;

let mut game_center = GameCenter::new();
game_center.initialize()?;

// 认证玩家
game_center.authenticate()?;

// 报告成就
game_center.report_achievement("first_win".to_string())?;

// 提交分数
game_center.submit_score("leaderboard_id".to_string(), 1000)?;

// 显示Game Center
game_center.show_game_center()?;
```

#### StoreKit应用内购买
```rust
use game_engine::platform::mobile::services::InAppPurchaseService;

let mut iap = InAppPurchaseService::new();
iap.initialize()?;

// 查询商品
let products = iap.query_products(vec![
    "com.game.coins100".to_string(),
    "com.game.premium".to_string(),
]).await?;

// 购买商品
let purchase_token = iap.purchase("com.game.coins100".to_string()).await?;

// 恢复购买
let restored = iap.restore_purchases().await?;
```

### Android平台特性

#### Google Play Games集成
```rust
use game_engine::platform::mobile::services::GooglePlayGames;

let mut gpg = GooglePlayGames::new();
gpg.initialize()?;

// 登录
gpg.sign_in()?;

// 解锁成就
gpg.unlock_achievement("achievement_id".to_string())?;

// 更新成就进度
gpg.update_achievement_progress("achievement_id".to_string(), 75)?;

// 提交分数
gpg.submit_score("leaderboard_id".to_string(), 1000)?;

// 显示排行榜
gpg.show_leaderboard("leaderboard_id".to_string())?;
```

#### Google Play Billing
```rust
use game_engine::platform::mobile::services::InAppPurchaseService;

let mut billing = InAppPurchaseService::new();
billing.initialize()?;

// 查询商品
let products = billing.query_products(vec![
    "com.game.coins100".to_string(),
]).await?;

// 购买商品
let token = billing.purchase("com.game.coins100".to_string()).await?;

// 消耗商品（消耗型）
billing.consume(token)?;
```

#### Firebase推送通知
```rust
use game_engine::platform::mobile::services::{PushNotificationService, NotificationPlatform, Notification};

let mut push = PushNotificationService::new(NotificationPlatform::Firebase);
push.initialize()?;

// 请求权限
let granted = push.request_permission()?;

// 发送本地通知
let notification = Notification::new(
    "Hello".to_string(),
    "Welcome to the game!".to_string()
);
push.send_local_notification(notification)?;

// 订阅主题
push.subscribe_to_topic("game_updates".to_string())?;
```

## 游戏机平台

### 成就系统
```rust
use game_engine::platform::console::{AchievementSystem, Achievement, AchievementStatus, ConsolePlatform};

let mut achievement_system = AchievementSystem::new(ConsolePlatform::PlayStation5);

// 注册成就
let achievement = Achievement {
    id: "first_blood".to_string(),
    name: "First Blood".to_string(),
    description: "Get your first elimination".to_string(),
    hidden: false,
    progress: 0.0,
    required_progress: 1.0,
    status: AchievementStatus::Locked,
    unlocked_at: None,
    gamerscore: 10,
    trophy_type: Some(TrophyType::Bronze),
};
achievement_system.register_achievement(achievement);

// 更新进度
achievement_system.update_progress("first_blood", 0.5)?;

// 完成时自动解锁
achievement_system.update_progress("first_blood", 1.0)?;

// 获取统计
let stats = achievement_system.get_stats();
println!("Completion: {:.1}%", stats.completion_percentage);
```

### 云存档系统
```rust
use game_engine::platform::console::{CloudSaveManager, SaveMetadata};
use std::path::PathBuf;

let save_dir = PathBuf::from("/path/to/saves");
let mut save_manager = CloudSaveManager::new(ConsolePlatform::PlayStation5, save_dir);
save_manager.initialize()?;

// 保存游戏
let metadata = SaveMetadata {
    game_version: "1.0.0".to_string(),
    player_level: 10,
    current_chapter: "Chapter 2".to_string(),
    completion_percentage: 25.0,
    playtime_seconds: 3600,
    custom_data: {
        let mut map = std::collections::HashMap::new();
        map.insert("difficulty".to_string(), "hard".to_string());
        map
    },
};

let save_data = b"player_state_data";
save_manager.save_game(1, save_data, metadata)?;

// 加载游戏
let loaded_data = save_manager.load_game(1)?;

// 同步到云端
save_manager.sync_all_to_cloud()?;
```

### 手柄扩展功能
```rust
use game_engine::platform::console::controller_extended::{
    ExtendedControllerManager, VibrationIntensity, LedColor
};

let manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);

// 设置震动
let vibration = VibrationIntensity::new(0.7, 0.5);
manager.set_vibration(0, vibration)?;

// 设置LED颜色（PS4/PS5/Switch Pro）
let color = LedColor::red();
manager.set_led_color(0, color)?;

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
    println!("Gyro: {:?}", motion.gyro);
    println!("Accel: {:?}", motion.accel);
}

// DualSense触觉反馈（PS5）
use game_engine::platform::console::controller_extended::HapticFeedback;
let haptic = HapticFeedback::new(0.5, 0.7);
manager.set_haptic_feedback(0, haptic)?;

// DualSense自适应触发器（PS5）
manager.set_trigger_resistance(0, 0.5, 0.8)?;
```

### 性能监控
```rust
use game_engine::platform::console::ConsolePerformanceMonitor;

let mut monitor = ConsolePerformanceMonitor::new();

// 更新帧时间
monitor.update_frame_time(16.7); // ms (approximately 60 FPS)

// 更新资源使用率
monitor.update_gpu_usage(0.85);
monitor.update_cpu_usage(0.65);
monitor.update_memory_usage(2048); // MB

// 检查性能
let fps = monitor.current_fps();
println!("FPS: {:.1}", fps);

if monitor.check_performance_issues(60) {
    println!("Performance issues detected!");
}
```

### 平台认证
```rust
use game_engine::platform::console::certification::{CertificationChecker, ConsolePlatform};

let checker = CertificationChecker::new(ConsolePlatform::PlayStation5);
let report = checker.check_all_requirements();

if report.all_passed() {
    println!("All certification requirements passed!");
} else {
    println!("Certification issues found:");
    println!("{}", report.generate_report());
}
```

## 跨平台API

### 统一平台服务
```rust
use game_engine::platform::unified::UnifiedPlatformService;

let mut service = UnifiedPlatformService::new();
service.initialize_all().await?;

// 认证
let player = service.authenticate()?;

// 成就
service.unlock_achievement("achievement_id".to_string())?;

// 排行榜
service.submit_score("leaderboard_id".to_string(), 1000)?;

// 显示UI
service.show_achievements()?;
service.show_leaderboard("leaderboard_id".to_string())?;

// 应用内购买
let products = service.query_iap_products(vec!["product_id".to_string()]).await?;
let token = service.purchase_iap("product_id".to_string()).await?;

// 推送通知
let granted = service.request_push_permission()?;
use game_engine::platform::unified::UnifiedNotification;
let notification = UnifiedNotification {
    title: "Hello".to_string(),
    body: "Welcome!".to_string(),
    data: std::collections::HashMap::new(),
};
service.send_local_notification(notification)?;
```

### 平台能力检测
```rust
use game_engine::platform::unified::PlatformCapabilities;

let caps = PlatformCapabilities::current();

if caps.supports_achievements {
    // 初始化成就系统
}

if caps.supports_iap {
    // 初始化应用内购买
}

if caps.supports_touch {
    // 添加触摸控制
}

if caps.supports_ray_tracing {
    // 启用光线追踪
}
```

## 平台特性检测

### 平台类型检测
```rust
use game_engine::platform::detection;

// 检测平台类型
if detection::is_mobile() {
    println!("Running on mobile platform");
}

if detection::is_desktop() {
    println!("Running on desktop platform");
}

if detection::is_console() {
    println!("Running on console platform");
}

if detection::is_web() {
    println!("Running on web platform");
}
```

### 操作系统检测
```rust
use game_engine::platform::detection;

if detection::is_android() {
    println!("Running on Android");
}

if detection::is_ios() {
    println!("Running on iOS");
}

if detection::is_windows() {
    println!("Running on Windows");
}

if detection::is_macos() {
    println!("Running on macOS");
}

if detection::is_linux() {
    println!("Running on Linux");
}
```

### 架构和特性检测
```rust
use game_engine::platform::detection;

// 架构检测
if detection::is_x86_64() {
    println!("Running on x86_64");
}

if detection::is_aarch64() {
    println!("Running on ARM64");
}

// SIMD支持
if detection::supports_simd() {
    println!("SIMD instructions available");
}
```

### 详细平台信息
```rust
use game_engine::platform::detection::PlatformInfo;

let info = PlatformInfo::current();
println!("OS: {}", info.os);
println!("Arch: {}", info.arch);
println!("Mobile: {}", info.is_mobile);
println!("Desktop: {}", info.is_desktop);
println!("Console: {}", info.is_console);
println!("Web: {}", info.is_web);
println!("SIMD: {}", info.supports_simd);
```

## 使用示例

### 完整的移动平台游戏示例
```rust
use game_engine::platform::mobile::services::{InAppPurchaseService, NotificationPlatform, PushNotificationService, GameCenter, GooglePlayGames};
use game_engine::platform::detection;

async fn setup_mobile_platform() -> Result<(), Box<dyn std::error::Error>> {
    let mut iap = InAppPurchaseService::new();
    iap.initialize()?;

    let mut push = PushNotificationService::new(NotificationPlatform::Firebase);
    push.initialize()?;

    if detection::is_ios() {
        let mut gc = GameCenter::new();
        gc.initialize()?;
        gc.authenticate()?;
    } else if detection::is_android() {
        let mut gpg = GooglePlayGames::new();
        gpg.initialize()?;
        gpg.sign_in()?;
    }

    Ok(())
}
```

### 完整的游戏机平台游戏示例
```rust
use game_engine::platform::console::{AchievementSystem, CloudSaveManager, ConsolePlatform, ConsoleConfig};
use std::path::PathBuf;

async fn setup_console_platform() -> Result<(), Box<dyn std::error::Error>> {
    // 配置
    let config = ConsoleConfig::from_hardware(ConsolePlatform::PlayStation5);
    println!("Target FPS: {}", config.target_fps);

    // 成就系统
    let mut achievements = AchievementSystem::new(ConsolePlatform::PlayStation5);
    // 注册成就...

    // 云存档
    let save_dir = PathBuf::from("/game/saves");
    let mut saves = CloudSaveManager::new(ConsolePlatform::PlayStation5, save_dir);
    saves.initialize()?;

    Ok(())
}
```

## 最佳实践

### 1. 平台适配
- 使用统一的平台API，避免平台特定代码
- 为不同平台提供不同的UI布局
- 测试所有目标平台

### 2. 性能优化
- 使用平台性能监控工具
- 针对低性能设备优化
- 使用平台特定的性能特性

### 3. 用户体验
- 提供平台原生的UI体验
- 支持平台特定的输入方式
- 遵守平台设计指南

### 4. 错误处理
- 优雅处理平台功能不可用
- 提供有意义的错误消息
- 实现适当的回退机制

### 5. 测试
- 在真实设备上测试
- 使用平台认证工具
- 进行性能测试

## 更多资源

- [Console Platform Guide](../console/CONSOLE_PLATFORM_GUIDE.md)
- [Platform Detection API](./detection.rs)
- [Unified Platform Services](./unified.rs)
- [Mobile Platform Services](./mobile/)
