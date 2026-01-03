//! API Index for Game Engine Symbols
//!
//! Scans and indexes all game engine types, functions, and methods
//! to provide fast lookup for code intelligence features.

use std::collections::HashMap;
use std::path::PathBuf;
use tower_lsp::lsp_types::*;
use tracing::{debug, info, warn};

use crate::lsp::symbol_info::SymbolInfo;

/// API Index storing all game engine symbols
pub struct ApiIndex {
    /// Map of file URI to symbols in that file
    files: HashMap<String, Vec<SymbolInfo>>,

    /// Map of symbol name to locations (for global search)
    symbol_locations: HashMap<String, Vec<Location>>,

    /// Map of symbol name to definition (for go-to-definition)
    definitions: HashMap<String, Location>,

    /// Map of position to symbol (for hover and references)
    position_to_symbol: HashMap<String, HashMap<Position, SymbolInfo>>,
}

impl ApiIndex {
    /// Create a new empty API index
    pub fn new() -> Self {
        info!("Creating new ApiIndex");
        Self {
            files: HashMap::new(),
            symbol_locations: HashMap::new(),
            definitions: HashMap::new(),
            position_to_symbol: HashMap::new(),
        }
    }

    /// Build the API index by scanning the codebase
    pub fn build(&mut self) -> anyhow::Result<()> {
        info!("Building API index...");

        // 代码库扫描（简化版本）
        // For now, we'll create a minimal index with game engine core types

        // Add core game engine symbols
        self.add_core_symbols();

        info!(
            "API index built with {} symbols",
            self.symbol_locations.len()
        );

        Ok(())
    }

    /// Add core game engine symbols to the index
    fn add_core_symbols(&mut self) {
        // This is a placeholder - in the real implementation,
        // we would parse all Rust files in the game_engine crate

        // Example: Entity struct
        let entity_symbol = SymbolInfo {
            name: "Entity".to_string(),
            kind: SymbolKind::STRUCT,
            documentation: "Represents a game entity with a unique ID.\n\n# Examples\n\n```rust\nlet entity = Entity::new();\n```".to_string(),
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 20),
            },
            selection_range: Range {
                start: Position::new(0, 7),
                end: Position::new(0, 12),
            },
            children: None,
        };

        // Example: Transform component
        let transform_symbol = SymbolInfo {
            name: "Transform".to_string(),
            kind: SymbolKind::STRUCT,
            documentation: "Transform component for position, rotation, and scale.\n\n# Fields\n\n- `position`: Vec3 - Position in world space\n- `rotation`: Quat - Rotation as quaternion\n- `scale`: Vec3 - Scale factors".to_string(),
            range: Range {
                start: Position::new(10, 0),
                end: Position::new(15, 0),
            },
            selection_range: Range {
                start: Position::new(10, 10),
                end: Position::new(10, 18),
            },
            children: None,
        };

        // Add to index (using dummy URIs for now)
        let dummy_uri = "file:///game_engine/src/entity.rs".to_string();
        self.files.insert(dummy_uri.clone(), vec![entity_symbol, transform_symbol]);

        // 从代码库添加更多符号（简化版本）
    }

    /// Find a symbol at a specific position in a file
    pub async fn find_symbol_at_position(
        &self,
        uri: &str,
        position: Position,
    ) -> Option<SymbolInfo> {
        if let Some(file_symbols) = self.position_to_symbol.get(uri) {
            file_symbols.get(&position).cloned()
        } else {
            None
        }
    }

    /// Find the definition of a symbol at a position
    pub async fn find_definition(
        &self,
        uri: &str,
        position: Position,
    ) -> Option<Location> {
        // Find the symbol at the position
        if let Some(symbol) = self.find_symbol_at_position(uri, position).await {
            // Look up its definition
            self.definitions.get(&symbol.name).cloned()
        } else {
            None
        }
    }

    /// Get all symbols in a document
    pub async fn document_symbols(&self, uri: &str) -> anyhow::Result<Vec<DocumentSymbol>> {
        if let Some(symbols) = self.files.get(uri) {
            Ok(symbols
                .iter()
                .map(|s| DocumentSymbol {
                    name: s.name.clone(),
                    kind: s.kind,
                    range: s.range,
                    selection_range: s.selection_range,
                    detail: None,
                    tags: None,
                    deprecated: None,
                    children: None,
                })
                .collect())
        } else {
            Ok(vec![])
        }
    }

    /// Search for symbols matching a query
    pub async fn search_symbols(&self, query: &str) -> anyhow::Result<Vec<SymbolInformation>> {
        let mut results = Vec::new();

        for (name, locations) in &self.symbol_locations {
            if name.contains(query) {
                for location in &locations[0..1.min(locations.len())] {
                    results.push(SymbolInformation {
                        name: name.clone(),
                        kind: SymbolKind::FUNCTION,  // 存储实际符号类型（简化版本）
                        location: location.clone(),
                        tags: None,
                        container_name: None,
                        deprecated: None,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Find all references to a symbol
    pub async fn find_references(
        &self,
        uri: &str,
        position: Position,
    ) -> anyhow::Result<Vec<Location>> {
        if let Some(symbol) = self.find_symbol_at_position(uri, position).await {
            if let Some(locations) = self.symbol_locations.get(&symbol.name) {
                Ok(locations.clone())
            } else {
                Ok(vec![])
            }
        } else {
            Ok(vec![])
        }
    }

    /// Rename a symbol and update all references
    pub async fn rename_symbol(
        &self,
        uri: &str,
        position: Position,
        new_name: &str,
    ) -> anyhow::Result<WorkspaceEdit> {
        if let Some(symbol) = self.find_symbol_at_position(uri, position).await {
            if let Some(locations) = self.symbol_locations.get(&symbol.name) {
                let mut document_changes = HashMap::new();

                for location in locations {
                    document_changes
                        .entry(location.uri.clone())
                        .or_insert_with(Vec::new)
                        .push(TextEdit {
                            range: location.range,
                            new_text: new_name.to_string(),
                        });
                }

                let mut workspace_edits = HashMap::new();
                for (uri, edits) in document_changes {
                    workspace_edits.insert(
                        uri,
                        TextDocumentEdit {
                            text_document: OptionalVersionedTextDocumentIdentifier {
                                uri,
                                version: None,
                            },
                            edits: edits.into(),
                        },
                    );
                }

                Ok(WorkspaceEdit {
                    changes: None,
                    document_changes: Some(WorkspaceEditDocumentChanges::Edits(
                        workspace_edits.values().cloned().collect(),
                    )),
                    change_annotations: None,
                })
            } else {
                Ok(WorkspaceEdit {
                    changes: Some(HashMap::new()),
                    document_changes: None,
                    change_annotations: None,
                })
            }
        } else {
            Ok(WorkspaceEdit {
                changes: Some(HashMap::new()),
                document_changes: None,
                change_annotations: None,
            })
        }
    }

    /// Get signature help for a function call
    pub async fn get_signature_help(
        &self,
        uri: &str,
        position: Position,
    ) -> Option<SignatureHelp> {
        // 签名帮助（简化版本）
        // This requires parsing the function call and looking up the function signature
        None
    }

    /// Get the number of symbols in the index
    pub fn len(&self) -> usize {
        self.symbol_locations.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.symbol_locations.is_empty()
    }
}
