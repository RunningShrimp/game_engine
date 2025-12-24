use crate::domain::physics::{Collider, ColliderId, PhysicsWorld, RigidBody, RigidBodyId, RigidBodyType, ShapeType};
use crate::domain::errors::{CompensationAction, DomainError, PhysicsError, RecoveryStrategy};
use glam::{Quat, Vec3};

#[cfg(test)]
mod rigid_body_id_tests {
    use super::*;

    #[test]
    fn test_rigid_body_id_creation() {
        let id = RigidBodyId::new(42);
        assert_eq!(id.as_u64(), 42);
    }

    #[test]
    fn test_rigid_body_id_equality() {
        let id1 = RigidBodyId::new(42);
        let id2 = RigidBodyId::new(42);
        let id3 = RigidBodyId::new(24);
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_rigid_body_id_hash() {
        use std::collections::HashSet;
        
        let id1 = RigidBodyId::new(42);
        let id2 = RigidBodyId::new(42);
        let id3 = RigidBodyId::new(24);
        
        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2);
        set.insert(id3);
        
        assert_eq!(set.len(), 2);
    }
}

#[cfg(test)]
mod collider_id_tests {
    use super::*;

    #[test]
    fn test_collider_id_creation() {
        let id = ColliderId::new(42);
        assert_eq!(id.as_u64(), 42);
    }

    #[test]
    fn test_collider_id_equality() {
        let id1 = ColliderId::new(42);
        let id2 = ColliderId::new(42);
        let id3 = ColliderId::new(24);
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_collider_id_hash() {
        use std::collections::HashSet;
        
        let id1 = ColliderId::new(42);
        let id2 = ColliderId::new(42);
        let id3 = ColliderId::new(24);
        
        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2);
        set.insert(id3);
        
        assert_eq!(set.len(), 2);
    }
}

#[cfg(test)]
mod rigid_body_type_tests {
    use super::*;

    #[test]
    fn test_rigid_body_type_values() {
        let types = vec![RigidBodyType::Fixed, RigidBodyType::Dynamic, RigidBodyType::Kinematic];
        assert_eq!(types.len(), 3);
    }

    #[test]
    fn test_rigid_body_type_clone() {
        let rbt = RigidBodyType::Dynamic;
        let cloned = rbt.clone();
        assert_eq!(rbt, cloned);
    }
}

#[cfg(test)]
mod shape_type_tests {
    use super::*;

    #[test]
    fn test_shape_type_sphere() {
        let shape = ShapeType::Sphere { radius: 1.0 };
        if let ShapeType::Sphere { radius } = shape {
            assert_eq!(radius, 1.0);
        } else {
            panic!("Expected Sphere shape");
        }
    }

    #[test]
    fn test_shape_type_cuboid() {
        let half_extents = Vec3::new(1.0, 2.0, 3.0);
        let shape = ShapeType::Cuboid { half_extents };
        if let ShapeType::Cuboid { half_extents: he } = shape {
            assert_eq!(he, half_extents);
        } else {
            panic!("Expected Cuboid shape");
        }
    }

    #[test]
    fn test_shape_type_capsule() {
        let shape = ShapeType::Capsule { radius: 0.5, height: 2.0 };
        if let ShapeType::Capsule { radius, height } = shape {
            assert_eq!(radius, 0.5);
            assert_eq!(height, 2.0);
        } else {
            panic!("Expected Capsule shape");
        }
    }

    #[test]
    fn test_shape_type_clone() {
        let shape = ShapeType::Sphere { radius: 1.0 };
        let cloned = shape.clone();
        assert_eq!(shape, cloned);
    }
}

#[cfg(test)]
mod rigid_body_creation_tests {
    use super::*;

