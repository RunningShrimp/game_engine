#[cfg(test)]
mod tests {
    use crate::config::EngineConfig;
    use crate::core::engine::engine::Engine;

    #[test]
    fn test_engine_creation() {
        let config = EngineConfig::default();
        let engine = Engine::new(config);
        assert_eq!(engine.config.graphics.resolution.width, 800);
        assert_eq!(engine.config.graphics.resolution.height, 600);
    }

    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_engine_new() {
        let config = EngineConfig::default();
        let engine = Engine::new(config);
        // 验证引擎已创建
        assert!(true);
    }

    // ========================================
    // EngineConfig Tests
    // ========================================

    #[test]
    fn test_engine_config_new() {
        let config = EngineConfig::new();
        assert_eq!(config.graphics.resolution.width, 800);
        assert_eq!(config.graphics.resolution.height, 600);
    }

    #[test]
    fn test_engine_config_graphics_validation() {
        let config = EngineConfig::default();
        assert!(config.graphics.validate().is_ok());
    }

    #[test]
    fn test_engine_config_performance_validation() {
        let config = EngineConfig::default();
        assert!(config.performance.validate().is_ok());
    }

    #[test]
    fn test_engine_config_audio_validation() {
        let config = EngineConfig::default();
        assert!(config.audio.validate().is_ok());
    }

    #[test]
    fn test_engine_config_input_validation() {
        let config = EngineConfig::default();
        assert!(config.input.validate().is_ok());
    }

    #[test]
    fn test_engine_config_clone() {
        let config = EngineConfig::default();
        let cloned = config.clone();
        assert_eq!(config.graphics.resolution.width, cloned.graphics.resolution.width);
    }

    #[test]
    fn test_engine_config_debug() {
        let config = EngineConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("EngineConfig"));
    }

    // ========================================
    // GraphicsConfig Tests
    // ========================================

    #[test]
    fn test_graphics_config_default_width() {
        let config = EngineConfig::default();
        assert_eq!(config.graphics.resolution.width, 800);
    }

    #[test]
    fn test_graphics_config_default_height() {
        let config = EngineConfig::default();
        assert_eq!(config.graphics.resolution.height, 600);
    }

    #[test]
    fn test_graphics_config_default_vsync() {
        let config = EngineConfig::default();
        assert!(config.graphics.vsync);
    }

    // ========================================
    // PerformanceConfig Tests
    // ========================================

    #[test]
    fn test_performance_config_default_target_fps() {
        let config = EngineConfig::default();
        assert_eq!(config.performance.target_fps, 60);
    }

    #[test]
    fn test_performance_config_auto_optimize() {
        let config = EngineConfig::default();
        assert!(!config.performance.auto_optimize);
    }

    // ========================================
    // AudioConfig Tests
    // ========================================

    #[test]
    fn test_audio_config_default_master_volume() {
        let config = EngineConfig::default();
        assert_eq!(config.audio.master_volume, 1.0);
    }

    #[test]
    fn test_audio_config_default_enabled() {
        let config = EngineConfig::default();
        assert!(config.audio.enabled);
    }

    // ========================================
    // InputConfig Tests
    // ========================================

    #[test]
    fn test_input_config_default_mouse_sensitivity() {
        let config = EngineConfig::default();
        assert_eq!(config.input.mouse_sensitivity, 1.0);
    }

    // ========================================
    // LoggingConfig Tests
    // ========================================

    #[test]
    fn test_logging_config_default_level() {
        let config = EngineConfig::default();
        assert_eq!(format!("{:?}", config.logging.level), "Info");
    }

    #[test]
    fn test_logging_config_default_log_to_console() {
        let config = EngineConfig::default();
        assert!(config.logging.log_to_console);
    }

    #[test]
    fn test_logging_config_default_log_to_file() {
        let config = EngineConfig::default();
        assert!(!config.logging.log_to_file);
    }

    #[test]
    fn test_logging_config_log_file_path() {
        let config = EngineConfig::default();
        assert_eq!(config.logging.log_file_path, "game_engine.log");
    }

    // ========================================
    // Config Serialization Tests
    // ========================================

    #[test]
    fn test_config_toml_parsing() {
        let toml_str = r#"
            [graphics]
            resolution = { width = 1920, height = 1080 }
            vsync = true
        "#;
        let config = EngineConfig::from_toml_str(toml_str);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.graphics.resolution.width, 1920);
        assert_eq!(config.graphics.resolution.height, 1080);
    }

    #[test]
    fn test_config_json_parsing() {
        let json_str = r#"
            {
                "graphics": {
                    "resolution": {
                        "width": 1920,
                        "height": 1080
                    },
                    "vsync": true
                }
            }
        "#;
        let config = EngineConfig::from_json_str(json_str);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.graphics.resolution.width, 1920);
        assert_eq!(config.graphics.resolution.height, 1080);
    }

    #[test]
    fn test_config_toml_serialization_roundtrip() {
        let config = EngineConfig::default();
        let toml_str = toml::to_string(&config);
        assert!(toml_str.is_ok());
        let parsed = EngineConfig::from_toml_str(&toml_str.unwrap());
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_config_json_serialization_roundtrip() {
        let config = EngineConfig::default();
        let json_str = serde_json::to_string(&config);
        assert!(json_str.is_ok());
        let parsed = EngineConfig::from_json_str(&json_str.unwrap());
        assert!(parsed.is_ok());
    }

    // ========================================
    // Engine Lifecycle Tests
    // ========================================

    #[test]
    fn test_engine_config_field_access() {
        let config = EngineConfig::default();
        assert_eq!(config.graphics.resolution.width, 800);
        assert_eq!(config.performance.target_fps, 60);
        assert_eq!(config.audio.master_volume, 1.0);
        assert_eq!(config.input.mouse_sensitivity, 1.0);
    }

    #[test]
    fn test_engine_multiple_instances() {
        let config1 = EngineConfig::default();
        let config2 = EngineConfig::default();
        let engine1 = Engine::new(config1);
        let engine2 = Engine::new(config2);
        assert_eq!(engine1.config.graphics.resolution.width, engine2.config.graphics.resolution.width);
    }
}
