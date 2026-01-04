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

pub mod api_mapping;
pub mod asset_converter;
pub mod component_mapping;
pub mod scene_migrator;
pub mod script_converter;
pub mod unity;
pub mod unreal;
pub mod wizard;

pub use unity::{UnityProjectImporter, UnityScene};
pub use unreal::{UnrealBlueprint, UnrealProjectImporter};

#[cfg(feature = "serde_yaml")]
pub use scene_migrator::{
    HierarchyNode, MigratedComponent, MigratedEntity, MigratedScene, SceneMetadata,
    SceneMigratorConfig, UnitySceneMigrator,
};

#[cfg(feature = "serde_yaml")]
pub use asset_converter::{
    AssetConverterConfig, AssetFormat, ConvertedAnimation, ConvertedMaterial, ConvertedModel,
    MaterialConversionMode, MaterialMapping, MeshData, SkeletonData, TextureQuality,
    UnityAssetConverter,
};

#[cfg(feature = "regex")]
pub use script_converter::{
    ConversionResult, ConvertedScript, ScriptLanguage, ScriptTarget, UnityScriptConverter,
};

pub use api_mapping::{
    APIMappingTable, APIMappingType, ConvertedScript as ApiConvertedScript,
    ScriptLanguage as ApiScriptLanguage, ScriptTarget as ApiScriptTarget, UnityAPICategory,
    UnityAPIMapping,
};
pub use component_mapping::{
    ComponentMapping, ComponentMappingRegistry, ComponentProperty, UnityComponent,
    UnityComponentType,
};
pub use wizard::{MigrationWizard, WizardError, WizardResult, quick_migrate};

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
            EngineType::Other => Err(MigrationError::UnsupportedEngine),
        }
    }

    /// 转换纹理
    async fn convert_textures(&self) -> Result<(), MigrationError> {
        use std::fs;

        // 查找所有纹理文件
        let assets_path = self.config.project_path.join("Assets");
        let mut texture_count = 0;

        // 递归查找纹理
        if let Ok(entries) = fs::read_dir(&assets_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let path = entry.path();
                    if file_type.is_file() {
                        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                        match extension {
                            "png" | "jpg" | "jpeg" | "tga" | "psd" | "gif" | "bmp" | "exr" => {
                                // 转换纹理
                                self.convert_single_texture(&path)?;
                                texture_count += 1;
                            }
                            _ => {}
                        }
                    } else if file_type.is_dir() {
                        // 递归处理子目录
                        self.convert_textures_recursive(&path, &mut texture_count)?;
                    }
                }
            }
        }

        tracing::info!("Converted {} textures", texture_count);
        Ok(())
    }

    /// 递归转换纹理
    fn convert_textures_recursive(
        &self,
        dir_path: &std::path::Path,
        count: &mut u32,
    ) -> Result<(), MigrationError> {
        use std::fs;

        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let path = entry.path();
                    if file_type.is_file() {
                        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                        match extension {
                            "png" | "jpg" | "jpeg" | "tga" | "psd" | "gif" | "bmp" | "exr" => {
                                self.convert_single_texture(&path)?;
                                *count += 1;
                            }
                            _ => {}
                        }
                    } else if file_type.is_dir() {
                        self.convert_textures_recursive(&path, count)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 转换单个纹理
    fn convert_single_texture(&self, texture_path: &std::path::Path) -> Result<(), MigrationError> {
        use std::fs;

        // 读取纹理文件
        let _texture_data = fs::read(texture_path)
            .map_err(|e| MigrationError::FileReadError(format!("Failed to read texture: {e}")))?;

        // 确定输出路径
        let relative_path =
            texture_path.strip_prefix(&self.config.project_path).unwrap_or(texture_path);
        let output_path = self.config.output_path.join(relative_path);

        // 创建输出目录
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                MigrationError::ConversionError(format!("Failed to create output dir: {e}"))
            })?;
        }

        // 复制纹理到输出目录（实际项目中可能需要格式转换）
        fs::copy(texture_path, &output_path)
            .map_err(|e| MigrationError::ConversionError(format!("Failed to copy texture: {e}")))?;

        tracing::debug!("Converted texture: {:?}", texture_path.file_name());
        Ok(())
    }

    /// 转换网格
    async fn convert_meshes(&self) -> Result<(), MigrationError> {
        use std::fs;

        let assets_path = self.config.project_path.join("Assets");
        let mut mesh_count = 0;

        if let Ok(entries) = fs::read_dir(&assets_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let path = entry.path();
                    if file_type.is_file() {
                        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                        match extension {
                            "fbx" | "obj" | "gltf" | "glb" => {
                                self.convert_single_mesh(&path)?;
                                mesh_count += 1;
                            }
                            _ => {}
                        }
                    } else if file_type.is_dir() {
                        self.convert_meshes_recursive(&path, &mut mesh_count)?;
                    }
                }
            }
        }

        tracing::info!("Converted {} meshes", mesh_count);
        Ok(())
    }

    /// 递归转换网格
    fn convert_meshes_recursive(
        &self,
        dir_path: &std::path::Path,
        count: &mut u32,
    ) -> Result<(), MigrationError> {
        use std::fs;

        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let path = entry.path();
                    if file_type.is_file() {
                        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                        match extension {
                            "fbx" | "obj" | "gltf" | "glb" => {
                                self.convert_single_mesh(&path)?;
                                *count += 1;
                            }
                            _ => {}
                        }
                    } else if file_type.is_dir() {
                        self.convert_meshes_recursive(&path, count)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 转换单个网格
    fn convert_single_mesh(&self, mesh_path: &std::path::Path) -> Result<(), MigrationError> {
        use std::fs;

        // 读取网格文件
        let _mesh_data = fs::read(mesh_path)
            .map_err(|e| MigrationError::FileReadError(format!("Failed to read mesh: {e}")))?;

        // 确定输出路径
        let relative_path = mesh_path.strip_prefix(&self.config.project_path).unwrap_or(mesh_path);
        let output_path = self.config.output_path.join(relative_path);

        // 创建输出目录
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                MigrationError::ConversionError(format!("Failed to create output dir: {e}"))
            })?;
        }

        // 复制网格到输出目录（实际项目中可能需要格式转换，如FBX→自定义格式）
        fs::copy(mesh_path, &output_path)
            .map_err(|e| MigrationError::ConversionError(format!("Failed to copy mesh: {e}")))?;

        tracing::debug!("Converted mesh: {:?}", mesh_path.file_name());
        Ok(())
    }

    /// 转换材质
    async fn convert_materials(&self) -> Result<(), MigrationError> {
        use std::fs;

        let assets_path = self.config.project_path.join("Assets");
        let mut material_count = 0;

        if let Ok(entries) = fs::read_dir(&assets_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let path = entry.path();
                    if file_type.is_file() {
                        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                        if extension == "mat" || extension == "asset" {
                            // 检查是否是材质文件
                            if let Ok(content) = fs::read_to_string(&path) {
                                if content.contains("Shader") || content.contains("Material") {
                                    self.convert_single_material(&path, &content)?;
                                    material_count += 1;
                                }
                            }
                        }
                    } else if file_type.is_dir() {
                        self.convert_materials_recursive(&path, &mut material_count)?;
                    }
                }
            }
        }

        tracing::info!("Converted {} materials", material_count);
        Ok(())
    }

    /// 递归转换材质
    fn convert_materials_recursive(
        &self,
        dir_path: &std::path::Path,
        count: &mut u32,
    ) -> Result<(), MigrationError> {
        use std::fs;

        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let path = entry.path();
                    if file_type.is_file() {
                        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                        if extension == "mat" || extension == "asset" {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if content.contains("Shader") || content.contains("Material") {
                                    self.convert_single_material(&path, &content)?;
                                    *count += 1;
                                }
                            }
                        }
                    } else if file_type.is_dir() {
                        self.convert_materials_recursive(&path, count)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 转换单个材质
    fn convert_single_material(
        &self,
        material_path: &std::path::Path,
        content: &str,
    ) -> Result<(), MigrationError> {
        use std::fs;

        // 解析Unity材质
        let shader_name = extract_shader_name(content);

        // 转换材质配置
        let converted_material = convert_unity_material_to_engine(content, &shader_name);

        // 确定输出路径
        let relative_path =
            material_path.strip_prefix(&self.config.project_path).unwrap_or(material_path);
        let output_path = self.config.output_path.join(relative_path);

        // 创建输出目录
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                MigrationError::ConversionError(format!("Failed to create output dir: {e}"))
            })?;
        }

        // 写入转换后的材质
        fs::write(&output_path, converted_material).map_err(|e| {
            MigrationError::ConversionError(format!("Failed to write material: {e}"))
        })?;

        tracing::debug!(
            "Converted material: {:?} with shader: {}",
            material_path.file_name(),
            shader_name
        );
        Ok(())
    }

    /// 转换场景
    async fn convert_scenes(&self) -> Result<(), MigrationError> {
        use std::fs;

        let assets_path = self.config.project_path.join("Assets");
        let mut scene_count = 0;

        if let Ok(entries) = fs::read_dir(&assets_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let path = entry.path();
                    if file_type.is_file() {
                        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                        if extension == "unity" {
                            #[cfg(feature = "serde_yaml")]
                            {
                                self.convert_single_scene(&path).await?;
                                scene_count += 1;
                            }
                        }
                    } else if file_type.is_dir() {
                        self.convert_scenes_recursive(&path, &mut scene_count).await?;
                    }
                }
            }
        }

        tracing::info!("Converted {} scenes", scene_count);
        Ok(())
    }

    /// 递归转换场景
    async fn convert_scenes_recursive(
        &self,
        dir_path: &std::path::Path,
        count: &mut u32,
    ) -> Result<(), MigrationError> {
        use std::fs;

        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let path = entry.path();
                    if file_type.is_file() {
                        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                        if extension == "unity" {
                            #[cfg(feature = "serde_yaml")]
                            {
                                self.convert_single_scene(&path).await?;
                                *count += 1;
                            }
                        }
                    } else if file_type.is_dir() {
                        // 使用Box::pin支持异步递归
                        Box::pin(self.convert_scenes_recursive(&path, count)).await?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 转换单个场景
    #[cfg(feature = "serde_yaml")]
    async fn convert_single_scene(
        &self,
        scene_path: &std::path::Path,
    ) -> Result<(), MigrationError> {
        // 使用UnityProjectImporter导入场景
        let importer = UnityProjectImporter::new();
        let scene_path_buf = scene_path.to_path_buf();
        let unity_scene = importer.import_scene(&scene_path_buf).await?;

        // 转换场景为引擎格式
        let engine_scene = convert_unity_scene_to_engine(&unity_scene);

        // 确定输出路径
        let relative_path =
            scene_path.strip_prefix(&self.config.project_path).unwrap_or(scene_path);
        let output_path = self.config.output_path.join(relative_path);

        // 创建输出目录
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                MigrationError::ConversionError(format!("Failed to create output dir: {}", e))
            })?;
        }

        // 写入转换后的场景
        std::fs::write(&output_path, engine_scene).map_err(|e| {
            MigrationError::ConversionError(format!("Failed to write scene: {}", e))
        })?;

        tracing::info!(
            "Converted scene: {} with {} GameObjects",
            unity_scene.name,
            unity_scene.game_objects.len()
        );
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
            MigrationError::FileReadError(msg) => write!(f, "File read error: {msg}"),
            MigrationError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            MigrationError::ConversionError(msg) => write!(f, "Conversion error: {msg}"),
            MigrationError::Other(msg) => write!(f, "Error: {msg}"),
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
    PhaseCompleted { phase: MigrationPhase },
    /// 资产转换
    AssetConverted {
        asset_path: PathBuf,
        asset_type: String,
    },
    /// 迁移完成
    Completed { result: MigrationResult },
    /// 迁移失败
    Failed { error: MigrationError },
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
// 辅助函数
// =============================================================================

