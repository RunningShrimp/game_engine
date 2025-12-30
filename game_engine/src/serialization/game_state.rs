// 游戏状态序列化
//
// 提供完整的游戏状态序列化、保存和加载功能。

use crate::ecs::Time;
use crate::scene::SerializedScene;
use crate::serialization::compat::bincode_compat;
use bevy_ecs::prelude::World;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 游戏状态
///
/// 包含完整的游戏运行时状态，包括场景、资源和全局变量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// 序列化版本
    pub version: u32,
    /// 场景数据
    pub scenes: Vec<SerializedScene>,
    /// 当前活动场景索引
    #[serde(default)]
    pub current_scene_index: Option<usize>,
    /// 全局变量
    #[serde(default)]
    pub global_variables: HashMap<String, String>,
    /// 游戏时间
    #[serde(default)]
    pub game_time: GameTime,
    /// 元数据
    #[serde(default)]
    pub metadata: GameStateMetadata,
}

/// 游戏时间信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameTime {
    /// 总游戏时间（秒）
    #[serde(default)]
    pub total_time: f32,
    /// 游戏帧数
    #[serde(default)]
    pub frame_count: u64,
    /// 时间缩放（用于慢动作或加速）
    #[serde(default = "default_time_scale")]
    pub time_scale: f32,
}

fn default_time_scale() -> f32 {
    1.0
}

/// 游戏状态元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameStateMetadata {
    /// 存档名称
    #[serde(default)]
    pub save_name: String,
    /// 游戏版本
    #[serde(default)]
    pub game_version: String,
    /// 创建时间（Unix timestamp）
    #[serde(default)]
    pub created_at: u64,
    /// 修改时间（Unix timestamp）
    #[serde(default)]
    pub modified_at: u64,
    /// 玩家进度信息
    #[serde(default)]
    pub progress: PlayerProgress,
    /// 截图数据（base64编码，可选）
    #[serde(default)]
    pub screenshot: Option<String>,
}

/// 玩家进度信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerProgress {
    /// 当前关卡
    #[serde(default)]
    pub current_level: String,
    /// 已解锁关卡
    #[serde(default)]
    pub unlocked_levels: Vec<String>,
    /// 得分
    #[serde(default)]
    pub score: u64,
    /// 游戏时长（秒）
    #[serde(default)]
    pub playtime_seconds: u64,
}

/// 序列化格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    /// RON格式（可读性好，适合编辑和调试）
    Ron,
    /// Bincode格式（二进制，体积小，加载快）
    Bincode,
    /// JSON格式（兼容性好，可读）
    Json,
}

impl SerializationFormat {
    /// 根据文件扩展名检测格式
    pub fn from_path(path: &str) -> Self {
        let path = std::path::Path::new(path);
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("ron") => SerializationFormat::Ron,
            Some("bin") | Some("bincode") => SerializationFormat::Bincode,
            Some("json") => SerializationFormat::Json,
            _ => SerializationFormat::Json, // 默认使用JSON
        }
    }
}

impl GameState {
    /// 当前序列化版本
    pub const CURRENT_VERSION: u32 = 1;

