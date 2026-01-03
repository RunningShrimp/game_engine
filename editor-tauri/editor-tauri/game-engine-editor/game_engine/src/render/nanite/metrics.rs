//! # Quality Metrics and Control
//!
//! Adaptive quality system that adjusts rendering quality based on performance.

use std::time::{Duration, Instant};

/// Configuration for quality metrics
#[derive(Clone, Debug)]
pub struct MetricsConfig {
    /// Target frame time in milliseconds
    pub target_frame_time_ms: f32,
    /// Minimum target FPS
    pub min_target_fps: f32,
    /// Maximum target FPS
    pub max_target_fps: f32,
    /// Quality adjustment speed (0-1)
    pub adjustment_speed: f32,
    /// Minimum quality multiplier
    pub min_quality_multiplier: f32,
    /// Maximum quality multiplier
    pub max_quality_multiplier: f32,
    /// Enable frame rate limiting
    pub enable_frame_rate_limit: bool,
    /// Frame time history size
    pub frame_time_history_size: usize,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            target_frame_time_ms: 16.67, // 60 FPS
            min_target_fps: 30.0,
            max_target_fps: 144.0,
            adjustment_speed: 0.1,
            min_quality_multiplier: 0.5,
            max_quality_multiplier: 2.0,
            enable_frame_rate_limit: false,
            frame_time_history_size: 60,
        }
    }
}

/// Screen space error calculation
#[derive(Clone, Debug)]
pub struct ScreenSpaceError {
    /// Error in pixels
    pub error_pixels: f32,
    /// Error as percentage of screen height
    pub error_percentage: f32,
    /// Acceptable threshold
    pub threshold: f32,
}

impl ScreenSpaceError {
    /// Calculate screen space error
    pub fn calculate(
        geometric_error: f32,
        distance: f32,
        screen_height: f32,
        fov_y: f32,
    ) -> Self {
        // Screen space error formula: SSE = (geometric_error * projection_scale) / distance
        let projection_scale = screen_height / (2.0 * (fov_y * 0.5).tan());
        let error_pixels = if distance > 0.0 {
            (geometric_error * projection_scale) / distance
        } else {
            0.0
        };

        let error_percentage = (error_pixels / screen_height) * 100.0;
        let threshold = 1.0; // 1 pixel threshold

        Self {
            error_pixels,
            error_percentage,
            threshold,
        }
    }

    /// Check if error is acceptable
    pub fn is_acceptable(&self) -> bool {
        self.error_pixels <= self.threshold
    }
}

/// Performance statistics
#[derive(Clone, Debug, Default)]
pub struct PerformanceStats {
    /// Current FPS
    pub fps: f32,
    /// Frame time in milliseconds
    pub frame_time_ms: f32,
    /// Average FPS over last N frames
    pub average_fps: f32,
    /// 1% low FPS (frame time percentile)
    pub fps_1_percent_low: f32,
    /// 0.1% low FPS
    pub fps_0_1_percent_low: f32,
    /// GPU frame time in milliseconds
    pub gpu_frame_time_ms: f32,
    /// CPU frame time in milliseconds
    pub cpu_frame_time_ms: f32,
    /// Draw calls count
    pub draw_calls: usize,
    /// Triangle count
    pub triangle_count: usize,
    /// GPU memory usage in MB
    pub gpu_memory_mb: f32,
}

impl PerformanceStats {
    /// Create new performance stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Update FPS from frame time
    pub fn update_fps(&mut self, frame_time_ms: f32) {
        self.frame_time_ms = frame_time_ms;
        self.fps = if frame_time_ms > 0.0 {
            1000.0 / frame_time_ms
        } else {
            0.0
        };
    }

    /// Calculate average FPS from history
    pub fn calculate_average(&mut self, frame_times: &[f32]) {
        if frame_times.is_empty() {
            return;
        }

        let avg_time: f32 = frame_times.iter().sum::<f32>() / frame_times.len() as f32;
        self.average_fps = if avg_time > 0.0 {
            1000.0 / avg_time
        } else {
            0.0
        };
    }

    /// Calculate percentile FPS
    pub fn calculate_percentiles(&mut self, mut frame_times: Vec<f32>) {
        if frame_times.len() < 10 {
            return;
        }

        frame_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // 1% low = value at 99th percentile
        let idx_1 = (frame_times.len() * 99 / 100).min(frame_times.len() - 1);
        self.fps_1_percent_low = if frame_times[idx_1] > 0.0 {
            1000.0 / frame_times[idx_1]
        } else {
            0.0
        };

        // 0.1% low = value at 99.9th percentile
        let idx_0_1 = (frame_times.len() * 999 / 1000).min(frame_times.len() - 1);
        self.fps_0_1_percent_low = if frame_times[idx_0_1] > 0.0 {
            1000.0 / frame_times[idx_0_1]
        } else {
            0.0
        };
    }
}

