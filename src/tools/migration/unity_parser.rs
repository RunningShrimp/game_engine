//! Unity project parser
//!
//! Parses Unity project files (.unity, .prefab, .meta) and extracts data for migration.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::error::{Error, Result};

/// Unity project structure
#[derive(Debug, Clone)]
pub struct UnityProject {
    pub project_path: PathBuf,
    pub assets_path: PathBuf,
    pub version: String,
    pub scenes: Vec<UnityScene>,
    pub prefabs: Vec<UnityPrefab>,
    pub metadata: HashMap<String, UnityMetadata>,
}

/// Unity scene
#[derive(Debug, Clone)]
pub struct UnityScene {
    pub name: String,
    pub path: PathBuf,
    pub game_objects: Vec<GameObject>,
    pub components: Vec<ComponentData>,
}

/// Unity prefab
#[derive(Debug, Clone)]
pub struct UnityPrefab {
    pub name: String,
    pub path: PathBuf,
    pub game_object: GameObject,
    pub components: Vec<ComponentData>,
}

/// Unity metadata (.meta files)
#[derive(Debug, Clone)]
pub struct UnityMetadata {
    pub path: PathBuf,
    pub guid: String,
    pub file_format_version: Option<String>,
    pub import_settings: HashMap<String, String>,
}

/// Unity GameObject
#[derive(Debug, Clone)]
pub struct GameObject {
    pub name: String,
    pub instance_id: u64,
    pub transform: Transform,
    pub components: Vec<ComponentRef>,
    pub children: Vec<GameObject>,
    pub layer: i32,
    pub tag: String,
    pub is_active: bool,
}

/// Transform component
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32), // Quaternion
    pub scale: (f32, f32, f32),
    pub parent: Option<u64>,
}

/// Component reference
#[derive(Debug, Clone)]
pub struct ComponentRef {
    pub component_type: String,
    pub instance_id: u64,
    pub enabled: bool,
}

/// Component data
#[derive(Debug, Clone)]
pub enum ComponentData {
    MeshRenderer {
        materials: Vec<String>,
        lightmap_index: i32,
    },
    MeshFilter {
        mesh: String,
    },
    BoxCollider {
        center: (f32, f32, f32),
        size: (f32, f32, f32),
    },
    SphereCollider {
        center: (f32, f32, f32),
        radius: f32,
    },
    Rigidbody {
        mass: f32,
        drag: f32,
        angular_drag: f32,
        use_gravity: bool,
        is_kinematic: bool,
    },
    Camera {
        clear_flags: String,
        background_color: (f32, f32, f32, f32),
        field_of_view: f32,
        near_clip_plane: f32,
        far_clip_plane: f32,
    },
    Light {
        type_: String,
        color: (f32, f32, f32),
        intensity: f32,
        range: f32,
    },
    AudioSource {
        clip: Option<String>,
        volume: f32,
        loop_: bool,
        play_on_awake: bool,
    },
    Animator {
        controller: Option<String>,
        avatar: Option<String>,
    },
    Script {
        script_name: String,
        properties: HashMap<String, ScriptProperty>,
    },
    Custom {
        type_name: String,
        properties: HashMap<String, String>,
    },
}

/// Script property
#[derive(Debug, Clone)]
pub enum ScriptProperty {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Vector3((f32, f32, f32)),
    Color((f32, f32, f32, f32)),
    Asset(String),
    List(Vec<ScriptProperty>),
}

/// Unity project parser
pub struct UnityParser;

impl UnityParser {
    /// Parse a Unity project directory
    pub fn parse_project(project_path: &Path) -> Result<UnityProject> {
        if !project_path.exists() {
            return Err(Error::IoError(format!(
                "Project path does not exist: {}",
                project_path.display()
            )));
        }

        let assets_path = project_path.join("Assets");
        if !assets_path.exists() {
            return Err(Error::IoError(
                "Assets folder not found. Is this a Unity project?".to_string(),
            ));
        }

        // Get Unity version from ProjectSettings
        let version = Self::read_unity_version(project_path)?;

        // Find all .unity files (scenes)
        let scenes = Self::find_scenes(&assets_path)?;

        // Find all .prefab files
        let prefabs = Self::find_prefabs(&assets_path)?;

        // Parse metadata files
        let metadata = Self::parse_metadata(&assets_path)?;

        Ok(UnityProject {
            project_path: project_path.to_path_buf(),
            assets_path,
            version,
            scenes,
            prefabs,
            metadata,
        })
    }

