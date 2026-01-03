//! # Collision Detection Tests
//!
//! 测试碰撞检测系统的基础功能。

use game_engine::physics::{CollisionWorld, ColliderShape};
use glam::Vec3;

#[test]
fn test_collision_world_creation() {
    let world = CollisionWorld::new();

    assert_eq!(world.body_count(), 0);
}

#[test]
fn test_collision_world_add_body() {
    let mut world = CollisionWorld::new();

    let body_id = world.add_body(Vec3::ZERO, ColliderShape::Sphere { radius: 1.0 });

    assert_eq!(world.body_count(), 1);
    assert_eq!(body_id, 0);
}

#[test]
fn test_collision_world_remove_body() {
    let mut world = CollisionWorld::new();

    let body_id = world.add_body(Vec3::ZERO, ColliderShape::Sphere { radius: 1.0 });
    world.remove_body(body_id);

    assert_eq!(world.body_count(), 0);
}

#[test]
fn test_sphere_sphere_collision() {
    let mut world = CollisionWorld::new();

    // 两个球体重叠
    world.add_body(Vec3::ZERO, ColliderShape::Sphere { radius: 1.0 });
    world.add_body(Vec3::new(1.0, 0.0, 0.0), ColliderShape::Sphere { radius: 1.0 });

    let collisions = world.detect_collisions();

    assert!(!collisions.is_empty());
}

#[test]
fn test_sphere_sphere_no_collision() {
    let mut world = CollisionWorld::new();

    // 两个球体不重叠
    world.add_body(Vec3::ZERO, ColliderShape::Sphere { radius: 1.0 });
    world.add_body(Vec3::new(3.0, 0.0, 0.0), ColliderShape::Sphere { radius: 1.0 });

    let collisions = world.detect_collisions();

    assert!(collisions.is_empty());
}

#[test]
fn test_box_box_collision() {
    let mut world = CollisionWorld::new();

    // 两个盒子重叠
    world.add_body(
        Vec3::ZERO,
        ColliderShape::Box {
            half_extents: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world.add_body(
        Vec3::new(1.5, 0.0, 0.0),
        ColliderShape::Box {
            half_extents: Vec3::new(1.0, 1.0, 1.0),
        },
    );

    let collisions = world.detect_collisions();

    assert!(!collisions.is_empty());
}

#[test]
fn test_collision_response() {
    let mut world = CollisionWorld::new();

    let body1 = world.add_body(Vec3::ZERO, ColliderShape::Sphere { radius: 1.0 });
    let body2 = world.add_body(Vec3::new(1.0, 0.0, 0.0), ColliderShape::Sphere { radius: 1.0 });

    // 模拟碰撞响应
    world.resolve_collisions();

    // 碰撞后物体应该分离
    let pos1 = world.get_body_position(body1).unwrap();
    let pos2 = world.get_body_position(body2).unwrap();

    let distance = pos1.distance(pos2);
    assert!(distance >= 2.0); // 两个半径之和
}

#[test]
fn test_broad_phase_aabb() {
    let mut world = CollisionWorld::new();

    // 远处的物体不应该被检测到碰撞
    world.add_body(Vec3::ZERO, ColliderShape::Sphere { radius: 1.0 });
    world.add_body(Vec3::new(100.0, 0.0, 0.0), ColliderShape::Sphere { radius: 1.0 });

    let broad_phase_hits = world.broad_phase_query();

    assert!(broad_phase_hits.is_empty());
}

#[test]
fn test_broad_phase_nearby() {
    let mut world = CollisionWorld::new();

    // 靠近的物体应该在broad phase中被检测到
    world.add_body(Vec3::ZERO, ColliderShape::Sphere { radius: 1.0 });
    world.add_body(Vec3::new(1.5, 0.0, 0.0), ColliderShape::Sphere { radius: 1.0 });

    let broad_phase_hits = world.broad_phase_query();

    assert!(!broad_phase_hits.is_empty());
}
