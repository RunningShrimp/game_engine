// Asset Manager for Tauri Game Engine Editor
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub path: String,
    pub name: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub size: u64,
    pub created: String,
    pub modified: String,
    pub thumbnail: Option<String>,
    pub metadata: Option<AssetMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    // Texture metadata
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub mipmaps: Option<u32>,

    // Mesh metadata
    pub vertices: Option<u32>,
    pub triangles: Option<u32>,

    // Audio metadata
    pub duration: Option<f64>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,

    // Scene metadata
    pub entities: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPreview {
    #[serde(rename = "type")]
    pub preview_type: String,
    pub content: String,
    pub metadata: Option<AssetMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub source: String,
    pub destination: String,
    pub success: bool,
    pub error: Option<String>,
}

pub struct AssetManager {
    base_path: PathBuf,
    thumbnail_cache: HashMap<String, String>,
}

impl AssetManager {
    pub fn new(base_path: String) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
            thumbnail_cache: HashMap::new(),
        }
    }

    fn get_asset_type(extension: &str) -> String {
        match extension.to_lowercase().as_str() {
            // Mesh formats
            "fbx" | "obj" | "gltf" | "glb" | "ply" | "stl" => "mesh".to_string(),

            // Texture formats
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tga" | "webp" | "hdr" | "exr" => "texture".to_string(),

            // Audio formats
            "mp3" | "wav" | "ogg" | "flac" | "aac" | "m4a" => "audio".to_string(),

            // Scene formats
            "scene" | "json" => "scene".to_string(),

            // Material formats
            "mat" | "material" => "material".to_string(),

            // Script formats
            "js" | "ts" | "lua" | "py" | "cs" => "script".to_string(),

            // Shader formats
            "wgsl" | "glsl" | "hlsl" | "vert" | "frag" => "shader".to_string(),

            _ => "unknown".to_string(),
        }
    }

    fn format_size(size: u64) -> u64 {
        size
    }

    fn format_time(time: std::time::SystemTime) -> String {
        use std::time::UNIX_EPOCH;
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
        let datetime: DateTime<Utc> = DateTime::from_timestamp(duration.as_secs() as i64, 0).unwrap();
        datetime.to_rfc3339()
    }

    pub fn list_assets(&self, path: &str) -> Result<Vec<AssetInfo>, String> {
        let full_path = self.base_path.join(path.strip_prefix('/').unwrap_or(path));

        if !full_path.exists() {
            return Err(format!("Path does not exist: {:?}", full_path));
        }

        let mut assets = Vec::new();

        let entries = fs::read_dir(&full_path)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let file_path = entry.path();
            let metadata = entry.metadata().map_err(|e| format!("Failed to get metadata: {}", e))?;

            // Skip hidden files
            if let Some(name) = file_path.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    continue;
                }
            }

            if metadata.is_file() {
                let name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let extension = file_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");

                let asset_type = Self::get_asset_type(extension);
                let relative_path = match file_path.strip_prefix(&self.base_path) {
                    Ok(path) => match path.to_str() {
                        Some(s) => s.to_string(),
                        None => continue,
                    },
                    Err(_) => continue,
                };

                let created = metadata.created().ok().map(Self::format_time);
                let modified = metadata.modified().ok().map(Self::format_time);

                let asset_info = AssetInfo {
                    path: format!("/{}", relative_path.replace('\\', "/")),
                    name,
                    asset_type,
                    size: metadata.len(),
                    created: created.unwrap_or_default(),
                    modified: modified.unwrap_or_default(),
                    thumbnail: None,  // 缩略图生成计划中（使用默认图标）
                    metadata: None,  // 元数据提取计划中（使用基本文件信息）
                };

                assets.push(asset_info);
            }
        }

        // Sort by name
        assets.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(assets)
    }

    pub fn get_asset_preview(&self, path: &str) -> Result<AssetPreview, String> {
        let full_path = self.base_path.join(path.strip_prefix('/').unwrap_or(path));

        if !full_path.exists() {
            return Err(format!("Asset does not exist: {:?}", full_path));
        }

        let extension = full_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let asset_type = Self::get_asset_type(&extension);
        let preview_type = match asset_type.as_str() {
            "texture" => "image",
            "mesh" => "model",
            "audio" => "audio",
            "script" | "shader" => "text",
            _ => "binary",
        };

        // For images, read and convert to base64
        let content = if preview_type == "image" {
            let image_data = fs::read(&full_path)
                .map_err(|e| format!("Failed to read image: {}", e))?;

            let mime_type = match extension.as_str() {
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "webp" => "image/webp",
                _ => "image/png",
            };

            format!("data:{};base64,{}", mime_type, base64::engine::general_purpose::STANDARD_NO_PAD.encode(&image_data))
        } else if preview_type == "text" {
            let text_content = fs::read_to_string(&full_path)
                .map_err(|e| format!("Failed to read text: {}", e))?;
            text_content
        } else {
            String::new()
        };

        Ok(AssetPreview {
            preview_type: preview_type.to_string(),
            content,
            metadata: None,  // 元数据提取计划中（使用基本文件信息）
        })
    }

    pub fn import_assets(&self, files: Vec<String>, dest: &str) -> Result<Vec<ImportResult>, String> {
        let dest_path = self.base_path.join(dest.strip_prefix('/').unwrap_or(dest));

        if !dest_path.exists() {
            fs::create_dir_all(&dest_path)
                .map_err(|e| format!("Failed to create destination directory: {}", e))?;
        }

        let mut results = Vec::new();

        for source in files {
            let source_path = PathBuf::from(&source);
            if !source_path.exists() {
                results.push(ImportResult {
                    source,
                    destination: String::new(),
                    success: false,
                    error: Some("Source file does not exist".to_string()),
                });
                continue;
            }

            let file_name = source_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let dest_file = dest_path.join(file_name);

            // Copy file
            match fs::copy(&source_path, &dest_file) {
                Ok(_) => {
                    let relative_dest = match dest_file.strip_prefix(&self.base_path) {
                        Ok(path) => match path.to_str() {
                            Some(s) => s.to_string(),
                            None => {
                                results.push(ImportResult {
                                    source,
                                    destination: String::new(),
                                    success: false,
                                    error: Some("Invalid path encoding".to_string()),
                                });
                                continue;
                            }
                        },
                        Err(_) => {
                            results.push(ImportResult {
                                source,
                                destination: String::new(),
                                success: false,
                                error: Some("Failed to get relative path".to_string()),
                            });
                            continue;
                        }
                    };

                    results.push(ImportResult {
                        source,
                        destination: format!("/{}", relative_dest.replace('\\', "/")),
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    results.push(ImportResult {
                        source,
                        destination: String::new(),
                        success: false,
                        error: Some(format!("Failed to copy: {}", e)),
                    });
                }
            }
        }

        Ok(results)
    }

    pub fn delete_asset(&self, path: &str) -> Result<(), String> {
        let full_path = self.base_path.join(path.strip_prefix('/').unwrap_or(path));

        if !full_path.exists() {
            return Err(format!("Asset does not exist: {:?}", full_path));
        }

        // Move to trash instead of permanent delete (platform-specific)
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("osascript")
                .args(&["-e", &format!("tell application \"Finder\" to move POSIX file \"{}\" to trash", full_path.display())])
                .output()
                .map_err(|e| format!("Failed to move to trash: {}", e))?;
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("powershell")
                .args(&["-Command", &format!("Move-Item -Path '{}' -Path '{:?}\\$Recycle.Bin' -Force", full_path.display(), std::env::var("SYSTEMDRIVE").unwrap_or_else(|_| "C:".to_string()))])
                .output()
                .map_err(|e| format!("Failed to move to trash: {}", e))?;
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            Command::new("gio")
                .args(&["trash", &format!("{}", full_path.display())])
                .output()
                .map_err(|e| format!("Failed to move to trash: {}", e))?;
        }

        Ok(())
    }

    pub fn rename_asset(&self, path: &str, new_name: &str) -> Result<(), String> {
        let full_path = self.base_path.join(path.strip_prefix('/').unwrap_or(path));

        if !full_path.exists() {
            return Err(format!("Asset does not exist: {:?}", full_path));
        }

        let new_path = full_path
            .parent()
            .map(|p| p.join(new_name))
            .ok_or_else(|| "Invalid path".to_string())?;

        fs::rename(&full_path, &new_path)
            .map_err(|e| format!("Failed to rename: {}", e))?;

        Ok(())
    }

    pub fn get_asset_dependencies(&self, path: &str) -> Result<Vec<String>, String> {
        let full_path = self.base_path.join(path.strip_prefix('/').unwrap_or(path));

        if !full_path.exists() {
            return Err(format!("Asset does not exist: {:?}", full_path));
        }

        // TODO: Implement dependency analysis
        // This would involve parsing the asset file and finding references to other assets
        let dependencies = Vec::new();

        Ok(dependencies)
    }

    pub fn create_folder(&self, path: &str, name: &str) -> Result<String, String> {
        let full_path = self.base_path.join(path.strip_prefix('/').unwrap_or(path));
        let new_folder = full_path.join(name);

        if new_folder.exists() {
            return Err("Folder already exists".to_string());
        }

        fs::create_dir_all(&new_folder)
            .map_err(|e| format!("Failed to create folder: {}", e))?;

        let relative_path = match new_folder.strip_prefix(&self.base_path) {
            Ok(path) => match path.to_str() {
                Some(s) => s.to_string(),
                None => return Err("Invalid path encoding".to_string()),
            },
            Err(_) => return Err("Failed to get relative path".to_string()),
        };

        Ok(format!("/{}", relative_path.replace('\\', "/")))
    }

    pub fn get_folder_tree(&self, path: &str) -> Result<Vec<FolderNode>, String> {
        let full_path = self.base_path.join(path.strip_prefix('/').unwrap_or(path));

        if !full_path.exists() {
            return Err(format!("Path does not exist: {:?}", full_path));
        }

        self.build_folder_tree(&full_path, "")
    }

    fn build_folder_tree(&self, path: &Path, relative_path: &str) -> Result<Vec<FolderNode>, String> {
        let mut nodes = Vec::new();

        let entries = fs::read_dir(path)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let file_path = entry.path();
            let metadata = entry.metadata().map_err(|e| format!("Failed to get metadata: {}", e))?;

            // Skip hidden files
            if let Some(name) = file_path.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    continue;
                }
            }

            if metadata.is_dir() {
                let name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let path_str = match file_path.strip_prefix(&self.base_path) {
                    Ok(path) => match path.to_str() {
                        Some(s) => s.to_string(),
                        None => continue,
                    },
                    Err(_) => continue,
                };

                let children = self.build_folder_tree(&file_path, &path_str)?;

                let asset_count = self.count_assets(&file_path);

                nodes.push(FolderNode {
                    path: format!("/{}", path_str.replace('\\', "/")),
                    name,
                    children,
                    expanded: false,
                    asset_count,
                });
            }
        }

        nodes.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(nodes)
    }

    fn count_assets(&self, path: &Path) -> u32 {
        let mut count = 0;

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        count += 1;
                    }
                }
            }
        }

        count
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderNode {
    pub path: String,
    pub name: String,
    pub children: Vec<FolderNode>,
    pub expanded: bool,
    pub asset_count: u32,
}

