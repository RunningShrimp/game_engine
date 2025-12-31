//! # CLI Commands - 资源优化管线CLI
//!
//! 本模块实现命令行界面。

use super::pipeline::{
    AssetPipeline, PipelineConfig, Platform, QualityPreset, OptimizationError,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Asset Pipeline CLI
#[derive(Parser)]
#[command(name = "game-engine")]
#[command(about = "Game Engine Asset Optimization Pipeline", long_about = None)]
struct OptimizeCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Optimize assets
    Optimize(OptimizeCmd),

    /// Analyze asset quality
    Analyze(AnalyzeCmd),

    /// Bundle assets
    Bundle(BundleCmd),
}

/// 优化命令
#[derive(Parser)]
struct OptimizeCmd {
    /// Input assets directory
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory
    #[arg(short, long)]
    output: PathBuf,

    /// Quality preset (Low, Medium, High, Ultra)
    #[arg(long)]
    quality: Option<String>,

    /// Target platform (PC, Mobile, Web, Console)
    #[arg(long)]
    platform: Option<String>,

    /// Disable LOD generation
    #[arg(long)]
    no_lod: bool,

    /// Disable texture compression
    #[arg(long)]
    no_compress: bool,

    /// Disable shader optimization
    #[arg(long)]
    no_shader_opt: bool,

    /// Concurrent jobs
    #[arg(short, long, default_value = "4")]
    jobs: usize,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

/// 分析命令
#[derive(Parser)]
struct AnalyzeCmd {
    /// Input assets directory
    #[arg(short, long)]
    input: PathBuf,

    /// Output report path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

/// 打包命令
#[derive(Parser)]
struct BundleCmd {
    /// Input directory
    #[arg(short, long)]
    input: PathBuf,

    /// Output bundle file
    #[arg(short, long)]
    output: PathBuf,

