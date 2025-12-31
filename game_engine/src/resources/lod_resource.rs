//! # LOD Resource Integration
//!
//! Integrates automatic LOD generation into the resource loading pipeline.
//!
//! ## Features
//!
//! - Automatic LOD generation during mesh import
//! - LOD caching and management
//! - Quality-based LOD selection
//! - Platform-aware LOD generation

use crate::render::lod_generator::{LODConfig, LODGenerator, LODGroup};
use crate::render::mesh_simplifier::Mesh;
use crate::render::quality_assessor::{QualityAssessor, QualityConfig};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Errors that can occur during LOD resource management
#[derive(Debug, thiserror::Error)]
pub enum LODResourceError {
    #[error("Mesh not found: {0}")]
    MeshNotFound(String),

    #[error("LOD generation failed: {0}")]
    GenerationFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// LOD resource with metadata
#[derive(Clone, Debug)]
pub struct LODResource {
    /// Resource identifier
    pub id: String,

    /// Original file path
    pub source_path: PathBuf,

    /// LOD levels
    pub lods: Arc<LODGroup>,

    /// Quality assessment
    pub quality_score: f32,

    /// Timestamp when LODs were generated
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl LODResource {
    /// Create a new LOD resource
    pub fn new(
        id: String,
        source_path: PathBuf,
        lods: LODGroup,
        quality_score: f32,
    ) -> Self {
        Self {
            id,
            source_path,
            lods: Arc::new(lods),
            quality_score,
            generated_at: chrono::Utc::now(),
        }
    }

    /// Get recommended LOD level for given screen size
    pub fn select_lod(&self, screen_size: f32) -> usize {
        self.lods.select_level(screen_size).index
    }

    /// Get total memory usage
    pub fn memory_usage(&self) -> usize {
        self.lods.total_memory_usage()
    }

    /// Check if LODs need regeneration
    pub fn needs_regeneration(&self, max_age_days: i64) -> bool {
        let age = chrono::Utc::now() - self.generated_at;
        age.num_days() > max_age_days
    }
}

/// LOD resource cache
pub struct LODResourceCache {
    /// Cached LOD resources by mesh ID
    cache: RwLock<HashMap<String, LODResource>>,

    /// Maximum cache size in bytes
    max_cache_size: usize,

    /// Current cache size
    current_size: Arc<RwLock<usize>>,
}

impl LODResourceCache {
    /// Create a new LOD resource cache
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_cache_size,
            current_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Insert LOD resource into cache
    pub async fn insert(&self, resource: LODResource) -> Result<(), LODResourceError> {
        let memory = resource.memory_usage();

        // Check if we need to evict
        {
            let current_size = *self.current_size.read().await;
            if current_size + memory > self.max_cache_size {
                self.evict_lru(memory).await?;
            }
        }

        // Insert resource
        {
            let mut cache = self.cache.write().await;
            cache.insert(resource.id.clone(), resource);
        }

        // Update current size
        let mut current_size = self.current_size.write().await;
        *current_size += memory;

        Ok(())
    }

    /// Get LOD resource from cache
    pub async fn get(&self, id: &str) -> Option<LODResource> {
        let cache = self.cache.read().await;
        cache.get(id).cloned()
    }

    /// Remove LRU (least recently used) items to free memory
    async fn evict_lru(&self, required_memory: usize) -> Result<(), LODResourceError> {
        // Simple strategy: clear 50% of cache when full
        let target_size = self.max_cache_size / 2;

        let mut cache = self.cache.write().await;
        let mut freed = 0;

        // Collect IDs to remove (we can't modify while iterating)
        let ids_to_remove: Vec<(String, usize)> = cache
            .iter()
            .map(|(id, resource)| (id.clone(), resource.memory_usage()))
            .collect();

        for (id, memory) in ids_to_remove {
            if freed >= required_memory || *self.current_size.read().await - freed < target_size {
                break;
            }

            cache.remove(&id);
            freed += memory;
        }

        // Update current size
        let mut current_size = self.current_size.write().await;
        *current_size -= freed;

        Ok(())
    }

    /// Clear entire cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();

        let mut current_size = self.current_size.write().await;
        *current_size = 0;
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let current_size = *self.current_size.read().await;

        CacheStats {
            total_resources: cache.len(),
            total_memory_bytes: current_size,
            max_memory_bytes: self.max_cache_size,
            utilization: current_size as f32 / self.max_cache_size as f32,
        }
    }
}

/// Cache statistics
#[derive(Clone, Debug)]
pub struct CacheStats {
    pub total_resources: usize,
    pub total_memory_bytes: usize,
    pub max_memory_bytes: usize,
    pub utilization: f32,
}

/// LOD resource manager - integrates LOD generation into resource pipeline
pub struct LODResourceManager {
    /// LOD generator
    generator: LODGenerator,

