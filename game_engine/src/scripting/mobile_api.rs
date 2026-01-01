//! 移动平台服务脚本API
//!
//! 提供Google Play Games、Game Center、推送通知等功能的脚本接口

use crate::scripting::{api::ScriptApi, system::ScriptValue, ScriptResult};
use crate::platform::mobile::{
    GooglePlayGames, GameCenter, PushNotificationService, Notification,
};
use std::sync::{Arc, Mutex};

/// 移动平台脚本API
pub struct MobileScriptApi {
    /// Google Play Games服务（Android）
    google_play_games: Arc<Mutex<GooglePlayGames>>,
    /// Game Center服务（iOS）
    game_center: Arc<Mutex<GameCenter>>,
    /// 推送通知服务
    push_notifications: Arc<Mutex<PushNotificationService>>,
}

impl MobileScriptApi {
    /// 创建新的移动平台脚本API
    pub fn new() -> Self {
        Self {
            google_play_games: Arc::new(Mutex::new(GooglePlayGames::new())),
            game_center: Arc::new(Mutex::new(GameCenter::new())),
            push_notifications: Arc::new(Mutex::new(PushNotificationService::new(
                crate::platform::mobile::NotificationPlatform::Firebase,
            ))),
        }
    }

    /// 注册所有移动平台API到脚本系统
    pub fn register_api(&self, api: &mut ScriptApi) {
        // ========== Google Play Games API ==========
        self.register_google_play_games_api(api);

        // ========== Game Center API ==========
        self.register_game_center_api(api);

        // ========== 推送通知API ==========
        self.register_push_notification_api(api);
    }

    /// 注册Google Play Games API
    fn register_google_play_games_api(&self, api: &mut ScriptApi) {
        let gpg = self.google_play_games.clone();

        // 初始化Google Play Games
        api.register_function("gpg_initialize", move |args| {
            let mut gpg_guard = match gpg.lock() {
                Ok(guard) => guard,
                Err(e) => return ScriptResult::Error(format!("Failed to acquire lock: {}", e)),
            };

            match gpg_guard.initialize() {
                Ok(()) => ScriptResult::Success(ScriptValue::String(
                    "Google Play Games initialized".to_string(),
                )),
                Err(e) => ScriptResult::Error(format!("Initialization failed: {}", e)),
            }
        });

        // 登录
        let gpg = self.google_play_games.clone();
        api.register_function("gpg_sign_in", move |args| {
            let mut gpg_guard = match gpg.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            match gpg_guard.sign_in() {
                Ok(()) => ScriptResult::Success(ScriptValue::Boolean(true)),
                Err(e) => {
                    tracing::error!("Sign in failed: {}", e);
                    ScriptResult::Error(format!("Sign in failed: {}", e))
                }
            }
        });

        // 登出
        let gpg = self.google_play_games.clone();
        api.register_function("gpg_sign_out", move |args| {
            let mut gpg_guard = match gpg.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            gpg_guard.sign_out();
            ScriptResult::Success(ScriptValue::String("Signed out".to_string()))
        });

        // 检查登录状态
        let gpg = self.google_play_games.clone();
        api.register_function("gpg_is_signed_in", move |args| {
            let gpg_guard = match gpg.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            ScriptResult::Success(ScriptValue::Boolean(gpg_guard.is_signed_in()))
        });

        // 获取当前玩家信息
        let gpg = self.google_play_games.clone();
        api.register_function("gpg_get_player", move |args| {
            let gpg_guard = match gpg.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            if let Some(player) = gpg_guard.get_current_player() {
                // 返回玩家信息作为对象
                let result = format!(
                    r#"{{"id": "{}", "name": "{}", "level": {}}}"#,
                    player.id, player.name, player.level
                );
                ScriptResult::Success(ScriptValue::String(result))
            } else {
                ScriptResult::Success(ScriptValue::String("null".to_string()))
            }
        });

        // 解锁成就
        let gpg = self.google_play_games.clone();
        api.register_function("gpg_unlock_achievement", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("gpg_unlock_achievement() requires achievement_id".to_string());
            }

            let achievement_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("achievement_id must be a string".to_string()),
            };

