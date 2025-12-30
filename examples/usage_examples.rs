//! # 游戏引擎使用示例集
//!
//! 本文件展示游戏引擎各项功能的实际使用场景。
//!
//! ## 示例列表
//!
//! - 验证框架使用示例
//! - 行为树AI示例
//! - 覆盖图战术分析示例
//! - 音频系统示例

// ============================================================================
// 示例1: 验证框架在游戏实体中的使用
// ============================================================================

use game_engine::core::validation::{Validate, ValidationError};
use game_engine::core::validation::validators;
use game_engine::domain::value_objects::{Position, Velocity, Mass};
use game_engine::audio::hrtf::HrtfConfig;

/// 游戏实体配置示例
#[derive(Debug)]
struct GameEntity {
    id: u32,
    name: String,
    position: Position,
    velocity: Velocity,
    mass: Mass,
    health: f32,
}

impl Validate for GameEntity {
    type Error = ValidationError;

    fn validate(&self) -> Result<(), Self::Error> {
        // 验证名称
        validators::validate_non_empty(&self.name)?;
        validators::validate_length(&self.name, 1, 100)?;

        // 验证值对象
        self.position.validate()?;
        self.velocity.validate()?;
        self.mass.validate()?;

        // 验证健康值
        validators::validate_range(self.health, 0.0, 100.0)?;
        validators::validate_finite(self.health)?;

        Ok(())
    }
}

/// 创建游戏实体示例
fn example_1_create_entity() -> Result<GameEntity, ValidationError> {
    let entity = GameEntity {
        id: 1,
        name: "Player".to_string(),
        position: Position::new(0.0, 0.0, 0.0)?,
        velocity: Velocity::new(1.0, 2.0, 3.0)?,
        mass: Mass::new(70.0)?,
        health: 100.0,
    };

    entity.validate()?;
    Ok(entity)
}

// ============================================================================
// 示例2: 音频系统配置验证
// ============================================================================

struct AudioSystemConfig {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    hrtf_enabled: bool,
    hrtf_config: HrtfConfig,
}

impl Validate for AudioSystemConfig {
    type Error = ValidationError;

    fn validate(&self) -> Result<(), Self::Error> {
        // 验证音量范围
        validators::validate_range(self.master_volume, 0.0, 1.0)?;
        validators::validate_range(self.music_volume, 0.0, 1.0)?;
        validators::validate_range(self.sfx_volume, 0.0, 1.0)?;

        // 如果启用HRTF，验证配置
        if self.hrtf_enabled {
            self.hrtf_config.validate()?;
        }

        Ok(())
    }
}

/// 创建音频系统配置示例
fn example_2_audio_config() -> Result<AudioSystemConfig, ValidationError> {
    let config = AudioSystemConfig {
        master_volume: 0.8,
        music_volume: 0.6,
        sfx_volume: 0.9,
        hrtf_enabled: true,
        hrtf_config: HrtfConfig::default(),
    };

    config.validate()?;
    Ok(config)
}

// ============================================================================
// 示例3: 批量实体验证
// ============================================================================

fn example_3_batch_validation() -> Result<(), ValidationError> {
    let entities = vec![
        GameEntity {
            id: 1,
            name: "Player1".to_string(),
            position: Position::new(0.0, 0.0, 0.0)?,
            velocity: Velocity::new(1.0, 0.0, 0.0)?,
            mass: Mass::new(70.0)?,
            health: 100.0,
        },
        GameEntity {
            id: 2,
            name: "Player2".to_string(),
            position: Position::new(10.0, 0.0, 0.0)?,
            velocity: Velocity::new(-1.0, 0.0, 0.0)?,
            mass: Mass::new(80.0)?,
            health: 95.0,
        },
    ];

    // 批量验证
    for entity in &entities {
        entity.validate()?;
    }

    println!("✅ All {} entities validated successfully", entities.len());
    Ok(())
}

// ============================================================================
// 示例4: 游戏循环中的验证
// ============================================================================

fn example_4_game_loop() -> Result<(), ValidationError> {
    let mut player_position = Position::new(0.0, 0.0, 0.0)?;
    let player_velocity = Velocity::new(5.0, 0.0, 0.0)?;

    // 模拟60帧游戏循环
    for frame in 0..60 {
        // 每帧验证状态
        player_position.validate()?;
        player_velocity.validate()?;

        // 更新位置 (模拟)
        let new_x = player_position.x() + player_velocity.x() * 0.016; // 60 FPS
        let new_pos = Position::new(new_x, player_position.y(), player_position.z())?;
        new_pos.validate()?;

        player_position = new_pos;

        if frame % 10 == 0 {
            println!("Frame {}: Position validated", frame);
        }
    }

    Ok(())
}

