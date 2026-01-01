# iOS Game Center集成指南

## 概述

本指南说明如何在iOS项目中集成游戏引擎的Game Center功能。

## 前置要求

- Xcode 14.0+
- iOS 12.0+
- GameKit框架（iOS系统自带）
- 有效Apple Developer账号（用于真机测试）

## 集成步骤

### 1. 创建iOS项目

#### 使用Xcode创建新项目

```bash
# 如果已有iOS项目，跳过此步骤
# 否则使用Xcode创建新的iOS App项目
```

#### 添加Rust库依赖

在Xcode项目中：

1. 选择项目 → Build Settings
2. 搜索 "Library Search Paths"
3. 添加库路径：
   - Debug: `target/debug`
   - Release: `target/release`

4. 添加以下系统框架：
   - `GameKit.framework`
   - `Foundation.framework`
   - `UIKit.framework`

### 2. 添加GameKit桥接代码

#### 选项A: 使用Objective-C（推荐）

1. 创建新文件：`GameCenterWrapper.m`
2. 将`docs/ios_gamecenter_bridge.m`的内容复制到文件中
3. 确保文件已添加到Xcode项目的"Compile Sources"

#### 选项B: 使用Swift

1. 创建Swift文件：`GameCenterWrapper.swift`
2. 创建Bridging Header：`GameEngine-Bridging-Header.h`

**GameCenterWrapper.swift**:
```swift
import Foundation
import GameKit

@objc public class GameCenterWrapper: NSObject {
    public static let shared = GameCenterWrapper()

    private override init() {
        super.init()
    }

    @objc public func initialize() -> Bool {
        if GKLocalPlayer.local.isAuthenticated {
            return true
        }

        GKLocalPlayer.local.authenticateHandler = { vc, error in
            if let error = error {
                print("Game Center: Authentication failed - \(error.localizedDescription)")
            }
        }

        return true
    }

    // ... 其他方法（见ios_gamecenter_bridge.m）
}
```

**GameEngine-Bridging-Header.h**:
```objective-c
#ifndef GameEngine_Bridging_Header_h
#define GameEngine_Bridging_Header_h

#import "GameCenterWrapper.h"

extern int gc_initialize_ffi(void);
extern int gc_authenticate_ffi(void);
extern int gc_report_achievement_ffi(const char* identifier);
extern int gc_submit_score_ffi(const char* leaderboard_id, int64_t score);
extern int gc_show_game_center_ffi(void);
extern int gc_show_leaderboard_ffi(const char* leaderboard_id);
extern int gc_show_achievements_ffi(void);

#endif
```

3. 在Build Settings中配置Bridging Header：
   - 搜索 "Objective-C Bridging Header"
   - 设置路径：`$(PROJECT_DIR)/GameEngine-Bridging-Header.h`

### 3. 配置Xcode项目

#### 添加链接库设置

在Build Settings → Other Linker Flags中添加：

```bash
-lgame_engine  # Rust库
-lc++          # C++标准库
-framework GameKit
-framework Foundation
-framework UIKit
```

#### 启用Game Center能力

1. 选择项目 → Signing & Capabilities
2. 点击 "+ Capability"
3. 添加 "Game Center"

### 4. 初始化游戏引擎

在`AppDelegate.swift`中：

```swift
import UIKit

@main
class AppDelegate: UIResponder, UIApplicationDelegate {

    var window: UIWindow?

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {

        // 初始化Game Center
        _ = GameCenterWrapper.shared.initialize()

        // 初始化游戏引擎
        // let engine = GameEngine.create()
        // engine.start()

        return true
    }
}
```

或在`AppDelegate.m`中：

```objective-c
#import "AppDelegate.h"
#import "GameCenterWrapper.h"

@implementation AppDelegate

- (BOOL)application:(UIApplication*)application
    didFinishLaunchingWithOptions:(NSDictionary*)launchOptions {

    // 初始化Game Center
    GameCenterWrapper* gc = [[GameCenterWrapper alloc] init];
    [gc initialize];

    // 初始化游戏引擎
    // GameEngine* engine = game_engine_create();
    // game_engine_start(engine);

    return YES;
}

@end
```

### 5. 配置Game Center成就和排行榜

#### 在App Store Connect中配置

