//! # Engine API Registry
//!
//! Central registry for all game engine API definitions used by LSP.
//! Includes components, systems, resources, and their documentation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Definition of an engine component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDefinition {
    /// Component name
    pub name: String,

    /// Module path (e.g., "game_engine::ecs::Transform")
    pub module: String,

    /// Component description
    pub description: String,

    /// Field definitions
    pub fields: Vec<FieldDefinition>,

    /// Associated methods
    pub methods: Vec<MethodDefinition>,

    /// Documentation markdown
    pub documentation: String,
}

/// Field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    /// Field name
    pub name: String,

    /// Field type
    pub type_name: String,

    /// Field description
    pub description: String,

    /// Whether field is public
    pub is_public: bool,
}

/// Method definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDefinition {
    /// Method name
    pub name: String,

    /// Return type
    pub return_type: String,

    /// Parameter definitions
    pub parameters: Vec<ParameterDefinition>,

    /// Method description
    pub description: String,

    /// Whether method is public
    pub is_public: bool,

    /// Whether method is async
    pub is_async: bool,
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    /// Parameter name
    pub name: String,

    /// Parameter type
    pub type_name: String,

    /// Parameter description
    pub description: String,
}

/// Definition of an engine system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDefinition {
    /// System name
    pub name: String,

    /// Module path
    pub module: String,

    /// System description
    pub description: String,

    /// System type (e.g., "System", "AsyncSystem")
    pub system_type: String,

    /// Query parameters
    pub queries: Vec<QueryDefinition>,

    /// System documentation
    pub documentation: String,
}

/// Query definition for systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryDefinition {
    /// Query name
    pub name: String,

    /// Query components
    pub components: Vec<String>,

    /// Query type (e.g., "Query", "Res", "ResMut")
    pub query_type: String,

    /// Whether query is mutable
    pub is_mutable: bool,
}

/// Definition of an engine resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDefinition {
    /// Resource name
    pub name: String,

    /// Module path
    pub module: String,

    /// Resource description
    pub description: String,

    /// Resource type
    pub resource_type: ResourceType,

    /// Available methods
    pub methods: Vec<MethodDefinition>,

    /// Documentation
    pub documentation: String,
}

/// Resource type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    /// Global resource
    Global,

    /// Local resource
    Local,

    /// Asset resource
    Asset,

    /// Configuration resource
    Config,
}

/// Engine API Registry
///
/// Holds all definitions for engine components, systems, and resources.
/// Thread-safe for concurrent access.
#[derive(Clone)]
pub struct EngineAPIRegistry {
    components: Arc<RwLock<HashMap<String, ComponentDefinition>>>,
    systems: Arc<RwLock<HashMap<String, SystemDefinition>>>,
    resources: Arc<RwLock<HashMap<String, ResourceDefinition>>>,
}

impl Default for EngineAPIRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineAPIRegistry {
    /// Create a new registry with default engine APIs
    pub fn new() -> Self {
        let registry = Self {
            components: Arc::new(RwLock::new(HashMap::new())),
            systems: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
        };

        // Register default engine APIs
        tokio::spawn({
            let registry = registry.clone();
            async move {
                registry.register_default_apis().await;
            }
        });

        registry
    }

