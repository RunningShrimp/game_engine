//! Performance Monitor Module
//!
//! This module provides real-time performance monitoring capabilities
//! for the game engine editor, collecting metrics from CPU, GPU, memory,
//! rendering, physics, and script execution.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Maximum number of historical data points to keep in memory
const MAX_HISTORY_POINTS: usize = 604800; // 7 days at 1 sample per second

/// Default alert thresholds
const DEFAULT_FPS_WARNING: f64 = 50.0;
const DEFAULT_FPS_CRITICAL: f64 = 30.0;
const DEFAULT_MEMORY_WARNING: f64 = 85.0; // percentage
const DEFAULT_MEMORY_CRITICAL: f64 = 95.0;
const DEFAULT_GPU_WARNING: f64 = 85.0; // percentage
const DEFAULT_GPU_CRITICAL: f64 = 95.0;
const DEFAULT_FRAME_TIME_WARNING: f64 = 20.0; // ms
const DEFAULT_FRAME_TIME_CRITICAL: f64 = 33.0; // ms
const DEFAULT_CPU_WARNING: f64 = 85.0; // percentage
const DEFAULT_CPU_CRITICAL: f64 = 95.0; // percentage

/// Core performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    // Frame metrics
    pub fps: f64,
    pub frame_time: f64,

    // CPU metrics
    pub cpu_usage: f64,
    pub cpu_usage_per_core: Vec<f64>,

    // GPU metrics
    pub gpu_usage: f64,
    pub gpu_memory: f64,
    pub gpu_memory_total: f64,

    // Memory metrics
    pub memory_used: f64,
    pub memory_total: f64,
    pub memory_used_by_system: f64,

    // Rendering metrics
    pub draw_calls: f64,
    pub triangles: f64,
    pub vertices: f64,

    // Physics metrics
    pub physics_time: f64,
    pub rigid_body_count: f64,
    pub collision_count: f64,

    // Script metrics
    pub script_time: f64,
    pub script_count: f64,

    // Audio metrics
    pub audio_time: f64,
    pub audio_source_count: f64,

    // Network metrics
    pub network_time: f64,
    pub network_bytes_received: f64,
    pub network_bytes_sent: f64,

    // Timestamp
    pub timestamp: i64,
}

/// Performance hotspot representing a function or system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHotspot {
    pub name: String,
    pub duration: f64,
    pub percentage: f64,
    pub call_count: f64,
    pub category: String, // "render", "physics", "script", "audio", "network", "other"
    pub children: Option<Vec<PerformanceHotspot>>,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub id: String,
    pub timestamp: i64,
    pub alert_type: String, // "fps", "memory", "gpu", "frame_time", "cpu", "leak"
    pub severity: String, // "info", "warning", "critical"
    pub message: String,
    pub value: f64,
    pub threshold: f64,
    pub acknowledged: bool,
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThreshold {
    pub fps_warning: f64,
    pub fps_critical: f64,
    pub memory_warning: f64,
    pub memory_critical: f64,
    pub gpu_warning: f64,
    pub gpu_critical: f64,
    pub frame_time_warning: f64,
    pub frame_time_critical: f64,
    pub cpu_warning: f64,
    pub cpu_critical: f64,
}

impl Default for AlertThreshold {
    fn default() -> Self {
        Self {
            fps_warning: DEFAULT_FPS_WARNING,
            fps_critical: DEFAULT_FPS_CRITICAL,
            memory_warning: DEFAULT_MEMORY_WARNING,
            memory_critical: DEFAULT_MEMORY_CRITICAL,
            gpu_warning: DEFAULT_GPU_WARNING,
            gpu_critical: DEFAULT_GPU_CRITICAL,
            frame_time_warning: DEFAULT_FRAME_TIME_WARNING,
            frame_time_critical: DEFAULT_FRAME_TIME_CRITICAL,
            cpu_warning: DEFAULT_CPU_WARNING,
            cpu_critical: DEFAULT_CPU_CRITICAL,
        }
    }
}

/// Performance statistics over a time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatistics {
    pub avg_fps: f64,
    pub min_fps: f64,
    pub max_fps: f64,
    pub avg_frame_time: f64,
    pub avg_cpu_usage: f64,
    pub avg_gpu_usage: f64,
    pub avg_memory_usage: f64,
    pub peak_memory_usage: f64,
    pub total_frames: f64,
    pub time_range_start: i64,
    pub time_range_end: i64,
}