1. 登录 [App Store Connect](https://appstoreconnect.apple.com)
2. 选择 "My Apps" → 你的应用
3. 导航到 "Game Center"

#### 添加成就

1. 点击 "Achievements" → "+" 按钮
2. 填写成就信息：
   - **Reference Name**: 成就内部名称（如：achievement_first_win）
   - **ID**: 成就唯一标识（必须与代码中一致）
   - **Points**: 成就点数（1-100）
3. 上传成就本地化资源（名称、描述）

#### 添加排行榜

1. 点击 "Leaderboards" → "+" 按钮
2. 填写排行榜信息：
   - **Reference Name**: 排行榜内部名称（如：leaderboard_high_scores）
   - **ID**: 排行榜唯一标识（必须与代码中一致）
   - **Score Format**: 分数格式（Integer、Time、Decimal等）
3. 配置排行榜本地化资源

### 6. 在游戏脚本中使用

#### JavaScript示例

```javascript
// 在游戏初始化时
gc_initialize();
gc_authenticate();

// 解锁成就
function on_player_wins() {
    gc_report_achievement("achievement_first_win");
}

// 提交分数
function on_game_over(score) {
    gc_submit_score("leaderboard_high_scores", score);
}

// 显示排行榜
function show_leaderboards_button() {
    gc_show_game_center();
}
```

#### Lua示例

```lua
-- 在游戏初始化时
gc_initialize()
local authenticated = gc_authenticate()

if authenticated then
    print("Game Center认证成功")

    -- 解锁成就
    gc_report_achievement("achievement_first_win")

    -- 提交分数
    gc_submit_score("leaderboard_high_scores", 10000)
end
```

## 真机测试

### 1. 配置签名

1. 在Xcode中选择项目 → Signing & Capabilities
2. 选择你的Team（需要Apple Developer账号）
3. Xcode会自动配置证书和配置文件

### 2. 配置测试用户

在App Store Connect中：

1. 导航到 "Game Center" → "Sandbox"
2. 点击 "+" 添加测试用户
3. 创建至少一个测试用户账号

### 3. 运行测试

1. 在Xcode中选择真机设备
2. 点击 "Run"（▶️）
3. 首次运行时，Game Center会提示登录
4. 使用测试用户账号登录

### 4. 调试技巧

#### 查看日志

```swift
// 在代码中添加日志
NSLog("Game Center initialized: %d", success);
```

在Xcode中查看：
1. 打开 "Console" (⇧⌘C)
2. 选择你的设备
3. 过滤 "Game Center"

#### 常见问题

**问题1**: Game Center UI不显示
- **解决**: 确保在主线程调用UI相关函数

**问题2**: 认证失败
- **解决**: 检查网络连接，使用沙盒测试用户

**问题3**: 成就不解锁
- **解决**: 确认成就ID在App Store Connect中配置正确

**问题4**: 排行榜不更新
- **解决**: 检查排行榜ID是否一致，确认分数格式正确

## 构建发布

### 1. 创建App Store Connect记录

1. 登录App Store Connect
2. 创建新的应用记录
3. 配置应用信息（名称、Bundle ID、类别等）

### 2. 配置Game Center

1. 在App Store Connect中添加Game Center能力
2. 创建所有成就和排行榜
3. 上传本地化资源（所有支持的语言）

### 3. 提交审核

1. 在Xcode中构建Archive
2. 在"Organizer"中上传到App Store Connect
3. 填写审核信息
4. 提交审核

## 测试清单

- [ ] 初始化Game Center成功
- [ ] 用户认证流程正常
- [ ] 成就解锁成功
- [ ] 排行榜分数提交成功
- [ ] Game Center UI正常显示
- [ ] 网络断开时有适当处理
- [ ] 取消认证不会崩溃应用

## 性能考虑

1. **批量报告成就**：
   ```javascript
   // 不好的做法
   for (let i = 0; i < 100; i++) {
       gc_report_achievement(`achievement_${i}`);
   }

   // 好的做法
   // 使用批量API（如果Game Center支持）
   ```

2. **延迟UI显示**：
   - 不要在游戏关键路径中显示Game Center UI
   - 在暂停菜单或设置中显示

3. **网络超时**：
   - 设置合理的超时时间（5-10秒）
   - 提供重试机制

## 安全考虑

1. **防作弊**：
   - 在服务器端验证分数
   - 使用Game Center的内置验证

2. **隐私**：
   - 遵守Apple的隐私政策
   - 请求用户许可后才显示Game Center UI

3. **错误处理**：
   - 所有Game Center调用都应有错误处理
   - 不要让Game Center错误导致游戏崩溃

## 资源链接

- [Apple GameKit Documentation](https://developer.apple.com/documentation/gamekit)
- [Game Center Programming Guide](https://developer.apple.com/library/archive/documentation/NetworkingInternet/Conceptual/GameKit_Guide/)
- [App Store Connect](https://appstoreconnect.apple.com)

## 故障排除

### 编译错误

**错误**: `Undefined symbols for architecture arm64`
- **解决**: 检查库文件路径，确认架构匹配

**错误**: `'GameKit/GameKit.h' file not found`
- **解决**: 在Build Phases → Link Binary With Libraries中添加GameKit.framework

### 运行时错误

**错误**: `Game Center is not available`
- **解决**: 确保设备支持Game Center，检查网络连接

**错误**: `The requested operation could not be completed`
- **解决**: 检查成就/排行榜ID是否在App Store Connect中配置

## 下一步

- 配置推送通知（P1-MOBILE-001子任务）
- 实现应用内购买（P1-MOBILE-001子任务）
- 添加社交分享功能

## 支持

如有问题，请查看：
- 完成报告：`docs/P1-MOBILE-001_COMPLETION_REPORT.md`
- 桥接代码：`docs/ios_gamecenter_bridge.m`
- 示例代码：`examples/mobile_api_example.rs`
