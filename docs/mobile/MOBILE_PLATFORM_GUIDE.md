# Mobile Platform Integration Guide

本指南介绍如何在游戏引擎中集成移动平台功能（iOS和Android）。

## 目录

1. [快速开始](#快速开始)
2. [iOS集成](#ios集成)
3. [Android集成](#android集成)
4. [移动服务](#移动服务)
5. [性能优化](#性能优化)
6. [生命周期管理](#生命周期管理)
7. [最佳实践](#最佳实践)

## 快速开始

### 添加依赖

在 `Cargo.toml` 中添加移动平台支持：

```toml
[dependencies]
game_engine = { path = ".", features = ["mobile"] }

# iOS特定依赖
[target.'cfg(target_os = "ios")'.dependencies]
objc = "0.2"
cocoa = "0.24"

# Android特定依赖
[target.'cfg(target_os = "android")'.dependencies]
ndk-glue = "0.7"
```

### 基本设置

```rust
use game_engine::platform::mobile;

fn main() {
    // 初始化移动生命周期
    let mut lifecycle = mobile::MobileLifecycle::new();
    lifecycle.initialize();

    // 设置生命周期回调
    lifecycle.add_callback(Box::new(MyLifecycleCallback));

    // 初始化移动服务
    let mut analytics = mobile::Analytics::new();
    analytics.initialize().unwrap();

    // ...
}
```

## iOS集成

### Xcode项目配置

1. 在Xcode项目中添加框架引用：
   - `StoreKit.framework` - 应用内购买
   - `GameKit.framework` - Game Center
   - `UserNotifications.framework` - 推送通知
   - `AdMob.framework` - 广告（通过CocoaPods）

2. Info.plist 配置：

```xml
<key>NSUserNotificationUsageDescription</key>
<string>需要通知权限以发送游戏更新</string>

<key>ITSAppUsesNonExemptEncryption</key>
<false/>
```

### iOS特定功能

#### 应用内购买

```rust
use game_engine::platform::mobile::{InAppPurchaseService, ProductType};

#[cfg(target_os = "ios")]
fn setup_in_app_purchases() {
    let mut iap = InAppPurchaseService::new();
    iap.initialize().unwrap();

    // 查询商品
    let products = iap.query_products(vec![
        "com.game.coin_pack_small".to_string(),
        "com.game.coin_pack_large".to_string(),
    ]).unwrap();

    // 购买商品
    if let Some(product) = products.first() {
        match iap.purchase(product.product_id.clone()) {
            Ok(token) => {
                println!("Purchase successful: {}", token);
            }
            Err(e) => {
                eprintln!("Purchase failed: {:?}", e);
            }
        }
    }
}
```

#### Game Center集成

```rust
use game_engine::platform::mobile::GameCenter;

fn setup_game_center() {
    let mut gc = GameCenter::new();
    gc.initialize().unwrap();

    // 认证玩家
    gc.authenticate().unwrap();

    // 报告成就
    gc.report_achievement("first_win".to_string()).unwrap();

    // 提交分数
    gc.submit_score("leaderboard_high_score".to_string(), 1000).unwrap();

    // 显示Game Center
    gc.show_game_center().unwrap();
}
```

#### 推送通知

```rust
use game_engine::platform::mobile::{PushNotificationService, NotificationPlatform};

fn setup_push_notifications() {
    let mut push = PushNotificationService::new(NotificationPlatform::APNs);
    push.initialize().unwrap();

    // 请求权限
    let granted = push.request_permission().unwrap();
    if granted {
        println!("Push notification permission granted");
    }
}
```

## Android集成

### Gradle配置

在 `android/app/build.gradle` 中添加：

```gradle
dependencies {
    implementation 'com.google.android.gms:play-services-games:22.0.1'
    implementation 'com.google.firebase:firebase-analytics:21.2.0'
    implementation 'com.google.firebase:firebase-messaging:23.1.0'
    implementation 'com.android.billingclient:billing:5.1.0'
}
```

### AndroidManifest.xml配置

```xml
<uses-permission android:name="android.permission.INTERNET"/>
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE"/>

<!-- 可选权限 -->
<uses-permission android:name="android.permission.VIBRATE"/>
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE"/>
```

### Android特定功能

#### Google Play Games

```rust
use game_engine::platform::mobile::GooglePlayGames;

fn setup_google_play_games() {
    let mut gpg = GooglePlayGames::new();
    gpg.initialize().unwrap();

    // 登录
    gpg.sign_in().unwrap();

    // 解锁成就
    gpg.unlock_achievement("achievement_first_blood".to_string()).unwrap();

    // 提交分数
    gpg.submit_score("leaderboard_top_scores".to_string(), 500).unwrap();

    // 显示排行榜
    gpg.show_leaderboard("leaderboard_top_scores".to_string()).unwrap();
}
```

#### Google Play Billing

```rust
use game_engine::platform::mobile::InAppPurchaseService;

#[cfg(target_os = "android")]
fn setup_billing() {
    let mut billing = InAppPurchaseService::new();
    billing.initialize().unwrap();

    // 查询商品
    let products = billing.query_products(vec![
        "com.game.premium".to_string(),
    ]).unwrap();

    // 购买
    if let Some(product) = products.first() {
        let token = billing.purchase(product.product_id.clone()).unwrap();

        // 消耗购买
        billing.consume(token).unwrap();
    }
}
```

## 移动服务

### 分析统计

```rust
use game_engine::platform::mobile::{Analytics, AnalyticsEvent, AnalyticsValue};
use std::collections::HashMap;

fn track_events() {
    let mut analytics = Analytics::new();
    analytics.initialize().unwrap();

    // 设置用户属性
    analytics.set_user_property("level".to_string(), "5".to_string());

    // 记录事件
    let mut params = HashMap::new();
    params.insert("level".to_string(), AnalyticsValue::Integer(5));
    params.insert("score".to_string(), AnalyticsValue::Integer(1000));

    let event = AnalyticsEvent {
        name: "level_complete".to_string(),
        parameters: params,
        timestamp: 0,
    };

    analytics.log_event(event).unwrap();
}
```

### 崩溃报告

```rust
use game_engine::platform::mobile::CrashReporting;

fn setup_crash_reporting() {
    let mut crash_reporting = CrashReporting::new();
    crash_reporting.initialize().unwrap();

    // 设置用户标识符
    crash_reporting.set_user_identifier("player_123".to_string());

    // 设置自定义键
    crash_reporting.set_custom_key("player_level".to_string(), "10".to_string());

    // 记录非致命错误
    crash_reporting.record_error("Custom error message".to_string());
}
```

### 广告

```rust
use game_engine::platform::mobile::{MobileAds, AdsConfig, AdType};

fn setup_ads() {
    let config = AdsConfig {
        test_mode: true,
        admob_app_id_android: Some("ca-app-pub-xxx~yyy".to_string()),
        admob_app_id_ios: Some("ca-app-pub-xxx~yyy".to_string()),
        personalized_ads: false,
        child_directed_treatment: false,
    };

    let mut ads = MobileAds::new(config);
    ads.initialize().unwrap();

    // 加载插屏广告
    ads.load_ad("ca-app-pub-xxx/yyy".to_string(), AdType::Interstitial)
        .unwrap();

    // 展示广告
    if ads.is_ad_loaded("ca-app-pub-xxx/yyy") {
        ads.show_ad("ca-app-pub-xxx/yyy").unwrap();
    }
}
```

### 社交分享

```rust
use game_engine::platform::mobile::{SocialSharing, ShareContent, SocialPlatform};

fn share_content() {
    let sharing = SocialSharing::new();
    sharing.initialize();

    let content = ShareContent {
        text: Some("Check out my high score!".to_string()),
        url: Some("https://game.example.com".to_string()),
        image_path: Some("/path/to/screenshot.png".to_string()),
        file_path: None,
    };

    match sharing.share(content, SocialPlatform::System) {
        Ok(ShareResult::Completed) => println!("Share completed"),
        Ok(ShareResult::Cancelled) => println!("Share cancelled"),
        Err(e) => eprintln!("Share failed: {:?}", e),
    }
}
```

## 性能优化

### 自适应性能

```rust
use game_engine::platform::mobile::{
    MobilePerformanceOptimizer, PerformanceConfig, PerformanceMode
};

fn setup_performance() {
    let config = PerformanceConfig {
        target_fps: 60,
        enable_adaptive_quality: true,
        thermal_throttling_enabled: true,
        battery_saving_mode: false,
        ..Default::default()
    };

    let mut optimizer = MobilePerformanceOptimizer::new(config);
    optimizer.initialize();

    // 检查性能模式
    let mode = optimizer.get_current_mode();
    match mode {
        PerformanceMode::BatterySaving => {
            // 降低质量以延长电池寿命
        }
        PerformanceMode::Balanced => {
            // 平衡模式
        }
        PerformanceMode::Performance => {
            // 最高性能
        }
    }
}
```

### 内存优化

```rust
use game_engine::platform::mobile::MobileLifecycle;

fn handle_memory_warnings(lifecycle: &mut MobileLifecycle) {
    lifecycle.handle_memory_warning();
    // 自动清理资源
}
```

## 生命周期管理

### 实现生命周期回调

```rust
use game_engine::platform::mobile::{LifecycleCallback, MobileLifecycle};

struct MyLifecycleCallback;

impl LifecycleCallback for MyLifecycleCallback {
    fn on_launch(&mut self) {
        println!("App launched!");
    }

    fn on_foreground(&mut self) {
        println!("App entered foreground - resume game");
    }

    fn on_background(&mut self) {
        println!("App entered background - pause game");
        // 保存游戏状态
    }

    fn on_memory_warning(&mut self) {
        println!("Memory warning - clearing cache");
        // 清理缓存
    }

    fn on_low_battery(&mut self, percentage: f32) {
        println!("Low battery: {}% - enabling battery saver", percentage);
        // 启用省电模式
    }

    fn on_network_change(&mut self, available: bool) {
        if available {
            println!("Network available - resuming online features");
        } else {
            println!("Network unavailable - pausing online features");
        }
    }
}

fn setup_lifecycle() {
    let mut lifecycle = MobileLifecycle::new();
    lifecycle.add_callback(Box::new(MyLifecycleCallback));
    lifecycle.initialize();
}
```

### 后台任务

```rust
use game_engine::platform::mobile::{MobileLifecycle, BackgroundTask, TaskStatus};
use std::time::Duration;

fn register_background_task(lifecycle: &mut MobileLifecycle) {
    let task = BackgroundTask {
        id: "save_game".to_string(),
        name: "Auto Save".to_string(),
        timeout: Duration::from_secs(30),
        status: TaskStatus::Pending,
    };

    lifecycle.register_background_task(task);

    // 开始任务
    lifecycle.start_background_task("save_game").unwrap();

    // 完成任务
    lifecycle.complete_background_task("save_game").unwrap();
}
```

## 最佳实践

### 1. 资源管理

- 在 `on_background` 回调中暂停游戏并保存状态
- 在 `on_memory_warning` 回调中清理缓存和未使用的资源
- 使用自适应质量调整图形设置

### 2. 网络处理

- 监听网络状态变化
- 在网络不可用时优雅降级
- 实现离线模式

### 3. 电池优化

- 在低电量时降低性能
- 避免频繁的GPS和网络请求
- 使用合适的帧率限制

### 4. 用户体验

- 实现加载屏幕
- 提供暂停菜单
- 支持多种屏幕方向
- 适配不同屏幕尺寸

### 5. 平台认证

iOS和Android应用必须满足以下要求：

- 正确处理应用生命周期事件
- 实现崩溃报告
- 提供适当的错误处理
- 遵守平台的UI指南

## 测试

### 模拟器测试

```bash
# iOS模拟器
xcrun simctl boot "iPhone 14"

# Android模拟器
emulator -avd pixel_4
```

### 真机测试

确保在以下设备上测试：
- iPhone (iOS 13+)
- iPad (iOS 13+)
- Android手机（API 21+）
- Android平板（API 21+）

### 性能测试

使用Xcode Instruments和Android Profiler监控：
- CPU使用率
- 内存使用
- GPU使用
- 电池消耗
- 网络流量

## 故障排除

### iOS常见问题

1. **签名错误**：确保配置文件和证书正确
2. **框架链接错误**：检查 `-framework` 链接标志
3. **权限错误**：在Info.plist中添加权限描述

### Android常见问题

1. **JNI崩溃**：确保JNI类型签名正确
2. **权限拒绝**：在AndroidManifest.xml中声明权限
3. **ProGuard问题**：添加必要的Keep规则

## 更多资源

- [iOS开发文档](https://developer.apple.com/documentation/)
- [Android开发文档](https://developer.android.com/docs)
- [Google Play Games](https://developers.google.com/games/)
- [AdMob文档](https://developers.google.com/admob/)
