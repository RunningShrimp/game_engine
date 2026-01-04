//! # Signature Help Module
//!
//! Provides function signature information and parameter hints.

use std::collections::HashMap;
use tower_lsp::lsp_types::{
    ParameterInformation, Position, Range, SignatureHelp, SignatureHelpContext,
    SignatureInformation, SignatureTriggerKind,
};

/// Function signature information
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,

    /// Full signature label
    pub label: String,

    /// Documentation
    pub documentation: Option<String>,

    /// Parameters
    pub parameters: Vec<Parameter>,

    /// Return type
    pub return_type: String,
}

/// Parameter information
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Parameter name
    pub name: String,

    /// Parameter label (for display)
    pub label: String,

    /// Parameter type
    pub param_type: String,

    /// Documentation
    pub documentation: Option<String>,

    /// Whether this parameter has a default value
    pub has_default: bool,
}

/// Signature help provider
pub struct SignatureHelpProvider {
    /// Registered function signatures
    signatures: HashMap<String, Vec<FunctionSignature>>,

    /// Common Rust standard library signatures
    std_signatures: HashMap<String, Vec<FunctionSignature>>,
}

impl SignatureHelpProvider {
    /// Create a new signature help provider
    pub fn new() -> Self {
        let mut provider = Self {
            signatures: HashMap::new(),
            std_signatures: HashMap::new(),
        };

        // Register standard library signatures
        provider.register_std_signatures();

        provider
    }

    /// Register a function signature
    ///
    /// # Arguments
    ///
    /// * `signature` - The function signature to register
    pub fn register_signature(&mut self, signature: FunctionSignature) {
        self.signatures
            .entry(signature.name.clone())
            .or_insert_with(Vec::new)
            .push(signature);
    }

    /// Get signature help for a function call
    ///
    /// # Arguments
    ///
    /// * `file_path` - File path
    /// * `line` - Current line text
    /// * `line_number` - Line number (0-based)
    /// * `character` - Character position (0-based)
    ///
    /// # Returns
    ///
    /// Signature help if available
    pub fn get_signature_help(
        &self,
        _file_path: &str,
        line: &str,
        line_number: usize,
        character: usize,
    ) -> Option<SignatureHelp> {
        // Extract function name and parameter index
        let (func_name, param_index) = self.parse_function_call(line, character)?;

        // Look up function signature
        let signatures = self
            .signatures
            .get(&func_name)
            .or_else(|| self.std_signatures.get(&func_name))?;

        // Find the best matching signature
        let signature = self.find_best_signature(signatures, param_index)?;

        // Convert to LSP SignatureHelp
        let active_parameter = if param_index < signature.parameters.len() {
            Some(param_index as u32)
        } else {
            None
        };

        Some(SignatureHelp {
            signatures: vec![SignatureInformation {
                label: signature.label.clone(),
                documentation: signature.documentation.clone().map(|d| {
                    tower_lsp::lsp_types::Documentation::MarkupContent(
                        tower_lsp::lsp_types::MarkupContent {
                            kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                            value: d,
                        },
                    )
                }),
                parameters: Some(
                    signature
                        .parameters
                        .iter()
                        .map(|p| ParameterInformation {
                            label: tower_lsp::lsp_types::ParameterLabel::Simple(p.label.clone()),
                            documentation: p.documentation.clone().map(|d| {
                                tower_lsp::lsp_types::Documentation::MarkupContent(
                                    tower_lsp::lsp_types::MarkupContent {
                                        kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                                        value: d,
                                    },
                                )
                            }),
                        })
                        .collect(),
                ),
                active_parameter,
            }],
            active_parameter,
        })
    }

    /// Trigger signature help on specific characters
    ///
    /// # Arguments
    ///
    /// * `context` - Signature help context
    ///
    /// # Returns
    ///
    /// Whether to trigger signature help
    pub fn should_trigger(&self, context: &SignatureHelpContext) -> bool {
        // Trigger on '(' and ','
        if context.trigger_character.is_some() {
            let trigger = context.trigger_character.as_ref().unwrap();
            return trigger == "(" || trigger == ",";
        }

        // Also trigger when user types inside function call
        context.is_retrigger
    }

    /// Parse function call to extract name and parameter index
    ///
    /// # Arguments
    ///
    /// * `line` - Line text
    /// * `character` - Cursor position
    ///
    /// # Returns
    ///
    /// Function name and parameter index
    fn parse_function_call(&self, line: &str, character: usize) -> Option<(String, usize)> {
        // Find the opening parenthesis
        let mut paren_count = 0;
        let mut paren_pos = None;

        for (i, c) in line.char_indices() {
            if c == '(' {
                paren_count += 1;
                if i < character {
                    paren_pos = Some(i);
                }
            } else if c == ')' {
                paren_count -= 1;
            }
        }

        let open_paren = paren_pos?;

        // Extract function name
        let before_paren = &line[..open_paren];
        let func_name = if let Some(last_space) = before_paren.rfind(' ') {
            before_paren[last_space + 1..].trim().to_string()
        } else if let Some(last_dot) = before_paren.rfind('.') {
            before_paren[last_dot + 1..].trim().to_string()
        } else if let Some(last_colon) = before_paren.rfind("::") {
            before_paren[last_colon + 2..].trim().to_string()
        } else {
            before_paren.trim().to_string()
        };

        // Calculate parameter index
        let substring = &line[open_paren..character];
        let commas = substring.matches(',').count();

        // Adjust for nested parentheses
        let param_index = commas;

        Some((func_name, param_index))
    }

