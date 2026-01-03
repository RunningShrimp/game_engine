//! # Mesh Clustering for Nanite Virtual Geometry
//!
//! This module implements clustering algorithms that divide high-poly meshes into
//! hierarchical clusters of triangles. Clustering is the foundation of Nanite's efficiency.

use std::collections::{HashMap, HashSet};
use crate::render::nanite::{Vec3, Mat4};

/// Configuration for cluster building
#[derive(Clone, Debug)]
pub struct ClusterConfig {
    /// Maximum triangles per cluster
    pub max_triangles_per_cluster: usize,
    /// Maximum hierarchy depth
    pub max_depth: u8,
    /// Error threshold for LOD simplification
    pub error_threshold: f32,
    /// Enable automatic edge preservation
    pub preserve_hard_edges: bool,
    /// Edge angle threshold (in radians)
    pub hard_edge_angle: f32,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            max_triangles_per_cluster: 128,
            max_depth: 8,
            error_threshold: 0.01,
            preserve_hard_edges: true,
            hard_edge_angle: std::f32::consts::PI / 4.0, // 45 degrees
        }
    }
}

/// A single triangle in a mesh
#[derive(Clone, Debug, PartialEq)]
pub struct Triangle {
    /// Vertex indices
    pub indices: [u32; 3],
    /// Triangle normal
    pub normal: Vec3,
    /// Triangle area
    pub area: f32,
    /// Triangle centroid
    pub centroid: Vec3,
    /// Screen space error at this LOD
    pub error: f32,
}

/// A cluster of triangles
#[derive(Clone, Debug)]
pub struct Cluster {
    /// Unique cluster ID
    pub id: u32,
    /// Triangles in this cluster
    pub triangles: Vec<Triangle>,
    /// Cluster LOD level (0 = highest detail)
    pub lod_level: u8,
    /// Cluster bounding box (min, max)
    pub bounds: (Vec3, Vec3),
    /// Cluster bounding sphere center
    pub sphere_center: Vec3,
    /// Cluster bounding sphere radius
    pub sphere_radius: f32,
    /// Parent cluster ID (None for root)
    pub parent: Option<u32>,
    /// Child cluster IDs
    pub children: Vec<u32>,
    /// Visibility importance (0-1, higher = more important)
    pub importance: f32,
    /// Approximate screen space error
    pub screen_space_error: f32,
    /// Cluster instance buffer offset (filled during rendering)
    pub instance_offset: Option<u32>,
}

impl Cluster {
    /// Create a new cluster from triangles
    pub fn new(id: u32, triangles: Vec<Triangle>, lod_level: u8) -> Self {
        let bounds = Self::compute_bounds(&triangles);
        let (sphere_center, sphere_radius) = Self::compute_bounding_sphere(&triangles, &bounds);

        Self {
            id,
            triangles,
            lod_level,
            bounds,
            sphere_center,
            sphere_radius,
            parent: None,
            children: Vec::new(),
            importance: 1.0,
            screen_space_error: 0.0,
            instance_offset: None,
        }
    }

    /// Compute axis-aligned bounding box
    fn compute_bounds(triangles: &[Triangle]) -> (Vec3, Vec3) {
        if triangles.is_empty() {
            return ([0.0; 3], [0.0; 3]);
        }

        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];

        for tri in triangles {
            for vertex_idx in &tri.indices {
                // Note: In real implementation, you'd look up actual vertex positions
                // For now, use centroid as approximation
                for i in 0..3 {
                    min[i] = min[i].min(tri.centroid[i]);
                    max[i] = max[i].max(tri.centroid[i]);
                }
            }
        }

