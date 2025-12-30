// 验证框架性能基准测试
//
// 测试验证框架在各种场景下的性能表现

use game_engine::core::validation::{Validate, ValidationError};
use game_engine::core::validation::validators;
use game_engine::domain::value_objects::{Position, Velocity, Mass, Scale, Volume, Duration};

#[cfg(test)]
mod validation_benches {
    use super::*;

    // ============================================================================
    // 基准测试1: 简单值对象验证
    // ============================================================================

    #[bench]
    fn bench_position_validation(b: &mut test::Bencher) {
        let pos = Position::new(1.0, 2.0, 3.0).unwrap();
        b.iter(|| {
            pos.validate().unwrap();
        });
    }

    #[bench]
    fn bench_velocity_validation(b: &mut test::Bencher) {
        let vel = Velocity::new(1.0, 2.0, 3.0).unwrap();
        b.iter(|| {
            vel.validate().unwrap();
        });
    }

    #[bench]
    fn bench_mass_validation(b: &mut test::Bencher) {
        let mass = Mass::new(70.0).unwrap();
        b.iter(|| {
            mass.validate().unwrap();
        });
    }

    // ============================================================================
    // 基准测试2: 批量验证
    // ============================================================================

    #[bench]
    fn bench_batch_position_validation_100(b: &mut test::Bencher) {
        let positions: Vec<Position> = (0..100)
            .map(|i| Position::new(i as f32, i as f32, i as f32).unwrap())
            .collect();

        b.iter(|| {
            for pos in &positions {
                pos.validate().unwrap();
            }
        });
    }

    #[bench]
    fn bench_batch_mass_validation_1000(b: &mut test::Bencher) {
        let masses: Vec<Mass> = (0..1000)
            .map(|i| Mass::new((i % 200) as f32).unwrap())
            .collect();

        b.iter(|| {
            for mass in &masses {
                mass.validate().unwrap();
            }
        });
    }

    // ============================================================================
    // 基准测试3: 复杂对象验证
    // ============================================================================

    struct GameEntity {
        position: Position,
        velocity: Velocity,
        mass: Mass,
    }

    impl Validate for GameEntity {
        type Error = ValidationError;

        fn validate(&self) -> Result<(), Self::Error> {
            self.position.validate()?;
            self.velocity.validate()?;
            self.mass.validate()?;
            Ok(())
        }
    }

    #[bench]
    fn bench_entity_validation(b: &mut test::Bencher) {
        let entity = GameEntity {
            position: Position::new(1.0, 2.0, 3.0).unwrap(),
            velocity: Velocity::new(1.0, 2.0, 3.0).unwrap(),
            mass: Mass::new(70.0).unwrap(),
        };

        b.iter(|| {
            entity.validate().unwrap();
        });
    }

    // ============================================================================
    // 基准测试4: 验证器函数性能
    // ============================================================================

    #[bench]
    fn bench_validate_finite(b: &mut test::Bencher) {
        b.iter(|| {
            validators::validate_finite(42.0).unwrap();
        });
    }

    #[bench]
    fn bench_validate_range(b: &mut test::Bencher) {
        b.iter(|| {
            validators::validate_range(50.0, 0.0, 100.0).unwrap();
        });
    }

    #[bench]
    fn bench_validate_non_negative(b: &mut test::Bencher) {
        b.iter(|| {
            validators::validate_non_negative_f32(42.0).unwrap();
        });
    }

    // ============================================================================
    // 基准测试5: 游戏循环模拟
    // ============================================================================

    #[bench]
    fn bench_game_loop_validation_60_frames(b: &mut test::Bencher) {
        let mut position = Position::new(0.0, 0.0, 0.0).unwrap();
        let velocity = Velocity::new(1.0, 0.0, 0.0).unwrap();

        b.iter(|| {
            for _ in 0..60 {
                position.validate().unwrap();
                velocity.validate().unwrap();

                // 更新位置
                let new_x = position.x() + velocity.x() * 0.016;
                position = Position::new(new_x, position.y(), position.z()).unwrap();
            }
        });
    }

    // ============================================================================
    // 基准测试6: 创建和验证
    // ============================================================================

    #[bench]
    fn bench_create_and_validate_position(b: &mut test::Bencher) {
        b.iter(|| {
            let pos = Position::new(1.0, 2.0, 3.0).unwrap();
            pos.validate().unwrap();
        });
    }

    #[bench]
    fn bench_create_and_validate_mass(b: &mut test::Bencher) {
        b.iter(|| {
            let mass = Mass::new(70.0).unwrap();
            mass.validate().unwrap();
        });
    }

    // ============================================================================
    // 基准测试7: 集合验证
    // ============================================================================

    #[bench]
    fn bench_vec_validation_10(b: &mut test::Bencher) {
        use game_engine::core::validation::trait_def::Validate;

        let positions: Vec<Position> = (0..10)
            .map(|i| Position::new(i as f32, i as f32, i as f32).unwrap())
            .collect();

        b.iter(|| {
            positions.validate().unwrap();
        });
    }

    #[bench]
    fn bench_vec_validation_100(b: &mut test::Bencher) {
        use game_engine::core::validation::trait_def::Validate;

        let positions: Vec<Position> = (0..100)
            .map(|i| Position::new(i as f32, i as f32, i as f32).unwrap())
            .collect();

        b.iter(|| {
            positions.validate().unwrap();
        });
    }
}
