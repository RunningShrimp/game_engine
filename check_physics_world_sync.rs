#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_world_send_sync() {
        // 测试PhysicsWorld是否实现了Send
        fn assert_send<T: Send>() {}
        assert_send::<PhysicsWorld>();
        
        // 测试PhysicsWorld是否实现了Sync
        fn assert_sync<T: Sync>() {}
        assert_sync::<PhysicsWorld>();
    }
}