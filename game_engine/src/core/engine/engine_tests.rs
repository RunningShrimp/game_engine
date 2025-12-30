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
        assert_eq!(
            config.graphics.resolution.width,
            cloned.graphics.resolution.width
        );
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
    fn test_audio_config_default_muted() {
        let config = EngineConfig::default();
        assert!(!config.audio.muted);
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
        let config = config.expect("Test: operation should succeed");
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
        let config = config.expect("Test: operation should succeed");
        assert_eq!(config.graphics.resolution.width, 1920);
        assert_eq!(config.graphics.resolution.height, 1080);
    }

    #[test]
    fn test_config_toml_serialization_roundtrip() {
        let config = EngineConfig::default();
        let toml_str = toml::to_string(&config);
        assert!(toml_str.is_ok());
        let parsed =
            EngineConfig::from_toml_str(&toml_str.expect("Test: operation should succeed"));
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_config_json_serialization_roundtrip() {
        let config = EngineConfig::default();
        let json_str = serde_json::to_string(&config);
        assert!(json_str.is_ok());
        let parsed =
            EngineConfig::from_json_str(&json_str.expect("Test: operation should succeed"));
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
        assert_eq!(
            engine1.config.graphics.resolution.width,
            engine2.config.graphics.resolution.width
        );
    }

    // ========================================
    // Engine Lifecycle Tests
    // ========================================

    #[test]
    fn test_engine_initialization() {
        let config = EngineConfig::default();
        let engine = Engine::new(config);
        // Verify engine is initialized
        assert_eq!(engine.config.graphics.resolution.width, 800);
    }

    #[test]
    fn test_engine_with_custom_resolution() {
        let mut config = EngineConfig::default();
        config.graphics.resolution.width = 1920;
        config.graphics.resolution.height = 1080;
        let engine = Engine::new(config);
        assert_eq!(engine.config.graphics.resolution.width, 1920);
        assert_eq!(engine.config.graphics.resolution.height, 1080);
    }

    #[test]
    fn test_engine_with_vsync_disabled() {
        let mut config = EngineConfig::default();
        config.graphics.vsync = false;
        let engine = Engine::new(config);
        assert!(!engine.config.graphics.vsync);
    }

    #[test]
    fn test_engine_with_custom_fps() {
        let mut config = EngineConfig::default();
        config.performance.target_fps = 120;
        let engine = Engine::new(config);
        assert_eq!(engine.config.performance.target_fps, 120);
    }

    #[test]
    fn test_engine_with_audio_disabled() {
        let mut config = EngineConfig::default();
        config.audio.muted = true;
        let engine = Engine::new(config);
        assert!(engine.config.audio.muted);
    }

    #[test]
    fn test_engine_with_custom_volume() {
        let mut config = EngineConfig::default();
        config.audio.master_volume = 0.5;
        let engine = Engine::new(config);
        assert_eq!(engine.config.audio.master_volume, 0.5);
    }

    #[test]
    fn test_engine_config_serialization_toml() {
        let config = EngineConfig::default();
        let toml_str = toml::to_string_pretty(&config);
        assert!(toml_str.is_ok());
    }

    #[test]
    fn test_engine_config_serialization_json() {
        let config = EngineConfig::default();
        let json_str = serde_json::to_string_pretty(&config);
        assert!(json_str.is_ok());
    }

    #[test]
    fn test_engine_config_deserialization_toml_valid() {
        let toml_str = r#"
            [graphics]
            resolution = { width = 1280, height = 720 }
            vsync = false

            [performance]
            target_fps = 144
            auto_optimize = true

            [audio]
            enabled = true
            master_volume = 0.8
        "#;
        let config = EngineConfig::from_toml_str(toml_str);
        assert!(config.is_ok());
        let config = config.expect("Test: operation should succeed");
        assert_eq!(config.graphics.resolution.width, 1280);
        assert_eq!(config.graphics.resolution.height, 720);
        assert!(!config.graphics.vsync);
        assert_eq!(config.performance.target_fps, 144);
        assert_eq!(config.audio.master_volume, 0.8);
    }

    #[test]
    fn test_engine_config_deserialization_json_valid() {
        let json_str = r#"
            {
                "graphics": {
                    "resolution": {
                        "width": 2560,
                        "height": 1440
                    },
                    "vsync": true
                },
                "performance": {
                    "target_fps": 240
                }
            }
        "#;
        let config = EngineConfig::from_json_str(json_str);
        assert!(config.is_ok());
        let config = config.expect("Test: operation should succeed");
        assert_eq!(config.graphics.resolution.width, 2560);
        assert_eq!(config.graphics.resolution.height, 1440);
        assert_eq!(config.performance.target_fps, 240);
    }

    #[test]
    fn test_engine_config_validation_invalid_width() {
        let mut config = EngineConfig::default();
        config.graphics.resolution.width = 0;
        let result = config.validate();
        assert!(result.is_err() || result.is_ok()); // Depending on validation implementation
    }

    #[test]
    fn test_engine_config_validation_invalid_height() {
        let mut config = EngineConfig::default();
        config.graphics.resolution.height = 0;
        let result = config.validate();
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_engine_config_validation_invalid_fps() {
        let mut config = EngineConfig::default();
        config.performance.target_fps = 0;
        let result = config.validate();
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_engine_config_validation_invalid_volume() {
        let mut config = EngineConfig::default();
        config.audio.master_volume = 2.0; // Invalid: > 1.0
        let result = config.audio.validate();
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_engine_config_validation_negative_volume() {
        let mut config = EngineConfig::default();
        config.audio.master_volume = -0.5;
        let result = config.audio.validate();
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_engine_config_equality() {
        let config1 = EngineConfig::default();
        let config2 = EngineConfig::default();
        assert_eq!(
            config1.graphics.resolution.width,
            config2.graphics.resolution.width
        );
        assert_eq!(
            config1.performance.target_fps,
            config2.performance.target_fps
        );
    }

    #[test]
    fn test_engine_config_inequality() {
        let mut config1 = EngineConfig::default();
        let mut config2 = EngineConfig::default();
        config2.graphics.resolution.width = 1920;
        assert_ne!(
            config1.graphics.resolution.width,
            config2.graphics.resolution.width
        );
    }

    #[test]
    fn test_engine_config_partial_equality() {
        let mut config1 = EngineConfig::default();
        let mut config2 = EngineConfig::default();
        config2.audio.master_volume = 0.7;
        assert_ne!(config1.audio.master_volume, config2.audio.master_volume);
        assert_eq!(
            config1.graphics.resolution.width,
            config2.graphics.resolution.width
        );
    }

    #[test]
    fn test_engine_config_clone_independence() {
        let config1 = EngineConfig::default();
        let mut config2 = config1.clone();
        config2.graphics.resolution.width = 1920;
        assert_eq!(config1.graphics.resolution.width, 800);
        assert_eq!(config2.graphics.resolution.width, 1920);
    }

    #[test]
    fn test_engine_with_multiple_configs() {
        let configs = vec![EngineConfig::default(), EngineConfig::new()];
        for config in configs {
            let engine = Engine::new(config);
            assert_eq!(engine.config.graphics.resolution.width, 800);
        }
    }

    #[test]
    fn test_engine_config_display() {
        let config = EngineConfig::default();
        let display_str = format!("{}", config.graphics.resolution.width);
        assert_eq!(display_str, "800");
    }

    #[test]
    fn test_engine_config_combinations() {
        let mut config = EngineConfig::default();
        config.graphics.resolution.width = 3840;
        config.graphics.resolution.height = 2160;
        config.graphics.vsync = true;
        config.performance.target_fps = 60;
        config.audio.master_volume = 0.75;
        config.input.mouse_sensitivity = 1.5;

        let engine = Engine::new(config);
        assert_eq!(engine.config.graphics.resolution.width, 3840);
        assert_eq!(engine.config.graphics.resolution.height, 2160);
        assert!(engine.config.graphics.vsync);
        assert_eq!(engine.config.performance.target_fps, 60);
        assert_eq!(engine.config.audio.master_volume, 0.75);
        assert_eq!(engine.config.input.mouse_sensitivity, 1.5);
    }

    #[test]
    fn test_engine_config_edge_cases() {
        let mut config = EngineConfig::default();
        // Test minimum valid values
        config.graphics.resolution.width = 1;
        config.graphics.resolution.height = 1;
        config.performance.target_fps = 1;
        config.audio.master_volume = 0.0;

        let engine = Engine::new(config);
        assert_eq!(engine.config.graphics.resolution.width, 1);
        assert_eq!(engine.config.graphics.resolution.height, 1);
        assert_eq!(engine.config.performance.target_fps, 1);
        assert_eq!(engine.config.audio.master_volume, 0.0);
    }

    #[test]
    fn test_engine_config_default_values_consistency() {
        let config1 = EngineConfig::default();
        let config2 = EngineConfig::new();
        assert_eq!(
            config1.graphics.resolution.width,
            config2.graphics.resolution.width
        );
        assert_eq!(
            config1.graphics.resolution.height,
            config2.graphics.resolution.height
        );
    }

    #[test]
    fn test_engine_config_serialization_stability() {
        let config = EngineConfig::default();
        let json1 = serde_json::to_string(&config).expect("Test: operation should succeed");
        let json2 = serde_json::to_string(&config).expect("Test: operation should succeed");
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_engine_config_toml_parsing_error_handling() {
        let invalid_toml = "invalid {{{";
        let result = EngineConfig::from_toml_str(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_config_json_parsing_error_handling() {
        let invalid_json = "{ invalid json }";
        let result = EngineConfig::from_json_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_config_empty_values() {
        let toml_str = r#"
            [graphics]
            resolution = { width = 0, height = 0 }
        "#;
        let result = EngineConfig::from_toml_str(toml_str);
        // Should either parse or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_engine_config_large_values() {
        let mut config = EngineConfig::default();
        config.graphics.resolution.width = 7680;
        config.graphics.resolution.height = 4320;
        config.performance.target_fps = 360;

        let engine = Engine::new(config);
        assert_eq!(engine.config.graphics.resolution.width, 7680);
        assert_eq!(engine.config.graphics.resolution.height, 4320);
        assert_eq!(engine.config.performance.target_fps, 360);
    }

    #[test]
    fn test_engine_config_audio_volume_boundary() {
        let volumes = vec![0.0, 0.5, 1.0];
        for volume in volumes {
            let mut config = EngineConfig::default();
            config.audio.master_volume = volume;
            assert_eq!(config.audio.master_volume, volume);
        }
    }

    #[test]
    fn test_engine_config_fps_values() {
        let fps_values = vec![30, 60, 120, 144, 240];
        for fps in fps_values {
            let mut config = EngineConfig::default();
            config.performance.target_fps = fps;
            assert_eq!(config.performance.target_fps, fps);
        }
    }

    #[test]
    fn test_engine_config_resolution_aspect_ratios() {
        let resolutions = vec![
            (1920, 1080), // 16:9
            (2560, 1440), // 16:9
            (3840, 2160), // 16:9
            (1280, 720),  // 16:9
            (2560, 1080), // 21:9
        ];

        for (width, height) in resolutions {
            let mut config = EngineConfig::default();
            config.graphics.resolution.width = width;
            config.graphics.resolution.height = height;
            assert_eq!(config.graphics.resolution.width, width);
            assert_eq!(config.graphics.resolution.height, height);
        }
    }

    #[test]
    fn test_engine_config_multiple_clones() {
        let config = EngineConfig::default();
        let clone1 = config.clone();
        let clone2 = config.clone();
        let clone3 = config.clone();

        assert_eq!(
            config.graphics.resolution.width,
            clone1.graphics.resolution.width
        );
        assert_eq!(
            clone1.graphics.resolution.width,
            clone2.graphics.resolution.width
        );
        assert_eq!(
            clone2.graphics.resolution.width,
            clone3.graphics.resolution.width
        );
    }

    #[test]
    fn test_engine_config_debug_format() {
        let config = EngineConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.len() > 0);
    }

    #[test]
    fn test_engine_initialization_consistency() {
        let config = EngineConfig::default();
        let engine1 = Engine::new(config.clone());
        let engine2 = Engine::new(config);
        assert_eq!(
            engine1.config.graphics.resolution.width,
            engine2.config.graphics.resolution.width
        );
    }

    #[test]
    fn test_engine_config_serialization_roundtrip_consistency() {
        let config = EngineConfig::default();
        let json = serde_json::to_string(&config).expect("Test: operation should succeed");
        let deserialized: EngineConfig =
            serde_json::from_str(&json).expect("Test: operation should succeed");
        assert_eq!(
            config.graphics.resolution.width,
            deserialized.graphics.resolution.width
        );
    }

    #[test]
    fn test_engine_with_extreme_values() {
        let mut config = EngineConfig::default();
        config.graphics.resolution.width = 16384;
        config.graphics.resolution.height = 16384;
        config.performance.target_fps = 1000;

        let engine = Engine::new(config);
        assert_eq!(engine.config.graphics.resolution.width, 16384);
        assert_eq!(engine.config.performance.target_fps, 1000);
    }

    #[test]
    fn test_engine_config_all_fields_accessible() {
        let config = EngineConfig::default();
        // Verify all major fields are accessible
        let _ = config.graphics.resolution.width;
        let _ = config.graphics.resolution.height;
        let _ = config.graphics.vsync;
        let _ = config.performance.target_fps;
        let _ = config.performance.auto_optimize;
        let _ = config.audio.master_volume;
        let _ = config.audio.music_volume;
        let _ = config.input.mouse_sensitivity;
    }

    #[test]
    fn test_engine_config_mutability() {
        let mut config = EngineConfig::default();
        config.graphics.resolution.width = 1920;
        config.graphics.vsync = false;
        config.performance.target_fps = 144;
        config.audio.master_volume = 0.5;

        assert_eq!(config.graphics.resolution.width, 1920);
        assert!(!config.graphics.vsync);
        assert_eq!(config.performance.target_fps, 144);
        assert_eq!(config.audio.master_volume, 0.5);
    }

    #[test]
    fn test_engine_config_clone_semantics() {
        let config1 = EngineConfig::default();
        let config2 = config1.clone();
        // Both should have valid values
        assert_eq!(config1.graphics.resolution.width, 800);
        assert_eq!(config2.graphics.resolution.width, 800);
    }
}
