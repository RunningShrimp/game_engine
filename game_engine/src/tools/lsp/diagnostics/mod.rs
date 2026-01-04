//! # Rustc Diagnostics Integration
//!
//! Integrates with rustc compiler to provide real-time diagnostics.

use std::path::PathBuf;
use std::process::Command;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};

/// Rustc diagnostics provider
pub struct RustcDiagnostics {
    /// Path to rustc executable
    rustc_path: PathBuf,

    /// Target directory for artifacts
    target_dir: PathBuf,

    /// Project root directory
    project_root: PathBuf,

    /// Edition of Rust to use
    edition: String,
}

impl RustcDiagnostics {
    /// Create a new rustc diagnostics provider
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            rustc_path: PathBuf::from("rustc"),
            target_dir: project_root.join("target"),
            project_root: project_root.clone(),
            edition: "2021".to_string(),
        }
    }

    /// Set the edition
    pub fn with_edition(mut self, edition: &str) -> Self {
        self.edition = edition.to_string();
        self
    }

    /// Check a single file for diagnostics
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to check
    ///
    /// # Returns
    ///
    /// List of diagnostics found in the file
    pub async fn check_file(&self, file_path: &str) -> Result<Vec<Diagnostic>, String> {
        // Create a temporary Cargo.toml for single file checking
        let temp_dir = self.target_dir.join("lsp-check");
        std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

        // Build rustc command
        let output = Command::new(&self.rustc_path)
            .arg(file_path)
            .arg("--crate-type")
            .arg("lib")
            .arg("--edition")
            .arg(&self.edition)
            .arg("--error-format")
            .arg("json")
            .arg("-o")
            .arg("/dev/null") // Don't generate output
            .current_dir(&self.project_root)
            .output()
            .map_err(|e| format!("Failed to execute rustc: {}", e))?;

        // Parse rustc JSON output
        if output.status.success() {
            Ok(Vec::new())
        } else {
            let diagnostics = self.parse_rustc_output(&String::from_utf8_lossy(&output.stderr))?;
            Ok(diagnostics)
        }
    }

    /// Check the entire project
    ///
    /// # Returns
    ///
    /// List of all diagnostics found in the project
    pub async fn check_project(&self) -> Result<Vec<Diagnostic>, String> {
        // Use cargo check instead of rustc for better integration
        let output = Command::new("cargo")
            .arg("check")
            .arg("--message-format")
            .arg("json")
            .arg("--color")
            .arg("never")
            .current_dir(&self.project_root)
            .output()
            .map_err(|e| format!("Failed to execute cargo check: {}", e))?;

        // Parse cargo JSON output
        let diagnostics = self.parse_cargo_output(&String::from_utf8_lossy(&output.stdout))?;
        Ok(diagnostics)
    }

    /// Parse rustc JSON output
    ///
    /// # Arguments
    ///
    /// * `output` - Raw rustc output
    ///
    /// # Returns
    ///
    /// Parsed diagnostics
    fn parse_rustc_output(&self, output: &str) -> Result<Vec<Diagnostic>, String> {
        let mut diagnostics = Vec::new();

        for line in output.lines() {
            if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(diagnostic) = self.parse_rustc_message(&json_msg) {
                    diagnostics.push(diagnostic);
                }
            }
        }

        Ok(diagnostics)
    }

    /// Parse cargo check JSON output
    ///
    /// # Arguments
    ///
    /// * `output` - Raw cargo output
    ///
    /// # Returns
    ///
    /// Parsed diagnostics
    fn parse_cargo_output(&self, output: &str) -> Result<Vec<Diagnostic>, String> {
        let mut diagnostics = Vec::new();

        for line in output.lines() {
            if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(diagnostic) = self.parse_cargo_message(&json_msg) {
                    diagnostics.push(diagnostic);
                }
            }
        }

        Ok(diagnostics)
    }

    /// Parse a single rustc JSON message
    ///
    /// # Arguments
    ///
    /// * `message` - Parsed JSON message
    ///
    /// # Returns
    ///
    /// Diagnostic if the message contains one
    fn parse_rustc_message(&self, message: &serde_json::Value) -> Option<Diagnostic> {
        // Rustc JSON message format:
        // {"message":"...","level":"error"," spans":[{"file_name":"...","line_start":1,...}]}

        let level = message.get("level")?.as_str()?;
        let severity = match level {
            "error" | "error: aborting" => Some(DiagnosticSeverity::ERROR),
            "warning" => Some(DiagnosticSeverity::WARNING),
            _ => None,
        };

        if severity.is_none() {
            return None;
        }

        let spans = message.get("spans")?.as_array()?;
        if spans.is_empty() {
            return None;
        }

        let span = &spans[0];
        let file_name = span.get("file_name")?.as_str()?;
        let line_start = span.get("line_start")?.as_u64()? as u32;
        let col_start = span.get("column_start")?.as_u64()? as u32;
        let line_end = span.get("line_end").as_u64().unwrap_or(line_start as u64) as u32;
        let col_end = span.get("column_end").as_u64().unwrap_or(col_start as u64) as u32;

        let rendered = message.get("message")?.as_str()?;

        Some(Diagnostic {
            range: Range {
                start: Position {
                    line: line_start.saturating_sub(1),
                    character: col_start.saturating_sub(1),
                },
                end: Position {
                    line: line_end.saturating_sub(1),
                    character: col_end.saturating_sub(1),
                },
            },
            severity,
            code: None,
            source: Some("rustc".to_string()),
            message: rendered.to_string(),
            related_information: None,
            tags: None,
            data: None,
        })
    }

    /// Parse a single cargo check JSON message
    ///
    /// # Arguments
    ///
    /// * `message` - Parsed JSON message
    ///
    /// # Returns
    ///
    /// Diagnostic if the message contains one
    fn parse_cargo_message(&self, message: &serde_json::Value) -> Option<Diagnostic> {
        // Cargo JSON message format:
        // {"message":"...","level":"error","target":{"src_path":"...","span":{...}}}

        let level = message.get("level")?.as_str()?;
        let severity = match level {
            "error" => Some(DiagnosticSeverity::ERROR),
            "warning" => Some(DiagnosticSeverity::WARNING),
            _ => None,
        };

        if severity.is_none() {
            return None;
        }

        let rendered = message.get("message")?.as_object()?.get("rendered")?.as_str()?;

        // Try to extract span information
        let (range, file_path) = if let Some(target) = message.get("target") {
            if let Some(span) = target.get("span") {
                let file_name = span.get("file_name")?.as_str()?;
                let line_start = span.get("line_start")?.as_u64()? as u32;
                let col_start = span.get("column_start")?.as_u64()? as u32;
                let line_end = span.get("line_end").as_u64().unwrap_or(line_start as u64) as u32;
                let col_end = span.get("column_end").as_u64().unwrap_or(col_start as u64) as u32;

                (
                    Range {
                        start: Position {
                            line: line_start.saturating_sub(1),
                            character: col_start.saturating_sub(1),
                        },
                        end: Position {
                            line: line_end.saturating_sub(1),
                            character: col_end.saturating_sub(1),
                        },
                    },
                    file_name.to_string(),
                )
            } else {
                // Fallback: no span information
                return None;
            }
        } else {
            return None;
        };

        Some(Diagnostic {
            range,
            severity,
            code: None,
            source: Some("rustc".to_string()),
            message: rendered.to_string(),
            related_information: None,
            tags: None,
            data: None,
        })
    }

    /// Get quick fixes for a diagnostic
    ///
    /// # Arguments
    ///
    /// * `diagnostic` - The diagnostic to get fixes for
    ///
    /// # Returns
    ///
    /// List of suggested fixes
    pub fn get_quick_fixes(&self, diagnostic: &Diagnostic) -> Vec<String> {
        let mut fixes = Vec::new();

        // Analyze the diagnostic message and suggest fixes
        let message = &diagnostic.message;

        // Missing semicolon
        if message.contains("expected one of") && message.contains("`;`") {
            fixes.push("Add semicolon at the end of the statement".to_string());
        }

        // Missing import
        if message.contains("cannot find") || message.contains("not found in this scope") {
            if let Some(type_name) = self.extract_type_name(message) {
                fixes.push(format!("Consider adding `use {};", type_name));
            }
        }

        // Type mismatch
        if message.contains("mismatched types") {
            fixes.push("Check the types and ensure they match".to_string());
            fixes.push("Consider using `as` to convert types".to_string());
        }

        // Missing trait implementation
        if message.contains("not implemented") {
            if let Some(trait_name) = self.extract_trait_name(message) {
                fixes.push(format!("Implement the {} trait", trait_name));
            }
        }

        fixes
    }

    /// Extract type name from error message
    fn extract_type_name(&self, message: &str) -> Option<String> {
        // Simple extraction logic
        if let Some(start) = message.find("cannot find type `") {
            let start = start + "cannot find type `".len();
            if let Some(end) = message[start..].find('`') {
                return Some(message[start..start + end].to_string());
            }
        }
        None
    }

    /// Extract trait name from error message
    fn extract_trait_name(&self, message: &str) -> Option<String> {
        // Simple extraction logic
        if let Some(start) = message.find("`") {
            if let Some(end) = message[start + 1..].find('`') {
                return Some(message[start + 1..start + 1 + end].to_string());
            }
        }
        None
    }

    /// Convert file path to URL
    ///
    /// # Arguments
    ///
    /// * `file_path` - File path relative to project root
    ///
    /// # Returns
    ///
    /// LSP URL
    pub fn file_path_to_url(&self, file_path: &str) -> Url {
        let absolute_path = self.project_root.join(file_path);
        Url::from_file_path(absolute_path).unwrap()
    }
}

