//! # Level of Detail (LOD) Management
//!
//! This module handles LOD selection and management for Nanite clusters.
//! It chooses appropriate LOD levels based on screen space error and distance.

use std::collections::HashMap;
use crate::render::nanite::{Vec3, ClusterHierarchy, Camera};

/// Configuration for LOD management
#[derive(Clone, Debug)]
pub struct LODConfig {
    /// Maximum LOD depth
    pub max_lod_depth: u8,
    /// Target screen space error (in pixels)
    pub target_screen_space_error: f32,
    /// LOD transition distance factor
    pub distance_factor: f32,
    /// Enable smooth LOD transitions
    pub smooth_transitions: bool,
    /// LOD transition speed (0-1)
    pub transition_speed: f32,
}

impl Default for LODConfig {
    fn default() -> Self {
        Self {
            max_lod_depth: 8,
            target_screen_space_error: 1.0,
            distance_factor: 1.0,
            smooth_transitions: true,
            transition_speed: 0.1,
        }
    }
}

/// A single LOD level
#[derive(Clone, Debug)]
pub struct LODLevel {
    /// LOD level (0 = highest detail)
    pub level: u8,
    /// Triangle count at this LOD
    pub triangle_count: usize,
    /// Screen space error at this LOD
    pub screen_space_error: f32,
    /// Maximum visible distance for this LOD
    pub max_distance: f32,
}

/// LOD selection result for a single cluster
#[derive(Clone, Debug)]
pub struct LODSelection {
    /// Cluster ID
    pub cluster_id: u32,
    /// Selected LOD level
    pub lod_level: u8,
    /// Predicted screen space error
    pub screen_space_error: f32,
    /// Distance from camera
    pub distance: f32,
    /// Visibility flag
    pub visible: bool,
}

impl LODSelection {
    /// Get triangle count for selected LOD
    pub fn triangle_count(&self) -> usize {
        // This would be looked up from actual LOD data
        // Simplified formula for now
        let base_count = 128usize;
        base_count >> self.lod_level as usize
    }
}

/// LOD manager for Nanite system
pub struct LODManager {
    config: LODConfig,
    /// Cache of LOD levels per cluster
    lod_cache: HashMap<u32, Vec<LODLevel>>,
    /// Previous frame selections for smooth transitions
    previous_selections: HashMap<u32, LODSelection>,
}

impl LODManager {
    /// Create a new LOD manager
    pub fn new(config: LODConfig) -> Result<Self, crate::render::nanite::LODError> {
        Ok(Self {
            config,
            lod_cache: HashMap::new(),
            previous_selections: HashMap::new(),
        })
    }

    /// Select LODs for all clusters based on camera view
    pub fn select_lods(
        &mut self,
        hierarchies: &[ClusterHierarchy],
        camera: &Camera,
        culling_results: &CullingResults,
        quality_metrics: &QualityMetrics,
    ) -> Result<Vec<LODSelection>, crate::render::nanite::LODError> {
        let mut selections = Vec::new();

        for hierarchy in hierarchies {
            for node in &hierarchy.nodes {
                let cluster = &node.cluster;

                // Check if cluster is visible
                let is_visible = culling_results.is_cluster_visible(cluster.id);
                if !is_visible {
                    continue;
                }

                // Calculate distance from camera
                let distance = self.calculate_distance(camera, &cluster.sphere_center);

                // Calculate screen space error
                let screen_space_error = self.calculate_screen_space_error(
                    cluster,
                    distance,
                    camera,
                    quality_metrics,
                );

                // Select appropriate LOD level
                let lod_level = self.select_lod_level(
                    cluster,
                    distance,
                    screen_space_error,
                    quality_metrics,
                );

                // Apply smooth transitions if enabled
                let final_lod_level = if self.config.smooth_transitions {
                    self.apply_smooth_transition(cluster.id, lod_level)
                } else {
                    lod_level
                };

                let selection = LODSelection {
                    cluster_id: cluster.id,
                    lod_level: final_lod_level,
                    screen_space_error,
                    distance,
                    visible: true,
                };

                selections.push(selection);
                self.previous_selections.insert(cluster.id, selection);
            }
        }

        Ok(selections)
    }

