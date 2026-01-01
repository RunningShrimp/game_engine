// GameCenterWrapper.m
// iOS GameKit FFI桥接实现

#import <Foundation/Foundation.h>
#import <GameKit/GameKit.h>

// ===== FFI函数声明 =====

/// 初始化Game Center
int gc_initialize_ffi(void) {
    if ([GKLocalPlayer localPlayer].authenticated) {
        return 1; // 已认证
    }

    // 触发认证（但不等待完成）
    [[GKLocalPlayer localPlayer] setAuthenticateHandler:(UIViewController^viewController, NSError^error) {
        if (viewController != nil) {
            // 需要显示登录界面（由主应用处理）
            NSLog(@"Game Center: View controller provided for authentication");
        } else if (error != nil) {
            NSLog(@"Game Center: Authentication failed - %@", error.localizedDescription);
        } else {
            NSLog(@"Game Center: Authentication successful");
        }
    }];

    return 1;
}

/// 认证用户
int gc_authenticate_ffi(void) {
    GKLocalPlayer* localPlayer = [GKLocalPlayer localPlayer];

    if (localPlayer.authenticated) {
        return 1; // 已认证
    }

    __block BOOL authenticationComplete = NO;
    __block BOOL authenticationSuccess = NO;

    [localPlayer setAuthenticateHandler:(UIViewController^viewController, NSError^error) {
        authenticationComplete = YES;
        authenticationSuccess = (error == nil);

        if (error != nil) {
            NSLog(@"Game Center: Authentication error - %@", error.localizedDescription);
        } else {
            NSLog(@"Game Center: User authenticated - %@", localPlayer.displayName);
        }
    }];

    // 等待认证完成（最多5秒）
    NSDate* deadline = [NSDate dateWithTimeIntervalSinceNow:5.0];
    while (!authenticationComplete && [[NSDate date] compare:deadline] == NSOrderedAscending) {
        [[NSRunLoop currentRunLoop] runMode:NSDefaultRunLoopMode beforeDate:[NSDate dateWithTimeIntervalSinceNow:0.1]];
    }

    return authenticationSuccess ? 1 : 0;
}

/// 报告成就
int gc_report_achievement_ffi(const char* identifier) {
    if (identifier == NULL) {
        return 0;
    }

    NSString* achievementId = [NSString stringWithUTF8String:identifier];

    GKAchievement* achievement = [[GKAchievement alloc] initWithIdentifier:achievementId];
    achievement.percentComplete = 100.0;
    achievement.showsCompletionBanner = YES;

    [GKAchievement reportAchievements:@[achievement] withCompletionHandler:^(NSError* error) {
        if (error != nil) {
            NSLog(@"Game Center: Achievement report failed - %@", error.localizedDescription);
        } else {
            NSLog(@"Game Center: Achievement reported - %@", achievementId);
        }
    }];

    return 1;
}

/// 提交分数
int gc_submit_score_ffi(const char* leaderboard_id, int64_t score) {
    if (leaderboard_id == NULL) {
        return 0;
    }

    NSString* leaderboardId = [NSString stringWithUTF8String:leaderboard_id];

    GKScore* scoreReporter = [[GKScore alloc] initWithLeaderboardIdentifier:leaderboardId];
    scoreReporter.value = score;

    [GKScore reportScores:@[scoreReporter] withCompletionHandler:^(NSError* error) {
        if (error != nil) {
            NSLog(@"Game Center: Score submission failed - %@", error.localizedDescription);
        } else {
            NSLog(@"Game Center: Score submitted - %lld to %@", score, leaderboardId);
        }
    }];

    return 1;
}

/// 显示Game Center仪表板
int gc_show_game_center_ffi(void) {
    GKGameCenterViewController* gameCenterController = [[GKGameCenterViewController alloc] init];

    if (gameCenterController == nil) {
        NSLog(@"Game Center: Failed to create game center view controller");
        return 0;
    }

    // 注意：需要在主线程显示ViewController
    // 此函数只触发显示，实际显示由主应用的UI系统处理
    dispatch_async(dispatch_get_main_queue(), ^{
        // 这里需要获取root view controller来present
        // 实际实现由主应用提供
        UIViewController* rootVC = [UIApplication sharedApplication].keyWindow.rootViewController;

        if (rootVC != nil) {
            [rootVC presentViewController:gameCenterController animated:YES completion:nil];
        } else {
            NSLog(@"Game Center: No root view controller available");
        }
    });

    return 1;
}

