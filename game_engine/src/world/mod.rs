//! 世界生成模块
//!
//! 提供异步世界生成功能，包括地形生成、物体放置、导航网格生成等。

/// 异步世界生成器
pub mod async_generator;

pub use async_generator::{
    AsyncWorldGenerator, GeneratedEntity, GenerationConfig, GenerationType, NavMeshData,
    NavMeshRegion, ObjectType, TerrainData, WorldGenerationError, WorldGenerationRequest,
    WorldGenerationResult,
};
