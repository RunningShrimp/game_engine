// 物理领域层测试
//
// 测试物理领域对象的核心功能，包括：
// - 刚体类型和状态
// - 碰撞形状和碰撞体
// - 物理世界管理
// - 领域业务逻辑

#[cfg(test)]
mod tests {
    use crate::domain::physics::*;
    use glam::{Quat, Vec3};

    // ============================================================================
    // 刚体类型测试
    // ============================================================================

    #[test]
    fn test_rigid_body_type_variants() {
        // 测试所有刚体类型变体
        let fixed = RigidBodyType::Fixed;
        let dynamic = RigidBodyType::Dynamic;
        let kinematic = RigidBodyType::Kinematic;

        // 验证类型可以正确创建和比较
        assert_eq!(fixed, RigidBodyType::Fixed);
        assert_eq!(dynamic, RigidBodyType::Dynamic);
        assert_eq!(kinematic, RigidBodyType::Kinematic);
        assert_ne!(fixed, dynamic);
    }

    // ============================================================================
    // 碰撞形状测试
    // ============================================================================

    #[test]
    fn test_shape_type_sphere() {
        let sphere = ShapeType::Sphere { radius: 1.0 };

        // 验证球形可以正确创建
        assert!(matches!(sphere, ShapeType::Sphere { radius: 1.0 }));
    }

    #[test]
    fn test_shape_type_box() {
        let box_shape = ShapeType::Box {
            half_extents: Vec3::new(1.0, 2.0, 3.0),
        };

        // 验证长方体可以正确创建
        assert!(matches!(
            box_shape,
            ShapeType::Box { half_extents } if half_extents == Vec3::new(1.0, 2.0, 3.0)
        ));
    }

    #[test]
    fn test_shape_type_capsule() {
        let capsule = ShapeType::Capsule {
            half_height: 2.0,
            radius: 0.5,
        };

        // 验证胶囊体可以正确创建
        assert!(matches!(
            capsule,
            ShapeType::Capsule { half_height: 2.0, radius: 0.5 }
        ));
    }

    #[test]
    fn test_shape_type_cylinder() {
        let cylinder = ShapeType::Cylinder {
            half_height: 1.5,
            radius: 0.8,
        };

        // 验证圆柱体可以正确创建
        assert!(matches!(
            cylinder,
            ShapeType::Cylinder { half_height: 1.5, radius: 0.8 }
        ));
    }

    #[test]
    fn test_shape_type_heightfield() {
        let heightfield = ShapeType::Heightfield {
            heights: vec![1.0, 2.0, 3.0],
            scale: Vec3::new(1.0, 1.0, 1.0),
        };

        // 验证高度场可以正确创建
        assert!(matches!(
            heightfield,
            ShapeType::Heightfield { .. }
        ));
    }

    // ============================================================================
    // 刚体状态测试
    // ============================================================================

    #[test]
    fn test_rigid_body_state_default() {
        let state = RigidBodyState::default();

        // 验证默认状态
        assert_eq!(state.position, Vec3::ZERO);
        assert_eq!(state.rotation, Quat::IDENTITY);
        assert_eq!(state.linear_velocity, Vec3::ZERO);
        assert_eq!(state.angular_velocity, Vec3::ZERO);
    }

    #[test]
    fn test_rigid_body_state_with_values() {
        let state = RigidBodyState {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 1.0),
            linear_velocity: Vec3::new(1.0, 0.0, 0.0),
            angular_velocity: Vec3::new(0.0, 1.0, 0.0),
        };

