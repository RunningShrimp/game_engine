//! # Console Achievement System
//!
//! Cross-platform achievement system for consoles.

use super::ConsolePlatform;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Achievement status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementStatus {
    Locked,
    InProgress,
    Unlocked,
}

/// Trophy type (PlayStation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrophyType {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

/// Achievement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub hidden: bool,
    pub progress: f32,
    pub required_progress: f32,
    pub status: AchievementStatus,
    pub unlocked_at: Option<std::time::SystemTime>,
    pub gamerscore: u32,                 // Xbox
    pub trophy_type: Option<TrophyType>, // PlayStation
}

/// Achievement statistics
#[derive(Debug, Clone, Copy)]
pub struct AchievementStats {
    pub total_count: usize,
    pub unlocked_count: usize,
    pub in_progress_count: usize,
    pub completion_percentage: f32,
    pub total_gamerscore: u32,
    pub earned_gamerscore: u32,
}

/// Achievement system
pub struct AchievementSystem {
    platform: ConsolePlatform,
    achievements: HashMap<String, Achievement>,
}

impl AchievementSystem {
    pub fn new(platform: ConsolePlatform) -> Self {
        Self {
            platform,
            achievements: HashMap::new(),
        }
    }

    /// Register an achievement
    pub fn register_achievement(&mut self, achievement: Achievement) {
        self.achievements.insert(achievement.id.clone(), achievement);
    }

    /// Update achievement progress
    pub fn update_progress(
        &mut self,
        achievement_id: &str,
        progress: f32,
    ) -> Result<(), AchievementError> {
        let achievement =
            self.achievements.get_mut(achievement_id).ok_or(AchievementError::NotFound)?;

        if achievement.status == AchievementStatus::Unlocked {
            return Ok(()); // Already unlocked
        }

        achievement.progress = progress.min(achievement.required_progress);

        if achievement.progress >= achievement.required_progress {
            achievement.status = AchievementStatus::Unlocked;
            achievement.unlocked_at = Some(std::time::SystemTime::now());
            return self.unlock_achievement(achievement_id);
        } else {
            achievement.status = AchievementStatus::InProgress;
        }

        Ok(())
    }

    /// Unlock an achievement
    pub fn unlock_achievement(&mut self, achievement_id: &str) -> Result<(), AchievementError> {
        let achievement =
            self.achievements.get_mut(achievement_id).ok_or(AchievementError::NotFound)?;

        if achievement.status == AchievementStatus::Unlocked {
            return Ok(()); // Already unlocked
        }

        achievement.status = AchievementStatus::Unlocked;
        achievement.unlocked_at = Some(std::time::SystemTime::now());
        achievement.progress = achievement.required_progress;

        // TODO: Call platform-specific achievement unlock API
        match self.platform {
            ConsolePlatform::NintendoSwitch => {
                // TODO: Call Nintendo achievement API
            }
            ConsolePlatform::PlayStation5 | ConsolePlatform::PlayStation4 => {
                // TODO: Call PlayStation trophy API
            }
            ConsolePlatform::XboxSeries | ConsolePlatform::XboxOne => {
                // TODO: Call Xbox achievement API
            }
        }

        tracing::info!("Achievement unlocked: {}", achievement_id);

        Ok(())
    }

    /// Get achievement
    pub fn get_achievement(&self, achievement_id: &str) -> Option<&Achievement> {
        self.achievements.get(achievement_id)
    }

    /// Get all achievements
    pub fn get_all_achievements(&self) -> Vec<&Achievement> {
        self.achievements.values().collect()
    }

    /// Get achievement statistics
    pub fn get_stats(&self) -> AchievementStats {
        let total_count = self.achievements.len();
        let unlocked_count = self
            .achievements
            .values()
            .filter(|a| a.status == AchievementStatus::Unlocked)
            .count();
        let in_progress_count = self
            .achievements
            .values()
            .filter(|a| a.status == AchievementStatus::InProgress)
            .count();

        let completion_percentage = if total_count > 0 {
            (unlocked_count as f32 / total_count as f32) * 100.0
        } else {
            0.0
        };

        let total_gamerscore = self.achievements.values().map(|a| a.gamerscore).sum();
        let earned_gamerscore: u32 = self
            .achievements
            .values()
            .filter(|a| a.status == AchievementStatus::Unlocked)
            .map(|a| a.gamerscore)
            .sum();

        AchievementStats {
            total_count,
            unlocked_count,
            in_progress_count,
            completion_percentage,
            total_gamerscore,
            earned_gamerscore,
        }
    }

