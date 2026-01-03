mod camera;
mod entity_manager;
mod geometry;
mod scene_manager;
mod webgpu_renderer;
mod asset_manager;
mod performance_monitor;
mod performance_commands;
mod importers;
mod animation_system;
mod behavior_tree;
mod batch_operations;
mod shortcuts;
mod asset_store;
mod tutorial;

// Plugin system module
pub mod plugin;

// New command modules
mod animation_commands;
mod asset_commands;

use std::sync::Mutex;
use entity_manager::{EntityManager, EntityManagerState};
use scene_manager::{SceneManagerState};
use webgpu_renderer::{FrameStats};
use performance_monitor::PerformanceMonitor;
use performance_commands::PerformanceMonitorState;
use importers::{ModelData, import_model};
use animation_system::AnimationSystem;
use behavior_tree::BehaviorTreeManager;
use batch_operations::{BatchOperationsManager};

// Tauri commands for WebGPU rendering

/// Initialize WebGPU renderer
#[tauri::command]
async fn initialize_renderer() -> Result<String, String> {
    // For now, WebGPU will be initialized from the frontend
    // This command is a placeholder for future backend initialization
    Ok("WebGPU renderer ready".to_string())
}

/// Render a frame (this will be called from the frontend render loop)
#[tauri::command]
async fn render_frame() -> Result<FrameStats, String> {
    // This command is a placeholder
    // In a real implementation, the rendering happens on the frontend via WebGPU
    Ok(FrameStats::default())
}

/// Get current frame statistics
#[tauri::command]
async fn get_frame_stats() -> Result<FrameStats, String> {
    Ok(FrameStats::default())
}


/// Set transform mode
#[tauri::command]
async fn set_transform_mode(mode: String) -> Result<(), String> {
    // 实现变换模式设置（简化版本）
    match mode.as_str() {
        "translate" | "rotate" | "scale" => {
            // 有效的变换模式
            // 在实际实现中，这里会更新编辑器状态
            Ok(())
        }
        _ => {
            Err(format!("无效的变换模式: {}。有效值: translate, rotate, scale", mode))
        }
    }
}

/// Original greet command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(EntityManagerState::new(EntityManager::new()))
        .manage(SceneManagerState::new(scene_manager::SceneManager::new()))
        .manage(PerformanceMonitorState::new(Mutex::new(PerformanceMonitor::new())))
        .manage(Mutex::new(AnimationSystem::new()))
        .manage(Mutex::new(BehaviorTreeManager::new()))
        .manage(Mutex::new(BatchOperationsManager::new()))
        .manage(asset_store::AssetStoreState::new())
        .manage(tutorial::init_tutorial_system())
        .invoke_handler(tauri::generate_handler![
            greet,
            initialize_renderer,
            render_frame,
            get_frame_stats,
            set_transform_mode,
            // import_3d_model,  // Temporarily disabled
            // Entity manager commands
            entity_manager::create_entity,
            entity_manager::get_entity,
            entity_manager::update_entity,
            entity_manager::delete_entity,
            entity_manager::list_entities,
            entity_manager::rename_entity,
            entity_manager::duplicate_entity,
            entity_manager::set_entity_visibility,
            entity_manager::set_entity_lock,
            entity_manager::reparent_entity,
            // Scene manager commands
            scene_manager::create_scene,
            scene_manager::get_current_scene,
            scene_manager::set_current_scene,
            scene_manager::update_scene,
            // Asset manager commands
            asset_manager::list_assets,
            asset_manager::get_asset_preview,
            asset_manager::import_assets,
            asset_manager::delete_asset,
            asset_manager::rename_asset,
            asset_manager::get_asset_dependencies,
            asset_manager::create_folder,
            asset_manager::get_folder_tree,
            // Performance monitoring commands
            performance_commands::get_performance_metrics,
            performance_commands::get_performance_hotspots,
            performance_commands::get_alert_history,
            performance_commands::acknowledge_alert,
            performance_commands::clear_alerts,
            performance_commands::set_alert_threshold,
            performance_commands::get_alert_thresholds,
            performance_commands::get_performance_history,
            performance_commands::get_performance_statistics,
            performance_commands::export_performance_data,
            performance_commands::update_performance_metrics,
            performance_commands::start_monitoring,
            performance_commands::stop_monitoring,
            performance_commands::is_monitoring_active,
            // Animation system commands
            animation_system::create_animation_clip,
            animation_system::save_animation_clip,
            animation_system::load_animation_clip,
            animation_system::delete_animation_clip,
            animation_system::list_animation_clips,
            animation_system::add_keyframe,
            animation_system::update_keyframe,
            animation_system::delete_keyframe,
            animation_system::evaluate_animation_at_time,
            // Behavior tree commands
            behavior_tree::create_behavior_tree,
            behavior_tree::save_behavior_tree,
            behavior_tree::load_behavior_tree,
            behavior_tree::list_behavior_trees,
            behavior_tree::delete_behavior_tree,
            behavior_tree::validate_behavior_tree,
            behavior_tree::execute_behavior_tree,
            behavior_tree::debug_behavior_step,
            behavior_tree::set_breakpoint,
            behavior_tree::clear_breakpoint,
            // Batch operations commands
            batch_operations::batch_delete,
            batch_operations::batch_rename,
            batch_operations::batch_move,
            batch_operations::batch_rotate,
            batch_operations::batch_scale,
            batch_operations::batch_toggle_visibility,
            batch_operations::batch_toggle_locked,
            batch_operations::batch_apply_material,
            batch_operations::batch_component_operation,
            batch_operations::align_entities,
            batch_operations::distribute_entities,
            // Shortcut persistence commands
            shortcuts::save_shortcut_config,
            shortcuts::load_shortcut_config,
            shortcuts::export_shortcut_config,
            shortcuts::import_shortcut_config,
            shortcuts::reset_shortcut_config,
            shortcuts::backup_shortcut_config,
            shortcuts::list_shortcut_backups,
            // Asset store commands
            asset_store::search_assets,
            asset_store::download_asset,
            asset_store::import_asset,
            asset_store::get_preview,
            asset_store::get_asset_details,
            asset_store::add_favorite,
            asset_store::remove_favorite,
            asset_store::get_favorites,
            asset_store::get_download_history,
            asset_store::get_categories,
            asset_store::get_asset_types,
            // Tutorial system commands
            tutorial::get_tutorials,
            tutorial::get_tutorial,
            tutorial::create_tutorial,
            tutorial::update_tutorial,
            tutorial::delete_tutorial,
            tutorial::get_tutorial_progress,
            tutorial::start_tutorial,
            tutorial::complete_tutorial_step,
            tutorial::save_tutorial_progress,
            tutorial::complete_tutorial,
            tutorial::get_user_stats,
            tutorial::get_leaderboard,
            tutorial::execute_tutorial_code,
            tutorial::verify_tutorial_answer,
            tutorial::add_user_xp,
            tutorial::award_badge,
            tutorial::check_user_achievements,
            tutorial::log_tutorial_hint,
            tutorial::load_tutorials_from_disk,
            // New enhanced animation commands
            animation_commands::save_animation_clip,
            animation_commands::delete_animation_clip,
            animation_commands::list_animation_clips,
            animation_commands::load_animation_clip,
            // New enhanced asset commands
            asset_commands::get_assets_by_tags,
            asset_commands::search_assets,
            asset_commands::get_assets_by_type,
            asset_commands::get_all_assets,
            asset_commands::delete_asset,
            asset_commands::update_asset_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
