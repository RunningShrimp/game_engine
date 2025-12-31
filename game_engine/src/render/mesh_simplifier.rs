//! # Mesh Simplification using Quadric Error Metrics
//!
//! This module implements mesh simplification using the Quadric Error Metrics (QEM) algorithm
//! as described by Garland and Heckbert (1997).
//!
//! ## Algorithm Overview
//!
//! 1. Initialize quadric error matrices for each vertex based on adjacent triangle planes
//! 2. Calculate initial edge collapse costs
//! 3. Iteratively collapse edges with minimum cost
//! 4. Update quadrics and affected edge costs after each collapse
//!
//! ## Usage
//!
//! ```rust
//! use game_engine::render::mesh_simplifier::{MeshSimplifier, SimplifyOptions};
//!
//! let mesh = Mesh::from_vertices_and_indices(vertices, indices);
//! let simplifier = MeshSimplifier::new(mesh);
//!
//! let options = SimplifyOptions {
//!     target_ratio: 0.5,  // Reduce to 50% triangles
//!     ..Default::default()
//! };
//!
//! let simplified_mesh = simplifier.simplify(&options);
//! ```

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::f64::consts::PI;

use nalgebra::{Matrix4, Vector3, Vector4};

/// Errors that can occur during mesh simplification
#[derive(Debug, thiserror::Error)]
pub enum SimplificationError {
    #[error("Invalid mesh: {0}")]
    InvalidMesh(String),

    #[error("Target triangle count {target} is below minimum {min}")]
    TargetTooSmall { target: usize, min: usize },

    #[error("Mesh has no triangles")]
    EmptyMesh,

    #[error("Non-manifold geometry detected at vertex {vtx}")]
    NonManifold { vtx: usize },
}

/// Options for mesh simplification
#[derive(Clone, Debug)]
pub struct SimplifyOptions {
    /// Target triangle count ratio (0.0 - 1.0)
    /// E.g., 0.5 means reduce to 50% of original triangles
    pub target_ratio: f32,

    /// Preserve boundary edges (edges with only one adjacent triangle)
    pub preserve_boundaries: bool,

    /// Preserve UV seams (edges where UVs are disconnected)
    pub preserve_uv_seams: bool,

    /// Minimum triangle count limit
    pub min_triangles: usize,

    /// Maximum error tolerance (prevents collapses with too high error)
    pub max_error: f64,
}

impl Default for SimplifyOptions {
    fn default() -> Self {
        Self {
            target_ratio: 0.5,
            preserve_boundaries: true,
            preserve_uv_seams: true,
            min_triangles: 100,
            max_error: 1.0,
        }
    }
}

/// Simple mesh representation for simplification
#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vector3<f32>>,
    pub normals: Option<Vec<Vector3<f32>>>,
    pub uvs: Option<Vec<[f32; 2]>>,
    pub indices: Vec<usize>,
}

impl Mesh {
    pub fn from_vertices_and_indices(
        vertices: Vec<[f32; 3]>,
        indices: Vec<usize>,
    ) -> Result<Self, SimplificationError> {
        if vertices.is_empty() {
            return Err(SimplificationError::InvalidMesh("No vertices".to_string()));
        }
        if indices.len() % 3 != 0 {
            return Err(SimplificationError::InvalidMesh(
                "Indices not divisible by 3".to_string(),
            ));
        }

        let vertices = vertices
            .into_iter()
            .map(|v| Vector3::new(v[0], v[1], v[2]))
            .collect();

        Ok(Self {
            vertices,
            normals: None,
            uvs: None,
            indices,
        })
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

/// Quadric error matrix for a vertex
///
/// Represents the sum of squared distances to all planes adjacent to a vertex.
/// For a plane defined by (a, b, c, d) where ax + by + cz + d = 0,
/// the fundamental error quadric is:
/// ```text
/// | a²  ab  ac  ad |
/// | ab  b²  bc  bd |
/// | ac  bc  c²  cd |
/// | ad  bd  cd  d² |
/// ```
#[derive(Clone, Debug)]
pub struct QuadricError {
    matrix: Matrix4<f64>,
}

impl std::ops::Add for QuadricError {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            matrix: self.matrix + other.matrix,
        }
    }
}

impl QuadricError {
    /// Create a new zero quadric
    pub fn new() -> Self {
        Self {
            matrix: Matrix4::zeros(),
        }
    }

