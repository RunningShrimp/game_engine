//! # Shader System Tests
//!
//! 测试着色器系统的基础功能。

use game_engine::render::shader::{Shader, ShaderStage, ShaderError};

#[test]
fn test_shader_creation() {
    let shader = Shader::new("test_shader");

    assert_eq!(shader.name(), "test_shader");
}

#[test]
fn test_shader_add_vertex_stage() {
    let mut shader = Shader::new("test_shader");

    let vertex_code = r#"
        #[vertex]
        fn vertex_main() -> [[builtin(position)]] vec4<f32> {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
    "#;

    let result = shader.add_stage(ShaderStage::Vertex, vertex_code);

    assert!(result.is_ok());
}

#[test]
fn test_shader_add_fragment_stage() {
    let mut shader = Shader::new("test_shader");

    let fragment_code = r#"
        #[fragment]
        fn fragment_main() -> [[location(0)]] vec4<f32> {
            return vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
    "#;

    let result = shader.add_stage(ShaderStage::Fragment, fragment_code);

    assert!(result.is_ok());
}

#[test]
fn test_shader_empty_code() {
    let mut shader = Shader::new("test_shader");

    let result = shader.add_stage(ShaderStage::Vertex, "");

    assert!(result.is_err());
}

#[test]
fn test_shader_get_stage() {
    let mut shader = Shader::new("test_shader");

    let vertex_code = r#"
        #[vertex]
        fn vertex_main() -> [[builtin(position)]] vec4<f32> {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
    "#;

    shader.add_stage(ShaderStage::Vertex, vertex_code).unwrap();

    let stage = shader.get_stage(ShaderStage::Vertex);

    assert!(stage.is_some());
    assert_eq!(stage.unwrap(), vertex_code);
}

#[test]
fn test_shader_remove_stage() {
    let mut shader = Shader::new("test_shader");

    let vertex_code = r#"
        #[vertex]
        fn vertex_main() -> [[builtin(position)]] vec4<f32> {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
    "#;

    shader.add_stage(ShaderStage::Vertex, vertex_code).unwrap();
    shader.remove_stage(ShaderStage::Vertex);

    let stage = shader.get_stage(ShaderStage::Vertex);

    assert!(stage.is_none());
}

#[test]
fn test_shader_has_all_stages() {
    let mut shader = Shader::new("test_shader");

    let vertex_code = r#"
        #[vertex]
        fn vertex_main() -> [[builtin(position)]] vec4<f32> {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
    "#;

    let fragment_code = r#"
        #[fragment]
        fn fragment_main() -> [[location(0)]] vec4<f32> {
            return vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
    "#;

    shader.add_stage(ShaderStage::Vertex, vertex_code).unwrap();
    shader.add_stage(ShaderStage::Fragment, fragment_code).unwrap();

    assert!(shader.has_stage(ShaderStage::Vertex));
    assert!(shader.has_stage(ShaderStage::Fragment));
    assert!(!shader.has_stage(ShaderStage::Compute));
}

#[test]
fn test_shader_clear() {
    let mut shader = Shader::new("test_shader");

    let vertex_code = r#"
        #[vertex]
        fn vertex_main() -> [[builtin(position)]] vec4<f32> {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
    "#;

    shader.add_stage(ShaderStage::Vertex, vertex_code).unwrap();
    shader.clear();

    assert!(!shader.has_stage(ShaderStage::Vertex));
}