/// Main performance monitor
pub struct PerformanceMonitor {
    metrics: PerformanceMetrics,
    history: VecDeque<PerformanceMetrics>,
    hotspots: Vec<PerformanceHotspot>,
    alerts: Vec<PerformanceAlert>,
    thresholds: AlertThreshold,
    last_frame_time: Instant,
    frame_times: VecDeque<Duration>,
    is_monitoring: bool,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Self::create_default_metrics(),
            history: VecDeque::with_capacity(MAX_HISTORY_POINTS),
            hotspots: Vec::new(),
            alerts: Vec::new(),
            thresholds: AlertThreshold::default(),
            last_frame_time: Instant::now(),
            frame_times: VecDeque::with_capacity(60),
            is_monitoring: false,
        }
    }

    fn create_default_metrics() -> PerformanceMetrics {
        PerformanceMetrics {
            fps: 60.0,
            frame_time: 16.67,
            cpu_usage: 0.0,
            cpu_usage_per_core: vec![0.0; 8],
            gpu_usage: 0.0,
            gpu_memory: 0.0,
            gpu_memory_total: 0.0,
            memory_used: 0.0,
            memory_total: 0.0,
            memory_used_by_system: 0.0,
            draw_calls: 0.0,
            triangles: 0.0,
            vertices: 0.0,
            physics_time: 0.0,
            rigid_body_count: 0.0,
            collision_count: 0.0,
            script_time: 0.0,
            script_count: 0.0,
            audio_time: 0.0,
            audio_source_count: 0.0,
            network_time: 0.0,
            network_bytes_received: 0.0,
            network_bytes_sent: 0.0,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn start_monitoring(&mut self) {
        self.is_monitoring = true;
        self.last_frame_time = Instant::now();
    }

    pub fn stop_monitoring(&mut self) {
        self.is_monitoring = false;
    }

    pub fn is_monitoring(&self) -> bool {
        self.is_monitoring
    }

    pub fn update(&mut self, partial_metrics: PartialMetrics) -> Result<(), String> {
        if !self.is_monitoring {
            return Ok(());
        }

        let now = Instant::now();
        let frame_duration = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;

        // Update frame times for FPS calculation
        self.frame_times.push_back(frame_duration);
        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }

        // Calculate FPS
        let avg_frame_duration: Duration = self.frame_times.iter().sum::<Duration>()
            / std::cmp::max(1, self.frame_times.len()) as u32;
        let fps = if avg_frame_duration.as_secs_f64() > 0.0 {
            1000.0 / avg_frame_duration.as_secs_f64()
        } else {
            60.0
        };

        // Update full metrics
        self.metrics.fps = fps;
        self.metrics.frame_time = avg_frame_duration.as_secs_f64() * 1000.0;
        self.metrics.timestamp = chrono::Utc::now().timestamp_millis();

        // Apply partial metrics
        partial_metrics.apply_to(&mut self.metrics);

        // Simulate system metrics (in real implementation, these would come from actual system calls)
        self.simulate_system_metrics();

        // Store in history
        self.history.push_back(self.metrics.clone());
        if self.history.len() > MAX_HISTORY_POINTS {
            self.history.pop_front();
        }

        // Check for alerts
        self.check_alerts();

        Ok(())
    }

    fn simulate_system_metrics(&mut self) {
        // In a real implementation, these would come from actual system monitoring
        // For now, we'll simulate some realistic values

        // CPU usage with some variation
        let cpu_variation = (self.metrics.timestamp % 1000) as f64 / 1000.0 * 20.0 - 10.0;
        self.metrics.cpu_usage = (30.0 + cpu_variation).max(0.0).min(100.0);

        // Per-core CPU usage
        for core in &mut self.metrics.cpu_usage_per_core {
            let core_variation = (self.metrics.timestamp % 500) as f64 / 500.0 * 30.0 - 15.0;
            *core = (25.0 + core_variation).max(0.0).min(100.0);
        }

        // GPU usage
        let gpu_variation = (self.metrics.timestamp % 800) as f64 / 800.0 * 25.0 - 12.5;
        self.metrics.gpu_usage = (40.0 + gpu_variation).max(0.0).min(100.0);

        // Memory (simulate using heuristics)
        self.metrics.memory_total = 16384.0; // 16 GB
        let mem_variation = (self.metrics.timestamp % 2000) as f64 / 2000.0 * 500.0;
        self.metrics.memory_used = (2048.0 + mem_variation).min(self.metrics.memory_total);

        // GPU memory
        self.metrics.gpu_memory_total = 8192.0; // 8 GB
        let gpu_mem_variation = (self.metrics.timestamp % 1500) as f64 / 1500.0 * 200.0;
        self.metrics.gpu_memory = (1024.0 + gpu_mem_variation).min(self.metrics.gpu_memory_total);
    }

    fn check_alerts(&mut self) {
        // Check FPS
        if self.metrics.fps < self.thresholds.fps_critical {
            self.add_alert(PerformanceAlert {
                id: Uuid::new_v4().to_string(),
                timestamp: self.metrics.timestamp,
                alert_type: "fps".to_string(),
                severity: "critical".to_string(),
                message: format!("Critical: FPS is too low ({:.1})", self.metrics.fps),
                value: self.metrics.fps,
                threshold: self.thresholds.fps_critical,
                acknowledged: false,
            });
        } else if self.metrics.fps < self.thresholds.fps_warning {
            self.add_alert(PerformanceAlert {
                id: Uuid::new_v4().to_string(),
                timestamp: self.metrics.timestamp,
                alert_type: "fps".to_string(),
                severity: "warning".to_string(),
                message: format!("Warning: FPS is below target ({:.1})", self.metrics.fps),
                value: self.metrics.fps,
                threshold: self.thresholds.fps_warning,
                acknowledged: false,
            });
        }

        // Check memory
        let memory_percent = (self.metrics.memory_used / self.metrics.memory_total) * 100.0;
        if memory_percent > self.thresholds.memory_critical {
            self.add_alert(PerformanceAlert {
                id: Uuid::new_v4().to_string(),
                timestamp: self.metrics.timestamp,
                alert_type: "memory".to_string(),
                severity: "critical".to_string(),
                message: format!("Critical: Memory usage is {:.1}%", memory_percent),
                value: memory_percent,
                threshold: self.thresholds.memory_critical,
                acknowledged: false,
            });
        } else if memory_percent > self.thresholds.memory_warning {
            self.add_alert(PerformanceAlert {
                id: Uuid::new_v4().to_string(),
                timestamp: self.metrics.timestamp,
                alert_type: "memory".to_string(),
                severity: "warning".to_string(),
                message: format!("Warning: High memory usage ({:.1}%)", memory_percent),
                value: memory_percent,
                threshold: self.thresholds.memory_warning,
                acknowledged: false,
            });
        }

        // Check GPU
        if self.metrics.gpu_usage > self.thresholds.gpu_critical {
            self.add_alert(PerformanceAlert {
                id: Uuid::new_v4().to_string(),
                timestamp: self.metrics.timestamp,
                alert_type: "gpu".to_string(),
                severity: "critical".to_string(),
                message: format!("Critical: GPU usage is {:.1}%", self.metrics.gpu_usage),
                value: self.metrics.gpu_usage,
                threshold: self.thresholds.gpu_critical,
                acknowledged: false,
            });
        } else if self.metrics.gpu_usage > self.thresholds.gpu_warning {
            self.add_alert(PerformanceAlert {
                id: Uuid::new_v4().to_string(),
                timestamp: self.metrics.timestamp,
                alert_type: "gpu".to_string(),
                severity: "warning".to_string(),
                message: format!("Warning: High GPU usage ({:.1}%)", self.metrics.gpu_usage),
                value: self.metrics.gpu_usage,
                threshold: self.thresholds.gpu_warning,
                acknowledged: false,
            });
        }

        // Check frame time
        if self.metrics.frame_time > self.thresholds.frame_time_critical {
            self.add_alert(PerformanceAlert {
                id: Uuid::new_v4().to_string(),
                timestamp: self.metrics.timestamp,
                alert_type: "frame_time".to_string(),
                severity: "critical".to_string(),
                message: format!("Critical: Frame time is {:.2}ms", self.metrics.frame_time),
                value: self.metrics.frame_time,
                threshold: self.thresholds.frame_time_critical,
                acknowledged: false,
            });
        } else if self.metrics.frame_time > self.thresholds.frame_time_warning {
            self.add_alert(PerformanceAlert {
                id: Uuid::new_v4().to_string(),
                timestamp: self.metrics.timestamp,
                alert_type: "frame_time".to_string(),
                severity: "warning".to_string(),
                message: format!("Warning: Frame time is {:.2}ms", self.metrics.frame_time),
                value: self.metrics.frame_time,
                threshold: self.thresholds.frame_time_warning,
                acknowledged: false,
            });
        }

        // Check CPU
        if self.metrics.cpu_usage > self.thresholds.cpu_critical {
            self.add_alert(PerformanceAlert {
                id: Uuid::new_v4().to_string(),
                timestamp: self.metrics.timestamp,
                alert_type: "cpu".to_string(),
                severity: "critical".to_string(),
                message: format!("Critical: CPU usage is {:.1}%", self.metrics.cpu_usage),
                value: self.metrics.cpu_usage,
                threshold: self.thresholds.cpu_critical,
                acknowledged: false,
            });
        } else if self.metrics.cpu_usage > self.thresholds.cpu_warning {
            self.add_alert(PerformanceAlert {
                id: Uuid::new_v4().to_string(),
                timestamp: self.metrics.timestamp,
                alert_type: "cpu".to_string(),
                severity: "warning".to_string(),
                message: format!("Warning: High CPU usage ({:.1}%)", self.metrics.cpu_usage),
                value: self.metrics.cpu_usage,
                threshold: self.thresholds.cpu_warning,
                acknowledged: false,
            });
        }
    }

    fn add_alert(&mut self, alert: PerformanceAlert) {
        // Avoid duplicate alerts of the same type within 5 seconds
        let recent_alerts: Vec<_> = self.alerts.iter()
            .filter(|a| a.timestamp > self.metrics.timestamp - 5000)
            .filter(|a| a.alert_type == alert.alert_type && a.severity == alert.severity)
            .collect();

        if recent_alerts.is_empty() {
            self.alerts.push(alert);
        }
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.clone()
    }

    pub fn get_history(&self, start_time: i64, end_time: i64) -> Vec<PerformanceMetrics> {
        self.history.iter()
            .filter(|m| m.timestamp >= start_time && m.timestamp <= end_time)
            .cloned()
            .collect()
    }

    pub fn get_statistics(&self, start_time: i64, end_time: i64) -> PerformanceStatistics {
        let relevant_metrics: Vec<_> = self.history.iter()
            .filter(|m| m.timestamp >= start_time && m.timestamp <= end_time)
            .collect();

        if relevant_metrics.is_empty() {
            return PerformanceStatistics {
                avg_fps: 0.0,
                min_fps: 0.0,
                max_fps: 0.0,
                avg_frame_time: 0.0,
                avg_cpu_usage: 0.0,
                avg_gpu_usage: 0.0,
                avg_memory_usage: 0.0,
                peak_memory_usage: 0.0,
                total_frames: 0.0,
                time_range_start: start_time,
                time_range_end: end_time,
            };
        }

        let count = relevant_metrics.len() as f64;
        let avg_fps = relevant_metrics.iter().map(|m| m.fps).sum::<f64>() / count;
        let min_fps = relevant_metrics.iter().map(|m| m.fps).fold(f64::INFINITY, f64::min);
        let max_fps = relevant_metrics.iter().map(|m| m.fps).fold(f64::NEG_INFINITY, f64::max);
        let avg_frame_time = relevant_metrics.iter().map(|m| m.frame_time).sum::<f64>() / count;
        let avg_cpu_usage = relevant_metrics.iter().map(|m| m.cpu_usage).sum::<f64>() / count;
        let avg_gpu_usage = relevant_metrics.iter().map(|m| m.gpu_usage).sum::<f64>() / count;
        let avg_memory_usage = relevant_metrics.iter().map(|m| m.memory_used).sum::<f64>() / count;
        let peak_memory_usage = relevant_metrics.iter().map(|m| m.memory_used).fold(f64::NEG_INFINITY, f64::max);

        PerformanceStatistics {
            avg_fps,
            min_fps,
            max_fps,
            avg_frame_time,
            avg_cpu_usage,
            avg_gpu_usage,
            avg_memory_usage,
            peak_memory_usage,
            total_frames: count,
            time_range_start: start_time,
            time_range_end: end_time,
        }
    }

    pub fn get_hotspots(&self) -> Vec<PerformanceHotspot> {
        // Generate simulated hotspots based on current metrics
        let mut hotspots = vec![
            PerformanceHotspot {
                name: "PhysicsSystem::update".to_string(),
                duration: self.metrics.physics_time,
                percentage: (self.metrics.physics_time / self.metrics.frame_time * 100.0).min(100.0),
                call_count: 60.0,
                category: "physics".to_string(),
                children: None,
            },
            PerformanceHotspot {
                name: "RenderSystem::render".to_string(),
                duration: self.metrics.frame_time * 0.4,
                percentage: 40.0,
                call_count: 60.0,
                category: "render".to_string(),
                children: None,
            },
            PerformanceHotspot {
                name: "ScriptSystem::update".to_string(),
                duration: self.metrics.script_time,
                percentage: (self.metrics.script_time / self.metrics.frame_time * 100.0).min(100.0),
                call_count: 120.0,
                category: "script".to_string(),
                children: None,
            },
        ];

        // Sort by duration (descending)
        hotspots.sort_by(|a, b| b.duration.partial_cmp(&a.duration).unwrap_or(std::cmp::Ordering::Equal));

        hotspots
    }

    pub fn get_alerts(&self) -> Vec<PerformanceAlert> {
        self.alerts.clone()
    }

    pub fn acknowledge_alert(&mut self, alert_id: &str) {
        if let Some(alert) = self.alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
        }
    }

    pub fn clear_alerts(&mut self) {
        self.alerts.clear();
    }

    pub fn set_threshold(&mut self, alert_type: &str, threshold: f64) -> Result<(), String> {
        match alert_type {
            "fps_warning" => self.thresholds.fps_warning = threshold,
            "fps_critical" => self.thresholds.fps_critical = threshold,
            "memory_warning" => self.thresholds.memory_warning = threshold,
            "memory_critical" => self.thresholds.memory_critical = threshold,
            "gpu_warning" => self.thresholds.gpu_warning = threshold,
            "gpu_critical" => self.thresholds.gpu_critical = threshold,
            "frame_time_warning" => self.thresholds.frame_time_warning = threshold,
            "frame_time_critical" => self.thresholds.frame_time_critical = threshold,
            "cpu_warning" => self.thresholds.cpu_warning = threshold,
            "cpu_critical" => self.thresholds.cpu_critical = threshold,
            _ => return Err(format!("Unknown alert type: {}", alert_type)),
        }
        Ok(())
    }

    pub fn get_thresholds(&self) -> AlertThreshold {
        self.thresholds.clone()
    }

    pub fn export_data(&self, format: &str, start_time: i64, end_time: i64) -> Result<String, String> {
        let history = self.get_history(start_time, end_time);

        match format {
            "json" => {
                serde_json::to_string_pretty(&history)
                    .map_err(|e| format!("Failed to serialize JSON: {}", e))
            }
            "csv" => {
                let mut csv = String::from("timestamp,fps,frame_time,cpu_usage,gpu_usage,memory_used,draw_calls,triangles\n");
                for m in &history {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{},{}\n",
                        m.timestamp, m.fps, m.frame_time, m.cpu_usage,
                        m.gpu_usage, m.memory_used, m.draw_calls, m.triangles
                    ));
                }
                Ok(csv)
            }
            _ => Err(format!("Unsupported export format: {}", format)),
        }
    }
}

