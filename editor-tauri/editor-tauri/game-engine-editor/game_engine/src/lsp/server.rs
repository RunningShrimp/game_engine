//! Game Engine LSP Server Implementation
//!
//! Implements the Language Server Protocol for the game engine,
//! providing code intelligence features like completion, go-to-definition,
//! hover information, and symbol search.

use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tracing::{debug, error, info, warn};

use crate::lsp::api_index::ApiIndex;
use crate::lsp::completion::CompletionProvider;
use crate::lsp::symbol_info::SymbolInfo;

/// Game Engine LSP Server
///
/// Implements the LanguageServer trait to provide IDE features
/// for game engine development.
pub struct GameEngineServer {
    /// LSP client for sending notifications and requests
    client: Client,

    /// API index storing all game engine symbols
    api_index: Arc<Mutex<ApiIndex>>,

    /// Completion provider for code completion
    completion_provider: Arc<Mutex<CompletionProvider>>,

    /// Server state
    initialized: Arc<Mutex<bool>>,
}

impl GameEngineServer {
    /// Create a new GameEngineServer instance
    pub fn new(client: Client) -> Self {
        info!("Creating GameEngineServer");

        Self {
            client,
            api_index: Arc::new(Mutex::new(ApiIndex::new())),
            completion_provider: Arc::new(Mutex::new(CompletionProvider::new())),
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    /// Log a message to the client
    async fn log_message(&self, message: &str) {
        self.client
            .log_message(MessageType::INFO, message)
            .await;
    }

    /// Show an error message to the client
    async fn show_error(&self, message: &str) {
        self.client
            .show_message(MessageType::ERROR, message)
            .await;
    }

    /// Validate that the server has been initialized
    async fn ensure_initialized(&self) -> Result<()> {
        let initialized = *self.initialized.lock().await;
        if !initialized {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(
                "Server not initialized",
            ));
        }
        Ok(())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for GameEngineServer {
    /// Initialize the LSP server
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        info!("Initializing GameEngineServer");

        // Log initialization
        self.log_message("Game Engine LSP Server initializing...").await;

        // Build API index (this will scan the codebase)
        let mut index = self.api_index.lock().await;
        if let Err(e) = index.build() {
            error!("Failed to build API index: {}", e);
            self.show_error(&format!("Failed to build API index: {}", e))
                .await;
        } else {
            info!("API index built successfully with {} symbols", index.len());
        }

        // Initialize completion provider
        let mut completion = self.completion_provider.lock().await;
        completion.set_api_index(index.clone());

        // Mark server as initialized
        *self.initialized.lock().await = true;

        // Return server capabilities
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // Text document synchronization
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),

                // Completion provider
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),

                // Hover provider
                hover_provider: Some(HoverProviderCapability::Simple(true)),

                // Definition provider
                definition_provider: Some(OneOf::Left(true)),

                // Document symbol provider
                document_symbol_provider: Some(OneOf::Left(true)),

                // Workspace symbol provider
                workspace_symbol_provider: Some(OneOf::Left(true)),

                // References provider
                references_provider: Some(OneOf::Left(true)),

                // Rename provider
                rename_provider: Some(OneOf::Left(true)),

                // Code action provider
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),

