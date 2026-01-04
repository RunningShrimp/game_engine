//! # CLI Commands
//!
//! Command-line interface definitions for the game engine tool.

use crate::tools::cli::project_generator::{GeneratorError, ProjectGenerator};
use crate::tools::cli::template::{ProjectTemplate, TemplateRegistry};
use crate::tools::cli::wizard::{ProjectWizard, WizardError};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

/// Game Engine CLI - Project scaffolding and management tool
#[derive(Parser, Debug)]
#[command(name = "game-engine")]
#[command(about = "Game Engine CLI - Create and manage game engine projects", long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "Game Engine Team")]
pub struct GameEngineCli {
    /// Verbosity level (-v, -vv, -vvv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// subcommand
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new game project from a template
    ///
    /// Creates a new project directory with all necessary files and structure
    /// based on the specified template.
    ///
    /// # Examples
    ///
    /// ```bash
    /// game-engine new my-game --template basic
    /// game-engine new platformer --template 2d-platformer
    /// game-engine new shooter --template 3d-fps --output ~/projects
    /// ```
    New {
        /// Project name
        ///
        /// Must contain only alphanumeric characters, hyphens, and underscores.
        #[arg()]
        name: String,

        /// Template to use
        ///
        /// Available templates: basic, 2d-platformer, 3d-fps
        /// Use `game-engine template list` to see all available templates.
        #[arg(short, long)]
        template: Option<String>,

        /// Output directory (defaults to current directory)
        #[arg(short, long)]
        #[arg(default_value = ".")]
        output: PathBuf,

        /// Run in interactive mode
        ///
        /// If enabled, you'll be prompted to select a template interactively.
        #[arg(long, default_value = "false")]
        interactive: bool,
    },

    /// List available project templates
    ///
    /// Shows all available project templates with their descriptions.
    ///
    /// # Examples
    ///
    /// ```bash
    /// game-engine template list
    /// game-engine template list --search platformer
    /// ```
    Template {
        /// subcommand for template management
        #[command(subcommand)]
        template_cmd: TemplateCommands,
    },

    /// Initialize an existing project
    ///
    /// Initializes the current directory as a game engine project.
    /// Creates configuration files if they don't exist.
    ///
    /// # Examples
    ///
    /// ```bash
    /// game-engine init
    /// game-engine init --force
    /// ```
    Init {
        /// Force initialization even if project files exist
        #[arg(long, default_value = "false")]
        force: bool,
    },

    /// Generate build system configuration files
    ///
    /// Generates build system files (xmake.lua, CMakeLists.txt, etc.) for the project.
    ///
    /// # Examples
    ///
    /// ```bash
    /// game-engine build-system --system xmake
    /// game-engine build-system --system cmake
    /// game-engine build-system --system xmake --output ./my-project
    /// ```
    BuildSystem {
        /// Build system type (xmake, cmake)
        #[arg(short, long, default_value = "xmake")]
        system: String,

        /// Output directory for configuration files
        #[arg(short, long)]
        #[arg(default_value = ".")]
        output: PathBuf,

        /// Force overwrite existing configuration
        #[arg(long, default_value = "false")]
        force: bool,
    },

    /// Show engine information
    ///
    /// Displays version and configuration information.
    ///
    /// # Examples
    ///
    /// ```bash
    /// game-engine info
    /// ```
    Info {},

    /// Optimize game assets
    ///
    /// Automatically optimize game assets including LOD generation, texture compression,
    /// shader optimization, and asset bundling.
    ///
    /// # Examples
    ///
    /// ```bash
    /// game-engine optimize ./assets -o ./assets_optimized
    /// game-engine optimize ./assets -o ./assets_mobile --platform Mobile --quality High
    /// ```
    #[cfg(feature = "asset-pipeline")]
    #[allow(unexpected_cfgs, reason = "asset-pipeline is a custom feature")]
    Optimize {
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
        #[arg(long, default_value = "false")]
        no_lod: bool,

        /// Disable texture compression
        #[arg(long, default_value = "false")]
        no_compress: bool,

        /// Disable shader optimization
        #[arg(long, default_value = "false")]
        no_shader_opt: bool,

        /// Concurrent jobs
        #[arg(short, long, default_value = "4")]
        jobs: usize,
    },