/// Partial metrics for updates
pub struct PartialMetrics {
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

impl PartialMetrics {
    pub fn apply_to(self, metrics: &mut PerformanceMetrics) {
        if let Some(v) = self.draw_calls {
            metrics.draw_calls = v;
        }
        if let Some(v) = self.triangles {
            metrics.triangles = v;
        }
        if let Some(v) = self.vertices {
            metrics.vertices = v;
        }
        if let Some(v) = self.physics_time {
            metrics.physics_time = v;
        }
        if let Some(v) = self.rigid_body_count {
            metrics.rigid_body_count = v;
        }
        if let Some(v) = self.collision_count {
            metrics.collision_count = v;
        }
        if let Some(v) = self.script_time {
            metrics.script_time = v;
        }
        if let Some(v) = self.script_count {
            metrics.script_count = v;
        }
        if let Some(v) = self.audio_time {
            metrics.audio_time = v;
        }
        if let Some(v) = self.audio_source_count {
            metrics.audio_source_count = v;
        }
        if let Some(v) = self.network_time {
            metrics.network_time = v;
        }
        if let Some(v) = self.network_bytes_received {
            metrics.network_bytes_received = v;
        }
        if let Some(v) = self.network_bytes_sent {
            metrics.network_bytes_sent = v;
        }
    }
}

impl Default for PartialMetrics {
    fn default() -> Self {
        Self {
            draw_calls: None,
            triangles: None,
            vertices: None,
            physics_time: None,
            rigid_body_count: None,
            collision_count: None,
            script_time: None,
            script_count: None,
            audio_time: None,
            audio_source_count: None,
            network_time: None,
            network_bytes_received: None,
            network_bytes_sent: None,
        }
    }
}
