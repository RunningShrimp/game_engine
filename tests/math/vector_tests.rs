//! # Math Library Tests (GLAM)
//!
//! 测试GLAM数学库的基础功能。

use glam::{Vec2, Vec3, Vec4, BVec2, BVec3, BVec4};

#[test]
fn test_vec2_creation() {
    let v = Vec2::new(1.0, 2.0);

    assert_eq!(v.x, 1.0);
    assert_eq!(v.y, 2.0);
}

#[test]
fn test_vec2_zero() {
    let v = Vec2::ZERO;

    assert_eq!(v.x, 0.0);
    assert_eq!(v.y, 0.0);
}

#[test]
fn test_vec2_one() {
    let v = Vec2::ONE;

    assert_eq!(v.x, 1.0);
    assert_eq!(v.y, 1.0);
}

#[test]
fn test_vec2_add() {
    let v1 = Vec2::new(1.0, 2.0);
    let v2 = Vec2::new(3.0, 4.0);

    let result = v1 + v2;

    assert_eq!(result.x, 4.0);
    assert_eq!(result.y, 6.0);
}

#[test]
fn test_vec2_sub() {
    let v1 = Vec2::new(5.0, 7.0);
    let v2 = Vec2::new(2.0, 3.0);

    let result = v1 - v2;

    assert_eq!(result.x, 3.0);
    assert_eq!(result.y, 4.0);
}

#[test]
fn test_vec2_mul_scalar() {
    let v = Vec2::new(2.0, 3.0);

    let result = v * 2.0;

    assert_eq!(result.x, 4.0);
    assert_eq!(result.y, 6.0);
}

#[test]
fn test_vec2_dot() {
    let v1 = Vec2::new(1.0, 2.0);
    let v2 = Vec2::new(3.0, 4.0);

    let result = v1.dot(v2);

    assert_eq!(result, 11.0); // 1*3 + 2*4 = 3 + 8 = 11
}

#[test]
fn test_vec2_length() {
    let v = Vec2::new(3.0, 4.0);

    let length = v.length();

    assert_eq!(length, 5.0);
}

#[test]
fn test_vec2_normalize() {
    let v = Vec2::new(3.0, 4.0);

    let normalized = v.normalize();

    assert!((normalized.length() - 1.0).abs() < 1e-6);
}

#[test]
fn test_vec2_distance() {
    let v1 = Vec2::new(0.0, 0.0);
    let v2 = Vec2::new(3.0, 4.0);

    let distance = v1.distance(v2);

    assert_eq!(distance, 5.0);
}

#[test]
fn test_vec3_creation() {
    let v = Vec3::new(1.0, 2.0, 3.0);

    assert_eq!(v.x, 1.0);
    assert_eq!(v.y, 2.0);
    assert_eq!(v.z, 3.0);
}

#[test]
fn test_vec3_cross() {
    let v1 = Vec3::X;
    let v2 = Vec3::Y;

    let result = v1.cross(v2);

    assert_eq!(result, Vec3::Z);
}

#[test]
fn test_vec3_dot() {
    let v1 = Vec3::new(1.0, 2.0, 3.0);
    let v2 = Vec3::new(4.0, 5.0, 6.0);

    let result = v1.dot(v2);

    assert_eq!(result, 32.0); // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
}

#[test]
fn test_vec3_lerp() {
    let v1 = Vec3::ZERO;
    let v2 = Vec3::ONE;

    let result = v1.lerp(v2, 0.5);

    assert_eq!(result, Vec3::splat(0.5));
}

#[test]
fn test_vec4_creation() {
    let v = Vec4::new(1.0, 2.0, 3.0, 4.0);

    assert_eq!(v.x, 1.0);
    assert_eq!(v.y, 2.0);
    assert_eq!(v.z, 3.0);
    assert_eq!(v.w, 4.0);
}

#[test]
fn test_bvec2_any() {
    let b = BVec2::new(true, false);

    assert!(b.any());
    assert!(!b.all());
}

#[test]
fn test_bvec3_all() {
    let b = BVec3::new(true, true, true);

    assert!(b.all());
    assert!(b.any());
}

#[test]
fn test_bvec4_none() {
    let b = BVec4::new(false, false, false, false);

    assert!(!b.any());
    assert!(!b.all());
}
