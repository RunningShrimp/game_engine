//! # Document Management for LSP
//!
//! Provides document caching and symbol indexing for the LSP server.

use crate::tools::lsp::registry::{ComponentDefinition, ResourceDefinition, SystemDefinition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Document cache entry
#[derive(Debug, Clone)]
pub struct DocumentCacheEntry {
    /// Document text
    pub text: String,

    /// Document version
    pub version: i32,

    /// Document language ID
    pub language_id: String,

    /// Last modification timestamp
    pub modified: std::time::Instant,
}

impl DocumentCacheEntry {
    /// Create a new document cache entry
    pub fn new(text: String, version: i32, language_id: String) -> Self {
        Self {
            text,
            version,
            language_id,
            modified: std::time::Instant::now(),
        }
    }

    /// Check if the entry is stale (older than specified duration)
    pub fn is_stale(&self, max_age: std::time::Duration) -> bool {
        self.modified.elapsed() > max_age
    }
}

/// Symbol location in source code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolLocation {
    /// File path (URI or local path)
    pub uri: String,

    /// Line number (0-based)
    pub line: u32,

    /// Character offset (0-based)
    pub character: u32,

    /// Symbol name
    pub name: String,

    /// Symbol kind
    pub kind: SymbolKind,
}

impl SymbolLocation {
    /// Create a new symbol location
    pub fn new(uri: String, line: u32, character: u32, name: String, kind: SymbolKind) -> Self {
        Self {
            uri,
            line,
            character,
            name,
            kind,
        }
    }

    /// Convert to LSP Location
    pub fn to_lsp_location(&self) -> tower_lsp::lsp_types::Location {
        tower_lsp::lsp_types::Location {
            uri: tower_lsp::lsp_types::Url::parse(&self.uri)
                .unwrap_or_else(|_| tower_lsp::lsp_types::Url::parse("file://unknown").unwrap()),
            range: tower_lsp::lsp_types::Range {
                start: tower_lsp::lsp_types::Position {
                    line: self.line,
                    character: self.character,
                },
                end: tower_lsp::lsp_types::Position {
                    line: self.line,
                    character: self.character + self.name.len() as u32,
                },
            },
        }
    }
}

/// Symbol kind for categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SymbolKind {
    /// Component
    Component,

    /// System
    System,

    /// Resource
    Resource,

    /// Function
    Function,

    /// Variable
    Variable,

    /// Struct
    Struct,

    /// Enum
    Enum,

    /// Constant
    Constant,

    /// Module
    Module,

    /// Unknown
    Unknown,
}

/// Symbol index for fast lookup
#[derive(Clone)]
pub struct SymbolIndex {
    /// Map from symbol name to locations
    pub(crate) index: Arc<RwLock<HashMap<String, Vec<SymbolLocation>>>>,

