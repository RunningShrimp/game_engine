//! # Context-Aware Completion Provider
//!
//! Provides contextually relevant code completions based on code location and semantic analysis.

use std::collections::HashMap;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Position, Range};

/// Context information for completion
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// Current line text
    pub line: String,

    /// Current position in document
    pub position: Position,

    /// Current file path
    pub file_path: String,

    /// Whether we're inside a function
    pub in_function: bool,

    /// Whether we're inside a struct definition
    pub in_struct: bool,

    /// Whether we're inside an impl block
    pub in_impl: bool,

    /// Current function name (if in function)
    pub function_name: Option<String>,

    /// Current struct name (if in struct)
    pub struct_name: Option<String>,
}

/// Context-aware completion provider
pub struct ContextAwareProvider {
    /// Cached completions for different contexts
    cached_completions: HashMap<String, Vec<CompletionItem>>,

    /// Engine API registry for completions
    registry: Option<crate::tools::lsp::registry::EngineAPIRegistry>,
}

impl ContextAwareProvider {
    /// Create a new context-aware provider
    pub fn new() -> Self {
        Self {
            cached_completions: HashMap::new(),
            registry: None,
        }
    }

    /// Set the engine API registry
    pub fn set_registry(&mut self, registry: crate::tools::lsp::registry::EngineAPIRegistry) {
        self.registry = Some(registry);
    }

    /// Get contextually relevant completions
    ///
    /// # Arguments
    ///
    /// * `context` - The completion context
    ///
    /// # Returns
    ///
    /// List of relevant completion items
    pub async fn get_completions(&self, context: &CompletionContext) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Add context-specific completions based on location
        if context.in_function {
            completions.extend(self.get_function_context_completions(context));
        }

        if context.in_struct {
            completions.extend(self.get_struct_context_completions(context));
        }

        if context.in_impl {
            completions.extend(self.get_impl_context_completions(context));
        }

        // Add general completions
        completions.extend(self.get_general_completions(context));

        completions
    }

    /// Get completions relevant inside a function
    fn get_function_context_completions(
        &self,
        _context: &CompletionContext,
    ) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "return".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Return from function".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "let".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Declare a variable".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "if".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Conditional statement".to_string()),
                insert_text: Some("if ${1:condition} {\n    $0\n}".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "match".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Pattern matching".to_string()),
                insert_text: Some("match ${1:expression} {\n    ${2:pattern} => $0\n}".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "loop".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Infinite loop".to_string()),
                insert_text: Some("loop {\n    $0\n}".to_string()),
                ..Default::default()
            },
        ]
    }

    /// Get completions relevant inside a struct definition
    fn get_struct_context_completions(&self, _context: &CompletionContext) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "pub".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Public visibility".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "fn".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Method declaration".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "impl".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Implementation block".to_string()),
                ..Default::default()
            },
        ]
    }

    /// Get completions relevant inside an impl block
    fn get_impl_context_completions(&self, context: &CompletionContext) -> Vec<CompletionItem> {
        let mut completions = vec![
            CompletionItem {
                label: "pub fn".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Public method".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "fn new".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Constructor".to_string()),
                insert_text: Some("fn new() -> Self {\n    Self { $0 }\n}".to_string()),
                ..Default::default()
            },
        ];

        // Add self parameter suggestions
        if let Some(struct_name) = &context.struct_name {
            completions.push(CompletionItem {
                label: "&self".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Immutable reference to self".to_string()),
                ..Default::default()
            });
            completions.push(CompletionItem {
                label: "&mut self".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Mutable reference to self".to_string()),
                ..Default::default()
            });
        }

        completions
    }

    /// Get general completions
    fn get_general_completions(&self, _context: &CompletionContext) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "fn".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Function declaration".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "struct".to_string(),
                kind: Some(CompletionItemKind::STRUCT),
                detail: Some("Struct definition".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "enum".to_string(),
                kind: Some(CompletionItemKind::ENUM),
                detail: Some("Enum definition".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "mod".to_string(),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some("Module declaration".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "use".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Import statement".to_string()),
                ..Default::default()
            },
        ]
    }

    /// Analyze the current context from code
    ///
    /// # Arguments
    ///
    /// * `lines` - Lines of code before the current position
    /// * `position` - Current position
    ///
    /// # Returns
    ///
    /// The analyzed completion context
    pub fn analyze_context(&self, lines: &[String], position: &Position) -> CompletionContext {
        let line = lines.get(position.line as usize).cloned().unwrap_or_default();

        let mut in_function = false;
        let mut in_struct = false;
        let mut in_impl = false;
        let mut function_name = None;
        let mut struct_name = None;

        // Count braces to determine context
        let mut brace_count = 0;
        let mut fn_depth = None;
        let mut struct_depth = None;
        let mut impl_depth = None;

        for (i, code_line) in lines.iter().enumerate().rev() {
            // Count braces
            brace_count += code_line.matches('{').count() as i32;
            brace_count -= code_line.matches('}').count() as i32;

            // Check for function
            if code_line.contains("fn ") && fn_depth.is_none() {
                fn_depth = Some(brace_count);
                if let Some(start) = code_line.find("fn ") {
                    let rest = &code_line[start + 3..];
                    if let Some(end) = rest.find('(') {
                        function_name = Some(rest[..end].trim().to_string());
                    }
                }
            }

            // Check for struct
            if code_line.contains("struct ") && struct_depth.is_none() {
                struct_depth = Some(brace_count);
                if let Some(start) = code_line.find("struct ") {
                    let rest = &code_line[start + 7..];
                    let name: String =
                        rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                    struct_name = Some(name);
                }
            }

            // Check for impl
            if code_line.contains("impl ") && impl_depth.is_none() {
                impl_depth = Some(brace_count);
            }

            // Determine if we're in each context
            if let Some(depth) = fn_depth {
                in_function = brace_count > depth;
            }
            if let Some(depth) = struct_depth {
                in_struct = brace_count > depth;
            }
            if let Some(depth) = impl_depth {
                in_impl = brace_count > depth;
            }
        }

        CompletionContext {
            line,
            position: position.clone(),
            file_path: String::new(), // Will be set by caller
            in_function,
            in_struct,
            in_impl,
            function_name,
            struct_name,
        }
    }
}

impl Default for ContextAwareProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_aware_provider_creation() {
        let provider = ContextAwareProvider::new();
        assert!(provider.cached_completions.is_empty());
    }

    #[test]
    fn test_analyze_context() {
        let provider = ContextAwareProvider::new();
        let lines = vec!["fn test() {".to_string(), "    let x = 42;".to_string()];
        let position = Position {
            line: 1,
            character: 10,
        };

        let context = provider.analyze_context(&lines, &position);
        assert!(context.in_function);
        assert_eq!(context.function_name, Some("test".to_string()));
    }

    #[test]
    fn test_get_function_context_completions() {
        let provider = ContextAwareProvider::new();
        let context = CompletionContext {
            line: "    ".to_string(),
            position: Position {
                line: 0,
                character: 4,
            },
            file_path: "test.rs".to_string(),
            in_function: true,
            in_struct: false,
            in_impl: false,
            function_name: Some("test".to_string()),
            struct_name: None,
        };

        let completions = provider.get_function_context_completions(&context);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "return"));
    }
}
