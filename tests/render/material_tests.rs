//! # Material System Tests
//!
//! 测试材质系统的基础功能。

use game_engine::render::material::Material;
use game_engine::render::texture::TextureFormat;

#[test]
fn test_material_creation() {
    let material = Material::new();

    assert!(material.name().is_empty());
}

#[test]
fn test_material_set_name() {
    let mut material = Material::new();
    material.set_name("test_material");

    assert_eq!(material.name(), "test_material");
}

#[test]
fn test_material_set_albedo() {
    let mut material = Material::new();
    material.set_albedo([1.0, 0.0, 0.0, 1.0]);

    assert_eq!(material.albedo(), [1.0, 0.0, 0.0, 1.0]);
}

#[test]
fn test_material_set_metallic() {
    let mut material = Material::new();
    material.set_metallic(0.8);

    assert_eq!(material.metallic(), 0.8);
}

#[test]
fn test_material_set_roughness() {
    let mut material = Material::new();
    material.set_roughness(0.5);

    assert_eq!(material.roughness(), 0.5);
}

#[test]
fn test_material_set_emissive() {
    let mut material = Material::new();
    material.set_emissive([1.0, 1.0, 0.0]);

    assert_eq!(material.emissive(), [1.0, 1.0, 0.0]);
}

#[test]
fn test_material_metallic_clamp() {
    let mut material = Material::new();
    material.set_metallic(1.5); // 应该被钳制到[0, 1]

    assert!(material.metallic() >= 0.0 && material.metallic() <= 1.0);
}

#[test]
fn test_material_roughness_clamp() {
    let mut material = Material::new();
    material.set_roughness(-0.5); // 应该被钳制到[0, 1]

    assert!(material.roughness() >= 0.0 && material.roughness() <= 1.0);
}

#[test]
fn test_material_texture_slot() {
    let mut material = Material::new();
    material.set_texture("albedo", "test.png");

    assert_eq!(material.texture("albedo"), Some("test.png"));
}

#[test]
fn test_material_clear() {
    let mut material = Material::new();
    material.set_name("test");
    material.set_albedo([1.0, 0.0, 0.0, 1.0]);

    material.clear();

    assert!(material.name().is_empty());
    assert_eq!(material.albedo(), [1.0, 1.0, 1.0, 1.0]); // 默认白色
}

#[test]
fn test_material_clone() {
    let mut material = Material::new();
    material.set_name("test");
    material.set_albedo([1.0, 0.0, 0.0, 1.0]);

    let cloned = material.clone();

    assert_eq!(cloned.name(), "test");
    assert_eq!(cloned.albedo(), [1.0, 0.0, 0.0, 1.0]);
}

#[test]
fn test_material_default_values() {
    let material = Material::new();

    // 默认值应该是白色、非金属、中等粗糙度
    assert_eq!(material.albedo(), [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(material.metallic(), 0.0);
    assert_eq!(material.roughness(), 0.5);
}