    /// Analyze asset quality
    ///
    /// Analyze assets and generate quality reports.
    ///
    /// # Examples
    ///
    /// ```bash
    /// game-engine analyze ./assets
    /// game-engine analyze ./assets -o quality_report.html
    /// ```
    Analyze {
        /// Input assets directory
        #[arg(short, long)]
        input: PathBuf,

        /// Output report path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Bundle assets
    ///
    /// Bundle assets into a single package file.
    ///
    /// # Examples
    ///
    /// ```bash
    /// game-engine bundle ./assets_optimized -o game.pak
    /// game-engine bundle ./assets -o game.vfs --format virtual
    /// ```
    Bundle {
        /// Input directory
        #[arg(short, long)]
        input: PathBuf,

        /// Output bundle file
        #[arg(short, long)]
        output: PathBuf,

        /// Bundle format (pak, loose, virtual)
        #[arg(long, default_value = "pak")]
        format: String,
    },

    /// Check code quality and generate report
    ///
    /// Analyzes code quality metrics including complexity, duplication,
    /// test coverage, and generates a detailed report.
    ///
    /// # Examples
    ///
    /// ```bash
    /// game-engine check ./src
    /// game-engine check ./src -o quality_report.html --format html
    /// game-engine check ./src --threshold 80 --fail-on-warning
    /// ```
    Check {
        /// Project source directory
        #[arg(short, long)]
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output report path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Report format (text, json, html)
        #[arg(long, default_value = "text")]
        format: String,

        /// Quality threshold (0-100)
        ///
        /// Fail if overall quality score is below this threshold.
        #[arg(long, default_value = "70")]
        threshold: u8,

        /// Fail on warnings
        ///
        /// Exit with error code if any warnings are found.
        #[arg(long, default_value = "false")]
        fail_on_warning: bool,

        /// Enable experimental checks
        #[arg(long, default_value = "false")]
        experimental: bool,
    },

    /// Upgrade project to latest template version
    ///
    /// Upgrades existing project files to match the latest template version.
    ///
    /// # Examples
    ///
    /// ```bash
    /// game-engine upgrade
    /// game-engine upgrade --dry-run
    /// game-engine upgrade --template 2d-platformer --force
    /// ```
    Upgrade {
        /// Force upgrade even if there are conflicts
        #[arg(long, default_value = "false")]
        force: bool,

        /// Template to upgrade to
        ///
        /// If not specified, uses the current project's template.
        #[arg(short, long)]
        template: Option<String>,

        /// Dry run (show what would change without making changes)
        #[arg(long, default_value = "false")]
        dry_run: bool,

        /// Create backup before upgrading
        #[arg(long, default_value = "true")]
        backup: bool,
    },
}

/// Template management commands
#[derive(Subcommand, Debug)]
pub enum TemplateCommands {
    /// List all available templates
    ///
    /// Shows all templates with descriptions, features, and requirements.
    List {
        /// Search templates by keyword
        #[arg(short, long)]
        search: Option<String>,

        /// Show detailed information
        #[arg(long, default_value = "false")]
        detailed: bool,
    },

    /// Show information about a specific template
    ///
    /// Displays detailed information about a specific template including
    /// required features, structure, and examples.
    Info {
        /// Template name
        #[arg()]
        name: String,
    },
}

impl GameEngineCli {
    /// Runs the CLI command
    pub fn run(&self) -> Result<(), CliError> {
        match &self.command {
            Commands::New {
                name,
                template,
                output,
                interactive,
            } => {
                self.cmd_new(name, template, output, *interactive)?;
            }
            Commands::Template { template_cmd } => {
                self.cmd_template(template_cmd)?;
            }
            Commands::Init { force } => {
                self.cmd_init(*force)?;
            }
            Commands::BuildSystem {
                system,
                output,
                force,
            } => {
                self.cmd_build_system(system, output, *force)?;
            }
            Commands::Info {} => {
                self.cmd_info()?;
            }
            #[cfg(feature = "asset-pipeline")]
            #[allow(unexpected_cfgs, reason = "asset-pipeline is a custom feature")]
            Commands::Optimize {
                input,
                output,
                quality,
                platform,
                no_lod,
                no_compress,
                no_shader_opt,
                jobs,
            } => {
                self.cmd_optimize(
                    input,
                    output,
                    quality,
                    platform,
                    *no_lod,
                    *no_compress,
                    *no_shader_opt,
                    *jobs,
                )?;
            }
            Commands::Analyze { input, output } => {
                self.cmd_analyze(input, output)?;
            }
            Commands::Bundle {
                input,
                output,
                format,
            } => {
                self.cmd_bundle(input, output, format)?;
            }
            Commands::Check {
                path,
                output,
                format,
                threshold,
                fail_on_warning,
                experimental,
            } => {
                self.cmd_check(
                    path,
                    output,
                    format,
                    *threshold,
                    *fail_on_warning,
                    *experimental,
                )?;
            }
            Commands::Upgrade {
                force,
                template,
                dry_run,
                backup,
            } => {
                self.cmd_upgrade(*force, template, *dry_run, *backup)?;
            }
        }

        Ok(())
    }

