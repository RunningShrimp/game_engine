//! 移动平台游戏示例
//!
//! 演示游戏引擎在移动平台上的完整功能

use game_engine::platform::mobile::{
    TouchEvent, GestureType, GestureEvent, GestureRecognizer,
    VirtualJoystick, VirtualButton, MobileInputManager,
    MobileConfig, MobilePerformanceMonitor, MobileAdaptivePerformance,
    GooglePlayGames, GameCenter, PushNotificationService, NotificationPlatform,
    Notification,
};
use game_engine::platform::mobile::input::SwipeDirection;
use glam::Vec2;
use std::collections::HashMap;

fn main() {
    println!("=== 游戏引擎移动平台功能演示 ===\n");

    // 示例1: 多点触控
    example_1_multi_touch();

    // 示例2: 手势识别
    example_2_gesture_recognition();

    // 示例3: 虚拟摇杆
    example_3_virtual_joystick();

    // 示例4: 虚拟按钮
    example_4_virtual_buttons();

    // 示例5: 移动性能优化
    example_5_performance_optimization();

    // 示例6: Google Play Games集成
    example_6_google_play_games();

    // 示例7: Game Center集成
    example_7_game_center();

    // 示例8: 推送通知
    example_8_push_notifications();

    // 示例9: 完整移动游戏架构
    example_9_full_mobile_game();
}

/// 示例1: 多点触控
fn example_1_multi_touch() {
    println!("=== 示例1: 多点触控系统 ===\n");

    // 创建移动输入管理器
    let mut input_manager = MobileInputManager::new();

    println!("✓ 多点触控支持:");
    println!("  - 最大同时触摸点: 10");
    println!("  - 触摸ID追踪");
    println!("  - 压力感应支持");
    println!("  - 触摸时间戳\n");

    // 模拟多点触控事件
    let touch1_start = TouchEvent::Started {
        touch_id: 1,
        position: Vec2::new(100.0, 200.0),
    };

    let touch2_start = TouchEvent::Started {
        touch_id: 2,
        position: Vec2::new(300.0, 200.0),
    };

    input_manager.handle_touch(&touch1_start);
    input_manager.handle_touch(&touch2_start);

    println!("✓ 同时处理2个触摸点:");
    println!("  - 触摸点1: (100, 200)");
    println!("  - 触摸点2: (300, 200)");
    println!("  - 活跃触摸数: {}\n", input_manager.gesture_recognizer.active_touches.len());
}

/// 示例2: 手势识别
fn example_2_gesture_recognition() {
    println!("=== 示例2: 手势识别系统 ===\n");

    let mut recognizer = GestureRecognizer::new();

    println!("✓ 支持的手势类型:");
    println!("  1. Tap - 点击");
    println!("  2. DoubleTap - 双击");
    println!("  3. LongPress - 长按 (0.5秒)");
    println!("  4. Swipe - 滑动 (上下左右)");
    println!("  5. Pinch - 双指缩放");
    println!("  6. Rotation - 双指旋转\n");

    // 模拟滑动事件
    let swipe_start = TouchEvent::Started {
        touch_id: 1,
        position: Vec2::new(100.0, 100.0),
    };

    let swipe_move = TouchEvent::Moved {
        touch_id: 1,
        position: Vec2::new(200.0, 100.0),
        delta: Vec2::new(100.0, 0.0),
    };

    let swipe_end = TouchEvent::Ended {
        touch_id: 1,
        position: Vec2::new(200.0, 100.0),
    };

    recognizer.handle_touch(&swipe_start);
    recognizer.handle_touch(&swipe_move);

    if let Some(GestureEvent { gesture_type, position, parameters }) = recognizer.handle_touch(&swipe_end) {
        println!("✓ 检测到手势: {:?}", gesture_type);
        println!("  - 位置: ({:.1}, {:.1})", position.x, position.y);

        if let GestureType::Swipe { direction } = gesture_type {
            println!("  - 方向: {:?}", direction);
            println!("  - 滑动距离: {:.1}", parameters.get("distance").unwrap_or(&0.0));
            println!("  - 滑动角度: {:.1}°\n", parameters.get("angle").unwrap_or(&0.0));
        }
    }
}

