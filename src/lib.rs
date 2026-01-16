//! Echidna - ChimeraX Bundle Development CLI
//!
//! A tool to streamline the development of ChimeraX bundles (extensions).

pub mod chimerax;
pub mod commands;
pub mod config;
pub mod error;
pub mod templates;

pub use config::Config;
pub use error::{EchidnaError, Result};