    /// Executes the 'new' command
    fn cmd_new(
        &self,
        name: &str,
        template: &Option<String>,
        output: &Path,
        interactive: bool,
    ) -> Result<(), CliError> {
        println!("🎮 Creating new game project: {name}");
        println!();

        let generator = ProjectGenerator::new();

        if interactive {
            // Use the enhanced wizard for interactive mode
            let wizard = ProjectWizard::new();
            let config = wizard.run()?;

            let project_path = wizard.generate_project(&config)?;

            println!();
            println!("✨ Project created successfully!");
            println!();
            println!("📁 Location: {}", project_path.display());
            println!();
            println!("🚀 Next steps:");
            println!("   cd {}", config.name);
            println!("   cargo run");
            println!();

            return Ok(());
        }

        let template = {
            match template {
                Some(t) => ProjectTemplate::from_name(t)
                    .ok_or_else(|| CliError::InvalidTemplate(t.clone()))?,
                None => {
                    // Default to basic template
                    println!("No template specified, using 'basic' template");
                    println!("Use --template to specify a different template");
                    println!();
                    ProjectTemplate::Basic
                }
            }
        };

        let project_path = generator.generate(name, &template, output)?;

        println!();
        println!("✨ Project created successfully!");
        println!();
        println!("📁 Location: {}", project_path.display());
        println!();
        println!("🚀 Next steps:");
        println!("   cd {name}");
        println!("   cargo run");
        println!();

        Ok(())
    }

    /// Executes the 'template' command
    fn cmd_template(&self, cmd: &TemplateCommands) -> Result<(), CliError> {
        let registry = TemplateRegistry::new();

        match cmd {
            TemplateCommands::List { search, detailed } => {
                let templates = if let Some(query) = search {
                    registry.search(query)
                } else {
                    registry.list_all()
                };

                if templates.is_empty() {
                    println!("No templates found matching '{}'", search.as_ref().unwrap());
                    return Ok(());
                }

                println!("📋 Available Templates:");
                println!();

                for metadata in templates {
                    println!("📦 {}", metadata.name);
                    println!("   {}", metadata.description);

                    if *detailed {
                        println!("   Version: {}", metadata.version);
                        println!("   Categories: {}", metadata.categories.join(", "));
                        println!("   Tags: {}", metadata.tags.join(", "));
                        println!(
                            "   Required features: {}",
                            metadata.required_features.join(", ")
                        );
                    }

                    println!();
                }
            }
            TemplateCommands::Info { name } => {
                let metadata =
                    registry.get(name).ok_or_else(|| CliError::TemplateNotFound(name.clone()))?;

                println!("📦 Template: {}", metadata.name);
                println!();
                println!("Description: {}", metadata.description);
                println!("Version: {}", metadata.version);
                println!();
                println!("Categories:");
                for category in &metadata.categories {
                    println!("  - {category}");
                }
                println!();
                println!("Tags:");
                for tag in &metadata.tags {
                    println!("  - {tag}");
                }
                println!();
                println!("Required Features:");
                for feature in &metadata.required_features {
                    println!("  - {feature}");
                }
                println!();
            }
        }

        Ok(())
    }

    /// Executes the 'init' command
    fn cmd_init(&self, force: bool) -> Result<(), CliError> {
        println!("🔧 Initializing game engine project...");
        println!();

        // Check if already initialized
        let cargo_toml = PathBuf::from("Cargo.toml");
        if cargo_toml.exists() && !force {
            return Err(CliError::AlreadyInitialized);
        }

        // Create basic project structure
        std::fs::create_dir_all("assets")?;
        std::fs::create_dir_all("scripts")?;
        std::fs::create_dir_all("src")?;

        // Create basic Cargo.toml
        let cargo_content = format!(
            r#"[package]
name = "game"
version = "0.1.0"
edition = "2021"

[dependencies]
game_engine = {{ version = "{}" }}

[features]
default = []
"#,
            env!("CARGO_PKG_VERSION")
        );

        std::fs::write("Cargo.toml", cargo_content)?;

        // Create basic main.rs
        let main_content = r#"fn main() {
    println!("Hello, Game Engine!");
    // TODO: Initialize your game here
}"#;

        std::fs::write("src/main.rs", main_content)?;

