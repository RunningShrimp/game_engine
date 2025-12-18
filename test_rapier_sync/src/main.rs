// 测试 Rapier3D 类型的 Send/Sync 实现

use rapier3d::prelude::*;

fn main() {
    // 测试 PhysicsPipeline 是否实现了 Send
    fn assert_send_physics_pipeline<T: Send>() {}
    assert_send_physics_pipeline::<PhysicsPipeline>();
    
    // 测试 PhysicsPipeline 是否实现了 Sync
    fn assert_sync_physics_pipeline<T: Sync>() {}
    assert_sync_physics_pipeline::<PhysicsPipeline>();
    
    // 测试其他关键类型
    fn assert_send_island_manager<T: Send>() {}
    assert_send_island_manager::<IslandManager>();
    
    fn assert_sync_island_manager<T: Sync>() {}
    assert_sync_island_manager::<IslandManager>();
    
    fn assert_send_broad_phase<T: Send>() {}
    assert_send_broad_phase::<DefaultBroadPhase>();
    
    fn assert_sync_broad_phase<T: Sync>() {}
    assert_sync_broad_phase::<DefaultBroadPhase>();
    
    fn assert_send_narrow_phase<T: Send>() {}
    assert_send_narrow_phase::<NarrowPhase>();
    
    fn assert_sync_narrow_phase<T: Sync>() {}
    assert_sync_narrow_phase::<NarrowPhase>();
    
    println!("All tests passed!");
}