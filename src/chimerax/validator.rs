//! ChimeraX path validation and interactive configuration.

use super::detect::get_symlink_target;
use crate::chimerax::{find_chimerax, get_chimerax_version};
use crate::config::{Config, GlobalConfig};
use crate::error::Result;
use colored::Colorize;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

/// Check if we're running in an interactive terminal.
fn is_interactive() -> bool {
    std::io::stdin().is_terminal() && std::io::stderr().is_terminal()
}

/// Result of validating a configured ChimeraX path.
#[derive(Debug)]
pub enum ValidationResult {
    /// The configured path is valid and version matches.
    Valid {
        path: PathBuf,
        version: Option<String>,
    },

    /// The configured path does not exist.
    PathNotFound {
        configured_path: PathBuf,
        detected_path: Option<PathBuf>,
        detected_version: Option<String>,
    },

    /// The ChimeraX version has changed.
    VersionChanged {
        path: PathBuf,
        old_version: String,
        new_version: String,
    },

    /// No ChimeraX is configured (use auto-detection).
    NotConfigured {
        detected_path: Option<PathBuf>,
        detected_version: Option<String>,
    },
}

/// Where to save the ChimeraX configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SaveTarget {
    /// Save to global config (~/.config/echidna/config.toml)
    Global,
    /// Use for this session only (don't save)
    SessionOnly,
}

/// Validates ChimeraX configuration and prompts user for updates if needed.
pub struct ChimeraXValidator;

impl ChimeraXValidator {
    /// Validate the ChimeraX configuration and return the validation result.
    ///
    /// Checks in order: project config, global config, then auto-detection.
    pub fn validate(
        global_config: &GlobalConfig,
        project_config: &Option<Config>,
    ) -> ValidationResult {
        // Check project config first
        if let Some(project) = project_config
            && let Some(ref path) = project.chimerax_path
        {
            if path.exists() {
                let version = get_chimerax_version(path).ok();
                return ValidationResult::Valid {
                    path: path.clone(),
                    version,
                };
            }
            // Path doesn't exist - try to detect
            let detected = find_chimerax();
            let detected_version = detected.as_ref().and_then(|p| get_chimerax_version(p).ok());
            return ValidationResult::PathNotFound {
                configured_path: path.clone(),
                detected_path: detected,
                detected_version,
            };
        }

        // Check global config
        if let Some(ref path) = global_config.chimerax_path {
            if path.exists() {
                let current_version = get_chimerax_version(path).ok();

                // Check if version changed
                if let (Some(cached), Some(current)) =
                    (&global_config.chimerax_version, &current_version)
                    && cached != current
                {
                    return ValidationResult::VersionChanged {
                        path: path.clone(),
                        old_version: cached.clone(),
                        new_version: current.clone(),
                    };
                }

                return ValidationResult::Valid {
                    path: path.clone(),
                    version: current_version,
                };
            }
            // Path doesn't exist - try to detect
            let detected = find_chimerax();
            let detected_version = detected.as_ref().and_then(|p| get_chimerax_version(p).ok());
            return ValidationResult::PathNotFound {
                configured_path: path.clone(),
                detected_path: detected,
                detected_version,
            };
        }

        // No configuration - try auto-detection
        let detected = find_chimerax();
        let detected_version = detected.as_ref().and_then(|p| get_chimerax_version(p).ok());
        ValidationResult::NotConfigured {
            detected_path: detected,
            detected_version,
        }
    }

