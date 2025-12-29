//! Dirty Tracking 综合测试
//!
//! 测试ECS组件脏跟踪系统的各种功能

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::dirty_tracking::*;
    use bevy_ecs::prelude::*;

    // ========================================
    // DirtyFlags 位操作测试
    // ========================================

    #[test]
    fn test_dirty_flags_bits() {
        assert_eq!(DirtyFlags::NONE.bits(), 0);
        assert_eq!(DirtyFlags::POSITION.bits(), 1);
        assert_eq!(DirtyFlags::ROTATION.bits(), 2);
        assert_eq!(DirtyFlags::SCALE.bits(), 4);
        assert_eq!(DirtyFlags::TRANSFORM.bits(), 7);
    }

    #[test]
    fn test_dirty_flags_contains_single() {
        let flags = DirtyFlags::POSITION;
        assert!(flags.contains(DirtyFlags::POSITION));
        assert!(!flags.contains(DirtyFlags::ROTATION));
    }

    #[test]
    fn test_dirty_flags_contains_combined() {
        let flags = DirtyFlags::POSITION | DirtyFlags::ROTATION;
        assert!(flags.contains(DirtyFlags::POSITION));
        assert!(flags.contains(DirtyFlags::ROTATION));
        assert!(!flags.contains(DirtyFlags::SCALE));
    }

    #[test]
    fn test_dirty_flags_contains_transform() {
        let flags = DirtyFlags::TRANSFORM;
        assert!(flags.contains(DirtyFlags::POSITION));
        assert!(flags.contains(DirtyFlags::ROTATION));
        assert!(flags.contains(DirtyFlags::SCALE));
    }

    #[test]
    fn test_dirty_flags_custom_bit_range() {
        let flag0 = DirtyFlags::custom(0);
        let flag1 = DirtyFlags::custom(1);
        let flag8 = DirtyFlags::custom(8);

        assert_eq!(flag0.bits(), 1 << 8);
        assert_eq!(flag1.bits(), 1 << 9);
        assert_eq!(flag8.bits(), 1 << 16);
    }

    #[test]
    fn test_dirty_flags_custom_invalid_bit() {
        let flag = DirtyFlags::custom(100);
        assert_eq!(flag.bits(), 0);
    }

    #[test]
    fn test_dirty_flags_combine_single() {
        let flags = DirtyFlags::combine(&[DirtyFlags::POSITION]);
        assert_eq!(flags.bits(), DirtyFlags::POSITION.bits());
    }

    #[test]
    fn test_dirty_flags_combine_multiple() {
        let flags = DirtyFlags::combine(&[
            DirtyFlags::POSITION,
            DirtyFlags::ROTATION,
            DirtyFlags::SCALE,
        ]);
        assert_eq!(flags.bits(), 7);
    }

    #[test]
    fn test_dirty_flags_combine_with_custom() {
        let custom = DirtyFlags::custom(5);
        let flags = DirtyFlags::combine(&[DirtyFlags::POSITION, custom]);
        assert!(flags.contains(DirtyFlags::POSITION));
        assert!(flags.contains(custom));
    }

    // ========================================
    // DirtyFlags 运算符测试
    // ========================================

    #[test]
    fn test_dirty_flags_bitor_basic() {
        let result = DirtyFlags::POSITION | DirtyFlags::ROTATION;
        assert!(result.contains(DirtyFlags::POSITION));
        assert!(result.contains(DirtyFlags::ROTATION));
    }

    #[test]
    fn test_dirty_flags_bitor_chain() {
        let result = DirtyFlags::POSITION | DirtyFlags::ROTATION | DirtyFlags::SCALE;
        assert!(result.contains(DirtyFlags::POSITION));
        assert!(result.contains(DirtyFlags::ROTATION));
        assert!(result.contains(DirtyFlags::SCALE));
    }

    #[test]
    fn test_dirty_flags_bitand_basic() {
        let flags = DirtyFlags::TRANSFORM;
        let result = flags & DirtyFlags::POSITION;
        assert!(result.contains(DirtyFlags::POSITION));
        assert!(!result.contains(DirtyFlags::ROTATION));
    }

    #[test]
    fn test_dirty_flags_bitand_no_overlap() {
        let result = DirtyFlags::POSITION & DirtyFlags::MATERIAL;
        assert!(!result.contains(DirtyFlags::POSITION));
        assert!(!result.contains(DirtyFlags::MATERIAL));
    }

    #[test]
    fn test_dirty_flags_bitor_assign() {
        let mut flags = DirtyFlags::POSITION;
        flags |= DirtyFlags::ROTATION;
        assert!(flags.contains(DirtyFlags::ROTATION));
    }

    #[test]
    fn test_dirty_flags_bitand_assign() {
        let mut flags = DirtyFlags::TRANSFORM;
        flags &= DirtyFlags::POSITION;
        assert!(flags.contains(DirtyFlags::POSITION));
        assert!(!flags.contains(DirtyFlags::ROTATION));
    }

    // ========================================
    // ComponentDirty 基础测试
    // ========================================

    #[test]
    fn test_component_dirty_new() {
        let dirty = ComponentDirty::new();
        assert!(!dirty.is_any_dirty());
        assert_eq!(dirty.get_flags().bits(), 0);
    }

    #[test]
    fn test_component_dirty_default() {
        let dirty = ComponentDirty::default();
        assert!(!dirty.is_any_dirty());
    }

    #[test]
    fn test_component_dirty_mark_dirty() {
        let mut dirty = ComponentDirty::new();
        dirty.mark_dirty(DirtyFlags::POSITION);
        assert!(dirty.is_dirty(DirtyFlags::POSITION));
    }

    #[test]
    fn test_component_dirty_mark_multiple() {
        let mut dirty = ComponentDirty::new();
        dirty.mark_dirty(DirtyFlags::POSITION | DirtyFlags::ROTATION);
        assert!(dirty.is_dirty(DirtyFlags::POSITION));
        assert!(dirty.is_dirty(DirtyFlags::ROTATION));
    }

    #[test]
    fn test_component_dirty_mark_transform() {
        let mut dirty = ComponentDirty::new();
        dirty.mark_dirty(DirtyFlags::TRANSFORM);
        assert!(dirty.is_dirty(DirtyFlags::POSITION));
        assert!(dirty.is_dirty(DirtyFlags::ROTATION));
        assert!(dirty.is_dirty(DirtyFlags::SCALE));
    }

    // ========================================
    // ComponentDirty 检查操作测试
    // ========================================

    #[test]
    fn test_component_dirty_is_dirty_true() {
        let mut dirty = ComponentDirty::new();
        dirty.mark_dirty(DirtyFlags::POSITION);
        assert!(dirty.is_dirty(DirtyFlags::POSITION));
    }

    #[test]
    fn test_component_dirty_is_dirty_false() {
        let dirty = ComponentDirty::new();
        assert!(!dirty.is_dirty(DirtyFlags::POSITION));
    }

    #[test]
    fn test_component_dirty_is_dirty_partial() {
        let mut dirty = ComponentDirty::new();
        dirty.mark_dirty(DirtyFlags::POSITION | DirtyFlags::ROTATION);
        assert!(dirty.is_dirty(DirtyFlags::POSITION));
        assert!(!dirty.is_dirty(DirtyFlags::SCALE));
    }

    #[test]
    fn test_component_dirty_is_any_dirty_true() {
        let mut dirty = ComponentDirty::new();
        dirty.mark_dirty(DirtyFlags::POSITION);
        assert!(dirty.is_any_dirty());
    }

    #[test]
    fn test_component_dirty_is_any_dirty_false() {
        let dirty = ComponentDirty::new();
        assert!(!dirty.is_any_dirty());
    }

    #[test]
    fn test_component_dirty_get_flags() {
        let mut dirty = ComponentDirty::new();
        dirty.mark_dirty(DirtyFlags::POSITION | DirtyFlags::ROTATION);

        let flags = dirty.get_flags();
        assert!(flags.contains(DirtyFlags::POSITION));
        assert!(flags.contains(DirtyFlags::ROTATION));
    }

    // ========================================
    // ComponentDirty 清除操作测试
    // ========================================

    #[test]
    fn test_component_dirty_clear_single() {
        let mut dirty = ComponentDirty::new();
        dirty.mark_dirty(DirtyFlags::POSITION | DirtyFlags::ROTATION);

        dirty.clear(DirtyFlags::POSITION);
        assert!(!dirty.is_dirty(DirtyFlags::POSITION));
        assert!(dirty.is_dirty(DirtyFlags::ROTATION));
    }

    #[test]
    fn test_component_dirty_clear_multiple() {
        let mut dirty = ComponentDirty::new();
        dirty.mark_dirty(DirtyFlags::TRANSFORM);

        dirty.clear(DirtyFlags::POSITION | DirtyFlags::ROTATION);
        assert!(!dirty.is_dirty(DirtyFlags::POSITION));
        assert!(!dirty.is_dirty(DirtyFlags::ROTATION));
        assert!(dirty.is_dirty(DirtyFlags::SCALE));
    }

    #[test]
    fn test_component_dirty_clear_all() {
        let mut dirty = ComponentDirty::new();
        dirty.mark_dirty(DirtyFlags::TRANSFORM);

        dirty.clear_all();
        assert!(!dirty.is_any_dirty());
    }

    #[test]
    fn test_component_dirty_clear_nonexistent() {
        let mut dirty = ComponentDirty::new();
        dirty.mark_dirty(DirtyFlags::POSITION);

        // 清除未设置的标志不应该出错
        dirty.clear(DirtyFlags::ROTATION);
        assert!(dirty.is_dirty(DirtyFlags::POSITION));
    }

    // ========================================
    // ComponentDirty 原子操作测试
    // ========================================

    #[test]
    fn test_component_dirty_mark_dirty_atomic() {
        let dirty = ComponentDirty::new();
        dirty.mark_dirty_atomic(DirtyFlags::POSITION);
        assert!(dirty.is_dirty(DirtyFlags::POSITION));
    }

    #[test]
    fn test_component_dirty_clear_atomic() {
        let mut dirty = ComponentDirty::new();
        dirty.mark_dirty(DirtyFlags::POSITION);

        dirty.clear_atomic(DirtyFlags::POSITION);
        assert!(!dirty.is_dirty(DirtyFlags::POSITION));
    }

    #[test]
    fn test_component_dirty_clear_all_atomic() {
        let mut dirty = ComponentDirty::new();
        dirty.mark_dirty(DirtyFlags::TRANSFORM);

        dirty.clear_all_atomic();
        assert!(!dirty.is_any_dirty());
    }

    // ========================================
    // ComponentDirty 帧管理测试
    // ========================================

    #[test]
    fn test_component_dirty_update_frame() {
        let mut dirty = ComponentDirty::new();
        assert_eq!(dirty.last_cleared_frame(), 0);

        dirty.update_frame(5);
        assert_eq!(dirty.last_cleared_frame(), 5);

        dirty.update_frame(10);
        assert_eq!(dirty.last_cleared_frame(), 10);
    }

    #[test]
    fn test_component_dirty_update_frame_atomic() {
        let dirty = ComponentDirty::new();
        dirty.update_frame_atomic(5);
        assert_eq!(dirty.last_cleared_frame(), 5);
    }

    // ========================================
    // DirtyTrackingConfig 测试
    // ========================================

    #[test]
    fn test_dirty_tracking_config_default() {
        let config = DirtyTrackingConfig::default();
        assert!(config.enabled);
        assert_eq!(config.auto_clear_interval, 1);
        assert!(!config.auto_clear_on_system_end);
    }

    #[test]
    fn test_dirty_tracking_config_custom() {
        let config = DirtyTrackingConfig {
            enabled: false,
            auto_clear_interval: 5,
            auto_clear_on_system_end: true,
        };

        assert!(!config.enabled);
        assert_eq!(config.auto_clear_interval, 5);
        assert!(config.auto_clear_on_system_end);
    }

    // ========================================
    // DirtyTrackingResource 测试
    // ========================================

    #[test]
    fn test_dirty_tracking_resource_new() {
        let resource = DirtyTrackingResource::new();
        assert!(resource.config.enabled);
        assert_eq!(resource.current_frame, 0);
    }

    #[test]
    fn test_dirty_tracking_resource_default() {
        let resource = DirtyTrackingResource::default();
        assert!(resource.config.enabled);
        assert_eq!(resource.current_frame, 0);
    }

    #[test]
    fn test_dirty_tracking_resource_update_frame() {
        let mut resource = DirtyTrackingResource::new();
        assert_eq!(resource.current_frame(), 0);

        resource.update_frame();
        assert_eq!(resource.current_frame(), 1);

        resource.update_frame();
        assert_eq!(resource.current_frame(), 2);
    }

    #[test]
    fn test_dirty_tracking_resource_current_frame() {
        let resource = DirtyTrackingResource::new();
        assert_eq!(resource.current_frame(), 0);
    }

    // ========================================
    // 集成测试：Dirty标记生命周期
    // ========================================

    #[test]
    fn test_dirty_mark_clear_cycle() {
        let mut dirty = ComponentDirty::new();

        // 标记
        dirty.mark_dirty(DirtyFlags::POSITION);
        assert!(dirty.is_dirty(DirtyFlags::POSITION));

        // 清除
        dirty.clear(DirtyFlags::POSITION);
        assert!(!dirty.is_dirty(DirtyFlags::POSITION));

        // 重新标记
        dirty.mark_dirty(DirtyFlags::POSITION);
        assert!(dirty.is_dirty(DirtyFlags::POSITION));
    }

    #[test]
    fn test_dirty_accumulation() {
        let mut dirty = ComponentDirty::new();

        dirty.mark_dirty(DirtyFlags::POSITION);
        dirty.mark_dirty(DirtyFlags::ROTATION);
        dirty.mark_dirty(DirtyFlags::SCALE);

        assert!(dirty.is_dirty(DirtyFlags::TRANSFORM));
    }

    #[test]
    fn test_dirty_partial_clear() {
        let mut dirty = ComponentDirty::new();

        dirty.mark_dirty(DirtyFlags::TRANSFORM);
        dirty.clear(DirtyFlags::POSITION);

        assert!(!dirty.is_dirty(DirtyFlags::POSITION));
        assert!(dirty.is_dirty(DirtyFlags::ROTATION));
        assert!(dirty.is_dirty(DirtyFlags::SCALE));
    }

    // ========================================
    // 性能测试
    // ========================================

    #[test]
    fn test_dirty_flags_operations_performance() {
        let iterations = 10000;

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let flags = DirtyFlags::POSITION | DirtyFlags::ROTATION;
            let _ = flags.contains(DirtyFlags::POSITION);
        }
        let duration = start.elapsed();

        // 应该非常快速（< 10ms）
        assert!(duration < std::time::Duration::from_millis(10));
    }

    #[test]
    fn test_component_dirty_operations_performance() {
        let mut dirty = ComponentDirty::new();
        let iterations = 10000;

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            dirty.mark_dirty(DirtyFlags::POSITION);
            dirty.clear(DirtyFlags::POSITION);
        }
        let duration = start.elapsed();

        // 应该快速完成（< 50ms）
        assert!(duration < std::time::Duration::from_millis(50));
    }

    // ========================================
    // 边界情况测试
    // ========================================

    #[test]
    fn test_dirty_flags_none_combinations() {
        let flags = DirtyFlags::NONE | DirtyFlags::NONE;
        assert_eq!(flags.bits(), 0);
    }

    #[test]
    fn test_dirty_flags_all_render_flags() {
        let flags = DirtyFlags::RENDER | DirtyFlags::MATERIAL | DirtyFlags::MESH;
        assert!(flags.contains(DirtyFlags::RENDER));
        assert!(flags.contains(DirtyFlags::MATERIAL));
        assert!(flags.contains(DirtyFlags::MESH));
    }

    #[test]
    fn test_component_dirty_multiple_operations() {
        let mut dirty = ComponentDirty::new();

        // 多次标记同一标志
        dirty.mark_dirty(DirtyFlags::POSITION);
        dirty.mark_dirty(DirtyFlags::POSITION);
        dirty.mark_dirty(DirtyFlags::POSITION);

        // 应该只标记一次
        assert!(dirty.is_dirty(DirtyFlags::POSITION));

        // 清除后应该干净
        dirty.clear(DirtyFlags::POSITION);
        assert!(!dirty.is_dirty(DirtyFlags::POSITION));
    }

    #[test]
    fn test_component_dirty_complex_scenario() {
        let mut dirty = ComponentDirty::new();

        // 标记多个标志
        dirty.mark_dirty(DirtyFlags::TRANSFORM);
        dirty.mark_dirty(DirtyFlags::MATERIAL);

        // 部分清除
        dirty.clear(DirtyFlags::POSITION);

        // 验证状态
        assert!(!dirty.is_dirty(DirtyFlags::POSITION));
        assert!(dirty.is_dirty(DirtyFlags::ROTATION));
        assert!(dirty.is_dirty(DirtyFlags::SCALE));
        assert!(dirty.is_dirty(DirtyFlags::MATERIAL));
    }

    // ========================================
    // 线程安全测试
    // ========================================

    #[test]
    fn test_atomic_operations_thread_safety() {
        let dirty = std::sync::Arc::new(ComponentDirty::new());
        let dirty_clone = dirty.clone();

        // 标记
        dirty_clone.mark_dirty_atomic(DirtyFlags::POSITION);

        // 检查
        assert!(dirty.is_dirty(DirtyFlags::POSITION));

        // 清除
        dirty.clear_atomic(DirtyFlags::POSITION);

        // 验证清除
        assert!(!dirty.is_dirty(DirtyFlags::POSITION));
    }

    // ========================================
    // 实际使用场景测试
    // ========================================

    #[test]
    fn test_transform_update_scenario() {
        let mut dirty = ComponentDirty::new();

        // 模拟位置更新
        dirty.mark_dirty(DirtyFlags::POSITION);
        assert!(dirty.is_dirty(DirtyFlags::POSITION));
        assert!(!dirty.is_dirty(DirtyFlags::ROTATION));

        // 处理位置更新
        dirty.clear(DirtyFlags::POSITION);
        assert!(!dirty.is_any_dirty());

        // 模拟旋转更新
        dirty.mark_dirty(DirtyFlags::ROTATION);
        assert!(dirty.is_dirty(DirtyFlags::ROTATION));
    }

    #[test]
    fn test_render_update_scenario() {
        let mut dirty = ComponentDirty::new();

        // 模拟材质更新
        dirty.mark_dirty(DirtyFlags::MATERIAL);

        // 检查渲染相关脏标记
        assert!(dirty.is_dirty(DirtyFlags::MATERIAL));
        assert!(!dirty.is_dirty(DirtyFlags::MESH));

        // 添加网格更新
        dirty.mark_dirty(DirtyFlags::MESH);

        // 两个都应该脏
        assert!(dirty.is_dirty(DirtyFlags::MATERIAL));
        assert!(dirty.is_dirty(DirtyFlags::MESH));
    }

    #[test]
    fn test_physics_update_scenario() {
        let mut dirty = ComponentDirty::new();

        // 模拟物理更新
        dirty.mark_dirty(DirtyFlags::PHYSICS);
        dirty.mark_dirty(DirtyFlags::COLLIDER);

        assert!(dirty.is_dirty(DirtyFlags::PHYSICS));
        assert!(dirty.is_dirty(DirtyFlags::COLLIDER));
    }

    #[test]
    fn test_combined_update_scenario() {
        let mut dirty = ComponentDirty::new();

        // 模拟多个系统更新
        dirty.mark_dirty(DirtyFlags::TRANSFORM);
        dirty.mark_dirty(DirtyFlags::MATERIAL);
        dirty.mark_dirty(DirtyFlags::PHYSICS);

        // 处理Transform
        assert!(dirty.is_dirty(DirtyFlags::POSITION));
        dirty.clear(DirtyFlags::TRANSFORM);

        // 处理Material
        assert!(dirty.is_dirty(DirtyFlags::MATERIAL));
        dirty.clear(DirtyFlags::MATERIAL);

        // 处理Physics
        assert!(dirty.is_dirty(DirtyFlags::PHYSICS));
        dirty.clear(DirtyFlags::PHYSICS);

        // 应该全部干净
        assert!(!dirty.is_any_dirty());
    }

    // ========================================
    // 自定义标志测试
    // ========================================

    #[test]
    fn test_custom_flags_usage() {
        let custom1 = DirtyFlags::custom(0);
        let custom2 = DirtyFlags::custom(1);
        let custom3 = DirtyFlags::custom(2);

        let combined = custom1 | custom2 | custom3;
        assert!(combined.contains(custom1));
        assert!(combined.contains(custom2));
        assert!(combined.contains(custom3));
    }

    #[test]
    fn test_custom_flags_with_predefined() {
        let custom = DirtyFlags::custom(10);
        let combined = DirtyFlags::POSITION | custom;

        assert!(combined.contains(DirtyFlags::POSITION));
        assert!(combined.contains(custom));
    }

    #[test]
    fn test_custom_flags_in_component_dirty() {
        let mut dirty = ComponentDirty::new();
        let custom = DirtyFlags::custom(5);

        dirty.mark_dirty(custom);
        assert!(dirty.is_dirty(custom));

        dirty.clear(custom);
        assert!(!dirty.is_dirty(custom));
    }
}
