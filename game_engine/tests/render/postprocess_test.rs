//! 后处理效果测试
//!
//! 测试后处理系统的核心功能，包括配置、参数验证等。

use game_engine::render::postprocess::{AntialiasingMode, FxaaQuality, PostProcessConfig, TonemapOperator};

#[test]
fn test_postprocess_config_default() {
    let config = PostProcessConfig::default();
    
    assert_eq!(config.antialiasing, AntialiasingMode::FXAA);
    assert_eq!(config.fxaa_quality, FxaaQuality::Medium);
    assert!(config.bloom_enabled);
    assert_eq!(config.bloom_intensity, 0.5);
    assert_eq!(config.bloom_threshold, 1.0);
    assert!(!config.ssao_enabled);
    assert!(config.tonemap_enabled);
    assert_eq!(config.tonemap_operator, TonemapOperator::ACES);
    assert_eq!(config.exposure, 1.0);
    assert_eq!(config.gamma, 2.2);
}

#[test]
fn test_postprocess_config_bloom_parameters() {
    let mut config = PostProcessConfig::default();
    
    // 测试Bloom参数的有效范围
    config.bloom_intensity = 0.0;
    assert_eq!(config.bloom_intensity, 0.0);
    
    config.bloom_intensity = 2.0;
    assert_eq!(config.bloom_intensity, 2.0);
    
    config.bloom_threshold = 0.5;
    assert_eq!(config.bloom_threshold, 0.5);
    
    config.bloom_radius = 10.0;
    assert_eq!(config.bloom_radius, 10.0);
}

#[test]
fn test_postprocess_config_ssao_parameters() {
    let mut config = PostProcessConfig::default();
    
    config.ssao_enabled = true;
    assert!(config.ssao_enabled);
    
    config.ssao_radius = 1.0;
    assert_eq!(config.ssao_radius, 1.0);
    
    config.ssao_intensity = 2.0;
    assert_eq!(config.ssao_intensity, 2.0);
    
    config.ssao_bias = 0.05;
    assert_eq!(config.ssao_bias, 0.05);
}

#[test]
fn test_postprocess_config_tonemap_operators() {
    let mut config = PostProcessConfig::default();
    
    // 测试不同的色调映射算法
    config.tonemap_operator = TonemapOperator::Reinhard;
    assert_eq!(config.tonemap_operator, TonemapOperator::Reinhard);
    
    config.tonemap_operator = TonemapOperator::ACES;
    assert_eq!(config.tonemap_operator, TonemapOperator::ACES);
    
    config.tonemap_operator = TonemapOperator::Filmic;
    assert_eq!(config.tonemap_operator, TonemapOperator::Filmic);
}

#[test]
fn test_postprocess_config_exposure() {
    let mut config = PostProcessConfig::default();
    
    config.exposure = 0.5;
    assert_eq!(config.exposure, 0.5);
    
    config.exposure = 2.0;
    assert_eq!(config.exposure, 2.0);
}

#[test]
fn test_postprocess_config_gamma() {
    let mut config = PostProcessConfig::default();
    
    config.gamma = 1.8;
    assert_eq!(config.gamma, 1.8);
    
    config.gamma = 2.4;
    assert_eq!(config.gamma, 2.4);
}

#[test]
fn test_postprocess_config_antialiasing_modes() {
    let mut config = PostProcessConfig::default();
    
    config.antialiasing = AntialiasingMode::None;
    assert_eq!(config.antialiasing, AntialiasingMode::None);
    
    config.antialiasing = AntialiasingMode::FXAA;
    assert_eq!(config.antialiasing, AntialiasingMode::FXAA);
    
    config.antialiasing = AntialiasingMode::TAA;
    assert_eq!(config.antialiasing, AntialiasingMode::TAA);
}

#[test]
fn test_postprocess_config_fxaa_quality() {
    let mut config = PostProcessConfig::default();
    
    config.fxaa_quality = FxaaQuality::Low;
    assert_eq!(config.fxaa_quality, FxaaQuality::Low);
    
    config.fxaa_quality = FxaaQuality::Medium;
    assert_eq!(config.fxaa_quality, FxaaQuality::Medium);
    
    config.fxaa_quality = FxaaQuality::High;
    assert_eq!(config.fxaa_quality, FxaaQuality::High);
    
    config.fxaa_quality = FxaaQuality::Ultra;
    assert_eq!(config.fxaa_quality, FxaaQuality::Ultra);
}

#[test]
fn test_postprocess_config_custom() {
    let config = PostProcessConfig {
        antialiasing: AntialiasingMode::TAA,
        fxaa_quality: FxaaQuality::High,
        bloom_enabled: false,
        bloom_intensity: 0.8,
        bloom_threshold: 1.5,
        bloom_radius: 8.0,
        ssao_enabled: true,
        ssao_radius: 0.8,
        ssao_intensity: 1.5,
        ssao_bias: 0.03,
        tonemap_enabled: true,
        tonemap_operator: TonemapOperator::Filmic,
        exposure: 1.2,
        gamma: 2.0,
    };
    
    assert_eq!(config.antialiasing, AntialiasingMode::TAA);
    assert_eq!(config.fxaa_quality, FxaaQuality::High);
    assert!(!config.bloom_enabled);
    assert!(config.ssao_enabled);
    assert_eq!(config.tonemap_operator, TonemapOperator::Filmic);
}