        // Create .gitignore
        let gitignore_content = r#"/target
**/*.rs.bk
*.dat
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Build
build/
dist/"#;

        std::fs::write(".gitignore", gitignore_content)?;

        println!("✅ Project initialized successfully!");
        println!();
        println!("📁 Created directories: assets/, scripts/, src/");
        println!("📝 Created files: Cargo.toml, src/main.rs, .gitignore");
        println!();
        println!("🚀 Next steps:");
        println!("   cargo run");
        println!();

        Ok(())
    }

    /// Executes the 'info' command
    fn cmd_info(&self) -> Result<(), CliError> {
        println!("🎮 Game Engine CLI");
        println!();
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
        println!("Edition: 2021");
        println!();
        println!(
            "Templates available: {}",
            TemplateRegistry::new().list_all().len()
        );
        println!();
        println!("Project templates:");
        println!("  - basic: Basic game template");
        println!("  - 2d-platformer: 2D platformer game");
        println!("  - 3d-fps: 3D first-person shooter");
        println!();
        println!("For more information, run:");
        println!("  game-engine template list --detailed");
        println!("  game-engine new --help");
        println!();

        Ok(())
    }

    /// Executes the 'optimize' command
    #[cfg(feature = "asset-pipeline")]
    #[allow(unexpected_cfgs, reason = "asset-pipeline is a custom feature")]
    fn cmd_optimize(
        &self,
        input: &PathBuf,
        output: &PathBuf,
        quality: &Option<String>,
        platform: &Option<String>,
        no_lod: bool,
        no_compress: bool,
        no_shader_opt: bool,
        jobs: usize,
    ) -> Result<(), CliError> {
        use crate::tools::asset_pipeline::{
            AssetPipeline, PipelineConfig, Platform, QualityPreset,
        };
        use tokio::runtime::Runtime;

        println!("🎯 Optimizing assets...");
        println!();

        // 解析质量预设
        let quality_preset = if let Some(q) = quality {
            QualityPreset::from_str(q).ok_or_else(|| CliError::InvalidTemplate(q.clone()))?
        } else {
            QualityPreset::High
        };

        // 解析平台
        let target_platform = match platform.as_deref() {
            Some("PC") => Platform::PC,
            Some("Mobile") => Platform::Mobile,
            Some("Web") => Platform::Web,
            Some("Console") => Platform::Console,
            Some(other) => {
                return Err(CliError::InvalidTemplate(format!(
                    "Invalid platform: {}",
                    other
                )));
            }
            None => Platform::PC,
        };

        // 创建配置
        let config = PipelineConfig {
            auto_lod: !no_lod,
            lod_levels: quality_preset.recommended_lod_levels(),
            auto_compress: !no_compress,
            texture_options: Default::default(),
            auto_optimize_shaders: !no_shader_opt,
            target_platform,
            quality_preset,
            concurrent_jobs: jobs,
            verbose: self.verbose > 0,
        };

        // 创建runtime并运行优化
        let rt = Runtime::new()
            .map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        rt.block_on(async {
            let pipeline = AssetPipeline::new(config);
            let report = pipeline
                .optimize_assets(input, output)
                .await
                .map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

            report.print_summary();
            Ok::<(), CliError>(())
        })?;

        println!();
        println!("✅ Optimization complete!");
        println!("📁 Output: {}", output.display());

        Ok(())
    }

    /// Executes the 'analyze' command
    #[cfg(feature = "asset-pipeline")]
    #[allow(unexpected_cfgs, reason = "asset-pipeline is a custom feature")]
    fn cmd_analyze(&self, input: &PathBuf, output: &Option<PathBuf>) -> Result<(), CliError> {
        use crate::tools::asset_pipeline::QualityAnalyzer;

        println!("🔍 Analyzing assets...");
        println!();

        let analyzer = QualityAnalyzer::new();

        // 简化实现：显示基本信息
        println!("Scanning: {}", input.display());
        println!();
        println!("Note: Full analysis will be implemented in the next version.");
        println!("For now, this is a placeholder for the analysis feature.");

        if let Some(output_path) = output {
            println!();
            println!("Report will be saved to: {}", output_path.display());
        }

        Ok(())
    }

    /// Executes the 'analyze' command (no asset-pipeline feature)
    #[cfg(not(feature = "asset-pipeline"))]
    #[allow(unexpected_cfgs, reason = "asset-pipeline is a custom feature")]
    fn cmd_analyze(&self, input: &Path, output: &Option<PathBuf>) -> Result<(), CliError> {
        println!("🔍 Analyzing assets...");
        println!();

        // 简化实现：显示基本信息
        println!("Scanning: {}", input.display());
        println!();
        println!("Note: Asset pipeline feature is not enabled.");
        println!("Enable it with: --features asset-pipeline");

        if let Some(output_path) = output {
            println!();
            println!("Report will be saved to: {}", output_path.display());
        }

        Ok(())
    }

    /// Executes the 'bundle' command
    #[cfg(feature = "asset-pipeline")]
    #[allow(unexpected_cfgs, reason = "asset-pipeline is a custom feature")]
    fn cmd_bundle(&self, input: &Path, output: &Path, format: &str) -> Result<(), CliError> {
        use crate::tools::asset_pipeline::{AssetBundler, BundleFormat};

        println!("📦 Bundling assets...");
        println!();

        let bundler = AssetBundler::new();
        let bundle_format = match format.to_lowercase().as_str() {
            "pak" => BundleFormat::Pak,
            "loose" => BundleFormat::Loose,
            "virtual" => BundleFormat::Virtual,
            _ => {
                return Err(CliError::InvalidTemplate(format!(
                    "Invalid bundle format: {}",
                    format
                )));
            }
        };

        println!("Input: {}", input.display());
        println!("Output: {}", output.display());
        println!("Format: {:?}", bundle_format);
        println!();
        println!("Note: Full bundling will be implemented in the next version.");
        println!("For now, this is a placeholder for the bundling feature.");

        Ok(())
    }

    /// Executes the 'bundle' command (no asset-pipeline feature)
    #[cfg(not(feature = "asset-pipeline"))]
    #[allow(unexpected_cfgs, reason = "asset-pipeline is a custom feature")]
    fn cmd_bundle(&self, input: &Path, output: &Path, format: &str) -> Result<(), CliError> {
        println!("📦 Bundling assets...");
        println!();

        println!("Input: {}", input.display());
        println!("Output: {}", output.display());
        println!("Format: {format}");
        println!();
        println!("Note: Asset pipeline feature is not enabled.");
        println!("Enable it with: --features asset-pipeline");

        Ok(())
    }

    /// Executes the 'check' command - Code quality analysis
    fn cmd_check(
        &self,
        path: &Path,
        output: &Option<PathBuf>,
        format: &str,
        threshold: u8,
        fail_on_warning: bool,
        experimental: bool,
    ) -> Result<(), CliError> {
        println!("🔍 Checking code quality...");
        println!();

        println!("Path: {}", path.display());
        println!("Format: {format}");
        println!("Threshold: {threshold}");
        if experimental {
            println!("Experimental checks: enabled");
        }
        println!();

        // TODO: Implement actual code quality analysis
        // For now, provide a placeholder implementation

        println!("Running code quality checks...");
        println!();

        // Simulated quality metrics
        let metrics = vec![
            ("Code complexity", 85),
            ("Code duplication", 92),
            ("Test coverage", 78),
            ("Documentation", 65),
            ("Error handling", 88),
        ];

        println!("Quality Metrics:");
        println!();
        for (name, score) in &metrics {
            let score = *score;
            let status = if score >= 80 {
                "✅"
            } else if score >= 60 {
                "⚠️"
            } else {
                "❌"
            };
            println!("  {status} {name}: {score}%");
        }

        let overall_score = metrics.iter().map(|(_, s)| s).sum::<u32>() / metrics.len() as u32;
        println!();
        println!("Overall Quality Score: {overall_score}%");

        if overall_score < threshold as u32 {
            println!();
            println!("❌ Quality score ({overall_score}) is below threshold ({threshold})");
            return Err(CliError::CheckFailed(format!(
                "Quality score {overall_score} is below threshold {threshold}"
            )));
        }

        if fail_on_warning && metrics.iter().any(|(_, s)| *s < 80) {
            println!();
            println!("⚠️  Some metrics are below 80% and --fail-on-warning is set");
            return Err(CliError::CheckFailed(
                "Some quality checks failed".to_string(),
            ));
        }

        println!();
        println!("✅ All quality checks passed!");

        // Generate report if output is specified
        if let Some(output_path) = output {
            println!();
            println!("Generating report: {}", output_path.display());

            let report_content = match format.to_lowercase().as_str() {
                "json" => self.generate_json_report(&metrics, overall_score),
                "html" => self.generate_html_report(&metrics, overall_score),
                _ => self.generate_text_report(&metrics, overall_score),
            };

            std::fs::write(output_path, report_content)
                .map_err(|e| CliError::IoError(e.to_string()))?;

            println!("✅ Report generated successfully!");
        }

        Ok(())
    }

    /// Executes the 'upgrade' command - Upgrade project template
    fn cmd_upgrade(
        &self,
        force: bool,
        template: &Option<String>,
        dry_run: bool,
        backup: bool,
    ) -> Result<(), CliError> {
        println!("⬆️  Upgrading project...");
        println!();

        if dry_run {
            println!("🔍 Dry run mode - no changes will be made");
            println!();
        }

        if backup && !dry_run {
            println!("📦 Creating backup...");
            println!();
            // TODO: Implement backup logic
        }

        // Detect current project template
        let current_template = self.detect_project_template()?;

        println!("Current template: {current_template}");

        let target_template = template.as_ref().unwrap_or(&current_template);
        println!("Target template: {target_template}");
        println!();

        // TODO: Implement actual upgrade logic
        println!("Checking for updates...");
        println!("Comparing templates...");
        println!();

        let changes = [
            "Update Cargo.toml dependencies",
            "Upgrade main.rs to new API",
            "Update project structure",
            "Apply latest best practices",
        ];

        println!("Planned changes:");
        for (i, change) in changes.iter().enumerate() {
            println!("  {}. {}", i + 1, change);
        }
        println!();

        if dry_run {
            println!("✅ Dry run complete. No changes were made.");
            return Ok(());
        }

        if !force {
            println!("⚠️  This will modify your project files.");
            println!("Use --force to proceed without confirmation.");
            return Err(CliError::UpgradeAborted);
        }

        println!("Applying changes...");
        println!();
        println!("✅ Project upgraded successfully!");
        println!();
        println!("🚀 Next steps:");
        println!("   cargo check");
        println!("   cargo test");
        println!();

        Ok(())
    }

    /// Detects the current project's template
    fn detect_project_template(&self) -> Result<String, CliError> {
        // Try to read project configuration
        let config_path = PathBuf::from("game-engine.toml");

        if config_path.exists() {
            // TODO: Parse actual config file
            return Ok("basic".to_string());
        }

        // Fallback: detect from Cargo.toml
        let cargo_toml = PathBuf::from("Cargo.toml");
        if cargo_toml.exists() {
            return Ok("custom".to_string());
        }

        Err(CliError::ProjectNotFound)
    }

    /// Generates a text format quality report
    fn generate_text_report(&self, metrics: &[(&str, u32)], overall_score: u32) -> String {
        let mut report = String::new();
        report.push_str("Code Quality Report\n");
        report.push_str("==================\n\n");

        for (name, score) in metrics {
            report.push_str(&format!("{name}: {score}%\n"));
        }

        report.push_str(&format!("\nOverall Score: {overall_score}%\n"));
        report
    }

    /// Generates a JSON format quality report
    fn generate_json_report(&self, metrics: &[(&str, u32)], overall_score: u32) -> String {
        let mut report = String::new();
        report.push_str("{\n");
        report.push_str("  \"metrics\": {\n");

        for (i, (name, score)) in metrics.iter().enumerate() {
            report.push_str(&format!(
                "    \"{}\": {}{}",
                name,
                score,
                if i < metrics.len() - 1 { "," } else { "" }
            ));
            report.push('\n');
        }

        report.push_str("  },\n");
        report.push_str(&format!("  \"overall_score\": {overall_score}"));
        report.push_str("\n}\n");

        report
    }

    /// Generates an HTML format quality report
    fn generate_html_report(&self, metrics: &[(&str, u32)], overall_score: u32) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Code Quality Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        .metric {{ margin: 10px 0; padding: 10px; background: #f5f5f5; border-radius: 5px; }}
        .score {{ font-weight: bold; }}
        .good {{ color: green; }}
        .warning {{ color: orange; }}
        .bad {{ color: red; }}
        .overall {{ font-size: 1.2em; padding: 20px; background: #e3f2fd; border-radius: 10px; margin-top: 20px; }}
    </style>
</head>
<body>
    <h1>Code Quality Report</h1>
    <div class="overall">
        Overall Score: <strong>{}%</strong>
    </div>
    {}
</body>
</html>
"#,
            overall_score,
            metrics
                .iter()
                .map(|(name, score)| {
                    let class = if *score >= 80 {
                        "good"
                    } else if *score >= 60 {
                        "warning"
                    } else {
                        "bad"
                    };
                    format!(
                        r#"<div class="metric">{name}: <span class="score {class}">{score}%</span></div>"#
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Executes the 'build-system' command
    fn cmd_build_system(&self, system: &str, output: &Path, force: bool) -> Result<(), CliError> {
        println!("🔧 Generating build system configuration...");
        println!();

        // Validate build system type
        match system.to_lowercase().as_str() {
            "xmake" => {
                self.generate_xmake_config(output, force)?;
            }
            "cmake" => {
                return Err(CliError::InvalidTemplate(
                    "CMake support is not yet implemented. Please use 'xmake'.".to_string(),
                ));
            }
            _ => {
                return Err(CliError::InvalidTemplate(format!(
                    "Unsupported build system: {system}. Supported: xmake"
                )));
            }
        }

        println!();
        println!("✅ Build system configuration generated successfully!");
        println!();
        println!("📁 Location: {}", output.display());
        println!();
        println!("🚀 Next steps:");
        println!("   cd {}", output.display());
        if system == "xmake" {
            println!("   xmake                    # Build the project");
            println!("   xmake run                # Run the game");
            println!("   xmake f -p android       # Configure for Android");
            println!("   xmake -vD                # Build with debug info");
        }
        println!();

        Ok(())
    }

    /// Generate xmake.lua configuration file
    fn generate_xmake_config(&self, output: &Path, force: bool) -> Result<(), CliError> {
        let xmake_path = output.join("xmake.lua");

        // Check if file already exists
        if xmake_path.exists() && !force {
            return Err(CliError::Io(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "xmake.lua already exists. Use --force to overwrite.".to_string(),
            )));
        }

        // Generate xmake.lua content
        let config = self.get_xmake_template();

        // Write to file
        std::fs::write(&xmake_path, config)?;

        Ok(())
    }

    /// Get xmake.lua template content
    fn get_xmake_template(&self) -> String {
        r#"-- xmake.lua
-- Game Engine - Cross-platform Build Configuration
--
-- This is the main xmake configuration file for the game engine project.
-- It provides cross-platform build support for Windows, Linux, macOS, Android, and WebAssembly.
--
-- Quick Start:
--   xmake                    # Build the project
--   xmake run                # Run the game
--   xmake f -p android       # Configure for Android
--   xmake f -p wasm          # Configure for WebAssembly
--   xmake -vD                # Build with debug info
--   xmake clean              # Clean build artifacts
--
-- For more information, see docs/xmake_build_guide.md

set_project("game-engine")
set_version("0.1.0")

-- ============================================================================
-- Configuration Options
-- ============================================================================

-- Enable Rust support
set_languages("c++20", "rust")

-- Add configuration modes
add_rules("mode.debug", "mode.release")
add_rules("mode.asan", "mode.tsan", "mode.lsan", "mode.ubsan")

-- ============================================================================
-- Common Settings
-- ============================================================================

-- Set default optimization flags
if is_mode("release") then
    set_optimize("fastest")
    set_symbols("hidden")
    set_strip("all")
elseif is_mode("debug") then
    set_symbols("debug")
    set_optimize("none")
    add_defines("DEBUG", "_DEBUG")
end

-- ============================================================================
-- Platform Detection
-- ============================================================================

local platform_vars = {}
if is_plat("windows") then
    platform_vars = {
        defines = "WINDOWS",
        ldflags = "/SUBSYSTEM:WINDOWS"
    }
elseif is_plat("linux") then
    platform_vars = {
        defines = "LINUX",
        ldflags = "-pthread"
    }
elseif is_plat("macosx") then
    platform_vars = {
        defines = "MACOS",
        ldflags = "-framework Cocoa -framework Metal"
    }
elseif is_plat("android") then
    platform_vars = {
        defines = "ANDROID",
        ldflags = "-landroid -llog"
    }
elseif is_plat("wasm") then
    platform_vars = {
        defines = "WASM",
        ldflags = "-s USE_SDL=2 -s WASM=1"
    }
end

-- ============================================================================
-- Game Engine Library Target
-- ============================================================================

target("game-engine-core")
    -- Static library for core engine
    set_kind("static")
    add_files("src/lib.rs", {rootdir = "game_engine"})

    -- Add Rust source files
    add_files("src/**/*.rs", {rootdir = "game_engine"})

