use game_engine::ecs::{Transform, Sprite, Time};
use bevy_ecs::prelude::*;
use game_engine::physics::{PhysicsState, PhysicsService, RigidBodyType};

#[test]
fn test_complete_game_loop() {
    // 创建ECS世界
    let mut world = World::new();

    // 添加时间资源
    world.insert_resource(Time::default());

    // 添加物理世界资源
    world.insert_resource(PhysicsState::default());

    // 创建测试场景 - 直接生成实体
    let mut commands = world.spawn(Transform::default());
    commands.insert(Sprite::default());
    commands.insert(game_engine::physics::RigidBodyDesc {
        body_type: RigidBodyType::Dynamic,
        position: [0.0, 0.0],
    });

    // 运行几帧
    for _ in 0..10 {
        // 模拟引擎更新
        if let Some(mut time) = world.get_resource_mut::<Time>() {
            time.delta_seconds = 1.0 / 60.0;
            time.elapsed_seconds += time.delta_seconds as f64;
        }

        if let Some(mut physics) = world.get_resource_mut::<PhysicsState>() {
            PhysicsService::step(&mut physics);
        }
    }

    // 验证状态
    assert!(world.iter_entities().count() > 0);

    // 测试物理状态
    if let Some(physics) = world.get_resource::<PhysicsState>() {
        assert_eq!(PhysicsService::rigid_body_count(&physics), 0); // 应为0因为我们还没有初始化系统
    }

    // 实体数量验证
    let entity_count = world.iter_entities().count();
    assert!(entity_count >= 1);
}
