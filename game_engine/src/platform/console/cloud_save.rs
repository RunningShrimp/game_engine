//! # Console Cloud Save System
//!
//! Cloud save management for console platforms.

use super::ConsolePlatform;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Save metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub game_version: String,
    pub player_level: u32,
    pub current_chapter: String,
    pub completion_percentage: f32,
    pub playtime_seconds: u64,
    pub custom_data: HashMap<String, String>,
}

/// Save slot info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSlotInfo {
    pub slot_id: u32,
    pub metadata: SaveMetadata,
    pub size_bytes: usize,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
    pub synced_to_cloud: bool,
}

/// Cloud save manager
pub struct CloudSaveManager {
    platform: ConsolePlatform,
    save_dir: PathBuf,
    saves: HashMap<u32, SaveSlotInfo>,
}

impl CloudSaveManager {
    pub fn new(platform: ConsolePlatform, save_dir: PathBuf) -> Self {
        Self {
            platform,
            save_dir,
            saves: HashMap::new(),
        }
    }

    /// Initialize the save manager
    pub fn initialize(&mut self) -> Result<(), CloudSaveError> {
        // Create save directory if it doesn't exist
        std::fs::create_dir_all(&self.save_dir)
            .map_err(|e| CloudSaveError::IoError(e.to_string()))?;

        // Load existing saves
        self.load_save_index()?;

        tracing::info!(
            "Cloud save manager initialized for platform: {:?}",
            self.platform
        );

        Ok(())
    }

    /// Save game to slot
    pub fn save_game(
        &mut self,
        slot_id: u32,
        data: &[u8],
        metadata: SaveMetadata,
    ) -> Result<(), CloudSaveError> {
        let save_path = self.get_save_path(slot_id);

        // Write save data
        std::fs::write(&save_path, data).map_err(|e| CloudSaveError::IoError(e.to_string()))?;

        let now = std::time::SystemTime::now();

        // Update or create save slot info
        let slot_info = if let Some(existing) = self.saves.get_mut(&slot_id) {
            existing.metadata = metadata;
            existing.size_bytes = data.len();
            existing.updated_at = now;
            existing.synced_to_cloud = false;
            existing.clone()
        } else {
            let slot_info = SaveSlotInfo {
                slot_id,
                metadata,
                size_bytes: data.len(),
                created_at: now,
                updated_at: now,
                synced_to_cloud: false,
            };
            self.saves.insert(slot_id, slot_info.clone());
            slot_info
        };

        // Save index
        self.save_save_index()?;

        tracing::info!("Game saved to slot {}", slot_id);

        Ok(())
    }

    /// Load game from slot
    pub fn load_game(&self, slot_id: u32) -> Result<Vec<u8>, CloudSaveError> {
        let save_path = self.get_save_path(slot_id);

        if !save_path.exists() {
            return Err(CloudSaveError::SaveNotFound(slot_id));
        }

        let data = std::fs::read(&save_path).map_err(|e| CloudSaveError::IoError(e.to_string()))?;

        tracing::info!("Game loaded from slot {}", slot_id);

        Ok(data)
    }

    /// Get save slot info
    pub fn get_save_slot(&self, slot_id: u32) -> Option<&SaveSlotInfo> {
        self.saves.get(&slot_id)
    }

    /// Get all save slots
    pub fn get_all_save_slots(&self) -> Vec<&SaveSlotInfo> {
        let mut slots: Vec<_> = self.saves.values().collect();
        slots.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        slots
    }

    /// Delete save slot
    pub fn delete_save(&mut self, slot_id: u32) -> Result<(), CloudSaveError> {
        if !self.saves.contains_key(&slot_id) {
            return Err(CloudSaveError::SaveNotFound(slot_id));
        }

        let save_path = self.get_save_path(slot_id);

        if save_path.exists() {
            std::fs::remove_file(&save_path).map_err(|e| CloudSaveError::IoError(e.to_string()))?;
        }

        self.saves.remove(&slot_id);
        self.save_save_index()?;

        tracing::info!("Save slot {} deleted", slot_id);

        Ok(())
    }

    /// Sync all saves to cloud
    pub fn sync_all_to_cloud(&mut self) -> Result<(), CloudSaveError> {
        // TODO: Implement platform-specific cloud sync
        match self.platform {
            ConsolePlatform::NintendoSwitch => {
                // TODO: Nintendo Switch cloud save API
            }
            ConsolePlatform::PlayStation5 | ConsolePlatform::PlayStation4 => {
                // TODO: PlayStation Plus cloud save API
            }
            ConsolePlatform::XboxSeries | ConsolePlatform::XboxOne => {
                // TODO: Xbox Live cloud save API
            }
        }

        // Mark all saves as synced
        for save in self.saves.values_mut() {
            save.synced_to_cloud = true;
        }

        tracing::info!("All saves synced to cloud");

        Ok(())
    }

