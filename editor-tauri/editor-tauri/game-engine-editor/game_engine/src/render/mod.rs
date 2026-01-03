//! # Rendering subsystems
//!
//! This module contains all rendering-related functionality, including the
//! Nanite virtual geometry system.

pub mod nanite;

// Re-export for convenience
pub use nanite::*;