    /// Reverse index: file -> symbols
    pub(crate) files_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl SymbolIndex {
    /// Create a new symbol index
    pub fn new() -> Self {
        Self {
            index: Arc::new(RwLock::new(HashMap::new())),
            files_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Index engine API symbols from registry
    pub async fn index_engine_api(
        &self,
        registry: &crate::tools::lsp::registry::EngineAPIRegistry,
    ) {
        let mut index = self.index.write().await;

        // Index components
        let components = registry.list_components().await;
        for component_name in &components {
            if let Some(component) = registry.get_component(component_name).await {
                let location = SymbolLocation::new(
                    format!("file:///{}", component.module.replace("::", "/")),
                    0,
                    0,
                    component.name.clone(),
                    SymbolKind::Component,
                );
                index.entry(component.name.clone()).or_insert_with(Vec::new).push(location);
            }
        }

        // Index systems
        let systems = registry.list_systems().await;
        for system_name in &systems {
            if let Some(system) = registry.get_system(system_name).await {
                let location = SymbolLocation::new(
                    format!("file:///{}", system.module.replace("::", "/")),
                    0,
                    0,
                    system.name.clone(),
                    SymbolKind::System,
                );
                index.entry(system.name.clone()).or_insert_with(Vec::new).push(location);
            }
        }

        // Index resources
        let resources = registry.list_resources().await;
        for resource_name in &resources {
            if let Some(resource) = registry.get_resource(resource_name).await {
                let location = SymbolLocation::new(
                    format!("file:///{}", resource.module.replace("::", "/")),
                    0,
                    0,
                    resource.name.clone(),
                    SymbolKind::Resource,
                );
                index.entry(resource.name.clone()).or_insert_with(Vec::new).push(location);
            }
        }
    }

    /// Add a symbol location
    pub async fn add_symbol(&self, name: String, location: SymbolLocation) {
        let mut index = self.index.write().await;
        index.entry(name).or_insert_with(Vec::new).push(location);
    }

    /// Find symbol by name
    pub async fn find_symbol(&self, name: &str) -> Vec<SymbolLocation> {
        let index = self.index.read().await;
        index.get(name).cloned().unwrap_or_default()
    }

    /// Index symbols from a document
    pub async fn index_document(&self, uri: &str, text: &str, language_id: &str) {
        let mut files_index = self.files_index.write().await;

        // Clear old symbols for this file
        if let Some(old_symbols) = files_index.get(uri) {
            let mut index = self.index.write().await;
            for symbol_name in old_symbols {
                if let Some(locations) = index.get_mut(symbol_name) {
                    locations.retain(|loc| loc.uri != uri);
                }
            }
        }

        // Parse and index new symbols based on language
        let symbols = self.extract_symbols(text, language_id);

        // Collect symbol names before moving symbols
        let symbol_names: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();

        // Add to index
        let mut index = self.index.write().await;
        for symbol in symbols {
            index.entry(symbol.name.clone()).or_insert_with(Vec::new).push(symbol.clone());
        }

        // Update files index
        files_index.insert(uri.to_string(), symbol_names);
    }

    /// Extract symbols from document text (simplified implementation)
    fn extract_symbols(&self, text: &str, language_id: &str) -> Vec<SymbolLocation> {
        let mut symbols = Vec::new();

        match language_id {
            "rust" => {
                self.extract_rust_symbols(text, &mut symbols);
            }
            "lua" => {
                self.extract_lua_symbols(text, &mut symbols);
            }
            "typescript" | "javascript" => {
                self.extract_javascript_symbols(text, &mut symbols);
            }
            "python" => {
                self.extract_python_symbols(text, &mut symbols);
            }
            _ => {
                // Unknown language, skip extraction
            }
        }

        symbols
    }

    /// Extract symbols from Rust code (simplified regex-based)
    fn extract_rust_symbols(&self, text: &str, symbols: &mut Vec<SymbolLocation>) {
        let lines: Vec<&str> = text.lines().collect();
        let uri = "file:///current_file"; // Placeholder

        for (line_num, line) in lines.iter().enumerate() {
            // Extract struct definitions
            if let Some(captures) = Self::extract_pattern(line, r"struct\s+(\w+)") {
                for name in captures {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: name.clone(),
                        kind: SymbolKind::Struct,
                    });
                }
            }

            // Extract enum definitions
            if let Some(captures) = Self::extract_pattern(line, r"enum\s+(\w+)") {
                for name in captures {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: name.clone(),
                        kind: SymbolKind::Enum,
                    });
                }
            }

            // Extract function definitions
            if let Some(captures) = Self::extract_pattern(line, r"fn\s+(\w+)\(") {
                for name in captures {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: name.clone(),
                        kind: SymbolKind::Function,
                    });
                }
            }

            // Extract const definitions
            if let Some(captures) = Self::extract_pattern(line, r"const\s+(\w+)") {
                for name in captures {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: name.clone(),
                        kind: SymbolKind::Constant,
                    });
                }
            }
        }
    }

    /// Extract symbols from Lua code
    fn extract_lua_symbols(&self, text: &str, symbols: &mut Vec<SymbolLocation>) {
        let lines: Vec<&str> = text.lines().collect();
        let uri = "file:///current_file";

        for (line_num, line) in lines.iter().enumerate() {
            // Extract function definitions: function name(...)
            if let Some(captures) = Self::extract_pattern(line, r"function\s+(\w+)\(") {
                for name in captures {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: name.clone(),
                        kind: SymbolKind::Function,
                    });
                }
            }

            // Extract method definitions: function object:method(...)
            if line.contains("function") && line.contains(":") && line.contains("(") {
                if let Some(start) = line.find("function") {
                    let after_func = &line[start + 8..];
                    let parts: Vec<&str> = after_func.split(':').collect();
                    if parts.len() >= 2 {
                        let method_name = parts[1].split('(').next().unwrap_or("").trim();
                        if !method_name.is_empty() {
                            symbols.push(SymbolLocation {
                                uri: uri.to_string(),
                                line: line_num as u32,
                                character: 0,
                                name: method_name.to_string(),
                                kind: SymbolKind::Function,
                            });
                        }
                    }
                }
            }

            // Extract local variables: local name = ...
            if line.starts_with("local ") && line.contains("=") {
                let after_local = line[6..].trim();
                let var_name = after_local.split('=').next().unwrap_or("").trim();
                if !var_name.is_empty() && !var_name.contains('(') {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: var_name.to_string(),
                        kind: SymbolKind::Variable,
                    });
                }
            }

            // Extract module.table = function patterns
            if line.contains(".") && line.contains("=") && line.contains("function") {
                let parts: Vec<&str> = line.split('.').collect();
                if parts.len() >= 2 {
                    let after_dot = parts[1];
                    let method_name = after_dot.split('=').next().unwrap_or("").trim();
                    if !method_name.is_empty() {
                        symbols.push(SymbolLocation {
                            uri: uri.to_string(),
                            line: line_num as u32,
                            character: 0,
                            name: method_name.to_string(),
                            kind: SymbolKind::Function,
                        });
                    }
                }
            }
        }
    }

    /// Extract symbols from JavaScript/TypeScript code
    fn extract_javascript_symbols(&self, text: &str, symbols: &mut Vec<SymbolLocation>) {
        let lines: Vec<&str> = text.lines().collect();
        let uri = "file:///current_file";

        for (line_num, line) in lines.iter().enumerate() {
            // Extract function definitions: function name(...)
            if line.contains("function") && line.contains("(") {
                if let Some(start) = line.find("function") {
                    let after_func = &line[start + 8..].trim();
                    let func_name = after_func.split('(').next().unwrap_or("").trim();
                    if !func_name.is_empty() && func_name != "{" {
                        symbols.push(SymbolLocation {
                            uri: uri.to_string(),
                            line: line_num as u32,
                            character: 0,
                            name: func_name.to_string(),
                            kind: SymbolKind::Function,
                        });
                    }
                }
            }

            // Extract arrow functions with names: const name = (params) => { ... }
            if (line.contains("const") || line.contains("let") || line.contains("var"))
                && line.contains("=>")
            {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    // Find the variable name (after const/let/var)
                    for (i, part) in parts.iter().enumerate() {
                        if *part == "const" || *part == "let" || *part == "var" {
                            if i + 1 < parts.len() {
                                let var_name = parts[i + 1].trim_end_matches('=').trim();
                                if !var_name.is_empty() {
                                    symbols.push(SymbolLocation {
                                        uri: uri.to_string(),
                                        line: line_num as u32,
                                        character: 0,
                                        name: var_name.to_string(),
                                        kind: SymbolKind::Function,
                                    });
                                }
                            }
                            break;
                        }
                    }
                }
            }

            // Extract class definitions: class Name { ... }
            if line.starts_with("class ") && (line.contains("{") || line.contains("extends")) {
                let after_class = line[6..].trim();
                let class_name = after_class
                    .split('{')
                    .next()
                    .or_else(|| after_class.split("extends").next())
                    .unwrap_or("")
                    .trim();
                if !class_name.is_empty() {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: class_name.to_string(),
                        kind: SymbolKind::Struct,
                    });
                }
            }

            // Extract class methods: method_name(params) { ... }
            if line.contains("(") && line.contains("{") && !line.contains("function") {
                let before_paren = line.split('(').next().unwrap_or("");
                let parts: Vec<&str> = before_paren.split_whitespace().collect();
                if parts.len() >= 1 {
                    let method_name = parts.last().unwrap_or(&"").trim();
                    // Check if it looks like a method (not a call)
                    if !method_name.is_empty() && !line.contains("=") {
                        symbols.push(SymbolLocation {
                            uri: uri.to_string(),
                            line: line_num as u32,
                            character: 0,
                            name: method_name.to_string(),
                            kind: SymbolKind::Function,
                        });
                    }
                }
            }

            // Extract interface definitions (TypeScript): interface Name { ... }
            if line.starts_with("interface ") && line.contains("{") {
                let after_interface = line[10..].trim();
                let interface_name = after_interface.split('{').next().unwrap_or("").trim();
                if !interface_name.is_empty() {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: interface_name.to_string(),
                        kind: SymbolKind::Struct,
                    });
                }
            }

            // Extract type definitions (TypeScript): type Name = ...
            if line.starts_with("type ") && line.contains("=") {
                let after_type = line[5..].trim();
                let type_name = after_type.split('=').next().unwrap_or("").trim();
                if !type_name.is_empty() {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: type_name.to_string(),
                        kind: SymbolKind::Struct,
                    });
                }
            }

            // Extract const/let/var variables (not arrow functions)
            if (line.starts_with("const ") || line.starts_with("let ") || line.starts_with("var "))
                && line.contains("=")
                && !line.contains("=>")
            {
                let after_kw = if line.starts_with("const ") {
                    line[6..].trim()
                } else if line.starts_with("let ") {
                    line[4..].trim()
                } else {
                    line[4..].trim()
                };

                let var_name = after_kw.split('=').next().unwrap_or("").trim();
                if !var_name.is_empty() && !var_name.contains("(") {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: var_name.to_string(),
                        kind: SymbolKind::Variable,
                    });
                }
            }
        }
    }

    /// Extract symbols from Python code
    fn extract_python_symbols(&self, text: &str, symbols: &mut Vec<SymbolLocation>) {
        let lines: Vec<&str> = text.lines().collect();
        let uri = "file:///current_file";

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Extract class definitions: class Name:
            if trimmed.starts_with("class ") && trimmed.ends_with(':') {
                let after_class = trimmed[6..].trim();
                let class_name = after_class.split(':').next().unwrap_or("").trim();
                // Handle inheritance: class Name(Parent):
                let class_name = class_name.split('(').next().unwrap_or(class_name).trim();
                if !class_name.is_empty() {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: class_name.to_string(),
                        kind: SymbolKind::Struct,
                    });
                }
            }

            // Extract function definitions: def name(...):
            if trimmed.starts_with("def ") && trimmed.contains('(') && trimmed.contains("):") {
                let after_def = trimmed[4..].trim();
                let func_name = after_def.split('(').next().unwrap_or("").trim();
                if !func_name.is_empty() {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: func_name.to_string(),
                        kind: SymbolKind::Function,
                    });
                }
            }

            // Extract async function definitions: async def name(...):
            if trimmed.starts_with("async def ") && trimmed.contains('(') {
                let after_async_def = trimmed[10..].trim();
                let func_name = after_async_def.split('(').next().unwrap_or("").trim();
                if !func_name.is_empty() {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: func_name.to_string(),
                        kind: SymbolKind::Function,
                    });
                }
            }

            // Extract module-level variables (uppercase or simple assignments)
            if trimmed.contains('=') && !trimmed.starts_with('#') {
                // Skip function/class definitions
                if !trimmed.starts_with("def ")
                    && !trimmed.starts_with("class ")
                    && !trimmed.starts_with("async ")
                    && !trimmed.starts_with("if ")
                    && !trimmed.starts_with("for ")
                    && !trimmed.starts_with("while ")
                {
                    let var_name = trimmed.split('=').next().unwrap_or("").trim();
                    // Only capture if it looks like a simple variable (no dots or brackets)
                    if !var_name.is_empty() && !var_name.contains('.') && !var_name.contains('[') {
                        // Consider it a constant if it's uppercase
                        let kind = if var_name.chars().all(|c| c.is_uppercase() || c == '_') {
                            SymbolKind::Constant
                        } else {
                            SymbolKind::Variable
                        };

                        symbols.push(SymbolLocation {
                            uri: uri.to_string(),
                            line: line_num as u32,
                            character: 0,
                            name: var_name.to_string(),
                            kind,
                        });
                    }
                }
            }

            // Extract decorators: @decorator
            if trimmed.starts_with('@') {
                let decorator_name = trimmed[1..].trim();
                if !decorator_name.is_empty() {
                    symbols.push(SymbolLocation {
                        uri: uri.to_string(),
                        line: line_num as u32,
                        character: 0,
                        name: decorator_name.to_string(),
                        kind: SymbolKind::Function,
                    });
                }
            }
        }
    }

    /// Extract pattern using regex (simplified - avoiding full regex dependency)
    fn extract_pattern(line: &str, pattern: &str) -> Option<Vec<String>> {
        // Very simple pattern matching (for demonstration)
        // In production, use the regex crate
        if pattern.contains("struct ") && line.contains("struct ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(pos) = parts.iter().position(|p| *p == "struct") {
                if pos + 1 < parts.len() {
                    let name = parts[pos + 1].trim_end_matches('{').trim_end_matches('(');
                    return Some(vec![name.to_string()]);
                }
            }
        } else if pattern.contains("function ") && line.contains("function ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(pos) = parts.iter().position(|p| *p == "function") {
                if pos + 1 < parts.len() {
                    let name = parts[pos + 1].trim_end_matches('(');
                    return Some(vec![name.to_string()]);
                }
            }
        } else if pattern.contains("class ") && line.contains("class ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(pos) = parts.iter().position(|p| *p == "class") {
                if pos + 1 < parts.len() {
                    let name = parts[pos + 1].trim_end_matches('{');
                    return Some(vec![name.to_string()]);
                }
            }
        }

        None
    }

    /// Clear all symbols
    pub async fn clear(&self) {
        self.index.write().await.clear();
        self.files_index.write().await.clear();
    }

    /// Get statistics
    pub async fn stats(&self) -> (usize, usize) {
        let index = self.index.read().await;
        let files_index = self.files_index.read().await;
        (index.len(), files_index.len())
    }
}

