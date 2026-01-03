//! Game Engine Language Server Protocol (LSP) Module
//!
//! Provides intelligent code completion, go-to-definition,
//! hover information, and other IDE features for game engine projects.

pub mod server;
pub mod api_index;
pub mod symbol_info;
pub mod completion;

pub use server::GameEngineServer;
pub use api_index::ApiIndex;
pub use symbol_info::{
    SymbolInfo, FunctionSignature, Parameter, StructField, EnumVariant, VariantData,
    TraitMethod, TraitInfo, CompletionItemData,
};
pub use completion::CompletionProvider;
