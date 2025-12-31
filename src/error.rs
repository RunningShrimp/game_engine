//! Error types for the game engine

use std::fmt;

/// Game engine error type
#[derive(Debug, Clone)]
pub enum Error {
    /// IO error
    IoError(String),
    /// Plugin not found
    PluginNotFound(String),
    /// Plugin load error
    PluginLoadError(String),
    /// Plugin error
    PluginError(String),
    /// Plugin incompatible error
    PluginIncompatibleError {
        plugin: String,
        required: String,
        current: String,
    },
    /// Plugin dependency error
    PluginDependencyError {
        plugin: String,
        missing: String,
    },
    /// Serialization error
    SerializationError(String),
    /// Deserialization error
    DeserializationError(String),
    /// Render error
    RenderError(String),
    /// Resource error
    ResourceError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(msg) => write!(f, "IO error: {}", msg),
            Error::PluginNotFound(name) => write!(f, "Plugin not found: {}", name),
            Error::PluginLoadError(msg) => write!(f, "Plugin load error: {}", msg),
            Error::PluginError(msg) => write!(f, "Plugin error: {}", msg),
            Error::PluginIncompatibleError { plugin, required, current } => {
                write!(f, "Plugin {} is incompatible: requires engine version {}, current version is {}",
                       plugin, required, current)
            }
            Error::PluginDependencyError { plugin, missing } => {
                write!(f, "Plugin {} depends on {} which is not available", plugin, missing)
            }
            Error::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Error::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            Error::RenderError(msg) => write!(f, "Render error: {}", msg),
            Error::ResourceError(msg) => write!(f, "Resource error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
