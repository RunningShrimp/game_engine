//! # LOD (Level of Detail) Generator
//!
//! This module provides automatic LOD generation for meshes using the
//! Quadric Error Metrics simplification algorithm.
//!
//! ## Overview
//!
//! The LODGenerator creates multiple detail levels from a high-quality mesh,
//! allowing the engine to use lower-detail versions for distant objects.
//!
//! ## Usage
//!
//! ```rust
//! use game_engine::render::lod_generator::{LODGenerator, LODConfig};
//!
//! let config = LODConfig {
//!     levels: vec![1.0, 0.5, 0.25, 0.125], // LOD levels (100%, 50%, 25%, 12.5%)
//!     ..Default::default()
//! };
//!
//! let generator = LODGenerator::new(config);
//! let lods = generator.generate_from_mesh(&high_quality_mesh)?;
//! ```
//!
//! ## LOD Levels
//!
//! - **LOD0**: Full quality (100% triangles)
//! - **LOD1**: Half detail (50% triangles)
//! - **LOD2**: Quarter detail (25% triangles)
//! - **LOD3**: Minimum detail (12.5% triangles)

use crate::render::mesh_simplifier::{Mesh, MeshSimplifier, SimplificationError, SimplifyOptions};
use std::fmt;

/// Errors that can occur during LOD generation
#[derive(Debug, thiserror::Error)]
pub enum LODGeneratorError {
    #[error("Simplification error at LOD level {level}: {source}")]
    SimplificationError {
        level: usize,
        source: SimplificationError,
    },

    #[error("Invalid LOD config: {0}")]
    InvalidConfig(String),

    #[error("Empty mesh - cannot generate LODs")]
    EmptyMesh,

    #[error("Too many LOD levels: {count} (max {max})")]
    TooManyLevels { count: usize, max: usize },
}

/// Configuration for LOD generation
#[derive(Clone, Debug)]
pub struct LODConfig {
    /// LOD level ratios (0.0 - 1.0)
    /// E.g., vec![1.0, 0.5, 0.25, 0.125] creates 4 LOD levels
    pub levels: Vec<f32>,

    /// Minimum triangles for the lowest LOD
    pub min_triangles: usize,

    /// Preserve boundary edges during simplification
    pub preserve_boundaries: bool,

    /// Preserve UV seams during simplification
    pub preserve_uv_seams: bool,

    /// Maximum allowed error for simplification
    pub max_error: f64,

    /// Auto-generate optimal LOD levels
    pub auto_generate: bool,

    /// Number of auto-generated levels (if auto_generate = true)
    pub auto_levels: usize,
}

impl Default for LODConfig {
    fn default() -> Self {
        Self {
            levels: vec![1.0, 0.5, 0.25, 0.125],
            min_triangles: 100,
            preserve_boundaries: true,
            preserve_uv_seams: true,
            max_error: 1.0,
            auto_generate: false,
            auto_levels: 4,
        }
    }
}

impl LODConfig {
    /// Create LOD config with custom levels
    pub fn with_levels(levels: Vec<f32>) -> Self {
        Self {
            levels,
            ..Default::default()
        }
    }

    /// Create LOD config with auto-generated levels
    pub fn auto_generate(num_levels: usize) -> Self {
        Self {
            auto_generate: true,
            auto_levels: num_levels,
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), LODGeneratorError> {
        if self.auto_generate {
            if self.auto_levels == 0 || self.auto_levels > 8 {
                return Err(LODGeneratorError::InvalidConfig(
                    "auto_levels must be between 1 and 8".to_string(),
                ));
            }
            return Ok(());
        }

        if self.levels.is_empty() {
            return Err(LODGeneratorError::InvalidConfig(
                "Must have at least one LOD level".to_string(),
            ));
        }

        if self.levels.len() > 8 {
            return Err(LODGeneratorError::TooManyLevels {
                count: self.levels.len(),
                max: 8,
            });
        }

        // Check levels are in descending order
        for i in 1..self.levels.len() {
            if self.levels[i] >= self.levels[i - 1] {
                return Err(LODGeneratorError::InvalidConfig(
                    "LOD levels must be in descending order".to_string(),
                ));
            }

            if self.levels[i] <= 0.0 || self.levels[i] > 1.0 {
                return Err(LODGeneratorError::InvalidConfig(
                    "LOD levels must be between 0.0 and 1.0".to_string(),
                ));
            }
        }

        // First level should always be 1.0 (full quality)
        if (self.levels[0] - 1.0).abs() > 0.001 {
            return Err(LODGeneratorError::InvalidConfig(
                "First LOD level must be 1.0 (full quality)".to_string(),
            ));
        }

        Ok(())
    }
}

/// A single LOD level
#[derive(Clone, Debug)]
pub struct LODLevel {
    /// Level index (0 = highest quality)
    pub index: usize,