        (min, max)
    }

    /// Compute bounding sphere
    fn compute_bounding_sphere(triangles: &[Triangle], bounds: &(Vec3, Vec3)) -> (Vec3, f32) {
        if triangles.is_empty() {
            return ([0.0; 3], 0.0);
        }

        // Center is average of bounds
        let center = [
            (bounds[0][0] + bounds[1][0]) * 0.5,
            (bounds[0][1] + bounds[1][1]) * 0.5,
            (bounds[0][2] + bounds[1][2]) * 0.5,
        ];

        // Radius is max distance from center to any triangle centroid
        let mut radius_sq = 0.0;
        for tri in triangles {
            let dx = tri.centroid[0] - center[0];
            let dy = tri.centroid[1] - center[1];
            let dz = tri.centroid[2] - center[2];
            let dist_sq = dx * dx + dy * dy + dz * dz;
            radius_sq = radius_sq.max(dist_sq);
        }

        (center, radius_sq.sqrt())
    }

    /// Get triangle count
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// Check if point is inside bounding sphere
    pub fn contains_point(&self, point: &Vec3) -> bool {
        let dx = point[0] - self.sphere_center[0];
        let dy = point[1] - self.sphere_center[1];
        let dz = point[2] - self.sphere_center[2];
        (dx * dx + dy * dy + dz * dz) <= (self.sphere_radius * self.sphere_radius)
    }
}

/// A node in the cluster hierarchy tree
#[derive(Clone, Debug)]
pub struct ClusterNode {
    /// Cluster at this node
    pub cluster: Cluster,
    /// LOD levels for this node (simplified versions of the cluster)
    pub lods: Vec<Vec<Triangle>>,
}

/// Complete cluster hierarchy for a mesh
#[derive(Clone, Debug)]
pub struct ClusterHierarchy {
    /// All nodes in the hierarchy
    pub nodes: Vec<ClusterNode>,
    /// Root node ID
    pub root_id: u32,
    /// Total triangle count in original mesh
    pub total_triangles: usize,
    /// Maximum LOD depth
    pub max_depth: u8,
    /// Bounding box of entire mesh
    pub mesh_bounds: (Vec3, Vec3),
}

impl ClusterHierarchy {
    /// Get a node by ID
    pub fn get_node(&self, id: u32) -> Option<&ClusterNode> {
        self.nodes.get(id as usize)
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: u32) -> Option<&mut ClusterNode> {
        self.nodes.get_mut(id as usize)
    }

    /// Get all leaf nodes
    pub fn get_leaves(&self) -> Vec<&ClusterNode> {
        let mut leaves = Vec::new();
        for node in &self.nodes {
            if node.cluster.children.is_empty() {
                leaves.push(node);
            }
        }
        leaves
    }

    /// Count total clusters
    pub fn cluster_count(&self) -> usize {
        self.nodes.len()
    }
}

/// Builder for creating cluster hierarchies
pub struct ClusterBuilder {
    config: ClusterConfig,
    next_cluster_id: u32,
}

impl ClusterBuilder {
    /// Create a new cluster builder
    pub fn new(config: ClusterConfig) -> Self {
        Self {
            config,
            next_cluster_id: 0,
        }
    }

    /// Build cluster hierarchy from mesh data
    pub fn build_hierarchy(
        &mut self,
        vertices: &[Vec3],
        indices: &[u32],
    ) -> Result<ClusterHierarchy, ClusterError> {
        if indices.len() % 3 != 0 {
            return Err(ClusterError::InvalidTriangleCount);
        }

        // Convert indices to triangles
        let triangles = self.create_triangles(vertices, indices)?;

        if triangles.is_empty() {
            return Err(ClusterError::EmptyMesh);
        }

        // Compute mesh bounds
        let mesh_bounds = self.compute_mesh_bounds(&triangles);

        // Build hierarchy recursively
        let mut nodes = Vec::new();
        let root_id = self.build_cluster_recursive(
            &triangles,
            0,
            &mut nodes,
            &mesh_bounds,
        )?;

        let max_depth = nodes.iter()
            .map(|n| n.cluster.lod_level)
            .max()
            .unwrap_or(0);

        Ok(ClusterHierarchy {
            nodes,
            root_id,
            total_triangles: triangles.len(),
            max_depth,
            mesh_bounds,
        })
    }

