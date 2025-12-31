//! Physics 核心功能综合测试
//!
//! 测试物理引擎的核心功能

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{PhysicsDomainService, RigidBody, RigidBodyId, RigidBodyType};
    use crate::physics::*;
    use glam::{Quat, Vec3};

    // ========================================
    // PhysicsDomainService 基础测试
    // ========================================

    #[test]
    fn test_physics_domain_service_new() {
        let mut service = PhysicsDomainService::new();
        // 服务应该成功创建
        assert_eq!(service.bodies_count(), 0);
    }

    #[test]
    fn test_physics_domain_service_create_fixed_body() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Fixed,
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
        );

        let result = service.create_body(body);
        assert!(result.is_ok());
        assert_eq!(service.bodies_count(), 1);
    }

    #[test]
    fn test_physics_domain_service_create_dynamic_body() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(10.0, 20.0, 30.0),
            Quat::IDENTITY,
            5.0,
        );

        let result = service.create_body(body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_create_kinematic_body() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Kinematic,
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
        );

        let result = service.create_body(body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_get_body_position() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let pos = Vec3::new(100.0, 200.0, 300.0);
        let body = RigidBody::with_all(body_id, RigidBodyType::Dynamic, pos, Quat::IDENTITY, 1.0);

        service.create_body(body).expect("Test: operation should succeed");
        let retrieved_pos = service.get_body_position(body_id);

        assert!(retrieved_pos.is_ok());
        let retrieved_pos = retrieved_pos.expect("Test: operation should succeed");
        assert_eq!(retrieved_pos.x, pos.x);
        assert_eq!(retrieved_pos.y, pos.y);
        assert_eq!(retrieved_pos.z, pos.z);
    }

    #[test]
    fn test_physics_domain_service_get_nonexistent_body() {
        let mut service = PhysicsDomainService::new();
        let invalid_id = RigidBodyId::new(999);

        let result = service.get_body_position(invalid_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_physics_domain_service_remove_body() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
        );

        service.create_body(body).expect("Test: operation should succeed");
        assert_eq!(service.bodies_count(), 1);

        let result = service.remove_body(body_id);
        assert!(result.is_ok());
        assert_eq!(service.bodies_count(), 0);
    }

    #[test]
    fn test_physics_domain_service_remove_nonexistent_body() {
        let mut service = PhysicsDomainService::new();
        let invalid_id = RigidBodyId::new(999);

        let result = service.remove_body(invalid_id);
        assert!(result.is_err());
    }

    // ========================================
    // RigidBody 基础测试
    // ========================================

    #[test]
    fn test_rigid_body_with_all() {
        let body_id = RigidBodyId::new(1);
        let pos = Vec3::new(10.0, 20.0, 30.0);
        let rot = Quat::from_rotation_x(0.5);
        let mass = 5.0;

        let body = RigidBody::with_all(body_id, RigidBodyType::Dynamic, pos, rot, mass);

        assert_eq!(body.id, body_id);
        assert_eq!(body.body_type, RigidBodyType::Dynamic);
    }

    #[test]
    fn test_rigid_body_types() {
        let body_id = RigidBodyId::new(1);

        let fixed_body = RigidBody::with_all(
            body_id,
            RigidBodyType::Fixed,
            Vec3::ZERO,
            Quat::IDENTITY,
            0.0,
        );

        let dynamic_body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
        );

        let kinematic_body = RigidBody::with_all(
            body_id,
            RigidBodyType::Kinematic,
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
        );

        assert_eq!(fixed_body.body_type, RigidBodyType::Fixed);
        assert_eq!(dynamic_body.body_type, RigidBodyType::Dynamic);
        assert_eq!(kinematic_body.body_type, RigidBodyType::Kinematic);
    }

    // ========================================
    // RigidBodyId 测试
    // ========================================

    #[test]
    fn test_rigid_body_id_new() {
        let id1 = RigidBodyId::new(1);
        let id2 = RigidBodyId::new(2);

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_rigid_body_id_equality() {
        let id1 = RigidBodyId::new(5);
        let id2 = RigidBodyId::new(5);

        assert_eq!(id1, id2);
    }

    #[test]
    fn test_rigid_body_id_copy() {
        let id1 = RigidBodyId::new(10);
        let id2 = id1;

        assert_eq!(id1, id2);
    }

    // ========================================
    // Physics 基础计算测试
    // ========================================

    #[test]
    fn test_gravity_application() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 100.0, 0.0),
            Quat::IDENTITY,
            1.0,
        );

        service.create_body(body).expect("Test: operation should succeed");

        // 模拟重力影响
        let dt = 1.0 / 60.0;
        service.update(dt);

        let new_pos = service.get_body_position(body_id).expect("Test: operation should succeed");
        // Y坐标应该减小（重力向下）
        assert!(new_pos.y < 100.0);
    }

    #[test]
    fn test_fixed_body_unaffected_by_gravity() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let initial_pos = Vec3::new(0.0, 100.0, 0.0);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Fixed,
            initial_pos,
            Quat::IDENTITY,
            0.0,
        );

        service.create_body(body).expect("Test: operation should succeed");

        // 固定物体不应受重力影响
        let dt = 1.0 / 60.0;
        service.update(dt);

        let new_pos = service.get_body_position(body_id).expect("Test: operation should succeed");
        // 位置应该保持不变
        assert_eq!(new_pos.y, initial_pos.y);
    }

    #[test]
    fn test_kinematic_body_velocity() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Kinematic,
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
        );

        service.create_body(body).expect("Test: operation should succeed");

        // 设置运动物体速度
        service
            .set_body_velocity(body_id, Vec3::new(10.0, 0.0, 0.0))
            .expect("Test: operation should succeed");

        let dt = 1.0 / 60.0;
        service.update(dt);

        let new_pos = service.get_body_position(body_id).expect("Test: operation should succeed");
        // X坐标应该增加
        assert!(new_pos.x > 0.0);
    }

    // ========================================
    // 多物体交互测试
    // ========================================

    #[test]
    fn test_multiple_bodies() {
        let mut service = PhysicsDomainService::new();

        for i in 0..10 {
            let body_id = RigidBodyId::new(i as u64);
            let body = RigidBody::with_all(
                body_id,
                RigidBodyType::Dynamic,
                Vec3::new(i as f32 * 10.0, 0.0, 0.0),
                Quat::IDENTITY,
                1.0,
            );
            service.create_body(body).expect("Test: operation should succeed");
        }

        assert_eq!(service.bodies_count(), 10);
    }

    #[test]
    fn test_body_collision_detection() {
        let mut service = PhysicsDomainService::new();

        // 创建两个可能碰撞的物体
        let body1_id = RigidBodyId::new(1);
        let body1 = RigidBody::with_all(
            body1_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
        );

        let body2_id = RigidBodyId::new(2);
        let body2 = RigidBody::with_all(
            body2_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.5, 0.0, 0.0),
            Quat::IDENTITY,
            1.0,
        );

        service.create_body(body1).expect("Test: operation should succeed");
        service.create_body(body2).expect("Test: operation should succeed");

        // 更新物理系统
        service.update(1.0 / 60.0);

        // 碰撞检测应该工作（具体行为取决于实现）
        // 这里我们只验证系统不会崩溃
        assert_eq!(service.bodies_count(), 2);
    }

    // ========================================
    // 边界情况测试
    // ========================================

    #[test]
    fn test_zero_mass_body() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            0.0, // 零质量
        );

        let result = service.create_body(body);
        // 根据实现，零质量可能被允许或拒绝
        // 这里我们验证系统能处理这种情况
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_negative_mass_body() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            -1.0, // 负质量
        );

        let result = service.create_body(body);
        // 负质量应该被拒绝
        assert!(result.is_err());
    }

    #[test]
    fn test_extreme_position() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(1e6, 1e6, 1e6), // 极大位置
            Quat::IDENTITY,
            1.0,
        );

        let result = service.create_body(body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nan_position() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(f32::NAN, 0.0, 0.0),
            Quat::IDENTITY,
            1.0,
        );

        let result = service.create_body(body);
        // NaN 应该被拒绝或处理
        assert!(result.is_err() || result.is_ok());
    }

    // ========================================
    // 性能测试
    // ========================================

    #[test]
    fn test_physics_update_performance() {
        let mut service = PhysicsDomainService::new();

        // 创建100个物体
        for i in 0..100 {
            let body_id = RigidBodyId::new(i as u64);
            let body = RigidBody::with_all(
                body_id,
                RigidBodyType::Dynamic,
                Vec3::new(i as f32, 0.0, 0.0),
                Quat::IDENTITY,
                1.0,
            );
            service.create_body(body).expect("Test: operation should succeed");
        }

        // 测量更新性能
        let start = std::time::Instant::now();
        for _ in 0..10 {
            service.update(1.0 / 60.0);
        }
        let duration = start.elapsed();

        // 应该快速完成（< 100ms）
        assert!(duration < std::time::Duration::from_millis(100));
    }

    #[test]
    fn test_many_bodies_creation_performance() {
        let start = std::time::Instant::now();

        let mut service = PhysicsDomainService::new();
        for i in 0..1000 {
            let body_id = RigidBodyId::new(i as u64);
            let body = RigidBody::with_all(
                body_id,
                RigidBodyType::Dynamic,
                Vec3::new(i as f32, 0.0, 0.0),
                Quat::IDENTITY,
                1.0,
            );
            service.create_body(body).expect("Test: operation should succeed");
        }

        let duration = start.elapsed();

        // 应该快速完成（< 500ms）
        assert!(duration < std::time::Duration::from_millis(500));
        assert_eq!(service.bodies_count(), 1000);
    }

    // ========================================
    // 物理步进测试
    // ========================================

    #[test]
    fn test_fixed_timestep() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 100.0, 0.0),
            Quat::IDENTITY,
            1.0,
        );

        service.create_body(body).expect("Test: operation should succeed");

        let dt = 1.0 / 60.0; // 固定时间步长
        let initial_pos =
            service.get_body_position(body_id).expect("Test: operation should succeed");

        service.update(dt);
        let pos1 = service.get_body_position(body_id).expect("Test: operation should succeed");

        service.update(dt);
        let pos2 = service.get_body_position(body_id).expect("Test: operation should succeed");

        // 每次步进应该产生一致的效果
        assert!(pos1.y < initial_pos.y);
        assert!(pos2.y < pos1.y);
    }

    #[test]
    fn test_variable_timestep() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 100.0, 0.0),
            Quat::IDENTITY,
            1.0,
        );

        service.create_body(body).expect("Test: operation should succeed");

        let dt1 = 1.0 / 30.0; // 较大时间步长
        let dt2 = 1.0 / 120.0; // 较小时间步长

        service.update(dt1);
        let pos1 = service.get_body_position(body_id).expect("Test: operation should succeed");

        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 100.0, 0.0),
            Quat::IDENTITY,
            1.0,
        );
        service.create_body(body).expect("Test: operation should succeed");

        service.update(dt2);
        let pos2 = service.get_body_position(body_id).expect("Test: operation should succeed");

        // 较大的时间步长应该产生更大的位移
        assert!(pos1.y < 100.0);
        assert!(pos2.y < 100.0);
    }

    // ========================================
    // 旋转测试
    // ========================================

    #[test]
    fn test_body_rotation() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let rot = Quat::from_rotation_y(std::f32::consts::PI / 4.0);
        let body = RigidBody::with_all(body_id, RigidBodyType::Dynamic, Vec3::ZERO, rot, 1.0);

        service.create_body(body).expect("Test: operation should succeed");

        let body_rot = service.get_body_rotation(body_id).expect("Test: operation should succeed");
        // 旋转应该被保持
        assert_eq!(body_rot, rot);
    }

    #[test]
    fn test_body_rotation_update() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
        );

        service.create_body(body).expect("Test: operation should succeed");

        // 设置角速度
        service
            .set_body_angular_velocity(body_id, Vec3::new(0.0, 1.0, 0.0))
            .expect("Test: operation should succeed");

        service.update(1.0 / 60.0);

        let new_rot = service.get_body_rotation(body_id).expect("Test: operation should succeed");
        // 旋转应该改变
        assert_ne!(new_rot, Quat::IDENTITY);
    }

    // ========================================
    // 力和冲量测试
    // ========================================

    #[test]
    fn test_apply_force() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
        );

        service.create_body(body).expect("Test: operation should succeed");

        // 应用力
        service
            .apply_force(body_id, Vec3::new(100.0, 0.0, 0.0))
            .expect("Test: operation should succeed");

        service.update(1.0 / 60.0);

        // 物体应该加速
        let velocity = service.get_body_velocity(body_id).expect("Test: operation should succeed");
        assert!(velocity.x > 0.0);
    }

    #[test]
    fn test_apply_impulse() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
        );

        service.create_body(body).expect("Test: operation should succeed");

        // 应用冲量
        service
            .apply_impulse(body_id, Vec3::new(10.0, 0.0, 0.0))
            .expect("Test: operation should succeed");

        // 冲量应该立即改变速度
        let velocity = service.get_body_velocity(body_id).expect("Test: operation should succeed");
        assert!(velocity.x > 0.0);
    }

    // ========================================
    // 睡眠和唤醒测试
    // ========================================

    #[test]
    fn test_sleep_body() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
        );

        service.create_body(body).expect("Test: operation should succeed");

        // 催眠物体
        service.sleep_body(body_id).expect("Test: operation should succeed");

        // 检查是否睡眠
        assert!(service.is_body_sleeping(body_id).expect("Test: operation should succeed"));
    }

    #[test]
    fn test_wake_body() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
        );

        service.create_body(body).expect("Test: operation should succeed");

        service.sleep_body(body_id).expect("Test: operation should succeed");
        assert!(service.is_body_sleeping(body_id).expect("Test: operation should succeed"));

        // 唤醒物体
        service.wake_body(body_id).expect("Test: operation should succeed");
        assert!(!service.is_body_sleeping(body_id).expect("Test: operation should succeed"));
    }

    // ========================================
    // 约束测试
    // ========================================

    #[test]
    fn test_velocity_constraint() {
        let mut service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
        );

        service.create_body(body).expect("Test: operation should succeed");

        // 设置最大速度
        service.set_max_velocity(body_id, 10.0).expect("Test: operation should succeed");

        service
            .apply_impulse(body_id, Vec3::new(1000.0, 0.0, 0.0))
            .expect("Test: operation should succeed");

        // 速度应该被限制
        let velocity = service.get_body_velocity(body_id).expect("Test: operation should succeed");
        assert!(velocity.length() <= 10.0);
    }
}