    /// 创建新的游戏状态
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            version: Self::CURRENT_VERSION,
            scenes: Vec::new(),
            current_scene_index: None,
            global_variables: HashMap::new(),
            game_time: GameTime::default(),
            metadata: GameStateMetadata {
                created_at: now,
                modified_at: now,
                ..Default::default()
            },
        }
    }

    /// 从World创建游戏状态
    pub fn from_world(world: &mut World, save_name: impl Into<String>) -> Self {
        let mut state = Self::new();
        state.metadata.save_name = save_name.into();

        // 从World中提取时间信息（如果存在）
        if let Some(time) = world.get_resource::<Time>() {
            state.game_time.total_time = time.elapsed_seconds as f32;
            // Time资源没有frame_count字段，使用默认值
            state.game_time.frame_count = 0;
        }

        // 序列化所有场景（这里简化为单一场景）
        let scene = SerializedScene::from_world(world, "main_scene");
        state.scenes.push(scene);
        state.current_scene_index = Some(0);

        state
    }

    /// 将游戏状态应用到World
    pub fn apply_to_world(&self, world: &mut World) -> Result<(), String> {
        // 清空World
        SerializedScene::clear_world(world);

        // 应用当前场景
        if let Some(index) = self.current_scene_index {
            if let Some(scene) = self.scenes.get(index) {
                scene.to_world(world);
            }
        }

        // 应用时间信息
        // 注意：Time资源需要由系统管理，这里只是记录数据

        Ok(())
    }

    /// 保存到文件
    pub fn save_to_file(
        &self,
        path: impl AsRef<Path>,
        format: SerializationFormat,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();

        match format {
            SerializationFormat::Ron => {
                let ron_str = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
                std::fs::write(path, ron_str)?;
            }
            SerializationFormat::Bincode => {
                let bincode_bytes = bincode_compat::serialize(self)?;
                std::fs::write(path, bincode_bytes)?;
            }
            SerializationFormat::Json => {
                let json_str = serde_json::to_string_pretty(self)?;
                std::fs::write(path, json_str)?;
            }
        }

        Ok(())
    }

    /// 从文件加载
    pub fn load_from_file(
        path: impl AsRef<Path>,
        format: SerializationFormat,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let data = std::fs::read(path)?;

        let state = match format {
            SerializationFormat::Ron => {
                let ron_str = std::str::from_utf8(&data)?;
                let mut state: GameState = ron::from_str(ron_str)?;

                // 版本迁移
                if state.version < Self::CURRENT_VERSION {
                    state = Self::migrate(state)?;
                }

                state
            }
            SerializationFormat::Bincode => {
                let mut state: GameState = bincode_compat::deserialize(&data)?;

                // 版本迁移
                if state.version < Self::CURRENT_VERSION {
                    state = Self::migrate(state)?;
                }

                state
            }
            SerializationFormat::Json => {
                let json_str = std::str::from_utf8(&data)?;
                let mut state: GameState = serde_json::from_str(json_str)?;

                // 版本迁移
                if state.version < Self::CURRENT_VERSION {
                    state = Self::migrate(state)?;
                }

                state
            }
        };

        Ok(state)
    }

    /// 自动检测格式并加载
    pub fn load_from_file_auto(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let extension =
            path.extension().and_then(|ext| ext.to_str()).ok_or("Invalid file extension")?;

        let format = match extension {
            "ron" => SerializationFormat::Ron,
            "bin" | "bincode" => SerializationFormat::Bincode,
            "json" => SerializationFormat::Json,
            _ => return Err("Unsupported file format".into()),
        };

        Self::load_from_file(path, format)
    }

    /// 版本迁移
    ///
    /// 将旧版本的游戏状态迁移到当前版本
    fn migrate(mut old_state: GameState) -> Result<GameState, String> {
        tracing::info!(
            "Migrating game state from version {} to {}",
            old_state.version,
            Self::CURRENT_VERSION
        );

        // 版本1迁移逻辑
        while old_state.version < Self::CURRENT_VERSION {
            old_state = match old_state.version {
                0 => Self::migrate_v0_to_v1(old_state)?,
                _ => {
                    return Err(format!("Unknown version: {}", old_state.version));
                }
            };
        }

        Ok(old_state)
    }

    /// 从版本0迁移到版本1
    fn migrate_v0_to_v1(mut state: GameState) -> Result<GameState, String> {
        // 示例迁移：添加新字段
        if state.game_time.time_scale == 0.0 {
            state.game_time.time_scale = 1.0;
        }

        state.version = 1;
        Ok(state)
    }

    /// 设置全局变量
    pub fn set_global_variable(&mut self, key: String, value: String) {
        self.global_variables.insert(key, value);
    }

    /// 获取全局变量
    pub fn get_global_variable(&self, key: &str) -> Option<&String> {
        self.global_variables.get(key)
    }

    /// 设置玩家进度
    pub fn set_progress(&mut self, progress: PlayerProgress) {
        self.metadata.progress = progress;
    }

    /// 获取玩家进度
    pub fn get_progress(&self) -> &PlayerProgress {
        &self.metadata.progress
    }

    /// 更新修改时间
    pub fn update_modified_time(&mut self) {
        self.metadata.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// 获取文件大小预估
    pub fn estimate_size(&self, format: SerializationFormat) -> usize {
        match format {
            SerializationFormat::Ron => {
                // RON通常是JSON的1.2-1.5倍大小
                let result: Result<String, _> =
                    ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default());
                result.map(|s| s.len()).unwrap_or(0)
            }
            SerializationFormat::Bincode => {
                // Bincode通常是最紧凑的
                let result: Result<Vec<u8>, _> = bincode_compat::serialize(self).map_err(Box::new);
                result.map(|v| v.len()).unwrap_or(0)
            }
            SerializationFormat::Json => {
                let result: Result<String, _> = serde_json::to_string(self);
                result.map(|s| s.len()).unwrap_or(0)
            }
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::Transform;
    use glam::{Quat, Vec3};

    #[test]
    fn test_game_state_creation() {
        let state = GameState::new();
        assert_eq!(state.version, GameState::CURRENT_VERSION);
        assert!(state.scenes.is_empty());
    }

    #[test]
    fn test_game_state_from_world() {
        let mut world = World::new();

        // 添加一些实体
        world.spawn(Transform {
            pos: Vec3::new(1.0, 2.0, 3.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        });

        let state = GameState::from_world(&mut world, "test_save");
        assert_eq!(state.scenes.len(), 1);
        assert_eq!(state.metadata.save_name, "test_save");
    }

    #[test]
    fn test_serialization_formats() {
        let mut world = World::new();
        world.spawn(Transform {
            pos: Vec3::new(1.0, 2.0, 3.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        });

        let state = GameState::from_world(&mut world, "test");

        // 测试RON格式
        let ron_path = "/tmp/test_save.ron";
        state.save_to_file(ron_path, SerializationFormat::Ron).unwrap();
        let loaded_ron = GameState::load_from_file(ron_path, SerializationFormat::Ron).unwrap();
        assert_eq!(loaded_ron.scenes.len(), state.scenes.len());

        // 测试Bincode格式
        let bin_path = "/tmp/test_save.bin";
        state.save_to_file(bin_path, SerializationFormat::Bincode).unwrap();
        let loaded_bin = GameState::load_from_file(bin_path, SerializationFormat::Bincode).unwrap();
        assert_eq!(loaded_bin.scenes.len(), state.scenes.len());

        // 测试JSON格式
        let json_path = "/tmp/test_save.json";
        state.save_to_file(json_path, SerializationFormat::Json).unwrap();
        let loaded_json = GameState::load_from_file(json_path, SerializationFormat::Json).unwrap();
        assert_eq!(loaded_json.scenes.len(), state.scenes.len());

        // 清理
        std::fs::remove_file(ron_path).ok();
        std::fs::remove_file(bin_path).ok();
        std::fs::remove_file(json_path).ok();
    }

    #[test]
    fn test_global_variables() {
        let mut state = GameState::new();
        state.set_global_variable("difficulty".to_string(), "hard".to_string());
        state.set_global_variable("level".to_string(), "5".to_string());

        assert_eq!(
            state.get_global_variable("difficulty"),
            Some(&"hard".to_string())
        );
        assert_eq!(state.get_global_variable("level"), Some(&"5".to_string()));
        assert_eq!(state.get_global_variable("nonexistent"), None);
    }

    #[test]
    fn test_player_progress() {
        let mut state = GameState::new();
        let progress = PlayerProgress {
            current_level: "level_5".to_string(),
            unlocked_levels: vec![
                "level_1".to_string(),
                "level_2".to_string(),
                "level_3".to_string(),
                "level_4".to_string(),
                "level_5".to_string(),
            ],
            score: 10000,
            playtime_seconds: 3600,
        };

        state.set_progress(progress);
        let loaded_progress = state.get_progress();

        assert_eq!(loaded_progress.current_level, "level_5");
        assert_eq!(loaded_progress.unlocked_levels.len(), 5);
        assert_eq!(loaded_progress.score, 10000);
        assert_eq!(loaded_progress.playtime_seconds, 3600);
    }

    #[test]
    fn test_version_migration() {
        // 创建旧版本状态
        let old_state = GameState {
            version: 0,
            scenes: vec![],
            current_scene_index: None,
            global_variables: HashMap::new(),
            game_time: GameTime {
                total_time: 10.0,
                frame_count: 600,
                time_scale: 0.0, // 测试迁移
            },
            metadata: GameStateMetadata::default(),
        };

        // 迁移到当前版本
        let migrated = GameState::migrate(old_state).unwrap();
        assert_eq!(migrated.version, GameState::CURRENT_VERSION);
        assert_eq!(migrated.game_time.time_scale, 1.0); // 应该被迁移设置为1.0
    }

    #[test]
    fn test_auto_format_detection() {
        let state = GameState::new();

        // 测试自动检测
        let ron_path = "/tmp/auto_test.ron";
        state.save_to_file(ron_path, SerializationFormat::Ron).unwrap();
        let loaded = GameState::load_from_file_auto(ron_path).unwrap();
        assert_eq!(loaded.version, state.version);

        std::fs::remove_file(ron_path).ok();
    }
}
