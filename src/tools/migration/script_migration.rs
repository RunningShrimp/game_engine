//! Script migration
//!
//! Converts Unity C# scripts to Lua scripts for the engine.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::error::{Error, Result};

/// Script migration configuration
#[derive(Debug, Clone)]
pub struct ScriptMigrationConfig {
    /// Output language
    pub target_language: TargetLanguage,
    /// Whether to preserve comments
    pub preserve_comments: bool,
    /// Whether to generate migration guide
    pub generate_guide: bool,
    /// Output directory
    pub output_dir: PathBuf,
}

/// Target scripting language
#[derive(Debug, Clone, Copy)]
pub enum TargetLanguage {
    Lua,
    Rust,
}

/// API mapping
#[derive(Debug, Clone)]
pub struct ApiMapping {
    pub unity_api: String,
    pub engine_api: String,
    pub requires_manual_conversion: bool,
    pub notes: String,
}

/// Migrated script
#[derive(Debug, Clone)]
pub struct MigratedScript {
    pub original_path: PathBuf,
    pub output_path: PathBuf,
    pub code: String,
    pub warnings: Vec<String>,
    pub manual_changes_required: Vec<String>,
}

/// Script migrator
pub struct ScriptMigrator {
    config: ScriptMigrationConfig,
    api_mappings: Vec<ApiMapping>,
}

impl ScriptMigrator {
    /// Create a new script migrator
    pub fn new(config: ScriptMigrationConfig) -> Self {
        let mappings = Self::create_default_mappings();

        Self {
            config,
            api_mappings: mappings,
        }
    }

    /// Create default API mappings
    fn create_default_mappings() -> Vec<ApiMapping> {
        vec![
            ApiMapping {
                unity_api: "Transform.position".to_string(),
                engine_api: "entity:get_position()".to_string(),
                requires_manual_conversion: false,
                notes: "Position is now a method call".to_string(),
            },
            ApiMapping {
                unity_api: "Transform.rotation".to_string(),
                engine_api: "entity:get_rotation()".to_string(),
                requires_manual_conversion: false,
                notes: "Rotation is now a method call".to_string(),
            },
            ApiMapping {
                unity_api: "Rigidbody.AddForce".to_string(),
                engine_api: "rigidbody:apply_force()".to_string(),
                requires_manual_conversion: false,
                notes: "".to_string(),
            },
            ApiMapping {
                unity_api: "Debug.Log".to_string(),
                engine_api: "print".to_string(),
                requires_manual_conversion: false,
                notes: "".to_string(),
            },
            ApiMapping {
                unity_api: "GameObject.Find".to_string(),
                engine_api: "world:find_entity()".to_string(),
                requires_manual_conversion: true,
                notes: "Entity finding may require different approach".to_string(),
            },
            ApiMapping {
                unity_api: "Start".to_string(),
                engine_api: "on_start".to_string(),
                requires_manual_conversion: false,
                notes: "Lifecycle method name changed".to_string(),
            },
            ApiMapping {
                unity_api: "Update".to_string(),
                engine_api: "on_update".to_string(),
                requires_manual_conversion: false,
                notes: "Lifecycle method name changed".to_string(),
            },
            ApiMapping {
                unity_api: "FixedUpdate".to_string(),
                engine_api: "on_fixed_update".to_string(),
                requires_manual_conversion: false,
                notes: "Lifecycle method name changed".to_string(),
            },
            ApiMapping {
                unity_api: "OnCollisionEnter".to_string(),
                engine_api: "on_collision_enter".to_string(),
                requires_manual_conversion: true,
                notes: "Collision system differs".to_string(),
            },
            ApiMapping {
                unity_api: "Input.GetKeyDown".to_string(),
                engine_api: "input:is_key_pressed()".to_string(),
                requires_manual_conversion: true,
                notes: "Input system uses different key codes".to_string(),
            },
        ]
    }

    /// Migrate a C# script to Lua
    pub fn migrate_script(&self, source_path: &Path) -> Result<MigratedScript> {
        if !source_path.exists() {
            return Err(Error::IoError(format!(
                "Script file not found: {}",
                source_path.display()
            )));
        }

        println!("Migrating script: {}", source_path.display());

        let source_code = std::fs::read_to_string(source_path)
            .map_err(|e| Error::IoError(format!("Failed to read script: {}", e)))?;

        let output_code = match self.config.target_language {
            TargetLanguage::Lua => self.convert_csharp_to_lua(&source_code),
            TargetLanguage::Rust => self.convert_csharp_to_rust(&source_code),
        };

        let file_name = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        let output_path = self.config.output_dir.join(format!("{}.lua", file_name));

        // Create output directory
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(MigratedScript {
            original_path: source_path.to_path_buf(),
            output_path,
            code: output_code.code,
            warnings: output_code.warnings,
            manual_changes_required: output_code.manual_changes,
        })
    }

