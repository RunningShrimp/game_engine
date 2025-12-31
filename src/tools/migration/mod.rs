//! Unity migration tools
//!
//! Tools for migrating Unity projects to the game engine.

pub mod unity_parser;
pub mod asset_converter;
pub mod scene_migrator;
pub mod script_migration;

pub use unity_parser::{UnityProject, UnityParser};
pub use asset_converter::{AssetConverter, ConverterConfig};
pub use scene_migrator::{SceneMigrator, MigrationConfig, MigratedScene};
pub use script_migration::{ScriptMigrator, ScriptMigrationConfig};

use std::path::{Path, PathBuf};
use crate::error::{Error, Result};

/// Migration manager
pub struct MigrationManager {
    project_path: PathBuf,
    output_path: PathBuf,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(project_path: PathBuf, output_path: PathBuf) -> Self {
        Self {
            project_path,
            output_path,
        }
    }

    /// Run full migration
    pub fn migrate(&self) -> Result<MigrationReport> {
        println!("Starting Unity migration...");
        println!("Source: {}", self.project_path.display());
        println!("Output: {}", self.output_path.display());

        let mut report = MigrationReport {
            source_project: self.project_path.clone(),
            output_directory: self.output_path.clone(),
            scenes_migrated: 0,
            prefabs_migrated: 0,
            assets_converted: 0,
            scripts_migrated: 0,
            warnings: vec![],
            errors: vec![],
        };

        // Parse Unity project
        println!("\n1. Parsing Unity project...");
        let project = UnityParser::parse_project(&self.project_path)?;
        println!("   Found {} scenes", project.scenes.len());
        println!("   Found {} prefabs", project.prefabs.len());
        println!("   Unity version: {}", project.version);

        // Migrate scenes
        println!("\n2. Migrating scenes...");
        let scene_config = crate::tools::migration::scene_migrator::MigrationConfig {
            preserve_ids: false,
            component_mapping: crate::tools::migration::scene_migrator::ComponentMappingStrategy::Direct,
            output_dir: self.output_path.join("scenes"),
        };

        let mut scene_migrator = SceneMigrator::new(scene_config);

        for scene in &project.scenes {
            match scene_migrator.migrate_scene(scene) {
                Ok(_) => {
                    report.scenes_migrated += 1;
                    println!("   ✓ Migrated scene: {}", scene.name);
                }
                Err(e) => {
                    report.errors.push(format!("Failed to migrate scene {}: {}", scene.name, e));
                    println!("   ✗ Failed to migrate scene {}: {}", scene.name, e);
                }
            }
        }

        // Convert assets
        println!("\n3. Converting assets...");
        let asset_config = crate::tools::migration::asset_converter::ConverterConfig {
            output_dir: self.output_path.join("assets"),
            ..Default::default()
        };

        let converter = AssetConverter::new(asset_config);

        // Find assets
        let asset_files = self.find_asset_files(&project.assets_path)?;
        println!("   Found {} asset files", asset_files.len());

        let results = converter.convert_assets(&asset_files);
        for result in &results {
            if result.success {
                report.assets_converted += 1;
            } else {
                for error in &result.errors {
                    report.errors.push(error.clone());
                }
            }
        }

        println!("   Converted {} assets", report.assets_converted);

        // Migrate scripts
        println!("\n4. Migrating scripts...");
        let script_config = crate::tools::migration::script_migration::ScriptMigrationConfig {
            target_language: crate::tools::migration::script_migration::TargetLanguage::Lua,
            preserve_comments: true,
            generate_guide: true,
            output_dir: self.output_path.join("scripts"),
        };

        let script_migrator = ScriptMigrator::new(script_config);

        let script_files = self.find_script_files(&project.assets_path)?;
        println!("   Found {} script files", script_files.len());

        for script_file in &script_files {
            match script_migrator.migrate_script(script_file) {
                Ok(_) => {
                    report.scripts_migrated += 1;
                }
                Err(e) => {
                    report.errors.push(format!("Failed to migrate script {:?}: {}", script_file, e));
                }
            }
        }

        println!("   Migrated {} scripts", report.scripts_migrated);

        // Generate migration guide
        println!("\n5. Generating migration guide...");
        self.generate_migration_guide(&project, &report)?;

        println!("\n✓ Migration complete!");
        println!("  Scenes: {}", report.scenes_migrated);
        println!("  Assets: {}", report.assets_converted);
        println!("  Scripts: {}", report.scripts_migrated);

        if !report.errors.is_empty() {
            println!("\n⚠ Errors: {}", report.errors.len());
        }

        if !report.warnings.is_empty() {
            println!("\n⚠ Warnings: {}", report.warnings.len());
        }

        Ok(report)
    }

