//! Error types for echidna.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for echidna operations.
pub type Result<T> = std::result::Result<T, EchidnaError>;

/// Errors that can occur during echidna operations.
#[derive(Error, Debug)]
pub enum EchidnaError {
    #[error("ChimeraX not found. Install ChimeraX or specify path with --chimerax")]
    ChimeraXNotFound,

    #[error("ChimeraX command failed: {0}")]
    ChimeraXCommandFailed(String),

    #[error("Not a valid bundle directory: {0} (missing pyproject.toml)")]
    NotBundleDirectory(PathBuf),

    #[error("No wheel found in dist/. Run 'echidna build' first.")]
    NoWheelFound,

    #[error("Directory already exists: {0}")]
    DirectoryExists(PathBuf),

    #[error("Template generation failed: {0}")]
    TemplateError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid name: {0}")]
    InvalidName(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
