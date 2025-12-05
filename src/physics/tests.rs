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
            let mut service = PhysicsDomainService::new();
            let body_id = RigidBodyId::new(1);
            let body = RigidBody::new(
                body_id,
                RigidBodyType::Dynamic,
                Vec3::new(x, y, 0.0),
                glam::Quat::IDENTITY,
            );
            prop_assert!(service.create_body(body).is_ok());
            let pos = service.get_body_position(body_id);
            prop_assert!(pos.is_ok());
        }
    }
}