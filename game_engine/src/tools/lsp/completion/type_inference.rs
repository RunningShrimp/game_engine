//! # Type Inference Engine
//!
//! Infers types for variables, expressions, and function calls.

use std::collections::HashMap;
use tower_lsp::lsp_types::Url;

/// Type information
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    /// Type name
    pub name: String,

    /// Full type path (e.g., `game_engine::ecs::Entity`)
    pub full_path: String,

    /// Type kind
    pub kind: TypeKind,

    /// Whether this type is mutable
    pub is_mutable: bool,

    /// Type parameters
    pub type_params: Vec<String>,
}

/// Type kind
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    /// Primitive type (i32, f32, bool, etc.)
    Primitive,

    /// Struct
    Struct,

    /// Enum
    Enum,

    /// Function
    Function,

    /// Reference
    Reference(Box<TypeKind>),

    /// Slice
    Slice(Box<TypeKind>),

    /// Array
    Array(Box<TypeKind>, usize),

    /// Option
    Option(Box<TypeKind>),

    /// Result
    Result(Box<TypeKind>, Box<TypeKind>),

    /// Vector
    Vec(Box<TypeKind>),

    /// HashMap
    HashMap(Box<TypeKind>, Box<TypeKind>),
}

/// Type constraint
#[derive(Debug, Clone)]
pub struct TypeConstraint {
    /// Variable name
    pub variable: String,

    /// Expected type
    pub expected_type: TypeInfo,

    /// Actual type (if known)
    pub actual_type: Option<TypeInfo>,
}

/// Type inference engine
pub struct TypeInferenceEngine {
    /// Symbol table: maps variable names to their types
    symbol_table: HashMap<String, TypeInfo>,

    /// Type constraints
    type_constraints: Vec<TypeConstraint>,

    /// Function signatures
    function_signatures: HashMap<String, FunctionSignature>,

    /// Current file being analyzed
    current_file: Option<String>,
}

/// Function signature information
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,

    /// Parameters
    pub parameters: Vec<Parameter>,

    /// Return type
    pub return_type: TypeInfo,

    /// Full path
    pub full_path: String,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Parameter name
    pub name: String,

    /// Parameter type
    pub param_type: TypeInfo,

    /// Whether this parameter has a default value
    pub has_default: bool,
}

