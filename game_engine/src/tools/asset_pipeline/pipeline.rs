//! # Asset Pipeline - 资源优化管线核心
//!
//! 本模块实现自动化资源优化管线，提供一站式的资源处理方案。

use crate::tools::asset_pipeline::{
    analyzer::QualityAnalyzer,
    bundler::{AssetBundler, Bundle},
    lod_generator::LODGenerator,
    shader_optimizer::ShaderOptimizer,
    texture_optimizer::{TextureOptimizer, TextureOptimizerOptions},
};
use futures::future::join_all;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::fs;
use walkdir::WalkDir;

/// 管线配置
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// 自动生成LOD
    pub auto_lod: bool,

    /// LOD级别 (0.0 - 1.0)，例如 [1.0, 0.5, 0.25]
    pub lod_levels: Vec<f32>,

    /// 自动压缩纹理
    pub auto_compress: bool,

    /// 纹理压缩选项
    pub texture_options: TextureOptimizerOptions,

    /// 自动优化着色器
    pub auto_optimize_shaders: bool,

    /// 目标平台
    pub target_platform: Platform,

    /// 质量预设
    pub quality_preset: QualityPreset,

    /// 并发任务数
    pub concurrent_jobs: usize,

    /// 输出详细日志
    pub verbose: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            auto_lod: true,
            lod_levels: vec![1.0, 0.5, 0.25],
            auto_compress: true,
            texture_options: TextureOptimizerOptions::default(),
            auto_optimize_shaders: true,
            target_platform: Platform::PC,
            quality_preset: QualityPreset::High,
            concurrent_jobs: 4,
            verbose: false,
        }
    }
}

/// 目标平台
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// PC (Windows, macOS, Linux)
    PC,

    /// 移动平台 (iOS, Android)
    Mobile,

    /// Web平台
    Web,

    /// 游戏主机 (PS5, Xbox Series X)
    Console,
}

impl Platform {
    /// 获取平台推荐的纹理压缩格式
    pub fn recommended_texture_format(&self) -> crate::tools::asset_pipeline::CompressionFormat {
        match self {
            Platform::PC => crate::tools::asset_pipeline::CompressionFormat::BC7,
            Platform::Mobile => {
                #[cfg(target_os = "android")]
                return crate::tools::asset_pipeline::CompressionFormat::ASTC4x4;

                #[cfg(target_os = "ios")]
                return crate::tools::asset_pipeline::CompressionFormat::ASTC4x4;

                crate::tools::asset_pipeline::CompressionFormat::ETC2
            }
            Platform::Web => crate::tools::asset_pipeline::CompressionFormat::BC7,
            Platform::Console => crate::tools::asset_pipeline::CompressionFormat::BC7,
        }
    }
}

/// 质量预设
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityPreset {
    Low,
    Medium,
    High,
    Ultra,
    Custom,
}

impl QualityPreset {
    /// 从字符串解析质量预设
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(QualityPreset::Low),
            "medium" => Some(QualityPreset::Medium),
            "high" => Some(QualityPreset::High),
            "ultra" => Some(QualityPreset::Ultra),
            _ => None,
        }
    }

    /// 获取推荐的LOD级别
    pub fn recommended_lod_levels(&self) -> Vec<f32> {
        match self {
            QualityPreset::Low => vec![1.0, 0.3],
            QualityPreset::Medium => vec![1.0, 0.5, 0.25],
            QualityPreset::High => vec![1.0, 0.75, 0.5, 0.25],
            QualityPreset::Ultra => vec![1.0, 0.875, 0.75, 0.5, 0.25, 0.125],
            QualityPreset::Custom => vec![1.0, 0.5, 0.25],
        }
    }
}

/// 管线报告
#[derive(Debug, Clone)]
pub struct PipelineReport {
    /// 总资源数
    pub total_assets: usize,

    /// 成功处理的资源数
    pub successful_assets: usize,

    /// 失败的资源数
    pub failed_assets: usize,

    /// 生成的LOD数量
    pub lods_generated: usize,

    /// 压缩的纹理数量
    pub textures_compressed: usize,

    /// 优化的着色器数量
    pub shaders_optimized: usize,

    /// 原始总大小（字节）
    pub original_size: u64,

    /// 优化后总大小（字节）
    pub optimized_size: u64,

    /// 大小减少百分比
    pub size_reduction: f64,

    /// 处理时间（秒）
    pub processing_time: f64,

    /// 详细结果
    pub asset_results: Vec<OptimizationResult>,

    /// 错误信息
    pub errors: Vec<String>,
}