    /// Calculate distance from camera to cluster
    fn calculate_distance(&self, camera: &Camera, cluster_center: &Vec3) -> f32 {
        let dx = cluster_center[0] - camera.position[0];
        let dy = cluster_center[1] - camera.position[1];
        let dz = cluster_center[2] - camera.position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Calculate screen space error for a cluster
    fn calculate_screen_space_error(
        &self,
        cluster: &Cluster,
        distance: f32,
        camera: &Camera,
        quality_metrics: &QualityMetrics,
    ) -> f32 {
        if distance < 0.001 {
            return 0.0;
        }

        // Simplified screen space error calculation
        // SSE = (error * projection_scale) / distance
        let error = cluster.screen_space_error.max(0.0001);
        let projection_scale = camera.projection_matrix[1][1] * camera.aspect_ratio;
        let sse = (error * projection_scale) / distance;

        // Apply quality adjustment
        sse * quality_metrics.quality_multiplier
    }

    /// Select LOD level based on distance and screen space error
    fn select_lod_level(
        &self,
        cluster: &Cluster,
        distance: f32,
        screen_space_error: f32,
        quality_metrics: &QualityMetrics,
    ) -> u8 {
        // Determine target LOD based on screen space error
        let mut target_lod = if screen_space_error < self.config.target_screen_space_error {
            0 // Use highest detail
        } else {
            // Calculate LOD level from error
            let error_ratio = screen_space_error / self.config.target_screen_space_error;
            (error_ratio.log2().ceil() as u8).min(self.config.max_lod_depth)
        };

        // Apply quality bias
        let quality_bias = (1.0 - quality_metrics.quality_multiplier).max(-0.5) * 2.0;
        target_lod = (target_lod as f32 + quality_bias).floor() as u8;
        target_lod = target_lod.min(self.config.max_lod_depth);

        // Respect cluster's LOD level range
        target_lod = target_lod.max(cluster.lod_level);

        target_lod
    }

    /// Apply smooth LOD transition
    fn apply_smooth_transition(&self, cluster_id: u32, target_lod: u8) -> u8 {
        if let Some(prev) = self.previous_selections.get(&cluster_id) {
            let diff = target_lod as i8 - prev.lod_level as i8;
            if diff.abs() <= 1 {
                return target_lod;
            }

            // Limit transition speed
            let max_change = (self.config.transition_speed * 10.0) as i8;
            let change = diff.signum() * diff.abs().min(max_change);

            (prev.lod_level as i8 + change).max(0) as u8
        } else {
            target_lod
        }
    }

    /// Calculate average LOD level from selections
    pub fn average_lod(&self, selections: &[LODSelection]) -> f32 {
        if selections.is_empty() {
            return 0.0;
        }

        let sum: f32 = selections.iter().map(|s| s.lod_level as f32).sum();
        sum / selections.len() as f32
    }

    /// Get LOD levels for a cluster
    pub fn get_cluster_lods(&self, cluster_id: u32) -> Option<&[LODLevel]> {
        self.lod_cache.get(&cluster_id).map(|v| v.as_slice())
    }

    /// Precompute LOD levels for a cluster
    pub fn precompute_cluster_lods(
        &mut self,
        cluster: &Cluster,
        base_triangle_count: usize,
    ) {
        let mut lods = Vec::new();

        for level in 0..=self.config.max_lod_depth {
            let reduction_factor = 0.5_f32.powi(level as i32);
            let triangle_count = (base_triangle_count as f32 * reduction_factor).max(1.0) as usize;
            let screen_space_error = 0.001 * (2_f32.powi(level as i32));

            // Calculate max distance for this LOD
            let max_distance = if level == 0 {
                f32::MAX
            } else {
                // Distance where SSE becomes acceptable
                (screen_space_error * 1000.0) / self.config.target_screen_space_error
            };

            lods.push(LODLevel {
                level,
                triangle_count,
                screen_space_error,
                max_distance,
            });
        }

        self.lod_cache.insert(cluster.id, lods);
    }

    /// Clear LOD cache
    pub fn clear_cache(&mut self) {
        self.lod_cache.clear();
        self.previous_selections.clear();
    }

    /// Get configuration
    pub fn config(&self) -> &LODConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: LODConfig) {
        self.config = config;
        self.clear_cache();
    }
}