                // Signature help provider
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: None,
                    ..Default::default()
                }),

                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "Game Engine LSP Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    /// Called when the client is ready to receive requests
    async fn initialized(&self, _: InitializedParams) {
        info!("Client initialized, server ready");
        self.log_message("Game Engine LSP Server ready! 🚀").await;
    }

    /// Shutdown the server
    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down GameEngineServer");
        self.log_message("Game Engine LSP Server shutting down...").await;
        Ok(())
    }

    /// Provide code completion
    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        self.ensure_initialized().await?;

        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        debug!("Completion requested at {:?}:{:?}", uri, position);

        let completion = self.completion_provider.lock().await;
        match completion.provide_completion(&uri, position).await {
            Ok(items) => Ok(Some(CompletionResponse::List(items))),
            Err(e) => {
                error!("Completion error: {}", e);
                Ok(None)
            }
        }
    }

    /// Provide hover information
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.ensure_initialized().await?;

        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        debug!("Hover requested at {:?}:{:?}", uri, position);

        let index = self.api_index.lock().await;
        if let Some(symbol) = index.find_symbol_at_position(&uri, position).await {
            Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: symbol.documentation,
                }),
                range: Some(symbol.range),
            }))
        } else {
            Ok(None)
        }
    }

    /// Go to definition
    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.ensure_initialized().await?;

        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        debug!("Go to definition requested at {:?}:{:?}", uri, position);

        let index = self.api_index.lock().await;
        if let Some(location) = index.find_definition(&uri, position).await {
            Ok(Some(GotoDefinitionResponse::Scalar(location)))
        } else {
            Ok(None)
        }
    }

    /// Provide document symbols
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        self.ensure_initialized().await?;

        let uri = params.text_document.uri;

        debug!("Document symbols requested for {:?}", uri);

        let index = self.api_index.lock().await;
        let symbols = index.document_symbols(&uri).await?;

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    /// Provide workspace symbols
    async fn symbol(&self, params: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
        self.ensure_initialized().await?;

        let query = &params.query;

        debug!("Workspace symbols requested for query: {}", query);

        let index = self.api_index.lock().await;
        let symbols = index.search_symbols(query).await?;

        Ok(Some(symbols))
    }

    /// Find references
    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        self.ensure_initialized().await?;

        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        debug!("References requested at {:?}:{:?}", uri, position);

        let index = self.api_index.lock().await;
        let references = index.find_references(&uri, position).await?;

        Ok(Some(references))
    }

    /// Rename symbol
    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        self.ensure_initialized().await?;

        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        debug!("Rename requested at {:?}:{:?} to {}", uri, position, new_name);

        let index = self.api_index.lock().await;
        let edit = index.rename_symbol(&uri, position, &new_name).await?;

        Ok(Some(edit))
    }

    /// Provide code actions
    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        self.ensure_initialized().await?;

        let uri = params.text_document.uri;
        let range = params.range;

        debug!("Code actions requested for {:?} at {:?}", uri, range);

        // 基本代码动作实现
        let mut actions = Vec::new();

        // 提取快速修复建议（如果有诊断信息）
        let diagnostics = self.diagnostics.lock().await;
        if let Some(file_diagnostics) = diagnostics.get(&uri) {
            for diagnostic in file_diagnostics {
                if diagnostic.range == range {
                    // 为错误诊断创建快速修复动作
                    if diagnostic.severity == Some(lsp_types::DiagnosticSeverity::ERROR) {
                        actions.push(lsp_types::CodeActionOrCommand::CodeAction(
                            lsp_types::CodeAction {
                                title: "查看文档以获取帮助".to_string(),
                                kind: Some(lsp_types::CodeActionKind::QUICKFIX),
                                diagnostics: Some(vec![diagnostic.clone()]),
                                edit: None,
                                command: Some(lsp_types::Command {
                                    title: "Open Documentation".to_string(),
                                    command: "engine.openDocumentation".to_string(),
                                    arguments: None,
                                }),
                                ..Default::default()
                            }
                        ));
                    }
                }
            }
        }

        // 添加通用的代码动作
        actions.push(lsp_types::CodeActionOrCommand::CodeAction(
            lsp_types::CodeAction {
                title: "格式化代码".to_string(),
                kind: Some(lsp_types::CodeActionKind::SOURCE_ORGANIZE_IMPORTS),
                diagnostics: None,
                edit: None,
                command: Some(lsp_types::Command {
                    title: "Format".to_string(),
                    command: "editor.action.formatDocument".to_string(),
                    arguments: None,
                }),
                ..Default::default()
            }
        ));

        Ok(Some(actions))
    }

    /// Signature help
    async fn signature_help(
        &self,
        params: SignatureHelpParams,
    ) -> Result<Option<SignatureHelp>> {
        self.ensure_initialized().await?;

        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        debug!("Signature help requested at {:?}:{:?}", uri, position);

        let index = self.api_index.lock().await;
        if let Some(help) = index.get_signature_help(&uri, position).await {
            Ok(Some(help))
        } else {
            Ok(None)
        }
    }
}
