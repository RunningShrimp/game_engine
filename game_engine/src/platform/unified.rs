//! # Unified Platform Services API
//!
//! Provides a unified, cross-platform API for platform-specific services with automatic fallback.

use super::console::ConsolePlatform;
use super::detection::{PlatformInfo, is_console, is_mobile};
use super::mobile::ProductInfo as MobileProductInfo;
use super::mobile::services::{
    Achievement as MobileAchievement, GameCenter, GooglePlayGames, InAppPurchaseService,
    Notification as MobileNotification, NotificationPlatform, PlayerInfo as MobilePlayerInfo,
    PushNotificationService, ServiceError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unified platform capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    /// Supports achievements/trophies
    pub supports_achievements: bool,
    /// Supports cloud saves
    pub supports_cloud_saves: bool,
    /// Supports in-app purchases
    pub supports_iap: bool,
    /// Supports push notifications
    pub supports_push_notifications: bool,
    /// Supports leaderboards
    pub supports_leaderboards: bool,
    /// Supports multiplayer
    pub supports_multiplayer: bool,
    /// Supports controller vibration
    pub supports_vibration: bool,
    /// Supports motion controls
    pub supports_motion_controls: bool,
    /// Supports touch input
    pub supports_touch: bool,
    /// Supports keyboard/mouse
    pub supports_keyboard_mouse: bool,
    /// Supports HDR
    pub supports_hdr: bool,
    /// Supports ray tracing
    pub supports_ray_tracing: bool,
}

impl PlatformCapabilities {
    /// Get capabilities for current platform
    pub fn current() -> Self {
        let info = PlatformInfo::current();

        Self {
            supports_achievements: true, // Most platforms support achievements
            supports_cloud_saves: !info.is_web,
            supports_iap: info.is_mobile || info.is_console,
            supports_push_notifications: info.is_mobile,
            supports_leaderboards: !info.is_web,
            supports_multiplayer: !info.is_web,
            supports_vibration: info.is_console || info.is_mobile,
            supports_motion_controls: info.is_mobile || info.is_console,
            supports_touch: info.is_mobile || info.is_web,
            supports_keyboard_mouse: info.is_desktop,
            supports_hdr: info.is_console,
            supports_ray_tracing: info.is_console,
        }
    }

    /// Get capabilities for console platform
    pub fn for_console(platform: ConsolePlatform) -> Self {
        Self {
            supports_achievements: true,
            supports_cloud_saves: true,
            supports_iap: true,
            supports_push_notifications: false,
            supports_leaderboards: true,
            supports_multiplayer: true,
            supports_vibration: true,
            supports_motion_controls: matches!(
                platform,
                ConsolePlatform::NintendoSwitch
                    | ConsolePlatform::PlayStation5
                    | ConsolePlatform::PlayStation4
            ),
            supports_touch: false,
            supports_keyboard_mouse: false,
            supports_hdr: platform.supports_hdr(),
            supports_ray_tracing: platform.supports_ray_tracing(),
        }
    }
}

/// Unified player info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedPlayerInfo {
    pub id: String,
    pub name: String,
    pub level: u32,
    pub avatar_url: Option<String>,
}

/// Unified achievement info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedAchievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub hidden: bool,
    pub unlocked: bool,
    pub progress: f32,
    pub unlocked_at: Option<std::time::SystemTime>,
}

/// Unified leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedLeaderboardEntry {
    pub player_id: String,
    pub player_name: String,
    pub score: i64,
    pub rank: u32,
}

/// Unified notification
#[derive(Debug, Clone)]
pub struct UnifiedNotification {
    pub title: String,
    pub body: String,
    pub data: HashMap<String, String>,
}

/// Unified product info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedProduct {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: String,
    pub localized_price: String,
}

/// Unified platform service interface
pub struct UnifiedPlatformService {
    capabilities: PlatformCapabilities,
    game_center: Option<GameCenter>,
    google_play_games: Option<GooglePlayGames>,
    iap_service: Option<InAppPurchaseService>,
    push_service: Option<PushNotificationService>,
}