    /// Validate configuration and prompt user for updates if needed.
    ///
    /// Returns the validated ChimeraX path, or None if ChimeraX was not found.
    pub fn validate_and_prompt(
        global_config: &mut GlobalConfig,
        project_config: &Option<Config>,
    ) -> Result<Option<PathBuf>> {
        // Check if auto-detection will be needed (no explicit config)
        let needs_detection = project_config
            .as_ref()
            .and_then(|c| c.chimerax_path.as_ref())
            .is_none()
            && global_config.chimerax_path.is_none();

        if needs_detection {
            eprintln!("{}", "Searching for ChimeraX...".dimmed());
        }

        let result = Self::validate(global_config, project_config);

        match result {
            ValidationResult::Valid { path, version } => {
                // Update cached version if we got one but didn't have it stored
                if let Some(v) = version
                    && global_config.chimerax_version.is_none()
                    && let Err(e) = global_config.update_version(v)
                {
                    eprintln!(
                        "{}",
                        format!("Warning: Could not save version cache: {}", e).yellow()
                    );
                }
                Ok(Some(path))
            }

            ValidationResult::PathNotFound {
                configured_path,
                detected_path,
                detected_version,
            } => Self::handle_path_not_found(
                global_config,
                &configured_path,
                detected_path,
                detected_version,
            ),

            ValidationResult::VersionChanged {
                path,
                old_version,
                new_version,
            } => Self::handle_version_changed(global_config, path, &old_version, &new_version),

            ValidationResult::NotConfigured {
                detected_path,
                detected_version,
            } => Self::handle_not_configured(global_config, detected_path, detected_version),
        }
    }

    fn handle_path_not_found(
        global_config: &mut GlobalConfig,
        configured_path: &Path,
        detected_path: Option<PathBuf>,
        detected_version: Option<String>,
    ) -> Result<Option<PathBuf>> {
        eprintln!(
            "\n{}",
            "\u{26a0} Warning: Configured ChimeraX path not found:".yellow()
        );
        eprintln!("  {}", configured_path.display());

        if let Some(ref path) = detected_path {
            let version_str = detected_version
                .as_ref()
                .map(|v| format!(" (v{})", v))
                .unwrap_or_default();
            eprintln!("\n{}", "? Auto-detected ChimeraX at:".green());
            eprintln!("  {}{}", path.display(), version_str);
            // Show symlink target if path is a symlink
            if let Some(target) = get_symlink_target(path) {
                eprintln!("  {}", format!("-> {}", target.display()).dimmed());
            }

            // In non-interactive mode, use detected path without saving
            if !is_interactive() {
                eprintln!(
                    "\n{}",
                    "Using detected path (non-interactive mode).".dimmed()
                );
                return Ok(Some(path.clone()));
            }

            let save_target = Self::prompt_save_target()?;

            match save_target {
                SaveTarget::Global => {
                    global_config.update_chimerax(path.clone(), detected_version)?;
                    eprintln!("\n{}", "\u{2713} Updated global configuration.".green());
                    if let Some(config_path) = GlobalConfig::path() {
                        eprintln!("  {}", config_path.display());
                    }
                    Ok(Some(path.clone()))
                }
                SaveTarget::SessionOnly => {
                    eprintln!(
                        "\n{}",
                        "Using detected path for this session only.".dimmed()
                    );
                    Ok(Some(path.clone()))
                }
            }
        } else {
            eprintln!("\n{}", "\u{2717} Could not auto-detect ChimeraX.".red());
            eprintln!("  Please install ChimeraX or specify the path with --chimerax");
            Ok(None)
        }
    }

    fn handle_version_changed(
        global_config: &mut GlobalConfig,
        path: PathBuf,
        old_version: &str,
        new_version: &str,
    ) -> Result<Option<PathBuf>> {
        eprintln!(
            "\n{}",
            format!(
                "\u{2139} ChimeraX version changed: {} \u{2192} {}",
                old_version, new_version
            )
            .cyan()
        );

        // In non-interactive mode, auto-update the cached version
        if !is_interactive() {
            if let Err(e) = global_config.update_version(new_version.to_string()) {
                eprintln!(
                    "{}",
                    format!("Warning: Could not update version cache: {}", e).yellow()
                );
            } else {
                eprintln!("\n{}", "\u{2713} Updated cached version.".green());
            }
            return Ok(Some(path));
        }

        let update = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Update cached version?")
            .default(true)
            .interact()?;

        if update {
            global_config.update_version(new_version.to_string())?;
            eprintln!("\n{}", "\u{2713} Updated cached version.".green());
        }

        Ok(Some(path))
    }