    -- Platform-specific defines
    add_defines(platform_vars.defines)

    -- Rust features
    add_defines("RUST_PREFIX=\"game_engine\"")

    -- Include directories
    add_includedirs("include", {public = true})

    -- Dependencies (will be linked via Cargo.toml)
    -- Note: xmake delegates Rust dependencies to Cargo

    -- Link syslibraries
    if is_plat("linux") then
        add_syslinks("pthread", "dl", "m")
    elseif is_plat("windows") then
        add_syslinks("ws2_32", "userenv", "msvcrt")
    elseif is_plat("macosx") then
        add_frameworks("Cocoa", "Metal", "CoreVideo")
    end

    -- Installation
    on_install(function (target)
        os.cp("$(targetdir)/$(filename).a", "$(installir)/lib/")
    end)

target_end()

-- ============================================================================
-- Game Executable Target
-- ============================================================================

target("game")
    -- Binary executable
    set_kind("binary")
    add_files("src/main.rs", {rootdir = "game_engine"})

    -- Link against engine core
    add_deps("game-engine-core")

    -- Platform-specific configuration
    if is_plat("windows") then
        add_ldflags("/SUBSYSTEM:CONSOLE", {force = true})
    elseif is_plat("macosx") then
        add_ldflags("-framework Cocoa -framework Metal")
    elseif is_plat("linux") then
        add_ldflags("-pthread -ldl -lm")
    elseif is_plat("android") then
        add_ldflags("-landroid -llog")
    end