    /// Reset all achievements (for testing)
    pub fn reset_all(&mut self) {
        for achievement in self.achievements.values_mut() {
            achievement.status = AchievementStatus::Locked;
            achievement.progress = 0.0;
            achievement.unlocked_at = None;
        }
    }
}

/// Achievement errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AchievementError {
    NotFound,
    AlreadyUnlocked,
    PlatformError(String),
}

impl std::fmt::Display for AchievementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AchievementError::NotFound => write!(f, "Achievement not found"),
            AchievementError::AlreadyUnlocked => write!(f, "Achievement already unlocked"),
            AchievementError::PlatformError(msg) => write!(f, "Platform error: {}", msg),
        }
    }
}

impl std::error::Error for AchievementError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_registration() {
        let mut system = AchievementSystem::new(ConsolePlatform::PlayStation5);

        let achievement = Achievement {
            id: "test_achievement".to_string(),
            name: "Test Achievement".to_string(),
            description: "Test description".to_string(),
            hidden: false,
            progress: 0.0,
            required_progress: 1.0,
            status: AchievementStatus::Locked,
            unlocked_at: None,
            gamerscore: 10,
            trophy_type: Some(TrophyType::Bronze),
        };

        system.register_achievement(achievement);

        assert!(system.get_achievement("test_achievement").is_some());
    }

    #[test]
    fn test_achievement_unlock() {
        let mut system = AchievementSystem::new(ConsolePlatform::PlayStation5);

        let achievement = Achievement {
            id: "test_achievement".to_string(),
            name: "Test Achievement".to_string(),
            description: "Test description".to_string(),
            hidden: false,
            progress: 0.0,
            required_progress: 1.0,
            status: AchievementStatus::Locked,
            unlocked_at: None,
            gamerscore: 10,
            trophy_type: Some(TrophyType::Bronze),
        };

        system.register_achievement(achievement);
        system.unlock_achievement("test_achievement").unwrap();

        let achievement = system.get_achievement("test_achievement").unwrap();
        assert_eq!(achievement.status, AchievementStatus::Unlocked);
        assert!(achievement.unlocked_at.is_some());
    }

    #[test]
    fn test_achievement_progress() {
        let mut system = AchievementSystem::new(ConsolePlatform::PlayStation5);

        let achievement = Achievement {
            id: "test_achievement".to_string(),
            name: "Test Achievement".to_string(),
            description: "Test description".to_string(),
            hidden: false,
            progress: 0.0,
            required_progress: 10.0,
            status: AchievementStatus::Locked,
            unlocked_at: None,
            gamerscore: 10,
            trophy_type: Some(TrophyType::Bronze),
        };

        system.register_achievement(achievement);
        system.update_progress("test_achievement", 5.0).unwrap();

        let achievement = system.get_achievement("test_achievement").unwrap();
        assert_eq!(achievement.progress, 5.0);
        assert_eq!(achievement.status, AchievementStatus::InProgress);

        system.update_progress("test_achievement", 10.0).unwrap();

        let achievement = system.get_achievement("test_achievement").unwrap();
        assert_eq!(achievement.status, AchievementStatus::Unlocked);
    }

    #[test]
    fn test_achievement_stats() {
        let mut system = AchievementSystem::new(ConsolePlatform::PlayStation5);

        for i in 0..10 {
            let achievement = Achievement {
                id: format!("achievement_{}", i),
                name: format!("Achievement {}", i),
                description: "Test".to_string(),
                hidden: false,
                progress: 0.0,
                required_progress: 1.0,
                status: AchievementStatus::Locked,
                unlocked_at: None,
                gamerscore: 10,
                trophy_type: Some(TrophyType::Bronze),
            };
            system.register_achievement(achievement);
        }

        // Unlock 5 achievements
        for i in 0..5 {
            system.unlock_achievement(&format!("achievement_{}", i)).unwrap();
        }

        let stats = system.get_stats();
        assert_eq!(stats.total_count, 10);
        assert_eq!(stats.unlocked_count, 5);
        assert_eq!(stats.completion_percentage, 50.0);
        assert_eq!(stats.total_gamerscore, 100);
        assert_eq!(stats.earned_gamerscore, 50);
    }
}
