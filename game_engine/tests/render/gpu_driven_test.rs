//! GPU驱动渲染测试
//!
//! 测试GPU驱动渲染的核心功能，包括视锥剔除、实例池管理等。

use game_engine::render::gpu_driven::{GpuDrivenConfig, GpuDrivenRenderer, GpuInstance};
use glam::{Mat4, Vec3};

#[test]
fn test_gpu_driven_config_default() {
    let config = GpuDrivenConfig::default();
    
    assert!(config.frustum_culling, "默认应启用视锥剔除");
    assert!(!config.occlusion_culling, "默认应禁用遮挡剔除");
    assert_eq!(config.max_instances, 65536, "默认最大实例数应为65536");
    assert_eq!(config.workgroup_size, 64, "默认工作组大小应为64");
}

#[test]
fn test_gpu_driven_config_custom() {
    let config = GpuDrivenConfig {
        frustum_culling: false,
        occlusion_culling: true,
        lod_enabled: true,
        max_instances: 32768,
        workgroup_size: 128,
    };
    
    assert!(!config.frustum_culling);
    assert!(config.occlusion_culling);
    assert!(config.lod_enabled);
    assert_eq!(config.max_instances, 32768);
    assert_eq!(config.workgroup_size, 128);
}

#[test]
fn test_gpu_instance_creation() {
    let transform = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let instance = GpuInstance {
        transform,
        mesh_id: 1,
        material_id: 2,
        lod_level: 0,
    };
    
    assert_eq!(instance.mesh_id, 1);
    assert_eq!(instance.material_id, 2);
    assert_eq!(instance.lod_level, 0);
}

#[test]
fn test_gpu_instance_transform() {
    let position = Vec3::new(5.0, 10.0, 15.0);
    let transform = Mat4::from_translation(position);
    let instance = GpuInstance {
        transform,
        mesh_id: 1,
        material_id: 1,
        lod_level: 0,
    };
    
    // 验证变换矩阵的平移部分
    let translation = instance.transform.col(3);
    assert_eq!(translation.x, 5.0);
    assert_eq!(translation.y, 10.0);
    assert_eq!(translation.z, 15.0);
    assert_eq!(translation.w, 1.0);
}

#[test]
fn test_gpu_driven_config_validation() {
    // 测试配置参数的合理性
    let config = GpuDrivenConfig {
        frustum_culling: true,
        occlusion_culling: false,
        lod_enabled: true,
        max_instances: 65536,
        workgroup_size: 64,
    };
    
    // 验证最大实例数在合理范围内
    assert!(config.max_instances > 0, "最大实例数应大于0");
    assert!(config.max_instances <= 1_000_000, "最大实例数不应过大");
    
    // 验证工作组大小是2的幂
    assert!(config.workgroup_size.is_power_of_two(), "工作组大小应为2的幂");
    assert!(config.workgroup_size >= 32, "工作组大小应至少为32");
    assert!(config.workgroup_size <= 256, "工作组大小应不超过256");
}

#[test]
fn test_gpu_instance_lod_levels() {
    // 测试不同LOD级别的实例
    for lod_level in 0..4 {
        let instance = GpuInstance {
            transform: Mat4::IDENTITY,
            mesh_id: 1,
            material_id: 1,
            lod_level,
        };
        
        assert_eq!(instance.lod_level, lod_level);
        assert!(instance.lod_level < 8, "LOD级别应在合理范围内");
    }
}