    /// Create quadric from a plane defined by normal and distance
    pub fn from_plane(normal: Vector3<f64>, distance: f64) -> Self {
        let normal = normal.normalize();
        let (a, b, c) = (normal.x, normal.y, normal.z);
        let d = distance;

        let matrix = Matrix4::new(
            a * a, a * b, a * c, a * d, a * b, b * b, b * c, b * d, a * c, b * c, c * c, c * d, a * d,
            b * d, c * d, d * d,
        );

        Self { matrix }
    }

    /// Add another quadric to this one
    pub fn add(&mut self, other: &QuadricError) {
        self.matrix += other.matrix;
    }

    /// Evaluate error at a given point
    pub fn evaluate(&self, point: Vector3<f64>) -> f64 {
        let v = Vector4::new(point.x, point.y, point.z, 1.0);
        let result = self.matrix * v;
        v.dot(&result)
    }

    /// Find optimal point that minimizes the quadric error
    ///
    /// Solves ∇Q(x) = 0, i.e., finds the vertex position that minimizes error
    pub fn optimal_point(&self) -> Option<Vector3<f64>> {
        // Extract upper-left 3x3 matrix and last column
        let m = self.matrix.fixed_view::<3, 3>(0, 0);
        let b = self.matrix.fixed_view::<3, 1>(0, 3);

        // Solve m * x = -b
        match m.try_inverse() {
            Some(inv) => {
                let result = -inv * b;
                Some(Vector3::new(result[0], result[1], result[2]))
            }
            None => None, // Singular matrix - no unique minimum
        }
    }
}

impl Default for QuadricError {
    fn default() -> Self {
        Self::new()
    }
}

/// Edge collapse operation
#[derive(Clone, Debug)]
struct EdgeCollapse {
    v0: usize,
    v1: usize,
    new_vertex: Vector3<f64>,
    cost: f64,
}

impl PartialEq for EdgeCollapse {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for EdgeCollapse {}

impl PartialOrd for EdgeCollapse {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // BinaryHeap is max-heap, reverse for min-heap behavior
        other.cost.partial_cmp(&self.cost)
    }
}

impl Ord for EdgeCollapse {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.partial_cmp(&self.cost).unwrap()
    }
}

/// Adjacency information for a vertex
#[derive(Clone, Debug)]
struct VertexAdjacency {
    adjacent_vertices: Vec<usize>,
    adjacent_triangles: Vec<usize>,
    is_boundary: bool,
}

impl VertexAdjacency {
    fn new() -> Self {
        Self {
            adjacent_vertices: Vec::new(),
            adjacent_triangles: Vec::new(),
            is_boundary: false,
        }
    }
}

/// Mesh simplifier using Quadric Error Metrics
pub struct MeshSimplifier {
    mesh: Mesh,
    quadrics: Vec<QuadricError>,
    adjacency: Vec<VertexAdjacency>,
    valid_vertices: Vec<bool>,
}

impl MeshSimplifier {
    /// Create a new simplifier from a mesh
    pub fn new(mesh: Mesh) -> Result<Self, SimplificationError> {
        if mesh.triangle_count() == 0 {
            return Err(SimplificationError::EmptyMesh);
        }

        let mut simplifier = Self {
            mesh,
            quadrics: Vec::new(),
            adjacency: Vec::new(),
            valid_vertices: Vec::new(),
        };

        simplifier.initialize()?;
        Ok(simplifier)
    }

    /// Initialize quadrics and build adjacency
    fn initialize(&mut self) -> Result<(), SimplificationError> {
        let n_vertices = self.mesh.vertex_count();
        self.quadrics = vec![QuadricError::new(); n_vertices];
        self.adjacency = vec![VertexAdjacency::new(); n_vertices];
        self.valid_vertices = vec![true; n_vertices];

        // Build adjacency and compute initial quadrics
        self.build_adjacency()?;

        Ok(())
    }

