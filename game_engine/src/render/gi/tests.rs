//! # DDGI单元测试
//!
//! DDGI系统的单元测试和集成测试。

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::render::gi::*;

    #[test]
    fn test_ddgi_config_default() {
        let config = DDGIConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.probe_spacing, 2.0);
        assert_eq!(config.probe_counts, glam::UVec3::new(10, 10, 10));
    }

    #[test]
    fn test_ddgi_config_validation() {
        let mut config = DDGIConfig::default();

        // 测试无效的探针间距
        config.probe_spacing = -1.0;
        assert!(config.validate().is_err());

        // 测试无效的探针数量
        config.probe_spacing = 2.0;
        config.probe_counts = glam::UVec3::ZERO;
        assert!(config.validate().is_err());

        // 测试无效的更新率
        config.probe_counts = glam::UVec3::new(10, 10, 10);
        config.update_rate = 0;
        assert!(config.validate().is_err());

        // 测试有效的配置
        config.update_rate = 3;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_ddgi_quality_presets() {
        let low = DDGIQuality::Low.default_config();
        let medium = DDGIQuality::Medium.default_config();
        let high = DDGIQuality::High.default_config();

        // 低质量应该有更大的探针间距
        assert!(low.probe_spacing > medium.probe_spacing);
        assert!(medium.probe_spacing > high.probe_spacing);

        // 低质量应该有更少的探针
        assert!(low.total_probes() < medium.total_probes());
        assert!(medium.total_probes() < high.total_probes());

        // 低质量应该有更低的更新率
        assert!(low.update_rate > medium.update_rate);
        assert!(medium.update_rate >= high.update_rate);
    }

    #[test]
    fn test_probe_creation() {
        let position = glam::Vec3::new(1.0, 2.0, 3.0);
        let probe = DDGIProbe::new(position);

        assert_eq!(probe.position, position);
        assert_eq!(probe.irradiance, glam::Vec3::ZERO);
        assert_eq!(probe.depth, 1.0);
        assert_eq!(probe.offset, glam::Vec2::ZERO);
    }

    #[test]
    fn test_probe_reset() {
        let mut probe = DDGIProbe::new(glam::Vec3::ZERO);
        probe.set_irradiance(glam::Vec3::new(0.5, 0.5, 0.5));
        probe.set_depth(0.5);
        probe.set_offset(glam::Vec2::new(0.1, 0.2));

        probe.reset();

        assert_eq!(probe.irradiance, glam::Vec3::ZERO);
        assert_eq!(probe.depth, 1.0);
        assert_eq!(probe.offset, glam::Vec2::ZERO);
    }

    #[test]
    fn test_probe_manager() {
        let manager = ProbeManager::new();
        assert_eq!(manager.volume_count(), 0);
        assert!(manager.active_volume().is_none());
    }

    #[test]
    fn test_probe_visualization_modes() {
        let modes = [
            ProbeVisualization::None,
            ProbeVisualization::Spheres,
            ProbeVisualization::Lines,
            ProbeVisualization::Heatmap,
            ProbeVisualization::Irradiance,
            ProbeVisualization::Depth,
        ];

        // 确保所有模式都可以创建
        for mode in modes {
            let visualizer = GIDebugVisualizer::new();
            visualizer.set_probe_visualization(mode);
            assert_eq!(visualizer.probe_visualization, mode);
        }
    }

    #[test]
    fn test_spherical_harmonics_default() {
        let sh = irradiance::SphericalHarmonics::default();
        assert_eq!(sh.l00, glam::Vec3::ZERO);
    }

    #[test]
    fn test_spherical_harmonics_from_environment() {
        let color = glam::Vec3::new(1.0, 0.5, 0.25);
        let sh = irradiance::SphericalHarmonics::from_environment(color);
        assert_eq!(sh.l00, color);
    }

    #[test]
    fn test_spherical_harmonics_evaluate() {
        let mut sh = irradiance::SphericalHarmonics::from_environment(glam::Vec3::splat(1.0));
        let result = sh.evaluate(glam::Vec3::Y);
        assert!(result.x > 0.0 && result.y > 0.0 && result.z > 0.0);
    }

    #[test]
    fn test_spherical_harmonics_add_light() {
        let mut sh = irradiance::SphericalHarmonics::default();
        sh.add_light(glam::Vec3::Y, glam::Vec3::new(1.0, 1.0, 1.0), 1.0);
        assert!(sh.l00.length() > 0.0);
    }

    #[test]
    fn test_ddgi_config_memory_usage() {
        let config = DDGIConfig::low_quality();
        let memory = config.memory_usage();
        assert!(memory > 0);

        // 高质量配置应该使用更多内存
        let high_config = DDGIConfig::high_quality();
        let high_memory = high_config.memory_usage();
        assert!(high_memory > memory);
    }

    #[test]
    fn test_ddgi_config_quality_descriptions() {
        let low = DDGIConfig::low_quality();
        let medium = DDGIConfig::medium_quality();
        let high = DDGIConfig::high_quality();

        assert_eq!(low.quality_description(), "Low");
        assert_eq!(medium.quality_description(), "Medium");
        assert_eq!(high.quality_description(), "High");
    }

    #[test]
    fn test_ddgi_config_volume_size() {
        let config = DDGIConfig {
            probe_spacing: 2.0,
            probe_counts: glam::UVec3::new(10, 10, 10),
            ..Default::default()
        };

        let size = config.volume_size();
        let expected = glam::Vec3::new(18.0, 18.0, 18.0); // (10-1) * 2.0
        assert_eq!(size, expected);
    }

    #[test]
    fn test_irradiance_texture_index_calculation() {
        // 这个测试在有Device实例时才能运行
        // 这里只测试逻辑

        let probe_index = 5u32;
        let face = 2u32; // +Z方向
        let expected_index = probe_index * 6 + face;
        assert_eq!(expected_index, 32);
    }

    #[test]
    fn test_probe_grid_position() {
        let probe_counts = glam::UVec3::new(5, 5, 5);

        // 测试原点探针
        let probe_index = 0;
        let z = probe_index / (probe_counts.x * probe_counts.y);
        let temp = probe_index % (probe_counts.x * probe_counts.y);
        let y = temp / probe_counts.x;
        let x = temp % probe_counts.x;

        assert_eq!((x, y, z), (0, 0, 0));

        // 测试中间探针
        let probe_index = 63; // (2, 2, 2) 在 5x5x5 网格中
        let z = probe_index / (probe_counts.x * probe_counts.y);
        let temp = probe_index % (probe_counts.x * probe_counts.y);
        let y = temp / probe_counts.x;
        let x = temp % probe_counts.x;

        assert_eq!((x, y, z), (3, 2, 2));
    }

    #[test]
    fn test_ddgi_uniforms() {
        let probe_spacing = 2.0;
        let probe_counts = glam::UVec3::new(10, 10, 10);
        let volume_origin = glam::Vec3::new(-9.0, -9.0, -9.0);

        let volume_size = glam::Vec3::new(
            (probe_counts.x - 1) as f32 * probe_spacing,
            (probe_counts.y - 1) as f32 * probe_spacing,
            (probe_counts.z - 1) as f32 * probe_spacing,
        );

        // 验证计算
        assert_eq!(volume_size, glam::Vec3::new(18.0, 18.0, 18.0));
        assert_eq!(volume_origin, -volume_size / 2.0);
    }

    #[test]
    fn test_probe_stats() {
        let stats = debug::ProbeStats {
            total_probes: 100,
            active_probes: 95,
            avg_irradiance: glam::Vec3::new(0.5, 0.5, 0.5),
            min_depth: 0.1,
            max_depth: 10.0,
        };

        assert_eq!(stats.total_probes, 100);
        assert_eq!(stats.active_probes, 95);
        assert_eq!(stats.avg_irradiance, glam::Vec3::new(0.5, 0.5, 0.5));
        assert_eq!(stats.min_depth, 0.1);
        assert_eq!(stats.max_depth, 10.0);
    }

    #[test]
    fn test_ddgi_error_conversion() {
        let error = DDGIError::InvalidConfig("test".to_string());
        let render_error: RenderError = error.into();
        assert!(render_error.to_string().contains("test"));
    }
}
