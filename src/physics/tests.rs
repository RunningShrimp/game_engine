#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use crate::physics::*;

    proptest! {
        #[test]
        fn physics_position_always_valid(
            x in -1000.0f32..1000.0,
            y in -1000.0f32..1000.0
        ) {
            let mut state = PhysicsState::default();
            let handle = PhysicsService::create_rigid_body(
                &mut state,
                RigidBodyType::Dynamic,
                [x, y]
            );
            let pos = PhysicsService::get_rigid_body_position(&state, handle);
            prop_assert!(pos.is_some());
        }
    }
}