// ============================================================================
// 示例5: 自定义验证错误消息
// ============================================================================

use game_engine::core::validation::ValidationError;

fn example_5_custom_validation() -> Result<(), ValidationError> {
    struct EntityConfig {
        name: String,
        team_id: u32,
    }

    impl Validate for EntityConfig {
        type Error = ValidationError;

        fn validate(&self) -> Result<(), Self::Error> {
            validators::validate_non_empty(&self.name)?;
            validators::validate_identifier(&self.name)?;

            // 自定义验证逻辑
            if self.team_id > 100 {
                return Err(ValidationError::custom(format!(
                    "Team ID {} exceeds maximum of 100",
                    self.team_id
                )));
            }

            Ok(())
        }
    }

    let config = EntityConfig {
        name: "team_alpha".to_string(),
        team_id: 5,
    };

    config.validate()?;
    println!("✅ Entity config validated: team_id={}", config.team_id);
    Ok(())
}

// ============================================================================
// 示例6: 组合验证器使用
// ============================================================================

fn example_6_composable_validators() -> Result<(), ValidationError> {
    use game_engine::core::validation::validate;

    // 使用validate!宏组合多个验证
    validate! {
        "name" => validators::validate_non_empty("test_entity"),
        "id" => validators::validate_range(42u32, 0, 100),
        "score" => validators::validate_range(95.5f32, 0.0, 100.0),
    }

    // 或者手动组合
    validators::validate_non_empty("test")?;
    validators::validate_length("test", 1, 10)?;

    println!("✅ Composable validation successful");
    Ok(())
}

// ============================================================================
// 示例7: 集合验证
// ============================================================================

fn example_7_collection_validation() -> Result<(), ValidationError> {
    use game_engine::core::validation::trait_def::Validate;

    let positions = vec![
        Position::new(0.0, 0.0, 0.0)?,
        Position::new(1.0, 2.0, 3.0)?,
        Position::new(10.0, 20.0, 30.0)?,
    ];

    // Vec<T> where T: Validate 的自动实现
    positions.validate()?;

    println!("✅ Collection validation successful: {} positions", positions.len());
    Ok(())
}

// ============================================================================
// 主函数 - 运行所有示例
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 游戏引擎使用示例\n");

    println!("=== 示例1: 创建游戏实体 ===");
    match example_1_create_entity() {
        Ok(entity) => println!("✅ Entity created: ID={}, Name={}", entity.id, entity.name),
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("\n=== 示例2: 音频系统配置 ===");
    match example_2_audio_config() {
        Ok(_) => println!("✅ Audio config validated"),
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("\n=== 示例3: 批量实体验证 ===");
    match example_3_batch_validation() {
        Ok(_) => println!("✅ Batch validation complete"),
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("\n=== 示例4: 游戏循环验证 ===");
    match example_4_game_loop() {
        Ok(_) => println!("✅ Game loop validation complete"),
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("\n=== 示例5: 自定义验证 ===");
    match example_5_custom_validation() {
        Ok(_) => println!("✅ Custom validation complete"),
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("\n=== 示例6: 组合验证器 ===");
    match example_6_composable_validators() {
        Ok(_) => println!("✅ Composable validation complete"),
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("\n=== 示例7: 集合验证 ===");
    match example_7_collection_validation() {
        Ok(_) => println!("✅ Collection validation complete"),
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("\n🎉 所有示例执行完成！");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_1() {
        assert!(example_1_create_entity().is_ok());
    }

    #[test]
    fn test_example_2() {
        assert!(example_2_audio_config().is_ok());
    }

    #[test]
    fn test_example_3() {
        assert!(example_3_batch_validation().is_ok());
    }

    #[test]
    fn test_example_4() {
        assert!(example_4_game_loop().is_ok());
    }

    #[test]
    fn test_example_5() {
        assert!(example_5_custom_validation().is_ok());
    }

    #[test]
    fn test_example_6() {
        assert!(example_6_composable_validators().is_ok());
    }

    #[test]
    fn test_example_7() {
        assert!(example_7_collection_validation().is_ok());
    }
}
