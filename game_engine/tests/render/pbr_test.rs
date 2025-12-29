//! PBR渲染测试
//!
//! 测试PBR渲染系统的核心功能，包括材质、光照、场景构建等。

use game_engine::render::pbr_renderer::Instance3D;
use glam::{Mat4, Vec3};

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_instance3d_vertex_layout() {
    // 测试Instance3D的顶点布局描述
    let layout = Instance3D::desc();
    
    assert_eq!(layout.array_stride, std::mem::size_of::<Instance3D>() as u64);
    assert_eq!(layout.step_mode, wgpu::VertexStepMode::Instance);
    assert_eq!(layout.attributes.len(), 4, "Model矩阵应占用4个属性槽位");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_instance3d_transform_matrix() {
    // 测试变换矩阵的正确性
    let translation = Vec3::new(10.0, 20.0, 30.0);
    let rotation = glam::Quat::IDENTITY;
    let scale = Vec3::ONE;
    
    let transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);
    let instance = Instance3D {
        model: transform.to_cols_array_2d(),
    };
    
    // 验证变换矩阵的平移部分
    let reconstructed = Mat4::from_cols_array_2d(&instance.model);
    let extracted_translation = reconstructed.col(3);
    
    assert!((extracted_translation.x - 10.0).abs() < 0.001);
    assert!((extracted_translation.y - 20.0).abs() < 0.001);
    assert!((extracted_translation.z - 30.0).abs() < 0.001);
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_instance3d_identity_matrix() {
    // 测试单位矩阵
    let instance = Instance3D {
        model: Mat4::IDENTITY.to_cols_array_2d(),
    };
    
    let reconstructed = Mat4::from_cols_array_2d(&instance.model);
    assert!(reconstructed.abs_diff_eq(&Mat4::IDENTITY, 0.001));
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_instance3d_rotation() {
    // 测试旋转变换
    let rotation = glam::Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0);
    let transform = Mat4::from_quat(rotation);
    
    let instance = Instance3D {
        model: transform.to_cols_array_2d(),
    };
    
    let reconstructed = Mat4::from_cols_array_2d(&instance.model);
    
    // 验证旋转后的X轴方向（应该变成Z轴方向）
    let rotated_x = reconstructed.col(0);
    assert!((rotated_x.z - 1.0).abs() < 0.001, "绕Y轴旋转90度后，X轴应指向Z方向");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_instance3d_scale() {
    // 测试缩放变换
    let scale = Vec3::new(2.0, 3.0, 4.0);
    let transform = Mat4::from_scale(scale);
    
    let instance = Instance3D {
        model: transform.to_cols_array_2d(),
    };
    
    let reconstructed = Mat4::from_cols_array_2d(&instance.model);
    
    // 验证缩放值
    let scale_x = reconstructed.col(0).length();
    let scale_y = reconstructed.col(1).length();
    let scale_z = reconstructed.col(2).length();
    
    assert!((scale_x - 2.0).abs() < 0.001);
    assert!((scale_y - 3.0).abs() < 0.001);
    assert!((scale_z - 4.0).abs() < 0.001);
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_instance3d_combined_transform() {
    // 测试组合变换（平移+旋转+缩放）
    let translation = Vec3::new(1.0, 2.0, 3.0);
    let rotation = glam::Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 4.0);
    let scale = Vec3::new(2.0, 2.0, 2.0);
    
    let transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);
    let instance = Instance3D {
        model: transform.to_cols_array_2d(),
    };
    
    let reconstructed = Mat4::from_cols_array_2d(&instance.model);
    
    // 验证平移
    let extracted_translation = reconstructed.col(3);
    assert!((extracted_translation.x - 1.0).abs() < 0.001);
    assert!((extracted_translation.y - 2.0).abs() < 0.001);
    assert!((extracted_translation.z - 3.0).abs() < 0.001);
    
    // 验证缩放（通过列向量的长度）
    let scale_x = reconstructed.col(0).length();
    assert!((scale_x - 2.0).abs() < 0.001);
}