    /// Create triangle structures from vertex and index data
    fn create_triangles(
        &self,
        vertices: &[Vec3],
        indices: &[u32],
    ) -> Result<Vec<Triangle>, ClusterError> {
        let mut triangles = Vec::new();

        for chunk in indices.chunks(3) {
            if chunk.len() != 3 {
                continue;
            }

            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            if i0 >= vertices.len() || i1 >= vertices.len() || i2 >= vertices.len() {
                return Err(ClusterError::InvalidVertexIndex);
            }

            let v0 = vertices[i0];
            let v1 = vertices[i1];
            let v2 = vertices[i2];

            // Compute triangle normal
            let edge1 = [
                v1[0] - v0[0],
                v1[1] - v0[1],
                v1[2] - v0[2],
            ];
            let edge2 = [
                v2[0] - v0[0],
                v2[1] - v0[1],
                v2[2] - v0[2],
            ];

            let normal = [
                edge1[1] * edge2[2] - edge1[2] * edge2[1],
                edge1[2] * edge2[0] - edge1[0] * edge2[2],
                edge1[0] * edge2[1] - edge1[1] * edge2[0],
            ];

            let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            let normal = if len > 0.0 {
                [normal[0] / len, normal[1] / len, normal[2] / len]
            } else {
                [0.0, 0.0, 1.0]
            };

            // Compute triangle area
            let area = len * 0.5;

            // Compute centroid
            let centroid = [
                (v0[0] + v1[0] + v2[0]) / 3.0,
                (v0[1] + v1[1] + v2[1]) / 3.0,
                (v0[2] + v1[2] + v2[2]) / 3.0,
            ];

            triangles.push(Triangle {
                indices: [chunk[0], chunk[1], chunk[2]],
                normal,
                area,
                centroid,
                error: 0.0,
            });
        }

        Ok(triangles)
    }

    /// Compute mesh bounding box
    fn compute_mesh_bounds(&self, triangles: &[Triangle]) -> (Vec3, Vec3) {
        if triangles.is_empty() {
            return ([0.0; 3], [0.0; 3]);
        }

        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];

        for tri in triangles {
            for i in 0..3 {
                min[i] = min[i].min(tri.centroid[i]);
                max[i] = max[i].max(tri.centroid[i]);
            }
        }

