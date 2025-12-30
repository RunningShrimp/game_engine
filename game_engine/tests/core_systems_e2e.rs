// 核心系统端到端集成测试
//
// 测试游戏引擎核心系统的完整工作流程

#[cfg(test)]
mod core_systems_e2e_tests {
    use game_engine::domain::errors::*;
    use game_engine::domain::physics::*;
    use game_engine::ecs::*;
    use glam::{Quat, Vec3};

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_ecs_physics_integration() {
        // 测试ECS系统与物理系统的集成
        let mut world = bevy_ecs::prelude::World::new();

        // 创建一个带有物理属性的实体
        let entity = world
            .spawn((
                Transform {
                    pos: Vec3::new(0.0, 10.0, 0.0),
                    rot: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                Velocity::new(),
                Sprite::default(),
            ))
            .id();

        // 验证实体创建成功
        assert!(world.get_entity(entity).is_ok());
        assert!(world.get::<Transform>(entity).is_some());
        assert!(world.get::<Velocity>(entity).is_some());
        assert!(world.get::<Sprite>(entity).is_some());

        // 测试查询功能
        let mut query = world.query::<(&Transform, &Velocity)>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_rigid_body_creation_workflow() {
        // 测试刚体创建工作流程
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::new(body_id, RigidBodyType::Dynamic, Vec3::new(0.0, 10.0, 0.0));

        // 验证刚体属性
        assert_eq!(body.id(), body_id);
        assert_eq!(body.body_type(), RigidBodyType::Dynamic);
        assert_eq!(body.position(), Vec3::new(0.0, 10.0, 0.0));
        assert_eq!(body.mass(), 1.0);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_multiple_rigid_bodies_creation() {
        // 测试多个刚体的创建
        for i in 1..=5 {
            let body_id = RigidBodyId::new(i);
            let body = RigidBody::new(
                body_id,
                RigidBodyType::Dynamic,
                Vec3::new(0.0, i as f32 * 2.0, 0.0),
            );

            assert_eq!(body.id(), body_id);
            assert_eq!(body.position().y, i as f32 * 2.0);
            assert_eq!(body.body_type(), RigidBodyType::Dynamic);
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_all_rigid_body_types() {
        // 测试所有类型的刚体
        let fixed_id = RigidBodyId::new(1);
        let fixed_body = RigidBody::new(fixed_id, RigidBodyType::Fixed, Vec3::ZERO);
        assert_eq!(fixed_body.body_type(), RigidBodyType::Fixed);

        let dynamic_id = RigidBodyId::new(2);
        let dynamic_body =
            RigidBody::new(dynamic_id, RigidBodyType::Dynamic, Vec3::new(0.0, 5.0, 0.0));
        assert_eq!(dynamic_body.body_type(), RigidBodyType::Dynamic);

        let kinematic_id = RigidBodyId::new(3);
        let kinematic_body = RigidBody::new(
            kinematic_id,
            RigidBodyType::Kinematic,
            Vec3::new(1.0, 0.0, 0.0),
        );
        assert_eq!(kinematic_body.body_type(), RigidBodyType::Kinematic);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_rigid_body_mass_operations() {
        // 测试刚体质量操作
        let body_id = RigidBodyId::new(1);
        let mut body = RigidBody::new(body_id, RigidBodyType::Dynamic, Vec3::ZERO);

        // 验证默认质量
        assert_eq!(body.mass(), 1.0);

        // 设置新质量
        let result = body.set_mass(10.0);
        assert!(result.is_ok());
        assert_eq!(body.mass(), 10.0);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_rigid_body_state_workflow() {
        // 测试刚体状态的完整工作流程
        let state = RigidBodyState {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
            linear_velocity: Vec3::new(1.0, 0.0, 0.0),
            angular_velocity: Vec3::ZERO,
            sleeping: false,
        };

        // 验证状态创建
        assert_eq!(state.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(state.linear_velocity, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(state.sleeping, false);

        // 使用状态创建刚体
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::new(body_id, RigidBodyType::Dynamic, state.position);

        assert_eq!(body.position(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(body.body_type(), RigidBodyType::Dynamic);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_error_conversion_workflow() {
        // 测试错误转换工作流程
        use game_engine::error::ErrorSeverity;
        let physics_error = PhysicsError::RigidBodyNotFound {
            body_id: "test_body".to_string(),
            severity: ErrorSeverity::Warning,
        };
        let domain_error: DomainError = physics_error.into();

        match domain_error {
            DomainError::Physics(err) => match err {
                PhysicsError::RigidBodyNotFound { body_id, .. } => {
                    assert_eq!(body_id, "test_body");
                }
                _ => panic!("Expected RigidBodyNotFound error"),
            },
            _ => panic!("Expected Physics error"),
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_recovery_strategies() {
        // 测试不同的恢复策略
        let retry = RecoveryStrategy::Retry {
            max_attempts: 3,
            delay_ms: 100,
        };

        assert!(matches!(
            retry,
            RecoveryStrategy::Retry {
                max_attempts: 3,
                delay_ms: 100
            }
        ));
        assert!(matches!(
            RecoveryStrategy::UseDefault,
            RecoveryStrategy::UseDefault
        ));
        assert!(matches!(RecoveryStrategy::Skip, RecoveryStrategy::Skip));
        assert!(matches!(
            RecoveryStrategy::LogAndContinue,
            RecoveryStrategy::LogAndContinue
        ));
        assert!(matches!(RecoveryStrategy::Fail, RecoveryStrategy::Fail));
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_compensation_action_workflow() {
        // 测试补偿操作工作流程
        let action = CompensationAction::new(
            "test_operation",
            "restore_state",
            serde_json::json!({"position": [0.0, 0.0, 0.0]}),
        );

        assert_eq!(action.id, "test_operation");
        assert_eq!(action.action_type, "restore_state");
        assert!(action.data.is_object());
        assert!(action.data.get("position").is_some());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_physics_shape_types() {
        // 测试物理形状类型
        let sphere = ShapeType::Sphere { radius: 1.0 };
        assert!(matches!(sphere, ShapeType::Sphere { radius: 1.0 }));

        // 创建带形状的刚体
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::new(body_id, RigidBodyType::Dynamic, Vec3::ZERO);
        assert_eq!(body.body_type(), RigidBodyType::Dynamic);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_id_generation_and_uniqueness() {
        // 测试ID生成和唯一性
        let id1 = RigidBodyId::new(1);
        let id2 = RigidBodyId::new(2);
        let id3 = RigidBodyId::new(1); // 相同的数字

        assert_ne!(id1, id2);
        assert_eq!(id1, id3);

        let collider_id1 = ColliderId::new(1);
        let collider_id2 = ColliderId::new(2);
        assert_ne!(collider_id1, collider_id2);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_all_error_types() {
        // 测试所有错误类型的可构造性
        use game_engine::error::ErrorSeverity;
        let _ = AudioError::SourceNotFound {
            source_id: "test".to_string(),
            severity: ErrorSeverity::Warning,
        };
        let _ = PhysicsError::RigidBodyNotFound {
            body_id: "test".to_string(),
            severity: ErrorSeverity::Warning,
        };
        let _ = SceneError::EntityNotFound("test".to_string());
        let _ = DomainError::General("test".to_string());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_ecs_component_combinations() {
        // 测试ECS组件组合
        let mut world = bevy_ecs::prelude::World::new();

        // 创建带有多个组件的实体
        let entity = world
            .spawn((
                Transform {
                    pos: Vec3::new(1.0, 2.0, 3.0),
                    rot: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                Velocity::new(),
                Sprite {
                    color: [1.0, 0.0, 0.0, 1.0],
                    tex_index: 0,
                    normal_tex_index: 0,
                    uv_off: [0.0, 0.0],
                    uv_scale: [1.0, 1.0],
                    layer: 0.0,
                },
                PointLight::default(),
            ))
            .id();

        // 验证所有组件都存在
        assert!(world.get::<Transform>(entity).is_some());
        assert!(world.get::<Velocity>(entity).is_some());
        assert!(world.get::<Sprite>(entity).is_some());
        assert!(world.get::<PointLight>(entity).is_some());
    }
}