impl UnifiedPlatformService {
    /// Create unified platform service for current platform
    pub fn new() -> Self {
        let info = PlatformInfo::current();
        let capabilities = PlatformCapabilities::current();

        let game_center = if info.os == "ios" {
            Some(GameCenter::new())
        } else {
            None
        };

        let google_play_games = if info.os == "android" {
            Some(GooglePlayGames::new())
        } else {
            None
        };

        let iap_service = if info.is_mobile || info.is_console {
            Some(InAppPurchaseService::new())
        } else {
            None
        };

        let push_service = if info.is_mobile {
            Some(PushNotificationService::new(if info.os == "android" {
                NotificationPlatform::Firebase
            } else {
                NotificationPlatform::APNs
            }))
        } else {
            None
        };

        Self {
            capabilities,
            game_center,
            google_play_games,
            iap_service,
            push_service,
        }
    }

    /// Get platform capabilities
    pub fn capabilities(&self) -> PlatformCapabilities {
        self.capabilities
    }

    /// Initialize all platform services
    pub async fn initialize_all(&mut self) -> Result<(), PlatformServiceError> {
        if let Some(gc) = &mut self.game_center {
            gc.initialize()?;
        }

        if let Some(gpg) = &mut self.google_play_games {
            gpg.initialize()?;
        }

        if let Some(iap) = &mut self.iap_service {
            iap.initialize()?;
        }

        if let Some(push) = &mut self.push_service {
            push.initialize()?;
        }

        tracing::info!("Unified platform service initialized");
        Ok(())
    }

    /// Authenticate with platform services (Game Center / Google Play Games)
    pub fn authenticate(&mut self) -> Result<Option<UnifiedPlayerInfo>, PlatformServiceError> {
        if let Some(gc) = &mut self.game_center {
            gc.authenticate()?;
            if let Some(player) = gc.get_current_player() {
                return Ok(Some(UnifiedPlayerInfo {
                    id: player.id.clone(),
                    name: player.name.clone(),
                    level: player.level,
                    avatar_url: None,
                }));
            }
        }

        if let Some(gpg) = &mut self.google_play_games {
            gpg.sign_in()?;
            if let Some(player) = gpg.get_current_player() {
                return Ok(Some(UnifiedPlayerInfo {
                    id: player.id.clone(),
                    name: player.name.clone(),
                    level: player.level,
                    avatar_url: player.avatar_url.clone(),
                }));
            }
        }

        if !self.capabilities.supports_achievements {
            return Ok(None); // Platform doesn't support authentication
        }

        Err(PlatformServiceError::NotAuthenticated)
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.game_center.as_ref().map(|gc| gc.is_authenticated()).unwrap_or(false)
            || self.google_play_games.as_ref().map(|gpg| gpg.is_signed_in()).unwrap_or(false)
    }

    /// Get current player info
    pub fn get_player_info(&self) -> Option<UnifiedPlayerInfo> {
        self.game_center
            .as_ref()
            .and_then(|gc| gc.get_current_player())
            .map(|p| UnifiedPlayerInfo {
                id: p.id.clone(),
                name: p.name.clone(),
                level: p.level,
                avatar_url: None,
            })
            .or_else(|| {
                self.google_play_games
                    .as_ref()
                    .and_then(|gpg| gpg.get_current_player())
                    .map(|p| UnifiedPlayerInfo {
                        id: p.id.clone(),
                        name: p.name.clone(),
                        level: p.level,
                        avatar_url: p.avatar_url.clone(),
                    })
            })
    }

    /// Submit score to leaderboard
    pub fn submit_score(
        &mut self,
        leaderboard_id: String,
        score: i64,
    ) -> Result<(), PlatformServiceError> {
        if !self.capabilities.supports_leaderboards {
            return Err(PlatformServiceError::NotSupported);
        }

        if let Some(gc) = &mut self.game_center {
            gc.submit_score(leaderboard_id, score)?;
            return Ok(());
        }

        if let Some(gpg) = &mut self.google_play_games {
            gpg.submit_score(leaderboard_id, score)?;
            return Ok(());
        }

        Err(PlatformServiceError::NotSupported)
    }

