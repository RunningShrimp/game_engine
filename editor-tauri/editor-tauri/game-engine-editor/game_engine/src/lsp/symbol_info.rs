//! Symbol Information Structures
//!
//! Defines structures for storing and managing symbol information
//! extracted from the game engine codebase.

use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::*;

/// Symbol information extracted from source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    /// Symbol name (e.g., "Entity", "Transform", "update")
    pub name: String,

    /// Symbol kind (struct, function, enum, etc.)
    pub kind: SymbolKind,

    /// Documentation in markdown format
    pub documentation: String,

    /// Full range of the symbol
    pub range: Range,

    /// Range for selecting the symbol name (usually smaller than range)
    pub selection_range: Range,

    /// Child symbols (if any)
    pub children: Option<Vec<SymbolInfo>>,
}

impl SymbolInfo {
    /// Create a new SymbolInfo
    pub fn new(
        name: String,
        kind: SymbolKind,
        documentation: String,
        range: Range,
        selection_range: Range,
    ) -> Self {
        Self {
            name,
            kind,
            documentation,
            range,
            selection_range,
            children: None,
        }
    }

    /// Create with children
    pub fn with_children(mut self, children: Vec<SymbolInfo>) -> Self {
        self.children = Some(children);
        self
    }

    /// Check if this symbol has children
    pub fn has_children(&self) -> bool {
        self.children
            .as_ref()
            .map(|c| !c.is_empty())
            .unwrap_or(false)
    }
}

/// Function signature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,

    /// Parameters
    pub parameters: Vec<Parameter>,

    /// Return type
    pub return_type: Option<String>,

    /// Documentation
    pub documentation: String,
}

/// Function parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,

    /// Parameter type
    pub type_name: String,

    /// Documentation
    pub documentation: Option<String>,
}

/// Struct field information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    /// Field name
    pub name: String,

    /// Field type
    pub type_name: String,

    /// Whether this field is public
    pub is_public: bool,

    /// Documentation
    pub documentation: Option<String>,
}

/// Enum variant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    /// Variant name
    pub name: String,

    /// Associated data (if any)
    pub data: Option<Vec<VariantData>>,

    /// Documentation
    pub documentation: Option<String>,
}

/// Data associated with an enum variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariantData {
    /// Unit variant (no data)
    Unit,
    /// Tuple variant with types
    Tuple(Vec<String>),
    /// Struct variant with named fields
    Struct(Vec<StructField>),
}

/// Trait method information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitMethod {
    /// Method name
    pub name: String,

    /// Parameters
    pub parameters: Vec<Parameter>,

    /// Return type
    pub return_type: Option<String>,

    /// Whether this is a required method (no default implementation)
    pub is_required: bool,

    /// Documentation
    pub documentation: Option<String>,
}

/// Trait information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitInfo {
    /// Trait name
    pub name: String,

    /// Generic parameters
    pub generics: Vec<String>,

    /// Super traits
    pub super_traits: Vec<String>,

    /// Methods defined by this trait
    pub methods: Vec<TraitMethod>,

    /// Documentation
    pub documentation: String,
}

/// Completion item data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItemData {
    /// Symbol kind
    pub kind: SymbolKind,

    /// Detail text (e.g., type signature)
    pub detail: Option<String>,

    /// Documentation in markdown format
    pub documentation: Option<String>,

    /// Insert text (if different from label)
    pub insert_text: Option<String>,

    /// Additional text edits (e.g., auto-import)
    pub additional_text_edits: Option<Vec<TextEdit>>,

    /// Command to execute after insertion
    pub command: Option<Command>,
}

impl CompletionItemData {
    /// Create a new CompletionItemData
    pub fn new(kind: SymbolKind) -> Self {
        Self {
            kind,
            detail: None,
            documentation: None,
            insert_text: None,
            additional_text_edits: None,
            command: None,
        }
    }

    /// Set detail text
    pub fn with_detail(mut self, detail: String) -> Self {
        self.detail = Some(detail);
        self
    }

    /// Set documentation
    pub fn with_documentation(mut self, documentation: String) -> Self {
        self.documentation = Some(documentation);
        self
    }

    /// Set insert text
    pub fn with_insert_text(mut self, insert_text: String) -> Self {
        self.insert_text = Some(insert_text);
        self
    }
}
