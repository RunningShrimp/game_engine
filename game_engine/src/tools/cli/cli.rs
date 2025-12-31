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
