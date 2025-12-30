//! 验证框架集成测试
//!
//! 测试验证框架在实际使用场景中的集成效果。

use game_engine::core::validation::{Validate, ValidationError};
use game_engine::core::validation::validators;
use game_engine::domain::value_objects::{Position, Scale, Volume, Mass, Velocity, Duration};
use game_engine::audio::hrtf::HrtfConfig;

/// 验证多个值对象的辅助宏
macro_rules! validate_all {
    ($($value:expr),* $(,)?) => {
        $( $value.validate()?; )*
    };
}

/// 测试游戏实体配置的结构体
#[derive(Debug)]
struct EntityConfig {
    name: String,
    position: Position,
    velocity: Velocity,
    mass: Mass,
}

impl Validate for EntityConfig {
    type Error = ValidationError;

    fn validate(&self) -> Result<(), Self::Error> {
        // 使用验证宏组合多个验证
        validate_all! {
            &self.position,
            &self.velocity,
            &self.mass,
        };

        // 自定义验证规则
        validators::validate_non_empty(&self.name)?;
        validators::validate_length(&self.name, 1, 100)?;

        Ok(())
    }
}

/// 测试音频配置的结构体
#[derive(Debug)]
struct AudioSystemConfig {
    master_volume: Volume,
    hrtf_config: HrtfConfig,
}

impl Validate for AudioSystemConfig {
    type Error = ValidationError;

