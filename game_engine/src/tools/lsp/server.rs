//! # Game Engine LSP Server
//!
//! Main LSP server implementation using tower-lsp.

use super::completion::{CompletionContext, CompletionProvider};
use super::diagnostics::DiagnosticProvider;
use super::hover::HoverProvider;
use super::registry::EngineAPIRegistry;
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
}

impl GameEngineLSP {
    /// Create a new LSP server instance
    pub fn new(client: Client) -> Self {
        let registry = EngineAPIRegistry::new();
        let completion_provider = CompletionProvider::new(registry.clone());
        let hover_provider = HoverProvider::new(registry.clone());
        let diagnostic_provider = DiagnosticProvider::new(registry.clone());

        Self {
            client,
            registry,
            completion_provider,
            hover_provider,
            diagnostic_provider,
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

        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
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
                .log_message(MessageType::INFO, format!("Text document capabilities: {:?}", capabilities))
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
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "Game Engine LSP Server shutting down...")
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, format!("File opened: {}", params.text_document.uri))
            .await;

        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        self.publish_diagnostics(uri, version, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Get full text from changes
        let text = params
            .content_changes
            .iter()
            .filter(|c| c.range.is_none())
            .map(|c| c.text.clone())
            .next();

        if let Some(text) = text {
            self.publish_diagnostics(uri, version, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, format!("File closed: {}", params.text_document.uri))
            .await;

        // Clear diagnostics
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
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

        // For now, return None as we don't have actual source code locations
        // In a full implementation, this would return locations to definitions
        Ok(None)
    }

    // Note: diagnostics method removed due to conflict with LanguageServer trait
    // Use publish_diagnostics in did_open/did_change instead

}

impl GameEngineLSP {
    /// Get current line from document (simplified implementation)
    async fn get_current_line(&self, _uri: &Url, _position: Position) -> Option<String> {
        // In a real implementation, this would retrieve the cached document content
        // For now, return empty string
        None
    }

    /// Get full document text (simplified implementation)
    async fn get_document_text(&self, _uri: &Url) -> Option<String> {
        // In a real implementation, this would retrieve the cached document content
        // For now, return None
        None
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
