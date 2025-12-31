//! # Hover Information
//!
//! Provides hover information for engine API.

use crate::tools::lsp::registry::{
    ComponentDefinition, FieldDefinition, MethodDefinition, ResourceDefinition,
};
use tower_lsp::lsp_types::Hover;
use tower_lsp::lsp_types::HoverContents;
use tower_lsp::lsp_types::MarkupContent;
use tower_lsp::lsp_types::MarkupKind;
use tower_lsp::lsp_types::Range;

/// Hover provider
pub struct HoverProvider {
    /// Engine API registry
    registry: super::registry::EngineAPIRegistry,
}

impl HoverProvider {
    /// Create a new hover provider
    pub fn new(registry: super::registry::EngineAPIRegistry) -> Self {
        Self { registry }
    }

    /// Get hover information for the given word and context
    pub async fn get_hover(&self, word: &str, line: &str) -> Option<Hover> {
        // Try to find as component
        if let Some(component) = self.registry.get_component(word).await {
            return Some(self.component_to_hover(&component));
        }

        // Try to find as resource
        if let Some(resource) = self.registry.get_resource(word).await {
            return Some(self.resource_to_hover(&resource));
        }

        // Try to find as field
        if let Some(hover) = self.find_field_hover(word, line).await {
            return Some(hover);
        }

        // Try to find as method
        if let Some(hover) = self.find_method_hover(word, line).await {
            return Some(hover);
        }

        None
    }

    /// Find hover information for a field
    async fn find_field_hover(&self, field_name: &str, line: &str) -> Option<Hover> {
        // Extract component name from line (simplified)
        let components = self.registry.list_components().await;
        for comp_name in components {
            if line.contains(&comp_name) {
                if let Some(component) = self.registry.get_component(&comp_name).await {
                    for field in &component.fields {
                        if field.name == field_name {
                            return Some(self.field_to_hover(field, &component));
                        }
                    }
                }
            }
        }
        None
    }

    /// Find hover information for a method
    async fn find_method_hover(&self, method_name: &str, line: &str) -> Option<Hover> {
        // Check components
        let components = self.registry.list_components().await;
        for comp_name in components {
            if line.contains(&comp_name) {
                if let Some(component) = self.registry.get_component(&comp_name).await {
                    for method in &component.methods {
                        if method.name == method_name {
                            return Some(self.method_to_hover(method, &component));
                        }
                    }
                }
            }
        }

        // Check resources
        let resources = self.registry.list_resources().await;
        for res_name in resources {
            if line.contains(&res_name) {
                if let Some(resource) = self.registry.get_resource(&res_name).await {
                    for method in &resource.methods {
                        if method.name == method_name {
                            return Some(self.method_to_hover_in_resource(method, &resource));
                        }
                    }
                }
            }
        }

        None
    }

    /// Convert component definition to hover
    fn component_to_hover(&self, component: &ComponentDefinition) -> Hover {
        let mut markdown = format!("# Component: `{}`\n\n", component.name);
        markdown.push_str(&format!("**Module:** `{}`\n\n", component.module));
        markdown.push_str(&format!("{}\n\n", component.description));

        if !component.fields.is_empty() {
            markdown.push_str("## Fields\n\n");
            for field in &component.fields {
                markdown.push_str(&format!(
                    "- **`{}: {}`** - {}\n",
                    field.name, field.type_name, field.description
                ));
            }
            markdown.push_str("\n");
        }

        if !component.methods.is_empty() {
            markdown.push_str("## Methods\n\n");
            for method in &component.methods {
                markdown.push_str(&format!(
                    "- **`{}{} -> {}`** - {}\n",
                    method.name,
                    self.format_params(&method.parameters),
                    method.return_type,
                    method.description
                ));
            }
            markdown.push_str("\n");
        }

        if !component.documentation.is_empty() {
            markdown.push_str("## Documentation\n\n");
            markdown.push_str(&component.documentation);
        }

        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
            range: None,
        }
    }

    /// Convert resource definition to hover
    fn resource_to_hover(&self, resource: &ResourceDefinition) -> Hover {
        let mut markdown = format!("# Resource: `{}`\n\n", resource.name);
        markdown.push_str(&format!("**Module:** `{}`\n\n", resource.module));
        markdown.push_str(&format!("{}\n\n", resource.description));

        if !resource.methods.is_empty() {
            markdown.push_str("## Methods\n\n");
            for method in &resource.methods {
                markdown.push_str(&format!(
                    "- **`{}{} -> {}`** - {}\n",
                    method.name,
                    self.format_params(&method.parameters),
                    method.return_type,
                    method.description
                ));
            }
            markdown.push_str("\n");
        }

        if !resource.documentation.is_empty() {
            markdown.push_str("## Documentation\n\n");
            markdown.push_str(&resource.documentation);
        }

        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
            range: None,
        }
    }

    /// Convert field definition to hover
    fn field_to_hover(&self, field: &FieldDefinition, component: &ComponentDefinition) -> Hover {
        let markdown = format!(
            "**`{}: {}`** (field of `{}`)\n\n{}\n\nType: `{}`",
            field.name, field.type_name, component.name, field.description, field.type_name
        );

        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
            range: None,
        }
    }

    /// Convert method definition to hover
    fn method_to_hover(&self, method: &MethodDefinition, component: &ComponentDefinition) -> Hover {
        let mut markdown = format!(
            "**`{}{} -> {}`** (method of `{}`)\n\n{}\n\n",
            method.name,
            self.format_params(&method.parameters),
            method.return_type,
            component.name,
            method.description
        );

        if method.is_async {
            markdown.push_str("*async method*\n\n");
        }

        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
            range: None,
        }
    }

    /// Convert method definition to hover (in resource)
    fn method_to_hover_in_resource(
        &self,
        method: &MethodDefinition,
        resource: &ResourceDefinition,
    ) -> Hover {
        let mut markdown = format!(
            "**`{}{} -> {}`** (method of `{}`)\n\n{}\n\n",
            method.name,
            self.format_params(&method.parameters),
            method.return_type,
            resource.name,
            method.description
        );

        if method.is_async {
            markdown.push_str("*async method*\n\n");
        }

        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
            range: None,
        }
    }

    /// Format method parameters for display
    fn format_params(&self, params: &[super::registry::ParameterDefinition]) -> String {
        if params.is_empty() {
            return String::new();
        }

        let formatted: Vec<String> =
            params.iter().map(|p| format!("{}: {}", p.name, p.type_name)).collect();

        format!("({})", formatted.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hover_component() {
        let registry = super::super::registry::EngineAPIRegistry::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let provider = HoverProvider::new(registry);

        let hover = provider.get_hover("Transform", "let t: Transform").await;
        assert!(hover.is_some());

        let hover = hover.unwrap();
        assert!(matches!(hover.contents, HoverContents::Markup(_)));
    }

    #[tokio::test]
    async fn test_hover_field() {
        let registry = super::super::registry::EngineAPIRegistry::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let provider = HoverProvider::new(registry);

        let hover = provider.get_hover("position", "let x = transform.position").await;
        assert!(hover.is_some());
    }
}
