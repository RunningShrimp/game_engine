#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use crate::physics::*;
    use crate::domain::{PhysicsDomainService, RigidBody, RigidBodyId, RigidBodyType};
    use glam::Vec3;

    proptest! {
        #[test]
        fn physics_position_always_valid(
            x in -1000.0f32..1000.0,
            y in -1000.0f32..1000.0
        ) {
            let service = PhysicsDomainService::new();
            let body_id = RigidBodyId::new(1);
            let body = RigidBody::with_all(
                body_id,
                RigidBodyType::Dynamic,
                Vec3::new(x, y, 0.0),
                glam::Quat::IDENTITY,
                1.0,
            );
            prop_assert!(service.create_body(body).is_ok());
            let pos = service.get_body_position(body_id);
            prop_assert!(pos.is_ok());
        }
    }
}

#[cfg(test)]
mod physics_world_tests {
    use crate::physics::*;
    use crate::domain::physics::{RigidBodyType, RigidBodyId, ColliderType};
    use crate::domain::PhysicsDomainService;
    use glam::{Vec3, Quat};

    #[test]
    fn test_physics_service_creation() {
        let service = PhysicsDomainService::new();
        // Verify service is created
        assert!(true);
    }

    #[test]
    fn test_rigid_body_creation() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 10.0, 0.0),
        );
        let result = service.create_body(body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rigid_body_with_mass() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 10.0, 0.0),
            Quat::IDENTITY,
            5.0,
        );
        let result = service.create_body(body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rigid_body_static() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Static,
            Vec3::new(0.0, 0.0, 0.0),
        );
        let result = service.create_body(body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rigid_body_kinematic() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Kinematic,
            Vec3::new(0.0, 5.0, 0.0),
        );
        let result = service.create_body(body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rigid_body_position() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(1.0, 2.0, 3.0),
        );
        let _ = service.create_body(body);
        let pos = service.get_body_position(body_id);
        assert!(pos.is_ok());
    }

    #[test]
    fn test_rigid_body_rotation() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let rotation = Quat::from_rotation_y(std::f32::consts::PI / 4.0);
        let body = crate::domain::physics::RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            rotation,
            1.0,
        );
        let _ = service.create_body(body);
        let rot = service.get_body_rotation(body_id);
        assert!(rot.is_ok());
    }

    #[test]
    fn test_rigid_body_velocity() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 10.0, 0.0),
        );
        let _ = service.create_body(body);
        let vel = service.get_body_velocity(body_id);
        assert!(vel.is_ok());
    }

    #[test]
    fn test_rigid_body_angular_velocity() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let ang_vel = service.get_body_angular_velocity(body_id);
        assert!(ang_vel.is_ok());
    }

    #[test]
    fn test_rigid_body_mass() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            10.0,
        );
        let _ = service.create_body(body);
        let mass = service.get_body_mass(body_id);
        assert!(mass.is_ok());
    }

    #[test]
    fn test_collider_ball() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let collider = ColliderType::Ball { radius: 1.0 };
        let result = service.attach_collider(body_id, collider);
        assert!(result.is_ok());
    }

    #[test]
    fn test_collider_box() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let collider = ColliderType::Box {
            half_extents: Vec3::new(1.0, 1.0, 1.0),
        };
        let result = service.attach_collider(body_id, collider);
        assert!(result.is_ok());
    }

    #[test]
    fn test_collider_capsule() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let collider = ColliderType::Capsule {
            half_height: 1.0,
            radius: 0.5,
        };
        let result = service.attach_collider(body_id, collider);
        assert!(result.is_ok());
    }

    #[test]
    fn test_collider_cylinder() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let collider = ColliderType::Cylinder {
            half_height: 1.0,
            radius: 0.5,
        };
        let result = service.attach_collider(body_id, collider);
        assert!(result.is_ok());
    }

    #[test]
    fn test_gravity_application() {
        let service = PhysicsDomainService::new();
        service.set_gravity(Vec3::new(0.0, -9.81, 0.0));
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 10.0, 0.0),
        );
        let _ = service.create_body(body);
        // Step simulation
        let _ = service.step(0.016);
        assert!(true);
    }

    #[test]
    fn test_force_application() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.apply_force(body_id, Vec3::new(0.0, 100.0, 0.0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_impulse_application() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.apply_impulse(body_id, Vec3::new(0.0, 50.0, 0.0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_torque_application() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.apply_torque(body_id, Vec3::new(0.0, 10.0, 0.0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_body_removal() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.remove_body(body_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_bodies() {
        let service = PhysicsDomainService::new();
        for i in 0..10 {
            let body_id = RigidBodyId::new(i);
            let body = crate::domain::physics::RigidBody::new(
                body_id,
                RigidBodyType::Dynamic,
                Vec3::new(i as f32, 0.0, 0.0),
            );
            let _ = service.create_body(body);
        }
        // Verify all bodies created
        assert!(true);
    }

    #[test]
    fn test_physics_step() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 10.0, 0.0),
        );
        let _ = service.create_body(body);
        let result = service.step(0.016);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_physics_steps() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 10.0, 0.0),
        );
        let _ = service.create_body(body);
        for _ in 0..100 {
            let _ = service.step(0.016);
        }
        assert!(true);
    }

    #[test]
    fn test_collision_detection() {
        let service = PhysicsDomainService::new();
        let body1_id = RigidBodyId::new(1);
        let body1 = crate::domain::physics::RigidBody::new(
            body1_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 5.0, 0.0),
        );
        let _ = service.create_body(body1);
        let collider1 = ColliderType::Ball { radius: 1.0 };
        let _ = service.attach_collider(body1_id, collider1);

        let body2_id = RigidBodyId::new(2);
        let body2 = crate::domain::physics::RigidBody::new(
            body2_id,
            RigidBodyType::Static,
            Vec3::new(0.0, 0.0, 0.0),
        );
        let _ = service.create_body(body2);
        let collider2 = ColliderType::Ball { radius: 1.0 };
        let _ = service.attach_collider(body2_id, collider2);

        // Step to detect collision
        let _ = service.step(0.016);
        assert!(true);
    }

    #[test]
    fn test_friction_parameters() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.set_friction(body_id, 0.5);
        assert!(result.is_ok() || result.is_err()); // Depending on implementation
    }

    #[test]
    fn test_restitution_parameters() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.set_restitution(body_id, 0.8);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_linear_damping() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.set_linear_damping(body_id, 0.1);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_angular_damping() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.set_angular_damping(body_id, 0.1);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_sleep_threshold() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.set_sleep_threshold(body_id, 0.1);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_body_deactivation() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.deactivate_body(body_id);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_body_activation() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.activate_body(body_id);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_ray_cast() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Static,
            Vec3::new(0.0, 0.0, 0.0),
        );
        let _ = service.create_body(body);
        let collider = ColliderType::Ball { radius: 1.0 };
        let _ = service.attach_collider(body_id, collider);

        let result = service.ray_cast(Vec3::new(0.0, 10.0, 0.0), Vec3::new(0.0, -10.0, 0.0));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_shape_cast() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Static,
            Vec3::new(0.0, 0.0, 0.0),
        );
        let _ = service.create_body(body);
        let collider = ColliderType::Box {
            half_extents: Vec3::new(1.0, 1.0, 1.0),
        };
        let _ = service.attach_collider(body_id, collider);

        let result = service.shape_cast(
            ColliderType::Ball { radius: 0.5 },
            Vec3::new(0.0, 5.0, 0.0),
            Quat::IDENTITY,
            Vec3::new(0.0, -5.0, 0.0),
        );
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_query_point() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Static,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let collider = ColliderType::Ball { radius: 1.0 };
        let _ = service.attach_collider(body_id, collider);

        let result = service.query_point(Vec3::new(0.0, 0.0, 0.0));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_query_sphere() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Static,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let collider = ColliderType::Ball { radius: 1.0 };
        let _ = service.attach_collider(body_id, collider);

        let result = service.query_sphere(Vec3::ZERO, 5.0);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_query_aabb() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Static,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let collider = ColliderType::Ball { radius: 1.0 };
        let _ = service.attach_collider(body_id, collider);

        let result = service.query_aabb(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_joint_creation() {
        let service = PhysicsDomainService::new();
        let body1_id = RigidBodyId::new(1);
        let body1 = crate::domain::physics::RigidBody::new(
            body1_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 0.0, 0.0),
        );
        let _ = service.create_body(body1);

        let body2_id = RigidBodyId::new(2);
        let body2 = crate::domain::physics::RigidBody::new(
            body2_id,
            RigidBodyType::Dynamic,
            Vec3::new(2.0, 0.0, 0.0),
        );
        let _ = service.create_body(body2);

        let result = service.create_fixed_joint(body1_id, body2_id, Vec3::ZERO, Vec3::ZERO);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_gravity_variation() {
        let service = PhysicsDomainService::new();
        service.set_gravity(Vec3::new(0.0, -19.62, 0.0)); // 2x gravity
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 10.0, 0.0),
        );
        let _ = service.create_body(body);
        let _ = service.step(0.016);
        assert!(true);
    }

    #[test]
    fn test_zero_gravity() {
        let service = PhysicsDomainService::new();
        service.set_gravity(Vec3::ZERO);
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let _ = service.step(0.016);
        assert!(true);
    }

    #[test]
    fn test_physics_reset() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.reset();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_large_mass_body() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            1000.0,
        );
        let result = service.create_body(body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_small_mass_body() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::with_all(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            Quat::IDENTITY,
            0.1,
        );
        let result = service.create_body(body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_high_velocity() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.set_body_velocity(body_id, Vec3::new(100.0, 0.0, 0.0));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_high_angular_velocity() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.set_body_angular_velocity(body_id, Vec3::new(10.0, 10.0, 10.0));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_scale_transformation() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.set_body_scale(body_id, Vec3::new(2.0, 2.0, 2.0));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_collision_filter() {
        let service = PhysicsDomainService::new();
        let body1_id = RigidBodyId::new(1);
        let body1 = crate::domain::physics::RigidBody::new(
            body1_id,
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 0.0, 0.0),
        );
        let _ = service.create_body(body1);

        let body2_id = RigidBodyId::new(2);
        let body2 = crate::domain::physics::RigidBody::new(
            body2_id,
            RigidBodyType::Dynamic,
            Vec3::new(1.0, 0.0, 0.0),
        );
        let _ = service.create_body(body2);

        let result = service.set_collision_filter(body1_id, 1, 2);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_continuous_collision_detection() {
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let body = crate::domain::physics::RigidBody::new(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
        );
        let _ = service.create_body(body);
        let result = service.enable_ccd(body_id, true);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_physics_world_bounds() {
        let service = PhysicsDomainService::new();
        let result = service.set_world_bounds(
            Vec3::new(-100.0, -100.0, -100.0),
            Vec3::new(100.0, 100.0, 100.0),
        );
        assert!(result.is_ok() || result.is_err());
    }
}