impl Default for SymbolIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Document cache for storing open documents
#[derive(Clone)]
pub struct DocumentCache {
    /// Cache storage
    cache: Arc<RwLock<HashMap<String, DocumentCacheEntry>>>,

    /// Maximum cache size
    max_size: usize,

    /// Maximum entry age
    max_age: std::time::Duration,
}

impl DocumentCache {
    /// Create a new document cache
    pub fn new(max_size: usize, max_age: std::time::Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            max_age,
        }
    }

    /// Get document from cache
    pub async fn get(&self, uri: &str) -> Option<DocumentCacheEntry> {
        let cache = self.cache.read().await;
        let entry = cache.get(uri)?.clone();

        // Check if entry is stale
        if entry.is_stale(self.max_age) {
            return None;
        }

        Some(entry)
    }

    /// Put document in cache
    pub async fn put(&self, uri: String, entry: DocumentCacheEntry) {
        let mut cache = self.cache.write().await;

        // Evict old entries if cache is full
        if cache.len() >= self.max_size {
            // Find and remove stale entries
            let stale_keys: Vec<String> = cache
                .iter()
                .filter(|(_, entry)| entry.is_stale(self.max_age))
                .map(|(uri, _)| uri.clone())
                .collect();

            for key in stale_keys {
                cache.remove(&key);
            }

            // If still full, remove oldest entry
            if cache.len() >= self.max_size {
                if let Some(oldest) =
                    cache.iter().min_by_key(|(_, entry)| entry.modified).map(|(uri, _)| uri.clone())
                {
                    cache.remove(&oldest);
                }
            }
        }

        cache.insert(uri, entry);
    }

    /// Remove document from cache
    pub async fn remove(&self, uri: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(uri);
    }

    /// Clear all documents
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics
    pub async fn stats(&self) -> usize {
        self.cache.read().await.len()
    }
}