    /// Build vertex adjacency from triangle list
    fn build_adjacency(&mut self) -> Result<(), SimplificationError> {
        // Temporary structures to count edge occurrences
        let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();

        // First pass: build triangle adjacency and count edges
        for tri_idx in 0..self.mesh.triangle_count() {
            let i0 = self.mesh.indices[tri_idx * 3];
            let i1 = self.mesh.indices[tri_idx * 3 + 1];
            let i2 = self.mesh.indices[tri_idx * 3 + 2];

            // Record triangle adjacency
            self.adjacency[i0].adjacent_triangles.push(tri_idx);
            self.adjacency[i1].adjacent_triangles.push(tri_idx);
            self.adjacency[i2].adjacent_triangles.push(tri_idx);

            // Count edge occurrences
            for &(v0, v1) in &[(i0, i1), (i1, i2), (i2, i0)] {
                let key = if v0 < v1 { (v0, v1) } else { (v1, v0) };
                *edge_count.entry(key).or_insert(0) += 1;
            }
        }

        // Second pass: identify boundary edges and build vertex adjacency
        for tri_idx in 0..self.mesh.triangle_count() {
            let i0 = self.mesh.indices[tri_idx * 3];
            let i1 = self.mesh.indices[tri_idx * 3 + 1];
            let i2 = self.mesh.indices[tri_idx * 3 + 2];

            // Check each edge
            for (v0, v1) in [(i0, i1), (i1, i2), (i2, i0)] {
                let key = if v0 < v1 { (v0, v1) } else { (v1, v0) };

                // Boundary edge appears only once
                if edge_count.get(&key) == Some(&1) {
                    self.adjacency[v0].is_boundary = true;
                    self.adjacency[v1].is_boundary = true;
                }

                // Add vertex adjacency
                if !self.adjacency[v0].adjacent_vertices.contains(&v1) {
                    self.adjacency[v0].adjacent_vertices.push(v1);
                }
                if !self.adjacency[v1].adjacent_vertices.contains(&v0) {
                    self.adjacency[v1].adjacent_vertices.push(v0);
                }
            }
        }

        // Compute quadrics for each vertex
        for vtx_idx in 0..self.mesh.vertex_count() {
            self.compute_vertex_quadric(vtx_idx);
        }

        Ok(())
    }

    /// Compute quadric error matrix for a vertex
    fn compute_vertex_quadric(&mut self, vtx_idx: usize) {
        let pos = self.mesh.vertices[vtx_idx].cast::<f64>();
        let mut quadric = QuadricError::new();

        // Add quadrics from all adjacent triangles
        for tri_idx in self.adjacency[vtx_idx].adjacent_triangles.clone() {
            let i0 = self.mesh.indices[tri_idx * 3];
            let i1 = self.mesh.indices[tri_idx * 3 + 1];
            let i2 = self.mesh.indices[tri_idx * 3 + 2];

            let v0 = self.mesh.vertices[i0].cast::<f64>();
            let v1 = self.mesh.vertices[i1].cast::<f64>();
            let v2 = self.mesh.vertices[i2].cast::<f64>();

            // Compute plane normal
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let normal = edge1.cross(&edge2);

            if normal.magnitude() < 1e-10 {
                continue; // Degenerate triangle
            }

            let normal = normal.normalize();
            let distance = -normal.dot(&v0);

            quadric.add(&QuadricError::from_plane(normal, distance));
        }

        self.quadrics[vtx_idx] = quadric;
    }

    /// Calculate cost of collapsing an edge
    fn calculate_edge_cost(&self, v0: usize, v1: usize) -> EdgeCollapse {
        // Combine quadrics
        let mut combined = self.quadrics[v0].clone();
        combined.add(&self.quadrics[v1]);

        // Find optimal point
        let new_vertex = combined.optimal_point().unwrap_or_else(|| {
            // Fallback: midpoint
            (self.mesh.vertices[v0].cast() + self.mesh.vertices[v1].cast()) / 2.0
        });

        // Calculate error
        let cost = combined.evaluate(new_vertex);

        EdgeCollapse {
            v0,
            v1,
            new_vertex,
            cost,
        }
    }

