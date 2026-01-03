// Test Entities
// 提供测试用的实体和数据结构

use std::collections::HashMap;

/// 测试用实体结构
#[derive(Debug, Clone)]
pub struct TestEntity {
    pub id: u64,
    pub name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub active: bool,
}

impl TestEntity {
    pub fn new(id: u64, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
            active: true,
        }
    }

    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = [x, y, z];
        self
    }

    pub fn with_rotation(mut self, x: f32, y: f32, z: f32, w: f32) -> Self {
        self.rotation = [x, y, z, w];
        self
    }

    pub fn with_scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = [x, y, z];
        self
    }

    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

impl Default for TestEntity {
    fn default() -> Self {
        Self::new(0, "default")
    }
}

/// 测试用材质
#[derive(Debug, Clone)]
pub struct TestMaterial {
    pub id: u64,
    pub name: String,
    pub albedo: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],
}

impl TestMaterial {
    pub fn new(id: u64, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            albedo: [1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
        }
    }

    pub fn with_albedo(mut self, r: f32, g: f32, b: f32) -> Self {
        self.albedo = [r, g, b];
        self
    }

    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic;
        self
    }

    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness;
        self
    }
}

impl Default for TestMaterial {
    fn default() -> Self {
        Self::new(0, "default_material")
    }
}

/// 测试用网格
#[derive(Debug, Clone)]
pub struct TestMesh {
    pub id: u64,
    pub name: String,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub bounds: TestBounds,
}

#[derive(Debug, Clone)]
pub struct TestBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl TestMesh {
    pub fn new(id: u64, name: &str, vertex_count: usize, triangle_count: usize) -> Self {
        Self {
            id,
            name: name.to_string(),
            vertex_count,
            triangle_count,
            bounds: TestBounds {
                min: [-1.0, -1.0, -1.0],
                max: [1.0, 1.0, 1.0],
            },
        }
    }

    pub fn with_bounds(mut self, min: [f32; 3], max: [f32; 3]) -> Self {
        self.bounds = TestBounds { min, max };
        self
    }
}

/// 测试用平台信息
#[derive(Debug, Clone)]
pub struct TestPlatformInfo {
    pub platform_type: String,
    pub os_version: String,
    pub device_model: String,
    pub capabilities: Vec<String>,
    pub memory_mb: u32,
    pub cpu_cores: u32,
    pub gpu_memory_mb: u32,
}

impl TestPlatformInfo {
    pub fn new(platform_type: &str) -> Self {
        Self {
            platform_type: platform_type.to_string(),
            os_version: "1.0.0".to_string(),
            device_model: "Test Device".to_string(),
            capabilities: Vec::new(),
            memory_mb: 1024,
            cpu_cores: 4,
            gpu_memory_mb: 512,
        }
    }

    pub fn with_capability(mut self, capability: &str) -> Self {
        self.capabilities.push(capability.to_string());
        self
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability.to_string())
    }
}

/// 测试用控制器输入状态
#[derive(Debug, Clone, PartialEq)]
pub struct TestControllerState {
    pub buttons: HashMap<String, bool>,
    pub axes: HashMap<String, f32>,
    pub connected: bool,
}

impl TestControllerState {
    pub fn new() -> Self {
        Self {
            buttons: HashMap::new(),
            axes: HashMap::new(),
            connected: true,
        }
    }

    pub fn with_button(mut self, name: &str, pressed: bool) -> Self {
        self.buttons.insert(name.to_string(), pressed);
        self
    }

    pub fn with_axis(mut self, name: &str, value: f32) -> Self {
        self.axes.insert(name.to_string(), value);
        self
    }

    pub fn is_button_pressed(&self, name: &str) -> bool {
        self.buttons.get(name).copied().unwrap_or(false)
    }

    pub fn get_axis(&self, name: &str) -> f32 {
        self.axes.get(name).copied().unwrap_or(0.0)
    }
}

impl Default for TestControllerState {
    fn default() -> Self {
        Self::new()
    }
}