/// Placeholder for Cluster (will be imported from clustering module)
#[derive(Clone, Debug)]
pub struct Cluster {
    pub id: u32,
    pub lod_level: u8,
    pub sphere_center: Vec3,
    pub sphere_radius: f32,
    pub screen_space_error: f32,
    pub triangle_count: usize,
}

/// Placeholder for CullingResults
#[derive(Clone, Debug)]
pub struct CullingResults {
    visible_clusters: std::collections::HashSet<u32>,
}

impl CullingResults {
    pub fn new() -> Self {
        Self {
            visible_clusters: std::collections::HashSet::new(),
        }
    }

    pub fn is_cluster_visible(&self, cluster_id: u32) -> bool {
        self.visible_clusters.contains(&cluster_id)
    }

    pub fn mark_visible(&mut self, cluster_id: u32) {
        self.visible_clusters.insert(cluster_id);
    }

    pub fn total_visible_clusters(&self) -> usize {
        self.visible_clusters.len()
    }

    pub fn total_visible_triangles(&self) -> usize {
        // Simplified - would need actual triangle counts
        self.visible_clusters.len() * 64 // Average estimate
    }

    pub fn total_culled_clusters(&self) -> usize {
        // Would need total cluster count
        0
    }
}

/// Quality metrics from adaptive quality system
#[derive(Clone, Debug)]
pub struct QualityMetrics {
    /// Quality multiplier (0.5 = low quality, 1.0 = normal, 1.5 = high quality)
    pub quality_multiplier: f32,
    /// Current frame time in ms
    pub frame_time_ms: f32,
    /// Target frame rate
    pub target_fps: f32,
    /// Available GPU memory in MB
    pub available_memory_mb: f32,
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            quality_multiplier: 1.0,
            frame_time_ms: 16.67,
            target_fps: 60.0,
            available_memory_mb: 1024.0,
        }
    }
}

/// Errors that can occur in LOD management
#[derive(Debug, thiserror::Error)]
pub enum LODError {
    #[error("Invalid LOD level: {0}")]
    InvalidLODLevel(u8),

    #[error("LOD cache miss for cluster: {0}")]
    CacheMiss(u32),

    #[error("LOD computation failed: {0}")]
    ComputationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_manager_creation() {
        let manager = LODManager::new(LODConfig::default()).unwrap();
        assert_eq!(manager.config().max_lod_depth, 8);
    }

    #[test]
    fn test_lod_selection() {
        let mut manager = LODManager::new(LODConfig::default()).unwrap();

        // Create mock cluster
        let cluster = Cluster {
            id: 0,
            lod_level: 0,
            sphere_center: [0.0, 0.0, 0.0],
            sphere_radius: 1.0,
            screen_space_error: 0.01,
            triangle_count: 128,
        };

        manager.precompute_cluster_lods(&cluster, 128);

        let lods = manager.get_cluster_lods(0);
        assert!(lods.is_some());
        assert!(!lods.unwrap().is_empty());
    }

    #[test]
    fn test_distance_calculation() {
        let manager = LODManager::new(LODConfig::default()).unwrap();

        let camera = Camera {
            position: [0.0, 0.0, 0.0],
            view_matrix: [[0.0; 4]; 4],
            projection_matrix: [[0.0; 4]; 4],
            fov_y: std::f32::consts::PI / 4.0,
            aspect_ratio: 16.0 / 9.0,
            near_plane: 0.1,
            far_plane: 1000.0,
        };

        let cluster_center = [10.0, 0.0, 0.0];
        let distance = manager.calculate_distance(&camera, &cluster_center);

        assert!((distance - 10.0).abs() < 0.001);
    }
}
