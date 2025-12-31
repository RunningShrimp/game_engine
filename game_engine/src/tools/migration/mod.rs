//! # Unity/UE5迁移工具
//!
//! 支持从Unity和Unreal Engine 5迁移项目到本引擎。
//!
//! ## 功能特性
//!
//! - **项目导入**: 解析Unity .unity 和 UE5 .umap 文件
//! - **资源转换**: 纹理、网格、材质转换
//! - **蓝图转换**: UE5蓝图转脚本系统
//! - **场景迁移**: 场景层级和组件转换

use crate::domain::events::{DomainEvent, EventError};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod unity;
pub mod unreal;

pub use unity::{UnityProjectImporter, UnityScene};
pub use unreal::{UnrealProjectImporter, UnrealBlueprint};

// =============================================================================
// 迁移配置
// =============================================================================

/// 引擎类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineType {
    /// Unity
    Unity,
    /// Unreal Engine 5
    Unreal5,
    /// 其他
    Other,
}

/// 迁移配置
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// 源引擎类型
    pub source_engine: EngineType,
    /// 项目路径
    pub project_path: PathBuf,
    /// 输出路径
    pub output_path: PathBuf,
    /// 是否转换纹理
    pub convert_textures: bool,
    /// 是否转换网格
    pub convert_meshes: bool,
    /// 是否转换材质
    pub convert_materials: bool,
    /// 是否转换场景
    pub convert_scenes: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            source_engine: EngineType::Other,
            project_path: PathBuf::new(),
            output_path: PathBuf::new(),
            convert_textures: true,
            convert_meshes: true,
            convert_materials: true,
            convert_scenes: true,
        }
    }
}

// =============================================================================
// 迁移进度
// =============================================================================

/// 迁移进度
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MigrationProgress {
    /// 总步骤数
    pub total_steps: u32,
    /// 已完成步骤
    pub completed_steps: u32,
    /// 当前阶段
    pub current_phase: MigrationPhase,
}

/// 迁移阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationPhase {
    /// 分析项目
    Analyzing,
    /// 转换纹理
    ConvertingTextures,
    /// 转换网格
    ConvertingMeshes,
    /// 转换材质
    ConvertingMaterials,
    /// 转换场景
    ConvertingScenes,
    /// 转换脚本
    ConvertingScripts,
    /// 完成
    Completed,
}

impl MigrationProgress {
    /// 获取进度百分比
    pub fn percentage(&self) -> f32 {
        if self.total_steps == 0 {
            return 0.0;
        }
        (self.completed_steps as f32 / self.total_steps as f32) * 100.0
    }
}

// =============================================================================
// 迁移管理器
// =============================================================================

/// 迁移管理器
pub struct MigrationManager {
    /// 配置
    config: MigrationConfig,
    /// 进度
    progress: MigrationProgress,
    /// 当前阶段
    current_phase: MigrationPhase,
}

impl MigrationManager {
    /// 创建新管理器
    pub fn new(config: MigrationConfig) -> Self {
        let total_steps = if config.convert_textures { 1 } else { 0 }
            + if config.convert_meshes { 1 } else { 0 }
            + if config.convert_materials { 1 } else { 0 }
            + if config.convert_scenes { 1 } else { 0 }
            + 1; // 分析阶段

        Self {
            config,
            progress: MigrationProgress {
                total_steps,
                completed_steps: 0,
                current_phase: MigrationPhase::Analyzing,
            },
            current_phase: MigrationPhase::Analyzing,
        }
    }

    /// 开始迁移
    pub async fn migrate(&mut self) -> Result<MigrationResult, MigrationError> {
        // 1. 分析项目
        self.current_phase = MigrationPhase::Analyzing;
        let analysis = self.analyze_project().await?;

        // 2. 转换纹理
        if self.config.convert_textures {
            self.current_phase = MigrationPhase::ConvertingTextures;
            self.convert_textures().await?;
            self.progress.completed_steps += 1;
        }

        // 3. 转换网格
        if self.config.convert_meshes {
            self.current_phase = MigrationPhase::ConvertingMeshes;
            self.convert_meshes().await?;
            self.progress.completed_steps += 1;
        }

        // 4. 转换材质
        if self.config.convert_materials {
            self.current_phase = MigrationPhase::ConvertingMaterials;
            self.convert_materials().await?;
            self.progress.completed_steps += 1;
        }

        // 5. 转换场景
        if self.config.convert_scenes {
            self.current_phase = MigrationPhase::ConvertingScenes;
            self.convert_scenes().await?;
            self.progress.completed_steps += 1;
        }

        self.current_phase = MigrationPhase::Completed;
        self.progress.completed_steps = self.progress.total_steps;

        Ok(MigrationResult {
            success: true,
            converted_assets: analysis.total_assets,
            warnings: vec![],
            errors: vec![],
        })
    }

