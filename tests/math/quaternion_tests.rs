//! # Quaternion Tests
//!
//! 测试四元数的基础功能。

use glam::{Quat, Vec3, Vec4};

#[test]
fn test_quat_identity() {
    let q = Quat::IDENTITY;

    assert_eq!(q.x, 0.0);
    assert_eq!(q.y, 0.0);
    assert_eq!(q.z, 0.0);
    assert_eq!(q.w, 1.0);
}

#[test]
fn test_quat_from_axis_angle() {
    let axis = Vec3::Y;
    let angle = std::f32::consts::FRAC_PI_2; // 90度

    let q = Quat::from_axis_angle(axis, angle);

    // 四元数应该是归一化的
    assert!((q.length() - 1.0).abs() < 1e-5);
}

#[test]
fn test_quat_from_euler() {
    let euler = Vec3::new(std::f32::consts::FRAC_PI_4, 0.0, 0.0); // 45度X轴旋转

    let q = Quat::from_euler(glam::EulerRot::XYZ, euler.x, euler.y, euler.z);

    // 四元数应该是归一化的
    assert!((q.length() - 1.0).abs() < 1e-5);
}

#[test]
fn test_quat_mul() {
    let q1 = Quat::from_axis_angle(Vec3::X, std::f32::consts::FRAC_PI_2);
    let q2 = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);

    let result = q1 * q2;

    // 四元数乘积应该归一化
    assert!((result.length() - 1.0).abs() < 1e-5);
}

#[test]
fn test_quat_rotate_vec3() {
    let axis = Vec3::Y;
    let angle = std::f32::consts::FRAC_PI_2; // 90度
    let q = Quat::from_axis_angle(axis, angle);

    let v = Vec3::X;
    let rotated = q.mul_vec3(v);

    // X轴绕Y轴旋转90度应该得到Z轴
    assert!(rotated.abs_diff_eq(Vec3::Z, 1e-5));
}

#[test]
fn test_quat_conjugate() {
    let q = Quat::from_xyzw(1.0, 2.0, 3.0, 4.0).normalize();
    let conj = q.conjugate();

    assert_eq!(conj.x, -q.x);
    assert_eq!(conj.y, -q.y);
    assert_eq!(conj.z, -q.z);
    assert_eq!(conj.w, q.w);
}

#[test]
fn test_quat_inverse() {
    let q = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_4);
    let inv = q.inverse();

    let result = q * inv;

    assert!(result.abs_diff_eq(Quat::IDENTITY, 1e-5));
}

#[test]
fn test_quat_slerp() {
    let q1 = Quat::IDENTITY;
    let q2 = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);

    let result = q1.slerp(q2, 0.5);

    // SLERP结果应该归一化
    assert!((result.length() - 1.0).abs() < 1e-5);
}

#[test]
fn test_quat_lerp() {
    let q1 = Quat::IDENTITY;
    let q2 = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);

    let result = q1.lerp(q2, 0.5);

    // LERP结果需要归一化
    let normalized = result.normalize();
    assert!((normalized.length() - 1.0).abs() < 1e-5);
}

#[test]
fn test_quat_dot() {
    let q1 = Quat::IDENTITY;
    let q2 = Quat::IDENTITY;

    let dot = q1.dot(q2);

    assert_eq!(dot, 1.0);
}

#[test]
fn test_quat_length_squared() {
    let q = Quat::IDENTITY;

    assert_eq!(q.length_squared(), 1.0);
}

#[test]
fn test_quat_length() {
    let q = Quat::IDENTITY;

    assert_eq!(q.length(), 1.0);
}

#[test]
fn test_quat_normalize() {
    let q = Quat::from_xyzw(1.0, 2.0, 3.0, 4.0);
    let normalized = q.normalize();

    assert!((normalized.length() - 1.0).abs() < 1e-5);
}

#[test]
fn test_quat_from_rotation_mat4() {
    let axis = Vec3::Y;
    let angle = std::f32::consts::FRAC_PI_4;

    let q1 = Quat::from_axis_angle(axis, angle);
    let mat4 = glam::Mat4::from_quat(q1);
    let q2 = Quat::from_rotation_mat4(&mat4);

    // 两个四元数应该相等或相反(表示相同的旋转)
    let dot = q1.dot(q2);
    assert!(dot.abs() > 0.99); // 允许一些浮点误差
}

#[test]
fn test_quat_to_axis_angle() {
    let axis = Vec3::Y.normalize();
    let angle = std::f32::consts::FRAC_PI_3; // 60度

    let q = Quat::from_axis_angle(axis, angle);
    let (out_axis, out_angle) = q.to_axis_angle();

    assert!(out_axis.abs_diff_eq(axis, 1e-5));
    assert!((out_angle - angle).abs() < 1e-5);
}
