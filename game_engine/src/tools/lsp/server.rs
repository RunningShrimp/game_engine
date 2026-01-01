//! # Game Engine LSP Server
//!
//! Main LSP server implementation using tower-lsp.

use super::code_actions::CodeActionsProvider;
use super::completion::{CompletionContext, CompletionProvider};
use super::diagnostics::DiagnosticProvider;
use super::documents::{DocumentCache, SymbolIndex};
use super::formatting::CodeFormatter;
use super::hover::HoverProvider;
use super::registry::EngineAPIRegistry;
use super::symbols::{DocumentSymbolsProvider, WorkspaceSymbolsProvider};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// Game Engine LSP Server
///
/// Implements the Language Server Protocol for the game engine.
pub struct GameEngineLSP {
    /// LSP client
    client: Client,

    /// Engine API registry
    registry: EngineAPIRegistry,

    /// Completion provider
    completion_provider: CompletionProvider,

    /// Hover provider
    hover_provider: HoverProvider,

    /// Diagnostic provider
    diagnostic_provider: DiagnosticProvider,

    /// Document cache for tracking open documents
    document_cache: DocumentCache,

    /// Symbol index for go-to-definition
    symbol_index: SymbolIndex,

    /// Document symbols provider
    document_symbols_provider: DocumentSymbolsProvider,

    /// Workspace symbols provider
    workspace_symbols_provider: WorkspaceSymbolsProvider,

    /// Code actions provider
    code_actions_provider: CodeActionsProvider,

    /// Code formatter
    code_formatter: CodeFormatter,
}

impl GameEngineLSP {
    /// Create a new LSP server instance
    pub fn new(client: Client) -> Self {
        let registry = EngineAPIRegistry::new();
        let completion_provider = CompletionProvider::new(registry.clone());
        let hover_provider = HoverProvider::new(registry.clone());
        let diagnostic_provider = DiagnosticProvider::new(registry.clone());
        let document_cache = DocumentCache::new(100, std::time::Duration::from_secs(300));
        let symbol_index = SymbolIndex::new();
        let document_symbols_provider = DocumentSymbolsProvider::new(symbol_index.clone());
        let workspace_symbols_provider = WorkspaceSymbolsProvider::new(symbol_index.clone());
        let code_actions_provider = CodeActionsProvider::new(registry.clone());
        let code_formatter = CodeFormatter::new();

        Self {
            client,
            registry,
            completion_provider,
            hover_provider,
            diagnostic_provider,
            document_cache,
            symbol_index,
            document_symbols_provider,
            workspace_symbols_provider,
            code_actions_provider,
            code_formatter,
        }
    }

    /// Extract word at position from document
    fn extract_word_at_position(text: &str, position: Position) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        if position.line as usize >= lines.len() {
            return None;
        }

        let line = lines[position.line as usize];
        let line_start = line.char_indices().nth(position.character as usize);

