use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 实体数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub id: u64,
    pub name: String,
    pub children: Vec<EntityData>,
}

/// 变换组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformData {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

/// 组件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ComponentData {
    Transform(TransformData),
    Mesh { mesh_path: String },
    Material { material_path: String },
    Light {
        light_type: String,
        color: [f32; 3],
        intensity: f32,
    },
    Camera {
        fov: f32,
        near: f32,
        far: f32,
    },
    RigidBody {
        body_type: String,
        mass: f32,
    },
    Collider {
        collider_type: String,
        size: [f32; 3],
    },
}

/// 资源数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetData {
    pub path: String,
    pub name: String,
    pub asset_type: String,
    pub thumbnail: Option<String>,
}

/// 控制台日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleLog {
    pub level: String,
    pub message: String,
    pub timestamp: u64,
}

/// 创建引擎实例
#[tauri::command]
pub async fn create_engine(state: tauri::State<'_, AppState>) -> Result<String, String> {
    // 生成引擎实例UUID
    let engine_uuid = uuid::Uuid::new_v4();

    // 在实际实现中，这里会创建游戏引擎实例
    // 由于引擎运行在独立进程中，这里只存储UUID作为标识
    let mut handle = state.engine_handle.lock().unwrap();
    *handle = Some(engine_uuid);

    Ok(format!("Engine created: {}", engine_uuid))
}

/// 获取场景中的所有实体
#[tauri::command]
pub async fn get_entities(state: tauri::State<'_, AppState>) -> Result<Vec<EntityData>, String> {
    let _handle = state.engine_handle.lock().unwrap();

    // 在实际实现中，这里会从引擎获取实体列表
    // 现在返回示例数据
    let entities = vec![
        EntityData {
            id: 1,
            name: "Main Camera".to_string(),
            children: vec![],
        },
        EntityData {
            id: 2,
            name: "Directional Light".to_string(),
            children: vec![],
        },
        EntityData {
            id: 3,
            name: "Cube".to_string(),
            children: vec![
                EntityData {
                    id: 4,
                    name: "Child Cube".to_string(),
                    children: vec![],
                },
            ],
        },
    ];

    Ok(entities)
}

/// 更新实体变换
#[tauri::command]
pub async fn update_transform(
    entity_id: u64,
    position: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let _handle = state.engine_handle.lock().unwrap();

    // 在实际实现中，这里会调用引擎API更新变换
    log::info!(
        "Update transform for entity {}: pos={:?} rot={:?} scale={:?}",
        entity_id,
        position,
        rotation,
        scale
    );

    Ok(())
}

/// 播放场景
#[tauri::command]
pub async fn play_scene(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut is_playing = state.is_playing.lock().unwrap();
    *is_playing = true;

    log::info!("Scene playback started");
    Ok(())
}

/// 停止场景
#[tauri::command]
pub async fn stop_scene(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut is_playing = state.is_playing.lock().unwrap();
    *is_playing = false;

    log::info!("Scene playback stopped");
    Ok(())
}

/// 暂停场景
#[tauri::command]
pub async fn pause_scene(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let is_playing = state.is_playing.lock().unwrap();
    if !*is_playing {
        return Err("Scene is not playing".to_string());
    }

    log::info!("Scene playback paused");
    Ok(())
}

/// 射线拾取
#[tauri::command]
pub async fn raycast(x: f32, y: f32, state: tauri::State<'_, AppState>) -> Result<Option<u64>, String> {
    let _handle = state.engine_handle.lock().unwrap();

    // 在实际实现中，这里会执行射线拾取
    log::info!("Raycast at ({}, {})", x, y);

    // 返回示例实体ID
    Ok(Some(1))
}

/// 获取实体组件
#[tauri::command]
pub async fn get_entity_components(
    entity_id: u64,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ComponentData>, String> {
    let _handle = state.engine_handle.lock().unwrap();

    // 返回示例组件数据
    let components = vec![
        ComponentData::Transform(TransformData {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }),
        ComponentData::Mesh {
            mesh_path: "/models/cube.glb".to_string(),
        },
    ];

    Ok(components)
}

/// 更新组件
#[tauri::command]
pub async fn update_component(
    entity_id: u64,
    component: ComponentData,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let _handle = state.engine_handle.lock().unwrap();

    log::info!("Update component for entity {}: {:?}", entity_id, component);
    Ok(())
}

/// 创建实体
#[tauri::command]
pub async fn create_entity(
    name: String,
    parent_id: Option<u64>,
    state: tauri::State<'_, AppState>,
) -> Result<u64, String> {
    let _handle = state.engine_handle.lock().unwrap();

    // 生成新实体ID
    let entity_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    log::info!("Created entity '{}' with ID {}", name, entity_id);
    Ok(entity_id)
}

/// 删除实体
#[tauri::command]
pub async fn delete_entity(
    entity_id: u64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let _handle = state.engine_handle.lock().unwrap();

    log::info!("Deleted entity {}", entity_id);
    Ok(())
}

/// 保存场景
#[tauri::command]
pub async fn save_scene(
    scene_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let _handle = state.engine_handle.lock().unwrap();

    log::info!("Saved scene to {}", scene_path);
    Ok(())
}

/// 加载场景
#[tauri::command]
pub async fn load_scene(
    scene_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let _handle = state.engine_handle.lock().unwrap();

    log::info!("Loaded scene from {}", scene_path);
    Ok(())
}

/// 获取资源列表
#[tauri::command]
pub async fn get_assets(
    asset_type: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AssetData>, String> {
    let _handle = state.engine_handle.lock().unwrap();

    // 返回示例资源数据
    let assets = vec![
        AssetData {
            path: "/models/cube.glb".to_string(),
            name: "cube.glb".to_string(),
            asset_type: "model".to_string(),
            thumbnail: None,
        },
        AssetData {
            path: "/materials/default.mat".to_string(),
            name: "default.mat".to_string(),
            asset_type: "material".to_string(),
            thumbnail: None,
        },
        AssetData {
            path: "/textures/brick.png".to_string(),
            name: "brick.png".to_string(),
            asset_type: "texture".to_string(),
            thumbnail: None,
        },
    ];

    Ok(assets)
}

/// 导入资源
#[tauri::command]
pub async fn import_asset(
    source_path: String,
    asset_type: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let _handle = state.engine_handle.lock().unwrap();

    log::info!("Imported asset from {}", source_path);
    Ok(source_path)
}

/// 获取控制台日志
#[tauri::command]
pub async fn get_console_logs(
    limit: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ConsoleLog>, String> {
    // 返回示例日志
    let logs = vec![
        ConsoleLog {
            level: "info".to_string(),
            message: "Engine initialized".to_string(),
            timestamp: 1234567890,
        },
        ConsoleLog {
            level: "warning".to_string(),
            message: "Texture not found: /textures/missing.png".to_string(),
            timestamp: 1234567891,
        },
    ];

    Ok(logs)
}