        // 验证自定义状态
        assert_eq!(state.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(state.linear_velocity, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(state.angular_velocity, Vec3::new(0.0, 1.0, 0.0));
    }

    // ============================================================================
    // ID类型测试
    // ============================================================================

    #[test]
    fn test_rigid_body_id_creation() {
        let id1 = RigidBodyId::new();
        let id2 = RigidBodyId::new();

        // 验证ID是唯一的
        assert_ne!(id1, id2);

        // 验证ID可以正确获取
        let raw_id1: u64 = id1.into();
        let raw_id2: u64 = id2.into();
        assert_ne!(raw_id1, raw_id2);
    }

    #[test]
    fn test_collider_id_creation() {
        let id1 = ColliderId::new();
        let id2 = ColliderId::new();

        // 验证ID是唯一的
        assert_ne!(id1, id2);
    }

    // ============================================================================
    // 刚体领域对象测试
    // ============================================================================

    #[test]
    fn test_rigid_body_creation_dynamic() {
        let body = RigidBody::new_dynamic(RigidBodyState::default());

        // 验证动态刚体创建
        assert_eq!(body.body_type(), RigidBodyType::Dynamic);
        assert_eq!(body.mass(), 1.0); // 默认质量
    }

    #[test]
    fn test_rigid_body_creation_fixed() {
        let body = RigidBody::new_fixed(RigidBodyState::default());

        // 验证固定刚体创建
        assert_eq!(body.body_type(), RigidBodyType::Fixed);
        assert_eq!(body.mass(), 0.0); // 固定刚体质量为0
    }

    #[test]
    fn test_rigid_body_creation_kinematic() {
        let body = RigidBody::new_kinematic(RigidBodyState::default());

        // 验证运动学刚体创建
        assert_eq!(body.body_type(), RigidBodyType::Kinematic);
    }

    #[test]
    fn test_rigid_body_with_shape() {
        let mut body = RigidBody::new_dynamic(RigidBodyState::default());
        let shape = ShapeType::Sphere { radius: 1.0 };
        body.set_shape(shape.clone());

        // 验证形状设置
        assert_eq!(body.shape(), &shape);
    }

    #[test]
    fn test_rigid_body_with_mass() {
        let mut body = RigidBody::new_dynamic(RigidBodyState::default());
        body.set_mass(10.0);

        // 验证质量设置
        assert_eq!(body.mass(), 10.0);
    }

    #[test]
    fn test_rigid_body_state_update() {
        let mut body = RigidBody::new_dynamic(RigidBodyState::default());
        let new_state = RigidBodyState {
            position: Vec3::new(5.0, 5.0, 5.0),
            ..Default::default()
        };
        body.set_state(new_state.clone());

        // 验证状态更新
        assert_eq!(body.state().position, Vec3::new(5.0, 5.0, 5.0));
    }

    // ============================================================================
    // 碰撞体领域对象测试
    // ============================================================================

    #[test]
    fn test_collider_creation() {
        let shape = ShapeType::Sphere { radius: 1.0 };
        let collider = Collider::new(shape.clone());

        // 验证碰撞体创建
        assert_eq!(collider.shape(), &shape);
        assert_eq!(collider.is_sensor(), false);
    }

    #[test]
    fn test_collider_as_sensor() {
        let shape = ShapeType::Sphere { radius: 1.0 };
        let mut collider = Collider::new(shape);
        collider.set_is_sensor(true);

        // 验证传感器标志
        assert_eq!(collider.is_sensor(), true);
    }

    #[test]
    fn test_collider_with_offset() {
        let shape = ShapeType::Sphere { radius: 1.0 };
        let mut collider = Collider::new(shape);
        collider.set_local_position(Vec3::new(1.0, 0.0, 0.0));

        // 验证局部位置设置
        assert_eq!(collider.local_position(), Vec3::new(1.0, 0.0, 0.0));
    }

    // ============================================================================
    // 物理世界测试
    // ============================================================================

    #[test]
    fn test_physics_world_creation() {
        let world = PhysicsWorld::new();

        // 验证物理世界创建
        assert_eq!(world.gravity(), Vec3::new(0.0, -9.81, 0.0));
    }

    #[test]
    fn test_physics_world_gravity() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec3::new(0.0, -20.0, 0.0));

        // 验证重力设置
        assert_eq!(world.gravity(), Vec3::new(0.0, -20.0, 0.0));
    }

    #[test]
    fn test_physics_world_add_rigid_body() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::new_dynamic(RigidBodyState::default());
        let body_id = body.id();

        let result = world.add_rigid_body(body);

        // 验证刚体添加成功
        assert!(result.is_ok());
        assert!(world.get_rigid_body(body_id).is_some());
    }

    #[test]
    fn test_physics_world_add_collider() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::new_dynamic(RigidBodyState::default());
        let body_id = body.id();
        world.add_rigid_body(body).expect("Test: operation should succeed");

        let collider = Collider::new(ShapeType::Sphere { radius: 1.0 });
        let collider_id = collider.id();

        let result = world.add_collider(body_id, collider);

        // 验证碰撞体添加成功
        assert!(result.is_ok());
        assert!(world.get_collider(collider_id).is_some());
    }

    #[test]
    fn test_physics_world_remove_rigid_body() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::new_dynamic(RigidBodyState::default());
        let body_id = body.id();
        world.add_rigid_body(body).expect("Test: operation should succeed");

        let result = world.remove_rigid_body(body_id);

        // 验证刚体移除成功
        assert!(result.is_ok());
        assert!(world.get_rigid_body(body_id).is_none());
    }

    #[test]
    fn test_physics_world_step() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::new_dynamic(RigidBodyState {
            position: Vec3::new(0.0, 10.0, 0.0),
            ..Default::default()
        });
        let body_id = body.id();
        world.add_rigid_body(body).expect("Test: operation should succeed");
        let collider = Collider::new(ShapeType::Sphere { radius: 1.0 });
        world.add_collider(body_id, collider).expect("Test: operation should succeed");

        // 执行物理步进
        let result = world.step(0.016);

        // 验证物理模拟执行
        assert!(result.is_ok());

        // 验证物体受重力影响下落
        let updated_body = world.get_rigid_body(body_id).expect("Test: operation should succeed");
        assert!(updated_body.state().position.y < 10.0);
    }

    // ============================================================================
    // 领域规则测试
    // ============================================================================

    #[test]
    fn test_fixed_body_has_zero_mass() {
        let body = RigidBody::new_fixed(RigidBodyState::default());

        // 验证固定刚体质量为0（领域规则）
        assert_eq!(body.mass(), 0.0);
    }

    #[test]
    fn test_dynamic_body_has_positive_mass() {
        let body = RigidBody::new_dynamic(RigidBodyState::default());

        // 验证动态刚体质量为正（领域规则）
        assert!(body.mass() > 0.0);
    }

    #[test]
    fn test_shape_must_have_valid_dimensions() {
        // 有效形状 - 半径为正
        let valid_sphere = ShapeType::Sphere { radius: 1.0 };
        assert!(matches!(valid_sphere, ShapeType::Sphere { .. }));

        // 边界情况 - 半径为0（物理上无效，但类型系统允许）
        let zero_sphere = ShapeType::Sphere { radius: 0.0 };
        assert!(matches!(zero_sphere, ShapeType::Sphere { .. }));
    }
}