impl TypeInferenceEngine {
    /// Create a new type inference engine
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            type_constraints: Vec::new(),
            function_signatures: HashMap::new(),
            current_file: None,
        }
    }

    /// Set the current file being analyzed
    pub fn set_current_file(&mut self, file_path: &str) {
        self.current_file = Some(file_path.to_string());
    }

    /// Infer the type of an expression
    ///
    /// # Arguments
    ///
    /// * `expression` - The expression to infer
    ///
    /// # Returns
    ///
    /// The inferred type, if successful
    pub fn infer_type(&self, expression: &str) -> Option<TypeInfo> {
        // Parse the expression and infer its type
        // This is a simplified implementation

        // Handle literals
        if let Some(int_value) = self.parse_integer_literal(expression) {
            return Some(TypeInfo {
                name: "i32".to_string(),
                full_path: "i32".to_string(),
                kind: TypeKind::Primitive,
                is_mutable: false,
                type_params: vec![],
            });
        }

        if let Some(float_value) = self.parse_float_literal(expression) {
            return Some(TypeInfo {
                name: "f32".to_string(),
                full_path: "f32".to_string(),
                kind: TypeKind::Primitive,
                is_mutable: false,
                type_params: vec![],
            });
        }

        if expression == "true" || expression == "false" {
            return Some(TypeInfo {
                name: "bool".to_string(),
                full_path: "bool".to_string(),
                kind: TypeKind::Primitive,
                is_mutable: false,
                type_params: vec![],
            });
        }

        // Handle variable references
        if self.symbol_table.contains_key(expression) {
            return self.symbol_table.get(expression).cloned();
        }

        // Handle function calls
        if let Some(func_name) = self.extract_function_name(expression) {
            return self.infer_function_return_type(&func_name);
        }

        None
    }

    /// Infer the type of a variable
    ///
    /// # Arguments
    ///
    /// * `variable_name` - The variable name
    ///
    /// # Returns
    ///
    /// The variable's type, if known
    pub fn infer_variable_type(&self, variable_name: &str) -> Option<TypeInfo> {
        self.symbol_table.get(variable_name).cloned()
    }

    /// Add a variable to the symbol table
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    /// * `type_info` - Variable type information
    pub fn add_variable(&mut self, name: String, type_info: TypeInfo) {
        self.symbol_table.insert(name, type_info);
    }

    /// Add a type constraint
    ///
    /// # Arguments
    ///
    /// * `constraint` - The type constraint to add
    pub fn add_constraint(&mut self, constraint: TypeConstraint) {
        self.type_constraints.push(constraint);
    }

    /// Register a function signature
    ///
    /// # Arguments
    ///
    /// * `signature` - The function signature to register
    pub fn register_function(&mut self, signature: FunctionSignature) {
        self.function_signatures.insert(signature.name.clone(), signature);
    }

    /// Get function signature by name
    ///
    /// # Arguments
    ///
    /// * `name` - Function name
    ///
    /// # Returns
    ///
    /// The function signature, if registered
    pub fn get_function_signature(&self, name: &str) -> Option<&FunctionSignature> {
        self.function_signatures.get(name)
    }

    /// Infer the return type of a function call
    ///
    /// # Arguments
    ///
    /// * `func_name` - Function name
    ///
    /// # Returns
    ///
    /// The function's return type, if known
    fn infer_function_return_type(&self, func_name: &str) -> Option<TypeInfo> {
        self.function_signatures.get(func_name).map(|sig| sig.return_type.clone())
    }

    /// Parse an integer literal
    fn parse_integer_literal(&self, expr: &str) -> Option<i64> {
        expr.trim().parse().ok()
    }

    /// Parse a float literal
    fn parse_float_literal(&self, expr: &str) -> Option<f64> {
        expr.trim().parse().ok()
    }

    /// Extract function name from a function call expression
    fn extract_function_name(&self, expr: &str) -> Option<String> {
        // Simplified: extract "foo" from "foo(...)"
        if let Some(pos) = expr.find('(') {
            let func_part = &expr[..pos];
            if let Some(last_dot) = func_part.rfind('.') {
                Some(func_part[last_dot + 1..].to_string())
            } else {
                Some(func_part.to_string())
            }
        } else {
            None
        }
    }

    /// Analyze a line of code and update symbol table
    ///
    /// # Arguments
    ///
    /// * `line` - The line of code to analyze
    pub fn analyze_line(&mut self, line: &str) {
        // Detect variable declarations: `let x: Type = ...;` or `let x = ...;`
        if line.trim().starts_with("let ") {
            self.analyze_let_binding(line);
        }

        // Detect function calls and update constraints
        if line.contains('(') && line.contains(')') {
            self.analyze_function_call(line);
        }
    }

    /// Analyze a let binding
    fn analyze_let_binding(&mut self, line: &str) {
        // Parse: `let x: Type = expr;` or `let mut x: Type = expr;` or `let x = expr;`
        let trimmed = line.trim();
        let is_mut = trimmed.starts_with("let mut ");
        let without_let = trimmed
            .strip_prefix("let ")
            .unwrap()
            .strip_prefix("mut ")
            .unwrap_or(trimmed.strip_prefix("let ").unwrap());

        // Split at '='
        if let Some(eq_pos) = without_let.find('=') {
            let var_part = &without_let[..eq_pos].trim();
            let expr_part = &without_let[eq_pos + 1..].trim().trim_end_matches(';');

            // Extract variable name and optional type annotation
            let (var_name, type_annotation) = if let Some(colon_pos) = var_part.find(':') {
                (
                    var_part[..colon_pos].trim().to_string(),
                    Some(var_part[colon_pos + 1..].trim().to_string()),
                )
            } else {
                (var_part.to_string(), None)
            };

            // Infer type from expression or annotation
            let type_info = if let Some(type_str) = type_annotation {
                TypeInfo {
                    name: type_str.clone(),
                    full_path: type_str,
                    kind: TypeKind::Struct, // Default to Struct
                    is_mutable: is_mut,
                    type_params: vec![],
                }
            } else {
                // Try to infer from expression
                self.infer_type(expr_part).unwrap_or_else(|| TypeInfo {
                    name: "_".to_string(),
                    full_path: "_".to_string(),
                    kind: TypeKind::Struct,
                    is_mutable: is_mut,
                    type_params: vec![],
                })
            };

            self.add_variable(var_name, type_info);
        }
    }

    /// Analyze a function call
    fn analyze_function_call(&mut self, line: &str) {
        // This is a placeholder for more sophisticated analysis
        // In a full implementation, this would:
        // 1. Parse the function call
        // 2. Look up the function signature
        // 3. Add type constraints for arguments
    }

    /// Suggest completions based on type context
    ///
    /// # Arguments
    ///
    /// * `partial` - Partial input
    /// * `expected_type` - Expected type (optional)
    ///
    /// # Returns
    ///
    /// List of suggested completions
    pub fn suggest_completions(
        &self,
        partial: &str,
        expected_type: Option<&TypeInfo>,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Add variables that match the expected type
        if let Some(expected) = expected_type {
            for (var_name, var_type) in &self.symbol_table {
                if var_type.name == expected.name
                    || partial.is_empty()
                    || var_name.starts_with(partial)
                {
                    suggestions.push(var_name.clone());
                }
            }
        }

        // Add functions that match the partial input
        for func_name in self.function_signatures.keys() {
            if partial.is_empty() || func_name.starts_with(partial) {
                suggestions.push(func_name.clone());
            }
        }

        suggestions
    }
}

impl Default for TypeInferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_inference_engine_creation() {
        let engine = TypeInferenceEngine::new();
        assert!(engine.symbol_table.is_empty());
    }

    #[test]
    fn test_add_variable() {
        let mut engine = TypeInferenceEngine::new();
        let type_info = TypeInfo {
            name: "i32".to_string(),
            full_path: "i32".to_string(),
            kind: TypeKind::Primitive,
            is_mutable: false,
            type_params: vec![],
        };

        engine.add_variable("x".to_string(), type_info.clone());
        assert_eq!(engine.infer_variable_type("x"), Some(type_info));
    }

    #[test]
    fn test_infer_integer_literal() {
        let engine = TypeInferenceEngine::new();
        let result = engine.infer_type("42");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "i32");
    }

    #[test]
    fn test_infer_float_literal() {
        let engine = TypeInferenceEngine::new();
        let result = engine.infer_type("3.14");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "f32");
    }

    #[test]
    fn test_infer_bool_literal() {
        let engine = TypeInferenceEngine::new();
        let result = engine.infer_type("true");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "bool");
    }

    #[test]
    fn test_analyze_let_binding() {
        let mut engine = TypeInferenceEngine::new();
        engine.analyze_line("let x: i32 = 42;");
        assert!(engine.infer_variable_type("x").is_some());
        assert_eq!(engine.infer_variable_type("x").unwrap().name, "i32");
    }

    #[test]
    fn test_analyze_let_binding_with_inference() {
        let mut engine = TypeInferenceEngine::new();
        engine.analyze_line("let x = 42;");
        assert!(engine.infer_variable_type("x").is_some());
        assert_eq!(engine.infer_variable_type("x").unwrap().name, "i32");
    }
}