/// 示例3: 虚拟摇杆
fn example_3_virtual_joystick() {
    println!("=== 示例3: 虚拟摇杆 ===\n");

    // 创建左侧虚拟摇杆（移动）
    let mut left_joystick = VirtualJoystick::new(
        "left_joystick".to_string(),
        Vec2::new(100.0, 500.0),  // 左下角位置
        120.0,                     // 大小
    );

    // 创建右侧虚拟摇杆（相机）
    let mut right_joystick = VirtualJoystick::new(
        "right_joystick".to_string(),
        Vec2::new(700.0, 500.0), // 右下角位置
        120.0,                    // 大小
    );

    println!("✓ 虚拟摇杆配置:");
    println!("  - 左摇杆: (100, 500) - 角色移动");
    println!("  - 右摇杆: (700, 500) - 相机控制");
    println!("  - 摇杆大小: 120x120 像素\n");

    // 模拟摇杆输入
    let touch_event = TouchEvent::Started {
        touch_id: 1,
        position: Vec2::new(100.0, 500.0),
    };

    left_joystick.handle_touch(&touch_event);

    // 模拟摇杆移动
    let move_event = TouchEvent::Moved {
        touch_id: 1,
        position: Vec2::new(150.0, 450.0), // 向右上移动
        delta: Vec2::new(50.0, -50.0),
    };

    left_joystick.handle_touch(&move_event);

    println!("✓ 摇杆状态:");
    println!("  - 激活: {}", left_joystick.active);
    println!("  - 值: ({:.2}, {:.2})", left_joystick.value.x, left_joystick.value.y);
    println!("  - 角度: {:.1}°", left_joystick.value.y.atan2(left_joystick.value.x).to_degrees());
    println!("  - 强度: {:.2}%\n", left_joystick.value.length() * 100.0);
}

/// 示例4: 虚拟按钮
fn example_4_virtual_buttons() {
    println!("=== 示例4: 虚拟按钮 ===\n");

    // 创建动作按钮
    let mut jump_button = VirtualButton::new(
        "jump".to_string(),
        Vec2::new(650.0, 400.0),
        Vec2::new(80.0, 80.0),
        "Jump".to_string(),
    );

    let mut attack_button = VirtualButton::new(
        "attack".to_string(),
        Vec2::new(750.0, 400.0),
        Vec2::new(80.0, 80.0),
        "Attack".to_string(),
    );

    let mut pause_button = VirtualButton::new(
        "pause".to_string(),
        Vec2::new(750.0, 50.0),
        Vec2::new(60.0, 60.0),
        "||".to_string(),
    );

    println!("✓ 虚拟按钮配置:");
    println!("  - Jump按钮: (650, 400) - 80x80");
    println!("  - Attack按钮: (750, 400) - 80x80");
    println!("  - Pause按钮: (750, 50) - 60x60\n");

    // 模拟按钮按下
    let touch_event = TouchEvent::Started {
        touch_id: 1,
        position: Vec2::new(650.0, 400.0),
    };

    jump_button.handle_touch(&touch_event);

    println!("✓ 按钮状态:");
    println!("  - Jump按钮: {}", if jump_button.pressed { "按下" } else { "释放" });
    println!("  - Attack按钮: {}", if attack_button.pressed { "按下" } else { "释放" });
    println!("  - Pause按钮: {}", if pause_button.pressed { "按下" } else { "释放" });
    println!();
}