    /// Read Unity version from ProjectSettings
    fn read_unity_version(project_path: &Path) -> Result<String> {
        let settings_path = project_path.join("ProjectSettings/ProjectVersion.txt");

        if !settings_path.exists() {
            return Ok("Unknown".to_string());
        }

        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| Error::IoError(format!("Failed to read ProjectVersion.txt: {}", e)))?;

        // Parse version line: "m_EditorVersion: 2021.3.0f1"
        for line in content.lines() {
            if line.contains("m_EditorVersion:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    return Ok(parts[1].trim().to_string());
                }
            }
        }

        Ok("Unknown".to_string())
    }

    /// Find all .unity scene files
    fn find_scenes(assets_path: &Path) -> Result<Vec<UnityScene>> {
        let mut scenes = Vec::new();

        let entries = std::fs::read_dir(assets_path)
            .map_err(|e| Error::IoError(format!("Failed to read Assets directory: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();
            Self::scan_directory_for_scenes(&path, &mut scenes)?;
        }

        Ok(scenes)
    }

    /// Recursively scan for scene files
    fn scan_directory_for_scenes(dir: &Path, scenes: &mut Vec<UnityScene>) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| Error::IoError(format!("Failed to read directory: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                Self::scan_directory_for_scenes(&path, scenes)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("unity") {
                let scene = Self::parse_scene(&path)?;
                scenes.push(scene);
            }
        }

        Ok(())
    }

    /// Parse a .unity scene file
    fn parse_scene(path: &Path) -> Result<UnityScene> {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // In a real implementation, this would parse the YAML scene file
        // For now, return a placeholder
        Ok(UnityScene {
            name,
            path: path.to_path_buf(),
            game_objects: vec![],
            components: vec![],
        })
    }

    /// Find all .prefab files
    fn find_prefabs(assets_path: &Path) -> Result<Vec<UnityPrefab>> {
        let mut prefabs = Vec::new();

        let entries = std::fs::read_dir(assets_path)
            .map_err(|e| Error::IoError(format!("Failed to read Assets directory: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();
            Self::scan_directory_for_prefabs(&path, &mut prefabs)?;
        }

        Ok(prefabs)
    }

    /// Recursively scan for prefab files
    fn scan_directory_for_prefabs(dir: &Path, prefabs: &mut Vec<UnityPrefab>) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| Error::IoError(format!("Failed to read directory: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                Self::scan_directory_for_prefabs(&path, prefabs)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("prefab") {
                let prefab = Self::parse_prefab(&path)?;
                prefabs.push(prefab);
            }
        }

        Ok(())
    }

    /// Parse a .prefab file
    fn parse_prefab(path: &Path) -> Result<UnityPrefab> {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // In a real implementation, this would parse the YAML prefab file
        Ok(UnityPrefab {
            name,
            path: path.to_path_buf(),
            game_object: GameObject {
                name: name.clone(),
                instance_id: 0,
                transform: Transform {
                    position: (0.0, 0.0, 0.0),
                    rotation: (0.0, 0.0, 0.0, 1.0),
                    scale: (1.0, 1.0, 1.0),
                    parent: None,
                },
                components: vec![],
                children: vec![],
                layer: 0,
                tag: "Untagged".to_string(),
                is_active: true,
            },
            components: vec![],
        })
    }

    /// Parse all .meta files
    fn parse_metadata(assets_path: &Path) -> Result<HashMap<String, UnityMetadata>> {
        let mut metadata = HashMap::new();

        let entries = std::fs::read_dir(assets_path)
            .map_err(|e| Error::IoError(format!("Failed to read Assets directory: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();
            Self::scan_directory_for_metadata(&path, &mut metadata)?;
        }

        Ok(metadata)
    }

    /// Recursively scan for .meta files
    fn scan_directory_for_metadata(
        dir: &Path,
        metadata: &mut HashMap<String, UnityMetadata>,
    ) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| Error::IoError(format!("Failed to read directory: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                Self::scan_directory_for_metadata(&path, metadata)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("meta") {
                if let Ok(meta) = Self::parse_metadata_file(&path) {
                    let asset_path = path.with_extension("");
                    if let Some(path_str) = asset_path.to_str() {
                        metadata.insert(path_str.to_string(), meta);
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse a .meta file
    fn parse_metadata_file(path: &Path) -> Result<UnityMetadata> {
        // In a real implementation, this would parse the YAML meta file
        Ok(UnityMetadata {
            path: path.to_path_buf(),
            guid: "00000000000000000000000000000000".to_string(),
            file_format_version: None,
            import_settings: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unity_parser() {
        // Test with a non-existent path
        let result = UnityParser::parse_project(Path::new("/nonexistent"));
        assert!(result.is_err());
    }
}
