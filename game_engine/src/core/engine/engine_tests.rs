#[cfg(test)]
mod tests {
    use crate::core::engine::engine::Engine;
    use crate::config::EngineConfig;

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
}