    #[test]
    fn test_rigid_body_new() {
        let body = RigidBody::new(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(1.0, 2.0, 3.0),
        );
        
        assert_eq!(body.id(), RigidBodyId::new(1));
        assert_eq!(body.body_type(), RigidBodyType::Dynamic);
        assert_eq!(body.position(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(body.rotation(), Quat::IDENTITY);
        assert_eq!(body.mass(), 1.0);
    }

    #[test]
    fn test_rigid_body_with_all() {
        let rotation = Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 4.0);
        let body = RigidBody::with_all(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(1.0, 2.0, 3.0),
            rotation,
            5.0,
        );
        
        assert_eq!(body.id(), RigidBodyId::new(1));
        assert_eq!(body.body_type(), RigidBodyType::Dynamic);
        assert_eq!(body.position(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(body.rotation(), rotation);
        assert_eq!(body.mass(), 5.0);
    }

    #[test]
    fn test_rigid_body_dynamic() {
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        assert_eq!(body.id(), RigidBodyId::new(1));
        assert_eq!(body.body_type(), RigidBodyType::Dynamic);
        assert_eq!(body.position(), Vec3::ZERO);
    }

    #[test]
    fn test_rigid_body_fixed() {
        let body = RigidBody::new(
            RigidBodyId::new(1),
            RigidBodyType::Fixed,
            Vec3::ZERO,
        );
        
        assert_eq!(body.body_type(), RigidBodyType::Fixed);
    }

    #[test]
    fn test_rigid_body_kinematic() {
        let body = RigidBody::new(
            RigidBodyId::new(1),
            RigidBodyType::Kinematic,
            Vec3::ZERO,
        );
        
        assert_eq!(body.body_type(), RigidBodyType::Kinematic);
    }
}

#[cfg(test)]
mod rigid_body_property_tests {
    use super::*;

    #[test]
    fn test_rigid_body_getters() {
        let body = RigidBody::new(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(1.0, 2.0, 3.0),
        );
        
        assert_eq!(body.id(), RigidBodyId::new(1));
        assert_eq!(body.body_type(), RigidBodyType::Dynamic);
        assert_eq!(body.position(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(body.rotation(), Quat::IDENTITY);
        assert_eq!(body.linear_velocity(), Vec3::ZERO);
        assert_eq!(body.angular_velocity(), Vec3::ZERO);
        assert_eq!(body.mass(), 1.0);
        assert_eq!(body.friction(), 0.5);
        assert_eq!(body.restitution(), 0.3);
    }

    #[test]
    fn test_rigid_body_set_mass_valid() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let result = body.set_mass(5.0);
        
        assert!(result.is_ok());
        assert_eq!(body.mass(), 5.0);
    }

    #[test]
    fn test_rigid_body_set_mass_invalid() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let result = body.set_mass(0.0);
        
        assert!(result.is_err());
        assert_eq!(body.mass(), 1.0);
    }

    #[test]
    fn test_rigid_body_set_mass_negative() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let result = body.set_mass(-1.0);
        
        assert!(result.is_err());
        assert_eq!(body.mass(), 1.0);
    }

    #[test]
    fn test_rigid_body_set_position() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        body.set_position(Vec3::new(10.0, 20.0, 30.0));
        
        assert_eq!(body.position(), Vec3::new(10.0, 20.0, 30.0));
    }

    #[test]
    fn test_rigid_body_set_rotation() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let rotation = Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0);
        body.set_rotation(rotation);
        
        assert_eq!(body.rotation(), rotation);
    }

    #[test]
    fn test_rigid_body_set_linear_velocity() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let velocity = Vec3::new(1.0, 2.0, 3.0);
        body.set_linear_velocity(velocity);
        
        assert_eq!(body.linear_velocity(), velocity);
    }

    #[test]
    fn test_rigid_body_set_angular_velocity() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let velocity = Vec3::new(0.5, 1.0, 1.5);
        body.set_angular_velocity(velocity);
        
        assert_eq!(body.angular_velocity(), velocity);
    }

    #[test]
    fn test_rigid_body_set_friction() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        body.set_friction(0.8);
        
        assert_eq!(body.friction(), 0.8);
    }

    #[test]
    fn test_rigid_body_set_restitution() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        body.set_restitution(0.9);
        
        assert_eq!(body.restitution(), 0.9);
    }
}

#[cfg(test)]
mod rigid_body_error_recovery_tests {
    use super::*;

    #[test]
    fn test_rigid_body_recover_from_invalid_parameter_with_retry() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let error = PhysicsError::InvalidParameter("Invalid mass".to_string());
        
        let result = body.recover_from_error(&error);
        