/// 示例5: 移动性能优化
fn example_5_performance_optimization() {
    println!("=== 示例5: 移动性能优化 ===\n");

    // 创建移动配置
    let config = MobileConfig {
        target_fps: 60,
        adaptive_fps: true,
        power_saving: true,
        thermal_throttling_detection: true,
        max_resolution_scale: 1.0,
        dynamic_resolution: true,
        touch_sensitivity: 1.0,
        gyroscope_enabled: false,
    };

    // 创建自适应性能管理器
    let mut adaptive_perf = MobileAdaptivePerformance::new(config);

    println!("✓ 自适应性能配置:");
    println!("  - 目标帧率: {} FPS", adaptive_perf.target_fps());
    println!("  - 分辨率缩放: {:.0}%", adaptive_perf.resolution_scale() * 100.0);
    println!("  - 自适应帧率: 启用");
    println!("  - 省电模式: 启用");
    println!("  - 热节流检测: 启用\n");

    // 模拟性能更新
    println!("✓ 性能自适应测试:");
    for frame in 0..10 {
        let frame_time = if frame < 5 { 16.67 } else { 25.0 }; // 前5帧正常，后5帧慢
        let timestamp = frame as f64 * 0.016;
        adaptive_perf.update(frame_time, timestamp);

        println!("  帧 {}: {:.2}ms -> 分辨率 {:.0}%, 目标 {} FPS",
            frame + 1,
            frame_time,
            adaptive_perf.resolution_scale() * 100.0,
            adaptive_perf.target_fps()
        );
    }
    println!();
}

/// 示例6: Google Play Games集成
fn example_6_google_play_games() {
    println!("=== 示例6: Google Play Games集成 ===\n");

    let mut gpg = GooglePlayGames::new();

    // 初始化
    println!("✓ 初始化Google Play Games...");
    if let Err(e) = gpg.initialize() {
        println!("  错误: {:?}\n", e);
        return;
    }
    println!("  成功!\n");

    // 登录
    println!("✓ 玩家登录...");
    if let Err(e) = gpg.sign_in() {
        println!("  错误: {:?}\n", e);
        return;
    }
    println!("  成功!\n");

    // 获取玩家信息
    if let Some(player) = gpg.get_current_player() {
        println!("✓ 玩家信息:");
        println!("  - ID: {}", player.id);
        println!("  - 名称: {}", player.name);
        println!("  - 等级: {}\n", player.level);
    }

    // 解锁成就
    println!("✓ 解锁成就...");
    let _ = gpg.unlock_achievement("achievement_first_win".to_string());
    println!("  - 'First Win' 成就已解锁!\n");

    // 更新成就进度
    println!("✓ 更新成就进度...");
    let _ = gpg.update_achievement_progress("achievement_kills".to_string(), 75);
    println!("  - 'Kills' 成就进度: 75%\n");

    // 提交分数
    println!("✓ 提交分数...");
    let _ = gpg.submit_score("leaderboard_highscore".to_string(), 10000);
    println!("  - 分数 10,000 已提交到排行榜\n");
}

/// 示例7: Game Center集成
fn example_7_game_center() {
    println!("=== 示例7: Game Center集成 ===\n");

    let mut gc = GameCenter::new();

    // 初始化
    println!("✓ 初始化Game Center...");
    if let Err(e) = gc.initialize() {
        println!("  错误: {:?}\n", e);
        return;
    }
    println!("  成功!\n");

    // 认证
    println!("✓ 玩家认证...");
    if let Err(e) = gc.authenticate() {
        println!("  错误: {:?}\n", e);
        return;
    }
    println!("  成功!\n");

    // 获取玩家信息
    if let Some(player) = gc.get_current_player() {
        println!("✓ 玩家信息:");
        println!("  - ID: {}", player.id);
        println!("  - 名称: {}", player.name);
        println!("  - 等级: {}\n", player.level);
    }

    // 报告成就
    println!("✓ 报告成就...");
    let _ = gc.report_achievement("achievement_level_10".to_string());
    println!("  - 'Level 10' 成就已报告!\n");

    // 提交分数
    println!("✓ 提交分数...");
    let _ = gc.submit_score("leaderboard_score".to_string(), 5000);
    println!("  - 分数 5,000 已提交到排行榜\n");
}