    fn handle_not_configured(
        global_config: &mut GlobalConfig,
        detected_path: Option<PathBuf>,
        detected_version: Option<String>,
    ) -> Result<Option<PathBuf>> {
        if let Some(ref path) = detected_path {
            let version_str = detected_version
                .as_ref()
                .map(|v| format!(" (v{})", v))
                .unwrap_or_default();

            eprintln!("\n{}", "\u{2713} Auto-detected ChimeraX:".green());
            eprintln!("  {}{}", path.display(), version_str);
            // Show symlink target if path is a symlink
            if let Some(target) = get_symlink_target(path) {
                eprintln!("  {}", format!("-> {}", target.display()).dimmed());
            }

            // In non-interactive mode, use detected path without prompting
            if !is_interactive() {
                return Ok(Some(path.clone()));
            }

            let save = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Save to global configuration?")
                .default(true)
                .interact()?;

            if save {
                global_config.update_chimerax(path.clone(), detected_version)?;
                eprintln!("\n{}", "\u{2713} Saved to global configuration.".green());
                if let Some(config_path) = GlobalConfig::path() {
                    eprintln!("  {}", config_path.display());
                }
            }

            Ok(Some(path.clone()))
        } else {
            eprintln!("\n{}", "\u{2717} Could not auto-detect ChimeraX.".red());
            eprintln!("  Please install ChimeraX or specify the path with --chimerax");
            Ok(None)
        }
    }

    fn prompt_save_target() -> Result<SaveTarget> {
        let options = &[
            "Yes, save to global config",
            "No, use for this session only",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Update configuration?")
            .items(options)
            .default(0)
            .interact()?;

        Ok(match selection {
            0 => SaveTarget::Global,
            _ => SaveTarget::SessionOnly,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validation_result_variants() {
        let valid = ValidationResult::Valid {
            path: PathBuf::from("/test"),
            version: Some("1.9".to_string()),
        };
        assert!(matches!(valid, ValidationResult::Valid { .. }));

        let not_found = ValidationResult::PathNotFound {
            configured_path: PathBuf::from("/old"),
            detected_path: Some(PathBuf::from("/new")),
            detected_version: Some("1.9".to_string()),
        };
        assert!(matches!(not_found, ValidationResult::PathNotFound { .. }));

        let version_changed = ValidationResult::VersionChanged {
            path: PathBuf::from("/test"),
            old_version: "1.8".to_string(),
            new_version: "1.9".to_string(),
        };
        assert!(matches!(
            version_changed,
            ValidationResult::VersionChanged { .. }
        ));

        let not_configured = ValidationResult::NotConfigured {
            detected_path: None,
            detected_version: None,
        };
        assert!(matches!(
            not_configured,
            ValidationResult::NotConfigured { .. }
        ));
    }

    #[test]
    fn test_save_target_values() {
        assert_eq!(SaveTarget::Global, SaveTarget::Global);
        assert_eq!(SaveTarget::SessionOnly, SaveTarget::SessionOnly);
        assert_ne!(SaveTarget::Global, SaveTarget::SessionOnly);
    }

    #[test]
    fn test_validate_with_empty_configs() {
        let global = GlobalConfig::default();
        let project: Option<Config> = None;

        let result = ChimeraXValidator::validate(&global, &project);
        // Without ChimeraX installed, this should be NotConfigured
        assert!(matches!(
            result,
            ValidationResult::NotConfigured { .. } | ValidationResult::Valid { .. }
        ));
    }

    #[test]
    fn test_validate_with_nonexistent_global_path() {
        let global = GlobalConfig {
            chimerax_path: Some(PathBuf::from("/nonexistent/path/to/chimerax")),
            chimerax_version: Some("1.8".to_string()),
        };
        let project: Option<Config> = None;

        let result = ChimeraXValidator::validate(&global, &project);
        assert!(matches!(result, ValidationResult::PathNotFound { .. }));
    }

    #[test]
    fn test_validate_project_config_takes_precedence() {
        let global = GlobalConfig {
            chimerax_path: Some(PathBuf::from("/global/chimerax")),
            chimerax_version: Some("1.8".to_string()),
        };
        let project = Some(Config {
            chimerax_path: Some(PathBuf::from("/project/chimerax")),
            ..Default::default()
        });

        let result = ChimeraXValidator::validate(&global, &project);
        // Project config should be checked first
        match result {
            ValidationResult::PathNotFound {
                configured_path, ..
            } => {
                assert_eq!(configured_path, PathBuf::from("/project/chimerax"));
            }
            _ => {
                // If the path exists (unlikely), that's also fine
            }
        }
    }
}
