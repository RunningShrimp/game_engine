//! # 开发工具模块
//!
//! 各种开发和调试工具集合。

pub mod resource_analysis;

pub mod ai_assistant;
pub mod doc_gen;
pub mod migration;

// DCC工具集成（条件编译）
pub mod dcc;
pub mod wasm_deploy;

// 资源导入工具（条件编译）
#[cfg(feature = "asset-importer")]
pub mod asset_importer;

// 资源优化管线（条件编译）
#[cfg(feature = "asset-pipeline")]
pub mod asset_pipeline;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "lsp")]
pub mod lsp;

#[cfg(feature = "csharp")]
pub mod csharp_sdk;

// 重新导出主要类型
pub use resource_analysis::{
    AnalysisResult, DependencyReportGenerator, RedundantAssetCleaner, ResourceDependencyGraph,
    ResourceNode, ResourceReferenceAnalyzer, ResourceScanner, ResourceType, UnusedResourceDetector,
    UsageStats, analyze_project_resources,
};

#[cfg(feature = "lsp")]
pub use lsp::{EngineAPIRegistry, GameEngineLSP};

#[cfg(feature = "csharp")]
pub use csharp_sdk::CSharpSdkGenerator;

// 重新导出资源导入工具（条件编译）
#[cfg(feature = "asset-importer")]
pub use asset_importer::{
    AssetFormat, AssetImportWizard, AssetImporter, AssetValidator, BatchImportSettings,
    BatchImporter, BatchProgress, BatchReport, CompressionFormat, DetectorError, FileAnalysis,
    ImportOptions, ImportResult, ImportSettings, PreviewData, ValidationIssue, ValidationResult,
    WizardResult, WizardStep,
};

// 重新导出DCC工具（条件编译）
#[cfg(feature = "dcc-tools")]
pub use dcc::{
    AnimationID, DCCAnimationEditor, DCCMaterialEditor, DCCToolkit, EdgeID, EditMode,
    EditorOperation, FaceID, GeneratedScript, KeyframeID, MaterialID, MeshEditor, ScriptGenerator,
    ScriptLanguage, TextureType, TransformTool, UVEditor, UVID, VertexID,
};

// WASM部署工具
pub use wasm_deploy::{
    BuildPhase, BuildStatus, DeploymentResult, DeploymentTarget, WasmBundle, WasmDeployConfig,
    WasmDeployError, WasmDeployTool, WasmOptLevel,
};

// 重新导出Asset Pipeline（条件编译）
#[cfg(feature = "asset-pipeline")]
pub use asset_pipeline::{
    AssetBundler, AssetMetadata, AssetPipeline, AssetProcessor, AssetType, Bundle, LODGenerator,
    MetricStatus, OptimizationError, OptimizationResult, PipelineConfig, PipelineReport, Platform,
    QualityAnalyzer, QualityPreset, QualityReport, ShaderOptimizer, TextureOptimizer,
};
