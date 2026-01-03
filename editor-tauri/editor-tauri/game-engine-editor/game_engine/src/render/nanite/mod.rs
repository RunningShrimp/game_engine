//! # Nanite Virtual Geometry System
//!
//! This module implements a virtual geometry system inspired by Unreal Engine 5's Nanite technology.
//! Nanite allows rendering of extremely high-poly meshes (millions of triangles) with real-time performance
//! through intelligent clustering, LOD management, and culling.
//!
//! ## Key Components:
//!
//! - **Clustering**: Divides meshes into hierarchical clusters of triangles
//! - **LOD Management**: Manages multiple levels of detail for each cluster
//! - **Culling**: Performs view frustum and occlusion culling at cluster granularity
//! - **Rendering**: GPU-driven rendering with compute shader acceleration
//! - **Buffer Management**: Efficient GPU buffer management for instance data
//!
//! ## Architecture:
//!
//! The system is organized into several modules:
//! - `clustering`: Mesh clustering and hierarchy building
//! - `lod_manager`: LOD selection and management
//! - `culling`: View frustum and occlusion culling
//! - `renderer`: Main Nanite renderer
//! - `buffer`: GPU buffer management
//! - `metrics`: Quality metrics and performance monitoring

pub mod clustering;
pub mod lod_manager;
pub mod culling;
pub mod renderer;
pub mod buffer;
pub mod metrics;

pub use clustering::{
    Cluster, ClusterNode, ClusterHierarchy, ClusterBuilder, TriangleCluster,
    ClusterError, ClusterConfig
};
pub use lod_manager::{
    LODLevel, LODManager, LODSelection, LODError, LODConfig
};
pub use culling::{
    CullingSystem, CullingResult, ClusterVisibility, OcclusionCulling,
    CullingConfig, CullingStats
};
pub use renderer::{
    NaniteRenderer, RenderContext, RenderStats, RenderError, RenderConfig
};
pub use buffer::{
    InstanceBuffer, BufferManager, BufferAllocation, BufferError,
    BufferConfig, InstanceData
};
pub use metrics::{
    QualityMetrics, QualityController, AdaptiveQuality, MetricsConfig,
    PerformanceStats, ScreenSpaceError
};

use std::sync::Arc;
use wgpu::*;

/// Main Nanite virtual geometry system
///
/// This struct manages the entire Nanite pipeline, from mesh clustering to rendering.
#[derive(Clone)]
pub struct NaniteSystem {
    /// Cluster hierarchy for all registered meshes
    hierarchies: Vec<ClusterHierarchy>,
    /// LOD manager
    lod_manager: Arc<LODManager>,
    /// Culling system
    culling_system: Arc<CullingSystem>,
    /// Buffer manager for GPU resources
    buffer_manager: Arc<BufferManager>,
    /// Quality controller for adaptive quality
    quality_controller: Arc<QualityController>,
    /// Configuration
    config: NaniteConfig,
}

/// Configuration for the Nanite system
#[derive(Clone, Debug)]
pub struct NaniteConfig {
    /// Maximum number of triangles per cluster
    pub max_triangles_per_cluster: usize,
    /// Maximum LOD depth
    pub max_lod_depth: u8,
    /// Target screen space error (in pixels)
    pub target_screen_space_error: f32,
    /// Enable occlusion culling
    pub enable_occlusion_culling: bool,
    /// Enable compute shader acceleration
    pub enable_compute_acceleration: bool,
    /// Minimum cluster size for culling
    pub min_culling_cluster_size: u32,
    /// Instance buffer size in MB
    pub instance_buffer_size_mb: u32,
}

impl Default for NaniteConfig {
    fn default() -> Self {
        Self {
            max_triangles_per_cluster: 128,
            max_lod_depth: 8,
            target_screen_space_error: 1.0,
            enable_occlusion_culling: true,
            enable_compute_acceleration: true,
            min_culling_cluster_size: 4,
            instance_buffer_size_mb: 256,
        }
    }
}

impl NaniteSystem {
    /// Create a new Nanite system
    pub fn new(device: &Device, config: NaniteConfig) -> Result<Self, NaniteError> {
        // Initialize LOD manager
        let lod_manager = Arc::new(LODManager::new(LODConfig {
            max_lod_depth: config.max_lod_depth,
            target_screen_space_error: config.target_screen_space_error,
            ..Default::default()
        })?);

        // Initialize culling system
        let culling_system = Arc::new(CullingSystem::new(CullingConfig {
            enable_occlusion_culling: config.enable_occlusion_culling,
            min_cluster_size: config.min_culling_cluster_size,
            ..Default::default()
        })?);

        // Initialize buffer manager
        let buffer_manager = Arc::new(BufferManager::new(device, BufferConfig {
            instance_buffer_size_mb: config.instance_buffer_size_mb,
            enable_compute_acceleration: config.enable_compute_acceleration,
            ..Default::default()
        })?);

        // Initialize quality controller
        let quality_controller = Arc::new(QualityController::new(MetricsConfig {
            target_frame_time_ms: 16.67, // 60 FPS
            min_target_fps: 30,
            max_target_fps: 144,
            ..Default::default()
        })?);

        Ok(Self {
            hierarchies: Vec::new(),
            lod_manager,
            culling_system,
            buffer_manager,
            quality_controller,
            config,
        })
    }

