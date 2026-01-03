# iOS Game Center Integration Guide

本文档介绍如何在游戏中集成Apple Game Center服务。

## 功能概述

### 支持的Game Center功能
- **玩家认证**: 通过Game Center登录
- **成就系统**: 解锁和管理游戏成就
- **排行榜**: 提交和查看分数
- **多人游戏**: 实时和回合制多人游戏
- **好友系统**: 访问好友列表
- **挑战功能**: 向好友发送挑战

## 设置

### 1. 配置App Store Connect

1. 登录 [App Store Connect](https://appstoreconnect.apple.com/)
2. 选择你的应用
3. 进入"功能"选项卡
4. 添加Game Center
5. 配置成就和排行榜

### 2. 配置Xcode项目

1. 在Xcode中打开项目
2. 选择应用target
3. 在"Signing & Capabilities"中添加"Game Center"能力

### 3. 代码集成

```rust
use game_engine::platform::mobile::services::GameCenter;

// 初始化Game Center
let mut game_center = GameCenter::new();
game_center.initialize()?;

// 认证玩家
game_center.authenticate()?;

if let Some(player) = game_center.get_current_player() {
    println!("Player: {} (ID: {})", player.name, player.id);
}
```

## 成就系统

### 定义成就

在App Store Connect中定义成就:
- 成就ID: `first_win`
- 成就名称: "First Win"
- 成就点数: 10

### 解锁成就

```rust
// 解锁完整成就
game_center.report_achievement("first_win".to_string())?;

// 带进度的成就 (0-100%)
game_center.report_achievement_progress(
    "kill_100_enemies".to_string(),
    45.0  // 45% complete
)?;
```

### 显示成就界面

```rust
// 显示Game Center成就视图
game_center.show_game_center()?;
```

## 排行榜

### 定义排行榜

在App Store Connect中定义排行榜:
- 排行榜ID: `high_scores`
- 排行榜名称: "High Scores"
- 分数排序: 升序或降序

### 提交分数

```rust
// 提交分数到排行榜
game_center.submit_score("high_scores".to_string(), 1000)?;
```

### 查看排行榜

```rust
// 加载排行榜分数
use game_engine::platform::mobile::ios_services::{LeaderboardTimeScope, LeaderboardPlayerScope};

let scores = game_center.load_leaderboard_scores(
    "high_scores".to_string(),
    LeaderboardTimeScope::AllTime,
    LeaderboardPlayerScope::Global
)?;

// 显示排行榜界面
game_center.show_leaderboard("high_scores".to_string())?;
```

## 多人游戏

### 启用多人游戏

```rust
game_center.enable_multiplayer()?;
```

### 创建匹配

```rust
let match_request = game_center.create_match(2, 4)?;
println!("Match created: {}", match_request.match_id);
```

### 处理多人游戏事件

```rust
// TODO: 实现多人游戏事件处理
// - 玩家连接/断开
// - 接收游戏数据
// - 发送游戏数据
```

## 最佳实践

### 1. 初始化时机
- 在应用启动时尽早初始化Game Center
- 在游戏主菜单显示前完成认证

### 2. 错误处理
- 优雅处理网络错误
- 提供离线模式
- 显示有意义的错误消息

### 3. 用户体验
- 不要在游戏过程中中断玩家
- 提供清晰的成就描述
- 允许玩家随时访问Game Center界面

### 4. 测试
- 使用Sandbox账号测试
- 测试网络断开场景
- 测试成就解锁流程

### 5. 性能
- 缓存玩家信息
- 批量提交成就和分数
- 异步处理网络请求

## 故障排除

### 常见问题

**问题**: Game Center认证失败
**解决方案**:
- 检查设备是否已登录Game Center
- 检查网络连接
- 验证Bundle ID配置

**问题**: 成就未解锁
**解决方案**:
- 验证成就ID是否正确
- 检查成就是否已在App Store Connect中配置
- 确保玩家已登录Game Center

**问题**: 排行榜分数未更新
**解决方案**:
- 检查排行榜ID是否正确
- 验证分数格式
- 查看Sandbox环境

## 代码示例

### 完整的Game Center集成

```rust
use game_engine::platform::mobile::services::GameCenter;
use std::collections::HashMap;

struct GameManager {
    game_center: GameCenter,
}

impl GameManager {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut game_center = GameCenter::new();
        game_center.initialize()?;
        game_center.authenticate()?;

        Ok(Self { game_center })
    }

    fn on_player_win(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 解锁成就
        self.game_center.report_achievement("first_win".to_string())?;

        // 提交分数
        let score = self.calculate_score();
        self.game_center.submit_score("high_scores".to_string(), score)?;

        Ok(())
    }

    fn on_progress_made(&mut self, progress: f32) -> Result<(), Box<dyn std::error::Error>> {
        // 更新成就进度
        let percent = (progress * 100.0).min(100.0);
        self.game_center.report_achievement_progress(
            "level_complete".to_string(),
            percent
        )?;

        Ok(())
    }

    fn show_leaderboards(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.game_center.show_leaderboard("high_scores".to_string())?;
        Ok(())
    }

    fn calculate_score(&self) -> i64 {
        // 实现分数计算逻辑
        1000
    }
}
```

## 参考资料

- [Apple Game Center Documentation](https://developer.apple.com/documentation/gamekit)
- [Game Center Programming Guide](https://developer.apple.com/library/archive/documentation/NetworkingInternet/Conceptual/GameKit_Guide/)
- [App Store Connect Help](https://help.apple.com/app-store-connect/)

## 技术支持

如有问题,请联系:
- Apple Developer Support: https://developer.apple.com/support/
- Game Center Forums: https://developer.apple.com/forums/