    /// Ratio of original triangles (0.0 - 1.0)
    pub ratio: f32,

    /// Screen size transition (when to switch to this LOD)
    /// 0.0 = never use, 1.0 = always use
    pub screen_size: f32,

    /// The simplified mesh
    pub mesh: Mesh,

    /// Triangle count
    pub triangle_count: usize,

    /// Approximate error metric
    pub error: f64,
}

impl LODLevel {
    /// Create a new LOD level
    pub fn new(index: usize, ratio: f32, mesh: Mesh, error: f64) -> Self {
        let triangle_count = mesh.triangle_count();

        // Calculate screen size based on index
        // LOD0: 1.0, LOD1: 0.5, LOD2: 0.25, etc.
        let screen_size = 1.0 / (1 << index) as f32;

        Self {
            index,
            ratio,
            screen_size,
            mesh,
            triangle_count,
            error,
        }
    }

    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        let vertex_size = std::mem::size_of::<[f32; 3]>() * self.mesh.vertex_count();
        let index_size = std::mem::size_of::<usize>() * self.mesh.indices.len();
        vertex_size + index_size
    }
}

/// Generated LOD levels for a mesh
#[derive(Clone, Debug)]
pub struct LODGroup {
    /// All LOD levels
    pub levels: Vec<LODLevel>,

    /// Original mesh (LOD0 - full quality)
    pub original: Mesh,
}

impl LODGroup {
    /// Create a new LOD group
    pub fn new(levels: Vec<LODLevel>, original: Mesh) -> Self {
        Self { levels, original }
    }

    /// Get LOD level by index
    pub fn get_level(&self, index: usize) -> Option<&LODLevel> {
        self.levels.get(index)
    }

    /// Select best LOD level based on screen size
    pub fn select_level(&self, screen_size: f32) -> &LODLevel {
        for level in &self.levels {
            if screen_size >= level.screen_size {
                return level;
            }
        }
        // Fallback to lowest detail
        self.levels.last().unwrap_or(&self.levels[0])
    }

    /// Get total memory usage of all LODs
    pub fn total_memory_usage(&self) -> usize {
        self.levels.iter().map(|l| l.memory_usage()).sum()
    }

    /// Get number of LOD levels
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }
}

/// LOD Generator - creates multiple detail levels from a mesh
pub struct LODGenerator {
    config: LODConfig,
}

impl LODGenerator {
    /// Create a new LOD generator with default config
    pub fn new() -> Self {
        Self {
            config: LODConfig::default(),
        }
    }

    /// Create a new LOD generator with custom config
    pub fn with_config(config: LODConfig) -> Result<Self, LODGeneratorError> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Generate LODs from a mesh
    pub fn generate_from_mesh(&self, mesh: &Mesh) -> Result<LODGroup, LODGeneratorError> {
        if mesh.triangle_count() == 0 {
            return Err(LODGeneratorError::EmptyMesh);
        }

        let levels_to_generate = if self.config.auto_generate {
            self.auto_generate_levels(mesh.triangle_count())
        } else {
            self.config.levels.clone()
        };

        let mut lod_levels = Vec::new();

        // LOD0 is always the original mesh
        lod_levels.push(LODLevel::new(
            0,
            1.0,
            mesh.clone(),
            0.0, // No error at full quality
        ));

        // Generate subsequent LOD levels
        for (idx, &ratio) in levels_to_generate.iter().skip(1).enumerate() {
            let lod_index = idx + 1;

            let options = SimplifyOptions {
                target_ratio: ratio,
                preserve_boundaries: self.config.preserve_boundaries,
                preserve_uv_seams: self.config.preserve_uv_seams,
                min_triangles: self.config.min_triangles,
                max_error: self.config.max_error,
            };

            match self.simplify_mesh(mesh, &options) {
                Ok((simplified_mesh, error)) => {
                    lod_levels.push(LODLevel::new(lod_index, ratio, simplified_mesh, error));
                }
                Err(e) => {
                    // If simplification fails, duplicate the previous level
                    let prev_level = lod_levels.last().unwrap();
                    lod_levels.push(LODLevel::new(
                        lod_index,
                        ratio,
                        prev_level.mesh.clone(),
                        prev_level.error,
                    ));
                }
            }
        }

        Ok(LODGroup::new(lod_levels, mesh.clone()))
    }

    /// Simplify a mesh with given options
    fn simplify_mesh(
        &self,
        mesh: &Mesh,
        options: &SimplifyOptions,
    ) -> Result<(Mesh, f64), SimplificationError> {
        let mut simplifier = MeshSimplifier::new(mesh.clone())?;
        let simplified = simplifier.simplify(options)?;

        // Calculate approximate error
        let error = self.calculate_error(mesh, &simplified);

        Ok((simplified, error))
    }