            let mut gpg_guard = match gpg.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            match gpg_guard.unlock_achievement(achievement_id) {
                Ok(()) => ScriptResult::Success(ScriptValue::String("Achievement unlocked".to_string())),
                Err(e) => ScriptResult::Error(format!("Failed to unlock achievement: {}", e)),
            }
        });

        // 更新成就进度
        let gpg = self.google_play_games.clone();
        api.register_function("gpg_set_achievement_progress", move |args| {
            if args.len() < 2 {
                return ScriptResult::Error(
                    "gpg_set_achievement_progress() requires achievement_id and progress".to_string(),
                );
            }

            let achievement_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("achievement_id must be a string".to_string()),
            };

            let progress = match args[1].as_number() {
                Some(n) => n as u32,
                None => return ScriptResult::Error("progress must be a number".to_string()),
            };

            let mut gpg_guard = match gpg.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            match gpg_guard.update_achievement_progress(achievement_id, progress) {
                Ok(()) => ScriptResult::Success(ScriptValue::String("Progress updated".to_string())),
                Err(e) => ScriptResult::Error(format!("Failed to update progress: {}", e)),
            }
        });

        // 提交分数
        let gpg = self.google_play_games.clone();
        api.register_function("gpg_submit_score", move |args| {
            if args.len() < 2 {
                return ScriptResult::Error("gpg_submit_score() requires leaderboard_id and score".to_string());
            }

            let leaderboard_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("leaderboard_id must be a string".to_string()),
            };

            let score = match args[1].as_number() {
                Some(n) => n as i64,
                None => return ScriptResult::Error("score must be a number".to_string()),
            };

            let mut gpg_guard = match gpg.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            match gpg_guard.submit_score(leaderboard_id, score) {
                Ok(()) => ScriptResult::Success(ScriptValue::String("Score submitted".to_string())),
                Err(e) => ScriptResult::Error(format!("Failed to submit score: {}", e)),
            }
        });

        // 显示排行榜
        let gpg = self.google_play_games.clone();
        api.register_function("gpg_show_leaderboard", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("gpg_show_leaderboard() requires leaderboard_id".to_string());
            }

            let leaderboard_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("leaderboard_id must be a string".to_string()),
            };

            let gpg_guard = match gpg.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            match gpg_guard.show_leaderboard(leaderboard_id) {
                Ok(()) => ScriptResult::Success(ScriptValue::String("Leaderboard shown".to_string())),
                Err(e) => ScriptResult::Error(format!("Failed to show leaderboard: {}", e)),
            }
        });

        // 显示成就
        let gpg = self.google_play_games.clone();
        api.register_function("gpg_show_achievements", move |args| {
            let gpg_guard = match gpg.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            match gpg_guard.show_achievements() {
                Ok(()) => ScriptResult::Success(ScriptValue::String("Achievements shown".to_string())),
                Err(e) => ScriptResult::Error(format!("Failed to show achievements: {}", e)),
            }
        });
    }

    /// 注册Game Center API
    fn register_game_center_api(&self, api: &mut ScriptApi) {
        let gc = self.game_center.clone();

        // 初始化Game Center
        api.register_function("gc_initialize", move |args| {
            let mut gc_guard = match gc.lock() {
                Ok(guard) => guard,
                Err(e) => return ScriptResult::Error(format!("Failed to acquire lock: {}", e)),
            };

            match gc_guard.initialize() {
                Ok(()) => ScriptResult::Success(ScriptValue::String("Game Center initialized".to_string())),
                Err(e) => ScriptResult::Error(format!("Initialization failed: {}", e)),
            }
        });

        // 认证
        let gc = self.game_center.clone();
        api.register_function("gc_authenticate", move |args| {
            let mut gc_guard = match gc.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            match gc_guard.authenticate() {
                Ok(()) => ScriptResult::Success(ScriptValue::Boolean(true)),
                Err(e) => ScriptResult::Error(format!("Authentication failed: {}", e)),
            }
        });

        // 报告成就
        let gc = self.game_center.clone();
        api.register_function("gc_report_achievement", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("gc_report_achievement() requires achievement_id".to_string());
            }

            let achievement_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("achievement_id must be a string".to_string()),
            };

            let mut gc_guard = match gc.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            match gc_guard.report_achievement(achievement_id) {
                Ok(()) => ScriptResult::Success(ScriptValue::String("Achievement reported".to_string())),
                Err(e) => ScriptResult::Error(format!("Failed to report achievement: {}", e)),
            }
        });

        // 提交分数
        let gc = self.game_center.clone();
        api.register_function("gc_submit_score", move |args| {
            if args.len() < 2 {
                return ScriptResult::Error("gc_submit_score() requires leaderboard_id and score".to_string());
            }

            let leaderboard_id = match &args[0] {
                ScriptValue::String(id) => id.clone(),
                _ => return ScriptResult::Error("leaderboard_id must be a string".to_string()),
            };

            let score = match args[1].as_number() {
                Some(n) => n as i64,
                None => return ScriptResult::Error("score must be a number".to_string()),
            };

            let mut gc_guard = match gc.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            match gc_guard.submit_score(leaderboard_id, score) {
                Ok(()) => ScriptResult::Success(ScriptValue::String("Score submitted".to_string())),
                Err(e) => ScriptResult::Error(format!("Failed to submit score: {}", e)),
            }
        });

        // 显示Game Center仪表板
        let gc = self.game_center.clone();
        api.register_function("gc_show_game_center", move |args| {
            let gc_guard = match gc.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            match gc_guard.show_game_center() {
                Ok(()) => ScriptResult::Success(ScriptValue::String("Game Center shown".to_string())),
                Err(e) => ScriptResult::Error(format!("Failed to show Game Center: {}", e)),
            }
        });
    }

    /// 注册推送通知API
    fn register_push_notification_api(&self, api: &mut ScriptApi) {
        let pn = self.push_notifications.clone();

        // 初始化推送通知
        api.register_function("push_initialize", move |args| {
            let mut pn_guard = match pn.lock() {
                Ok(guard) => guard,
                Err(e) => return ScriptResult::Error(format!("Failed to acquire lock: {}", e)),
            };

            match pn_guard.initialize() {
                Ok(()) => ScriptResult::Success(ScriptValue::String("Push notifications initialized".to_string())),
                Err(e) => ScriptResult::Error(format!("Initialization failed: {}", e)),
            }
        });

        // 请求通知权限
        let pn = self.push_notifications.clone();
        api.register_function("push_request_permission", move |args| {
            let mut pn_guard = match pn.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            match pn_guard.request_permission() {
                Ok(granted) => ScriptResult::Success(ScriptValue::Boolean(granted)),
                Err(e) => ScriptResult::Error(format!("Failed to request permission: {}", e)),
            }
        });

        // 发送本地通知
        let pn = self.push_notifications.clone();
        api.register_function("push_send_local", move |args| {
            if args.len() < 2 {
                return ScriptResult::Error("push_send_local() requires title and body".to_string());
            }

            let title = match &args[0] {
                ScriptValue::String(s) => s.clone(),
                _ => return ScriptResult::Error("title must be a string".to_string()),
            };

            let body = match &args[1] {
                ScriptValue::String(s) => s.clone(),
                _ => return ScriptResult::Error("body must be a string".to_string()),
            };

            let notification = Notification::new(title, body);

            let pn_guard = match pn.lock() {
                Ok(guard) => guard,
                Err(_) => return ScriptResult::Error("Failed to acquire lock".to_string()),
            };

            match pn_guard.send_local_notification(notification) {
                Ok(()) => ScriptResult::Success(ScriptValue::String("Notification sent".to_string())),
                Err(e) => ScriptResult::Error(format!("Failed to send notification: {}", e)),
            }
        });
    }
}

impl Default for MobileScriptApi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_api_creation() {
        let api = MobileScriptApi::new();
        // Should not panic
    }

    #[test]
    fn test_google_play_games_mock() {
        let gpg = GooglePlayGames::new();
        assert!(!gpg.is_signed_in());
    }
}