/// 测试用GPU信息
#[derive(Debug, Clone)]
pub struct TestGPUInfo {
    pub name: String,
    pub vendor: String,
    pub memory_mb: u32,
    pub supports_raytracing: bool,
    pub supports_mesh_shaders: bool,
    pub supports_variable_rate_shading: bool,
}

impl TestGPUInfo {
    pub fn new(name: &str, vendor: &str) -> Self {
        Self {
            name: name.to_string(),
            vendor: vendor.to_string(),
            memory_mb: 4096,
            supports_raytracing: false,
            supports_mesh_shaders: false,
            supports_variable_rate_shading: false,
        }
    }

    pub fn with_raytracing(mut self) -> Self {
        self.supports_raytracing = true;
        self
    }

    pub fn with_mesh_shaders(mut self) -> Self {
        self.supports_mesh_shaders = true;
        self
    }
}

/// 测试用场景配置
#[derive(Debug, Clone)]
pub struct TestSceneConfig {
    pub name: String,
    pub entity_count: usize,
    pub light_count: usize,
    pub camera_count: usize,
    pub physics_enabled: bool,
    pub audio_enabled: bool,
}

impl TestSceneConfig {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            entity_count: 10,
            light_count: 3,
            camera_count: 1,
            physics_enabled: true,
            audio_enabled: true,
        }
    }

    pub fn with_entity_count(mut self, count: usize) -> Self {
        self.entity_count = count;
        self
    }

    pub fn with_light_count(mut self, count: usize) -> Self {
        self.light_count = count;
        self
    }

    pub fn with_physics(mut self, enabled: bool) -> Self {
        self.physics_enabled = enabled;
        self
    }
}

impl Default for TestSceneConfig {
    fn default() -> Self {
        Self::new("test_scene")
    }
}

/// 创建常用的测试实体集合
pub fn create_test_entities(count: usize) -> Vec<TestEntity> {
    (0..count)
        .map(|i| TestEntity::new(i as u64, &format!("entity_{}", i)))
        .collect()
}

/// 创建常用的测试材质集合
pub fn create_test_materials(count: usize) -> Vec<TestMaterial> {
    (0..count)
        .map(|i| TestMaterial::new(i as u64, &format!("material_{}", i)))
        .collect()
}

/// 创建常用的测试网格集合
pub fn create_test_meshes(count: usize) -> Vec<TestMesh> {
    (0..count)
        .map(|i| {
            TestMesh::new(
                i as u64,
                &format!("mesh_{}", i),
                100 + i * 10,
                50 + i * 5,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_builder() {
        let entity = TestEntity::new(1, "test")
            .with_position(1.0, 2.0, 3.0)
            .with_active(false);

        assert_eq!(entity.id, 1);
        assert_eq!(entity.name, "test");
        assert_eq!(entity.position, [1.0, 2.0, 3.0]);
        assert_eq!(entity.active, false);
    }

    #[test]
    fn test_material_builder() {
        let material = TestMaterial::new(1, "red")
            .with_albedo(1.0, 0.0, 0.0)
            .with_metallic(0.8);

        assert_eq!(material.albedo, [1.0, 0.0, 0.0]);
        assert_eq!(material.metallic, 0.8);
    }

    #[test]
    fn test_controller_state() {
        let state = TestControllerState::new()
            .with_button("A", true)
            .with_axis("left_x", 0.5);

        assert!(state.is_button_pressed("A"));
        assert_eq!(state.get_axis("left_x"), 0.5);
        assert!(!state.is_button_pressed("B"));
    }

    #[test]
    fn test_platform_info() {
        let platform = TestPlatformInfo::new("PS5")
            .with_capability("haptic_feedback")
            .with_capability("adaptive_triggers");

        assert!(platform.has_capability("haptic_feedback"));
        assert!(platform.has_capability("adaptive_triggers"));
        assert!(!platform.has_capability("raytracing"));
    }

    #[test]
    fn test_create_collections() {
        let entities = create_test_entities(5);
        assert_eq!(entities.len(), 5);
        assert_eq!(entities[0].name, "entity_0");

        let materials = create_test_materials(3);
        assert_eq!(materials.len(), 3);

        let meshes = create_test_meshes(2);
        assert_eq!(meshes.len(), 2);
    }
}