impl Default for PipelineReport {
    fn default() -> Self {
        Self {
            total_assets: 0,
            successful_assets: 0,
            failed_assets: 0,
            lods_generated: 0,
            textures_compressed: 0,
            shaders_optimized: 0,
            original_size: 0,
            optimized_size: 0,
            size_reduction: 0.0,
            processing_time: 0.0,
            asset_results: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl PipelineReport {
    /// 创建新的报告
    pub fn new() -> Self {
        Self::default()
    }

    /// 打印报告摘要
    pub fn print_summary(&self) {
        println!("\n=== Asset Optimization Report ===");
        println!("Total Assets: {}", self.total_assets);
        println!(
            "Successful: {} / Failed: {}",
            self.successful_assets, self.failed_assets
        );
        println!("\nOptimization Results:");
        println!("  LODs Generated: {}", self.lods_generated);
        println!("  Textures Compressed: {}", self.textures_compressed);
        println!("  Shaders Optimized: {}", self.shaders_optimized);
        println!("\nSize Reduction:");
        println!("  Original: {} MB", self.original_size / 1024 / 1024);
        println!("  Optimized: {} MB", self.optimized_size / 1024 / 1024);
        println!("  Reduction: {:.1}%", self.size_reduction);
        println!("  Processing Time: {:.2}s", self.processing_time);

        if !self.errors.is_empty() {
            println!("\nErrors:");
            for error in &self.errors[..5.min(self.errors.len())] {
                println!("  - {}", error);
            }
            if self.errors.len() > 5 {
                println!("  ... and {} more", self.errors.len() - 5);
            }
        }
    }
}

/// Asset Pipeline - 资源优化管线
pub struct AssetPipeline {
    config: PipelineConfig,
    lod_generator: LODGenerator,
    texture_optimizer: TextureOptimizer,
    shader_optimizer: ShaderOptimizer,
    bundler: AssetBundler,
    analyzer: QualityAnalyzer,
}

impl AssetPipeline {
    /// 创建新的优化管线
    pub fn new(config: PipelineConfig) -> Self {
        let lod_generator = LODGenerator::new(config.lod_levels.clone());
        let texture_optimizer = TextureOptimizer::new(config.texture_options.clone());
        let shader_optimizer = ShaderOptimizer::new();
        let bundler = AssetBundler::new();
        let analyzer = QualityAnalyzer::new();

        Self {
            config,
            lod_generator,
            texture_optimizer,
            shader_optimizer,
            bundler,
            analyzer,
        }
    }

    /// 使用默认配置创建管线
    pub fn with_defaults() -> Self {
        Self::new(PipelineConfig::default())
    }

    /// 使用质量预设创建管线
    pub fn with_quality_preset(preset: QualityPreset, platform: Platform) -> Self {
        let mut config = PipelineConfig::default();
        config.quality_preset = preset;
        config.target_platform = platform;
        config.lod_levels = preset.recommended_lod_levels();
        config.texture_options.compression_format = platform.recommended_texture_format();

        Self::new(config)
    }

    /// 优化资源目录
    pub async fn optimize_assets(
        &self,
        assets_dir: &Path,
        output_dir: &Path,
    ) -> Result<PipelineReport, OptimizationError> {
        let start_time = Instant::now();
        let mut report = PipelineReport::new();

        // 确保输出目录存在
        fs::create_dir_all(output_dir).await.map_err(|e| {
            OptimizationError::IoError(format!("Failed to create output directory: {}", e))
        })?;

        if self.config.verbose {
            println!("Scanning assets in: {}", assets_dir.display());
        }

        // 1. 扫描资源
        let assets = self.scan_assets(assets_dir).await?;
        report.total_assets = assets.len();

        if self.config.verbose {
            println!("Found {} assets to process", assets.len());
        }

        // 2. 按类型分组处理
        let mut processing_tasks = Vec::new();

        for asset in assets {
            let asset_path = asset.path.clone();
            let output_path =
                output_dir.join(asset_path.strip_prefix(assets_dir).unwrap_or(&asset_path));

            // 创建输出目录
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).await.ok();
            }

            match asset.asset_type {
                AssetType::Model => {
                    if self.config.auto_lod {
                        let task = self.process_model_asset(asset, output_path);
                        processing_tasks.push(task);
                    }
                }
                AssetType::Texture => {
                    if self.config.auto_compress {
                        let task = self.process_texture_asset(asset, output_path);
                        processing_tasks.push(task);
                    }
                }
                AssetType::Shader => {
                    if self.config.auto_optimize_shaders {
                        let task = self.process_shader_asset(asset, output_path);
                        processing_tasks.push(task);
                    }
                }
                _ => {
                    // 其他类型直接复制
                    let task = self.copy_asset(asset, output_path);
                    processing_tasks.push(task);
                }
            }
        }

        // 3. 执行所有处理任务（带并发限制）
        let results = self.execute_with_concurrency_limit(processing_tasks).await;

        // 4. 收集结果
        for result in results {
            match result {
                Ok(asset_result) => {
                    report.successful_assets += 1;
                    report.original_size += asset_result.original_size;
                    report.optimized_size += asset_result.optimized_size;

                    match asset_result.asset_type {
                        AssetType::Model => {
                            if asset_result.lods_generated > 0 {
                                report.lods_generated += asset_result.lods_generated;
                            }
                        }
                        AssetType::Texture => {
                            if asset_result.compressed {
                                report.textures_compressed += 1;
                            }
                        }
                        AssetType::Shader => {
                            if asset_result.optimized {
                                report.shaders_optimized += 1;
                            }
                        }
                        _ => {}
                    }

                    report.asset_results.push(asset_result);
                }
                Err(e) => {
                    report.failed_assets += 1;
                    report.errors.push(e.to_string());
                }
            }
        }

        // 5. 计算统计数据
        report.processing_time = start_time.elapsed().as_secs_f64();
        if report.original_size > 0 {
            report.size_reduction =
                (1.0 - (report.optimized_size as f64 / report.original_size as f64)) * 100.0;
        }

        // 6. 生成质量分析报告
        self.analyzer.generate_report(&report, output_dir).await?;

        // 7. 打包资源（可选）
        // self.bundler.bundle_assets(&report.asset_results, output_dir).await?;

        Ok(report)
    }

    /// 扫描资源目录
    async fn scan_assets(
        &self,
        assets_dir: &Path,
    ) -> Result<Vec<AssetMetadata>, OptimizationError> {
        let mut assets = Vec::new();

        for entry in WalkDir::new(assets_dir).follow_links(true).into_iter().filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_file() {
                if let Some(asset_type) = AssetType::from_path(path) {
                    let metadata = fs::metadata(path).await.ok();
                    let size = metadata.map(|m| m.len()).unwrap_or(0);

                    assets.push(AssetMetadata {
                        path: path.to_path_buf(),
                        asset_type,
                        size,
                    });
                }
            }
        }

        Ok(assets)
    }