        assert!(result.is_ok());
        assert_eq!(body.mass(), 1.0);
        assert_eq!(body.position(), Vec3::ZERO);
    }

    #[test]
    fn test_rigid_body_recover_with_use_default() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::new(10.0, 20.0, 30.0));
        body.set_linear_velocity(Vec3::new(1.0, 2.0, 3.0));
        body.set_angular_velocity(Vec3::new(0.5, 1.0, 1.5));
        body.recovery_strategy = RecoveryStrategy::UseDefault;
        
        let error = PhysicsError::InvalidParameter("Test error".to_string());
        let result = body.recover_from_error(&error);
        
        assert!(result.is_ok());
        assert_eq!(body.mass(), 1.0);
        assert_eq!(body.linear_velocity(), Vec3::ZERO);
        assert_eq!(body.angular_velocity(), Vec3::ZERO);
    }

    #[test]
    fn test_rigid_body_recover_with_skip() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::new(10.0, 20.0, 30.0));
        body.recovery_strategy = RecoveryStrategy::Skip;
        
        let error = PhysicsError::InvalidParameter("Test error".to_string());
        let result = body.recover_from_error(&error);
        
        assert!(result.is_ok());
        assert_eq!(body.position(), Vec3::new(10.0, 20.0, 30.0));
    }

    #[test]
    fn test_rigid_body_recover_with_fail() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        body.recovery_strategy = RecoveryStrategy::Fail;
        
        let error = PhysicsError::InvalidParameter("Test error".to_string());
        let result = body.recover_from_error(&error);
        
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod rigid_body_compensation_tests {
    use super::*;

    #[test]
    fn test_rigid_body_create_compensation() {
        let body = RigidBody::with_all(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 4.0),
            5.0,
        );
        body.set_linear_velocity(Vec3::new(1.0, 2.0, 3.0));
        body.set_angular_velocity(Vec3::new(0.5, 1.0, 1.5));
        
        let compensation = body.create_compensation();
        
        assert_eq!(compensation.resource_id, "rigid_body_1");
        assert_eq!(compensation.action, "restore_physics_state");
        
        let pos = compensation.data.get("position").and_then(|v| v.as_array());
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().len(), 3);
    }

    #[test]
    fn test_rigid_body_restore_from_compensation() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        let compensation = CompensationAction::new(
            "rigid_body_1".to_string(),
            "restore_physics_state".to_string(),
            serde_json::json!({
                "position": [10.0, 20.0, 30.0],
                "rotation": [0.0, 0.707, 0.0, 0.707],
                "linear_velocity": [1.0, 2.0, 3.0],
                "angular_velocity": [0.5, 1.0, 1.5],
                "mass": 5.0,
                "sleeping": false
            }),
        );
        
        let result = body.restore_from_compensation(&compensation);
        
        assert!(result.is_ok());
        assert_eq!(body.position(), Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(body.linear_velocity(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(body.angular_velocity(), Vec3::new(0.5, 1.0, 1.5));
        assert_eq!(body.mass(), 5.0);
    }

    #[test]
    fn test_rigid_body_restore_roundtrip() {
        let mut body = RigidBody::with_all(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 4.0),
            5.0,
        );
        body.set_linear_velocity(Vec3::new(1.0, 2.0, 3.0));
        body.set_angular_velocity(Vec3::new(0.5, 1.0, 1.5));
        
        let original_pos = body.position();
        let original_vel = body.linear_velocity();
        
        let compensation = body.create_compensation();
        
        body.set_position(Vec3::ZERO);
        body.set_linear_velocity(Vec3::ZERO);
        
        body.restore_from_compensation(&compensation).unwrap();
        
        assert_eq!(body.position(), original_pos);
        assert_eq!(body.linear_velocity(), original_vel);
    }
}

#[cfg(test)]
mod collider_tests {
    use super::*;

    #[test]
    fn test_collider_new() {
        let collider = Collider::new(
            ColliderId::new(1),
            RigidBodyId::new(10),
            ShapeType::Sphere { radius: 1.0 },
            1.0,
        );
        
        assert_eq!(collider.id(), ColliderId::new(1));
        assert_eq!(collider.body_id(), RigidBodyId::new(10));
        assert_eq!(collider.density(), 1.0);
    }

