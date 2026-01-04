//! # Interactive Project Wizard
//!
//! Provides an interactive wizard for creating new game engine projects with
//! step-by-step configuration.

use crate::tools::cli::project_generator::{GeneratorError, ProjectGenerator};
use crate::tools::cli::template::{ProjectTemplate, TemplateRegistry};
use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during wizard execution
#[derive(Debug, Error)]
pub enum WizardError {
    #[error("Generator error: {0}")]
    Generator(#[from] GeneratorError),

    #[error("User cancelled wizard")]
    Cancelled,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result type for wizard operations
pub type WizardResult<T> = Result<T, WizardError>;

/// Project configuration collected from wizard
#[derive(Debug, Clone)]
pub struct WizardConfig {
    /// Project name
    pub name: String,
    /// Selected template
    pub template: ProjectTemplate,
    /// Output directory
    pub output_dir: PathBuf,
    /// Selected features
    pub features: Vec<String>,
    /// Scripting language preference
    pub scripting_language: Option<ScriptingLanguage>,
    /// Enable LSP support
    pub enable_lsp: bool,
    /// Enable debug UI
    pub enable_debug_ui: bool,
    /// Enable physics
    pub enable_physics: bool,
    /// Enable networking
    pub enable_networking: bool,
}

/// Supported scripting languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptingLanguage {
    /// JavaScript/TypeScript (QuickJS)
    JavaScript,
    /// Lua (mlua)
    Lua,
    /// Python (PyO3)
    Python,
    /// Rust (native)
    Rust,
}

impl ScriptingLanguage {
    /// Returns all available scripting languages
    pub fn all() -> Vec<Self> {
        vec![Self::JavaScript, Self::Lua, Self::Python, Self::Rust]
    }

    /// Returns the language name as a string
    pub fn name(&self) -> &str {
        match self {
            Self::JavaScript => "JavaScript/TypeScript",
            Self::Lua => "Lua",
            Self::Python => "Python",
            Self::Rust => "Rust (Native)",
        }
    }

