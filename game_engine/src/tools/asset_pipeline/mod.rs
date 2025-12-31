//! # Asset Pipeline - 资源优化管线
//!
//! 本模块提供一站式资源优化解决方案。
//!
//! ## 功能特性
//!
//! - **自动LOD生成**: 为3D模型自动生成多级细节（LOD）
//! - **纹理压缩**: 支持多种压缩格式（BC1/BC3/BC7/ASTC/ETC2）
//! - **着色器优化**: 优化WGSL着色器代码，减小体积
//! - **资源打包**: 将资源打包成Pak、松散文件或虚拟文件系统
//! - **质量分析**: 生成详细的质量分析报告
//!
//! ## 使用示例
//!
//! ### CLI命令
//!
//! ```bash
//! # 优化资源
//! game-engine optimize ./assets -o ./assets_optimized --quality High --platform PC
//!
//! # 分析资源质量
//! game-engine analyze ./assets -o quality_report.html
//!
//! # 打包资源
//! game-engine bundle ./assets_optimized -o game.pak --format pak
//! ```
//!
//! ### 编程API
//!
//! ```rust,no_run
//! use game_engine::tools::asset_pipeline::{
//!     AssetPipeline, PipelineConfig, Platform, QualityPreset
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // 创建配置
//! let config = PipelineConfig {
//!     auto_lod: true,
//!     lod_levels: vec![1.0, 0.5, 0.25],
//!     auto_compress: true,
//!     auto_optimize_shaders: true,
//!     target_platform: Platform::PC,
//!     quality_preset: QualityPreset::High,
//!     ..Default::default()
//! };
//!
//! // 创建管线
//! let pipeline = AssetPipeline::new(config);
//!
//! // 运行优化
//! let report = pipeline.optimize_assets(
//!     std::path::Path::new("./assets"),
//!     std::path::Path::new("./assets_optimized")
//! ).await?;
//!
//! // 打印报告
//! report.print_summary();
//! # Ok(())
//! # }
//! ```

pub mod analyzer;
pub mod bundler;
pub mod cli;
pub mod lod_generator;
pub mod pipeline;
pub mod shader_optimizer;
pub mod texture_optimizer;

// 重新导出主要类型
pub use pipeline::{
    AssetMetadata, AssetPipeline, AssetProcessor, AssetType, OptimizationError, OptimizationResult,
    PipelineConfig, PipelineReport, Platform, QualityPreset,
};

pub use analyzer::{MetricStatus, QualityAnalyzer, QualityReport, QualityTargets};

pub use bundler::{
    AssetBundler, Bundle, BundleEntry, BundleFormat, BundleMetadata, CompressionAlgorithm,
};

pub use lod_generator::LODGenerator;

pub use texture_optimizer::{
    CompressedTexture, CompressionFormat, TextureOptimizer, TextureOptimizerOptions,
};

pub use shader_optimizer::{OptimizationLevel, ShaderOptimizationReport, ShaderOptimizer};

// 重新导出CLI功能
#[cfg(feature = "cli")]
pub use cli::{print_help, run_asset_pipeline_cli};
