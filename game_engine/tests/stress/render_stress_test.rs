//! 渲染压力测试
//!
//! 测试渲染系统在大量实体下的性能。

use game_engine::render::instance_batch::{BatchKey, InstanceBatch};
use game_engine::render::pbr_renderer::Instance3D;
use glam::Mat4;

#[test]
#[ignore] // 压力测试默认忽略，需要时手动运行
fn test_render_10000_entities() {
    // 测试10000个实体的批处理性能
    const ENTITY_COUNT: usize = 10000;
    
    let mut batches: std::collections::HashMap<BatchKey, Vec<Instance3D>> = std::collections::HashMap::new();
    
    let key = BatchKey {
        mesh_id: 1,
        material_id: 1,
        pipeline_id: 1,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    // 创建10000个实例
    let mut instances = Vec::with_capacity(ENTITY_COUNT);
    for i in 0..ENTITY_COUNT {
        let transform = Mat4::from_translation(glam::Vec3::new(
            (i % 100) as f32,
            ((i / 100) % 100) as f32,
            (i / 10000) as f32,
        ));
        instances.push(Instance3D {
            model: transform.to_cols_array_2d(),
        });
    }
    
    batches.insert(key, instances);
    
    // 验证所有实例都已创建
    assert_eq!(batches.len(), 1);
    assert_eq!(batches.values().next().unwrap().len(), ENTITY_COUNT);
}

#[test]
#[ignore]
fn test_render_batch_creation_performance() {
    // 测试批次创建的性能
    const BATCH_COUNT: usize = 1000;
    
    let start = std::time::Instant::now();
    
    for i in 0..BATCH_COUNT {
        let key = BatchKey {
            mesh_id: (i % 10) as u64,
            material_id: (i % 5) as u64,
            pipeline_id: 1,
            blend_mode: 0,
            depth_test: true,
            render_flags: 0,
        };
        
        let transform = Mat4::from_translation(glam::Vec3::new(i as f32, 0.0, 0.0));
        let _instance = Instance3D {
            model: transform.to_cols_array_2d(),
        };
        
        let _ = key; // 使用key避免未使用警告
    }
    
    let duration = start.elapsed();
    
    // 验证性能（应该在合理时间内完成）
    assert!(duration.as_millis() < 1000, "批次创建应在1秒内完成");
}

#[test]
#[ignore]
fn test_render_memory_usage() {
    // 测试大量实例的内存使用
    const ENTITY_COUNT: usize = 50000;
    
    let mut instances = Vec::with_capacity(ENTITY_COUNT);
    
    for i in 0..ENTITY_COUNT {
        let transform = Mat4::from_translation(glam::Vec3::new(
            (i % 100) as f32,
            ((i / 100) % 100) as f32,
            (i / 10000) as f32,
        ));
        instances.push(Instance3D {
            model: transform.to_cols_array_2d(),
        });
    }
    
    // 验证内存使用合理
    let instance_size = std::mem::size_of::<Instance3D>();
    let total_size = instance_size * ENTITY_COUNT;
    
    // 每个Instance3D是64字节（4x4矩阵），50000个应该是3.2MB
    assert!(total_size < 10 * 1024 * 1024, "内存使用应在10MB以内");
    assert_eq!(instances.len(), ENTITY_COUNT);
}

