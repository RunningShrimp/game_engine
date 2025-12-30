// 测试 Rapier3D 类型的 Send/Sync 实现

use rapier3d::prelude::*;

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_rapier_physics_pipeline_send_sync() {
    // 测试 PhysicsPipeline 是否实现了 Send
    fn assert_send_physics_pipeline<T: Send>() {}
    assert_send_physics_pipeline::<PhysicsPipeline>();

    // 测试 PhysicsPipeline 是否实现了 Sync
    fn assert_sync_physics_pipeline<T: Sync>() {}
    assert_sync_physics_pipeline::<PhysicsPipeline>();
}