        if let Some((char_idx, _)) = line_start {
            // Find word boundaries
            let word_start = line[..char_idx]
                .rfind(|c: char| !c.is_alphanumeric() && c != '_')
                .map(|i| i + 1)
                .unwrap_or(0);

            let word_end = line[char_idx..]
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .map(|i| char_idx + i)
                .unwrap_or(line.len());

            Some(line[word_start..word_end].to_string())
        } else {
            None
        }
    }

    /// Get the line at position
    fn get_line_at_position(text: &str, position: Position) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        if position.line as usize >= lines.len() {
            return None;
        }
        Some(lines[position.line as usize].to_string())
    }

    /// Publish diagnostics for a document
    async fn publish_diagnostics(&self, uri: Url, version: i32, text: &str) {
        let diagnostics = self.diagnostic_provider.analyze(text, uri.as_str()).await;

        self.client.publish_diagnostics(uri, diagnostics, Some(version)).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for GameEngineLSP {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(MessageType::INFO, "Game Engine LSP Server initializing...")
            .await;

        // Log client capabilities
        if let Some(capabilities) = params.capabilities.text_document {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Text document capabilities: {:?}", capabilities),
                )
                .await;
        }

        let server_capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(vec![".".to_string(), "<".to_string(), ",".to_string()]),
                work_done_progress_options: Default::default(),
                all_commit_characters: None,
                completion_item: None,
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            workspace_symbol_provider: Some(OneOf::Left(true)),
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            document_formatting_provider: Some(OneOf::Left(true)),
            document_range_formatting_provider: Some(OneOf::Left(true)),
            // Note: diagnostic_provider is not available in tower-lsp 0.20
            // We use client.publish_diagnostics in did_open/did_change instead
            ..Default::default()
        };

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Game Engine LSP Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: server_capabilities,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Game Engine LSP Server initialized!")
            .await;

        // Wait for registry to populate
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let component_count = self.registry.list_components().await.len();
        let system_count = self.registry.list_systems().await.len();
        let resource_count = self.registry.list_resources().await.len();

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Registry loaded: {} components, {} systems, {} resources",
                    component_count, system_count, resource_count
                ),
            )
            .await;

        // Index engine API symbols for go-to-definition
        self.symbol_index.index_engine_api(&self.registry).await;
        self.client.log_message(MessageType::INFO, "Engine API symbols indexed").await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "Game Engine LSP Server shutting down...")
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();
        let version = params.text_document.version;
        let language_id = params.text_document.language_id.clone();

        self.client
            .log_message(MessageType::INFO, format!("File opened: {}", uri))
            .await;

        // Cache the document
        use super::documents::DocumentCacheEntry;
        let entry = DocumentCacheEntry {
            text: text.clone(),
            version,
            language_id: language_id.clone(),
            modified: std::time::Instant::now(),
        };
        self.document_cache.put(uri.to_string(), entry).await;

        // Index symbols in the document
        self.symbol_index.index_document(&uri.to_string(), &text, &language_id).await;

        // Publish diagnostics
        self.publish_diagnostics(uri, version, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;

        // Get full text from changes
        let text = params
            .content_changes
            .iter()
            .filter(|c| c.range.is_none())
            .map(|c| c.text.clone())
            .next();

        if let Some(text) = text {
            // Get language_id from cache before updating
            let language_id = if let Some(entry) = self.document_cache.get(&uri.to_string()).await {
                entry.language_id.clone()
            } else {
                return;
            };

            // Update the cached document
            use super::documents::DocumentCacheEntry;
            let entry = DocumentCacheEntry {
                text: text.clone(),
                version,
                language_id: language_id.clone(),
                modified: std::time::Instant::now(),
            };
            self.document_cache.put(uri.to_string(), entry).await;

            // Re-index symbols in the document
            self.symbol_index.index_document(&uri.to_string(), &text, &language_id).await;

            // Publish diagnostics
            self.publish_diagnostics(uri, version, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        self.client
            .log_message(MessageType::INFO, format!("File closed: {}", uri))
            .await;

        // Remove from cache
        self.document_cache.remove(&uri.to_string()).await;

        // Clear diagnostics
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let text_document_position = params.text_document_position;
        let uri = text_document_position.text_document.uri;

        // Get current document content (simplified - in real implementation, cache document content)
        // For now, return empty completions if we can't get the document
        let line = match self.get_current_line(&uri, text_document_position.position).await {
            Some(line) => line,
            None => return Ok(None),
        };

        let context = CompletionContext {
            line: line.clone(),
            cursor_offset: text_document_position.position.character as usize,
            file_path: uri.to_string(),
            in_macro: false,
            macro_name: None,
        };

        let items = self.completion_provider.get_completions(&context).await;

        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CompletionResponse::Array(items)))
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let text_document_position = params.text_document_position_params;
        let uri = text_document_position.text_document.uri;

        let text = match self.get_document_text(&uri).await {
            Some(text) => text,
            None => return Ok(None),
        };

        let line = match Self::get_line_at_position(&text, text_document_position.position) {
            Some(line) => line,
            None => return Ok(None),
        };

        let word = match Self::extract_word_at_position(&text, text_document_position.position) {
            Some(word) => word,
            None => return Ok(None),
        };

        let hover = self.hover_provider.get_hover(&word, &line).await;
        Ok(hover)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let text_document_position = params.text_document_position_params;
        let uri = text_document_position.text_document.uri;

        // Get document text from cache
        let text = match self.get_document_text(&uri).await {
            Some(text) => text,
            None => return Ok(None),
        };

        // Extract the word at the cursor position
        let word = match Self::extract_word_at_position(&text, text_document_position.position) {
            Some(word) => word,
            None => return Ok(None),
        };

        // Find symbol definitions
        let locations = self.symbol_index.find_symbol(&word).await;

        if locations.is_empty() {
            Ok(None)
        } else {
            // Convert symbol locations to LSP locations
            let lsp_locations: Vec<Location> =
                locations.iter().map(|loc| loc.to_lsp_location()).collect();

            Ok(Some(GotoDefinitionResponse::Array(lsp_locations)))
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.to_string();
        let symbols = self.document_symbols_provider.get_document_symbols(&uri).await;

        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(DocumentSymbolResponse::Nested(symbols)))
        }
    }

    // Note: workspace_symbol is not in tower-lsp 0.20 LanguageServer trait
    // This would need to be implemented as a custom request handler
    // For now, we'll skip it and focus on other features

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let range = params.range;
        let context = params.context;

        let actions = self.code_actions_provider.get_code_actions(&uri, &range, &context).await;

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CodeActionResponse::Array(actions)))
        }
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let options = params.options;

        let text = match self.get_document_text(&uri).await {
            Some(text) => text,
            None => return Ok(None),
        };

        let formatted = self.code_formatter.format(&text, &options);

        Ok(Some(vec![TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: text.lines().count() as u32,
                    character: 0,
                },
            },
            new_text: formatted,
        }]))
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let range = params.range;
        let options = params.options;

        let text = match self.get_document_text(&uri).await {
            Some(text) => text,
            None => return Ok(None),
        };

        let edits = self.code_formatter.format_range(&text, &range, &options);
        Ok(Some(edits))
    }

    // Note: diagnostics method removed due to conflict with LanguageServer trait
    // Use publish_diagnostics in did_open/did_change instead
}

