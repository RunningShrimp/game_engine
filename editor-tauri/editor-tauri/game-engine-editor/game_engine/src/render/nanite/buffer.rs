//! # Buffer Management for Nanite
//!
//! Efficient GPU buffer management for instance data and cluster information.

use std::collections::HashMap;
use wgpu::*;
use crate::render::nanite::{ClusterHierarchy, LODSelection};

/// Configuration for buffer management
#[derive(Clone, Debug)]
pub struct BufferConfig {
    /// Instance buffer size in MB
    pub instance_buffer_size_mb: u32,
    /// Enable compute shader acceleration
    pub enable_compute_acceleration: bool,
    /// Buffer alignment requirement
    pub buffer_alignment: u32,
    /// Enable memory defragmentation
    pub enable_defragmentation: bool,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            instance_buffer_size_mb: 256,
            enable_compute_acceleration: true,
            buffer_alignment: 256, // Typical UBO alignment
            enable_defragmentation: true,
        }
    }
}

/// Instance data for a single cluster
#[repr(C)]
#[derive(Clone, Debug, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    /// Model matrix (row 0)
    pub model_matrix_0: [f32; 4],
    /// Model matrix (row 1)
    pub model_matrix_1: [f32; 4],
    /// Model matrix (row 2)
    pub model_matrix_2: [f32; 4],
    /// Model matrix (row 3)
    pub model_matrix_3: [f32; 4],
    /// LOD level
    pub lod_level: u32,
    /// Cluster ID
    pub cluster_id: u32,
    /// Padding
    pub padding: [u32; 2],
}

impl Default for InstanceData {
    fn default() -> Self {
        Self {
            model_matrix_0: [1.0, 0.0, 0.0, 0.0],
            model_matrix_1: [0.0, 1.0, 0.0, 0.0],
            model_matrix_2: [0.0, 0.0, 1.0, 0.0],
            model_matrix_3: [0.0, 0.0, 0.0, 1.0],
            lod_level: 0,
            cluster_id: 0,
            padding: [0, 0],
        }
    }
}

/// Buffer allocation
#[derive(Clone, Debug)]
pub struct BufferAllocation {
    /// Buffer ID
    pub buffer_id: u32,
    /// Offset in bytes
    pub offset: u64,
    /// Size in bytes
    pub size: u64,
    /// Alignment
    pub alignment: u64,
}

/// GPU buffer with metadata
pub struct GPUBuffer {
    /// Underlying wgpu buffer
    pub buffer: Buffer,
    /// Buffer size in bytes
    pub size: u64,
    /// Buffer usage
    pub usage: BufferUsages,
    /// Allocated regions
    allocations: Vec<BufferAllocation>,
    /// Free regions
    free_regions: Vec<(u64, u64)>, // (offset, size)
}

impl GPUBuffer {
    /// Create new GPU buffer
    pub fn new(device: &Device, size: u64, usage: BufferUsages, label: &str) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            size,
            usage,
            allocations: Vec::new(),
            free_regions: vec![(0, size)],
        }
    }

    /// Allocate region in buffer
    pub fn allocate(&mut self, size: u64, alignment: u64) -> Option<BufferAllocation> {
        // Align size
        let aligned_size = ((size + alignment - 1) / alignment) * alignment;

        // Find suitable free region
        for i in 0..self.free_regions.len() {
            let (offset, region_size) = self.free_regions[i];

            // Check alignment
            let aligned_offset = ((offset + alignment - 1) / alignment) * alignment;
            let padding = aligned_offset - offset;

            if region_size >= aligned_size + padding {
                // Allocate from this region
                let allocation = BufferAllocation {
                    buffer_id: 0,
                    offset: aligned_offset,
                    size: aligned_size,
                    alignment,
                };

                // Update free regions
                let remaining_start = aligned_offset + aligned_size;
                let remaining_size = region_size - (aligned_offset - offset) - aligned_size;

                self.free_regions.remove(i);

                if remaining_size > alignment {
                    self.free_regions.push((remaining_start, remaining_size));
                }

                self.allocations.push(allocation.clone());
                return Some(allocation);
            }
        }

        None // No suitable region found
    }

    /// Free allocation
    pub fn free(&mut self, allocation: &BufferAllocation) {
        // Find and remove allocation
        if let Some(pos) = self.allocations.iter().position(|a| a.offset == allocation.offset) {
            self.allocations.remove(pos);

            // Add to free regions
            self.free_regions.push((allocation.offset, allocation.size));

            // Merge adjacent free regions
            self.merge_free_regions();
        }
    }

    /// Merge adjacent free regions
    fn merge_free_regions(&mut self) {
        self.free_regions.sort_by_key(|(offset, _)| *offset);

        let mut merged = Vec::new();
        let mut iter = self.free_regions.iter().peekable();

        while let Some(&(offset, size)) = iter.next() {
            let mut current_offset = offset;
            let mut current_size = size;

            // Merge with next region if adjacent
            while let Some(&(next_offset, next_size)) = iter.peek() {
                if current_offset + current_size == next_offset {
                    current_size += next_size;
                    iter.next();
                } else {
                    break;
                }
            }

            merged.push((current_offset, current_size));
        }

        self.free_regions = merged;
    }

    /// Get buffer usage percentage
    pub fn usage_percentage(&self) -> f32 {
        let used: u64 = self.allocations.iter().map(|a| a.size).sum();
        (used as f32 / self.size as f32) * 100.0
    }
}

