//! Virtual environment creation for IDE support.
//!
//! This module creates a minimal venv that references ChimeraX's Python,
//! allowing IDEs and type checkers to recognize chimerax modules.
//!
//! Uses uv if available for faster venv creation, with fallback to manual symlinks.

mod builder;
mod configs;

pub use builder::{VenvBackend, VenvBuilder};
pub use configs::{ConfigGenerator, ConfigType};
