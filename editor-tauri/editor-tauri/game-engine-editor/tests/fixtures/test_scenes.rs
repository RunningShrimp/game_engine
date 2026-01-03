// Test Scenes
// 提供测试场景和配置

use crate::fixtures::test_entities::{TestEntity, TestMaterial, TestMesh, TestSceneConfig};

/// 测试场景
#[derive(Debug, Clone)]
pub struct TestScene {
    pub config: TestSceneConfig,
    pub entities: Vec<TestEntity>,
    pub materials: Vec<TestMaterial>,
    pub meshes: Vec<TestMesh>,
}

impl TestScene {
    pub fn new(config: TestSceneConfig) -> Self {
        Self {
            config,
            entities: Vec::new(),
            materials: Vec::new(),
            meshes: Vec::new(),
        }
    }

    pub fn with_entities(mut self, entities: Vec<TestEntity>) -> Self {
        self.entities = entities;
        self
    }

    pub fn with_materials(mut self, materials: Vec<TestMaterial>) -> Self {
        self.materials = materials;
        self
    }

    pub fn with_meshes(mut self, meshes: Vec<TestMesh>) -> Self {
        self.meshes = meshes;
        self
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }
}

/// 场景构建器
pub struct SceneBuilder {
    config: TestSceneConfig,
    entities: Vec<TestEntity>,
    materials: Vec<TestMaterial>,
    meshes: Vec<TestMesh>,
}

impl SceneBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            config: TestSceneConfig::new(name),
            entities: Vec::new(),
            materials: Vec::new(),
            meshes: Vec::new(),
        }
    }

    pub fn with_config(mut self, config: TestSceneConfig) -> Self {
        self.config = config;
        self
    }

    pub fn add_entity(mut self, entity: TestEntity) -> Self {
        self.entities.push(entity);
        self
    }

    pub fn add_material(mut self, material: TestMaterial) -> Self {
        self.materials.push(material);
        self
    }

    pub fn add_mesh(mut self, mesh: TestMesh) -> Self {
        self.meshes.push(mesh);
        self
    }

    pub fn build(self) -> TestScene {
        TestScene {
            config: self.config,
            entities: self.entities,
            materials: self.materials,
            meshes: self.meshes,
        }
    }
}

/// 预定义场景：简单场景
pub fn create_simple_scene() -> TestScene {
    let entities = vec![
        TestEntity::new(0, "player").with_position(0.0, 1.0, 0.0),
        TestEntity::new(1, "ground").with_scale(10.0, 1.0, 10.0),
    ];

    let materials = vec![
        TestMaterial::new(0, "player_mat").with_albedo(0.2, 0.6, 1.0),
        TestMaterial::new(1, "ground_mat").with_albedo(0.5, 0.5, 0.5),
    ];

    let meshes = vec![
        TestMesh::new(0, "player_mesh", 1000, 500),
        TestMesh::new(1, "ground_mesh", 4, 2),
    ];

    TestScene {
        config: TestSceneConfig::new("simple_scene")
            .with_entity_count(2)
            .with_light_count(1),
        entities,
        materials,
        meshes,
    }
}

/// 预定义场景：复杂场景
pub fn create_complex_scene() -> TestScene {
    let mut entities = Vec::new();
    let mut materials = Vec::new();
    let mut meshes = Vec::new();

    // 主角
    entities.push(TestEntity::new(0, "hero").with_position(0.0, 1.0, 0.0));
    materials.push(TestMaterial::new(0, "hero_mat").with_albedo(1.0, 0.2, 0.2));
    meshes.push(TestMesh::new(0, "hero_mesh", 5000, 2500));

    // 多个NPC
    for i in 1..=5 {
        let angle = (i as f32 / 5.0) * std::f32::consts::PI * 2.0;
        let x = angle.cos() * 3.0;
        let z = angle.sin() * 3.0;

        entities.push(
            TestEntity::new(i, &format!("npc_{}", i))
                .with_position(x, 1.0, z),
        );
        materials.push(
            TestMaterial::new(i, &format!("npc_{}_mat", i))
                .with_albedo(0.2, 0.8, 0.2),
        );
        meshes.push(TestMesh::new(i, &format!("npc_{}_mesh", i), 3000, 1500));
    }

    // 环境物体
    for i in 6..=15 {
        entities.push(TestEntity::new(i, &format!("prop_{}", i)));
        materials.push(TestMaterial::new(i, &format!("prop_{}_mat", i)));
        meshes.push(TestMesh::new(i, &format!("prop_{}_mesh", i), 500, 250));
    }

    // 地面
    entities.push(
        TestEntity::new(16, "ground").with_scale(20.0, 1.0, 20.0),
    );
    materials.push(TestMaterial::new(16, "ground_mat").with_albedo(0.3, 0.3, 0.3));
    meshes.push(TestMesh::new(16, "ground_mesh", 4, 2));

    TestScene {
        config: TestSceneConfig::new("complex_scene")
            .with_entity_count(17)
            .with_light_count(5),
        entities,
        materials,
        meshes,
    }
}