// Tauri commands
#[tauri::command]
pub async fn list_assets(path: String) -> Result<Vec<AssetInfo>, String> {
    let manager = AssetManager::new("./Assets".to_string());
    manager.list_assets(&path)
}

#[tauri::command]
pub async fn get_asset_preview(path: String) -> Result<AssetPreview, String> {
    let manager = AssetManager::new("./Assets".to_string());
    manager.get_asset_preview(&path)
}

#[tauri::command]
pub async fn import_assets(files: Vec<String>, dest: String) -> Result<Vec<ImportResult>, String> {
    let manager = AssetManager::new("./Assets".to_string());
    manager.import_assets(files, &dest)
}

#[tauri::command]
pub async fn delete_asset(path: String) -> Result<(), String> {
    let manager = AssetManager::new("./Assets".to_string());
    manager.delete_asset(&path)
}

#[tauri::command]
pub async fn rename_asset(path: String, new_name: String) -> Result<(), String> {
    let manager = AssetManager::new("./Assets".to_string());
    manager.rename_asset(&path, &new_name)
}

#[tauri::command]
pub async fn get_asset_dependencies(path: String) -> Result<Vec<String>, String> {
    let manager = AssetManager::new("./Assets".to_string());
    manager.get_asset_dependencies(&path)
}

#[tauri::command]
pub async fn create_folder(path: String, name: String) -> Result<String, String> {
    let manager = AssetManager::new("./Assets".to_string());
    manager.create_folder(&path, &name)
}

#[tauri::command]
pub async fn get_folder_tree(path: String) -> Result<Vec<FolderNode>, String> {
    let manager = AssetManager::new("./Assets".to_string());
    manager.get_folder_tree(&path)
}
