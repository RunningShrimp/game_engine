//! # Code Completion
//!
//! Provides code completion suggestions for engine API.

use crate::tools::lsp::registry::{ComponentDefinition, MethodDefinition, ResourceDefinition, SystemDefinition};
use tower_lsp::lsp_types::CompletionItem;
use tower_lsp::lsp_types::CompletionItemKind;

/// Completion context information
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// Current line in the document
    pub line: String,

    /// Cursor position in line
    pub cursor_offset: usize,

    /// Current file path
    pub file_path: String,

    /// Whether we're in a macro invocation
    pub in_macro: bool,

    /// Current macro name (if in macro)
    pub macro_name: Option<String>,
}

/// Completion provider
pub struct CompletionProvider {
    /// Engine API registry
    registry: super::registry::EngineAPIRegistry,
}

impl CompletionProvider {
    /// Create a new completion provider
    pub fn new(registry: super::registry::EngineAPIRegistry) -> Self {
        Self { registry }
    }

    /// Get completion items for the given context
    pub async fn get_completions(&self, context: &CompletionContext) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Analyze context to determine what to complete
        if self.is_in_system_query(&context.line) {
            self.complete_system_query(&context.line, &mut items).await;
        } else if self.is_in_resource_access(&context.line) {
            self.complete_resource_access(&context.line, &mut items).await;
        } else if self.is_in_component_field(&context.line) {
            self.complete_component_fields(&context.line, &mut items).await;
        } else {
            // General completion
            self.complete_general(&context.line, &mut items).await;
        }

