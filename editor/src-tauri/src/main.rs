#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod events;
mod state;

use commands::*;
use state::AppState;
use std::sync::Mutex;

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();

    // 构建Tauri应用
    tauri::Builder::default()
        .manage(AppState {
            engine_handle: Mutex::new(None),
            is_playing: Mutex::new(false),
            selected_entity: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            create_engine,
            get_entities,
            update_transform,
            play_scene,
            stop_scene,
            pause_scene,
            raycast,
            get_entity_components,
            update_component,
            create_entity,
            delete_entity,
            save_scene,
            load_scene,
            get_assets,
            import_asset,
            get_console_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
