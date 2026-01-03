//! # Collider Tests
//!
//! 测试碰撞体系统的基础功能。

use game_engine::physics::ColliderShape;
use glam::Vec3;

#[test]
fn test_sphere_collider_creation() {
    let collider = ColliderShape::Sphere { radius: 1.0 };

    if let ColliderShape::Sphere { radius } = collider {
        assert_eq!(radius, 1.0);
    } else {
        panic!("Expected sphere collider");
    }
}

#[test]
fn test_box_collider_creation() {
    let collider = ColliderShape::Box {
        half_extents: Vec3::new(1.0, 2.0, 3.0),
    };

    if let ColliderShape::Box { half_extents } = collider {
        assert_eq!(half_extents, Vec3::new(1.0, 2.0, 3.0));
    } else {
        panic!("Expected box collider");
    }
}

#[test]
fn test_capsule_collider_creation() {
    let collider = ColliderShape::Capsule {
        height: 2.0,
        radius: 0.5,
    };

    if let ColliderShape::Capsule { height, radius } = collider {
        assert_eq!(height, 2.0);
        assert_eq!(radius, 0.5);
    } else {
        panic!("Expected capsule collider");
    }
}

#[test]
fn test_collider_volume_sphere() {
    let collider = ColliderShape::Sphere { radius: 1.0 };

    // 球体体积 = (4/3)πr³
    let volume = collider.volume();

    assert!((volume - 4.18879).abs() < 0.001);
}

#[test]
fn test_collider_volume_box() {
    let collider = ColliderShape::Box {
        half_extents: Vec3::new(1.0, 1.0, 1.0),
    };

    // 盒子体积 = 8 * x * y * z (因为half_extents)
    let volume = collider.volume();

    assert!((volume - 8.0).abs() < 0.001);
}

#[test]
fn test_collider_bounds_sphere() {
    let collider = ColliderShape::Sphere { radius: 1.0 };

    let bounds = collider.bounds(Vec3::ZERO);

    assert_eq!(bounds.min, Vec3::new(-1.0, -1.0, -1.0));
    assert_eq!(bounds.max, Vec3::new(1.0, 1.0, 1.0));
}

#[test]
fn test_collider_bounds_box() {
    let collider = ColliderShape::Box {
        half_extents: Vec3::new(0.5, 1.0, 1.5),
    };

    let bounds = collider.bounds(Vec3::new(1.0, 2.0, 3.0));

    assert_eq!(bounds.min, Vec3::new(0.5, 1.0, 1.5));
    assert_eq!(bounds.max, Vec3::new(1.5, 3.0, 4.5));
}

#[test]
fn test_collider_clone() {
    let collider = ColliderShape::Sphere { radius: 1.0 };
    let cloned = collider.clone();

    if let ColliderShape::Sphere { radius } = cloned {
        assert_eq!(radius, 1.0);
    } else {
        panic!("Cloned collider has wrong type");
    }
}
