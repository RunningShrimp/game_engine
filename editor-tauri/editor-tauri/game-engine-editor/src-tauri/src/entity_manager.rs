// Entity management system for the game engine editor

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

// Entity data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Vector3 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::default(),
            rotation: Quaternion::default(),
            scale: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    #[serde(rename = "type")]
    pub component_type: String,
    pub name: String,
    pub enabled: bool,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub transform: Transform,
    pub components: Vec<Component>,
    pub children: Vec<Entity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub visible: bool,
    pub locked: bool,
}

// Entity manager state
pub struct EntityManager {
    entities: HashMap<String, Entity>,
    next_id: u64,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create_entity(&mut self, name: String) -> Entity {
        let id = self.next_id.to_string();
        self.next_id += 1;

        let entity = Entity {
            id: id.clone(),
            name,
            transform: Transform::default(),
            components: Vec::new(),
            children: Vec::new(),
            parent_id: None,
            visible: true,
            locked: false,
        };

        self.entities.insert(id, entity.clone());
        entity
    }

    pub fn get_entity(&self, id: &str) -> Option<&Entity> {
        self.entities.get(id)
    }

    pub fn update_entity(&mut self, id: &str, entity: Entity) -> Result<(), String> {
        if self.entities.contains_key(id) {
            self.entities.insert(id.to_string(), entity);
            Ok(())
        } else {
            Err(format!("Entity with id {} not found", id))
        }
    }

    pub fn delete_entity(&mut self, id: &str) -> Result<Entity, String> {
        self.entities
            .remove(id)
            .ok_or_else(|| format!("Entity with id {} not found", id))
    }

    pub fn list_entities(&self) -> Vec<Entity> {
        self.entities.values().cloned().collect()
    }

    pub fn rename_entity(&mut self, id: &str, new_name: String) -> Result<(), String> {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.name = new_name;
            Ok(())
        } else {
            Err(format!("Entity with id {} not found", id))
        }
    }

    pub fn duplicate_entity(&mut self, id: &str) -> Result<Entity, String> {
        let original = self.entities.get(id).ok_or_else(|| format!("Entity with id {} not found", id))?.clone();

        let new_id = self.next_id.to_string();
        self.next_id += 1;

        let mut new_entity = original.clone();
        new_entity.id = new_id.clone();
        new_entity.name = format!("{} (Copy)", original.name);

        self.entities.insert(new_id, new_entity.clone());
        Ok(new_entity)
    }

    pub fn set_entity_visibility(&mut self, id: &str, visible: bool) -> Result<(), String> {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.visible = visible;
            Ok(())
        } else {
            Err(format!("Entity with id {} not found", id))
        }
    }

    pub fn set_entity_lock(&mut self, id: &str, locked: bool) -> Result<(), String> {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.locked = locked;
            Ok(())
        } else {
            Err(format!("Entity with id {} not found", id))
        }
    }

    pub fn reparent_entity(&mut self, id: &str, new_parent_id: Option<String>) -> Result<(), String> {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.parent_id = new_parent_id;
            Ok(())
        } else {
            Err(format!("Entity with id {} not found", id))
        }
    }
}

// Global state for the entity manager
pub type EntityManagerState = Mutex<EntityManager>;

// Tauri commands
#[tauri::command]
pub fn create_entity(state: State<EntityManagerState>, name: String) -> Result<Entity, String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    Ok(manager.create_entity(name))
}

#[tauri::command]
pub fn get_entity(state: State<EntityManagerState>, id: String) -> Result<Option<Entity>, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_entity(&id).cloned())
}

#[tauri::command]
pub fn update_entity(state: State<EntityManagerState>, entity: Entity) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    let id = entity.id.clone();
    manager.update_entity(&id, entity)
}

#[tauri::command]
pub fn delete_entity(state: State<EntityManagerState>, id: String) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.delete_entity(&id)?;
    Ok(())
}

#[tauri::command]
pub fn list_entities(state: State<EntityManagerState>) -> Result<Vec<Entity>, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    Ok(manager.list_entities())
}

#[tauri::command]
pub fn rename_entity(
    state: State<EntityManagerState>,
    id: String,
    new_name: String,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.rename_entity(&id, new_name)
}

#[tauri::command]
pub fn duplicate_entity(state: State<EntityManagerState>, id: String) -> Result<Entity, String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.duplicate_entity(&id)
}

#[tauri::command]
pub fn set_entity_visibility(
    state: State<EntityManagerState>,
    id: String,
    visible: bool,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.set_entity_visibility(&id, visible)
}

#[tauri::command]
pub fn set_entity_lock(
    state: State<EntityManagerState>,
    id: String,
    locked: bool,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.set_entity_lock(&id, locked)
}

#[tauri::command]
pub fn reparent_entity(
    state: State<EntityManagerState>,
    id: String,
    new_parent_id: Option<String>,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.reparent_entity(&id, new_parent_id)
}
