use game_engine::ecs::*;
use game_engine::physics::*;
use game_engine::audio::*;
use bevy_ecs::prelude::World;
use glam::{Vec3, Quat};

#[test]
fn test_ecs_integration() {
    let mut world = World::default();
    
    // 创建实体
    let entity = world.spawn((
        Transform::default(),
        Sprite::default(),
    )).id();
    
    // 验证实体存在
    assert!(world.get::<Transform>(entity).is_some());
    assert!(world.get::<Sprite>(entity).is_some());
}

#[test]
fn test_physics_integration() {
    // 创建物理世界
    let physics = PhysicsWorld::default();
    
    // 验证物理世界初始化成功
    assert_eq!(physics.rigid_body_set.len(), 0);
}

#[test]
fn test_audio_integration() {
    // 创建音频系统
    let audio_system = AudioSystem::new();
    
    // 验证音频系统初始化成功
    assert_eq!(audio_system.master_volume, 1.0);
}

#[test]
fn test_full_game_loop() {
    // 创建ECS世界
    let mut world = World::default();
    
    // 创建游戏实体
    let player = world.spawn((
        Transform {
            pos: Vec3::new(0.0, 0.0, 0.0),
            rot: Quat::IDENTITY,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        Sprite {
            color: [1.0, 0.0, 0.0, 1.0],
            tex_index: 0,
            normal_tex_index: 0,
            uv_off: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
            layer: 0.0,
        },
    )).id();
    
    // 创建音频系统
    let mut audio_system = AudioSystem::new();
    audio_system.set_master_volume(0.8);
    
    // 验证游戏状态
    assert!(world.get::<Transform>(player).is_some());
    assert_eq!(audio_system.master_volume, 0.8);
}