/// 显示排行榜
int gc_show_leaderboard_ffi(const char* leaderboard_id) {
    if (leaderboard_id == NULL) {
        return 0;
    }

    NSString* leaderboardId = [NSString stringWithUTF8String:leaderboard_id];

    GKLeaderboardViewController* leaderboardVC = [[GKLeaderboardViewController alloc] init];
    leaderboardVC.leaderboardIdentifier = leaderboardId;

    if (leaderboardVC == nil) {
        NSLog(@"Game Center: Failed to create leaderboard view controller");
        return 0;
    }

    dispatch_async(dispatch_get_main_queue(), ^{
        UIViewController* rootVC = [UIApplication sharedApplication].keyWindow.rootViewController;

        if (rootVC != nil) {
            [rootVC presentViewController:leaderboardVC animated:YES completion:nil];
        } else {
            NSLog(@"Game Center: No root view controller available");
        }
    });

    return 1;
}

/// 显示成就
int gc_show_achievements_ffi(void) {
    GKAchievementViewController* achievementsVC = [[GKAchievementViewController alloc] init];

    if (achievementsVC == nil) {
        NSLog(@"Game Center: Failed to create achievements view controller");
        return 0;
    }

    dispatch_async(dispatch_get_main_queue(), ^{
        UIViewController* rootVC = [UIApplication sharedApplication].keyWindow.rootViewController;

        if (rootVC != nil) {
            [rootVC presentViewController:achievementsVC animated:YES completion:nil];
        } else {
            NSLog(@"Game Center: No root view controller available");
        }
    });

    return 1;
}

// ===== Swift版本（推荐） =====

/*
 如果使用Swift，创建GameCenterWrapper.swift：

 ```swift
 import Foundation
 import GameKit

 @objc public class GameCenterWrapper: NSObject {

     public static let shared = GameCenterWrapper()

     private override init() {
         super.init()
     }

     // MARK: - 初始化

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

     // MARK: - 认证

     @objc public func authenticate() -> Bool {
         let player = GKLocalPlayer.local

         guard !player.isAuthenticated else {
             return true
         }

         var authenticated = false
         let semaphore = DispatchSemaphore(value: 0)

         player.authenticateHandler = { vc, error in
             if let error = error {
                 print("Game Center: Authentication error - \(error.localizedDescription)")
             } else {
                 authenticated = true
                 print("Game Center: Authenticated - \(player.displayName ?? "Unknown")")
             }
             semaphore.signal()
         }

         _ = semaphore.wait(timeout: .now() + 5)
         return authenticated
     }

     // MARK: - 成就

     @objc public func reportAchievement(identifier: String) -> Bool {
         let achievement = GKAchievement(identifier: identifier)
         achievement.percentComplete = 100.0
         achievement.showsCompletionBanner = true

         GKAchievement.report([achievement]) { error in
             if let error = error {
                 print("Game Center: Achievement report failed - \(error.localizedDescription)")
             } else {
                 print("Game Center: Achievement reported - \(identifier)")
             }
         }

         return true
     }

     // MARK: - 排行榜

     @objc public func submitScore(leaderboardId: String, score: Int64) -> Bool {
         let scoreReporter = GKScore(leaderboardIdentifier: leaderboardId)
         scoreReporter.value = score

         GKScore.report([scoreReporter]) { error in
             if let error = error {
                 print("Game Center: Score submission failed - \(error.localizedDescription)")
             } else {
                 print("Game Center: Score submitted - \(score) to \(leaderboardId)")
             }
         }

         return true
     }

     // MARK: - UI

     @objc public func showGameCenter() -> Bool {
         guard let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
               let rootVC = windowScene.windows.first?.rootViewController else {
             return false
         }

         let gcVC = GKGameCenterViewController()
         rootVC.present(gcVC, animated: true)
         return true
     }

     @objc public func showLeaderboard(leaderboardId: String) -> Bool {
         guard let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
               let rootVC = windowScene.windows.first?.rootViewController else {
             return false
         }

         let leaderboardVC = GKLeaderboardViewController()
         leaderboardVC.leaderboardIdentifier = leaderboardId
         rootVC.present(leaderboardVC, animated: true)
         return true
     }

     @objc public func showAchievements() -> Bool {
         guard let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
               let rootVC = windowScene.windows.first?.rootViewController else {
             return false
         }

         let achievementsVC = GKAchievementViewController()
         rootVC.present(achievementsVC, animated: true)
         return true
     }
 }
 ```

 使用Swift版本时，需要在Bridging Header中声明C函数：

 ```objective-c
 // GameEngine-Bridging-Header.h
 #import "GameCenterWrapper.h"

 extern int gc_initialize_ffi(void);
 extern int gc_authenticate_ffi(void);
 extern int gc_report_achievement_ffi(const char* identifier);
 extern int gc_submit_score_ffi(const char* leaderboard_id, int64_t score);
 extern int gc_show_game_center_ffi(void);
 extern int gc_show_leaderboard_ffi(const char* leaderboard_id);
 extern int gc_show_achievements_ffi(void);
 ```
 */