    /// Simplify the mesh according to options
    pub fn simplify(&mut self, options: &SimplifyOptions) -> Result<Mesh, SimplificationError> {
        let target_triangles = (self.mesh.triangle_count() as f32 * options.target_ratio) as usize;

        if target_triangles < options.min_triangles {
            return Err(SimplificationError::TargetTooSmall {
                target: target_triangles,
                min: options.min_triangles,
            });
        }

        // Initialize priority queue with all edges
        let mut edge_queue = self.initialize_edge_queue(options);

        // Main simplification loop
        let mut current_triangles = self.mesh.triangle_count();

        while current_triangles > target_triangles {
            // Get next edge to collapse
            let collapse = match self.pop_valid_edge(&mut edge_queue, options) {
                Some(c) => c,
                None => break, // No more valid edges
            };

            if !self.valid_vertices[collapse.v0] || !self.valid_vertices[collapse.v1] {
                continue;
            }

            // Check error tolerance
            if collapse.cost > options.max_error {
                break;
            }

            // Perform edge collapse
            self.collapse_edge(&collapse, options);

            // Update affected edges
            self.update_affected_edges(&collapse, &mut edge_queue, options);

            current_triangles = self.count_valid_triangles();
        }

        // Build simplified mesh
        Ok(self.build_simplified_mesh())
    }

    /// Initialize edge queue with all valid edges
    fn initialize_edge_queue(&self, options: &SimplifyOptions) -> BinaryHeap<EdgeCollapse> {
        let mut queue = BinaryHeap::new();

        for vtx_idx in 0..self.mesh.vertex_count() {
            if !self.valid_vertices[vtx_idx] {
                continue;
            }

            // Skip boundary vertices if preserving boundaries
            if options.preserve_boundaries && self.adjacency[vtx_idx].is_boundary {
                continue;
            }

            // Add all edges from this vertex
            for &adj_vtx in &self.adjacency[vtx_idx].adjacent_vertices {
                if adj_vtx > vtx_idx {
                    // Only add each edge once
                    continue;
                }

                if options.preserve_boundaries
                    && (self.adjacency[adj_vtx].is_boundary
                        || self.adjacency[vtx_idx].is_boundary)
                {
                    continue;
                }

                let collapse = self.calculate_edge_cost(vtx_idx, adj_vtx);
                queue.push(collapse);
            }
        }

        queue
    }

    /// Pop next valid edge from queue
    fn pop_valid_edge(
        &self,
        queue: &mut BinaryHeap<EdgeCollapse>,
        options: &SimplifyOptions,
    ) -> Option<EdgeCollapse> {
        while let Some(collapse) = queue.pop() {
            if !self.valid_vertices[collapse.v0] || !self.valid_vertices[collapse.v1] {
                continue;
            }

            // Check boundary preservation
            if options.preserve_boundaries
                && (self.adjacency[collapse.v0].is_boundary
                    || self.adjacency[collapse.v1].is_boundary)
            {
                continue;
            }

            // Recalculate cost to ensure it's still valid
            let recalculated = self.calculate_edge_cost(collapse.v0, collapse.v1);

            // Accept if cost is close enough (handles floating point errors)
            if (recalculated.cost - collapse.cost).abs() < 1e-6 {
                return Some(recalculated);
            }
        }

        None
    }

    /// Collapse an edge, merging v1 into v0
    fn collapse_edge(&mut self, collapse: &EdgeCollapse, options: &SimplifyOptions) {
        let v0 = collapse.v0;
        let v1 = collapse.v1;

        // Update vertex position
        self.mesh.vertices[v0] = collapse.new_vertex.cast();

        // Merge quadrics
        let q0 = self.quadrics[v0].clone();
        let q1 = self.quadrics[v1].clone();
        self.quadrics[v0] = q0 + q1;

        // Update adjacency: all triangles and edges from v1 now connect to v0
        for &adj_vtx in &self.adjacency[v1].adjacent_vertices.clone() {
            if adj_vtx == v0 {
                continue;
            }

            if !self.adjacency[v0].adjacent_vertices.contains(&adj_vtx) {
                self.adjacency[v0].adjacent_vertices.push(adj_vtx);
            }

            // Update v1's neighbors to point to v0
            if let Some(pos) = self.adjacency[adj_vtx]
                .adjacent_vertices
                .iter()
                .position(|&v| v == v1)
            {
                self.adjacency[adj_vtx].adjacent_vertices[pos] = v0;
            }
        }

        // Mark v1 as invalid
        self.valid_vertices[v1] = false;
    }

    /// Update affected edges after a collapse
    fn update_affected_edges(
        &mut self,
        collapse: &EdgeCollapse,
        queue: &mut BinaryHeap<EdgeCollapse>,
        options: &SimplifyOptions,
    ) {
        let v0 = collapse.v0;

        // Update all edges incident to v0
        for &adj_vtx in &self.adjacency[v0].adjacent_vertices.clone() {
            if !self.valid_vertices[adj_vtx] {
                continue;
            }

            let new_collapse = self.calculate_edge_cost(v0, adj_vtx);
            queue.push(new_collapse);
        }
    }

