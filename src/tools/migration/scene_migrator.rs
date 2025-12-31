//! Scene migrator
//!
//! Migrates Unity scenes to the engine's entity-component system.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::error::{Error, Result};
use super::unity_parser::{GameObject, Transform, ComponentData, UnityScene};

/// Migration configuration
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Whether to preserve object IDs
    pub preserve_ids: bool,
    /// Component mapping strategy
    pub component_mapping: ComponentMappingStrategy,
    /// Output directory
    pub output_dir: PathBuf,
}

/// Component mapping strategy
#[derive(Debug, Clone, Copy)]
pub enum ComponentMappingStrategy {
    /// Map to equivalent components
    Direct,
    /// Convert to custom components
    Custom,
    /// Skip unsupported components
    Skip,
    /// Error on unsupported components
    Strict,
}

/// Migrated entity
#[derive(Debug, Clone)]
pub struct MigratedEntity {
    pub name: String,
    pub original_id: u64,
    pub new_id: u64,
    pub components: Vec<MigratedComponent>,
    pub children: Vec<u64>,
    pub parent_id: Option<u64>,
}

/// Migrated component
#[derive(Debug, Clone)]
pub struct MigratedComponent {
    pub component_type: String,
    pub data: HashMap<String, serde_json::Value>,
    pub conversion_notes: Vec<String>,
}

/// Migrated scene
#[derive(Debug, Clone)]
pub struct MigratedScene {
    pub name: String,
    pub entities: HashMap<u64, MigratedEntity>,
    pub root_entities: Vec<u64>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Scene migrator
pub struct SceneMigrator {
    config: MigrationConfig,
    next_entity_id: u64,
}

impl SceneMigrator {
    /// Create a new scene migrator
    pub fn new(config: MigrationConfig) -> Self {
        Self {
            config,
            next_entity_id: 1,
        }
    }

    /// Migrate a Unity scene
    pub fn migrate_scene(&mut self, scene: &UnityScene) -> Result<MigratedScene> {
        println!("Migrating scene: {}", scene.name);

        let mut migrated = MigratedScene {
            name: scene.name.clone(),
            entities: HashMap::new(),
            root_entities: vec![],
            warnings: vec![],
            errors: vec![],
        };

        // Migrate all game objects
        for game_object in &scene.game_objects {
            if let Err(e) = self.migrate_game_object(game_object, &mut migrated) {
                migrated.errors.push(format!(
                    "Failed to migrate {}: {}",
                    game_object.name, e
                ));
            }
        }

        // Find root entities (those without parents)
        for entity in migrated.entities.values() {
            if entity.parent_id.is_none() {
                migrated.root_entities.push(entity.new_id);
            }
        }

        println!("Migrated {} entities", migrated.entities.len());

        Ok(migrated)
    }

    /// Migrate a game object to an entity
    fn migrate_game_object(
        &mut self,
        game_object: &GameObject,
        scene: &mut MigratedScene,
    ) -> Result<u64> {
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;

        let mut components = Vec::new();

        // Add transform component
        let transform = self.migrate_transform(&game_object.transform)?;
        components.push(transform);

        // Add name component
        let mut name_data = HashMap::new();
        name_data.insert("name".to_string(), serde_json::json!(game_object.name));
        components.push(MigratedComponent {
            component_type: "Name".to_string(),
            data: name_data,
            conversion_notes: vec![],
        });

        // Add layer component
        let mut layer_data = HashMap::new();
        layer_data.insert("layer".to_string(), serde_json::json!(game_object.layer));
        components.push(MigratedComponent {
            component_type: "Layer".to_string(),
            data: layer_data,
            conversion_notes: vec![],
        });

        // Migrate components
        for component_ref in &game_object.components {
            if let Ok(component) =
                self.migrate_component(&component_ref.component_type, &component_ref.enabled)
            {
                components.push(component);
            }
        }

        // Migrate children
        let mut children = Vec::new();
        for child in &game_object.children {
            match self.migrate_game_object(child, scene) {
                Ok(child_id) => children.push(child_id),
                Err(e) => {
                    scene.warnings.push(format!(
                        "Failed to migrate child {}: {}",
                        child.name, e
                    ));
                }
            }
        }

        // Create entity
        let entity = MigratedEntity {
            name: game_object.name.clone(),
            original_id: game_object.instance_id,
            new_id: entity_id,
            components,
            children,
            parent_id: None, // Will be set by parent
        };

        scene.entities.insert(entity_id, entity);

        Ok(entity_id)
    }

    /// Migrate transform component
    fn migrate_transform(&self, transform: &Transform) -> Result<MigratedComponent> {
        let mut data = HashMap::new();

        data.insert(
            "position".to_string(),
            serde_json::json!([
                transform.position.0,
                transform.position.1,
                transform.position.2
            ]),
        );

        data.insert(
            "rotation".to_string(),
            serde_json::json!([
                transform.rotation.0,
                transform.rotation.1,
                transform.rotation.2,
                transform.rotation.3
            ]),
        );

        data.insert(
            "scale".to_string(),
            serde_json::json!([
                transform.scale.0,
                transform.scale.1,
                transform.scale.2
            ]),
        );

        Ok(MigratedComponent {
            component_type: "Transform".to_string(),
            data,
            conversion_notes: vec![],
        })
    }

    /// Migrate a component
    fn migrate_component(
        &self,
        component_type: &str,
        enabled: &bool,
    ) -> Result<MigratedComponent> {
        let mut notes = vec![];

        // Map Unity component types to engine component types
        let engine_type = match component_type {
            "MeshRenderer" => "MeshRenderer",
            "MeshFilter" => "MeshFilter",
            "BoxCollider" => "BoxCollider",
            "SphereCollider" => "SphereCollider",
            "Rigidbody" => "Rigidbody",
            "Camera" => "Camera",
            "Light" => "Light",
            "AudioSource" => "AudioSource",
            "Animator" => "Animator",
            _ => {
                notes.push(format!("Unknown component type: {}", component_type));
                match self.config.component_mapping {
                    ComponentMappingStrategy::Strict => {
                        return Err(Error::IoError(format!(
                            "Unsupported component: {}",
                            component_type
                        )))
                    }
                    _ => {
                        notes.push("Component skipped".to_string());
                        return Ok(MigratedComponent {
                            component_type: component_type.to_string(),
                            data: HashMap::new(),
                            conversion_notes: notes,
                        });
                    }
                }
            }
        };

        let mut data = HashMap::new();
        data.insert("enabled".to_string(), serde_json::json!(enabled));

        Ok(MigratedComponent {
            component_type: engine_type.to_string(),
            data,
            conversion_notes: notes,
        })
    }

    /// Save migrated scene to file
    pub fn save_scene(&self, scene: &MigratedScene) -> Result<PathBuf> {
        let output_path = self
            .config
            .output_dir
            .join(format!("{}.ron", scene.name));

        // Create output directory
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // In a real implementation, this would serialize the scene
        println!("Saving migrated scene to: {}", output_path.display());

        Ok(output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_migrator() {
        let config = MigrationConfig {
            preserve_ids: false,
            component_mapping: ComponentMappingStrategy::Direct,
            output_dir: PathBuf::from("/tmp"),
        };

        let migrator = SceneMigrator::new(config);

        // Create a test scene
        let scene = UnityScene {
            name: "TestScene".to_string(),
            path: PathBuf::from("TestScene.unity"),
            game_objects: vec![],
            components: vec![],
        };

        let result = migrator.migrate_scene(&scene);
        assert!(result.is_ok());
    }
}