        items
    }

    /// Check if cursor is in a system query parameter
    fn is_in_system_query(&self, line: &str) -> bool {
        line.contains("Query<") || line.contains("Res<") || line.contains("ResMut<")
    }

    /// Check if cursor is in a resource access
    fn is_in_resource_access(&self, line: &str) -> bool {
        line.contains("Res<") || line.contains("ResMut<")
    }

    /// Check if cursor is in a component field access
    fn is_in_component_field(&self, line: &str) -> bool {
        // Check if line contains component-like patterns
        line.contains(".")
    }

    /// Complete general code (components, systems, resources)
    async fn complete_general(&self, line: &str, items: &mut Vec<CompletionItem>) {
        // Add component completions
        let components = self.registry.list_components().await;
        for name in components {
            if let Some(component) = self.registry.get_component(&name).await {
                items.push(self.component_to_completion(&component));
            }
        }

        // Add resource completions
        let resources = self.registry.list_resources().await;
        for name in resources {
            if let Some(resource) = self.registry.get_resource(&name).await {
                items.push(self.resource_to_completion(&resource));
            }
        }

        // Add system completions
        let systems = self.registry.list_systems().await;
        for name in systems {
            if let Some(system) = self.registry.get_system(&name).await {
                items.push(self.system_to_completion(&system));
            }
        }
    }

    /// Complete system query components
    async fn complete_system_query(&self, line: &str, items: &mut Vec<CompletionItem>) {
        // Extract query type
        let query_type = if line.contains("Query<") {
            "Query"
        } else if line.contains("Res<") {
            "Res"
        } else if line.contains("ResMut<") {
            "ResMut"
        } else {
            return;
        };

        // Add component completions
        let components = self.registry.list_components().await;
        for name in components {
            if let Some(component) = self.registry.get_component(&name).await {
                let label = format!("{}<{}>", query_type, component.name);
                items.push(CompletionItem {
                    label: component.name.clone(),
                    kind: Some(CompletionItemKind::STRUCT),
                    detail: Some(format!("{} component", component.name)),
                    documentation: Some(tower_lsp::lsp_types::Documentation::MarkupContent(
                        tower_lsp::lsp_types::MarkupContent {
                            kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                            value: component.documentation.clone(),
                        },
                    )),
                    insert_text: Some(component.name.clone()),
                    ..Default::default()
                });
            }
        }

        // Add resource completions
        let resources = self.registry.list_resources().await;
        for name in resources {
            if let Some(resource) = self.registry.get_resource(&name).await {
                let label = format!("{}<{}>", query_type, resource.name);
                items.push(CompletionItem {
                    label: resource.name.clone(),
                    kind: Some(CompletionItemKind::STRUCT),
                    detail: Some(format!("{} resource", resource.name)),
                    documentation: Some(tower_lsp::lsp_types::Documentation::MarkupContent(
                        tower_lsp::lsp_types::MarkupContent {
                            kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                            value: resource.documentation.clone(),
                        },
                    )),
                    insert_text: Some(resource.name.clone()),
                    ..Default::default()
                });
            }
        }
    }

    /// Complete resource access
    async fn complete_resource_access(&self, line: &str, items: &mut Vec<CompletionItem>) {
        let resources = self.registry.list_resources().await;
        for name in resources {
            if let Some(resource) = self.registry.get_resource(&name).await {
                items.push(self.resource_to_completion(&resource));
            }
        }
    }

    /// Complete component fields
    async fn complete_component_fields(&self, line: &str, items: &mut Vec<CompletionItem>) {
        // Try to extract component name from line
        // This is a simplified version - real implementation would parse the AST
        let components = self.registry.list_components().await;
        for name in components {
            if line.contains(&format!("{}.", name)) || line.contains(&format!("<{}>", name)) {
                if let Some(component) = self.registry.get_component(&name).await {
                    for field in &component.fields {
                        items.push(CompletionItem {
                            label: field.name.clone(),
                            kind: Some(CompletionItemKind::FIELD),
                            detail: Some(field.type_name.clone()),
                            documentation: Some(tower_lsp::lsp_types::Documentation::MarkupContent(
                                tower_lsp::lsp_types::MarkupContent {
                                    kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                                    value: field.description.clone(),
                                },
                            )),
                            ..Default::default()
                        });
                    }

                    // Add methods
                    for method in &component.methods {
                        items.push(self.method_to_completion(method));
                    }
                }
            }
        }
    }

    /// Convert component definition to completion item
    fn component_to_completion(&self, component: &ComponentDefinition) -> CompletionItem {
        CompletionItem {
            label: component.name.clone(),
            kind: Some(CompletionItemKind::STRUCT),
            detail: Some(format!("Component: {}", component.name)),
            documentation: Some(tower_lsp::lsp_types::Documentation::MarkupContent(
                tower_lsp::lsp_types::MarkupContent {
                    kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                    value: component.documentation.clone(),
                },
            )),
            insert_text: Some(component.name.clone()),
            ..Default::default()
        }
    }

    /// Convert resource definition to completion item
    fn resource_to_completion(&self, resource: &ResourceDefinition) -> CompletionItem {
        CompletionItem {
            label: resource.name.clone(),
            kind: Some(CompletionItemKind::STRUCT),
            detail: Some(format!("Resource: {}", resource.name)),
            documentation: Some(tower_lsp::lsp_types::Documentation::MarkupContent(
                tower_lsp::lsp_types::MarkupContent {
                    kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                    value: resource.documentation.clone(),
                },
            )),
            insert_text: Some(resource.name.clone()),
            ..Default::default()
        }
    }

    /// Convert system definition to completion item
    fn system_to_completion(&self, system: &SystemDefinition) -> CompletionItem {
        CompletionItem {
            label: system.name.clone(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(format!("System: {}", system.system_type)),
            documentation: Some(tower_lsp::lsp_types::Documentation::MarkupContent(
                tower_lsp::lsp_types::MarkupContent {
                    kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                    value: system.documentation.clone(),
                },
            )),
            insert_text: Some(system.name.clone()),
            ..Default::default()
        }
    }

    /// Convert method definition to completion item
    fn method_to_completion(&self, method: &MethodDefinition) -> CompletionItem {
        CompletionItem {
            label: method.name.clone(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some(format!("{} -> {}", method.name, method.return_type)),
            documentation: Some(tower_lsp::lsp_types::Documentation::MarkupContent(
                tower_lsp::lsp_types::MarkupContent {
                    kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                    value: method.description.clone(),
                },
            )),
            insert_text: Some(method.name.clone()),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_completion_provider() {
        let registry = super::super::registry::EngineAPIRegistry::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let provider = CompletionProvider::new(registry);

        let context = CompletionContext {
            line: "fn system(query: Query<".to_string(),
            cursor_offset: 20,
            file_path: "test.rs".to_string(),
            in_macro: false,
            macro_name: None,
        };

        let completions = provider.get_completions(&context).await;
        assert!(!completions.is_empty());
    }

    #[tokio::test]
    async fn test_system_query_completion() {
        let registry = super::super::registry::EngineAPIRegistry::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let provider = CompletionProvider::new(registry);

        let context = CompletionContext {
            line: "fn system(query: Query<Trans".to_string(),
            cursor_offset: 25,
            file_path: "test.rs".to_string(),
            in_macro: false,
            macro_name: None,
        };

        let completions = provider.get_completions(&context).await;
        // Should complete to Transform
        assert!(completions.iter().any(|c| c.label == "Transform"));
    }
}