        (min, max)
    }

    /// Recursively build cluster hierarchy
    fn build_cluster_recursive(
        &mut self,
        triangles: &[Triangle],
        depth: u8,
        nodes: &mut Vec<ClusterNode>,
        mesh_bounds: &(Vec3, Vec3),
    ) -> Result<u32, ClusterError> {
        if depth >= self.config.max_depth || triangles.len() <= self.config.max_triangles_per_cluster {
            // Leaf node - create single cluster
            let cluster_id = self.next_cluster_id;
            self.next_cluster_id += 1;

            let cluster = Cluster::new(cluster_id, triangles.to_vec(), depth);
            let mut lods = Vec::new();

            // Generate LODs for this cluster
            if depth > 0 {
                lods.push(self.simplify_triangles(triangles, 0.5)?);
            }
            if depth > 2 {
                lods.push(self.simplify_triangles(triangles, 0.25)?);
            }

            nodes.push(ClusterNode {
                cluster,
                lods,
            });

            return Ok(cluster_id);
        }

        // Split triangles into clusters based on spatial proximity
        let clusters = self.spatial_clustering(triangles, mesh_bounds)?;

        if clusters.len() == 1 {
            // Couldn't split further, create leaf node
            return self.build_cluster_recursive(triangles, self.config.max_depth, nodes, mesh_bounds);
        }

        // Create parent cluster with all triangles
        let parent_id = self.next_cluster_id;
        self.next_cluster_id += 1;

        let parent_cluster = Cluster::new(parent_id, triangles.to_vec(), depth);
        nodes.push(ClusterNode {
            cluster: parent_cluster,
            lods: Vec::new(),
        });

        // Recursively build children
        let mut child_ids = Vec::new();
        for cluster_triangles in clusters {
            let cluster_bounds = self.compute_mesh_bounds(&cluster_triangles);
            let child_id = self.build_cluster_recursive(
                &cluster_triangles,
                depth + 1,
                nodes,
                &cluster_bounds,
            )?;
            child_ids.push(child_id);

            // Set parent relationship
            if let Some(node) = nodes.get_mut(child_id as usize) {
                node.cluster.parent = Some(parent_id);
            }
        }

        // Update parent's children
        if let Some(node) = nodes.get_mut(parent_id as usize) {
            node.cluster.children = child_ids;
        }

        Ok(parent_id)
    }

    /// Spatial clustering - group nearby triangles
    fn spatial_clustering(
        &self,
        triangles: &[Triangle],
        bounds: &(Vec3, Vec3),
    ) -> Result<Vec<Vec<Triangle>>, ClusterError> {
        if triangles.len() <= self.config.max_triangles_per_cluster {
            return Ok(vec![triangles.to_vec()]);
        }

        // Split along the longest axis
        let size = [
            bounds[1][0] - bounds[0][0],
            bounds[1][1] - bounds[0][1],
            bounds[1][2] - bounds[0][2],
        ];

        let split_axis = if size[0] >= size[1] && size[0] >= size[2] {
            0
        } else if size[1] >= size[2] {
            1
        } else {
            2
        };

        let split_pos = (bounds[0][split_axis] + bounds[1][split_axis]) * 0.5;

        let mut cluster0 = Vec::new();
        let mut cluster1 = Vec::new();

        for tri in triangles {
            if tri.centroid[split_axis] < split_pos {
                cluster0.push(tri.clone());
            } else {
                cluster1.push(tri.clone());
            }
        }

        // If one cluster is empty, try different split
        if cluster0.is_empty() || cluster1.is_empty() {
            // Split by index instead
            let mid = triangles.len() / 2;
            return Ok(vec![
                triangles[..mid].to_vec(),
                triangles[mid..].to_vec(),
            ]);
        }

        Ok(vec![cluster0, cluster1])
    }

    /// Simplify triangles (reduce detail for LOD)
    fn simplify_triangles(
        &self,
        triangles: &[Triangle],
        reduction_factor: f32,
    ) -> Result<Vec<Triangle>, ClusterError> {
        let target_count = (triangles.len() as f32 * reduction_factor).ceil() as usize;
        if target_count >= triangles.len() || target_count < 1 {
            return Ok(triangles.to_vec());
        }

        // Simple approach: sort by area and keep largest triangles
        let mut sorted: Vec<_> = triangles.iter().enumerate().collect();
        sorted.sort_by(|a, b| {
            b.1.area.partial_cmp(&a.1.area).unwrap_or(std::cmp::Ordering::Equal)
        });

        sorted.truncate(target_count);

        let mut result = Vec::new();
        for (_, tri) in sorted {
            let mut tri_clone = tri.clone();
            tri_clone.error = self.config.error_threshold;
            result.push(tri_clone);
        }

        Ok(result)
    }
}

/// Errors that can occur during clustering
#[derive(Debug, thiserror::Error)]
pub enum ClusterError {
    #[error("Invalid triangle count in index buffer")]
    InvalidTriangleCount,

    #[error("Invalid vertex index")]
    InvalidVertexIndex,

    #[error("Empty mesh provided")]
    EmptyMesh,

    #[error("Clustering failed: {0}")]
    ClusteringFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_creation() {
        let triangles = vec![
            Triangle {
                indices: [0, 1, 2],
                normal: [0.0, 0.0, 1.0],
                area: 1.0,
                centroid: [0.0, 0.0, 0.0],
                error: 0.0,
            }
        ];

        let cluster = Cluster::new(0, triangles, 0);
        assert_eq!(cluster.id, 0);
        assert_eq!(cluster.triangle_count(), 1);
    }

    #[test]
    fn test_hierarchy_builder() {
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];

        let indices = vec![0, 1, 2, 1, 3, 2];

        let mut builder = ClusterBuilder::new(ClusterConfig::default());
        let hierarchy = builder.build_hierarchy(&vertices, &indices).unwrap();

        assert!(!hierarchy.nodes.is_empty());
        assert_eq!(hierarchy.total_triangles, 2);
    }

    #[test]
    fn test_triangle_creation() {
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];

        let indices = vec![0, 1, 2];

        let builder = ClusterBuilder::new(ClusterConfig::default());
        let triangles = builder.create_triangles(&vertices, &indices).unwrap();

        assert_eq!(triangles.len(), 1);
        assert!((triangles[0].area - 0.5).abs() < 0.001);
    }
}
