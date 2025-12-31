//! # Automatic Quality Assessment for LOD Generation
//!
//! This module provides quality assessment and automatic LOD selection
//! based on mesh complexity and visual fidelity requirements.
//!
//! ## Features
//!
//! - Automatic mesh complexity analysis
//! - Quality metrics calculation
//! - Optimal LOD level selection
//! - Visual fidelity prediction

use crate::render::lod_generator::{LODGroup, LODLevel};
use std::fmt;

/// Quality assessment result
#[derive(Clone, Debug)]
pub struct QualityAssessment {
    /// Overall quality score (0.0 - 1.0)
    pub overall_score: f32,

    /// Geometric complexity score
    pub geometric_score: f32,

    /// Topology quality score
    pub topology_score: f32,

    /// Visual fidelity score (predicted)
    pub visual_fidelity: f32,

    /// Recommended LOD level
    pub recommended_lod: usize,

    /// Reason for recommendation
    pub reason: String,
}

/// Mesh complexity metrics
#[derive(Clone, Debug)]
pub struct ComplexityMetrics {
    /// Triangle count
    pub triangle_count: usize,

    /// Vertex count
    pub vertex_count: usize,

    /// Triangle-to-vertex ratio
    pub tv_ratio: f32,

    /// Estimated screen coverage at which LOD is needed
    pub lod_threshold: f32,

    /// Mesh complexity category
    pub complexity: MeshComplexity,
}

/// Mesh complexity category
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshComplexity {
    /// Very simple mesh (< 100 triangles)
    VeryLow,

    /// Low complexity (100 - 1,000 triangles)
    Low,

    /// Medium complexity (1,000 - 10,000 triangles)
    Medium,

    /// High complexity (10,000 - 100,000 triangles)
    High,

    /// Very high complexity (> 100,000 triangles)
    VeryHigh,
}

impl MeshComplexity {
    /// Determine complexity category from triangle count
    pub fn from_triangle_count(count: usize) -> Self {
        if count < 100 {
            Self::VeryLow
        } else if count < 1_000 {
            Self::Low
        } else if count < 10_000 {
            Self::Medium
        } else if count < 100_000 {
            Self::High
        } else {
            Self::VeryHigh
        }
    }

    /// Get recommended number of LOD levels
    pub fn recommended_lod_levels(&self) -> usize {
        match self {
            Self::VeryLow => 1,  // Only LOD0 needed
            Self::Low => 2,      // LOD0, LOD1
            Self::Medium => 3,   // LOD0, LOD1, LOD2
            Self::High => 4,     // LOD0, LOD1, LOD2, LOD3
            Self::VeryHigh => 5, // LOD0-LOD4
        }
    }

    /// Get LOD level ratios for this complexity
    pub fn recommended_ratios(&self) -> Vec<f32> {
        match self {
            Self::VeryLow => vec![1.0],
            Self::Low => vec![1.0, 0.5],
            Self::Medium => vec![1.0, 0.5, 0.25],
            Self::High => vec![1.0, 0.5, 0.25, 0.125],
            Self::VeryHigh => vec![1.0, 0.5, 0.25, 0.125, 0.0625],
        }
    }
}

/// Quality assessment configuration
#[derive(Clone, Debug)]
pub struct QualityConfig {
    /// Target quality level
    pub target_quality: TargetQuality,

    /// Platform constraints
    pub platform: PlatformConstraints,

    /// Performance requirements
    pub performance: PerformanceRequirements,
}

/// Target quality level
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetQuality {
    /// Maximum quality (PC high-end)
    Ultra,

    /// High quality (PC mid-range)
    High,

    /// Medium quality (PC low-end / mobile high-end)
    Medium,

    /// Low quality (mobile mid-range)
    Low,

    /// Minimum quality (mobile low-end)
    Minimum,
}