/// Main buffer manager
pub struct BufferManager {
    config: BufferConfig,
    /// Instance buffers
    instance_buffers: Vec<GPUBuffer>,
    /// Cluster data buffers
    cluster_buffers: Vec<GPUBuffer>,
    /// Buffer ID counter
    next_buffer_id: u32,
    /// Instance data cache
    instance_cache: HashMap<u32, InstanceData>,
    /// Total allocated memory in bytes
    total_allocated: u64,
}

impl BufferManager {
    /// Create new buffer manager
    pub fn new(device: &Device, config: BufferConfig) -> Result<Self, BufferError> {
        let instance_buffer_size = (config.instance_buffer_size_mb * 1024 * 1024) as u64;

        let mut instance_buffers = Vec::new();

        // Create initial instance buffer
        let buffer = GPUBuffer::new(
            device,
            instance_buffer_size,
            BufferUsages::STORAGE | BufferUsages::VERTEX | BufferUsages::COPY_DST,
            "Nanite Instance Buffer",
        );
        instance_buffers.push(buffer);

        Ok(Self {
            config,
            instance_buffers,
            cluster_buffers: Vec::new(),
            next_buffer_id: 0,
            instance_cache: HashMap::new(),
            total_allocated: instance_buffer_size,
        })
    }

    /// Upload mesh instances to GPU
    pub fn upload_mesh_instances(
        &mut self,
        device: &Device,
        hierarchy: &ClusterHierarchy,
    ) -> Result<(), BufferError> {
        for node in &hierarchy.nodes {
            let cluster = &node.cluster;

            let instance_data = InstanceData {
                model_matrix_0: [1.0, 0.0, 0.0, 0.0],
                model_matrix_1: [0.0, 1.0, 0.0, 0.0],
                model_matrix_2: [0.0, 0.0, 1.0, 0.0],
                model_matrix_3: [0.0, 0.0, 0.0, 1.0],
                lod_level: cluster.lod_level as u32,
                cluster_id: cluster.id,
                padding: [0, 0],
            };

            self.instance_cache.insert(cluster.id, instance_data);
        }

        Ok(())
    }

    /// Update instance buffers with current LOD selections
    pub fn update_instances(
        &mut self,
        device: &Device,
        queue: &Queue,
        lod_selections: &[LODSelection],
    ) -> Result<(), BufferError> {
        // Collect instance data for visible clusters
        let mut instances = Vec::new();

        for selection in lod_selections {
            if !selection.visible {
                continue;
            }

            if let Some(base_data) = self.instance_cache.get(&selection.cluster_id) {
                let mut data = *base_data;
                data.lod_level = selection.lod_level;
                instances.push(data);
            }
        }

        // Allocate and upload
        let instance_size = std::mem::size_of::<InstanceData>() as u64;
        let total_size = instances.len() as u64 * instance_size;

        // Find buffer with enough space
        let mut allocated = None;
        for buffer in &mut self.instance_buffers {
            if let Some(alloc) = buffer.allocate(total_size, self.config.buffer_alignment as u64) {
                allocated = Some((buffer, alloc));
                break;
            }
        }

        // If no space, create new buffer
        if allocated.is_none() {
            let new_buffer = GPUBuffer::new(
                device,
                (self.config.instance_buffer_size_mb * 1024 * 1024) as u64,
                BufferUsages::STORAGE | BufferUsages::VERTEX | BufferUsages::COPY_DST,
                &format!("Nanite Instance Buffer {}", self.instance_buffers.len()),
            );
            self.total_allocated += (self.config.instance_buffer_size_mb * 1024 * 1024) as u64;

            if let Some(alloc) = new_buffer.allocate(total_size, self.config.buffer_alignment as u64) {
                self.instance_buffers.push(new_buffer);
                allocated = Some((self.instance_buffers.last_mut().unwrap(), alloc));
            }
        }

        // Upload to buffer
        if let Some((buffer, allocation)) = allocated {
            let bytes: &[u8] = bytemuck::cast_slice(&instances);
            queue.write_buffer(&buffer.buffer, allocation.offset, bytes);
        }

        Ok(())
    }