/// 示例8: 推送通知
fn example_8_push_notifications() {
    println!("=== 示例8: 推送通知 ===\n");

    // Android - Firebase
    println!("✓ Firebase Cloud Messaging (Android):");
    let mut fcm_service = PushNotificationService::new(NotificationPlatform::Firebase);

    let _ = fcm_service.initialize();
    println!("  - 初始化: 成功");

    if let Ok(granted) = fcm_service.request_permission() {
        println!("  - 通知权限: {}", if granted { "已授予" } else { "被拒绝" });
    }

    let notification = Notification::new(
        "Energy Restored!".to_string(),
        "Your energy is fully restored. Come back and play!".to_string(),
    )
    .with_icon("notification_icon.png".to_string())
    .with_data("type".to_string(), "energy_restored".to_string())
    .with_data("amount".to_string(), "100".to_string());

    let _ = fcm_service.send_local_notification(notification);
    println!("  - 发送通知: 成功\n");

    // iOS - APNs
    println!("✓ Apple Push Notification Service (iOS):");
    let mut apns_service = PushNotificationService::new(NotificationPlatform::APNs);

    let _ = apns_service.initialize();
    println!("  - 初始化: 成功");

    if let Ok(granted) = apns_service.request_permission() {
        println!("  - 通知权限: {}", if granted { "已授予" } else { "被拒绝" });
    }

    let notification = Notification::new(
        "Gift Ready!".to_string(),
        "You have a free gift waiting for you!".to_string(),
    )
    .with_icon("gift_icon.png".to_string());

    let _ = apns_service.send_local_notification(notification);
    println!("  - 发送通知: 成功\n");
}

/// 示例9: 完整移动游戏架构
fn example_9_full_mobile_game() {
    println!("=== 示例9: 完整移动游戏架构 ===\n");

    println!("✓ 移动游戏完整架构:");
    println!();
    println!("┌─────────────────────────────────────────────┐");
    println!("│          移动游戏引擎架构                     │");
    println!("├─────────────────────────────────────────────┤");
    println!("│  输入层                                      │");
    println!("│  ├── 多点触控系统 (10点同时)                │");
    println!("│  ├── 手势识别 (Tap/Swipe/Pinch/Rotation)   │");
    println!("│  ├── 虚拟摇杆 (左摇杆-移动, 右摇杆-相机)    │");
    println!("│  └── 虚拟按钮 (动作/技能/暂停)              │");
    println!("├─────────────────────────────────────────────┤");
    println!("│  性能层                                      │");
    println!("│  ├── 自适应帧率 (30/60 FPS)                │");
    println!("│  ├── 动态分辨率缩放 (50%-100%)              │");
    println!("│  ├── 热节流检测                             │");
    println!("│  └── 电池优化                               │");
    println!("├─────────────────────────────────────────────┤");
    println!("│  服务层                                      │");
    println!("│  ├── Google Play Games (成就/排行榜)       │");
    println!("│  ├── Game Center (成就/排行榜)              │");
    println!("│  ├── Firebase Cloud Messaging (Android)     │");
    println!("│  └── Apple Push Notification (iOS)          │");
    println!("├─────────────────────────────────────────────┤");
    println!("│  游戏层                                      │");
    println!("│  ├── 角色控制器                              │");
    println!("│  ├── 相机系统                                │");
    println!("│  ├── UI系统                                  │");
    println!("│  └── 音频系统                                │");
    println!("└─────────────────────────────────────────────┘");
    println!();

    println!("✓ 典型移动游戏操作流程:");
    println!("  1. 玩家使用左摇杆控制角色移动");
    println!("  2. 玩家使用右摇杆控制相机视角");
    println!("  3. 点击'Jump'按钮执行跳跃");
    println!("  4. 点击'Attack'按钮执行攻击");
    println!("  5. 滑动屏幕进行快捷操作");
    println!("  6. 双指缩放调整小地图");
    println!();

    println!("✓ 性能优化策略:");
    println!("  - 目标帧率: 60 FPS (高端设备), 30 FPS (中低端)");
    println!("  - 分辨率: 根据性能动态调整 (50%-100%)");
    println!("  - 纹理: 使用ASTC/ETC2压缩");
    println!("  - 阴影: 降低质量或禁用");
    println!("  - 后处理: 简化或禁用");
    println!("  - Draw Distance: 50-100米");
    println!();

    println!("✓ 电池优化:");
    println!("  - 低电量时降低到30 FPS");
    println!("  - 热节流时降低分辨率和帧率");
    println!("  - 后台时暂停渲染");
    println!("  - 使用自适应性能");
    println!();

    println!("✓ 平台集成:");
    println!("  Android:");
    println!("    - Google Play Games 成就系统");
    println!("    - Firebase 推送通知");
    println!("    - 应用内购买 (Google Play Billing)");
    println!("    - 广告 (AdMob)");
    println!();
    println!("  iOS:");
    println!("    - Game Center 成就系统");
    println!("    - APNs 推送通知");
    println!("    - Store Kit 应用内购买");
    println!("    - TestFlight 测试");
    println!();
}