impl TargetQuality {
    /// Get minimum triangle ratio for this quality level
    pub fn min_triangle_ratio(&self) -> f32 {
        match self {
            Self::Ultra => 0.25,     // Keep at least 25%
            Self::High => 0.125,     // Keep at least 12.5%
            Self::Medium => 0.0625,  // Keep at least 6.25%
            Self::Low => 0.03125,    // Keep at least 3.125%
            Self::Minimum => 0.0156, // Keep at least 1.5625%
        }
    }

    /// Get max error tolerance for this quality level
    pub fn max_error_tolerance(&self) -> f64 {
        match self {
            Self::Ultra => 0.01,  // Very low error tolerance
            Self::High => 0.05,   // Low error tolerance
            Self::Medium => 0.1,  // Medium error tolerance
            Self::Low => 0.25,    // High error tolerance
            Self::Minimum => 0.5, // Very high error tolerance
        }
    }
}

/// Platform constraints
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformConstraints {
    /// Desktop PC (no strict memory/size constraints)
    Desktop,

    /// Mobile device (memory constraints)
    Mobile,

    /// Web browser (download size constraints)
    Web,

    /// Console (balanced constraints)
    Console,
}

impl PlatformConstraints {
    /// Get minimum triangle count for this platform
    pub fn min_triangles(&self) -> usize {
        match self {
            Self::Desktop => 100,
            Self::Mobile => 50,
            Self::Web => 30,
            Self::Console => 80,
        }
    }

    /// Get recommended LOD bias (prefer lower/higher detail)
    pub fn lod_bias(&self) -> f32 {
        match self {
            Self::Desktop => 0.0, // No bias
            Self::Mobile => -0.5, // Prefer lower detail
            Self::Web => -1.0,    // Strongly prefer lower detail
            Self::Console => 0.0, // No bias
        }
    }
}

/// Performance requirements
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerformanceRequirements {
    /// 60+ FPS target
    High,

    /// 30-60 FPS target
    Medium,

    /// 30 FPS target
    Low,
}

impl PerformanceRequirements {
    /// Get triangle budget multiplier
    pub fn triangle_multiplier(&self) -> f32 {
        match self {
            Self::High => 0.7,   // Reduce triangles by 30%
            Self::Medium => 1.0, // Keep all triangles
            Self::Low => 1.5,    // Allow 50% more triangles
        }
    }
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            target_quality: TargetQuality::High,
            platform: PlatformConstraints::Desktop,
            performance: PerformanceRequirements::Medium,
        }
    }
}

/// Automatic quality assessor
pub struct QualityAssessor {
    config: QualityConfig,
}

impl QualityAssessor {
    /// Create a new quality assessor with default config
    pub fn new() -> Self {
        Self {
            config: QualityConfig::default(),
        }
    }

    /// Create a new quality assessor with custom config
    pub fn with_config(config: QualityConfig) -> Self {
        Self { config }
    }

    /// Assess quality of a mesh's LODs
    pub fn assess_lods(&self, lods: &LODGroup) -> QualityAssessment {
        let metrics = self.calculate_complexity(lods);

        let geometric_score = self.calculate_geometric_score(&metrics);
        let topology_score = self.calculate_topology_score(&metrics);
        let visual_fidelity = self.predict_visual_fidelity(&metrics, lods);
        let overall_score = (geometric_score + topology_score + visual_fidelity) / 3.0;

        let recommended_lod = self.recommend_lod_level(&metrics, lods);
        let reason = self.explain_recommendation(&metrics, recommended_lod);

        QualityAssessment {
            overall_score,
            geometric_score,
            topology_score,
            visual_fidelity,
            recommended_lod,
            reason,
        }
    }

