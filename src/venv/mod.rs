//! Virtual environment creation for IDE support.
//!
//! This module creates a minimal venv that references ChimeraX's Python,
//! allowing IDEs and type checkers to recognize chimerax modules.

mod builder;
mod configs;

pub use builder::VenvBuilder;
pub use configs::{ConfigGenerator, ConfigType};
