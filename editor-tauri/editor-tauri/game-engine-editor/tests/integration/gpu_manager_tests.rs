// GPU Manager Integration Tests
// 测试GPU管理器的所有功能

use crate::fixtures::mock_platforms::MockGPUManager;
use crate::fixtures::test_entities::TestGPUInfo;

#[cfg(test)]
mod gpu_manager_tests {
    use super::*;

    // ============================================================================
    // VRAM管理测试
    // ============================================================================

    #[test]
    fn test_vram_allocation() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        // 分配VRAM
        assert!(gpu_manager.allocate_vram(1000).is_ok());
        assert_eq!(gpu_manager.vram_used_mb, 1000);
    }

    #[test]
    fn test_vram_allocation_multiple() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        // 多次分配
        assert!(gpu_manager.allocate_vram(500).is_ok());
        assert!(gpu_manager.allocate_vram(1000).is_ok());
        assert!(gpu_manager.allocate_vram(1500).is_ok());

        assert_eq!(gpu_manager.vram_used_mb, 3000);
    }

    #[test]
    fn test_vram_free() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        gpu_manager.allocate_vram(2000).unwrap();
        gpu_manager.free_vram(500);

        assert_eq!(gpu_manager.vram_used_mb, 1500);
    }

    #[test]
    fn test_vram_free_all() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        gpu_manager.allocate_vram(3000).unwrap();
        gpu_manager.free_vram(3000);

        assert_eq!(gpu_manager.vram_used_mb, 0);
    }

    #[test]
    fn test_vram_insufficient_memory() {
        let gpu_info = TestGPUInfo::new("GTX 1050", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        // 尝试分配超过总VRAM的量
        let result = gpu_manager.allocate_vram(5000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient VRAM"));
    }

    #[test]
    fn test_vram_usage_percentage() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        gpu_manager.allocate_vram(2048).unwrap();

        let usage = gpu_manager.get_vram_usage();
        assert_eq!(usage, 2048.0 / 4096.0);
    }

    #[test]
    fn test_vram_usage_full() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        gpu_manager.allocate_vram(4096).unwrap();

        let usage = gpu_manager.get_vram_usage();
        assert_eq!(usage, 1.0);
    }

    #[test]
    fn test_vram_free_underflow() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        gpu_manager.allocate_vram(1000).unwrap();
        gpu_manager.free_vram(2000); // 尝试释放超过已分配的量

        // 应该被截断到0
        assert_eq!(gpu_manager.vram_used_mb, 0);
    }

    // ============================================================================
    // 视锥剔除测试
    // ============================================================================

    #[test]
    fn test_frustum_culling_enabled_by_default() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let gpu_manager = MockGPUManager::new(gpu_info);

        assert!(gpu_manager.frustum_culling_enabled);
    }

    #[test]
    fn test_enable_frustum_culling() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        gpu_manager.disable_feature("frustum_culling").unwrap();
        assert!(!gpu_manager.frustum_culling_enabled);

        gpu_manager.enable_feature("frustum_culling").unwrap();
        assert!(gpu_manager.frustum_culling_enabled);
    }

    #[test]
    fn test_disable_frustum_culling() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        gpu_manager.disable_feature("frustum_culling").unwrap();
        assert!(!gpu_manager.is_feature_enabled("frustum_culling"));
    }

    // ============================================================================
    // 遮挡剔除测试
    // ============================================================================

    #[test]
    fn test_occlusion_culling_disabled_by_default() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let gpu_manager = MockGPUManager::new(gpu_info);

        assert!(!gpu_manager.occlusion_culling_enabled);
    }

    #[test]
    fn test_enable_occlusion_culling_with_raytracing() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA")
            .with_raytracing();
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        let result = gpu_manager.enable_feature("occlusion_culling");
        assert!(result.is_ok());
        assert!(gpu_manager.occlusion_culling_enabled);
    }

    #[test]
    fn test_enable_occlusion_culling_without_raytracing() {
        let gpu_info = TestGPUInfo::new("GTX 1050", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        let result = gpu_manager.enable_feature("occlusion_culling");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("raytracing support"));
    }

    // ============================================================================
    // 距离剔除测试
    // ============================================================================

    #[test]
    fn test_distance_culling_enabled_by_default() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let gpu_manager = MockGPUManager::new(gpu_info);

        assert!(gpu_manager.distance_culling_enabled);
    }

    #[test]
    fn test_toggle_distance_culling() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        gpu_manager.disable_feature("distance_culling").unwrap();
        assert!(!gpu_manager.distance_culling_enabled);

        gpu_manager.enable_feature("distance_culling").unwrap();
        assert!(gpu_manager.distance_culling_enabled);
    }

    // ============================================================================
    // 间接绘制测试
    // ============================================================================

    #[test]
    fn test_indirect_draw_disabled_by_default() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let gpu_manager = MockGPUManager::new(gpu_info);

        assert!(!gpu_manager.indirect_draw_enabled);
    }

    #[test]
    fn test_enable_indirect_draw_with_mesh_shaders() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA")
            .with_mesh_shaders();
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        let result = gpu_manager.enable_feature("indirect_draw");
        assert!(result.is_ok());
        assert!(gpu_manager.indirect_draw_enabled);
    }

    #[test]
    fn test_enable_indirect_draw_without_mesh_shaders() {
        let gpu_info = TestGPUInfo::new("GTX 1050", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        let result = gpu_manager.enable_feature("indirect_draw");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mesh shader support"));
    }

    // ============================================================================
    // 多特性组合测试
    // ============================================================================

    #[test]
    fn test_enable_all_culling_features() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA")
            .with_raytracing();
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        gpu_manager.enable_feature("frustum_culling").unwrap();
        gpu_manager.enable_feature("occlusion_culling").unwrap();
        gpu_manager.enable_feature("distance_culling").unwrap();

        assert!(gpu_manager.frustum_culling_enabled);
        assert!(gpu_manager.occlusion_culling_enabled);
        assert!(gpu_manager.distance_culling_enabled);
    }

    #[test]
    fn test_enable_all_features_with_full_gpu() {
        let gpu_info = TestGPUInfo::new("RTX 4090", "NVIDIA")
            .with_raytracing()
            .with_mesh_shaders();
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        gpu_manager.enable_feature("frustum_culling").unwrap();
        gpu_manager.enable_feature("occlusion_culling").unwrap();
        gpu_manager.enable_feature("distance_culling").unwrap();
        gpu_manager.enable_feature("indirect_draw").unwrap();

        assert!(gpu_manager.frustum_culling_enabled);
        assert!(gpu_manager.occlusion_culling_enabled);
        assert!(gpu_manager.distance_culling_enabled);
        assert!(gpu_manager.indirect_draw_enabled);
    }

    // ============================================================================
    // 特性禁用测试
    // ============================================================================

    #[test]
    fn test_disable_all_features() {
        let gpu_info = TestGPUInfo::new("RTX 4090", "NVIDIA")
            .with_raytracing()
            .with_mesh_shaders();
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        // 启用所有特性
        gpu_manager.enable_feature("occlusion_culling").unwrap();
        gpu_manager.enable_feature("indirect_draw").unwrap();

        // 禁用所有特性
        gpu_manager.disable_feature("frustum_culling").unwrap();
        gpu_manager.disable_feature("occlusion_culling").unwrap();
        gpu_manager.disable_feature("distance_culling").unwrap();
        gpu_manager.disable_feature("indirect_draw").unwrap();

        assert!(!gpu_manager.frustum_culling_enabled);
        assert!(!gpu_manager.occlusion_culling_enabled);
        assert!(!gpu_manager.distance_culling_enabled);
        assert!(!gpu_manager.indirect_draw_enabled);
    }

    // ============================================================================
    // 特性状态查询测试
    // ============================================================================

    #[test]
    fn test_is_feature_enabled() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        assert!(gpu_manager.is_feature_enabled("frustum_culling"));
        assert!(!gpu_manager.is_feature_enabled("occlusion_culling"));

        gpu_manager.disable_feature("frustum_culling").unwrap();
        assert!(!gpu_manager.is_feature_enabled("frustum_culling"));
    }

    #[test]
    fn test_unknown_feature() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        let result = gpu_manager.enable_feature("unknown_feature");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown feature"));

        assert!(!gpu_manager.is_feature_enabled("unknown_feature"));
    }

    // ============================================================================
    // GPU能力测试
    // ============================================================================

    #[test]
    fn test_gpu_info() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let gpu_manager = MockGPUManager::new(gpu_info.clone());

        assert_eq!(gpu_manager.gpu_info.name, "RTX 3080");
        assert_eq!(gpu_manager.gpu_info.vendor, "NVIDIA");
        assert_eq!(gpu_manager.gpu_info.memory_mb, 4096);
    }

    #[test]
    fn test_gpu_raytracing_capability() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA")
            .with_raytracing();
        let gpu_manager = MockGPUManager::new(gpu_info);

        assert!(gpu_manager.gpu_info.supports_raytracing);
    }

    #[test]
    fn test_gpu_mesh_shader_capability() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA")
            .with_mesh_shaders();
        let gpu_manager = MockGPUManager::new(gpu_info);

        assert!(gpu_manager.gpu_info.supports_mesh_shaders);
    }

    #[test]
    fn test_gpu_variable_rate_shading_capability() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_info_with_vrs = gpu_info.clone();
        gpu_info_with_vrs.supports_variable_rate_shading = true;

        let gpu_manager = MockGPUManager::new(gpu_info_with_vrs);
        assert!(gpu_manager.gpu_info.supports_variable_rate_shading);
    }

    // ============================================================================
    // 性能测试
    // ============================================================================

    #[test]
    fn test_vram_allocation_performance() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        let start = std::time::Instant::now();

        for _ in 0..10000 {
            gpu_manager.allocate_vram(1).unwrap();
            gpu_manager.free_vram(1);
        }

        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 100,
            "VRAM operations too slow: {:?}",
            duration
        );
    }

    #[test]
    fn test_feature_toggle_performance() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA")
            .with_raytracing()
            .with_mesh_shaders();
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        let start = std::time::Instant::now();

        for _ in 0..1000 {
            gpu_manager.enable_feature("occlusion_culling").ok();
            gpu_manager.disable_feature("occlusion_culling").ok();
        }

        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 50,
            "Feature toggle too slow: {:?}",
            duration
        );
    }

    #[test]
    fn test_vram_usage_calculation_performance() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        gpu_manager.allocate_vram(2048).unwrap();

        let start = std::time::Instant::now();

        for _ in 0..10000 {
            let _ = gpu_manager.get_vram_usage();
        }

        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 50,
            "VRAM usage calculation too slow: {:?}",
            duration
        );
    }

    // ============================================================================
    // 边界条件和错误处理测试
    // ============================================================================

    #[test]
    fn test_zero_vram_allocation() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        assert!(gpu_manager.allocate_vram(0).is_ok());
        assert_eq!(gpu_manager.vram_used_mb, 0);
    }

    #[test]
    fn test_exact_vram_limit() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        // 分配正好等于总VRAM的量
        let result = gpu_manager.allocate_vram(4096);
        assert!(result.is_ok());
        assert_eq!(gpu_manager.vram_used_mb, 4096);
    }

    #[test]
    fn test_one_byte_over_limit() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        gpu_manager.allocate_vram(4096).unwrap();

        // 尝试分配额外的1MB
        let result = gpu_manager.allocate_vram(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_fragmented_vram() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        // 模拟碎片化分配
        gpu_manager.allocate_vram(1000).unwrap();
        gpu_manager.allocate_vram(500).unwrap();
        gpu_manager.allocate_vram(2000).unwrap();

        assert_eq!(gpu_manager.vram_used_mb, 3500);

        // 释放中间的块
        gpu_manager.free_vram(500);
        assert_eq!(gpu_manager.vram_used_mb, 3000);

        // 应该还能分配
        assert!(gpu_manager.allocate_vram(1096).is_ok());
    }

    // ============================================================================
    // 多GPU场景测试
    // ============================================================================

    #[test]
    fn test_multiple_gpu_managers() {
        let gpu1 = TestGPUInfo::new("RTX 3080", "NVIDIA");
        let gpu2 = TestGPUInfo::new("RX 6800", "AMD");

        let mut manager1 = MockGPUManager::new(gpu1);
        let mut manager2 = MockGPUManager::new(gpu2);

        manager1.allocate_vram(2000).unwrap();
        manager2.allocate_vram(3000).unwrap();

        assert_eq!(manager1.vram_used_mb, 2000);
        assert_eq!(manager2.vram_used_mb, 3000);

        // 两个GPU管理器应该独立
        assert_ne!(manager1.vram_used_mb, manager2.vram_used_mb);
    }

    // ============================================================================
    // 自适应策略测试
    // ============================================================================

    #[test]
    fn test_adaptive_quality_low_vram() {
        let gpu_info = TestGPUInfo::new("GTX 1050", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        // 低VRAM GPU应该禁用某些特性
        assert!(!gpu_manager.occlusion_culling_enabled);
        assert!(!gpu_manager.indirect_draw_enabled);

        // 基本剔除应该仍然启用
        assert!(gpu_manager.frustum_culling_enabled);
        assert!(gpu_manager.distance_culling_enabled);
    }

    #[test]
    fn test_adaptive_quality_high_vram() {
        let gpu_info = TestGPUInfo::new("RTX 4090", "NVIDIA")
            .with_raytracing()
            .with_mesh_shaders();
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        // 高VRAM GPU可以启用所有特性
        gpu_manager.enable_feature("occlusion_culling").unwrap();
        gpu_manager.enable_feature("indirect_draw").unwrap();

        assert!(gpu_manager.frustum_culling_enabled);
        assert!(gpu_manager.occlusion_culling_enabled);
        assert!(gpu_manager.distance_culling_enabled);
        assert!(gpu_manager.indirect_draw_enabled);
    }

    // ============================================================================
    // VRAM管理策略测试
    // ============================================================================

    #[test]
    fn test_aggressive_vram_management() {
        let gpu_info = TestGPUInfo::new("GTX 1050", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        // 使用70%的VRAM
        gpu_manager.allocate_vram(2867).unwrap();
        let usage = gpu_manager.get_vram_usage();

        // 低VRAM GPU应该更激进地管理内存
        assert!(usage > 0.6 && usage < 0.8);
    }

    #[test]
    fn test_conservative_vram_management() {
        let gpu_info = TestGPUInfo::new("RTX 4090", "NVIDIA");
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        // 使用30%的VRAM
        gpu_manager.allocate_vram(12288).unwrap();
        let usage = gpu_manager.get_vram_usage();

        // 高VRAM GPU可以更保守
        assert!(usage > 0.2 && usage < 0.4);
    }
}