    /// Find the best matching signature for a parameter count
    ///
    /// # Arguments
    ///
    /// * `signatures` - Available signatures
    /// * `param_index` - Current parameter index
    ///
    /// # Returns
    ///
    /// Best matching signature
    fn find_best_signature(
        &self,
        signatures: &[FunctionSignature],
        param_index: usize,
    ) -> Option<&FunctionSignature> {
        // Find signature with matching parameter count
        // or the closest match
        let mut best_match = None;
        let mut_best_diff = usize::MAX;

        for sig in signatures {
            let param_count = sig.parameters.len();

            // Exact match
            if param_index < param_count {
                return Some(sig);
            }

            // Track closest match
            let diff = if param_index >= param_count {
                param_index - param_count
            } else {
                param_count - param_index
            };

            if diff < _best_diff {
                _best_diff = diff;
                best_match = Some(sig);
            }
        }

        best_match
    }

    /// Register standard library signatures
    fn register_std_signatures(&mut self) {
        // Vec::new()
        self.std_signatures.insert(
            "Vec::new".to_string(),
            vec![FunctionSignature {
                name: "Vec::new".to_string(),
                label: "fn Vec::new() -> Vec<T>".to_string(),
                documentation: Some("Creates a new, empty Vec.".to_string()),
                parameters: vec![],
                return_type: "Vec<T>".to_string(),
            }],
        );

        // Vec::with_capacity()
        self.std_signatures.insert(
            "Vec::with_capacity".to_string(),
            vec![FunctionSignature {
                name: "Vec::with_capacity".to_string(),
                label: "fn Vec::with_capacity(capacity: usize) -> Vec<T>".to_string(),
                documentation: Some(
                    "Creates a new Vec<T> with the specified capacity.".to_string(),
                ),
                parameters: vec![Parameter {
                    name: "capacity".to_string(),
                    label: "capacity: usize".to_string(),
                    param_type: "usize".to_string(),
                    documentation: Some("The desired capacity.".to_string()),
                    has_default: false,
                }],
                return_type: "Vec<T>".to_string(),
            }],
        );

        // HashMap::new()
        self.std_signatures.insert(
            "HashMap::new".to_string(),
            vec![FunctionSignature {
                name: "HashMap::new".to_string(),
                label: "fn HashMap::new() -> HashMap<K, V>".to_string(),
                documentation: Some("Creates a new, empty HashMap.".to_string()),
                parameters: vec![],
                return_type: "HashMap<K, V>".to_string(),
            }],
        );

        // println!
        self.std_signatures.insert(
            "println".to_string(),
            vec![FunctionSignature {
                name: "println".to_string(),
                label: "fn println!(fmt: ...)".to_string(),
                documentation: Some("Prints to the standard output with a newline.".to_string()),
                parameters: vec![Parameter {
                    name: "fmt".to_string(),
                    label: "fmt: ...".to_string(),
                    param_type: "...".to_string(),
                    documentation: Some("Formatting arguments.".to_string()),
                    has_default: false,
                }],
                return_type: "()".to_string(),
            }],
        );
    }
}

impl Default for SignatureHelpProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_help_provider_creation() {
        let provider = SignatureHelpProvider::new();
        assert!(!provider.std_signatures.is_empty());
    }

    #[test]
    fn test_register_signature() {
        let mut provider = SignatureHelpProvider::new();
        let signature = FunctionSignature {
            name: "test_func".to_string(),
            label: "fn test_func(x: i32)".to_string(),
            documentation: None,
            parameters: vec![],
            return_type: "()".to_string(),
        };

        provider.register_signature(signature);
        assert!(provider.signatures.contains_key("test_func"));
    }

    #[test]
    fn test_parse_function_call() {
        let provider = SignatureHelpProvider::new();

        // Simple function call
        let result = provider.parse_function_call("println!(\"test\", x)", 12);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, "println");

        // Method call
        let result = provider.parse_function_call("vec.push(42)", 10);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, "push");
    }

    #[test]
    fn test_should_trigger() {
        let provider = SignatureHelpProvider::new();

        // Trigger on '('
        let context = SignatureHelpContext {
            trigger_kind: SignatureTriggerKind::Invoked,
            trigger_character: Some("(".to_string()),
            is_retrigger: false,
        };
        assert!(provider.should_trigger(&context));

        // Trigger on ','
        let context = SignatureHelpContext {
            trigger_kind: SignatureTriggerKind::Invoked,
            trigger_character: Some(",".to_string()),
            is_retrigger: false,
        };
        assert!(provider.should_trigger(&context));
    }

    #[test]
    fn test_get_signature_help() {
        let provider = SignatureHelpProvider::new();

        // Test Vec::new
        let result = provider.get_signature_help("test.rs", "let v = Vec::new();", 0, 13);
        assert!(result.is_some());
        let sig_help = result.unwrap();
        assert_eq!(sig_help.signatures.len(), 1);
        assert_eq!(sig_help.signatures[0].label, "fn Vec::new() -> Vec<T>");
    }

    #[test]
    fn test_find_best_signature() {
        let provider = SignatureHelpProvider::new();

        let sig1 = FunctionSignature {
            name: "test".to_string(),
            label: "fn test()".to_string(),
            documentation: None,
            parameters: vec![],
            return_type: "()".to_string(),
        };

        let sig2 = FunctionSignature {
            name: "test".to_string(),
            label: "fn test(x: i32)".to_string(),
            documentation: None,
            parameters: vec![Parameter {
                name: "x".to_string(),
                label: "x: i32".to_string(),
                param_type: "i32".to_string(),
                documentation: None,
                has_default: false,
            }],
            return_type: "()".to_string(),
        };

        let signatures = vec![sig1, sig2];

        // Should match sig2 for parameter index 0
        let result = provider.find_best_signature(&signatures, 0);
        assert!(result.is_some());
        assert_eq!(result.unwrap().parameters.len(), 1);
    }
}