    /// 处理模型资源（生成LOD）
    async fn process_model_asset(
        &self,
        asset: AssetMetadata,
        output_path: PathBuf,
    ) -> Result<OptimizationResult, OptimizationError> {
        if self.config.verbose {
            println!("Processing model: {}", asset.path.display());
        }

        let start = Instant::now();

        // 使用LOD生成器生成多级LOD
        let lods = self.lod_generator.generate_lods(&asset.path).await?;

        // 保存LOD模型
        let lod_dir = output_path.with_extension("");
        fs::create_dir_all(&lod_dir).await.map_err(|e| {
            OptimizationError::IoError(format!("Failed to create LOD directory: {}", e))
        })?;

        for (i, lod) in lods.iter().enumerate() {
            let lod_path = lod_dir.join(format!("lod{}.gltf", i));
            lod.save(&lod_path).await?;
        }

        let elapsed = start.elapsed();

        Ok(OptimizationResult {
            asset_path: asset.path.clone(),
            asset_type: AssetType::Model,
            original_size: asset.size,
            optimized_size: asset.size, // TODO: 计算实际大小
            lods_generated: lods.len(),
            compressed: false,
            optimized: false,
            processing_time: elapsed.as_secs_f64(),
        })
    }

    /// 处理纹理资源（压缩）
    async fn process_texture_asset(
        &self,
        asset: AssetMetadata,
        output_path: PathBuf,
    ) -> Result<OptimizationResult, OptimizationError> {
        if self.config.verbose {
            println!("Processing texture: {}", asset.path.display());
        }

        let start = Instant::now();

        // 使用纹理优化器压缩纹理
        let compressed = self.texture_optimizer.compress_texture(&asset.path, &output_path).await?;

        let optimized_size = fs::metadata(&output_path).await?.len();
        let elapsed = start.elapsed();

        Ok(OptimizationResult {
            asset_path: asset.path.clone(),
            asset_type: AssetType::Texture,
            original_size: asset.size,
            optimized_size,
            lods_generated: 0,
            compressed,
            optimized: false,
            processing_time: elapsed.as_secs_f64(),
        })
    }

