//! # Game Engine CLI Tool
//!
//! Command-line interface tool for creating and managing game engine projects.
//!
//! ## Features
//!
//! - Project scaffolding from templates
//! - Template management
//! - Project initialization
//!
//! ## Usage
//!
//! ```bash
//! # Create a new project from a template
//! game-engine new my-game --template basic
//!
//! # List available templates
//! game-engine template list
//!
//! # Initialize an existing project
//! game-engine init
//! ```
//!
//! ## Available Templates
//!
//! - `basic` - Basic game template with minimal setup
//! - `2d-platformer` - 2D platformer game template
//! - `3d-fps` - 3D first-person shooter template

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod dependency;
#[cfg(feature = "cli")]
pub mod project_generator;
#[cfg(feature = "cli")]
pub mod template;
#[cfg(feature = "cli")]
pub mod wizard;

#[cfg(feature = "cli")]
pub use cli::GameEngineCli;
#[cfg(feature = "cli")]
pub use wizard::{ProjectWizard, WizardConfig, WizardError, WizardResult};
