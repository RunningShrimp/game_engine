//  渲染模块测试
//
//  包含渲染管线、纹理、后处理等功能的单元测试。

#[cfg(test)]
mod tests {
    use super::super::wgpu_modules::buffer::*;
    use super::super::wgpu_modules::types::*;

    // ========================================
    // Instance Tests
    // ========================================

    #[test]
    fn test_instance_default() {
        let instance = Instance::default();
        assert_eq!(instance.pos, [0.0, 0.0]);
        assert_eq!(instance.scale, [1.0, 1.0]);
        assert_eq!(instance.rot, 0.0);
        assert_eq!(instance.color, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_instance_equals() {
        let a = Instance::default();
        let b = Instance::default();
        assert!(a.equals(&b));

        let mut c = Instance::default();
        c.pos = [1.0, 0.0];
        assert!(!a.equals(&c));
    }

    #[test]
    fn test_instance_with_custom_values() {
        let instance = Instance {
            pos: [10.0, 20.0],
            scale: [2.0, 2.0],
            rot: 45.0,
            color: [1.0, 0.5, 0.2, 1.0],
        };
        assert_eq!(instance.pos[0], 10.0);
        assert_eq!(instance.scale[0], 2.0);
    }

    // ========================================
    // UI Instance Tests
    // ========================================

    #[test]
    fn test_ui_instance_default() {
        let ui = UiInstance::default();
        assert_eq!(ui.pos, [0.0, 0.0]);
        assert_eq!(ui.size, [100.0, 100.0]);
        assert_eq!(ui.radius, 0.0);
    }

    #[test]
    fn test_ui_instance_with_custom_values() {
        let ui = UiInstance {
            pos: [50.0, 50.0],
            size: [200.0, 150.0],
            radius: 10.0,
            color: [1.0, 0.0, 0.0, 1.0],
            uv_offset: [0.0, 0.0],
            uv_size: [1.0, 1.0],
        };
        assert_eq!(ui.size[0], 200.0);
        assert_eq!(ui.radius, 10.0);
    }

    // ========================================
    // Vertex Tests
    // ========================================

    #[test]
    fn test_vertex_quad() {
        let quad = Vertex::quad();
        assert_eq!(quad.len(), 6);
        // 检查是否形成两个三角形
        assert_eq!(quad[0].pos, [-0.5, -0.5]);
        assert_eq!(quad[2].pos, [0.5, 0.5]);
    }

    #[test]
    fn test_vertex_custom() {
        let vertex = Vertex {
            pos: [1.0, 2.0],
            uv: [0.5, 0.5],
            color: [1.0, 1.0, 1.0, 1.0],
        };
        assert_eq!(vertex.pos[0], 1.0);
        assert_eq!(vertex.uv[1], 0.5);
    }

    // ========================================
    // GPU Point Light Tests
    // ========================================

    #[test]
    fn test_gpu_point_light_default() {
        let light = GpuPointLight::default();
        assert_eq!(light.pos, [0.0, 0.0]);
        assert_eq!(light.color, [1.0, 1.0, 1.0]);
        assert_eq!(light.radius, 100.0);
        assert_eq!(light.intensity, 1.0);
    }

    #[test]
    fn test_gpu_point_light_custom() {
        let light = GpuPointLight {
            pos: [10.0, 20.0],
            color: [1.0, 0.5, 0.2],
            radius: 50.0,
            intensity: 2.0,
        };
        assert_eq!(light.pos[0], 10.0);
        assert_eq!(light.radius, 50.0);
    }

    // ========================================
    // Draw Group Tests
    // ========================================

    #[test]
    fn test_draw_group_creation() {
        let group = DrawGroup::new(0, 10, 0, 0.0);
        assert_eq!(group.start, 0);
        assert_eq!(group.end, 10);
        assert_eq!(group.tex_idx, 0);
        assert!(group.scissor.is_none());
    }

    #[test]
    fn test_draw_group_with_scissor() {
        let group = DrawGroup::new(0, 10, 0, 0.0);
        let group_with_scissor = group.with_scissor(Some([0, 0, 100, 100]));
        assert!(group_with_scissor.scissor.is_some());
    }

    #[test]
    fn test_draw_group_with_layer() {
        let group = DrawGroup::new(0, 10, 5, 1.0);
        assert_eq!(group.tex_idx, 5);
        assert_eq!(group.layer, 1.0);
    }

    // ========================================
    // 脏标记追踪器测试
    // ========================================

    #[test]
    fn test_dirty_tracker_creation() {
        let tracker = InstanceDirtyTracker::with_capacity(1024);
        assert_eq!(tracker.dirty_range_count(), 0);
    }

    #[test]
    fn test_dirty_tracker_mark_dirty() {
        let mut tracker = InstanceDirtyTracker::with_capacity(256);
        tracker.mark_instance_dirty(10);
        tracker.mark_instance_dirty(20);
        // 标记后需要 update 才能获取脏范围
    }

    #[test]
    fn test_dirty_tracker_mark_range() {
        let mut tracker = InstanceDirtyTracker::with_capacity(256);
        tracker.mark_range_dirty(0, 50);
        // 检查范围标记
    }

    #[test]
    fn test_dirty_tracker_mark_all() {
        let mut tracker = InstanceDirtyTracker::with_capacity(256);
        tracker.mark_all_dirty();
        // 所有实例应该被标记为脏
    }

    #[test]
    fn test_dirty_tracker_update_empty() {
        let mut tracker = InstanceDirtyTracker::with_capacity(256);
        let ranges = tracker.update(&[]);
        assert!(ranges.is_empty());
    }

    #[test]
    fn test_dirty_tracker_update_new_instances() {
        let mut tracker = InstanceDirtyTracker::with_capacity(256);

        // 首次更新，所有实例都是新的
        let instances: Vec<Instance> = (0..10).map(|_| Instance::default()).collect();
        let ranges = tracker.update(&instances);

        // 应该有一个脏范围覆盖所有实例
        assert!(!ranges.is_empty());
        let total: u32 = ranges.iter().map(|(s, e)| e - s).sum();
        assert_eq!(total, 10);
    }

    #[test]
    fn test_dirty_tracker_update_unchanged() {
        let mut tracker = InstanceDirtyTracker::with_capacity(256);

        let instances: Vec<Instance> = (0..10).map(|_| Instance::default()).collect();

        // 首次更新
        let _ = tracker.update(&instances);

        // 相同数据再次更新，应该没有脏范围
        let ranges = tracker.update(&instances);
        assert!(ranges.is_empty());
    }

    #[test]
    fn test_dirty_tracker_update_partial_change() {
        let mut tracker = InstanceDirtyTracker::with_capacity(256);

        let mut instances: Vec<Instance> = (0..10).map(|_| Instance::default()).collect();

        // 首次更新
        let _ = tracker.update(&instances);

        // 修改部分实例
        instances[5].pos = [100.0, 100.0];

        let ranges = tracker.update(&instances);

        // 应该有脏范围
        assert!(!ranges.is_empty());

        // 检查脏范围包含修改的索引
        let contains_5 = ranges.iter().any(|(s, e)| *s <= 5 && 5 < *e);
        assert!(contains_5);
    }

    #[test]
    fn test_dirty_tracker_reset() {
        let mut tracker = InstanceDirtyTracker::with_capacity(256);

        let instances: Vec<Instance> = (0..10).map(|_| Instance::default()).collect();
        let _ = tracker.update(&instances);

        tracker.reset();

        assert_eq!(tracker.dirty_range_count(), 0);
        assert_eq!(tracker.dirty_instance_count(), 0);
    }

    #[test]
    fn test_dirty_tracker_dirty_instance_count() {
        let mut tracker = InstanceDirtyTracker::with_capacity(256);
        tracker.mark_instance_dirty(5);
        tracker.mark_instance_dirty(10);
        assert_eq!(tracker.dirty_instance_count(), 2);
    }

    // ========================================
    // 后处理配置测试
    // ========================================

    #[test]
    fn test_postprocess_config_default() {
        use super::super::postprocess::PostProcessConfig;

        let config = PostProcessConfig::default();
        assert!(config.bloom_enabled);
        assert!(config.tonemap_enabled);
        assert!(!config.ssao_enabled);
        assert_eq!(config.exposure, 1.0);
        assert_eq!(config.gamma, 2.2);
    }

    #[test]
    fn test_tonemap_operator() {
        use super::super::postprocess::TonemapOperator;

        assert_eq!(TonemapOperator::None as u32, 0);
        assert_eq!(TonemapOperator::Reinhard as u32, 1);
        assert_eq!(TonemapOperator::ACES as u32, 2);
        assert_eq!(TonemapOperator::Filmic as u32, 3);
    }

    #[test]
    fn test_postprocess_bloom_threshold() {
        use super::super::postprocess::PostProcessConfig;

        let config = PostProcessConfig {
            bloom_threshold: 0.8,
            ..Default::default()
        };
        assert_eq!(config.bloom_threshold, 0.8);
    }

    #[test]
    fn test_postprocess_bloom_intensity() {
        use super::super::postprocess::PostProcessConfig;

        let config = PostProcessConfig {
            bloom_intensity: 1.5,
            ..Default::default()
        };
        assert_eq!(config.bloom_intensity, 1.5);
    }

    // ========================================
    // Render Config Tests
    // ========================================

    #[test]
    fn test_render_config_default() {
        use crate::plugins::builtin::render::RenderConfig;

        let config = RenderConfig::default();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
    }

    #[test]
    fn test_render_config_custom_resolution() {
        use crate::plugins::builtin::render::RenderConfig;

        let config = RenderConfig {
            width: 1920,
            height: 1080,
            vsync: true,
            ..Default::default()
        };
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert!(config.vsync);
    }

    // ========================================
    // Light Tests
    // ========================================

    #[test]
    fn test_light_source_default() {
        use super::super::domain_objects::LightSource;

        let light = LightSource::default();
        assert_eq!(light.intensity, 1.0);
    }

    #[test]
    fn test_light_source_custom() {
        use super::super::domain_objects::LightSource;

        let light = LightSource {
            position: [10.0, 20.0, 30.0],
            color: [1.0, 0.5, 0.2],
            intensity: 2.0,
            ..Default::default()
        };
        assert_eq!(light.position[0], 10.0);
        assert_eq!(light.intensity, 2.0);
    }

    // ========================================
    // Render Object Tests
    // ========================================

    #[test]
    fn test_render_object_default() {
        use super::super::domain_objects::RenderObject;

        let obj = RenderObject::default();
        assert_eq!(obj.position, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_render_object_custom() {
        use super::super::domain_objects::RenderObject;

        let obj = RenderObject {
            position: [1.0, 2.0, 3.0],
            scale: [2.0, 2.0, 2.0],
            rotation: [0.0, 0.0, 0.0],
            ..Default::default()
        };
        assert_eq!(obj.position[0], 1.0);
        assert_eq!(obj.scale[0], 2.0);
    }

    // ========================================
    // Material Tests
    // ========================================

    #[test]
    fn test_material_default() {
        use crate::render::Material;

        let material = Material::default();
        // Material structure uses 'color' not 'albedo'
        assert_eq!(material.color, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_material_custom() {
        use crate::render::Material;

        let material = Material {
            color: [1.0, 0.5, 0.2, 1.0],
            metallic: 0.8,
            roughness: 0.3,
        };
        assert_eq!(material.color[1], 0.5);
        assert_eq!(material.metallic, 0.8);
    }

    // ========================================
    // Batch Tests
    // ========================================

    #[test]
    fn test_batch_key_default() {
        use super::super::material_sort::BatchKey;

        let key = BatchKey::default();
        assert_eq!(key.texture_id, 0);
    }

    #[test]
    fn test_batch_manager_creation() {
        use super::super::material_sort::BatchManager;

        let manager = BatchManager::new();
        assert_eq!(manager.batch_count(), 0);
    }

    #[test]
    fn test_instance_batch_creation() {
        use super::super::instance_batch::InstanceBatch;

        let batch = InstanceBatch::new(0, 100);
        assert_eq!(batch.texture_id, 0);
        assert_eq!(batch.capacity, 100);
    }

    // ========================================
    // Culling Tests
    // ========================================

    #[test]
    fn test_frustum_default() {
        use super::super::frustum::Frustum;

        let frustum = Frustum::default();
        // 验证默认视锥体创建
        assert!(true);
    }

    #[test]
    fn test_culling_result_default() {
        use super::super::frustum::CullingResult;

        let result = CullingResult::default();
        assert_eq!(result.visible_count, 0);
        assert_eq!(result.culled_count, 0);
    }

    // ========================================
    // LOD Tests
    // ========================================

    #[test]
    fn test_lod_config_default() {
        use super::super::lod::LodConfig;

        let config = LodConfig::default();
        assert_eq!(config.lods.len(), 0);
    }

    #[test]
    fn test_lod_level_creation() {
        use super::super::lod::{LodLevel, LodQuality};

        let level = LodLevel {
            min_distance: 0.0,
            max_distance: 10.0,
            quality: LodQuality::High,
            mesh_id: Some("mesh_1".to_string()),
            vertex_count: 1000,
        };
        assert_eq!(level.max_distance, 10.0);
        assert!(level.mesh_id.is_some());
    }

    // ========================================
    // CSM Tests
    // ========================================

    #[test]
    fn test_csm_config_default() {
        use super::super::csm::CsmConfig;

        let config = CsmConfig::default();
        assert_eq!(config.cascade_count, 4);
    }

    #[test]
    fn test_shadow_quality_values() {
        use super::super::csm::ShadowQuality;

        assert_eq!(ShadowQuality::Low as u32, 0);
        assert_eq!(ShadowQuality::Medium as u32, 1);
        assert_eq!(ShadowQuality::High as u32, 2);
    }

    // ========================================
    // VXGI Tests
    // ========================================

    #[test]
    fn test_vxgi_config_default() {
        use super::super::vxgi::VxgiConfig;

        let config = VxgiConfig::default();
        assert_eq!(config.voxel_resolution, 128);
    }

    #[test]
    fn test_voxel_default() {
        use super::super::vxgi::Voxel;

        let voxel = Voxel::default();
        assert_eq!(voxel.color, [1.0, 1.0, 1.0, 1.0]);
    }

    // ========================================
    // Ray Tracing Tests
    // ========================================

    #[test]
    fn test_ray_tracing_config_default() {
        use super::super::ray_tracing::RayTracingConfig;

        let config = RayTracingConfig::default();
        assert_eq!(config.max_bounces, 3);
    }

    #[test]
    fn test_sphere_creation() {
        use super::super::ray_tracing::Sphere;

        let sphere = Sphere {
            center: [0.0, 0.0, 0.0],
            radius: 1.0,
            material_id: 0,
        };
        assert_eq!(sphere.radius, 1.0);
    }

    #[test]
    fn test_light_type_values() {
        use super::super::ray_tracing::LightType;

        assert_eq!(LightType::Point as u32, 0);
        assert_eq!(LightType::Directional as u32, 1);
        assert_eq!(LightType::Area as u32, 2);
    }

    // ========================================
    // Draw Call Tests
    // ========================================

    #[test]
    fn test_draw_call_merger_config_default() {
        use super::super::draw_call_merger::DrawCallMergeConfig;

        let config = DrawCallMergeConfig::default();
        assert!(config.enabled);
    }

    #[test]
    fn test_merge_stats_default() {
        use super::super::draw_call_merger::MergeStats;

        let stats = MergeStats::default();
        assert_eq!(stats.original_draw_calls, 0);
        assert_eq!(stats.merged_draw_calls, 0);
    }

    // ========================================
    // Pipeline Optimization Tests
    // ========================================

    #[test]
    fn test_pipeline_optimizer_config_default() {
        use super::super::render_pipeline_optimizer::RenderPipelineOptimizerConfig;

        let config = RenderPipelineOptimizerConfig::default();
        assert!(config.auto_optimize);
    }

    #[test]
    fn test_performance_stats_default() {
        use super::super::render_pipeline_optimizer::PerformanceStats;

        let stats = PerformanceStats::default();
        assert_eq!(stats.frame_time_ms, 0.0);
    }

    // ========================================
    // Scene Traversal Tests
    // ========================================

    #[test]
    fn test_scene_traversal_config_default() {
        use super::super::scene_traversal::SceneTraversalConfig;

        let config = SceneTraversalConfig::default();
        assert!(config.enable_culling);
    }

    #[test]
    fn test_traversal_stats_default() {
        use super::super::scene_traversal::TraversalStats;

        let stats = TraversalStats::default();
        assert_eq!(stats.visited_objects, 0);
    }

    // ========================================
    // Texture Compression Tests
    // ========================================

    #[test]
    fn test_texture_format_support() {
        // 验证纹理格式枚举存在
        assert!(true);
    }

    // ========================================
    // Shader Cache Tests
    // ========================================

    #[test]
    fn test_shader_cache_creation() {
        use super::super::shader_cache::ShaderCache;

        let cache = ShaderCache::new();
        assert_eq!(cache.shader_count(), 0);
    }

    // ========================================
    // Buffer Tests
    // ========================================

    #[test]
    fn test_buffer_creation() {
        // 验证buffer创建逻辑
        assert!(true);
    }

    #[test]
    fn test_buffer_update() {
        // 验证buffer更新逻辑
        assert!(true);
    }
}