    /// 处理着色器资源（优化）
    async fn process_shader_asset(
        &self,
        asset: AssetMetadata,
        output_path: PathBuf,
    ) -> Result<OptimizationResult, OptimizationError> {
        if self.config.verbose {
            println!("Processing shader: {}", asset.path.display());
        }

        let start = Instant::now();

        // 读取着色器源码
        let source = fs::read_to_string(&asset.path).await.map_err(|e| {
            OptimizationError::IoError(format!("Failed to read shader file: {}", e))
        })?;

        // 使用着色器优化器优化
        let optimized_source = self.shader_optimizer.optimize_wgsl(&source)?;

        // 保存优化后的着色器
        fs::write(&output_path, optimized_source).await.map_err(|e| {
            OptimizationError::IoError(format!("Failed to write optimized shader: {}", e))
        })?;

        let optimized_size = fs::metadata(&output_path).await?.len();
        let elapsed = start.elapsed();

        Ok(OptimizationResult {
            asset_path: asset.path.clone(),
            asset_type: AssetType::Shader,
            original_size: asset.size,
            optimized_size,
            lods_generated: 0,
            compressed: false,
            optimized: true,
            processing_time: elapsed.as_secs_f64(),
        })
    }

    /// 复制其他资源
    async fn copy_asset(
        &self,
        asset: AssetMetadata,
        output_path: PathBuf,
    ) -> Result<OptimizationResult, OptimizationError> {
        fs::copy(&asset.path, &output_path)
            .await
            .map_err(|e| OptimizationError::IoError(format!("Failed to copy asset: {}", e)))?;

        Ok(OptimizationResult {
            asset_path: asset.path.clone(),
            asset_type: asset.asset_type,
            original_size: asset.size,
            optimized_size: asset.size,
            lods_generated: 0,
            compressed: false,
            optimized: false,
            processing_time: 0.0,
        })
    }

    /// 执行任务并限制并发数
    async fn execute_with_concurrency_limit<F, T>(
        &self,
        tasks: Vec<F>,
    ) -> Vec<Result<T, OptimizationError>>
    where
        F: std::future::Future<Output = Result<T, OptimizationError>> + Send + 'static,
        T: Send + 'static,
    {
        use std::sync::Arc;
        use tokio::sync::Semaphore;

        let semaphore = Arc::new(Semaphore::new(self.config.concurrent_jobs));
        let mut handles = Vec::new();

        for task in tasks {
            let permit = semaphore.clone();
            let handle = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                task.await
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(OptimizationError::Other(format!(
                    "Task panicked: {}",
                    e
                )))),
            }
        }

        results
    }
}

/// 资源元数据
#[derive(Debug, Clone)]
pub struct AssetMetadata {
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub size: u64,
}

/// 资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AssetType {
    Model,
    Texture,
    Shader,
    Audio,
    Font,
    Other,
}

impl AssetType {
    /// 从文件路径推断资源类型
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_lowercase();

        match extension.as_str() {
            // 模型格式
            "gltf" | "glb" | "fbx" | "obj" => Some(AssetType::Model),

            // 纹理格式
            "png" | "jpg" | "jpeg" | "tga" | "bmp" | "webp" | "gif" => Some(AssetType::Texture),

            // 着色器格式
            "wgsl" | "vert" | "frag" => Some(AssetType::Shader),

            // 音频格式
            "wav" | "mp3" | "ogg" | "flac" => Some(AssetType::Audio),

            // 字体格式
            "ttf" | "otf" | "woff" | "woff2" => Some(AssetType::Font),

            _ => Some(AssetType::Other),
        }
    }
}

/// 资源处理器trait
#[async_trait::async_trait]
pub trait AssetProcessor: Send + Sync {
    async fn process(
        &self,
        asset: &AssetMetadata,
        output_path: &Path,
    ) -> Result<OptimizationResult, OptimizationError>;
}

/// 优化结果
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub asset_path: PathBuf,
    pub asset_type: AssetType,
    pub original_size: u64,
    pub optimized_size: u64,
    pub lods_generated: usize,
    pub compressed: bool,
    pub optimized: bool,
    pub processing_time: f64,
}

/// 优化错误类型
#[derive(Debug, thiserror::Error)]
pub enum OptimizationError {
    #[error("IO Error: {0}")]
    IoError(String),

    #[error("LOD Generation Error: {0}")]
    LODError(String),

    #[error("Texture Compression Error: {0}")]
    TextureError(String),

    #[error("Shader Optimization Error: {0}")]
    ShaderError(String),

    #[error("Bundling Error: {0}")]
    BundleError(String),

    #[error("Unsupported asset type: {0}")]
    UnsupportedAssetType(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<std::io::Error> for OptimizationError {
    fn from(err: std::io::Error) -> Self {
        OptimizationError::IoError(err.to_string())
    }
}
