//! # Rigid Body Tests
//!
//! 测试刚体物理系统的基础功能。

use game_engine::physics::RigidBody;
use game_engine::physics::ColliderShape;
use glam::Vec3;

#[test]
fn test_rigidbody_creation() {
    let body = RigidBody::new();

    assert_eq!(body.position(), Vec3::ZERO);
    assert_eq!(body.velocity(), Vec3::ZERO);
    assert_eq!(body.mass(), 1.0);
}

#[test]
fn test_rigidbody_set_position() {
    let mut body = RigidBody::new();
    body.set_position(Vec3::new(1.0, 2.0, 3.0));

    assert_eq!(body.position(), Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn test_rigidbody_set_velocity() {
    let mut body = RigidBody::new();
    body.set_velocity(Vec3::new(5.0, 0.0, 0.0));

    assert_eq!(body.velocity(), Vec3::new(5.0, 0.0, 0.0));
}

#[test]
fn test_rigidbody_set_mass() {
    let mut body = RigidBody::new();
    body.set_mass(10.0);

    assert_eq!(body.mass(), 10.0);
}

#[test]
fn test_rigidbody_zero_mass() {
    let mut body = RigidBody::new();
    body.set_mass(0.0);

    // 质量为0应该是静态物体
    assert!(body.is_static());
}

#[test]
fn test_rigidbody_infinite_mass() {
    let mut body = RigidBody::new();
    body.set_mass(f32::INFINITY);

    // 无限质量应该是静态物体
    assert!(body.is_static());
}

#[test]
fn test_rigidbody_apply_force() {
    let mut body = RigidBody::new();
    body.set_mass(2.0);
    body.apply_force(Vec3::new(10.0, 0.0, 0.0));

    // F = ma -> a = F/m = 10/2 = 5
    // 一个时间步后速度应该增加
    body.update(0.016);

    assert!(body.velocity().x > 0.0);
}

#[test]
fn test_rigidbody_apply_impulse() {
    let mut body = RigidBody::new();
    body.set_mass(2.0);
    body.apply_impulse(Vec3::new(5.0, 0.0, 0.0));

    // 冲量直接改变速度: Δv = J/m = 5/2 = 2.5
    assert_eq!(body.velocity().x, 2.5);
}

#[test]
fn test_rigidbody_gravity() {
    let mut body = RigidBody::new();
    body.set_position(Vec3::new(0.0, 10.0, 0.0));

    // 应用力: 重力
    let gravity = Vec3::new(0.0, -9.81, 0.0);
    body.apply_force(gravity);

    body.update(0.016);

    // 物体应该向下移动
    assert!(body.position().y < 10.0);
    assert!(body.velocity().y < 0.0);
}

#[test]
fn test_rigidbody_set_collider() {
    let mut body = RigidBody::new();
    let collider = ColliderShape::Sphere { radius: 1.0 };

    body.set_collider(collider);

    assert!(body.collider().is_some());
}

#[test]
fn test_rigidbody_friction() {
    let mut body = RigidBody::new();
    body.set_friction(0.5);

    assert_eq!(body.friction(), 0.5);
}

#[test]
fn test_rigidbody_restitution() {
    let mut body = RigidBody::new();
    body.set_restitution(0.8);

    assert_eq!(body.restitution(), 0.8);
}

#[test]
fn test_rigidbody_restitution_clamp() {
    let mut body = RigidBody::new();
    body.set_restitution(1.5); // 应该被钳制到[0, 1]

    assert!(body.restitution() >= 0.0 && body.restitution() <= 1.0);
}