    /// Calculate complexity metrics from LOD group
    fn calculate_complexity(&self, lods: &LODGroup) -> ComplexityMetrics {
        let original = &lods.original;
        let triangle_count = original.triangle_count();
        let vertex_count = original.vertex_count();
        let tv_ratio = vertex_count as f32 / triangle_count.max(1) as f32;

        let complexity = MeshComplexity::from_triangle_count(triangle_count);

        // Calculate LOD threshold based on complexity
        // More complex meshes need LODs at larger screen sizes
        let lod_threshold = match complexity {
            MeshComplexity::VeryLow => 0.1,
            MeshComplexity::Low => 0.2,
            MeshComplexity::Medium => 0.3,
            MeshComplexity::High => 0.5,
            MeshComplexity::VeryHigh => 0.7,
        };

        ComplexityMetrics {
            triangle_count,
            vertex_count,
            tv_ratio,
            lod_threshold,
            complexity,
        }
    }

    /// Calculate geometric quality score
    fn calculate_geometric_score(&self, metrics: &ComplexityMetrics) -> f32 {
        // Score based on triangle count and T/V ratio
        let triangle_score = match metrics.complexity {
            MeshComplexity::VeryLow => 1.0,
            MeshComplexity::Low => 0.9,
            MeshComplexity::Medium => 0.7,
            MeshComplexity::High => 0.5,
            MeshComplexity::VeryHigh => 0.3,
        };

        // T/V ratio should be close to 2.0 for closed meshes
        let tv_score = 1.0 - (metrics.tv_ratio - 2.0).abs() / 2.0;
        let tv_score = tv_score.max(0.0).min(1.0);

        (triangle_score + tv_score) / 2.0
    }

    /// Calculate topology quality score
    fn calculate_topology_score(&self, metrics: &ComplexityMetrics) -> f32 {
        // Simple heuristic: lower T/V ratio indicates better topology
        // (fewer vertices per triangle = more shared vertices = better)
        let optimal_ratio = 1.5; // Approximately optimal for typical meshes
        let deviation = (metrics.tv_ratio - optimal_ratio).abs();

        let score = 1.0 - (deviation / 2.0);
        score.max(0.0).min(1.0)
    }

    /// Predict visual fidelity at given LOD level
    fn predict_visual_fidelity(&self, metrics: &ComplexityMetrics, lods: &LODGroup) -> f32 {
        // Get LOD0 and lowest LOD
        let lod0 = lods.get_level(0).unwrap();
        let lowest_lod = lods.get_level(lods.level_count() - 1).unwrap();

        // Calculate reduction factor
        let reduction = lowest_lod.triangle_count as f32 / lod0.triangle_count as f32;

        // Adjust for platform constraints
        let platform_factor = match self.config.platform {
            PlatformConstraints::Desktop => 1.0,
            PlatformConstraints::Mobile => 0.8,
            PlatformConstraints::Web => 0.7,
            PlatformConstraints::Console => 0.9,
        };

        // Visual fidelity decreases with triangle reduction
        let fidelity = reduction * platform_factor;

        fidelity.max(0.0).min(1.0)
    }

    /// Recommend optimal LOD level
    fn recommend_lod_level(&self, metrics: &ComplexityMetrics, lods: &LODGroup) -> usize {
        let recommended_levels = metrics.complexity.recommended_lod_levels();
        let platform_bias = self.config.platform.lod_bias();

        // Find best LOD level based on complexity and platform
        let mut best_level = 0;
        let mut best_score = 0.0;

        for (idx, level) in lods.levels.iter().enumerate() {
            // Calculate score for this LOD level
            let score = level.ratio + platform_bias;

            // Apply minimum quality constraint
            let min_ratio = self.config.target_quality.min_triangle_ratio();
            if level.ratio < min_ratio {
                break; // Don't use levels below minimum quality
            }

            if score > best_score {
                best_score = score;
                best_level = idx;
            }
        }

        best_level.min(recommended_levels - 1)
    }

    /// Explain LOD recommendation
    fn explain_recommendation(&self, metrics: &ComplexityMetrics, level: usize) -> String {
        let complexity_str = match metrics.complexity {
            MeshComplexity::VeryLow => "very low",
            MeshComplexity::Low => "low",
            MeshComplexity::Medium => "medium",
            MeshComplexity::High => "high",
            MeshComplexity::VeryHigh => "very high",
        };

        format!(
            "Mesh has {} complexity ({} triangles). Recommended LOD{} based on {} quality target and {} platform.",
            complexity_str,
            metrics.triangle_count,
            level,
            format!("{:?}", self.config.target_quality).to_lowercase(),
            format!("{:?}", self.config.platform).to_lowercase()
        )
    }