    /// Find all asset files
    fn find_asset_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        let entries = std::fs::read_dir(dir)
            .map_err(|e| Error::IoError(format!("Failed to read directory: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                files.extend(self.find_asset_files(&path)?);
            } else if let Some(ext) = path.extension() {
                match ext.to_str().unwrap_or("").to_lowercase().as_str() {
                    "png" | "jpg" | "jpeg" | "tga" | "psd" | "fbx" | "obj" | "wav" | "mp3"
                    | "ogg" | "mat" | "anim" => {
                        files.push(path);
                    }
                    _ => {}
                }
            }
        }

        Ok(files)
    }

    /// Find all script files
    fn find_script_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        let entries = std::fs::read_dir(dir)
            .map_err(|e| Error::IoError(format!("Failed to read directory: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                files.extend(self.find_script_files(&path)?);
            } else if path.extension().and_then(|e| e.to_str()) == Some("cs") {
                files.push(path);
            }
        }

        Ok(files)
    }

    /// Generate migration guide
    fn generate_migration_guide(
        &self,
        project: &UnityProject,
        report: &MigrationReport,
    ) -> Result<()> {
        let guide_path = self.output_path.join("MIGRATION_GUIDE.md");

        let mut guide = String::new();

        guide.push_str("# Unity Migration Guide\n\n");
        guide.push_str("This document provides guidance for the migrated Unity project.\n\n");

        guide.push_str("## Project Information\n\n");
        guide.push_str(&format!("- **Unity Version**: {}\n", project.version));
        guide.push_str(&format!("- **Source Path**: {}\n", project.project_path.display()));
        guide.push_str(&format!("- **Output Path**: {}\n", self.output_path.display()));
        guide.push_str("\n");

        guide.push_str("## Migration Summary\n\n");
        guide.push_str(&format!("- **Scenes Migrated**: {}\n", report.scenes_migrated));
        guide.push_str(&format!("- **Assets Converted**: {}\n", report.assets_converted));
        guide.push_str(&format!("- **Scripts Migrated**: {}\n", report.scripts_migrated));
        guide.push_str("\n");

        if !report.errors.is_empty() {
            guide.push_str("## Errors\n\n");
            for error in &report.errors {
                guide.push_str(&format!("- {}\n", error));
            }
            guide.push_str("\n");
        }

        if !report.warnings.is_empty() {
            guide.push_str("## Warnings\n\n");
            for warning in &report.warnings {
                guide.push_str(&format!("- {}\n", warning));
            }
            guide.push_str("\n");
        }

        guide.push_str("## Next Steps\n\n");
        guide.push_str("1. Review all migrated scenes\n");
        guide.push_str("2. Check asset conversions\n");
        guide.push_str("3. Review and test migrated scripts\n");
        guide.push_str("4. Adjust material parameters\n");
        guide.push_str("5. Test physics and colliders\n");
        guide.push_str("6. Verify lighting and cameras\n");
        guide.push_str("\n");

        guide.push_str("## Known Limitations\n\n");
        guide.push_str("- Shader conversion requires manual implementation\n");
        guide.push_str("- Complex animations may need adjustment\n");
        guide.push_str("- Physics simulation may differ\n");
        guide.push_str("- Custom editor scripts are not migrated\n");
        guide.push_str("- Third-party plugins need manual porting\n");
        guide.push_str("\n");

        std::fs::write(&guide_path, guide)
            .map_err(|e| Error::IoError(format!("Failed to write migration guide: {}", e)))?;

        println!("   Generated: {}", guide_path.display());

        Ok(())
    }
}

/// Migration report
#[derive(Debug, Clone)]
pub struct MigrationReport {
    pub source_project: PathBuf,
    pub output_directory: PathBuf,
    pub scenes_migrated: usize,
    pub prefabs_migrated: usize,
    pub assets_converted: usize,
    pub scripts_migrated: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}
