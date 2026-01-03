//! Tauri commands for performance monitoring
//!
//! This module exposes the performance monitoring functionality
//! to the frontend through Tauri's command system.

use crate::performance_monitor::{PerformanceMonitor, PartialMetrics};
use std::sync::Mutex;
use tauri::State;

/// Global performance monitor state
pub type PerformanceMonitorState = Mutex<PerformanceMonitor>;

/// Get current performance metrics
#[tauri::command]
pub fn get_performance_metrics(
    monitor: State<'_, PerformanceMonitorState>,
) -> Result<crate::performance_monitor::PerformanceMetrics, String> {
    let monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(monitor.get_metrics())
}

/// Get performance hotspots
#[tauri::command]
pub fn get_performance_hotspots(
    monitor: State<'_, PerformanceMonitorState>,
) -> Result<Vec<crate::performance_monitor::PerformanceHotspot>, String> {
    let monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(monitor.get_hotspots())
}

/// Get alert history
#[tauri::command]
pub fn get_alert_history(
    monitor: State<'_, PerformanceMonitorState>,
) -> Result<Vec<crate::performance_monitor::PerformanceAlert>, String> {
    let monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(monitor.get_alerts())
}

/// Acknowledge an alert
#[tauri::command]
pub fn acknowledge_alert(
    monitor: State<'_, PerformanceMonitorState>,
    alert_id: String,
) -> Result<(), String> {
    let mut monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    monitor.acknowledge_alert(&alert_id);
    Ok(())
}

/// Clear all alerts
#[tauri::command]
pub fn clear_alerts(
    monitor: State<'_, PerformanceMonitorState>,
) -> Result<(), String> {
    let mut monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    monitor.clear_alerts();
    Ok(())
}

/// Set alert threshold
#[tauri::command]
pub fn set_alert_threshold(
    monitor: State<'_, PerformanceMonitorState>,
    alert_type: String,
    threshold: f64,
) -> Result<(), String> {
    let mut monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    monitor.set_threshold(&alert_type, threshold)
}

/// Get alert thresholds
#[tauri::command]
pub fn get_alert_thresholds(
    monitor: State<'_, PerformanceMonitorState>,
) -> Result<crate::performance_monitor::AlertThreshold, String> {
    let monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(monitor.get_thresholds())
}

/// Get historical performance data
#[tauri::command]
pub fn get_performance_history(
    monitor: State<'_, PerformanceMonitorState>,
    start_time: i64,
    end_time: i64,
) -> Result<Vec<crate::performance_monitor::PerformanceMetrics>, String> {
    let monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(monitor.get_history(start_time, end_time))
}

/// Get performance statistics
#[tauri::command]
pub fn get_performance_statistics(
    monitor: State<'_, PerformanceMonitorState>,
    start_time: i64,
    end_time: i64,
) -> Result<crate::performance_monitor::PerformanceStatistics, String> {
    let monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(monitor.get_statistics(start_time, end_time))
}

/// Export performance data
#[tauri::command]
pub fn export_performance_data(
    monitor: State<'_, PerformanceMonitorState>,
    format: String,
    start_time: i64,
    end_time: i64,
) -> Result<String, String> {
    let monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    monitor.export_data(&format, start_time, end_time)
}

/// Update performance metrics (called by the engine)
#[tauri::command]
pub fn update_performance_metrics(
    monitor: State<'_, PerformanceMonitorState>,
    metrics: PartialMetricsUpdate,
) -> Result<(), String> {
    let mut monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    let partial = PartialMetrics {
        draw_calls: metrics.draw_calls,
        triangles: metrics.triangles,
        vertices: metrics.vertices,
        physics_time: metrics.physics_time,
        rigid_body_count: metrics.rigid_body_count,
        collision_count: metrics.collision_count,
        script_time: metrics.script_time,
        script_count: metrics.script_count,
        audio_time: metrics.audio_time,
        audio_source_count: metrics.audio_source_count,
        network_time: metrics.network_time,
        network_bytes_received: metrics.network_bytes_received,
        network_bytes_sent: metrics.network_bytes_sent,
    };
    monitor.update(partial)
}

/// Start monitoring
#[tauri::command]
pub fn start_monitoring(
    monitor: State<'_, PerformanceMonitorState>,
) -> Result<(), String> {
    let mut monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    monitor.start_monitoring();
    Ok(())
}

/// Stop monitoring
#[tauri::command]
pub fn stop_monitoring(
    monitor: State<'_, PerformanceMonitorState>,
) -> Result<(), String> {
    let mut monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    monitor.stop_monitoring();
    Ok(())
}

/// Check if monitoring is active
#[tauri::command]
pub fn is_monitoring_active(
    monitor: State<'_, PerformanceMonitorState>,
) -> Result<bool, String> {
    let monitor = monitor.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(monitor.is_monitoring())
}

/// Partial metrics update from engine
#[derive(serde::Deserialize)]
pub struct PartialMetricsUpdate {
    pub draw_calls: Option<f64>,
    pub triangles: Option<f64>,
    pub vertices: Option<f64>,
    pub physics_time: Option<f64>,
    pub rigid_body_count: Option<f64>,
    pub collision_count: Option<f64>,
    pub script_time: Option<f64>,
    pub script_count: Option<f64>,
    pub audio_time: Option<f64>,
    pub audio_source_count: Option<f64>,
    pub network_time: Option<f64>,
    pub network_bytes_received: Option<f64>,
    pub network_bytes_sent: Option<f64>,
}