    /// Unlock achievement
    pub fn unlock_achievement(
        &mut self,
        achievement_id: String,
    ) -> Result<(), PlatformServiceError> {
        if !self.capabilities.supports_achievements {
            return Err(PlatformServiceError::NotSupported);
        }

        if let Some(gc) = &mut self.game_center {
            gc.report_achievement(achievement_id)?;
            return Ok(());
        }

        if let Some(gpg) = &mut self.google_play_games {
            gpg.unlock_achievement(achievement_id)?;
            return Ok(());
        }

        Err(PlatformServiceError::NotSupported)
    }

    /// Update achievement progress
    pub fn update_achievement_progress(
        &mut self,
        achievement_id: String,
        progress: f32,
    ) -> Result<(), PlatformServiceError> {
        if !self.capabilities.supports_achievements {
            return Err(PlatformServiceError::NotSupported);
        }

        if let Some(gpg) = &mut self.google_play_games {
            gpg.update_achievement_progress(achievement_id, (progress * 100.0) as u32)?;
            return Ok(());
        }

        // Game Center uses report_achievement with percent_complete
        if let Some(gc) = &mut self.game_center {
            gc.report_achievement(achievement_id)?;
            return Ok(());
        }

        Err(PlatformServiceError::NotSupported)
    }

    /// Show achievements UI
    pub fn show_achievements(&self) -> Result<(), PlatformServiceError> {
        if !self.capabilities.supports_achievements {
            return Err(PlatformServiceError::NotSupported);
        }

        if let Some(gc) = &self.game_center {
            gc.show_game_center()?;
            return Ok(());
        }

        if let Some(gpg) = &self.google_play_games {
            gpg.show_achievements()?;
            return Ok(());
        }

        Err(PlatformServiceError::NotSupported)
    }

    /// Show leaderboard UI
    pub fn show_leaderboard(&self, leaderboard_id: String) -> Result<(), PlatformServiceError> {
        if !self.capabilities.supports_leaderboards {
            return Err(PlatformServiceError::NotSupported);
        }

        if let Some(gpg) = &self.google_play_games {
            gpg.show_leaderboard(leaderboard_id)?;
            return Ok(());
        }

        Err(PlatformServiceError::NotSupported)
    }

    /// Query in-app purchase products
    pub async fn query_iap_products(
        &mut self,
        product_ids: Vec<String>,
    ) -> Result<Vec<UnifiedProduct>, PlatformServiceError> {
        if !self.capabilities.supports_iap {
            return Err(PlatformServiceError::NotSupported);
        }

        if let Some(iap) = &mut self.iap_service {
            let products = iap.query_products(product_ids)?;
            Ok(products
                .into_iter()
                .map(|p| UnifiedProduct {
                    id: p.product_id,
                    name: p.title,
                    description: p.description,
                    price: p.price.clone(),
                    localized_price: p.price,
                })
                .collect())
        } else {
            Err(PlatformServiceError::NotSupported)
        }
    }

    /// Purchase product
    pub async fn purchase_iap(
        &mut self,
        product_id: String,
    ) -> Result<String, PlatformServiceError> {
        if !self.capabilities.supports_iap {
            return Err(PlatformServiceError::NotSupported);
        }

        if let Some(iap) = &self.iap_service {
            let token = iap.purchase(product_id)?;
            Ok(token)
        } else {
            Err(PlatformServiceError::NotSupported)
        }
    }

    /// Consume purchased product (for consumables)
    pub async fn consume_iap(
        &mut self,
        purchase_token: String,
    ) -> Result<(), PlatformServiceError> {
        if !self.capabilities.supports_iap {
            return Err(PlatformServiceError::NotSupported);
        }

        if let Some(iap) = &self.iap_service {
            iap.consume(purchase_token)?;
            Ok(())
        } else {
            Err(PlatformServiceError::NotSupported)
        }
    }

    /// Restore purchases
    pub async fn restore_purchases(&mut self) -> Result<Vec<UnifiedProduct>, PlatformServiceError> {
        if !self.capabilities.supports_iap {
            return Err(PlatformServiceError::NotSupported);
        }

        if let Some(iap) = &self.iap_service {
            let purchases = iap.restore_purchases()?;
            // Convert purchases to unified products
            Ok(purchases
                .into_iter()
                .map(|p| UnifiedProduct {
                    id: p.product_id.clone(),
                    name: p.product_id.clone(),
                    description: String::new(),
                    price: String::new(),
                    localized_price: String::new(),
                })
                .collect())
        } else {
            Err(PlatformServiceError::NotSupported)
        }
    }

