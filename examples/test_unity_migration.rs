//! Unity迁移工具功能测试
//!
//! 快速测试Unity迁移工具的各个组件。

use game_engine::tools::migration::{
    APIMappingTable, ComponentMappingRegistry, UnityComponentType,
    UnityAPICategory
};

fn main() {
    println!("=== Unity迁移工具功能测试 ===\n");

    // 测试1: API映射表
    test_api_mapping();

    // 测试2: 组件映射
    test_component_mapping();

    // 测试3: API转换
    test_api_conversion();

    println!("✅ 所有测试通过!");
}

fn test_api_mapping() {
    println!("📋 测试1: API映射表");
    println!("════════════════════════════════════════════════════════");

    let api_table = APIMappingTable::new();

    // 测试GameObject API
    assert!(
        api_table.get_mapping("GameObject.Find").is_some(),
        "GameObject.Find映射应该存在"
    );
    println!("✓ GameObject.Find → find_entity");

    // 测试Transform API
    assert!(
        api_table.get_mapping("transform.position").is_some(),
        "transform.position映射应该存在"
    );
    println!("✓ transform.position → translation");

    // 测试Input API
    assert!(
        api_table.get_mapping("Input.GetAxis").is_some(),
        "Input.GetAxis映射应该存在"
    );
    println!("✓ Input.GetAxis → get_axis");

    println!();
}

fn test_component_mapping() {
    println!("📦 测试2: 组件映射");
    println!("════════════════════════════════════════════════════════");

    let registry = ComponentMappingRegistry::new();

    // 测试Transform组件
    let transform_mapping = registry.get_mapping(&UnityComponentType::Transform);
    assert!(transform_mapping.is_some(), "Transform映射应该存在");
    assert_eq!(transform_mapping.unwrap().engine_component, "Transform");
    println!("✓ Transform → Transform");

    // 测试Rigidbody组件
    let rb_mapping = registry.get_mapping(&UnityComponentType::Rigidbody);
    assert!(rb_mapping.is_some(), "Rigidbody映射应该存在");
    assert_eq!(rb_mapping.unwrap().engine_component, "RigidBody");
    println!("✓ Rigidbody → RigidBody");

    // 测试支持检查
    assert!(registry.is_supported(&UnityComponentType::Camera));
    println!("✓ Camera组件支持检查");

    println!();
}

fn test_api_conversion() {
    println!("🔄 测试3: API转换");
    println!("════════════════════════════════════════════════════════");

    let api_table = APIMappingTable::new();

    // 测试直接映射
    let converted = api_table.convert_api("Camera.main");
    assert_eq!(converted, Some("primary_camera".to_string()));
    println!("✓ Camera.main → primary_camera (直接映射)");

    // 测试方法调用映射
    let converted = api_table.convert_api("GameObject.Find");
    assert_eq!(converted, Some("find_entity".to_string()));
    println!("✓ GameObject.Find → find_entity (方法调用)");

    // 测试属性访问器映射
    let converted = api_table.convert_api("transform.position");
    assert!(converted.is_some());
    println!("✓ transform.position → get_translation() (属性访问器)");

    println!();
}