    /// Convert C# to Lua
    fn convert_csharp_to_lua(&self, code: &str) -> ConversionResult {
        let mut warnings = Vec::new();
        let mut manual_changes = Vec::new();
        let mut output = String::new();

        // Add header comment
        output.push_str("-- Auto-converted from C#\n");
        output.push_str("-- Manual review required\n\n");

        // Simple line-by-line conversion
        // In a real implementation, this would use proper AST parsing
        for line in code.lines() {
            let converted = if line.contains("class ") {
                // Class definition -> Lua table
                let class_name = extract_class_name(line);
                format!("local {} = {{}}\n", class_name)
            } else if line.contains("void Start(") {
                "function on_start(self)\n".to_string()
            } else if line.contains("void Update(") {
                "function on_update(self, dt)\n".to_string()
            } else if line.contains("void FixedUpdate(") {
                "function on_fixed_update(self, dt)\n".to_string()
            } else if line.contains("void OnCollisionEnter(") {
                "function on_collision_enter(self, other)\n".to_string()
            } else if line.contains("Debug.Log(") {
                let log_content = extract_function_arg(line, "Debug.Log");
                format!("print({})\n", log_content)
            } else if line.contains("//") {
                if self.config.preserve_comments {
                    format!("--{}\n", &line[line.find("//").unwrap() + 2..])
                } else {
                    String::new()
                }
            } else if line.trim().is_empty() {
                "\n".to_string()
            } else if line.contains("public ") {
                // Public field -> Lua table property
                format!("-- TODO: Convert field: {}\n", line.trim())
            } else {
                format!("-- TODO: {}\n", line.trim())
            };

            output.push_str(&converted);
        }

        // Add migration notes
        manual_changes.push("Review all TODO comments".to_string());
        manual_changes.push("Check variable declarations".to_string());
        manual_changes.push("Verify API mappings".to_string());

        ConversionResult {
            code: output,
            warnings,
            manual_changes,
        }
    }

    /// Convert C# to Rust
    fn convert_csharp_to_rust(&self, code: &str) -> ConversionResult {
        let mut warnings = Vec::new();
        let mut manual_changes = Vec::new();
        let mut output = String::new();

        // Add header comment
        output.push_str("// Auto-converted from C#\n");
        output.push_str("// Manual review required\n\n");

        // Simple line-by-line conversion
        for line in code.lines() {
            let converted = if line.contains("class ") {
                let class_name = extract_class_name(line);
                format!("pub struct {};\n", class_name)
            } else if line.contains("void Start(") {
                "pub fn on_start(&mut self) {\n".to_string()
            } else if line.contains("void Update(") {
                "pub fn on_update(&mut self, dt: f32) {\n".to_string()
            } else if line.contains("Debug.Log(") {
                let log_content = extract_function_arg(line, "Debug.Log");
                format!("println!(\"{}\", {});\n", log_content, log_content)
            } else if line.contains("//") {
                if self.config.preserve_comments {
                    format!("//{}\n", &line[line.find("//").unwrap() + 2..])
                } else {
                    String::new()
                }
            } else {
                format!("// TODO: {}\n", line.trim())
            };

            output.push_str(&converted);
        }

        manual_changes.push("Add proper type annotations".to_string());
        manual_changes.push("Implement trait requirements".to_string());
        manual_changes.push("Handle memory management".to_string());

        ConversionResult {
            code: output,
            warnings,
            manual_changes,
        }
    }

    /// Generate migration guide
    pub fn generate_migration_guide(&self, scripts: &[MigratedScript]) -> String {
        let mut guide = String::new();

        guide.push_str("# Unity Script Migration Guide\n\n");
        guide.push_str("This document provides guidance for manually reviewing migrated scripts.\n\n");

        guide.push_str("## General Notes\n\n");
        guide.push_str("- All scripts require manual review\n");
        guide.push_str("- Type systems differ significantly\n");
        guide.push_str("- API mappings may not be 1:1\n");
        guide.push_str("- Test thoroughly after conversion\n\n");

        guide.push_str("## Common API Mappings\n\n");
        for mapping in &self.api_mappings {
            guide.push_str(&format!(
                "### `{}` → `{}`\n\n",
                mapping.unity_api, mapping.engine_api
            ));
            if !mapping.notes.is_empty() {
                guide.push_str(&format!("**Note:** {}\n\n", mapping.notes));
            }
            if mapping.requires_manual_conversion {
                guide.push_str("**⚠️ Manual conversion required**\n\n");
            }
        }

        guide.push_str("## Scripts Requiring Attention\n\n");
        for script in scripts {
            if !script.manual_changes_required.is_empty() {
                guide.push_str(&format!("### {}\n\n", script.original_path.display()));
                for change in &script.manual_changes_required {
                    guide.push_str(&format!("- {}\n", change));
                }
                guide.push_str("\n");
            }
        }

        guide
    }

    /// Get API mappings for a specific Unity API
    pub fn get_api_mapping(&self, unity_api: &str) -> Option<&ApiMapping> {
        self.api_mappings
            .iter()
            .find(|m| unity_api.contains(&m.unity_api))
    }
}

/// Conversion result
struct ConversionResult {
    code: String,
    warnings: Vec<String>,
    manual_changes: Vec<String>,
}

/// Extract class name from C# class declaration
fn extract_class_name(line: &str) -> &str {
    if let Some(start) = line.find("class ") {
        let after_class = &line[start + 6..];
        if let Some(end) = after_class.find(&[':', '{'][..]) {
            return after_class[..end].trim();
        }
    }
    "Unknown"
}

/// Extract function argument
fn extract_function_arg(line: &str, function_name: &str) -> &str {
    if let Some(start) = line.find(function_name) {
        let after_func = &line[start + function_name.len()..];
        if let Some(arg_start) = after_func.find('(') {
            if let Some(arg_end) = after_func.find(')') {
                return after_func[arg_start + 1..arg_end].trim();
            }
        }
    }
    ""
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_migrator() {
        let config = ScriptMigrationConfig {
            target_language: TargetLanguage::Lua,
            preserve_comments: true,
            generate_guide: true,
            output_dir: PathBuf::from("/tmp"),
        };

        let migrator = ScriptMigrator::new(config);

        // Test API mapping lookup
        let mapping = migrator.get_api_mapping("Transform.position");
        assert!(mapping.is_some());
    }
}