/// Quality metrics for adaptive control
#[derive(Clone, Debug)]
pub struct QualityMetrics {
    /// Quality multiplier (0.5 = low, 1.0 = normal, 2.0 = high)
    pub quality_multiplier: f32,
    /// Current frame time in milliseconds
    pub frame_time_ms: f32,
    /// Target frame rate
    pub target_fps: f32,
    /// Available GPU memory in MB
    pub available_memory_mb: f32,
    /// Screen space error target
    pub screen_space_error_target: f32,
    /// LOD bias adjustment
    pub lod_bias: f32,
    /// Resolution scale
    pub resolution_scale: f32,
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            quality_multiplier: 1.0,
            frame_time_ms: 16.67,
            target_fps: 60.0,
            available_memory_mb: 1024.0,
            screen_space_error_target: 1.0,
            lod_bias: 0.0,
            resolution_scale: 1.0,
        }
    }
}

/// Quality controller for adaptive quality adjustment
pub struct QualityController {
    config: MetricsConfig,
    /// Current quality level
    current_quality: f32,
    /// Target quality level
    target_quality: f32,
    /// Frame time history
    frame_times: Vec<f32>,
    /// Last update time
    last_update: Instant,
    /// Performance statistics
    stats: PerformanceStats,
    /// Is quality stabilized
    stabilized: bool,
    /// Consecutive frames within target
    stable_frame_count: usize,
}

impl QualityController {
    /// Create new quality controller
    pub fn new(config: MetricsConfig) -> Result<Self, crate::render::nanite::QualityError> {
        Ok(Self {
            config,
            current_quality: 1.0,
            target_quality: 1.0,
            frame_times: Vec::with_capacity(config.frame_time_history_size),
            last_update: Instant::now(),
            stats: PerformanceStats::new(),
            stabilized: false,
            stable_frame_count: 0,
        })
    }

    /// Update quality controller (call once per frame)
    pub fn update(&mut self, delta_time: f32) -> Result<QualityMetrics, crate::render::nanite::QualityError> {
        // Record frame time
        self.frame_times.push(delta_time * 1000.0); // Convert to ms
        if self.frame_times.len() > self.config.frame_time_history_size {
            self.frame_times.remove(0);
        }

        // Update statistics
        self.stats.frame_time_ms = delta_time * 1000.0;
        self.stats.update_fps(delta_time * 1000.0);
        self.stats.calculate_average(&self.frame_times);

        if self.frame_times.len() > 10 {
            self.stats.calculate_percentiles(self.frame_times.clone());
        }

        // Adjust quality based on performance
        self.adjust_quality()?;

        // Build quality metrics
        let metrics = QualityMetrics {
            quality_multiplier: self.current_quality,
            frame_time_ms: self.stats.frame_time_ms,
            target_fps: 1000.0 / self.config.target_frame_time_ms,
            available_memory_mb: 1024.0, // Would get from actual GPU
            screen_space_error_target: self.config.target_frame_time_ms / self.current_quality,
            lod_bias: (1.0 - self.current_quality).max(-1.0),
            resolution_scale: self.current_quality.min(1.5).max(0.5),
        };

        self.last_update = Instant::now();

        Ok(metrics)
    }

    /// Adjust quality based on frame time
    fn adjust_quality(&mut self) -> Result<(), crate::render::nanite::QualityError> {
        let avg_frame_time = if self.frame_times.is_empty() {
            self.config.target_frame_time_ms
        } else {
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
        };

        let frame_time_ratio = avg_frame_time / self.config.target_frame_time_ms;

        // Determine target quality
        if frame_time_ratio > 1.2 {
            // Running too slow, decrease quality
            self.target_quality = (self.target_quality * (1.0 - self.config.adjustment_speed))
                .max(self.config.min_quality_multiplier);
            self.stable_frame_count = 0;
            self.stabilized = false;
        } else if frame_time_ratio < 0.9 {
            // Running fast, can increase quality
            self.target_quality = (self.target_quality * (1.0 + self.config.adjustment_speed))
                .min(self.config.max_quality_multiplier);

            self.stable_frame_count += 1;
            if self.stable_frame_count > 30 {
                self.stabilized = true;
            }
        } else {
            // Within target range
            self.stable_frame_count += 1;
            if self.stable_frame_count > 30 {
                self.stabilized = true;
            }
        }

        // Smoothly transition to target quality
        let diff = self.target_quality - self.current_quality;
        if diff.abs() > 0.01 {
            self.current_quality += diff * self.config.adjustment_speed;
        } else {
            self.current_quality = self.target_quality;
        }

        Ok(())
    }

    /// Get current performance statistics
    pub fn stats(&self) -> &PerformanceStats {
        &self.stats
    }

    /// Check if quality is stabilized
    pub fn is_stabilized(&self) -> bool {
        self.stabilized
    }

