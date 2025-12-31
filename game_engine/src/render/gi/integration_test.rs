//! # DDGI集成测试
//!
//! 测试DDGI系统的完整功能。

#[cfg(test)]
mod integration_tests {
    use super::super::*;

    #[test]
    fn test_ddgi_full_workflow() {
        // 测试完整的DDGI工作流程

        // 1. 创建配置
        let config = DDGIConfig::medium_quality();
        assert!(config.validate().is_ok());

        // 2. 验证配置参数
        assert_eq!(config.probe_spacing, 2.0);
        assert_eq!(config.probe_counts, glam::UVec3::new(10, 10, 10));
        assert_eq!(config.total_probes(), 1000);

        // 3. 测试质量描述
        assert_eq!(config.quality_description(), "Medium");

        // 4. 测试内存计算
        let memory = config.memory_usage();
        assert!(memory > 0);
        println!("Memory usage: {} MB", memory / (1024 * 1024));
    }

    #[test]
    fn test_probe_manager_workflow() {
        // 测试探针管理器工作流程

        let manager = ProbeManager::new();
        assert_eq!(manager.volume_count(), 0);

        // 注意：实际的DDGIVolume创建需要Device，这里只测试管理器逻辑
        assert!(manager.active_volume().is_none());
    }

    #[test]
    fn test_debug_visualizer_workflow() {
        // 测试调试可视化器工作流程

        let visualizer = GIDebugVisualizer::new();
        assert!(visualizer.show_probes);

        // 测试设置
        visualizer.set_show_probes(false);
        visualizer.set_show_irradiance(true);
        visualizer.set_probe_visualization(ProbeVisualization::Heatmap);
    }

    #[test]
    fn test_quality_presets() {
        // 测试所有质量预设

        let low = DDGIConfig::low_quality();
        let medium = DDGIConfig::medium_quality();
        let high = DDGIConfig::high_quality();

        // 验证质量级别
        assert_eq!(low.quality_description(), "Low");
        assert_eq!(medium.quality_description(), "Medium");
        assert_eq!(high.quality_description(), "High");

        // 验证探针数量递增
        assert!(low.total_probes() < medium.total_probes());
        assert!(medium.total_probes() < high.total_probes());

        // 验证更新率
        assert!(low.update_rate > medium.update_rate);
        assert!(medium.update_rate >= high.update_rate);
    }

    #[test]
    fn test_spherical_harmonics_workflow() {
        // 测试球谐函数工作流程

        let mut sh = irradiance::SphericalHarmonics::default();

        // 添加光源
        sh.add_light(glam::Vec3::Y, glam::Vec3::new(1.0, 1.0, 1.0), 1.0);
        assert!(sh.l00.length() > 0.0);

        // 评估方向
        let result = sh.evaluate(glam::Vec3::Y);
        assert!(result.length() > 0.0);

        // 转换为数组
        let array = sh.to_vec4_array();
        assert_eq!(array.len(), 3);
    }

    #[test]
    fn test_ddgi_error_handling() {
        // 测试错误处理

        // 无效配置
        let mut config = DDGIConfig::default();
        config.probe_spacing = 0.0;
        assert!(config.validate().is_err());

        config.probe_spacing = 2.0;
        config.probe_counts = glam::UVec3::ZERO;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_memory_estimates() {
        // 测试内存估算

        let configs = vec![
            DDGIConfig::low_quality(),
            DDGIConfig::medium_quality(),
            DDGIConfig::high_quality(),
        ];

        for (i, config) in configs.iter().enumerate() {
            let memory = config.memory_usage();
            let memory_mb = memory / (1024 * 1024);

            println!(
                "Config {}: {} probes, {} MB",
                i,
                config.total_probes(),
                memory_mb
            );

            // 验证内存递增
            if i > 0 {
                let prev_memory = configs[i - 1].memory_usage();
                assert!(memory > prev_memory);
            }
        }
    }

    #[test]
    fn test_volume_size_calculation() {
        // 测试体积大小计算

        let config = DDGIConfig {
            probe_spacing: 2.0,
            probe_counts: glam::UVec3::new(10, 10, 10),
            ..Default::default()
        };

        let size = config.volume_size();
        let expected = glam::Vec3::new(18.0, 18.0, 18.0); // (10-1) * 2.0

        assert_eq!(size, expected);
    }
}