    /// Calculate error between original and simplified mesh
    fn calculate_error(&self, original: &Mesh, simplified: &Mesh) -> f64 {
        // Simple metric: vertex count difference
        let vertex_ratio = simplified.vertex_count() as f64 / original.vertex_count() as f64;

        // Error increases as ratio decreases
        1.0 - vertex_ratio
    }

    /// Auto-generate optimal LOD levels
    fn auto_generate_levels(&self, triangle_count: usize) -> Vec<f32> {
        let num_levels = self.config.auto_levels;

        // Generate levels using exponential decay
        // LOD0: 1.0, LOD1: 0.5, LOD2: 0.25, LOD3: 0.125, etc.
        (0..num_levels).map(|i| 1.0 / (2_f32).powi(i as i32)).collect()
    }

    /// Calculate triangle budget for each LOD level
    fn calculate_triangle_budgets(&self, original_count: usize) -> Vec<usize> {
        let levels = if self.config.auto_generate {
            self.auto_generate_levels(original_count)
        } else {
            self.config.levels.clone()
        };

        levels
            .iter()
            .map(|&ratio| ((original_count as f32 * ratio) as usize).max(self.config.min_triangles))
            .collect()
    }
}

impl Default for LODGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick LOD generation with default settings
pub fn generate_lods(mesh: &Mesh) -> Result<LODGroup, LODGeneratorError> {
    let generator = LODGenerator::new();
    generator.generate_from_mesh(mesh)
}

/// Quick LOD generation with custom levels
pub fn generate_lods_with_levels(
    mesh: &Mesh,
    levels: Vec<f32>,
) -> Result<LODGroup, LODGeneratorError> {
    let config = LODConfig::with_levels(levels);
    let generator = LODGenerator::with_config(config)?;
    generator.generate_from_mesh(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_mesh() -> Mesh {
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
        ];

        let indices = vec![
            // Front face
            0, 1, 2, 1, 3, 2, // Back face
            4, 5, 6, 5, 7, 6, // Top face
            2, 3, 6, 3, 7, 6, // Bottom face
            0, 1, 4, 1, 5, 4, // Right face
            1, 5, 3, 5, 7, 3, // Left face
            0, 4, 2, 4, 6, 2,
        ];

        Mesh::from_vertices_and_indices(vertices, indices).unwrap()
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let config = LODConfig::default();
        assert!(config.validate().is_ok());

        // Empty levels
        let config = LODConfig {
            levels: vec![],
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid order
        let config = LODConfig {
            levels: vec![1.0, 0.7, 0.9], // Not descending
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_auto_generate_levels() {
        let config = LODConfig::auto_generate(4);
        assert!(config.validate().is_ok());

        let generator = LODGenerator::with_config(config).unwrap();
        let levels = generator.auto_generate_levels(1000);

        assert_eq!(levels.len(), 4);
        assert_eq!(levels[0], 1.0);
        assert_eq!(levels[1], 0.5);
        assert_eq!(levels[2], 0.25);
        assert_eq!(levels[3], 0.125);
    }

    #[test]
    fn test_lod_generation() {
        let mesh = create_test_mesh();
        let generator = LODGenerator::new();

        let result = generator.generate_from_mesh(&mesh);
        assert!(result.is_ok());

        let lods = result.unwrap();
        assert_eq!(lods.level_count(), 4); // Default 4 levels

        // LOD0 should be full quality
        let lod0 = lods.get_level(0).unwrap();
        assert_eq!(lod0.ratio, 1.0);
        assert_eq!(lod0.triangle_count, mesh.triangle_count());
    }

    #[test]
    fn test_lod_selection() {
        let mesh = create_test_mesh();
        let lods = generate_lods(&mesh).unwrap();

        // Screen size 1.0 should select LOD0
        let selected = lods.select_level(1.0);
        assert_eq!(selected.index, 0);

        // Screen size 0.3 should select LOD1
        let selected = lods.select_level(0.3);
        assert_eq!(selected.index, 1);

        // Screen size 0.1 should select LOD2 or LOD3
        let selected = lods.select_level(0.1);
        assert!(selected.index >= 2);
    }

    #[test]
    fn test_memory_usage() {
        let mesh = create_test_mesh();
        let lods = generate_lods(&mesh).unwrap();

        let total_memory = lods.total_memory_usage();
        assert!(total_memory > 0);

        // Each LOD should have some memory usage
        for level in &lods.levels {
            assert!(level.memory_usage() > 0);
        }
    }
}
