//  Test Helper Types for Render Batch Tests
//
//  This module provides simplified types used in testing that may not exist
//  in the main rendering API yet. These are meant for testing purposes only.

/// DrawCall - Simplified representation for testing
///
/// NOTE: This is a test helper type. The actual rendering implementation
/// may use different structures. This exists to make tests compile.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawCall {
    /// Pipeline ID
    pub pipeline_id: u64,
    /// Vertex buffer ID
    pub vertex_buffer_id: u64,
    /// Index buffer ID (optional)
    pub index_buffer_id: Option<u64>,
    /// Vertex count
    pub vertex_count: u32,
    /// Index count
    pub index_count: u32,
    /// Instance count
    pub instance_count: u32,
    /// Material ID
    pub material_id: u64,
}

/// RenderBatch - Collection of draw calls for testing
#[derive(Debug, Clone, PartialEq)]
pub struct RenderBatch {
    /// Pipeline ID for this batch
    pub pipeline_id: u64,
    /// Draw calls in this batch
    pub draw_calls: Vec<DrawCall>,
}

/// OptimizationStrategy for batch optimizer tests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum OptimizationStrategy {
    /// No optimization
    None,
    /// Merge by pipeline
    #[default]
    MergeByPipeline,
    /// Merge by material
    MergeByMaterial,
    /// Merge by mesh
    MergeByMesh,
}