impl GameEngineLSP {
    /// Get current line from document
    async fn get_current_line(&self, uri: &Url, position: Position) -> Option<String> {
        if let Some(entry) = self.document_cache.get(uri.as_str()).await {
            Self::get_line_at_position(&entry.text, position)
        } else {
            None
        }
    }

    /// Get full document text
    async fn get_document_text(&self, uri: &Url) -> Option<String> {
        if let Some(entry) = self.document_cache.get(uri.as_str()).await {
            Some(entry.text)
        } else {
            None
        }
    }
}

/// Serve the LSP server
pub async fn serve() -> std::io::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = tower_lsp::LspService::new(|client| GameEngineLSP::new(client));
    tower_lsp::Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_word() {
        let text = "let transform = Transform::new();";
        let position = Position::new(0, 14); // At 'transform'
        let word = GameEngineLSP::extract_word_at_position(text, position);
        assert_eq!(word, Some("transform".to_string()));
    }

    #[test]
    fn test_extract_word_at_boundaries() {
        let text = "let x = transform.position;";
        let position = Position::new(0, 20); // At 'position'
        let word = GameEngineLSP::extract_word_at_position(text, position);
        assert_eq!(word, Some("position".to_string()));
    }

    #[test]
    fn test_get_line() {
        let text = "line1\nline2\nline3";
        let position = Position::new(1, 0);
        let line = GameEngineLSP::get_line_at_position(text, position);
        assert_eq!(line, Some("line2".to_string()));
    }
}
