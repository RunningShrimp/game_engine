// 测试 PhysicsWorld 的 Send/Sync 实现

use game_engine::domain::physics::PhysicsWorld;

fn main() {
    // 测试 PhysicsWorld 是否实现了 Send
    fn assert_send_physics_world<T: Send>() {}
    assert_send_physics_world::<PhysicsWorld>();
    
    // 测试 PhysicsWorld 是否实现了 Sync
    fn assert_sync_physics_world<T: Sync>() {}
    assert_sync_physics_world::<PhysicsWorld>();
    
    println!("PhysicsWorld Send/Sync tests passed!");
}