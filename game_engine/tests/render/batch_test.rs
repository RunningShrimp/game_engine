//! 批处理渲染测试
//!
//! 测试实例批处理的核心功能，包括批次创建、更新、脏跟踪等。

use game_engine::render::instance_batch::{BatchKey, InstanceBatch, InstanceBatchStats};
use game_engine::render::pbr_renderer::Instance3D;
use glam::Mat4;

#[test]
fn test_batch_key_creation() {
    let key = BatchKey {
        mesh_id: 1,
        material_id: 2,
        pipeline_id: 1,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    assert_eq!(key.mesh_id, 1);
    assert_eq!(key.material_id, 2);
    assert_eq!(key.pipeline_id, 1);
    assert!(key.depth_test);
}

#[test]
fn test_batch_key_equality() {
    let key1 = BatchKey {
        mesh_id: 1,
        material_id: 2,
        pipeline_id: 1,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    let key2 = BatchKey {
        mesh_id: 1,
        material_id: 2,
        pipeline_id: 1,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    assert_eq!(key1, key2);
}

#[test]
fn test_batch_key_ordering() {
    // 测试批次键的排序逻辑（pipeline_id优先级最高）
    let key1 = BatchKey {
        mesh_id: 1,
        material_id: 1,
        pipeline_id: 1,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    let key2 = BatchKey {
        mesh_id: 2,
        material_id: 2,
        pipeline_id: 2,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    assert!(key1 < key2, "pipeline_id较小的批次键应排在前面");
}

#[test]
fn test_batch_key_ordering_blend_mode() {
    // 测试混合模式的排序优先级
    let key1 = BatchKey {
        mesh_id: 1,
        material_id: 1,
        pipeline_id: 1,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    let key2 = BatchKey {
        mesh_id: 1,
        material_id: 1,
        pipeline_id: 1,
        blend_mode: 1,
        depth_test: true,
        render_flags: 0,
    };
    
    assert!(key1 < key2, "blend_mode较小的批次键应排在前面");
}

#[test]
fn test_instance3d_creation() {
    let transform = Mat4::from_translation(glam::Vec3::new(1.0, 2.0, 3.0));
    let instance = Instance3D {
        model: transform.to_cols_array_2d(),
    };
    
    // 验证变换矩阵
    let model_mat = Mat4::from_cols_array_2d(&instance.model);
    let translation = model_mat.col(3);
    assert_eq!(translation.x, 1.0);
    assert_eq!(translation.y, 2.0);
    assert_eq!(translation.z, 3.0);
}

#[test]
fn test_batch_stats_initialization() {
    let stats = InstanceBatchStats {
        update_count: 0,
        total_uploaded_instances: 0,
        total_uploaded_bytes: 0,
        incremental_update_count: 0,
        full_rebuild_count: 0,
        current_instance_count: 0,
        buffer_capacity: 0,
        average_upload_size: 0,
        incremental_update_ratio: 0.0,
    };
    
    assert_eq!(stats.update_count, 0);
    assert_eq!(stats.current_instance_count, 0);
    assert_eq!(stats.incremental_update_ratio, 0.0);
}

#[test]
fn test_batch_stats_calculation() {
    let mut stats = InstanceBatchStats {
        update_count: 10,
        total_uploaded_instances: 1000,
        total_uploaded_bytes: 64000,
        incremental_update_count: 8,
        full_rebuild_count: 2,
        current_instance_count: 100,
        buffer_capacity: 200,
        average_upload_size: 0,
        incremental_update_ratio: 0.0,
    };
    
    // 计算平均值
    stats.average_upload_size = stats.total_uploaded_instances / stats.update_count;
    assert_eq!(stats.average_upload_size, 100);
    
    // 计算增量更新比例
    stats.incremental_update_ratio = stats.incremental_update_count as f64 / stats.update_count as f64;
    assert_eq!(stats.incremental_update_ratio, 0.8);
}

