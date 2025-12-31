//! # Diagnostics
//!
//! Provides real-time error checking and diagnostics for engine API usage.

use tower_lsp::lsp_types::Diagnostic;
use tower_lsp::lsp_types::DiagnosticSeverity;
use tower_lsp::lsp_types::Position;
use tower_lsp::lsp_types::Range;
use tower_lsp::lsp_types::NumberOrString;

/// Diagnostic provider
pub struct DiagnosticProvider {
    /// Engine API registry
    registry: super::registry::EngineAPIRegistry,
}

impl DiagnosticProvider {
    /// Create a new diagnostic provider
    pub fn new(registry: super::registry::EngineAPIRegistry) -> Self {
        Self { registry }
    }

    /// Analyze document and return diagnostics
    pub async fn analyze(&self, content: &str, uri: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Analyze line by line
        for (line_idx, line) in content.lines().enumerate() {
            self.analyze_line(line, line_idx, uri, &mut diagnostics).await;
        }

        diagnostics
    }

    /// Analyze a single line
    async fn analyze_line(&self, line: &str, line_idx: usize, _uri: &str, diagnostics: &mut Vec<Diagnostic>) {
        // Check for unknown components in queries
        if let Some(query_start) = line.find("Query<") {
            let query_content = &line[query_start..];
            if let Some(query_end) = query_content.find('>') {
                let components_str = &query_content[6..query_end]; // Skip "Query<"

                // Split by comma to get individual components
                for component in components_str.split(',') {
                    let component = component.trim();
                    if !component.is_empty() && !self.is_valid_component(component).await {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position::new(line_idx as u32, query_start as u32),
                                end: Position::new(line_idx as u32, (query_start + query_end) as u32),
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("unknown-component".to_string())),
                            source: Some("game-engine-lsp".to_string()),
                            message: format!("Unknown component: '{}'", component),
                            related_information: None,
                            tags: None,
                            data: None,
                            code_description: None,
                        });
                    }
                }
            }
        }

        // Check for unknown resources
        if let Some(res_start) = line.find("Res<") {
            let res_content = &line[res_start..];
            if let Some(res_end) = res_content.find('>') {
                let resource_name = &res_content[4..res_end]; // Skip "Res<"

                let resource = resource_name.trim();
                if !resource.is_empty() && !self.is_valid_resource(resource).await {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position::new(line_idx as u32, res_start as u32),
                            end: Position::new(line_idx as u32, (res_start + res_end) as u32),
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("unknown-resource".to_string())),
                        source: Some("game-engine-lsp".to_string()),
                        message: format!("Unknown resource: '{}'", resource),
                        related_information: None,
                        tags: None,
                        data: None,
                        code_description: None,
                    });
                }
            }
        }

        // Check for unknown mutable resources
        if let Some(res_start) = line.find("ResMut<") {
            let res_content = &line[res_start..];
            if let Some(res_end) = res_content.find('>') {
                let resource_name = &res_content[7..res_end]; // Skip "ResMut<"

                let resource = resource_name.trim();
                if !resource.is_empty() && !self.is_valid_resource(resource).await {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position::new(line_idx as u32, res_start as u32),
                            end: Position::new(line_idx as u32, (res_start + res_end) as u32),
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("unknown-resource".to_string())),
                        source: Some("game-engine-lsp".to_string()),
                        message: format!("Unknown resource: '{}'", resource),
                        related_information: None,
                        tags: None,
                        data: None,
                        code_description: None,
                    });
                }
            }
        }

        // Check for mutable resource in read-only context
        if line.contains("fn system(") && line.contains("ResMut<") {
            // This is a simplified check - real implementation would analyze function signature
            // For now, just add a warning
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position::new(line_idx as u32, 0),
                    end: Position::new(line_idx as u32, line.len() as u32),
                },
                severity: Some(DiagnosticSeverity::WARNING),
                code: Some(NumberOrString::String("mutable-resource-warning".to_string())),
                source: Some("game-engine-lsp".to_string()),
                message: "Using ResMut - ensure this resource is meant to be mutated".to_string(),
                related_information: None,
                tags: None,
                data: None,
                code_description: None,
            });
        }
    }

    /// Check if a component name is valid
    async fn is_valid_component(&self, name: &str) -> bool {
        self.registry.get_component(name).await.is_some()
    }

    /// Check if a resource name is valid
    async fn is_valid_resource(&self, name: &str) -> bool {
        self.registry.get_resource(name).await.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_diagnostics_unknown_component() {
        let registry = super::super::registry::EngineAPIRegistry::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let provider = DiagnosticProvider::new(registry);

        let code = "fn system(query: Query<UnknownComponent>) {";
        let diagnostics = provider.analyze(code, "test.rs").await;

        assert!(!diagnostics.is_empty());
        assert!(diagnostics.iter().any(|d| d.message.contains("Unknown component")));
    }

    #[tokio::test]
    async fn test_diagnostics_valid_component() {
        let registry = super::super::registry::EngineAPIRegistry::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let provider = DiagnosticProvider::new(registry);

        let code = "fn system(query: Query<Transform>) {";
        let diagnostics = provider.analyze(code, "test.rs").await;

        // Should not have errors for valid component
        assert!(diagnostics
            .iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR))
            .count() == 0);
    }

    #[tokio::test]
    async fn test_diagnostics_unknown_resource() {
        let registry = super::super::registry::EngineAPIRegistry::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let provider = DiagnosticProvider::new(registry);

        let code = "fn system(res: Res<UnknownResource>) {";
        let diagnostics = provider.analyze(code, "test.rs").await;

        assert!(!diagnostics.is_empty());
        assert!(diagnostics.iter().any(|d| d.message.contains("Unknown resource")));
    }
}
