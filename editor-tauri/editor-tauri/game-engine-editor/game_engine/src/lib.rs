//! # Game Engine with Nanite Virtual Geometry
//!
//! This is a modular game engine featuring advanced rendering capabilities,
//! including Nanite-style virtual geometry for high-poly mesh rendering.

pub mod render;

// Re-export common types
pub use render::nanite::*;

// LSP support (requires 'lsp' feature)
#[cfg(feature = "lsp")]
pub mod lsp;

pub mod prelude {
    //! Common imports for convenience

    pub use crate::render::nanite::{
        NaniteSystem,
        NaniteConfig,
        Camera,
        ClusterHierarchy,
        QualityPreset,
        PerformanceStats,
    };
}