    /// Register a mesh with the Nanite system
    pub fn register_mesh(
        &mut self,
        device: &Device,
        vertices: &[Vec3],
        indices: &[u32],
    ) -> Result<usize, NaniteError> {
        // Build cluster hierarchy
        let cluster_config = ClusterConfig {
            max_triangles_per_cluster: self.config.max_triangles_per_cluster,
            max_depth: self.config.max_lod_depth,
            ..Default::default()
        };

        let mut builder = ClusterBuilder::new(cluster_config);
        let hierarchy = builder.build_hierarchy(vertices, indices)?;

        let mesh_id = self.hierarchies.len();
        self.hierarchies.push(hierarchy);

        // Upload instance data to GPU
        self.buffer_manager.upload_mesh_instances(device, &self.hierarchies[mesh_id])?;

        Ok(mesh_id)
    }

    /// Update the Nanite system (call once per frame)
    pub fn update(
        &mut self,
        device: &Device,
        queue: &Queue,
        camera: &Camera,
        delta_time: f32,
    ) -> Result<RenderStats, NaniteError> {
        // Update quality metrics
        let quality_metrics = self.quality_controller.update(delta_time)?;

        // Perform culling
        let culling_results = self.culling_system.cull_all(
            device,
            queue,
            &self.hierarchies,
            camera,
            &quality_metrics,
        )?;

        // Select LODs
        let lod_selections = self.lod_manager.select_lods(
            &self.hierarchies,
            camera,
            &culling_results,
            &quality_metrics,
        )?;

        // Update instance buffers
        self.buffer_manager.update_instances(device, queue, &lod_selections)?;

        Ok(RenderStats {
            visible_clusters: culling_results.total_visible_clusters(),
            visible_triangles: culling_results.total_visible_triangles(),
            culled_clusters: culling_results.total_culled_clusters(),
            average_lod: lod_selections.average_lod(),
            frame_time_ms: delta_time * 1000.0,
            gpu_memory_mb: self.buffer_manager.memory_usage_mb(),
        })
    }

    /// Get reference to LOD manager
    pub fn lod_manager(&self) -> &Arc<LODManager> {
        &self.lod_manager
    }

    /// Get reference to culling system
    pub fn culling_system(&self) -> &Arc<CullingSystem> {
        &self.culling_system
    }

    /// Get reference to buffer manager
    pub fn buffer_manager(&self) -> &Arc<BufferManager> {
        &self.buffer_manager
    }

    /// Get reference to quality controller
    pub fn quality_controller(&self) -> &Arc<QualityController> {
        &self.quality_controller
    }

    /// Get cluster hierarchy for a mesh
    pub fn hierarchy(&self, mesh_id: usize) -> Option<&ClusterHierarchy> {
        self.hierarchies.get(mesh_id)
    }

    /// Get number of registered meshes
    pub fn mesh_count(&self) -> usize {
        self.hierarchies.len()
    }
}

/// Simple camera struct for culling calculations
#[derive(Clone, Debug)]
pub struct Camera {
    pub position: Vec3,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub fov_y: f32,
    pub aspect_ratio: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

/// 3D vector
pub type Vec3 = [f32; 3];

/// 4x4 matrix
pub type Mat4 = [[f32; 4]; 4];

/// Errors that can occur in the Nanite system
#[derive(Debug, thiserror::Error)]
pub enum NaniteError {
    #[error("Clustering error: {0}")]
    Clustering(#[from] ClusterError),

    #[error("LOD error: {0}")]
    LOD(#[from] LODError),

    #[error("Culling error: {0}")]
    Culling(String),

    #[error("Buffer error: {0}")]
    Buffer(#[from] BufferError),

    #[error("Render error: {0}")]
    Render(String),

    #[error("Invalid mesh ID: {0}")]
    InvalidMeshId(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nanite_config_default() {
        let config = NaniteConfig::default();
        assert_eq!(config.max_triangles_per_cluster, 128);
        assert_eq!(config.max_lod_depth, 8);
    }
}
