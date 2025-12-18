//! 渲染模块测试
//!
//! 包含渲染管线、纹理、后处理等功能的单元测试。

#[cfg(test)]
mod tests {
    use super::super::wgpu_modules::buffer::*;
    use super::super::wgpu_modules::types::*;

    // ========================================
    // 类型测试
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
    fn test_ui_instance_default() {
        let ui = UiInstance::default();
        assert_eq!(ui.pos, [0.0, 0.0]);
        assert_eq!(ui.size, [100.0, 100.0]);
        assert_eq!(ui.radius, 0.0);
    }

    #[test]
    fn test_vertex_quad() {
        let quad = Vertex::quad();
        assert_eq!(quad.len(), 6);
        // 检查是否形成两个三角形
        assert_eq!(quad[0].pos, [-0.5, -0.5]);
        assert_eq!(quad[2].pos, [0.5, 0.5]);
    }

    #[test]
    fn test_gpu_point_light_default() {
        let light = GpuPointLight::default();
        assert_eq!(light.pos, [0.0, 0.0]);
        assert_eq!(light.color, [1.0, 1.0, 1.0]);
        assert_eq!(light.radius, 100.0);
        assert_eq!(light.intensity, 1.0);
    }

    #[test]
    fn test_draw_group_creation() {
        let group = DrawGroup::new(0, 10, 0, 0.0);
        assert_eq!(group.start, 0);
        assert_eq!(group.end, 10);
        assert_eq!(group.tex_idx, 0);
        assert!(group.scissor.is_none());

        let group_with_scissor = group.with_scissor(Some([0, 0, 100, 100]));
        assert!(group_with_scissor.scissor.is_some());
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
}
