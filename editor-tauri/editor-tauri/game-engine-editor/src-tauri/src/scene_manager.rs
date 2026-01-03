// Scene management system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

use crate::entity_manager::{Entity, EntityManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneSettings {
    pub gravity: [f32; 3],
    pub ambient_light: [f32; 3],
    pub background_color: [f32; 4],
    pub fog_enabled: bool,
    pub fog_color: [f32; 3],
    pub fog_density: f32,
}

impl Default for SceneSettings {
    fn default() -> Self {
        Self {
            gravity: [0.0, -9.81, 0.0],
            ambient_light: [0.2, 0.2, 0.2],
            background_color: [0.1, 0.1, 0.15, 1.0],
            fog_enabled: false,
            fog_color: [0.5, 0.5, 0.5],
            fog_density: 0.01,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    pub name: String,
    pub entities: Vec<Entity>,
    pub settings: SceneSettings,
}

pub struct SceneManager {
    scenes: HashMap<String, Scene>,
    current_scene_id: Option<String>,
    next_id: u64,
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
            current_scene_id: None,
            next_id: 1,
        }
    }

    pub fn create_scene(&mut self, name: String) -> Scene {
        let id = self.next_id.to_string();
        self.next_id += 1;

        let scene = Scene {
            id: id.clone(),
            name,
            entities: Vec::new(),
            settings: SceneSettings::default(),
        };

        self.scenes.insert(id, scene.clone());
        scene
    }

    pub fn get_current_scene(&self) -> Option<&Scene> {
        self.current_scene_id
            .as_ref()
            .and_then(|id| self.scenes.get(id))
    }

    pub fn set_current_scene(&mut self, id: &str) -> Result<(), String> {
        if self.scenes.contains_key(id) {
            self.current_scene_id = Some(id.to_string());
            Ok(())
        } else {
            Err(format!("Scene with id {} not found", id))
        }
    }

    pub fn update_scene(&mut self, scene: Scene) -> Result<(), String> {
        if self.scenes.contains_key(&scene.id) {
            self.scenes.insert(scene.id.clone(), scene);
            Ok(())
        } else {
            Err(format!("Scene with id {} not found", scene.id))
        }
    }
}

pub type SceneManagerState = Mutex<SceneManager>;

#[tauri::command]
pub fn create_scene(state: State<SceneManagerState>, name: String) -> Result<Scene, String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    Ok(manager.create_scene(name))
}

#[tauri::command]
pub fn get_current_scene(state: State<SceneManagerState>) -> Result<Option<Scene>, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_current_scene().cloned())
}

#[tauri::command]
pub fn set_current_scene(state: State<SceneManagerState>, id: String) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.set_current_scene(&id)
}

#[tauri::command]
pub fn update_scene(state: State<SceneManagerState>, scene: Scene) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.update_scene(scene)
}