    /// Quality assessor
    assessor: QualityAssessor,

    /// LOD cache
    cache: LODResourceCache,

    /// Auto-generate LODs on import
    auto_generate: bool,

    /// Regenerate LODs after this many days (0 = never)
    regenerate_after_days: i64,
}

impl LODResourceManager {
    /// Create a new LOD resource manager
    pub fn new(
        lod_config: LODConfig,
        quality_config: QualityConfig,
        cache_size: usize,
    ) -> Result<Self, LODResourceError> {
        let generator = LODGenerator::with_config(lod_config)
            .map_err(|e| LODResourceError::GenerationFailed(e.to_string()))?;

        let assessor = QualityAssessor::with_config(quality_config);
        let cache = LODResourceCache::new(cache_size);

        Ok(Self {
            generator,
            assessor,
            cache,
            auto_generate: true,
            regenerate_after_days: 30, // Default: regenerate after 30 days
        })
    }

    /// Create LOD resource manager with defaults
    pub fn with_defaults() -> Result<Self, LODResourceError> {
        Self::new(
            LODConfig::default(),
            QualityConfig::default(),
            1024 * 1024 * 1024, // 1GB cache
        )
    }

    /// Generate LODs from a mesh
    pub async fn generate_lods(
        &self,
        mesh_id: String,
        mesh: &Mesh,
    ) -> Result<LODResource, LODResourceError> {
        // Generate LODs
        let lods = self
            .generator
            .generate_from_mesh(mesh)
            .map_err(|e| LODResourceError::GenerationFailed(e.to_string()))?;

        // Assess quality
        let assessment = self.assessor.assess_lods(&lods);

        // Create resource
        let resource = LODResource::new(
            mesh_id.clone(),
            PathBuf::from(format!("{}.lod", mesh_id)),
            lods,
            assessment.overall_score,
        );

        // Cache it
        self.cache.insert(resource.clone()).await?;

        Ok(resource)
    }

    /// Get LOD resource from cache or generate
    pub async fn get_or_generate(
        &self,
        mesh_id: String,
        mesh: &Mesh,
    ) -> Result<LODResource, LODResourceError> {
        // Try cache first
        if let Some(cached) = self.cache.get(&mesh_id).await {
            // Check if needs regeneration
            if cached.needs_regeneration(self.regenerate_after_days) {
                // Regenerate
                return self.generate_lods(mesh_id, mesh).await;
            }

            return Ok(cached);
        }

        // Not in cache, generate
        self.generate_lods(mesh_id, mesh).await
    }

    /// Set auto-generation flag
    pub fn set_auto_generate(&mut self, auto: bool) {
        self.auto_generate = auto;
    }

    /// Set regeneration period
    pub fn set_regeneration_period(&mut self, days: i64) {
        self.regenerate_after_days = days;
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        self.cache.stats().await
    }

    /// Clear LOD cache
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }
}

/// Helper function to integrate LOD generation into mesh loading pipeline
pub async fn load_mesh_with_lods(
    manager: &LODResourceManager,
    mesh_id: String,
    mesh: Mesh,
) -> Result<LODResource, LODResourceError> {
    if manager.auto_generate {
        manager.get_or_generate(mesh_id, &mesh).await
    } else {
        // Don't auto-generate, return error
        Err(LODResourceError::MeshNotFound(
            "LOD generation disabled".to_string(),
        ))
    }
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
        ];

        let indices = vec![0, 1, 2, 1, 3, 2];

        Mesh::from_vertices_and_indices(vertices, indices).unwrap()
    }

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let cache = LODResourceCache::new(1024 * 1024); // 1MB cache

        let mesh = create_test_mesh();
        let lods = LODGenerator::new()
            .generate_from_mesh(&mesh)
            .unwrap();

        let resource = LODResource::new(
            "test_mesh".to_string(),
            PathBuf::from("test.obj"),
            lods,
            0.9,
        );

        cache.insert(resource.clone()).await.unwrap();

        let retrieved = cache.get("test_mesh").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test_mesh");
    }

    #[tokio::test]
    async fn test_manager_generation() {
        let manager = LODResourceManager::with_defaults().unwrap();

        let mesh = create_test_mesh();
        let resource = manager.generate_lods("test".to_string(), &mesh).await.unwrap();

        assert_eq!(resource.id, "test");
        assert!(resource.quality_score > 0.0);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = LODResourceCache::new(1024 * 1024);

        let stats = cache.stats().await;
        assert_eq!(stats.total_resources, 0);
        assert_eq!(stats.utilization, 0.0);
    }
}
