//! # Symbol Management for LSP
//!
//! Provides document symbols and workspace symbols functionality.

use super::documents::{SymbolIndex, SymbolKind, SymbolLocation};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

/// Document symbols provider
pub struct DocumentSymbolsProvider {
    symbol_index: SymbolIndex,
}

impl DocumentSymbolsProvider {
    /// Create a new document symbols provider
    pub fn new(symbol_index: SymbolIndex) -> Self {
        Self { symbol_index }
    }

    /// Get document symbols for a file
    pub async fn get_document_symbols(&self, uri: &str) -> Vec<DocumentSymbol> {
        // Get symbols from the symbol index
        let files_index = self.symbol_index.files_index.read().await;

        if let Some(symbol_names) = files_index.get(uri) {
            let index = self.symbol_index.index.read().await;
            let mut symbols = Vec::new();

            for symbol_name in symbol_names {
                if let Some(locations) = index.get(symbol_name) {
                    // Find locations in this file
                    for location in locations {
                        if location.uri == uri {
                            symbols.push(self.location_to_document_symbol(location));
                        }
                    }
                }
            }

            symbols
        } else {
            Vec::new()
        }
    }

    /// Convert SymbolLocation to DocumentSymbol
    fn location_to_document_symbol(&self, location: &SymbolLocation) -> DocumentSymbol {
        DocumentSymbol {
            name: location.name.clone(),
            detail: None,
            kind: self.symbol_kind_to_document_symbol_kind(&location.kind),
            tags: None,
            deprecated: None,
            range: Range {
                start: Position {
                    line: location.line,
                    character: location.character,
                },
                end: Position {
                    line: location.line,
                    character: location.character + location.name.len() as u32,
                },
            },
            selection_range: Range {
                start: Position {
                    line: location.line,
                    character: location.character,
                },
                end: Position {
                    line: location.line,
                    character: location.character + location.name.len() as u32,
                },
            },
            children: None,
        }
    }

    /// Convert SymbolKind to LSP DocumentSymbol kind
    fn symbol_kind_to_document_symbol_kind(
        &self,
        kind: &SymbolKind,
    ) -> tower_lsp::lsp_types::SymbolKind {
        match kind {
            SymbolKind::Component => tower_lsp::lsp_types::SymbolKind::STRUCT,
            SymbolKind::System => tower_lsp::lsp_types::SymbolKind::FUNCTION,
            SymbolKind::Resource => tower_lsp::lsp_types::SymbolKind::STRUCT,
            SymbolKind::Function => tower_lsp::lsp_types::SymbolKind::FUNCTION,
            SymbolKind::Variable => tower_lsp::lsp_types::SymbolKind::VARIABLE,
            SymbolKind::Struct => tower_lsp::lsp_types::SymbolKind::STRUCT,
            SymbolKind::Enum => tower_lsp::lsp_types::SymbolKind::ENUM,
            SymbolKind::Constant => tower_lsp::lsp_types::SymbolKind::CONSTANT,
            SymbolKind::Module => tower_lsp::lsp_types::SymbolKind::MODULE,
            SymbolKind::Unknown => tower_lsp::lsp_types::SymbolKind::NULL,
        }
    }
}

/// Workspace symbols provider
pub struct WorkspaceSymbolsProvider {
    symbol_index: SymbolIndex,
}

impl WorkspaceSymbolsProvider {
    /// Create a new workspace symbols provider
    pub fn new(symbol_index: SymbolIndex) -> Self {
        Self { symbol_index }
    }

    /// Search for symbols in the workspace
    pub async fn search_symbols(&self, query: &str) -> Vec<SymbolInformation> {
        let index = self.symbol_index.index.read().await;
        let mut results = Vec::new();

        // Simple prefix matching (can be enhanced with fuzzy matching)
        let query_lower = query.to_lowercase();

        for (symbol_name, locations) in index.iter() {
            if symbol_name.to_lowercase().contains(&query_lower) {
                for location in locations {
                    results.push(SymbolInformation {
                        name: symbol_name.clone(),
                        kind: self.symbol_kind_to_lsp_kind(&location.kind),
                        tags: None,
                        deprecated: None,
                        location: location.to_lsp_location(),
                        container_name: None,
                    });
                }
            }
        }

        // Sort by name for better UX
        results.sort_by(|a, b| a.name.cmp(&b.name));
        results
    }

    /// Convert SymbolKind to LSP SymbolKind
    fn symbol_kind_to_lsp_kind(&self, kind: &SymbolKind) -> tower_lsp::lsp_types::SymbolKind {
        match kind {
            SymbolKind::Component => tower_lsp::lsp_types::SymbolKind::STRUCT,
            SymbolKind::System => tower_lsp::lsp_types::SymbolKind::FUNCTION,
            SymbolKind::Resource => tower_lsp::lsp_types::SymbolKind::STRUCT,
            SymbolKind::Function => tower_lsp::lsp_types::SymbolKind::FUNCTION,
            SymbolKind::Variable => tower_lsp::lsp_types::SymbolKind::VARIABLE,
            SymbolKind::Struct => tower_lsp::lsp_types::SymbolKind::STRUCT,
            SymbolKind::Enum => tower_lsp::lsp_types::SymbolKind::ENUM,
            SymbolKind::Constant => tower_lsp::lsp_types::SymbolKind::CONSTANT,
            SymbolKind::Module => tower_lsp::lsp_types::SymbolKind::MODULE,
            SymbolKind::Unknown => tower_lsp::lsp_types::SymbolKind::NULL,
        }
    }
}