    /// Count valid (non-degenerate) triangles
    fn count_valid_triangles(&self) -> usize {
        let mut count = 0;

        for tri_idx in 0..self.mesh.triangle_count() {
            let i0 = self.mesh.indices[tri_idx * 3];
            let i1 = self.mesh.indices[tri_idx * 3 + 1];
            let i2 = self.mesh.indices[tri_idx * 3 + 2];

            // Check if all vertices are valid and distinct
            if self.valid_vertices[i0]
                && self.valid_vertices[i1]
                && self.valid_vertices[i2]
                && i0 != i1
                && i1 != i2
                && i0 != i2
            {
                count += 1;
            }
        }

        count
    }

    /// Build the simplified mesh from current state
    fn build_simplified_mesh(&self) -> Mesh {
        // Create vertex remapping
        let mut vertex_map: HashMap<usize, usize> = HashMap::new();
        let mut new_vertices = Vec::new();
        let mut new_normals = self.mesh.normals.as_ref().map(|_| Vec::new());
        let mut new_uvs = self.mesh.uvs.as_ref().map(|_| Vec::new());

        for (old_idx, _) in self.vertices().iter().enumerate() {
            if !self.valid_vertices[old_idx] {
                continue;
            }

            let new_idx = new_vertices.len();
            vertex_map.insert(old_idx, new_idx);

            new_vertices.push(self.mesh.vertices[old_idx]);

            if let Some(normals) = &self.mesh.normals {
                new_normals.as_mut().unwrap().push(normals[old_idx]);
            }

            if let Some(uvs) = &self.mesh.uvs {
                new_uvs.as_mut().unwrap().push(uvs[old_idx]);
            }
        }

        // Remap indices
        let mut new_indices = Vec::new();
        for tri_idx in 0..self.mesh.triangle_count() {
            let i0 = self.mesh.indices[tri_idx * 3];
            let i1 = self.mesh.indices[tri_idx * 3 + 1];
            let i2 = self.mesh.indices[tri_idx * 3 + 2];

            // Skip degenerate or invalid triangles
            if !self.valid_vertices[i0]
                || !self.valid_vertices[i1]
                || !self.valid_vertices[i2]
                || i0 == i1
                || i1 == i2
                || i0 == i2
            {
                continue;
            }

            new_indices.push(vertex_map[&i0]);
            new_indices.push(vertex_map[&i1]);
            new_indices.push(vertex_map[&i2]);
        }

        Mesh {
            vertices: new_vertices,
            normals: new_normals,
            uvs: new_uvs,
            indices: new_indices,
        }
    }

    /// Get reference to mesh
    fn vertices(&self) -> &[Vector3<f32>] {
        &self.mesh.vertices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadric_from_plane() {
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let distance = 0.0;

        let quadric = QuadricError::from_plane(normal, distance);

        // Evaluate at origin (should be 0 for plane y=0)
        let error = quadric.evaluate(Vector3::new(0.0, 0.0, 0.0));
        assert!((error - 0.0).abs() < 1e-10);

        // Evaluate at (0, 1, 0) (should be 1.0)
        let error = quadric.evaluate(Vector3::new(0.0, 1.0, 0.0));
        assert!((error - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_quadric_addition() {
        let q1 = QuadricError::from_plane(Vector3::new(1.0, 0.0, 0.0), 0.0);
        let q2 = QuadricError::from_plane(Vector3::new(0.0, 1.0, 0.0), 0.0);

        let mut combined = q1.clone();
        combined.add(&q2);

        // Evaluate at (1, 1, 0) - should have error from both planes
        let error = combined.evaluate(Vector3::new(1.0, 1.0, 0.0));
        assert!((error - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_mesh_creation() {
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![0, 1, 2];

        let mesh = Mesh::from_vertices_and_indices(vertices, indices).unwrap();
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.triangle_count(), 1);
    }

    #[test]
    fn test_simplifier_creation() {
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let indices = vec![0, 1, 2, 1, 3, 2];

        let mesh = Mesh::from_vertices_and_indices(vertices, indices).unwrap();
        let simplifier = MeshSimplifier::new(mesh);

        assert!(simplifier.is_ok());
    }
}