/// 完整的游戏循环示例
pub struct MobileGame {
    input_manager: MobileInputManager,
    adaptive_performance: MobileAdaptivePerformance,
    google_play_games: Option<GooglePlayGames>,
    game_center: Option<GameCenter>,
    push_notification: Option<PushNotificationService>,
}

impl MobileGame {
    /// 创建新的移动游戏
    pub fn new() -> Self {
        // 创建移动配置
        let config = MobileConfig {
            target_fps: 60,
            adaptive_fps: true,
            power_saving: true,
            thermal_throttling_detection: true,
            max_resolution_scale: 1.0,
            dynamic_resolution: true,
            touch_sensitivity: 1.0,
            gyroscope_enabled: false,
        };

        // 创建虚拟控件
        let mut input_manager = MobileInputManager::new();

        // 添加左摇杆（移动）
        let left_joystick = VirtualJoystick::new(
            "left_stick".to_string(),
            Vec2::new(100.0, 500.0),
            120.0,
        );
        input_manager.add_joystick(left_joystick);

        // 添加右摇杆（相机）
        let right_joystick = VirtualJoystick::new(
            "right_stick".to_string(),
            Vec2::new(700.0, 500.0),
            120.0,
        );
        input_manager.add_joystick(right_joystick);

        // 添加按钮
        let jump_button = VirtualButton::new(
            "jump".to_string(),
            Vec2::new(650.0, 400.0),
            Vec2::new(80.0, 80.0),
            "Jump".to_string(),
        );
        input_manager.add_button(jump_button);

        let attack_button = VirtualButton::new(
            "attack".to_string(),
            Vec2::new(750.0, 400.0),
            Vec2::new(80.0, 80.0),
            "Attack".to_string(),
        );
        input_manager.add_button(attack_button);

        Self {
            input_manager,
            adaptive_performance: MobileAdaptivePerformance::new(config),
            google_play_games: None,
            game_center: None,
            push_notification: None,
        }
    }

    /// 初始化平台服务
    pub fn initialize_platform_services(&mut self, platform: &str) {
        match platform {
            "android" => {
                let mut gpg = GooglePlayGames::new();
                let _ = gpg.initialize();
                let _ = gpg.sign_in();
                self.google_play_games = Some(gpg);

                let mut fcm = PushNotificationService::new(NotificationPlatform::Firebase);
                let _ = fcm.initialize();
                self.push_notification = Some(fcm);
            }
            "ios" => {
                let mut gc = GameCenter::new();
                let _ = gc.initialize();
                let _ = gc.authenticate();
                self.game_center = Some(gc);

                let mut apns = PushNotificationService::new(NotificationPlatform::APNs);
                let _ = apns.initialize();
                self.push_notification = Some(apns);
            }
            _ => {}
        }
    }

    /// 游戏更新
    pub fn update(&mut self, delta_time: f32) {
        // 更新自适应性能
        let frame_time_ms = delta_time * 1000.0;
        self.adaptive_performance.update(frame_time_ms, 0.0);

        // 更新输入
        // 处理触摸事件...

        // 更新游戏逻辑
        // 根据摇杆值移动角色
        // 检查按钮状态
        // 处理手势事件
    }

    /// 渲染
    pub fn render(&self) {
        // 使用当前分辨率缩放渲染
        let resolution_scale = self.adaptive_performance.resolution_scale();
        // 渲染游戏...
    }
}

impl Default for MobileGame {
    fn default() -> Self {
        Self::new()
    }
}