    /// Returns the feature flag name
    pub fn feature_flag(&self) -> &str {
        match self {
            Self::JavaScript => "typescript",
            Self::Lua => "mlua",
            Self::Python => "pyo3",
            Self::Rust => "", // Native, no feature needed
        }
    }
}

/// Interactive project wizard
pub struct ProjectWizard {
    theme: ColorfulTheme,
}

impl ProjectWizard {
    /// Creates a new project wizard
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
        }
    }

    /// Runs the interactive wizard
    pub fn run(&self) -> WizardResult<WizardConfig> {
        println!("🎮 Game Engine Project Wizard");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        println!("Welcome! This wizard will help you create a new game project.");
        println!("You can cancel at any time by pressing Ctrl+C.");
        println!();

        // Step 1: Project name
        let name = self.ask_project_name()?;
        println!();

        // Step 2: Template selection
        let template = self.ask_template()?;
        println!();

        // Step 3: Output directory
        let output_dir = self.ask_output_directory()?;
        println!();

        // Step 4: Features
        let features = self.ask_features()?;
        println!();

        // Step 5: Scripting language
        let scripting_language = self.ask_scripting_language()?;
        println!();

        // Step 6: Additional options
        let (enable_lsp, enable_debug_ui, enable_physics, enable_networking) =
            self.ask_additional_options()?;
        println!();

        // Step 7: Confirmation
        let config = WizardConfig {
            name: name.clone(),
            template,
            output_dir,
            features,
            scripting_language,
            enable_lsp,
            enable_debug_ui,
            enable_physics,
            enable_networking,
        };

        self.show_summary(&config)?;
        println!();

        if !self.ask_confirmation("Create project with these settings?")? {
            return Err(WizardError::Cancelled);
        }

        Ok(config)
    }

    /// Asks for project name
    fn ask_project_name(&self) -> WizardResult<String> {
        let name: String = Input::with_theme(&self.theme)
            .with_prompt("Project name")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.is_empty() {
                    return Err("Project name cannot be empty");
                }
                if input.len() > 64 {
                    return Err("Project name must be 64 characters or less");
                }
                if !input.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                    return Err("Project name can only contain alphanumeric characters, hyphens, and underscores");
                }
                Ok(())
            })
            .interact_text()
            .map_err(|_| WizardError::Cancelled)?;

        Ok(name)
    }

    /// Asks for template selection
    fn ask_template(&self) -> WizardResult<ProjectTemplate> {
        let templates = ProjectTemplate::all();
        let registry = TemplateRegistry::new();

        let items: Vec<String> = templates
            .iter()
            .map(|t| {
                let metadata = registry.get(t.name()).unwrap();
                format!("{} - {}", t.name(), metadata.description)
            })
            .collect();

        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select a project template")
            .items(&items)
            .default(0)
            .interact()
            .map_err(|_| WizardError::Cancelled)?;

        Ok(templates[selection].clone())
    }

    /// Asks for output directory
    fn ask_output_directory(&self) -> WizardResult<PathBuf> {
        let default = PathBuf::from(".");

        let dir: String = Input::with_theme(&self.theme)
            .with_prompt("Output directory")
            .default(".".to_string())
            .validate_with(|input: &String| -> Result<(), &str> {
                let path = PathBuf::from(input);
                if !path.exists() {
                    return Err("Directory does not exist");
                }
                if !path.is_dir() {
                    return Err("Path is not a directory");
                }
                Ok(())
            })
            .interact_text()
            .map_err(|_| WizardError::Cancelled)?;

        Ok(PathBuf::from(dir))
    }

    /// Asks for feature selection
    fn ask_features(&self) -> WizardResult<Vec<String>> {
        let available_features = vec![
            ("lsp", "Language Server Protocol support"),
            ("debug-ui", "Debug UI and visualization tools"),
            ("physics", "Physics simulation (Rapier)"),
            ("networking", "Networking and multiplayer support"),
            ("ai", "AI and behavior tree systems"),
            ("audio", "3D spatial audio"),
            ("xr", "VR/AR support"),
        ];

        let items: Vec<String> = available_features
            .iter()
            .map(|(name, desc)| format!("{name} - {desc}"))
            .collect();

        let selections = MultiSelect::with_theme(&self.theme)
            .with_prompt("Select features to enable (space to select, enter to confirm)")
            .items(&items)
            .interact()
            .map_err(|_| WizardError::Cancelled)?;

        let features: Vec<String> =
            selections.iter().map(|&i| available_features[i].0.to_string()).collect();

        Ok(features)
    }

    /// Asks for scripting language preference
    fn ask_scripting_language(&self) -> WizardResult<Option<ScriptingLanguage>> {
        if !self.ask_confirmation("Do you want to enable scripting support?")? {
            return Ok(None);
        }

        let languages = ScriptingLanguage::all();
        let items: Vec<String> = languages.iter().map(|lang| lang.name().to_string()).collect();

        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select scripting language")
            .items(&items)
            .default(0)
            .interact()
            .map_err(|_| WizardError::Cancelled)?;

        Ok(Some(languages[selection]))
    }

    /// Asks for additional options
    fn ask_additional_options(&self) -> WizardResult<(bool, bool, bool, bool)> {
        let enable_lsp = self.ask_confirmation("Enable LSP server for IDE support?")?;
        let enable_debug_ui = self.ask_confirmation("Enable debug UI?")?;
        let enable_physics = self.ask_confirmation("Enable physics simulation?")?;
        let enable_networking = self.ask_confirmation("Enable networking support?")?;

        Ok((
            enable_lsp,
            enable_debug_ui,
            enable_physics,
            enable_networking,
        ))
    }

    /// Shows configuration summary
    fn show_summary(&self, config: &WizardConfig) -> WizardResult<()> {
        println!("📋 Project Configuration Summary");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Project name:        {}", config.name);
        println!("  Template:            {}", config.template.name());
        println!("  Output directory:    {}", config.output_dir.display());
        println!("  Features:            {}", config.features.join(", "));
        if let Some(lang) = config.scripting_language {
            println!("  Scripting language:  {}", lang.name());
        } else {
            println!("  Scripting language:  None");
        }
        println!(
            "  LSP support:         {}",
            if config.enable_lsp { "Yes" } else { "No" }
        );
        println!(
            "  Debug UI:            {}",
            if config.enable_debug_ui { "Yes" } else { "No" }
        );
        println!(
            "  Physics:             {}",
            if config.enable_physics { "Yes" } else { "No" }
        );
        println!(
            "  Networking:          {}",
            if config.enable_networking {
                "Yes"
            } else {
                "No"
            }
        );
        println!();

        Ok(())
    }

    /// Asks for confirmation
    fn ask_confirmation(&self, prompt: &str) -> WizardResult<bool> {
        Confirm::with_theme(&self.theme)
            .with_prompt(prompt)
            .default(true)
            .interact()
            .map_err(|_| WizardError::Cancelled)
    }

    /// Generates the project from wizard configuration
    pub fn generate_project(&self, config: &WizardConfig) -> WizardResult<PathBuf> {
        let generator = ProjectGenerator::new();
        let project_path =
            generator.generate(&config.name, &config.template, &config.output_dir)?;

        // TODO: Apply additional configuration (features, scripting language, etc.)
        // This would involve modifying the generated Cargo.toml and other files

        Ok(project_path)
    }
}

impl Default for ProjectWizard {
    fn default() -> Self {
        Self::new()
    }
}