    /// Request push notification permission
    pub fn request_push_permission(&mut self) -> Result<bool, PlatformServiceError> {
        if !self.capabilities.supports_push_notifications {
            return Err(PlatformServiceError::NotSupported);
        }

        if let Some(push) = &mut self.push_service {
            let granted = push.request_permission()?;
            Ok(granted)
        } else {
            Err(PlatformServiceError::NotSupported)
        }
    }

    /// Send local push notification
    pub fn send_local_notification(
        &self,
        notification: UnifiedNotification,
    ) -> Result<(), PlatformServiceError> {
        if !self.capabilities.supports_push_notifications {
            return Err(PlatformServiceError::NotSupported);
        }

        if let Some(push) = &self.push_service {
            let mobile_notification =
                MobileNotification::new(notification.title, notification.body);
            push.send_local_notification(mobile_notification)?;
            Ok(())
        } else {
            Err(PlatformServiceError::NotSupported)
        }
    }

    /// Subscribe to push notification topic (Android only)
    pub fn subscribe_to_topic(&self, topic: String) -> Result<(), PlatformServiceError> {
        if !self.capabilities.supports_push_notifications {
            return Err(PlatformServiceError::NotSupported);
        }

        if let Some(push) = &self.push_service {
            push.subscribe_to_topic(topic)?;
            Ok(())
        } else {
            Err(PlatformServiceError::NotSupported)
        }
    }

    /// Unsubscribe from push notification topic
    pub fn unsubscribe_from_topic(&self, topic: String) -> Result<(), PlatformServiceError> {
        if !self.capabilities.supports_push_notifications {
            return Err(PlatformServiceError::NotSupported);
        }

        if let Some(push) = &self.push_service {
            push.unsubscribe_from_topic(topic)?;
            Ok(())
        } else {
            Err(PlatformServiceError::NotSupported)
        }
    }
}

impl Default for UnifiedPlatformService {
    fn default() -> Self {
        Self::new()
    }
}

/// Platform service errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformServiceError {
    NotSupported,
    NotInitialized,
    NotAuthenticated,
    ServiceError(String),
}

impl std::fmt::Display for PlatformServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatformServiceError::NotSupported => {
                write!(f, "Feature not supported on this platform")
            }
            PlatformServiceError::NotInitialized => write!(f, "Platform service not initialized"),
            PlatformServiceError::NotAuthenticated => write!(f, "User not authenticated"),
            PlatformServiceError::ServiceError(msg) => write!(f, "Service error: {}", msg),
        }
    }
}

impl std::error::Error for PlatformServiceError {}

impl From<ServiceError> for PlatformServiceError {
    fn from(err: ServiceError) -> Self {
        PlatformServiceError::ServiceError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_capabilities() {
        let capabilities = PlatformCapabilities::current();
        let info = PlatformInfo::current();

        // Test that capabilities match platform
        if info.is_mobile {
            assert!(capabilities.supports_touch);
            assert!(capabilities.supports_iap);
        }

        if info.is_desktop {
            assert!(capabilities.supports_keyboard_mouse);
        }

        if info.is_console {
            assert!(capabilities.supports_vibration);
        }
    }

    #[test]
    fn test_unified_service_creation() {
        let service = UnifiedPlatformService::new();
        let capabilities = service.capabilities();

        // Verify capabilities are set correctly
        let info = PlatformInfo::current();
        assert_eq!(capabilities.supports_touch, info.is_mobile || info.is_web);
        assert_eq!(capabilities.supports_keyboard_mouse, info.is_desktop);
    }

    #[test]
    fn test_console_capabilities() {
        let ps5_caps = PlatformCapabilities::for_console(ConsolePlatform::PlayStation5);
        assert!(ps5_caps.supports_achievements);
        assert!(ps5_caps.supports_ray_tracing);
        assert!(ps5_caps.supports_hdr);

        let switch_caps = PlatformCapabilities::for_console(ConsolePlatform::NintendoSwitch);
        assert!(switch_caps.supports_achievements);
        assert!(!switch_caps.supports_ray_tracing);
    }
}
