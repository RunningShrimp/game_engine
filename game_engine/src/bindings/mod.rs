//  Language Binding Layer (FFI)
//
//  This module provides a unified binding layer design that can be reused
//  across different scripting languages (JavaScript, Python, Lua, etc.).
//
//  Architecture:
//  ```
//  ┌─────────────────────────────────────────────────────────────┐
//  │                    Host Languages                           │
//  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
//  │  │   JS    │  │ Python  │  │   Lua   │  │  WASM   │        │
//  │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘        │
//  │       │            │            │            │              │
//  │       v            v            v            v              │
//  │  ┌─────────────────────────────────────────────────────┐   │
//  │  │              Binding Adapters (per-language)        │   │
//  │  └───────────────────────┬─────────────────────────────┘   │
//  │                          │                                  │
//  │                          v                                  │
//  │  ┌─────────────────────────────────────────────────────┐   │
//  │  │         Unified Command/Event Protocol              │   │
//  │  │    (BindingCommand enum + BindingEvent enum)        │   │
//  │  └───────────────────────┬─────────────────────────────┘   │
//  │                          │                                  │
//  │                          v                                  │
//  │  ┌─────────────────────────────────────────────────────┐   │
//  │  │              Core Engine Services                    │   │
//  │  │  (ECS World, Renderer, Physics, Audio, etc.)        │   │
//  │  └─────────────────────────────────────────────────────┘   │
//  └─────────────────────────────────────────────────────────────┘
//  ```

/// JavaScript语言绑定 - 为JavaScript提供脚本绑定接口
pub mod js;
/// 绑定通信协议 - 定义脚本和引擎之间的通信协议
pub mod protocol;

pub use protocol::*;
