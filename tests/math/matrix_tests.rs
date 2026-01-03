//! # Math Matrix Tests
//!
//! 测试矩阵运算的基础功能。

use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4, Quat};

#[test]
fn test_mat2_identity() {
    let m = Mat2::IDENTITY;

    assert_eq!(m.x_axis, Vec2::X);
    assert_eq!(m.y_axis, Vec2::Y);
}

#[test]
fn test_mat2_determinant() {
    let m = Mat2::from_cols_array(&[1.0, 2.0, 3.0, 4.0]);

    let det = m.determinant();

    assert_eq!(det, -2.0); // 1*4 - 2*3 = 4 - 6 = -2
}

#[test]
fn test_mat2_mul() {
    let m1 = Mat2::from_cols_array(&[1.0, 2.0, 3.0, 4.0]);
    let m2 = Mat2::from_cols_array(&[5.0, 6.0, 7.0, 8.0]);

    let result = m1 * m2;

    let expected = Mat2::from_cols_array(&[19.0, 22.0, 43.0, 50.0]);

    assert_eq!(result, expected);
}

#[test]
fn test_mat2_inverse() {
    let m = Mat2::from_cols_array(&[4.0, 7.0, 2.0, 6.0]);

    let inv = m.inverse();

    let result = m * inv;

    assert!(result.abs_diff_eq(Mat2::IDENTITY, 1e-5));
}

#[test]
fn test_mat3_identity() {
    let m = Mat3::IDENTITY;

    assert_eq!(m.x_axis, Vec3::X);
    assert_eq!(m.y_axis, Vec3::Y);
    assert_eq!(m.z_axis, Vec3::Z);
}

#[test]
fn test_mat3_determinant() {
    let m = Mat3::from_cols_array(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);

    let det = m.determinant();

    assert_eq!(det, 0.0); // 矩阵的行是线性相关的
}

#[test]
fn test_mat3_from_axis_angle() {
    let axis = Vec3::Y;
    let angle = std::f32::consts::PI / 2.0;

    let m = Mat3::from_axis_angle(axis, angle);

    let v = Vec3::X;
    let rotated = m * v;

    assert!(rotated.abs_diff_eq(Vec3::Z, 1e-5));
}

#[test]
fn test_mat3_mul_vec3() {
    let m = Mat3::from_scale(Vec3::splat(2.0));
    let v = Vec3::new(1.0, 2.0, 3.0);

    let result = m * v;

    assert_eq!(result, Vec3::new(2.0, 4.0, 6.0));
}

#[test]
fn test_mat4_identity() {
    let m = Mat4::IDENTITY;

    assert_eq!(m.x_axis, Vec4::X);
    assert_eq!(m.y_axis, Vec4::Y);
    assert_eq!(m.z_axis, Vec4::Z);
    assert_eq!(m.w_axis, Vec4::W);
}

#[test]
fn test_mat4_from_translation() {
    let translation = Vec3::new(1.0, 2.0, 3.0);
    let m = Mat4::from_translation(translation);

    let point = Vec3::ZERO;
    let transformed = m.transform_point3(point);

    assert_eq!(transformed, translation);
}

#[test]
fn test_mat4_from_scale() {
    let scale = Vec3::new(2.0, 3.0, 4.0);
    let m = Mat4::from_scale(scale);

    let point = Vec3::new(1.0, 1.0, 1.0);
    let transformed = m.transform_point3(point);

    assert_eq!(transformed, scale);
}

#[test]
fn test_mat4_from_rotation() {
    let axis = Vec3::Y;
    let angle = std::f32::consts::PI;
    let m = Mat4::from_axis_angle(axis, angle);

    let point = Vec3::new(1.0, 0.0, 0.0);
    let transformed = m.transform_point3(point);

    assert!(transformed.abs_diff_eq(Vec3::new(-1.0, 0.0, 0.0), 1e-5));
}

#[test]
fn test_mat4_from_quat() {
    let quat = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);
    let m = Mat4::from_quat(quat);

    let point = Vec3::X;
    let transformed = m.transform_point3(point);

    assert!(transformed.abs_diff_eq(Vec3::Z, 1e-5));
}

#[test]
fn test_mat4_trs() {
    let translation = Vec3::new(1.0, 2.0, 3.0);
    let rotation = Quat::IDENTITY;
    let scale = Vec3::splat(2.0);

    let m = Mat4::from_scale_rotation_translation(scale, rotation, translation);

    let point = Vec3::ZERO;
    let transformed = m.transform_point3(point);

    assert_eq!(transformed, translation);
}

#[test]
fn test_mat4_mul_mat4() {
    let m1 = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
    let m2 = Mat4::from_scale(Vec3::splat(2.0));

    let result = m1 * m2;

    let point = Vec3::new(1.0, 0.0, 0.0);
    let transformed = result.transform_point3(point);

    assert_eq!(transformed, Vec3::new(3.0, 0.0, 0.0)); // (1+1)*2
}

#[test]
fn test_mat4_determinant() {
    let m = Mat4::IDENTITY;

    assert_eq!(m.determinant(), 1.0);
}

#[test]
fn test_mat4_inverse() {
    let m = Mat4::from_scale_rotation_translation(
        Vec3::new(2.0, 3.0, 4.0),
        Quat::IDENTITY,
        Vec3::new(5.0, 6.0, 7.0),
    );

    let inv = m.inverse();
    let result = m * inv;

    assert!(result.abs_diff_eq(Mat4::IDENTITY, 1e-5));
}

#[test]
fn test_mat4_transpose() {
    let m = Mat4::from_cols_array(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0]);

    let transposed = m.transpose();

    assert_eq!(transposed.x_axis, Vec4::new(1.0, 5.0, 9.0, 13.0));
    assert_eq!(transposed.y_axis, Vec4::new(2.0, 6.0, 10.0, 14.0));
}
