# 推送通知服务配置指南

## 概述

本指南说明如何为游戏引擎配置推送通知服务：
- **Android**: Firebase Cloud Messaging (FCM)
- **iOS**: Apple Push Notification Service (APNs)

## Android FCM配置

### 1. 创建Firebase项目

1. 访问 [Firebase Console](https://console.firebase.google.com/)
2. 创建新项目或选择现有项目
3. 添加Android应用：
   - 下载 `google-services.json`
   - 放置在 `app/` 目录下

### 2. 配置build.gradle

**项目级 build.gradle**:
```gradle
buildscript {
    dependencies {
        classpath 'com.google.gms:google-services:4.3.15'
    }
}
```

**应用级 build.gradle**:
```gradle
plugins {
    id 'com.google.gms.google-services'
}

dependencies {
    implementation 'com.google.firebase:firebase-messaging:23.4.0'
}
```

### 3. 创建FCM服务类

**FCMWrapper.kt**:
```kotlin
package com.gameengine.mobile

import com.google.firebase.messaging.FirebaseMessaging
import com.google.firebase.messaging.FirebaseMessagingService

class FCMWrapper : FirebaseMessagingService() {

    override fun onNewToken(token: String) {
        // 将token发送到游戏引擎
        Log.d("FCM", "New token: $token")
    }

    override fun onMessageReceived(remoteMessage: RemoteMessage) {
        // 处理收到的消息
        val title = remoteMessage.notification?.title ?: "通知"
        val body = remoteMessage.notification?.body ?: ""

        // 显示通知
        showNotification(title, body)
    }

    private fun showNotification(title: String, body: String) {
        // 实现通知显示逻辑
    }

    companion object {
        init {
            System.loadLibrary("game_engine")
        }

        external fun initialize(): Boolean
        external fun requestPermission(): Boolean
        external fun subscribeToTopic(topic: String): Boolean
    }
}
```

**FFI桥接实现**:
```kotlin
// FCMFFI.kt
package com.gameengine.mobile

class FCMFFI {

    external fun initialize(): Int {
        return try {
            FirebaseMessaging.getInstance().token.addOnSuccessListener { token ->
                // Token available
            }
            1 // Success
        } catch (e: Exception) {
            0 // Failure
        }
    }

    external fun requestPermission(): Int {
        // Android 13+需要请求POST_NOTIFICATIONS权限
        return 1 // Authorized
    }

    external fun subscribeToTopic(topic: String): Int {
        FirebaseMessaging.getInstance().subscribeToTopic(topic)
            .addOnCompleteListener { task ->
                if (task.isSuccessful) {
                    Log.d("FCM", "Subscribed to $topic")
                }
            }
        return 1
    }
}
```

### 4. AndroidManifest.xml配置

```xml
<manifest>
    <application>
        <service
            android:name=".FCMWrapper"
            android:exported="false">
            <intent-filter>
                <action android:name="com.google.firebase.MESSAGING_EVENT" />
            </intent-filter>
        </service>

        <!-- Android 13+通知权限 -->
        <uses-permission android:name="android.permission.POST_NOTIFICATIONS"/>
    </application>
</manifest>
```

## iOS APNs配置

### 1. 创建推送证书

1. 访问 [Apple Developer](https://developer.apple.com/)
2. Certificates, Identifiers & Profiles → Identifiers
3. 创建App ID（启用Push Notifications能力）
4. 创建推送证书（开发/生产）

### 2. 创建NotificationService扩展

**NotificationService.swift**:
```swift
import UserNotifications
import UIKit

class NotificationService: NSObject {

    static let shared = NotificationService()

    func requestPermission(completion: @escaping (Bool) -> Void) {
        let center = UNUserNotificationCenter.current()
        center.requestAuthorization(options: [.alert, .sound, .badge]) { granted, error in
            DispatchQueue.main.async {
                completion(granted)
            }
        }
    }

    func sendLocalNotification(title: String, body: String) {
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        content.sound = .default

        let trigger = UNTimeIntervalNotificationTrigger(timeInterval: 1, repeats: false)
        let request = UNNotificationRequest(identifier: UUID().uuidString, content: content, trigger: trigger)

        UNUserNotificationCenter.current().add(request)
    }
}
```

### 3. FFI桥接实现

**PushNotificationsFFI.m**:
```objective-c
#import <Foundation/Foundation.h>
#import <UserNotifications/UserNotifications.h>

int apns_initialize_ffi(void) {
    // 注册远程通知
    UIUserNotificationSettings* settings = [UIUserNotificationSettings settingsForTypes:
        UIUserNotificationTypeAlert | UIUserNotificationTypeSound | UIUserNotificationTypeBadge
        categories:nil];

    [[UIApplication sharedApplication] registerUserNotificationSettings:settings];

    return 1; // Success
}

int apns_request_permission_ffi(void) {
    UNUserNotificationCenter* center = [UNUserNotificationCenter currentNotificationCenter];

    dispatch_semaphore_t sem = dispatch_semaphore_create(0);
    __block BOOL granted = NO;

    [center requestAuthorizationWithOptions:(UNAuthorizationOptionAlert | UNAuthorizationOptionSound)
        completionHandler:^(BOOL success, NSError* error) {
            granted = success;
            dispatch_semaphore_signal(sem);
        }];

    dispatch_semaphore_wait(sem, dispatch_time(DISPATCH_TIME_NOW, 5 * NSEC_PER_SEC));

    return granted ? 2 : 1; // 2=Authorized, 1=Denied
}

int apns_send_local_notification_ffi(const char* title, const char* body) {
    NSString* titleStr = [NSString stringWithUTF8String:title];
    NSString* bodyStr = [NSString stringWithUTF8String:body];

    UNMutableNotificationContent* content = [[UNMutableNotificationContent alloc] init];
    content.title = titleStr;
    content.body = bodyStr;
    content.sound = [UNNotificationSound defaultSound];

    UNTimeIntervalNotificationTrigger* trigger = [UNTimeIntervalNotificationTrigger triggerWithTimeInterval:1 repeats:NO];
    UNNotificationRequest* request = [UNNotificationRequest requestWithIdentifier:[[NSUUID UUID] UUIDString] content:content trigger:trigger];

    [[UNUserNotificationCenter currentNotificationCenter] addNotificationRequest:request withCompletionHandler:nil];

    return 1; // Success
}
```

## 脚本API使用

### JavaScript示例

```javascript
// 初始化推送通知
push_initialize();

// 请求权限
const granted = push_request_permission();
if (granted) {
    console.log("通知权限已授予");

    // 发送本地通知
    push_send_local("每日奖励", "登录游戏领取每日奖励！");

    // 订阅主题（仅Android FCM）
    push_subscribe_to_topic("news_updates");
} else {
    console.log("通知权限被拒绝");
}
```

### Lua示例

```lua
-- 初始化
push_initialize()

-- 请求权限
local granted = push_request_permission()

if granted then
    print("通知权限已授予")

    -- 发送通知
    push_send_local("游戏邀请", "你的好友邀请你加入游戏！")
end
```

## 测试

### Android测试

1. 发送测试通知：
   - Firebase Console → Cloud Messaging
   - 发送消息到测试设备

2. 验证主题订阅：
   - 订阅测试主题
   - 发送主题消息

### iOS测试

1. 发送测试推送：
   - 使用第三方工具（如 Pusher）
   - 或使用APNs Tester工具

2. 验证本地通知：
   - 调用 `push_send_local()`
   - 检查通知是否显示

## 常见问题

**Android**: FCM token为null
- 检查 `google-services.json` 是否正确放置
- 确认项目配置正确

**iOS**: 权限请求不显示
- iOS需要在Info.plist中添加权限说明
- 检查Push Notifications capability是否启用

**跨平台**: 通知不显示
- 确认权限已授予
- 检查通知渠道设置（Android 8.0+）
- 检查系统通知设置

## 性能建议

1. **批量操作**: 避免频繁发送通知
2. **主题订阅**: 使用主题而非单点发送
3. **本地缓存**: 缓存token和权限状态
4. **重试机制**: 实现指数退避重试

## 安全建议

1. **验证消息**: 在服务端验证所有推送消息
2. **加密敏感数据**: 不在通知中包含敏感信息
3. **Token管理**: 安全存储和传输FCM/APNs token

## 下一步

- 配置远程通知服务器
- 实现通知数据加密
- 添加通知统计和分析