/// 提取Unity材质中的Shader名称
fn extract_shader_name(material_content: &str) -> String {
    // 在Unity材质文件中查找Shader引用
    if let Some(line) = material_content.lines().find(|line| line.contains("shader")) {
        // 提取Shader名称
        if let Some(start) = line.find('"') {
            if let Some(end) = line[start + 1..].find('"') {
                return line[start + 1..start + 1 + end].to_string();
            }
        }
    }
    "Standard".to_string() // 默认Shader
}

/// 转换Unity材质到引擎材质
fn convert_unity_material_to_engine(material_content: &str, shader_name: &str) -> String {
    // 这里是一个简化的材质转换实现
    // 实际项目中需要解析完整的Unity材质结构

    format!(
        r#"# Converted Unity Material
# Original Shader: {shader_name}

engine_material:
  shader_type: "{shader_name}"
  properties:
    albedo_color: [1.0, 1.0, 1.0, 1.0]
    metallic: 0.0
    smoothness: 0.5
    normal_scale: 1.0
  textures: {{}}
"#
    )
}

/// 转换Unity场景到引擎场景
fn convert_unity_scene_to_engine(unity_scene: &UnityScene) -> String {
    let mut scene_yaml = format!("# Converted Unity Scene: {}\n\n", unity_scene.name);
    scene_yaml.push_str("entities:\n");

    // 转换每个GameObject
    for game_obj in &unity_scene.game_objects {
        scene_yaml.push_str(&format!("- name: \"{}\"\n", game_obj.name));
        scene_yaml.push_str(&format!(
            "  position: [{}, {}, {}]\n",
            game_obj.transform.position.0,
            game_obj.transform.position.1,
            game_obj.transform.position.2
        ));
        scene_yaml.push_str(&format!(
            "  rotation: [{}, {}, {}, {}]\n",
            game_obj.transform.rotation.0,
            game_obj.transform.rotation.1,
            game_obj.transform.rotation.2,
            game_obj.transform.rotation.3
        ));
        scene_yaml.push_str(&format!(
            "  scale: [{}, {}, {}]\n",
            game_obj.transform.scale.0, game_obj.transform.scale.1, game_obj.transform.scale.2
        ));

        // 添加组件
        if !game_obj.components.is_empty() {
            scene_yaml.push_str("  components:\n");
            for component in &game_obj.components {
                scene_yaml.push_str(&format!("    - {component}\n"));
            }
        }

        scene_yaml.push('\n');
    }

    scene_yaml
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