    /// Register all default engine APIs
    async fn register_default_apis(&self) {
        // Register common ECS components
        self.register_component(ComponentDefinition {
            name: "Transform".to_string(),
            module: "game_engine::ecs".to_string(),
            description: "Position, rotation, and scale component".to_string(),
            fields: vec![
                FieldDefinition {
                    name: "position".to_string(),
                    type_name: "Vec3".to_string(),
                    description: "World space position".to_string(),
                    is_public: true,
                },
                FieldDefinition {
                    name: "rotation".to_string(),
                    type_name: "Quat".to_string(),
                    description: "Rotation quaternion".to_string(),
                    is_public: true,
                },
                FieldDefinition {
                    name: "scale".to_string(),
                    type_name: "Vec3".to_string(),
                    description: "Scale factors".to_string(),
                    is_public: true,
                },
            ],
            methods: vec![
                MethodDefinition {
                    name: "new".to_string(),
                    return_type: "Self".to_string(),
                    parameters: vec![],
                    description: "Create a new transform with default values".to_string(),
                    is_public: true,
                    is_async: false,
                },
                MethodDefinition {
                    name: "with_position".to_string(),
                    return_type: "Self".to_string(),
                    parameters: vec![ParameterDefinition {
                        name: "pos".to_string(),
                        type_name: "Vec3".to_string(),
                        description: "Position to set".to_string(),
                    }],
                    description: "Set position and return self".to_string(),
                    is_public: true,
                    is_async: false,
                },
            ],
            documentation: "Transform component for entity positioning in 3D space.\n\n# Examples\n\n```rust\nuse game_engine::ecs::Transform;\n\nlet transform = Transform::new()\n    .with_position(Vec3::new(1.0, 2.0, 3.0));\n```".to_string(),
        }).await;

        self.register_component(ComponentDefinition {
            name: "Velocity".to_string(),
            module: "game_engine::ecs".to_string(),
            description: "Linear and angular velocity".to_string(),
            fields: vec![
                FieldDefinition {
                    name: "linear".to_string(),
                    type_name: "Vec3".to_string(),
                    description: "Linear velocity".to_string(),
                    is_public: true,
                },
                FieldDefinition {
                    name: "angular".to_string(),
                    type_name: "Vec3".to_string(),
                    description: "Angular velocity".to_string(),
                    is_public: true,
                },
            ],
            methods: vec![],
            documentation: "Velocity component for physics movement.\n\nAutomatically updated by the physics system.".to_string(),
        }).await;

        self.register_component(ComponentDefinition {
            name: "Health".to_string(),
            module: "game_engine::ecs".to_string(),
            description: "Health component for game entities".to_string(),
            fields: vec![
                FieldDefinition {
                    name: "current".to_string(),
                    type_name: "f32".to_string(),
                    description: "Current health value".to_string(),
                    is_public: true,
                },
                FieldDefinition {
                    name: "max".to_string(),
                    type_name: "f32".to_string(),
                    description: "Maximum health value".to_string(),
                    is_public: true,
                },
            ],
            methods: vec![
                MethodDefinition {
                    name: "is_alive".to_string(),
                    return_type: "bool".to_string(),
                    parameters: vec![],
                    description: "Check if entity is alive".to_string(),
                    is_public: true,
                    is_async: false,
                },
                MethodDefinition {
                    name: "damage".to_string(),
                    return_type: "()".to_string(),
                    parameters: vec![ParameterDefinition {
                        name: "amount".to_string(),
                        type_name: "f32".to_string(),
                        description: "Damage amount".to_string(),
                    }],
                    description: "Apply damage to health".to_string(),
                    is_public: true,
                    is_async: false,
                },
                MethodDefinition {
                    name: "heal".to_string(),
                    return_type: "()".to_string(),
                    parameters: vec![ParameterDefinition {
                        name: "amount".to_string(),
                        type_name: "f32".to_string(),
                        description: "Heal amount".to_string(),
                    }],
                    description: "Heal entity".to_string(),
                    is_public: true,
                    is_async: false,
                },
            ],
            documentation: "Health component for game entities.\n\n# Examples\n\n```rust\nuse game_engine::ecs::Health;\n\nlet mut health = Health::new(100.0);\nhealth.damage(20.0);\nassert!(health.is_alive());\n```".to_string(),
        }).await;

        // Register systems
        self.register_system(SystemDefinition {
            name: "PhysicsSystem".to_string(),
            module: "game_engine::physics".to_string(),
            description: "Updates physics simulation".to_string(),
            system_type: "System".to_string(),
            queries: vec![
                QueryDefinition {
                    name: "entities".to_string(),
                    components: vec!["Transform".to_string(), "Velocity".to_string()],
                    query_type: "Query".to_string(),
                    is_mutable: true,
                },
            ],
            documentation: "Physics system that updates entity positions based on velocities.\n\nRuns every frame to simulate movement and collisions.".to_string(),
        }).await;

        self.register_system(SystemDefinition {
            name: "RenderSystem".to_string(),
            module: "game_engine::render".to_string(),
            description: "Renders all visible entities".to_string(),
            system_type: "System".to_string(),
            queries: vec![
                QueryDefinition {
                    name: "renderables".to_string(),
                    components: vec!["Transform".to_string(), "Mesh".to_string(), "Material".to_string()],
                    query_type: "Query".to_string(),
                    is_mutable: false,
                },
            ],
            documentation: "Render system that draws all visible entities to the screen.\n\nHandles deferred rendering, shadows, and post-processing.".to_string(),
        }).await;

        // Register resources
        self.register_resource(ResourceDefinition {
            name: "Time".to_string(),
            module: "game_engine::core".to_string(),
            description: "Global time resource".to_string(),
            resource_type: ResourceType::Global,
            methods: vec![
                MethodDefinition {
                    name: "delta_time".to_string(),
                    return_type: "f32".to_string(),
                    parameters: vec![],
                    description: "Get time since last frame".to_string(),
                    is_public: true,
                    is_async: false,
                },
                MethodDefinition {
                    name: "elapsed".to_string(),
                    return_type: "Duration".to_string(),
                    parameters: vec![],
                    description: "Get total elapsed time".to_string(),
                    is_public: true,
                    is_async: false,
                },
            ],
            documentation: "Global time resource for frame-independent calculations.\n\n# Examples\n\n```rust\nuse game_engine::core::Time;\n\nfn system(time: Res<Time>) {\n    let delta = time.delta_time();\n    // Use delta for frame-independent movement\n}\n```".to_string(),
        }).await;

        self.register_resource(ResourceDefinition {
            name: "AssetServer".to_string(),
            module: "game_engine::resources".to_string(),
            description: "Asset loading and management".to_string(),
            resource_type: ResourceType::Asset,
            methods: vec![
                MethodDefinition {
                    name: "load".to_string(),
                    return_type: "Handle<T>".to_string(),
                    parameters: vec![ParameterDefinition {
                        name: "path".to_string(),
                        type_name: "&str".to_string(),
                        description: "Path to asset file".to_string(),
                    }],
                    description: "Load an asset from disk".to_string(),
                    is_public: true,
                    is_async: true,
                },
            ],
            documentation: "Asset server for loading and managing game assets.\n\nSupports hot reloading and async loading.".to_string(),
        }).await;
    }