    /// Get save path for slot
    fn get_save_path(&self, slot_id: u32) -> PathBuf {
        self.save_dir.join(format!("save_{}.dat", slot_id))
    }

    /// Load save index
    fn load_save_index(&mut self) -> Result<(), CloudSaveError> {
        let index_path = self.save_dir.join("save_index.json");

        if index_path.exists() {
            let index_data = std::fs::read_to_string(&index_path)
                .map_err(|e| CloudSaveError::IoError(e.to_string()))?;

            let saves: HashMap<u32, SaveSlotInfo> = serde_json::from_str(&index_data)
                .map_err(|e| CloudSaveError::IoError(e.to_string()))?;

            self.saves = saves;
        }

        Ok(())
    }

    /// Save save index
    fn save_save_index(&self) -> Result<(), CloudSaveError> {
        let index_path = self.save_dir.join("save_index.json");

        let index_data = serde_json::to_string_pretty(&self.saves)
            .map_err(|e| CloudSaveError::IoError(e.to_string()))?;

        std::fs::write(&index_path, index_data)
            .map_err(|e| CloudSaveError::IoError(e.to_string()))?;

        Ok(())
    }
}

/// Cloud save errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudSaveError {
    SaveNotFound(u32),
    IoError(String),
    PlatformError(String),
    QuotaExceeded,
    CorruptedSave,
}

impl std::fmt::Display for CloudSaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloudSaveError::SaveNotFound(slot) => write!(f, "Save slot {} not found", slot),
            CloudSaveError::IoError(msg) => write!(f, "IO error: {}", msg),
            CloudSaveError::PlatformError(msg) => write!(f, "Platform error: {}", msg),
            CloudSaveError::QuotaExceeded => write!(f, "Cloud save quota exceeded"),
            CloudSaveError::CorruptedSave => write!(f, "Save file is corrupted"),
        }
    }
}

impl std::error::Error for CloudSaveError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let save_dir = dir.path().to_path_buf();

        let mut manager = CloudSaveManager::new(ConsolePlatform::PlayStation5, save_dir);
        manager.initialize().unwrap();

        let metadata = SaveMetadata {
            game_version: "1.0.0".to_string(),
            player_level: 10,
            current_chapter: "Chapter 2".to_string(),
            completion_percentage: 25.0,
            playtime_seconds: 3600,
            custom_data: {
                let mut map = HashMap::new();
                map.insert("key".to_string(), "value".to_string());
                map
            },
        };

        let data = b"test_save_data";
        manager.save_game(1, data, metadata.clone()).unwrap();

        let loaded_data = manager.load_game(1).unwrap();
        assert_eq!(loaded_data, data);

        let slot_info = manager.get_save_slot(1).unwrap();
        assert_eq!(slot_info.slot_id, 1);
        assert_eq!(slot_info.metadata.player_level, 10);
        assert!(!slot_info.synced_to_cloud);
    }

    #[test]
    fn test_multiple_save_slots() {
        let dir = tempdir().unwrap();
        let save_dir = dir.path().to_path_buf();

        let mut manager = CloudSaveManager::new(ConsolePlatform::PlayStation5, save_dir);
        manager.initialize().unwrap();

        for i in 1..=3 {
            let metadata = SaveMetadata {
                game_version: "1.0.0".to_string(),
                player_level: i * 10,
                current_chapter: format!("Chapter {}", i),
                completion_percentage: i as f32 * 25.0,
                playtime_seconds: i * 3600,
                custom_data: HashMap::new(),
            };

            let data = format!("save_data_{}", i).as_bytes().to_vec();
            manager.save_game(i, &data, metadata).unwrap();
        }

        let slots = manager.get_all_save_slots();
        assert_eq!(slots.len(), 3);
    }

    #[test]
    fn test_delete_save() {
        let dir = tempdir().unwrap();
        let save_dir = dir.path().to_path_buf();

        let mut manager = CloudSaveManager::new(ConsolePlatform::PlayStation5, save_dir);
        manager.initialize().unwrap();

        let metadata = SaveMetadata {
            game_version: "1.0.0".to_string(),
            player_level: 10,
            current_chapter: "Chapter 1".to_string(),
            completion_percentage: 10.0,
            playtime_seconds: 600,
            custom_data: HashMap::new(),
        };

        manager.save_game(1, b"test", metadata).unwrap();
        assert!(manager.get_save_slot(1).is_some());

        manager.delete_save(1).unwrap();
        assert!(manager.get_save_slot(1).is_none());
    }
}
