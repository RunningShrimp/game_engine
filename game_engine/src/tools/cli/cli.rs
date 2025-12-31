//! # CLI Commands
//!
//! Command-line interface definitions for the game engine tool.

use crate::tools::cli::project_generator::{GeneratorError, ProjectGenerator};
use crate::tools::cli::template::{ProjectTemplate, TemplateRegistry};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
            Commands::Info {} => {
                self.cmd_info()?;
            }
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
                self.cmd_optimize(input, output, quality, platform, *no_lod, *no_compress, *no_shader_opt, *jobs)?;
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
        }

        Ok(())
    }

    /// Executes the 'new' command
    fn cmd_new(
        &self,
        name: &str,
        template: &Option<String>,
        output: &PathBuf,
        interactive: bool,
    ) -> Result<(), CliError> {
        println!("🎮 Creating new game project: {}", name);
        println!();

        let generator = ProjectGenerator::new();

        let template = if interactive {
            generator.generate_interactive(name, output)?;
            return Ok(());
        } else {
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
        println!("   cd {}", name);
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
                let metadata = registry
                    .get(name)
                    .ok_or_else(|| CliError::TemplateNotFound(name.clone()))?;

                println!("📦 Template: {}", metadata.name);
                println!();
                println!("Description: {}", metadata.description);
                println!("Version: {}", metadata.version);
                println!();
                println!("Categories:");
                for category in &metadata.categories {
                    println!("  - {}", category);
                }
                println!();
                println!("Tags:");
                for tag in &metadata.tags {
                    println!("  - {}", tag);
                }
                println!();
                println!("Required Features:");
                for feature in &metadata.required_features {
                    println!("  - {}", feature);
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
        println!("Templates available: {}", TemplateRegistry::new().list_all().len());
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
        use crate::tools::asset_pipeline::{PipelineConfig, Platform, QualityPreset, AssetPipeline};
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
                return Err(CliError::InvalidTemplate(format!("Invalid platform: {}", other)))
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
        let rt = Runtime::new().map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        rt.block_on(async {
            let pipeline = AssetPipeline::new(config);
            let report = pipeline.optimize_assets(input, output).await
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

    /// Executes the 'bundle' command
    fn cmd_bundle(
        &self,
        input: &PathBuf,
        output: &PathBuf,
        format: &str,
    ) -> Result<(), CliError> {
        use crate::tools::asset_pipeline::{AssetBundler, BundleFormat};

        println!("📦 Bundling assets...");
        println!();

        let bundler = AssetBundler::new();
        let bundle_format = match format.to_lowercase().as_str() {
            "pak" => BundleFormat::Pak,
            "loose" => BundleFormat::Loose,
            "virtual" => BundleFormat::Virtual,
            _ => return Err(CliError::InvalidTemplate(format!("Invalid bundle format: {}", format))),
        };

        println!("Input: {}", input.display());
        println!("Output: {}", output.display());
        println!("Format: {:?}", bundle_format);
        println!();
        println!("Note: Full bundling will be implemented in the next version.");
        println!("For now, this is a placeholder for the bundling feature.");

        Ok(())
    }
}

/// CLI error types
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Generator error: {0}")]
    GeneratorError(#[from] GeneratorError),

    #[error("Invalid template: {0}")]
    InvalidTemplate(String),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Project already initialized")]
    AlreadyInitialized,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
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
        let cli = GameEngineCli::try_parse_from([
            "game-engine",
            "new",
            "my-game",
            "--template",
            "basic",
        ]);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_template_list_command() {
        let cli = GameEngineCli::try_parse_from(["game-engine", "template", "list"]);
        assert!(cli.is_ok());
    }
}