    /// Allocate cluster data buffer
    pub fn allocate_cluster_buffer(
        &mut self,
        device: &Device,
        size: u64,
    ) -> Result<BufferAllocation, BufferError> {
        let aligned_size = ((size + self.config.buffer_alignment as u64 - 1) /
            self.config.buffer_alignment as u64) * self.config.buffer_alignment as u64;

        for buffer in &mut self.cluster_buffers {
            if let Some(alloc) = buffer.allocate(aligned_size, self.config.buffer_alignment as u64) {
                return Ok(alloc);
            }
        }

        // Create new cluster buffer
        let buffer_size = ((self.config.instance_buffer_size_mb * 1024 * 1024) as u64).max(aligned_size * 2);
        let buffer = GPUBuffer::new(
            device,
            buffer_size,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            "Nanite Cluster Buffer",
        );

        self.total_allocated += buffer_size;

        if let Some(alloc) = buffer.allocate(aligned_size, self.config.buffer_alignment as u64) {
            self.cluster_buffers.push(buffer);
            Ok(alloc)
        } else {
            Err(BufferError::AllocationFailed(aligned_size))
        }
    }

    /// Get memory usage in MB
    pub fn memory_usage_mb(&self) -> f32 {
        self.total_allocated as f32 / (1024.0 * 1024.0)
    }

    /// Get instance buffer usage percentage
    pub fn instance_buffer_usage(&self) -> f32 {
        if self.instance_buffers.is_empty() {
            return 0.0;
        }

        let total_usage: f32 = self.instance_buffers.iter()
            .map(|b| b.usage_percentage())
            .sum();

        total_usage / self.instance_buffers.len() as f32
    }

    /// Defragment buffers (move allocations to reduce fragmentation)
    pub fn defragment(&mut self, device: &Device, queue: &Queue) -> Result<(), BufferError> {
        if !self.config.enable_defragmentation {
            return Ok(());
        }

        // Simple defragmentation: allocate new buffer and copy
        for buffer in &mut self.instance_buffers {
            if buffer.usage_percentage() < 50.0 {
                continue; // Skip if not fragmented
            }

            // Create new compact buffer
            let new_buffer = GPUBuffer::new(
                device,
                buffer.size,
                buffer.usage,
                "Defragmented Buffer",
            );

            // Copy allocations (simplified - would need actual data)
            for alloc in &buffer.allocations {
                if let Some(new_alloc) = new_buffer.allocate(alloc.size, alloc.alignment) {
                    // In real implementation, would copy data here
                }
            }

            // Swap buffers (old one will be dropped)
            *buffer = new_buffer;
        }

        Ok(())
    }

    /// Clear all buffers
    pub fn clear(&mut self) {
        self.instance_buffers.clear();
        self.cluster_buffers.clear();
        self.instance_cache.clear();
        self.total_allocated = 0;
    }

    /// Get total allocated memory in bytes
    pub fn total_allocated(&self) -> u64 {
        self.total_allocated
    }

    /// Get buffer count
    pub fn buffer_count(&self) -> usize {
        self.instance_buffers.len() + self.cluster_buffers.len()
    }
}

/// Errors that can occur in buffer management
#[derive(Debug, thiserror::Error)]
pub enum BufferError {
    #[error("Failed to allocate buffer of size {0} bytes")]
    AllocationFailed(u64),

    #[error("Buffer creation failed: {0}")]
    CreationFailed(String),

    #[error("Buffer upload failed: {0}")]
    UploadFailed(String),

    #[error("Invalid buffer offset: {0}")]
    InvalidOffset(u64),

    #[error("Buffer out of memory")]
    OutOfMemory,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_data_size() {
        assert_eq!(std::mem::size_of::<InstanceData>(), 128);
    }

    #[test]
    fn test_instance_data_default() {
        let data = InstanceData::default();
        assert_eq!(data.lod_level, 0);
        assert_eq!(data.cluster_id, 0);
    }

    #[test]
    fn test_buffer_allocation() {
        let mut buffer = GPUBuffer::new(
            &MockDevice::new(),
            1024,
            BufferUsages::STORAGE,
            "Test Buffer",
        );

        let alloc1 = buffer.allocate(128, 256);
        assert!(alloc1.is_some());

        let alloc2 = buffer.allocate(128, 256);
        assert!(alloc2.is_some());

        // Check usage
        assert!(buffer.usage_percentage() > 0.0);
    }

    #[test]
    fn test_buffer_free() {
        let mut buffer = GPUBuffer::new(
            &MockDevice::new(),
            1024,
            BufferUsages::STORAGE,
            "Test Buffer",
        );

        let alloc = buffer.allocate(128, 256).unwrap();
        buffer.free(&alloc);

        // After freeing, should be able to allocate again
        let alloc2 = buffer.allocate(128, 256);
        assert!(alloc2.is_some());
    }

    // Mock device for testing
    struct MockDevice;
    impl MockDevice {
        fn new() -> Self { Self }
    }
}