    /// 分析项目
    async fn analyze_project(&self) -> Result<ProjectAnalysis, MigrationError> {
        match self.config.source_engine {
            EngineType::Unity => {
                let importer = UnityProjectImporter::new();
                importer.analyze(&self.config.project_path).await
            }
            EngineType::Unreal5 => {
                let importer = UnrealProjectImporter::new();
                importer.analyze(&self.config.project_path).await
            }
            EngineType::Other => {
                Err(MigrationError::UnsupportedEngine)
            }
        }
    }

    /// 转换纹理
    async fn convert_textures(&self) -> Result<(), MigrationError> {
        // TODO: 实现纹理转换
        Ok(())
    }

    /// 转换网格
    async fn convert_meshes(&self) -> Result<(), MigrationError> {
        // TODO: 实现网格转换
        Ok(())
    }

    /// 转换材质
    async fn convert_materials(&self) -> Result<(), MigrationError> {
        // TODO: 实现材质转换
        Ok(())
    }

    /// 转换场景
    async fn convert_scenes(&self) -> Result<(), MigrationError> {
        // TODO: 实现场景转换
        Ok(())
    }

    /// 获取进度
    pub fn get_progress(&self) -> MigrationProgress {
        self.progress
    }
}

/// 项目分析结果
#[derive(Debug, Clone)]
pub struct ProjectAnalysis {
    /// 总资产数
    pub total_assets: u32,
    /// 纹理数量
    pub texture_count: u32,
    /// 网格数量
    pub mesh_count: u32,
    /// 材质数量
    pub material_count: u32,
    /// 场景数量
    pub scene_count: u32,
    /// 脚本数量
    pub script_count: u32,
}

/// 迁移结果
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// 是否成功
    pub success: bool,
    /// 转换的资产数
    pub converted_assets: u32,
    /// 警告列表
    pub warnings: Vec<String>,
    /// 错误列表
    pub errors: Vec<String>,
}

/// 迁移错误
#[derive(Debug, Clone)]
pub enum MigrationError {
    /// 不支持的引擎
    UnsupportedEngine,
    /// 项目路径无效
    InvalidProjectPath,
    /// 文件读取错误
    FileReadError(String),
    /// 解析错误
    ParseError(String),
    /// 转换错误
    ConversionError(String),
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::UnsupportedEngine => write!(f, "Unsupported engine"),
            MigrationError::InvalidProjectPath => write!(f, "Invalid project path"),
            MigrationError::FileReadError(msg) => write!(f, "File read error: {}", msg),
            MigrationError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            MigrationError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            MigrationError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for MigrationError {}

// =============================================================================
// 迁移事件
// =============================================================================

/// 迁移事件
#[derive(Debug, Clone)]
pub enum MigrationEvent {
    /// 开始迁移
    Started {
        engine_type: EngineType,
        project_path: PathBuf,
    },
    /// 阶段完成
    PhaseCompleted {
        phase: MigrationPhase,
    },
    /// 资产转换
    AssetConverted {
        asset_path: PathBuf,
        asset_type: String,
    },
    /// 迁移完成
    Completed {
        result: MigrationResult,
    },
    /// 迁移失败
    Failed {
        error: MigrationError,
    },
}

impl DomainEvent for MigrationEvent {
    fn event_type(&self) -> &'static str {
        match self {
            MigrationEvent::Started { .. } => "Started",
            MigrationEvent::PhaseCompleted { .. } => "PhaseCompleted",
            MigrationEvent::AssetConverted { .. } => "AssetConverted",
            MigrationEvent::Completed { .. } => "Completed",
            MigrationEvent::Failed { .. } => "Failed",
        }
    }

    fn apply(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }

    fn revert(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// =============================================================================
// ECS集成
// =============================================================================

/// 迁移管理器资源
#[derive(Resource)]
pub struct MigrationManagerResource {
    pub manager: MigrationManager,
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = MigrationConfig::default();
        assert_eq!(config.source_engine, EngineType::Other);
    }

    #[test]
    fn test_progress_percentage() {
        let progress = MigrationProgress {
            total_steps: 10,
            completed_steps: 5,
            current_phase: MigrationPhase::Analyzing,
        };

        assert_eq!(progress.percentage(), 50.0);
    }

    #[test]
    fn test_manager_creation() {
        let config = MigrationConfig {
            convert_textures: true,
            convert_meshes: true,
            ..Default::default()
        };

        let manager = MigrationManager::new(config);
        assert_eq!(manager.progress.total_steps, 3); // 分析 + 纹理 + 网格
    }
}