    #[test]
    fn test_collider_cuboid() {
        let collider = Collider::cuboid(ColliderId::new(1), Vec3::new(1.0, 2.0, 3.0));
        
        assert_eq!(collider.id(), ColliderId::new(1));
        assert_eq!(collider.half_extents(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_collider_ball() {
        let collider = Collider::ball(ColliderId::new(1), 2.0);
        
        assert_eq!(collider.id(), ColliderId::new(1));
        assert_eq!(collider.radius(), 2.0);
    }

    #[test]
    fn test_collider_getters() {
        let collider = Collider::new(
            ColliderId::new(1),
            RigidBodyId::new(10),
            ShapeType::Sphere { radius: 1.0 },
            1.0,
        );
        
        assert_eq!(collider.id(), ColliderId::new(1));
        assert_eq!(collider.body_id(), RigidBodyId::new(10));
        assert_eq!(collider.density(), 1.0);
        assert_eq!(collider.friction(), 0.5);
        assert_eq!(collider.restitution(), 0.3);
    }

    #[test]
    fn test_collider_set_friction() {
        let mut collider = Collider::cuboid(ColliderId::new(1), Vec3::ONE);
        collider.set_friction(0.8);
        
        assert_eq!(collider.friction(), 0.8);
    }

    #[test]
    fn test_collider_set_restitution() {
        let mut collider = Collider::cuboid(ColliderId::new(1), Vec3::ONE);
        collider.set_restitution(0.9);
        
        assert_eq!(collider.restitution(), 0.9);
    }
}

#[cfg(test)]
mod physics_world_tests {
    use super::*;

    #[test]
    fn test_physics_world_new() {
        let world = PhysicsWorld::new();
        
        assert_eq!(world.get_world().bodies.len(), 0);
    }

    #[test]
    fn test_physics_world_add_body() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        let result = world.add_body(body);
        
        assert!(result.is_ok());
        assert_eq!(world.get_world().bodies.len(), 1);
    }

    #[test]
    fn test_physics_world_add_duplicate_body() {
        let mut world = PhysicsWorld::new();
        let body1 = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let body2 = RigidBody::dynamic(RigidBodyId::new(1), Vec3::new(1.0, 0.0, 0.0));
        
        world.add_body(body1).unwrap();
        let result = world.add_body(body2);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_physics_world_get_body() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        world.add_body(body).unwrap();
        
        let retrieved = world.get_body(RigidBodyId::new(1));
        
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_physics_world_get_body_mut() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        world.add_body(body).unwrap();
        
        let retrieved = world.get_body_mut(RigidBodyId::new(1));
        
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_physics_world_remove_body() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        world.add_body(body).unwrap();
        
        let result = world.remove_body(RigidBodyId::new(1));
        
        assert!(result.is_ok());
        assert_eq!(world.get_world().bodies.len(), 0);
    }

    #[test]
    fn test_physics_world_remove_nonexistent_body() {
        let mut world = PhysicsWorld::new();
        
        let result = world.remove_body(RigidBodyId::new(1));
        
        assert!(result.is_err());
    }

    #[test]
    fn test_physics_world_step() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::new(0.0, 10.0, 0.0));
        world.add_body(body).unwrap();
        
        let result = world.step(0.016);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_world_get_body_state() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::new(1.0, 2.0, 3.0));
        world.add_body(body).unwrap();
        
        let state = world.get_body_state(RigidBodyId::new(1));
        
        assert!(state.is_some());
        assert_eq!(state.unwrap().position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_physics_world_raycast() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::new(0.0, 0.0, 0.0));
        world.add_body(body).unwrap();
        
        let result = world.raycast(Vec3::new(-10.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_world_apply_impulse() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        world.add_body(body).unwrap();
        
        let result = world.apply_impulse(RigidBodyId::new(1), Vec3::new(0.0, 100.0, 0.0));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_world_set_body_position() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        world.add_body(body).unwrap();
        
        let result = world.set_body_position(RigidBodyId::new(1), Vec3::new(10.0, 20.0, 30.0));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_world_get_body_position() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::new(1.0, 2.0, 3.0));
        world.add_body(body).unwrap();
        
        let result = world.get_body_position(RigidBodyId::new(1));
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec3::new(1.0, 2.0, 3.0));
    }
}

#[cfg(test)]
mod physics_edge_cases_tests {
    use super::*;

    #[test]
    fn test_rigid_body_extreme_positions() {
        let extreme_pos = Vec3::new(1000000.0, 1000000.0, 1000000.0);
        let body = RigidBody::dynamic(RigidBodyId::new(1), extreme_pos);
        
        assert_eq!(body.position(), extreme_pos);
    }

    #[test]
    fn test_rigid_body_zero_mass_error() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let result = body.set_mass(0.0);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_rigid_body_negative_mass_error() {
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let result = body.set_mass(-1.0);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_collider_extreme_friction() {
        let mut collider = Collider::cuboid(ColliderId::new(1), Vec3::ONE);
        collider.set_friction(10.0);
        
        assert_eq!(collider.friction(), 10.0);
    }

    #[test]
    fn test_collider_extreme_restitution() {
        let mut collider = Collider::cuboid(ColliderId::new(1), Vec3::ONE);
        collider.set_restitution(2.0);
        
        assert_eq!(collider.restitution(), 2.0);
    }

    #[test]
    fn test_physics_world_multiple_bodies() {
        let mut world = PhysicsWorld::new();
        
        for i in 0..100 {
            let body = RigidBody::dynamic(RigidBodyId::new(i), Vec3::new(i as f32, 0.0, 0.0));
            world.add_body(body).unwrap();
        }
        
        assert_eq!(world.get_world().bodies.len(), 100);
    }

    #[test]
    fn test_physics_world_step_with_zero_delta() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        world.add_body(body).unwrap();
        
        let result = world.step(0.0);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_world_step_with_negative_delta() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        world.add_body(body).unwrap();
        
        let result = world.step(-0.016);
        
        assert!(result.is_err());
    }
}
