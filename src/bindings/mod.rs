//! Language Binding Layer (FFI)
//!
//! This module provides a unified binding layer design that can be reused
//! across different scripting languages (JavaScript, Python, Lua, etc.).
//!
//! Architecture:
//! ```
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Host Languages                           │
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
//! │  │   JS    │  │ Python  │  │   Lua   │  │  WASM   │        │
//! │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘        │
//! │       │            │            │            │              │
//! │       v            v            v            v              │
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │              Binding Adapters (per-language)        │   │
//! │  └───────────────────────┬─────────────────────────────┘   │
//! │                          │                                  │
//! │                          v                                  │
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │         Unified Command/Event Protocol              │   │
//! │  │    (BindingCommand enum + BindingEvent enum)        │   │
//! │  └───────────────────────┬─────────────────────────────┘   │
//! │                          │                                  │
//! │                          v                                  │
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │              Core Engine Services                    │   │
//! │  │  (ECS World, Renderer, Physics, Audio, etc.)        │   │
//! │  └─────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod js;
pub mod protocol;

pub use protocol::*;
