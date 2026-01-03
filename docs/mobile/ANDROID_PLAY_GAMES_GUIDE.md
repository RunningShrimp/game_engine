# Android Google Play Games Integration Guide

本文档介绍如何在游戏中集成Google Play Games服务。

## 功能概述

### 支持的Play Games功能
- **玩家登录**: 通过Google账号登录
- **成就系统**: 解锁和管理游戏成就
- **排行榜**: 提交和查看分数
- **多人游戏**: 实时和回合制多人游戏
- **云存档**: 保存游戏进度到云端
- **游戏事件**: 记录游戏事件分析

## 设置

### 1. 配置Google Play Console

1. 登录 [Google Play Console](https://play.google.com/console)
2. 选择你的应用
3. 进入"Play Games Services" > "Setup and management"
4. 配置成就和排行榜
5. 生成资源配置文件

### 2. 添加依赖

在`AndroidManifest.xml`中添加:
```xml
<meta-data android:name="com.google.android.gms.games.APP_ID"
    android:value="@string/app_id" />
<meta-data android:name="com.google.android.gms.version"
    android:value="@integer/google_play_services_version" />
```

### 3. 代码集成

```rust
use game_engine::platform::mobile::services::GooglePlayGames;

// 初始化Google Play Games
let mut gpg = GooglePlayGames::new();
gpg.initialize()?;

// 登录
gpg.sign_in()?;

if let Some(player) = gpg.get_current_player() {
    println!("Player: {} (ID: {})", player.name, player.id);
}
```

## 成就系统

### 定义成就

在Google Play Console中定义成就:
- 成就ID: `achievement_first_win`
- 成就名称: "First Win"
- 成就类型: 标准、增量或隐藏

### 解锁成就

```rust
// 解锁标准成就
gpg.unlock_achievement("achievement_first_win".to_string())?;

// 更新增量成就进度 (0-100%)
gpg.update_achievement_progress("achievement_kill_100_enemies".to_string(), 45)?;

// 解锁隐藏成就
gpg.unlock_achievement("achievement_secret".to_string())?;
```

### 显示成就界面

```rust
// 显示Play Games成就界面
gpg.show_achievements()?;
```

## 排行榜

### 定义排行榜

在Google Play Console中定义排行榜:
- 排行榜ID: `leaderboard_high_scores`
- 排行榜名称: "High Scores"
- 分数格式: 数值、时间或货币

### 提交分数

```rust
// 提交分数到排行榜
gpg.submit_score("leaderboard_high_scores".to_string(), 1000)?;
```

### 查看排行榜

```rust
// 显示排行榜界面
gpg.show_leaderboard("leaderboard_high_scores".to_string())?;

// 加载排行榜分数
use game_engine::platform::mobile::android_services::{LeaderboardTimeSpan, LeaderboardCollection};

let scores = gpg.load_leaderboard_scores(
    "leaderboard_high_scores".to_string(),
    LeaderboardTimeSpan::AllTime,
    LeaderboardCollection::Public
)?;
```

## 云存档

### 启用云存档

```rust
gpg.enable_saved_games()?;
```

### 保存游戏数据

```rust
let save_data = vec![1u8, 2, 3, 4]; // 游戏存档数据
gpg.save_game_data(save_data, "Main Quest Progress".to_string())?;
```

### 加载游戏数据

```rust
let loaded_data = gpg.load_game_data()?;
println!("Loaded {} bytes", loaded_data.len());
```

## 多人游戏

### 启用多人游戏

```rust
gpg.enable_multiplayer()?;
```

### 创建多人游戏

```rust
// 实时多人游戏
let min_players = 2;
let max_players = 4;
// TODO: 创建实时多人游戏房间

// 回合制多人游戏
// TODO: 创建回合制多人游戏房间
```

## Firebase集成

### 启用Firebase

```rust
use game_engine::platform::mobile::android_services::FirebaseService;

let mut firebase = FirebaseService::new();
firebase.initialize()?;

// 启用Analytics
firebase.enable_analytics()?;

// 启用Crashlytics
firebase.enable_crashlytics()?;

// 启用Remote Config
firebase.enable_remote_config()?;
```

### 记录事件

```rust
// 记录分析事件
let mut params = std::collections::HashMap::new();
params.insert("level".to_string(), "5".to_string());
params.insert("score".to_string(), "1000".to_string());
firebase.log_event("level_complete".to_string(), params)?;
```

### 记录错误

```rust
// 记录异常到Crashlytics
firebase.record_exception("Critical error in game loop".to_string())?;
```

### 获取远程配置

```rust
// 获取字符串配置
let difficulty = firebase.get_config_value("difficulty_level".to_string())?;

// 获取布尔配置
let enable_feature = firebase.get_config_bool("new_feature_enabled".to_string())?;
```

## 权限管理

### 请求权限

```rust
use game_engine::platform::mobile::android_services::{PermissionManager, PermissionType};

let mut permissions = PermissionManager::new();
permissions.initialize()?;

// 请求单个权限
let status = permissions.request_permission(PermissionType::Storage)?;

// 请求多个权限
let permissions_to_request = vec![
    PermissionType::Storage,
    PermissionType::Location,
];
let results = permissions.request_permissions(permissions_to_request)?;
```

### 检查权限

```rust
let has_storage = permissions.check_permission(PermissionType::Storage);
```

## 推送通知

### 初始化FCM

```rust
use game_engine::platform::mobile::services::{PushNotificationService, NotificationPlatform};

let mut fcm = PushNotificationService::new(NotificationPlatform::Firebase);
fcm.initialize()?;

// 请求通知权限
let granted = fcm.request_permission()?;
```

### 发送本地通知

```rust
use game_engine::platform::mobile::services::Notification;

let notification = Notification::new(
    "Energy Restored!".to_string(),
    "Your energy is now full.".to_string()
);
fcm.send_local_notification(notification)?;
```

### 订阅主题

```rust
// 订阅游戏更新主题
fcm.subscribe_to_topic("game_updates".to_string())?;

// 取消订阅
fcm.unsubscribe_from_topic("game_updates".to_string())?;
```

## 最佳实践

### 1. 初始化时机
- 在应用启动时初始化Play Games
- 在主菜单前完成登录
- 提供静默初始化选项

### 2. 错误处理
- 优雅处理登录失败
- 提供离线模式
- 显示有意义的错误消息

### 3. 用户体验
- 不要强制登录
- 提供清晰的成就描述
- 允许玩家随时访问Play Games界面

### 4. 测试
- 使用测试账号测试
- 测试网络断开场景
- 测试成就解锁流程
- 测试云存档同步

### 5. 性能
- 缓存玩家信息
- 批量提交成就和分数
- 异步处理网络请求

## 故障排除

### 常见问题

**问题**: Play Games登录失败
**解决方案**:
- 检查设备是否已登录Google账号
- 检查网络连接
- 验证应用签名和SHA-1指纹

**问题**: 成就未解锁
**解决方案**:
- 验证成就ID是否正确
- 检查成就是否已在Play Console中配置
- 确保玩家已登录Play Games

**问题**: 排行榜分数未更新
**解决方案**:
- 检查排行榜ID是否正确
- 验证分数格式
- 查看测试账号的分数

**问题**: 云存档未同步
**解决方案**:
- 检查Play Games应用权限
- 验证存档大小限制
- 检查网络连接

## 代码示例

### 完整的Play Games集成

```rust
use game_engine::platform::mobile::services::{GooglePlayGames, InAppPurchaseService};
use std::collections::HashMap;

struct GameManager {
    play_games: GooglePlayGames,
    iap: InAppPurchaseService,
}

impl GameManager {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut play_games = GooglePlayGames::new();
        play_games.initialize()?;
        play_games.sign_in()?;

        let mut iap = InAppPurchaseService::new();
        iap.initialize()?;

        Ok(Self { play_games, iap })
    }

    fn on_player_win(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 解锁成就
        self.play_games.unlock_achievement("achievement_first_win".to_string())?;

        // 提交分数
        let score = self.calculate_score();
        self.play_games.submit_score("leaderboard_high_scores".to_string(), score)?;

        // 保存游戏进度到云端
        let save_data = self.serialize_game_state()?;
        self.play_games.save_game_data(save_data, "After Win".to_string())?;

        Ok(())
    }

    fn on_progress_made(&mut self, enemies_killed: u32) -> Result<(), Box<dyn std::error::Error>> {
        // 更新成就进度
        self.play_games.update_achievement_progress(
            "achievement_kill_100_enemies".to_string(),
            enemies_killed
        )?;

        Ok(())
    }

    fn show_leaderboards(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.play_games.show_leaderboard("leaderboard_high_scores".to_string())?;
        Ok(())
    }

    fn calculate_score(&self) -> i64 {
        // 实现分数计算逻辑
        1000
    }

    fn serialize_game_state(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // 实现游戏状态序列化
        Ok(vec![1u8, 2, 3, 4])
    }
}
```

## 参考资料

- [Google Play Games Services](https://developers.google.com/games/services)
- [Android Developer Guide](https://developer.android.com/training/google-play)
- [Firebase Documentation](https://firebase.google.com/docs)

## 技术支持

如有问题,请联系:
- Google Play Developer Support: https://support.google.com/googleplay/android-developer
- Stack Overflow: https://stackoverflow.com/questions/tagged/google-play-games
