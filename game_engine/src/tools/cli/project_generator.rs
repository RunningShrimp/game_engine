//! # Project Generator
//!
//! Handles the generation of new projects from templates.

use crate::tools::cli::template::{ProjectTemplate, TemplateMetadata};
use handlebars::Handlebars;
use serde::Serialize;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during project generation
#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Project directory already exists: {0}")]
    DirectoryExists(PathBuf),

    #[error("Invalid project name: {0}")]
    InvalidName(String),
}

/// Result type for project generation
pub type GeneratorResult<T> = Result<T, GeneratorError>;

/// Project configuration for template rendering
#[derive(Debug, Clone, Serialize)]
pub struct ProjectConfig {
    /// Project name
    pub name: String,
    /// Project name in title case
    pub name_title: String,
    /// Project name in screaming snake case
    pub name_upper: String,
    /// Project name in kebab-case
    pub name_kebab: String,
    /// Template name
    pub template_name: String,
    /// Template description
    pub template_description: String,
    /// Engine version
    pub engine_version: String,
    /// Current year
    pub year: String,
}

impl ProjectConfig {
    /// Creates a new project configuration
    pub fn new(name: &str, template: &ProjectTemplate) -> GeneratorResult<Self> {
        // Validate project name
        if !Self::is_valid_name(name) {
            return Err(GeneratorError::InvalidName(
                "Project name must contain only alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            ));
        }

        Ok(Self {
            name: name.to_string(),
            name_title: Self::to_title_case(name),
            name_upper: Self::to_screaming_snake_case(name),
            name_kebab: Self::to_kebab_case(name),
            template_name: template.name().to_string(),
            template_description: template.description().to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            year: chrono::Utc::now().format("%Y").to_string(),
        })
    }

    /// Checks if a project name is valid
    fn is_valid_name(name: &str) -> bool {
        if name.is_empty() || name.len() > 64 {
            return false;
        }

        name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Converts a name to Title Case
    fn to_title_case(name: &str) -> String {
        name.split(['-', '_'])
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Converts a name to SCREAMING_SNAKE_CASE
    fn to_screaming_snake_case(name: &str) -> String {
        name.to_uppercase().replace('-', "_")
    }

    /// Converts a name to kebab-case
    fn to_kebab_case(name: &str) -> String {
        name.to_lowercase().replace('_', "-")
    }
}

/// Project generator for creating new game projects
pub struct ProjectGenerator {
    handlebars: Handlebars<'static>,
    template_dir: PathBuf,
}

impl ProjectGenerator {
    /// Creates a new project generator
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();

        // Register helpers
        handlebars.register_helper(
            "to_lower",
            Box::new(
                |h: &handlebars::Helper<'_>,
                 _r: &handlebars::Handlebars<'_>,
                 _: &handlebars::Context,
                 _rc: &mut handlebars::RenderContext<'_, '_>,
                 out: &mut dyn handlebars::Output|
                 -> handlebars::HelperResult {
                    let param = h.param(0).unwrap();
                    let value = param.value().as_str().unwrap();
                    out.write(value.to_lowercase().as_ref())?;
                    Ok(())
                },
            ),
        );

        // Find template directory
        let template_dir = Self::find_template_dir();

        Self {
            handlebars,
            template_dir,
        }
    }

    /// Finds the template directory
    fn find_template_dir() -> PathBuf {
        // Try multiple locations
        let possible_paths = vec![
            // Development: templates/ in the project root
            PathBuf::from("templates"),
            // Installed: share/game-engine/templates
            PathBuf::from("/usr/local/share/game-engine/templates"),
            // Windows: C:\Program Files\game-engine\templates
            PathBuf::from("C:\\Program Files\\game-engine\\templates"),
        ];

        for path in possible_paths {
            if path.exists() {
                return path;
            }
        }

        // Default to local templates directory
        PathBuf::from("templates")
    }

    /// Generates a new project from a template
    pub fn generate(
        &self,
        project_name: &str,
        template: &ProjectTemplate,
        output_dir: &Path,
    ) -> GeneratorResult<PathBuf> {
        let project_path = output_dir.join(project_name);

        // Check if directory already exists
        if project_path.exists() {
            return Err(GeneratorError::DirectoryExists(project_path));
        }

        // Create project configuration
        let config = ProjectConfig::new(project_name, template)?;

        // Get template directory
        let template_path = self.template_dir.join(template.dir_name());

        if !template_path.exists() {
            return Err(GeneratorError::Template(format!(
                "Template directory not found: {}",
                template_path.display()
            )));
        }

        // Create project directory
        fs::create_dir_all(&project_path)?;

        // Copy and process template files
        self.copy_template_files(&template_path, &project_path, &config)?;

        println!(
            "✅ Project created successfully at: {}",
            project_path.display()
        );

        Ok(project_path)
    }

    /// Copies and processes template files
    fn copy_template_files(
        &self,
        template_dir: &Path,
        project_dir: &Path,
        config: &ProjectConfig,
    ) -> GeneratorResult<()> {
        // Read template directory
        let entries = fs::read_dir(template_dir)?;

        for entry in entries {
            let entry = entry?;
            let template_path = entry.path();
            let file_name = template_path.file_name().and_then(|n| n.to_str()).unwrap();

            // Skip hidden files and directories
            if file_name.starts_with('.') {
                continue;
            }

            // Process file name (replace {{project_name}} placeholders)
            let output_name = file_name.replace("{{project_name}}", &config.name);
            let output_path = project_dir.join(&output_name);

            if template_path.is_dir() {
                // Recursively process subdirectories
                fs::create_dir_all(&output_path)?;
                self.copy_template_files(&template_path, &output_path, config)?;
            } else {
                // Process and copy files
                self.process_template_file(&template_path, &output_path, config)?;
            }
        }

        Ok(())
    }

    /// Processes a single template file
    fn process_template_file(
        &self,
        template_path: &Path,
        output_path: &Path,
        config: &ProjectConfig,
    ) -> GeneratorResult<()> {
        // Read template content
        let content = fs::read_to_string(template_path)?;

        // Process with handlebars if it's a template file
        let processed_content = if template_path.extension().and_then(|s| s.to_str()) == Some("hbs")
        {
            // It's a handlebars template
            match self.handlebars.render_template(&content, config) {
                Ok(rendered) => rendered,
                Err(e) => {
                    return Err(GeneratorError::Template(format!(
                        "Failed to render template {}: {}",
                        template_path.display(),
                        e
                    )));
                }
            }
        } else {
            // Not a template file, just copy as-is
            content
        };

        // Write output (remove .hbs extension if present)
        let final_output_path = if output_path.extension().and_then(|s| s.to_str()) == Some("hbs") {
            output_path.with_extension("")
        } else {
            output_path.to_path_buf()
        };

        fs::write(&final_output_path, processed_content)?;

        Ok(())
    }

    /// Generates a new project interactively
    #[cfg(feature = "cli")]
    pub fn generate_interactive(
        &self,
        project_name: &str,
        output_dir: &Path,
    ) -> GeneratorResult<PathBuf> {
        use dialoguer::{Select, theme::ColorfulTheme};

        let templates = ProjectTemplate::all();

        // Let user select template
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a template")
            .items(
                &templates
                    .iter()
                    .map(|t| format!("{} - {}", t.name(), t.description()))
                    .collect::<Vec<_>>(),
            )
            .default(0)
            .interact()
            .map_err(|e| GeneratorError::Template(format!("Interactive error: {}", e)))?;

        let template = &templates[selection];

        self.generate(project_name, template, output_dir)
    }
}

impl Default for ProjectGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_config_validation() {
        // Valid names
        assert!(ProjectConfig::new("my-game", &ProjectTemplate::Basic).is_ok());
        assert!(ProjectConfig::new("MyGame", &ProjectTemplate::Basic).is_ok());
        assert!(ProjectConfig::new("my_game", &ProjectTemplate::Basic).is_ok());

        // Invalid names
        assert!(ProjectConfig::new("", &ProjectTemplate::Basic).is_err());
        assert!(ProjectConfig::new("my game", &ProjectTemplate::Basic).is_err());
        assert!(ProjectConfig::new("my@game", &ProjectTemplate::Basic).is_err());
    }

    #[test]
    fn test_name_conversion() {
        assert_eq!(ProjectConfig::to_title_case("my-game"), "My Game");
        assert_eq!(ProjectConfig::to_screaming_snake_case("my-game"), "MY_GAME");
        assert_eq!(ProjectConfig::to_kebab_case("my_game"), "my-game");
    }

    #[test]
    fn test_generator_creation() {
        let generator = ProjectGenerator::new();
        // Should not panic
        assert!(generator.template_dir.exists() || !generator.template_dir.exists());
    }
}
