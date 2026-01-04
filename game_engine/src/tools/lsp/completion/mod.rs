//! # Code Completion Module
//!
//! Enhanced code completion with type inference, auto-import, and context awareness.

pub mod auto_import;
pub mod context_aware;
pub mod fuzzy_match;
pub mod type_inference;

pub use auto_import::AutoImportManager;
pub use context_aware::ContextAwareProvider;
pub use type_inference::TypeInferenceEngine;