impl Default for RustcDiagnostics {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rustc_diagnostics_creation() {
        let diagnostics = RustcDiagnostics::new(PathBuf::from("."));
        assert_eq!(diagnostics.edition, "2021");
    }

    #[test]
    fn test_with_edition() {
        let diagnostics = RustcDiagnostics::new(PathBuf::from(".")).with_edition("2018");
        assert_eq!(diagnostics.edition, "2018");
    }

    #[test]
    fn test_extract_type_name() {
        let diagnostics = RustcDiagnostics::new(PathBuf::from("."));
        let message = "error[E0433]: failed to resolve: use of undeclared type `Vec`";
        let result = diagnostics.extract_type_name(message);
        assert_eq!(result, Some("Vec".to_string()));
    }

    #[test]
    fn test_extract_trait_name() {
        let diagnostics = RustcDiagnostics::new(PathBuf::from("."));
        let message = "the trait `Clone` is not implemented";
        let result = diagnostics.extract_trait_name(message);
        assert_eq!(result, Some("Clone".to_string()));
    }

    #[test]
    fn test_get_quick_fixes() {
        let diagnostics = RustcDiagnostics::new(PathBuf::from("."));

        let diag = Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 10,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            source: Some("rustc".to_string()),
            message:
                "expected one of 12 possible variables\nfound `Vec`\nnote: consider adding `;`"
                    .to_string(),
            related_information: None,
            tags: None,
            data: None,
        };

        let fixes = diagnostics.get_quick_fixes(&diag);
        assert!(!fixes.is_empty());
    }
}
