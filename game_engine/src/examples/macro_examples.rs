//! # 宏使用示例
//!
//! 展示如何使用game_engine_macros提供的派生宏来减少代码重复。

use bevy_ecs::component::Component;
use game_engine_macros::{Constructor, Serializable};
use serde::{Deserialize, Serialize};

// ============================================================================
// 示例1: Constructor宏
// ============================================================================

/// 使用Constructor宏自动生成构造函数
#[derive(Constructor, Debug, Clone, PartialEq)]
pub struct Player {
    name: String,
    level: u32,
    health: f32,
    mana: f32,
}

// Constructor宏自动生成以下代码:
// impl Player {
//     #[inline]
//     pub fn new(name: String, level: u32, health: f32, mana: f32) -> Self {
//         Self { name, level, health, mana }
//     }
// }

#[cfg(test)]
mod constructor_tests {
    use super::*;

    #[test]
    fn test_constructor_macro() {
        let player = Player::new("Alice".to_string(), 10, 100.0, 50.0);

        assert_eq!(player.name, "Alice");
        assert_eq!(player.level, 10);
        assert_eq!(player.health, 100.0);
        assert_eq!(player.mana, 50.0);
    }

    #[test]
    fn test_constructor_clone() {
        let player1 = Player::new("Bob".to_string(), 5, 80.0, 40.0);
        let player2 = player1.clone();

        assert_eq!(player1, player2);
    }
}

// ============================================================================
// 示例2: ECS组件包装器
// ============================================================================

/// ECS组件示例 - 使用Constructor宏
#[derive(Constructor, Component, Debug, Clone, Copy, PartialEq)]
pub struct Velocity2 {
    x: f32,
    y: f32,
    z: f32,
}

impl Velocity2 {
    /// 从Vec3创建Velocity2
    pub fn from_vec3(v: glam::Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }

    /// 转换为Vec3
    pub fn to_vec3(&self) -> glam::Vec3 {
        glam::Vec3::new(self.x, self.y, self.z)
    }
}

#[derive(Constructor, Component, Debug, Clone, Copy)]
pub struct Position2 {
    x: f32,
    y: f32,
    z: f32,
}

impl Position2 {
    /// 从Vec3创建Position2
    pub fn from_vec3(v: glam::Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }

    /// 转换为Vec3
    pub fn to_vec3(&self) -> glam::Vec3 {
        glam::Vec3::new(self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod component_tests {
    use super::*;
    use bevy_ecs::world::World;

    #[test]
    fn test_velocity2() {
        let velocity = Velocity2::new(1.0, 2.0, 3.0);

        assert_eq!(velocity.x, 1.0);
        assert_eq!(velocity.y, 2.0);
        assert_eq!(velocity.z, 3.0);

        // 测试与Vec3的转换
        let vec3 = glam::Vec3::new(4.0, 5.0, 6.0);
        let velocity = Velocity2::from_vec3(vec3);
        assert_eq!(velocity.to_vec3(), vec3);
    }

    #[test]
    fn test_component_in_world() {
        let mut world = World::new();

        // Component trait已通过#[derive(Component)]自动实现
        let entity = world.spawn(Velocity2::new(1.0, 2.0, 3.0)).id();

        // 可以查询组件
        let velocity = world.get::<Velocity2>(entity).unwrap();
        assert_eq!(velocity.x, 1.0);
        assert_eq!(velocity.y, 2.0);
        assert_eq!(velocity.z, 3.0);
    }
}

// ============================================================================
// 示例3: Serializable宏
// ============================================================================

/// 使用Serializable宏自动生成序列化方法
#[derive(Serializable, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameState {
    score: u32,
    level: u32,
    player_name: String,
}

// Serializable宏自动生成以下代码:
// impl GameState {
//     pub fn serialize(&self) -> Result<Vec<u8>, SerializationError>;
//     pub fn deserialize(data: &[u8]) -> Result<Self, SerializationError>;
// }

#[cfg(test)]
mod serializable_tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let state = GameState {
            score: 1000,
            level: 5,
            player_name: "Alice".to_string(),
        };

        // 序列化
        let serialized = state.serialize().unwrap();
        assert!(!serialized.is_empty());

        // 反序列化
        let deserialized = GameState::deserialize(&serialized).unwrap();
        assert_eq!(deserialized, state);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original = GameState {
            score: 9999,
            level: 99,
            player_name: "TestPlayer".to_string(),
        };

        // 序列化 -> 反序列化
        let data = original.serialize().unwrap();
        let restored = GameState::deserialize(&data).unwrap();

        assert_eq!(restored.score, original.score);
        assert_eq!(restored.level, original.level);
        assert_eq!(restored.player_name, original.player_name);
    }
}

// ============================================================================
// 示例4: 组合多个宏
// ============================================================================

/// 组合使用多个宏获得最大收益
#[derive(Constructor, Serializable, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Enemy {
    name: String,
    enemy_type: String,
    health: f32,
    damage: f32,
}

// 自动生成:
// 1. new() 构造函数 (Constructor)
// 2. serialize() / deserialize() 方法 (Serializable)

#[cfg(test)]
mod combined_macros_tests {
    use super::*;

    #[test]
    fn test_combined_macros() {
        // 使用构造函数
        let enemy = Enemy::new("Goblin".to_string(), "Monster".to_string(), 50.0, 10.0);

        // 序列化
        let data = enemy.serialize().unwrap();

        // 反序列化
        let restored = Enemy::deserialize(&data).unwrap();

        assert_eq!(restored, enemy);
    }
}

// ============================================================================
// 示例5: 实际游戏场景
// ============================================================================

/// 游戏状态管理器
#[derive(Constructor, Serializable, Serialize, Deserialize, Debug, Clone)]
pub struct GameManager {
    current_level: u32,
    total_score: u32,
    player_alive: bool,
}

impl GameManager {
    /// 保存游戏状态
    pub fn save_game(&self) -> Result<Vec<u8>, crate::error::SerializationError> {
        self.serialize()
    }

    /// 加载游戏状态
    pub fn load_game(data: &[u8]) -> Result<Self, crate::error::SerializationError> {
        Self::deserialize(data)
    }

    /// 检查玩家是否存活
    pub fn is_player_alive(&self) -> bool {
        self.player_alive
    }
}

#[cfg(test)]
mod game_manager_tests {
    use super::*;

    #[test]
    fn test_save_load_game() {
        // 创建游戏状态
        let game = GameManager::new(10, 5000, true);

        // 保存游戏
        let saved_data = game.save_game().unwrap();

        // 加载游戏
        let loaded_game = GameManager::load_game(&saved_data).unwrap();

        assert_eq!(loaded_game.current_level, 10);
        assert_eq!(loaded_game.total_score, 5000);
        assert_eq!(loaded_game.is_player_alive(), true);
    }
}