    /// Register a component definition
    pub async fn register_component(&self, component: ComponentDefinition) {
        let mut components = self.components.write().await;
        components.insert(component.name.clone(), component);
    }

    /// Register a system definition
    pub async fn register_system(&self, system: SystemDefinition) {
        let mut systems = self.systems.write().await;
        systems.insert(system.name.clone(), system);
    }

    /// Register a resource definition
    pub async fn register_resource(&self, resource: ResourceDefinition) {
        let mut resources = self.resources.write().await;
        resources.insert(resource.name.clone(), resource);
    }

    /// Get a component by name
    pub async fn get_component(&self, name: &str) -> Option<ComponentDefinition> {
        let components = self.components.read().await;
        components.get(name).cloned()
    }

    /// Get a system by name
    pub async fn get_system(&self, name: &str) -> Option<SystemDefinition> {
        let systems = self.systems.read().await;
        systems.get(name).cloned()
    }

    /// Get a resource by name
    pub async fn get_resource(&self, name: &str) -> Option<ResourceDefinition> {
        let resources = self.resources.read().await;
        resources.get(name).cloned()
    }

    /// List all component names
    pub async fn list_components(&self) -> Vec<String> {
        let components = self.components.read().await;
        components.keys().cloned().collect()
    }

    /// List all system names
    pub async fn list_systems(&self) -> Vec<String> {
        let systems = self.systems.read().await;
        systems.keys().cloned().collect()
    }

    /// List all resource names
    pub async fn list_resources(&self) -> Vec<String> {
        let resources = self.resources.read().await;
        resources.keys().cloned().collect()
    }

    /// Search components by pattern
    pub async fn search_components(&self, pattern: &str) -> Vec<ComponentDefinition> {
        let components = self.components.read().await;
        components
            .values()
            .filter(|c| {
                c.name.to_lowercase().contains(&pattern.to_lowercase())
                    || c.module.to_lowercase().contains(&pattern.to_lowercase())
            })
            .cloned()
            .collect()
    }

    /// Search systems by pattern
    pub async fn search_systems(&self, pattern: &str) -> Vec<SystemDefinition> {
        let systems = self.systems.read().await;
        systems
            .values()
            .filter(|s| {
                s.name.to_lowercase().contains(&pattern.to_lowercase())
                    || s.module.to_lowercase().contains(&pattern.to_lowercase())
            })
            .cloned()
            .collect()
    }

    /// Search resources by pattern
    pub async fn search_resources(&self, pattern: &str) -> Vec<ResourceDefinition> {
        let resources = self.resources.read().await;
        resources
            .values()
            .filter(|r| {
                r.name.to_lowercase().contains(&pattern.to_lowercase())
                    || r.module.to_lowercase().contains(&pattern.to_lowercase())
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = EngineAPIRegistry::new();

        // Wait for default registration
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let components = registry.list_components().await;
        assert!(!components.is_empty());

        let systems = registry.list_systems().await;
        assert!(!systems.is_empty());

        let resources = registry.list_resources().await;
        assert!(!resources.is_empty());
    }

    #[tokio::test]
    async fn test_component_retrieval() {
        let registry = EngineAPIRegistry::new();

        // Wait for default registration
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let transform = registry.get_component("Transform").await;
        assert!(transform.is_some());

        let transform = transform.unwrap();
        assert_eq!(transform.name, "Transform");
        assert_eq!(transform.module, "game_engine::ecs");
        assert!(!transform.fields.is_empty());
    }

    #[tokio::test]
    async fn test_custom_registration() {
        let registry = EngineAPIRegistry::new();

        let component = ComponentDefinition {
            name: "CustomComponent".to_string(),
            module: "test::module".to_string(),
            description: "Test component".to_string(),
            fields: vec![],
            methods: vec![],
            documentation: String::new(),
        };

        registry.register_component(component).await;

        let retrieved = registry.get_component("CustomComponent").await;
        assert!(retrieved.is_some());
    }
}