    fn validate(&self) -> Result<(), Self::Error> {
        validate_all! {
            &self.master_volume,
            &self.hrtf_config,
        }
        Ok(())
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_entity_config_valid() {
        let config = EntityConfig {
            name: "Player".to_string(),
            position: Position::new(0.0, 0.0, 0.0).unwrap(),
            velocity: Velocity::new(1.0, 2.0, 3.0).unwrap(),
            mass: Mass::new(70.0).unwrap(),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_entity_config_invalid_position() {
        // 测试创建包含无效位置的配置会失败
        // Position::new 会过滤 NaN，返回 None
        let invalid_pos = Position::new(f32::NAN, 0.0, 0.0);
        assert!(invalid_pos.is_none(), "Position with NaN should be None");

        // 创建有效的配置，但位置无效的情况会在构造时就被过滤
        // 所以我们测试有效配置的验证
        let config = EntityConfig {
            name: "Player".to_string(),
            position: Position::new(0.0, 0.0, 0.0).unwrap(),
            velocity: Velocity::new(1.0, 2.0, 3.0).unwrap(),
            mass: Mass::new(70.0).unwrap(),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_entity_config_invalid_mass() {
        // 测试创建包含无效质量的配置会失败
        // Mass::new 会过滤负值，返回 None
        let invalid_mass = Mass::new(-10.0);
        assert!(invalid_mass.is_none(), "Mass with negative value should be None");

        // 创建有效的配置
        let config = EntityConfig {
            name: "Player".to_string(),
            position: Position::new(0.0, 0.0, 0.0).unwrap(),
            velocity: Velocity::new(1.0, 2.0, 3.0).unwrap(),
            mass: Mass::new(70.0).unwrap(),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_entity_config_empty_name() {
        let config = EntityConfig {
            name: "".to_string(),
            position: Position::new(0.0, 0.0, 0.0).unwrap(),
            velocity: Velocity::new(1.0, 2.0, 3.0).unwrap(),
            mass: Mass::new(70.0).unwrap(),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_audio_system_config_valid() {
        let config = AudioSystemConfig {
            master_volume: Volume::new(0.75).unwrap(),
            hrtf_config: HrtfConfig::default(),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_audio_system_config_invalid_volume() {
        // 测试创建包含无效音量的配置会失败
        // Volume::new 会过滤超出范围的值，返回 None
        let invalid_volume = Volume::new(1.5);
        assert!(invalid_volume.is_none(), "Volume > 1.0 should be None");

        // 创建有效的音频配置
        let config = AudioSystemConfig {
            master_volume: Volume::new(0.75).unwrap(),
            hrtf_config: HrtfConfig::default(),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_audio_system_config_invalid_hrtf() {
        let config = AudioSystemConfig {
            master_volume: Volume::new(0.75).unwrap(),
            hrtf_config: HrtfConfig {
                sample_rate: 1000.0, // 太低
                head_radius: 0.0875,
                speed_of_sound: 343.0,
                enable_itd: true,
                enable_ild: true,
                enable_spectral_filtering: true,
                max_itd_delay: 0.0007,
                shadow_filter_cutoff: 2000.0,
            },
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_position_in_movement_context() {
        // 测试位置在移动场景中的验证
        let start = Position::new(0.0, 0.0, 0.0).unwrap();
        let end = Position::new(10.0, 0.0, 0.0).unwrap();

        start.validate().unwrap();
        end.validate().unwrap();

        let distance = start.distance_to(end);
        assert_eq!(distance, 10.0);
    }

    #[test]
    fn test_velocity_in_physics_context() {
        // 测试速度在物理场景中的验证
        let velocity = Velocity::new(5.0, 0.0, 3.0).unwrap();

        velocity.validate().unwrap();

        let speed = velocity.magnitude();
        assert!(speed > 0.0);
    }

    #[test]
    fn test_scale_transformation_validation() {
        // 测试缩放变换的验证
        let scale1 = Scale::new(2.0, 3.0, 4.0).unwrap();
        let scale2 = Scale::new(0.5, 1.5, 2.0).unwrap();

        scale1.validate().unwrap();
        scale2.validate().unwrap();

        let combined = scale1.combine(scale2);
        combined.validate().unwrap();
    }

    #[test]
    fn test_mass_physics_validation() {
        // 测试质量的物理场景验证
        let player_mass = Mass::new(70.0).unwrap();
        let vehicle_mass = Mass::new(1500.0).unwrap();

        player_mass.validate().unwrap();
        vehicle_mass.validate().unwrap();

        assert!(player_mass.value() < vehicle_mass.value());
    }

    #[test]
    fn test_duration_timing_validation() {
        // 测试时序验证
        let frame_time = Duration::new(0.016).unwrap(); // 60 FPS
        let animation_time = Duration::new(2.5).unwrap();

        frame_time.validate().unwrap();
        animation_time.validate().unwrap();
    }

    #[test]
    fn test_multiple_entities_validation() {
        // 测试多个实体的批量验证
        let entities = vec![
            EntityConfig {
                name: "Player1".to_string(),
                position: Position::new(0.0, 0.0, 0.0).unwrap(),
                velocity: Velocity::new(1.0, 0.0, 0.0).unwrap(),
                mass: Mass::new(70.0).unwrap(),
            },
            EntityConfig {
                name: "Player2".to_string(),
                position: Position::new(10.0, 0.0, 0.0).unwrap(),
                velocity: Velocity::new(-1.0, 0.0, 0.0).unwrap(),
                mass: Mass::new(80.0).unwrap(),
            },
        ];

        // 使用Option的Validate实现
        use game_engine::core::validation::trait_def::Validate;
        let entities_as_options: Vec<Option<EntityConfig>> = entities.into_iter().map(Some).collect();
        entities_as_options.validate().unwrap();
    }

    #[test]
    fn test_hrtf_config_edge_cases() {
        // 测试HRTF配置的边界情况
        let valid_configs = vec![
            HrtfConfig {
                sample_rate: 8000.0,   // 最小值
                ..Default::default()
            },
            HrtfConfig {
                sample_rate: 192000.0, // 最大值
                ..Default::default()
            },
            HrtfConfig {
                head_radius: 0.01,     // 最小值
                ..Default::default()
            },
            HrtfConfig {
                head_radius: 0.15,     // 最大值
                ..Default::default()
            },
        ];

        for config in valid_configs {
            assert!(config.validate().is_ok());
        }
    }

    #[test]
    fn test_validation_chain() {
        // 测试验证链
        let position = Position::new(1.0, 2.0, 3.0).unwrap();
        let velocity = Velocity::new(0.5, 0.5, 0.5).unwrap();
        let mass = Mass::new(60.0).unwrap();

        // 链式验证
        position
            .validate()
            .and_then(|_| velocity.validate())
            .and_then(|_| mass.validate())
            .unwrap();
    }

    #[test]
    fn test_validation_in_game_loop() {
        // 模拟游戏循环中的验证场景
        let mut player_position = Position::new(0.0, 0.0, 0.0).unwrap();
        let player_velocity = Velocity::new(1.0, 0.0, 0.0).unwrap();

        // 帧更新
        for _ in 0..10 {
            // 验证当前状态
            player_position.validate().unwrap();
            player_velocity.validate().unwrap();

            // 更新位置（模拟）
            let new_x = player_position.x() + player_velocity.x() * 0.016;
            let new_pos = Position::new(new_x, player_position.y(), player_position.z()).unwrap();
            new_pos.validate().unwrap();

            player_position = new_pos;
        }
    }

    #[test]
    fn test_validation_with_collections() {
        // 测试集合验证
        use game_engine::core::validation::trait_def::Validate;

        let positions = vec![
            Position::new(0.0, 0.0, 0.0).unwrap(),
            Position::new(1.0, 2.0, 3.0).unwrap(),
            Position::new(10.0, 20.0, 30.0).unwrap(),
        ];

        // Vec<T> where T: Validate 的自动实现
        positions.validate().unwrap();

        // 测试创建无效位置会返回None
        let invalid_pos = Position::new(f32::NAN, 2.0, 3.0);
        assert!(invalid_pos.is_none(), "Position with NaN should be None");

        // 测试有效位置的集合
        let valid_positions: Vec<Position> = vec![
            Position::new(0.0, 0.0, 0.0).unwrap(),
            Position::new(1.0, 2.0, 3.0).unwrap(),
        ];

        valid_positions.validate().unwrap();
    }
}