/// 预定义场景：性能测试场景
pub fn create_performance_scene() -> TestScene {
    let entity_count = 1000;
    let mut entities = Vec::with_capacity(entity_count);
    let mut meshes = Vec::new();

    // 创建大量实体用于性能测试
    for i in 0..entity_count {
        let x = (i % 50) as f32 * 2.0;
        let y = ((i / 50) % 20) as f32 * 2.0;
        let z = ((i / 1000) as f32) * 2.0;

        entities.push(
            TestEntity::new(i, &format!("perf_entity_{}", i))
                .with_position(x, y, z),
        );

        meshes.push(TestMesh::new(
            i,
            &format!("perf_mesh_{}", i),
            100,
            50,
        ));
    }

    TestScene {
        config: TestSceneConfig::new("performance_scene")
            .with_entity_count(entity_count)
            .with_light_count(10),
        entities,
        materials: vec![TestMaterial::new(0, "default_mat")],
        meshes,
    }
}

/// 预定义场景：空场景
pub fn create_empty_scene() -> TestScene {
    TestScene {
        config: TestSceneConfig::new("empty_scene")
            .with_entity_count(0)
            .with_light_count(0)
            .with_physics(false)
            .with_audio(false),
        entities: Vec::new(),
        materials: Vec::new(),
        meshes: Vec::new(),
    }
}

/// 预定义场景：多平台测试场景
pub fn create_multiplatform_scene() -> TestScene {
    let entities = vec![
        TestEntity::new(0, "player"),
        TestEntity::new(1, "enemy"),
        TestEntity::new(2, "pickup"),
    ];

    let materials = vec![
        TestMaterial::new(0, "player_mat").with_metallic(0.5).with_roughness(0.3),
        TestMaterial::new(1, "enemy_mat").with_albedo(1.0, 0.0, 0.0),
        TestMaterial::new(2, "pickup_mat").with_albedo(1.0, 1.0, 0.0).with_emissive(0.5, 0.5, 0.0),
    ];

    let meshes = vec![
        TestMesh::new(0, "player_mesh", 2000, 1000),
        TestMesh::new(1, "enemy_mesh", 1500, 750),
        TestMesh::new(2, "pickup_mesh", 100, 50),
    ];

    TestScene {
        config: TestSceneConfig::new("multiplatform_scene")
            .with_entity_count(3)
            .with_light_count(2),
        entities,
        materials,
        meshes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_builder() {
        let scene = SceneBuilder::new("test_scene")
            .add_entity(TestEntity::new(0, "entity1"))
            .add_entity(TestEntity::new(1, "entity2"))
            .add_material(TestMaterial::new(0, "mat1"))
            .add_mesh(TestMesh::new(0, "mesh1", 100, 50))
            .build();

        assert_eq!(scene.entity_count(), 2);
        assert_eq!(scene.material_count(), 1);
        assert_eq!(scene.mesh_count(), 1);
    }

    #[test]
    fn test_simple_scene() {
        let scene = create_simple_scene();
        assert_eq!(scene.entity_count(), 2);
        assert_eq!(scene.material_count(), 2);
        assert_eq!(scene.entities[0].name, "player");
    }

    #[test]
    fn test_complex_scene() {
        let scene = create_complex_scene();
        assert_eq!(scene.entity_count(), 17);
        assert_eq!(scene.config.light_count, 5);
        // 检查NPC是否存在
        assert!(scene.entities.iter().any(|e| e.name.starts_with("npc_")));
    }

    #[test]
    fn test_performance_scene() {
        let scene = create_performance_scene();
        assert_eq!(scene.entity_count(), 1000);
        assert_eq!(scene.mesh_count(), 1000);
    }

    #[test]
    fn test_empty_scene() {
        let scene = create_empty_scene();
        assert_eq!(scene.entity_count(), 0);
        assert_eq!(scene.material_count(), 0);
        assert_eq!(scene.mesh_count(), 0);
        assert!(!scene.config.physics_enabled);
        assert!(!scene.config.audio_enabled);
    }

    #[test]
    fn test_multiplatform_scene() {
        let scene = create_multiplatform_scene();
        assert_eq!(scene.entity_count(), 3);
        // 检查拾取物材质有自发光
        let pickup_mat = &scene.materials[2];
        assert_eq!(pickup_mat.emissive, [0.5, 0.5, 0.0]);
    }
}