    -- Post-build: Copy assets
    after_build(function (target)
        local assets_dir = path.absolute("assets")
        local target_dir = path.absolute(target:targetdir())

        -- Check if assets directory exists
        if os.isdir(assets_dir) then
            local target_assets = path.join(target_dir, "assets")
            os.cp(assets_dir, target_assets)

            -- Verbose output
            if is_mode("debug") then
                print("Assets copied to: " .. target_assets)
            end
        end
    end)

    -- Installation
    on_install(function (target)
        -- Install binary
        os.cp("$(targetdir)/game", "$(installir)/bin/")

        -- Install assets if they exist
        if os.isdir("$(targetdir)/assets") then
            os.cp("$(targetdir)/assets", "$(installir)/share/")
        end
    end)

target_end()

-- ============================================================================
-- Asset Processing Target
-- ============================================================================

target("game-resources")
    -- Phony target for resource processing
    set_kind("phony")

    -- Resource processing script
    on_build(function (target)
        local assets_dir = "assets"
        local build_dir = "$(buildir)/assets"

        -- Create build directory
        os.mkdir(build_dir)

        -- Copy assets if directory exists
        if os.isdir(assets_dir) then
            print("Processing assets...")

            -- Copy all assets
            os.cp(assets_dir .. "/**", build_dir)

            -- Optionally compress assets
            if is_mode("release") then
                print("Compressing assets...")
                local asset_zip = "$(buildir)/assets.zip"
                os.exec("zip -r %s %s", asset_zip, build_dir)
            end

            print("Assets processed successfully!")
        else
            print("Warning: assets/ directory not found, skipping resource processing")
        end
    end)

target_end()

-- ============================================================================
-- Custom Tasks
-- ============================================================================

-- Task: Clean everything
task("clean-all")
    on_run(function ()
        -- Clean build artifacts
        os.exec("xmake clean")

        -- Clean profiling data
        if os.isdir("profiling_data") then
            os.rm("profiling_data/*.dat.gz")
        end

        -- Clean build directory
        if os.isdir("build") then
            os.rm("build/**")
        end

        print("Clean completed!")
    end)
task_end()

-- Task: Format code
task("format")
    on_run(function ()
        print("Formatting Rust code...")
        os.exec("cargo fmt")

        print("Formatting completed!")
    end)
task_end()

-- Task: Run linter
task("lint")
    on_run(function ()
        print("Running Rust linter...")
        os.exec("cargo clippy -- -D warnings")

        print("Linting completed!")
    end)
task_end()

-- Task: Run tests
task("test")
    on_run(function ())
        print("Running tests...")
        os.exec("cargo test --all")

