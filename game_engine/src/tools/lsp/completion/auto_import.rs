//! # Auto Import Manager
//!
//! Automatically manages use statements for imported types.

use std::collections::{HashMap, HashSet};
use tower_lsp::lsp_types::Url;

/// Auto import manager
pub struct AutoImportManager {
    /// Imported modules for each file
    imported_modules: HashMap<String, HashSet<String>>,

    /// Available types and their import paths
    available_types: HashMap<String, String>,

    /// Standard library types
    std_types: HashMap<String, String>,
}

impl AutoImportManager {
    /// Create a new auto import manager
    pub fn new() -> Self {
        let mut std_types = HashMap::new();

        // Populate standard library types
        std_types.insert("Vec".to_string(), "std::vec::Vec".to_string());
        std_types.insert(
            "HashMap".to_string(),
            "std::collections::HashMap".to_string(),
        );
        std_types.insert(
            "HashSet".to_string(),
            "std::collections::HashSet".to_string(),
        );
        std_types.insert("String".to_string(), "std::string::String".to_string());
        std_types.insert("Option".to_string(), "std::option::Option".to_string());
        std_types.insert("Result".to_string(), "std::result::Result".to_string());
        std_types.insert("Box".to_string(), "std::boxed::Box".to_string());
        std_types.insert("Rc".to_string(), "std::rc::Rc".to_string());
        std_types.insert("Arc".to_string(), "std::sync::Arc".to_string());
        std_types.insert("Mutex".to_string(), "std::sync::Mutex".to_string());
        std_types.insert("RwLock".to_string(), "std::sync::RwLock".to_string());

        Self {
            imported_modules: HashMap::new(),
            available_types: HashMap::new(),
            std_types,
        }
    }

    /// Get the import path for a type
    ///
    /// # Arguments
    ///
    /// * `type_name` - The type name
    ///
    /// # Returns
    ///
    /// The import path, if available
    pub fn get_import_path(&self, type_name: &str) -> Option<String> {
        // Check standard library types first
        if let Some(path) = self.std_types.get(type_name) {
            return Some(path.clone());
        }

        // Check project-specific types
        self.available_types.get(type_name).cloned()
    }

    /// Generate a use statement for a type
    ///
    /// # Arguments
    ///
    /// * `type_name` - The type name
    ///
    /// # Returns
    ///
    /// The use statement, if available
    pub fn generate_use_statement(&self, type_name: &str) -> Option<String> {
        let import_path = self.get_import_path(type_name)?;
        Some(format!("use {};", import_path))
    }

    /// Check if a type is already imported in a file
    ///
    /// # Arguments
    ///
    /// * `file_path` - The file path
    /// * `type_name` - The type name
    ///
    /// # Returns
    ///
    /// True if the type is already imported
    pub fn is_imported(&self, file_path: &str, type_name: &str) -> bool {
        if let Some(imported) = self.imported_modules.get(file_path) {
            imported.contains(type_name)
        } else {
            false
        }
    }

    /// Add an imported module to a file
    ///
    /// # Arguments
    ///
    /// * `file_path` - The file path
    /// * `module_path` - The module path
    pub fn add_import(&mut self, file_path: String, module_path: String) {
        self.imported_modules
            .entry(file_path)
            .or_insert_with(HashSet::new)
            .insert(module_path);
    }

    /// Register an available type
    ///
    /// # Arguments
    ///
    /// * `type_name` - The type name
    /// * `import_path` - The import path
    pub fn register_type(&mut self, type_name: String, import_path: String) {
        self.available_types.insert(type_name, import_path);
    }

    /// Suggest auto-import for a type
    ///
    /// # Arguments
    ///
    /// * `file_path` - The file path
    /// * `type_name` - The type name
    ///
    /// # Returns
    ///
    /// The use statement to insert, if the type needs to be imported
    pub fn suggest_import(&self, file_path: &str, type_name: &str) -> Option<String> {
        if self.is_imported(file_path, type_name) {
            None
        } else {
            self.generate_use_statement(type_name)
        }
    }
}

impl Default for AutoImportManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_import_manager_creation() {
        let manager = AutoImportManager::new();
        assert!(!manager.std_types.is_empty());
    }

    #[test]
    fn test_get_import_path_std_type() {
        let manager = AutoImportManager::new();
        let path = manager.get_import_path("Vec");
        assert_eq!(path, Some("std::vec::Vec".to_string()));
    }

    #[test]
    fn test_generate_use_statement() {
        let manager = AutoImportManager::new();
        let use_stmt = manager.generate_use_statement("Vec");
        assert_eq!(use_stmt, Some("use std::vec::Vec;".to_string()));
    }

    #[test]
    fn test_register_custom_type() {
        let mut manager = AutoImportManager::new();
        manager.register_type("MyType".to_string(), "my_module::MyType".to_string());
        let path = manager.get_import_path("MyType");
        assert_eq!(path, Some("my_module::MyType".to_string()));
    }

    #[test]
    fn test_is_imported() {
        let mut manager = AutoImportManager::new();
        assert!(!manager.is_imported("test.rs", "Vec"));

        manager.add_import("test.rs".to_string(), "std::vec::Vec".to_string());
        assert!(manager.is_imported("test.rs", "Vec"));
    }

    #[test]
    fn test_suggest_import() {
        let manager = AutoImportManager::new();
        let suggestion = manager.suggest_import("test.rs", "Vec");
        assert_eq!(suggestion, Some("use std::vec::Vec;".to_string()));
    }
}