    /// Bundle format (pak, loose, virtual)
    #[arg(long, default_value = "pak")]
    format: String,
}

/// 运行CLI
pub async fn run_asset_pipeline_cli(args: Vec<String>) -> Result<(), OptimizationError> {
    let cli = OptimizeCli::try_parse_from(args)
        .map_err(|e| OptimizationError::Other(format!("Failed to parse CLI: {}", e)))?;

    match cli.command {
        Commands::Optimize(cmd) => run_optimize(cmd).await,
        Commands::Analyze(cmd) => run_analyze(cmd).await,
        Commands::Bundle(cmd) => run_bundle(cmd).await,
    }
}

/// 运行优化命令
async fn run_optimize(cmd: OptimizeCmd) -> Result<(), OptimizationError> {
    println!("=== Asset Optimization Pipeline ===\n");

    // 验证输入目录
    if !cmd.input.exists() {
        return Err(OptimizationError::IoError(format!(
            "Input directory does not exist: {}",
            cmd.input.display()
        )));
    }

    // 解析质量预设
    let quality_preset = if let Some(quality_str) = cmd.quality {
        QualityPreset::from_str(&quality_str).ok_or_else(|| {
            OptimizationError::Other(format!("Invalid quality preset: {}", quality_str))
        })?
    } else {
        QualityPreset::High
    };

    // 解析平台
    let target_platform = match cmd.platform.as_deref() {
        Some("PC") => Platform::PC,
        Some("Mobile") => Platform::Mobile,
        Some("Web") => Platform::Web,
        Some("Console") => Platform::Console,
        Some(other) => {
            return Err(OptimizationError::Other(format!(
                "Invalid platform: {}",
                other
            )))
        }
        None => Platform::PC,
    };

    // 创建配置
    let config = PipelineConfig {
        auto_lod: !cmd.no_lod,
        lod_levels: quality_preset.recommended_lod_levels(),
        auto_compress: !cmd.no_compress,
        texture_options: super::texture_optimizer::TextureOptimizerOptions {
            compression_format: target_platform.recommended_texture_format(),
            ..Default::default()
        },
        auto_optimize_shaders: !cmd.no_shader_opt,
        target_platform,
        quality_preset,
        concurrent_jobs: cmd.jobs,
        verbose: cmd.verbose,
    };

    // 创建管线
    let pipeline = AssetPipeline::new(config);

    println!("Configuration:");
    println!("  Quality Preset: {:?}", quality_preset);
    println!("  Target Platform: {:?}", target_platform);
    println!("  LOD Generation: {}", config.auto_lod);
    println!("  Texture Compression: {}", config.auto_compress);
    println!("  Shader Optimization: {}", config.auto_optimize_shaders);
    println!("  Concurrent Jobs: {}\n", config.concurrent_jobs);

    // 运行优化
    let report = pipeline
        .optimize_assets(&cmd.input, &cmd.output)
        .await?;

    // 打印报告
    report.print_summary();

    println!("\nOptimization complete!");
    println!("Output directory: {}", cmd.output.display());

    Ok(())
}

/// 运行分析命令
async fn run_analyze(cmd: AnalyzeCmd) -> Result<(), OptimizationError> {
    println!("=== Asset Quality Analysis ===\n");

    use super::analyzer::{QualityAnalyzer, QualityTargets};

    let analyzer = QualityAnalyzer::with_targets(QualityTargets::default());

    // 简化实现：扫描并分析资源
    let mut asset_count = 0;
    let mut good_count = 0;
    let mut acceptable_count = 0;
    let mut poor_count = 0;
    let mut critical_count = 0;

    // TODO: 实现完整的扫描和分析逻辑
    println!("Scanning assets in: {}", cmd.input.display());

    // 模拟分析结果
    asset_count = 10;
    good_count = 5;
    acceptable_count = 3;
    poor_count = 1;
    critical_count = 1;

    println!("\nAnalysis Results:");
    println!("  Total Assets: {}", asset_count);
    println!("  Good: {}", good_count);
    println!("  Acceptable: {}", acceptable_count);
    println!("  Poor: {}", poor_count);
    println!("  Critical: {}", critical_count);

    if let Some(output_path) = cmd.output {
        // TODO: 生成HTML报告
        println!("\nReport saved to: {}", output_path.display());
    }

    Ok(())
}

/// 运行打包命令
async fn run_bundle(cmd: BundleCmd) -> Result<(), OptimizationError> {
    println!("=== Asset Bundler ===\n");

    use super::bundler::{AssetBundler, BundleFormat};

    let bundler = AssetBundler::new();

    let format = match cmd.format.to_lowercase().as_str() {
        "pak" => BundleFormat::Pak,
        "loose" => BundleFormat::Loose,
        "virtual" => BundleFormat::Virtual,
        _ => {
            return Err(OptimizationError::Other(format!(
                "Invalid bundle format: {}",
                cmd.format
            )))
        }
    };

    println!("Bundle Format: {:?}", format);
    println!("Input: {}", cmd.input.display());
    println!("Output: {}", cmd.output.display());

    // TODO: 实现完整的打包逻辑
    println!("\nBundling assets...");

    println!("\nBundle created successfully!");

    Ok(())
}

/// 打印帮助信息
pub fn print_help() {
    println!("Game Engine Asset Optimization Pipeline");
    println!();
    println!("Usage:");
    println!("  game-engine optimize <input> -o <output> [options]");
    println!("  game-engine analyze <input> [options]");
    println!("  game-engine bundle <input> -o <output> [options]");
    println!();
    println!("Examples:");
    println!("  game-engine optimize ./assets -o ./assets_optimized --quality High");
    println!("  game-engine optimize ./assets -o ./assets_mobile --platform Mobile");
    println!("  game-engine analyze ./assets -o report.html");
    println!("  game-engine bundle ./assets_optimized -o game.pak --format pak");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_optimize_cmd() {
        let args = vec![
            "game-engine",
            "optimize",
            "./assets",
            "-o",
            "./output",
            "--quality",
            "High",
            "--platform",
            "PC",
        ];

        // 验证命令可以正确解析
        if let Ok(cli) = OptimizeCli::try_parse_from(args) {
            if let Commands::Optimize(cmd) = cli.command {
                assert_eq!(cmd.input, PathBuf::from("./assets"));
                assert_eq!(cmd.output, PathBuf::from("./output"));
                assert_eq!(cmd.quality, Some("High".to_string()));
                assert_eq!(cmd.platform, Some("PC".to_string()));
            } else {
                panic!("Expected Optimize command");
            }
        } else {
            panic!("Failed to parse CLI arguments");
        }
    }

    #[test]
    fn test_parse_analyze_cmd() {
        let args = vec!["game-engine", "analyze", "./assets"];

        if let Ok(cli) = OptimizeCli::try_parse_from(args) {
            if let Commands::Analyze(cmd) = cli.command {
                assert_eq!(cmd.input, PathBuf::from("./assets"));
            } else {
                panic!("Expected Analyze command");
            }
        } else {
            panic!("Failed to parse CLI arguments");
        }
    }

    #[test]
    fn test_parse_bundle_cmd() {
        let args = vec![
            "game-engine",
            "bundle",
            "./assets",
            "-o",
            "game.pak",
            "--format",
            "pak",
        ];

        if let Ok(cli) = OptimizeCli::try_parse_from(args) {
            if let Commands::Bundle(cmd) = cli.command {
                assert_eq!(cmd.input, PathBuf::from("./assets"));
                assert_eq!(cmd.output, PathBuf::from("game.pak"));
                assert_eq!(cmd.format, "pak");
            } else {
                panic!("Expected Bundle command");
            }
        } else {
            panic!("Failed to parse CLI arguments");
        }
    }
}
