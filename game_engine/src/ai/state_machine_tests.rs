//! 状态机系统测试
//!
//! 测试有限状态机的功能。

#[cfg(test)]
mod tests {
    use super::super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_idle_state_lifecycle() {
        let mut state = IdleState;
        state.enter();
        assert_eq!(state.update(0.016), None);
        state.exit();
    }

    #[test]
    fn test_event_creation() {
        let event = Event {
            name: "test_event".to_string(),
            data: Some("test_data".to_string()),
        };
        assert_eq!(event.name, "test_event");
        assert_eq!(event.data, Some("test_data".to_string()));
    }

    #[test]
    fn test_transition_creation() {
        let transition = Transition {
            condition: "health_low".to_string(),
            target_state: "flee".to_string(),
        };
        assert_eq!(transition.condition, "health_low");
        assert_eq!(transition.target_state, "flee");
    }

    #[test]
    fn test_state_trait_object() {
        // Test that State trait works as trait object
        let mut state: Box<dyn State> = Box::new(IdleState);
        state.enter();
        state.update(0.016);
        state.exit();
    }

    #[test]
    fn test_multiple_state_updates() {
        let mut state = IdleState;
        state.enter();
        for _ in 0..10 {
            assert_eq!(state.update(0.016), None);
        }
        state.exit();
    }

    #[test]
    fn test_state_trait_send_sync() {
        // Verify that State trait is Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<IdleState>();
    }
}
