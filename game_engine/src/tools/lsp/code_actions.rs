//! # Code Actions for LSP
//!
//! Provides code actions and quick fixes for common issues.

use super::registry::EngineAPIRegistry;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

/// Code actions provider
pub struct CodeActionsProvider {
    registry: EngineAPIRegistry,
}

impl CodeActionsProvider {
    /// Create a new code actions provider
    pub fn new(registry: EngineAPIRegistry) -> Self {
        Self { registry }
    }

    /// Get code actions for a document
    pub async fn get_code_actions(
        &self,
        uri: &Url,
        range: &Range,
        context: &CodeActionContext,
    ) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();

        // Add quick fixes for diagnostics
        for diagnostic in &context.diagnostics {
            if let Some(quick_fix) = self.get_quick_fix(uri, diagnostic).await {
                actions.push(CodeActionOrCommand::CodeAction(quick_fix));
            }
        }

        // Add import suggestions for unknown components
        if let Some(import_action) = self.suggest_import(uri, range).await {
            actions.push(CodeActionOrCommand::CodeAction(import_action));
        }

        actions
    }

    /// Get quick fix for a diagnostic
    async fn get_quick_fix(&self, uri: &Url, diagnostic: &Diagnostic) -> Option<CodeAction> {
        let message = diagnostic.message.as_str();

        // Quick fix for unknown component
        if message.contains("unknown component") || message.contains("Unknown component") {
            if let Some(component_name) = self.extract_component_name(message) {
                return self.create_import_fix(uri, &component_name, &diagnostic.range).await;
            }
        }

        // Quick fix for unknown resource
        if message.contains("unknown resource") || message.contains("Unknown resource") {
            if let Some(resource_name) = self.extract_resource_name(message) {
                return self
                    .create_resource_import_fix(uri, &resource_name, &diagnostic.range)
                    .await;
            }
        }

        None
    }

    /// Extract component name from error message
    fn extract_component_name(&self, message: &str) -> Option<String> {
        // Simple extraction - can be enhanced with regex
        if let Some(start) = message.find("`") {
            if let Some(end) = message[start + 1..].find("`") {
                return Some(message[start + 1..start + 1 + end].to_string());
            }
        }
        None
    }

    /// Extract resource name from error message
    fn extract_resource_name(&self, message: &str) -> Option<String> {
        self.extract_component_name(message) // Same logic for now
    }

    /// Create import fix for component
    async fn create_import_fix(
        &self,
        uri: &Url,
        component_name: &str,
        range: &Range,
    ) -> Option<CodeAction> {
        // Check if component exists in registry
        if self.registry.get_component(component_name).await.is_some() {
            let edit = TextEdit {
                range: *range,
                new_text: format!("use game_engine::ecs::{};\n", component_name),
            };

            Some(CodeAction {
                title: format!("Import {}", component_name),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: Some({
                        let mut changes = std::collections::HashMap::new();
                        changes.insert(uri.clone(), vec![edit]);
                        changes
                    }),
                    document_changes: None,
                    change_annotations: None,
                }),
                command: None,
                is_preferred: Some(true),
                disabled: None,
                data: None,
            })
        } else {
            None
        }
    }

    /// Create import fix for resource
    async fn create_resource_import_fix(
        &self,
        uri: &Url,
        resource_name: &str,
        range: &Range,
    ) -> Option<CodeAction> {
        if self.registry.get_resource(resource_name).await.is_some() {
            let edit = TextEdit {
                range: *range,
                new_text: format!("use game_engine::ecs::{};\n", resource_name),
            };

            Some(CodeAction {
                title: format!("Import {}", resource_name),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: Some({
                        let mut changes = std::collections::HashMap::new();
                        changes.insert(uri.clone(), vec![edit]);
                        changes
                    }),
                    document_changes: None,
                    change_annotations: None,
                }),
                command: None,
                is_preferred: Some(true),
                disabled: None,
                data: None,
            })
        } else {
            None
        }
    }

    /// Suggest import for unknown symbol
    async fn suggest_import(&self, uri: &Url, range: &Range) -> Option<CodeAction> {
        // This would analyze the code and suggest imports
        // For now, return None as a placeholder
        None
    }
}