impl Default for DocumentCache {
    fn default() -> Self {
        Self::new(100, std::time::Duration::from_secs(300)) // 100 documents, 5 minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_location_creation() {
        let location = SymbolLocation::new(
            "file:///test.rs".to_string(),
            10,
            5,
            "TestStruct".to_string(),
            SymbolKind::Struct,
        );

        assert_eq!(location.line, 10);
        assert_eq!(location.character, 5);
        assert_eq!(location.name, "TestStruct");
    }

    #[test]
    fn test_symbol_index_new() {
        let index = SymbolIndex::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (symbol_count, file_count) = rt.block_on(index.stats());
        assert_eq!(symbol_count, 0);
        assert_eq!(file_count, 0);
    }

    #[test]
    fn test_symbol_index_add() {
        let index = SymbolIndex::new();
        let location = SymbolLocation::new(
            "file:///test.rs".to_string(),
            0,
            0,
            "test_symbol".to_string(),
            SymbolKind::Function,
        );

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            index.add_symbol("test_symbol".to_string(), location).await;

            let results = index.find_symbol("test_symbol").await;
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].name, "test_symbol");
        });
    }

    #[test]
    fn test_document_cache() {
        let cache = DocumentCache::new(10, std::time::Duration::from_secs(60));

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let entry = DocumentCacheEntry::new("test content".to_string(), 1, "rust".to_string());

            cache.put("file:///test.rs".to_string(), entry).await;

            let retrieved = cache.get("file:///test.rs").await;
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().text, "test content");
        });
    }
}