    /// Get current quality level
    pub fn current_quality(&self) -> f32 {
        self.current_quality
    }

    /// Set target quality manually
    pub fn set_target_quality(&mut self, quality: f32) {
        self.target_quality = quality.clamp(
            self.config.min_quality_multiplier,
            self.config.max_quality_multiplier,
        );
        self.stabilized = false;
        self.stable_frame_count = 0;
    }

    /// Reset quality to default
    pub fn reset_quality(&mut self) {
        self.target_quality = 1.0;
        self.current_quality = 1.0;
        self.stabilized = false;
        self.stable_frame_count = 0;
        self.frame_times.clear();
    }

    /// Get configuration
    pub fn config(&self) -> &MetricsConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: MetricsConfig) {
        self.config = config;
        self.frame_times.truncate(config.frame_time_history_size);
    }

    /// Force quality level (bypasses adaptive adjustment)
    pub fn force_quality(&mut self, quality: f32) {
        self.current_quality = quality.clamp(
            self.config.min_quality_multiplier,
            self.config.max_quality_multiplier,
        );
        self.target_quality = self.current_quality;
        self.stabilized = true;
    }

    /// Calculate screen space error for a cluster
    pub fn calculate_sse(
        &self,
        geometric_error: f32,
        distance: f32,
        screen_height: f32,
        fov_y: f32,
    ) -> ScreenSpaceError {
        ScreenSpaceError::calculate(geometric_error, distance, screen_height, fov_y)
    }

    /// Get recommended LOD bias
    pub fn get_lod_bias(&self) -> f32 {
        // Negative bias = higher quality
        // Positive bias = lower quality
        (1.0 - self.current_quality) * 2.0
    }

    /// Get recommended resolution scale
    pub fn get_resolution_scale(&self) -> f32 {
        self.current_quality.min(1.5).max(0.5)
    }
}

/// Adaptive quality preset
#[derive(Clone, Copy, Debug)]
pub enum QualityPreset {
    /// Maximum quality (may impact performance)
    Ultra,
    /// High quality
    High,
    /// Balanced quality and performance
    Medium,
    /// Low quality for better performance
    Low,
    /// Minimum quality
    Potato,
}

impl QualityPreset {
    /// Get quality multiplier for preset
    pub fn quality_multiplier(self) -> f32 {
        match self {
            QualityPreset::Ultra => 2.0,
            QualityPreset::High => 1.5,
            QualityPreset::Medium => 1.0,
            QualityPreset::Low => 0.75,
            QualityPreset::Potato => 0.5,
        }
    }

    /// Get target FPS for preset
    pub fn target_fps(self) -> f32 {
        match self {
            QualityPreset::Ultra => 30.0,
            QualityPreset::High => 60.0,
            QualityPreset::Medium => 60.0,
            QualityPreset::Low => 90.0,
            QualityPreset::Potato => 120.0,
        }
    }

    /// Get SSE threshold for preset
    pub fn sse_threshold(self) -> f32 {
        match self {
            QualityPreset::Ultra => 0.5,
            QualityPreset::High => 1.0,
            QualityPreset::Medium => 1.5,
            QualityPreset::Low => 2.0,
            QualityPreset::Potato => 3.0,
        }
    }
}

/// Errors that can occur in quality control
#[derive(Debug, thiserror::Error)]
pub enum QualityError {
    #[error("Invalid quality multiplier: {0}")]
    InvalidQualityMultiplier(f32),

    #[error("Quality adjustment failed: {0}")]
    AdjustmentFailed(String),

    #[error("Frame time history overflow")]
    HistoryOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_controller_creation() {
        let controller = QualityController::new(MetricsConfig::default()).unwrap();
        assert_eq!(controller.current_quality(), 1.0);
    }

    #[test]
    fn test_quality_adjustment() {
        let mut controller = QualityController::new(MetricsConfig::default()).unwrap();

        // Simulate slow frame time
        for _ in 0..10 {
            controller.update(0.025).unwrap(); // 25ms per frame
        }

        // Quality should decrease
        assert!(controller.current_quality() < 1.0);
    }

    #[test]
    fn test_screen_space_error() {
        let sse = ScreenSpaceError::calculate(0.1, 10.0, 1080.0, std::f32::consts::PI / 4.0);
        assert!(sse.error_pixels > 0.0);
        assert!(sse.error_percentage > 0.0);
    }

    #[test]
    fn test_quality_presets() {
        assert_eq!(QualityPreset::Ultra.quality_multiplier(), 2.0);
        assert_eq!(QualityPreset::Medium.quality_multiplier(), 1.0);
        assert_eq!(QualityPreset::Potato.quality_multiplier(), 0.5);
    }

    #[test]
    fn test_performance_stats() {
        let mut stats = PerformanceStats::new();
        stats.update_fps(16.67);
        assert!((stats.fps - 60.0).abs() < 1.0);
    }
}