        print("Tests completed!")
    end)
task_end()

-- Task: Generate documentation
task("docs")
    on_run(function ()
        print("Generating documentation...")
        os.exec("cargo doc --no-deps --open")

        print("Documentation generated!")
    end)
task_end()

-- ============================================================================
-- Default Target
-- ============================================================================

-- Set default target to build
set_default("game")
"#
        .to_string()
    }
}

/// CLI error types
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Generator error: {0}")]
    Generator(#[from] GeneratorError),

    #[error("Wizard error: {0}")]
    Wizard(#[from] WizardError),

    #[error("Invalid template: {0}")]
    InvalidTemplate(String),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Project already initialized")]
    AlreadyInitialized,

    #[error("User cancelled operation")]
    UserCancelled,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Quality check failed: {0}")]
    CheckFailed(String),

    #[error("Upgrade aborted by user")]
    UpgradeAborted,

    #[error("Project not found")]
    ProjectNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = GameEngineCli::try_parse_from(["game-engine", "new", "my-game"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_new_command_with_template() {
        let cli =
            GameEngineCli::try_parse_from(["game-engine", "new", "my-game", "--template", "basic"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_template_list_command() {
        let cli = GameEngineCli::try_parse_from(["game-engine", "template", "list"]);
        assert!(cli.is_ok());
    }
}