    /// Auto-generate optimal LOD config for a mesh
    pub fn generate_optimal_config(&self, triangle_count: usize) -> (Vec<f32>, f64, usize) {
        let complexity = MeshComplexity::from_triangle_count(triangle_count);
        let ratios = complexity.recommended_ratios();
        let max_error = self.config.target_quality.max_error_tolerance();
        let min_triangles = self.config.platform.min_triangles();

        (ratios, max_error, min_triangles)
    }
}

impl Default for QualityAssessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick quality assessment with default settings
pub fn assess_quality(lods: &LODGroup) -> QualityAssessment {
    let assessor = QualityAssessor::new();
    assessor.assess_lods(lods)
}

/// Quick complexity analysis
pub fn analyze_complexity(triangle_count: usize) -> ComplexityMetrics {
    let complexity = MeshComplexity::from_triangle_count(triangle_count);
    let tv_ratio = 2.0; // Assume optimal for analysis without full mesh

    let lod_threshold = match complexity {
        MeshComplexity::VeryLow => 0.1,
        MeshComplexity::Low => 0.2,
        MeshComplexity::Medium => 0.3,
        MeshComplexity::High => 0.5,
        MeshComplexity::VeryHigh => 0.7,
    };

    ComplexityMetrics {
        triangle_count,
        vertex_count: (triangle_count as f32 * tv_ratio) as usize,
        tv_ratio,
        lod_threshold,
        complexity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::lod_generator::generate_lods;
    use crate::render::mesh_simplifier::Mesh;

    fn create_test_mesh() -> Mesh {
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];

        let indices = vec![0, 1, 2, 1, 3, 2];

        Mesh::from_vertices_and_indices(vertices, indices).unwrap()
    }

    #[test]
    fn test_complexity_classification() {
        assert_eq!(
            MeshComplexity::from_triangle_count(50),
            MeshComplexity::VeryLow
        );
        assert_eq!(
            MeshComplexity::from_triangle_count(500),
            MeshComplexity::Low
        );
        assert_eq!(
            MeshComplexity::from_triangle_count(5000),
            MeshComplexity::Medium
        );
        assert_eq!(
            MeshComplexity::from_triangle_count(50000),
            MeshComplexity::High
        );
        assert_eq!(
            MeshComplexity::from_triangle_count(500000),
            MeshComplexity::VeryHigh
        );
    }

    #[test]
    fn test_quality_assessment() {
        let mesh = create_test_mesh();
        let lods = generate_lods(&mesh).unwrap();

        let assessment = assess_quality(&lods);

        assert!(assessment.overall_score >= 0.0 && assessment.overall_score <= 1.0);
        assert!(assessment.geometric_score >= 0.0 && assessment.geometric_score <= 1.0);
        assert!(assessment.topology_score >= 0.0 && assessment.topology_score <= 1.0);
        assert!(assessment.visual_fidelity >= 0.0 && assessment.visual_fidelity <= 1.0);
    }

    #[test]
    fn test_complexity_analysis() {
        let metrics = analyze_complexity(5000);

        assert_eq!(metrics.complexity, MeshComplexity::Medium);
        assert_eq!(metrics.triangle_count, 5000);
        assert!(metrics.lod_threshold > 0.0);
    }

    #[test]
    fn test_optimal_config_generation() {
        let assessor = QualityAssessor::new();

        let (ratios, max_error, min_triangles) = assessor.generate_optimal_config(5000);

        assert!(!ratios.is_empty());
        assert_eq!(ratios[0], 1.0); // First level is always full quality
        assert!(max_error > 0.0);
        assert!(min_triangles > 0);
    }
}
