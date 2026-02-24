//! Configuration file handling for echidna.

use crate::error::{EchidnaError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The configuration file name.
pub const CONFIG_FILE_NAME: &str = "echidna.toml";

/// Configuration from echidna.toml.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Config {
    /// Bundle name (e.g., "ChimeraX-Example")
    pub bundle_name: Option<String>,

    /// Python package name (e.g., "chimerax.example")
    pub package_name: Option<String>,

    /// Path to ChimeraX executable
    pub chimerax_path: Option<PathBuf>,

    /// Default script to run on `echidna run`
    pub default_script: Option<PathBuf>,

    /// Install as user bundle by default
    #[serde(default)]
    pub user_install: bool,
}

impl Config {
    /// Load configuration from echidna.toml in the given directory or its parents.
    ///
    /// Returns `Ok(None)` if no configuration file is found.
    pub fn load(start_dir: &Path) -> Result<Option<Self>> {
        let mut current = start_dir
            .canonicalize()
            .unwrap_or_else(|_| start_dir.to_path_buf());

        loop {
            let config_path = current.join("echidna.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                let config: Self = toml::from_str(&content)?;
                return Ok(Some(config));
            }

            if !current.pop() {
                break;
            }
        }

        Ok(None)
    }

    /// Load configuration from the current directory.
    pub fn load_from_cwd() -> Result<Option<Self>> {
        let cwd = std::env::current_dir()?;
        Self::load(&cwd)
    }

    /// Parse configuration from a TOML string.
    pub fn from_toml(content: &str) -> Result<Self> {
        let config: Self = toml::from_str(content)?;
        Ok(config)
    }
}

/// Global configuration stored in ~/.config/echidna/config.toml (or platform equivalent).
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct GlobalConfig {
    /// Path to ChimeraX executable.
    pub chimerax_path: Option<PathBuf>,

    /// Cached ChimeraX version.
    pub chimerax_version: Option<String>,
}

impl GlobalConfig {
    /// Get the path to the global configuration file.
    ///
    /// Uses XDG base directories on Linux, appropriate locations on macOS/Windows.
    pub fn path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "echidna").map(|dirs| dirs.config_dir().join("config.toml"))
    }

    /// Load the global configuration from the default location.
    ///
    /// Returns `Ok(None)` if the file does not exist.
    pub fn load() -> Result<Option<Self>> {
        let Some(path) = Self::path() else {
            return Ok(None);
        };

        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(Some(config))
    }

    /// Save the global configuration to the default location.
    ///
    /// Creates the parent directory if it does not exist.
    pub fn save(&self) -> Result<()> {
        let path = Self::path().ok_or_else(|| {
            EchidnaError::ConfigError("Could not determine global config path".into())
        })?;

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| EchidnaError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Update the ChimeraX path and version, then save.
    pub fn update_chimerax(&mut self, path: PathBuf, version: Option<String>) -> Result<()> {
        self.chimerax_path = Some(path);
        self.chimerax_version = version;
        self.save()
    }

    /// Update only the cached version, then save.
    pub fn update_version(&mut self, version: String) -> Result<()> {
        self.chimerax_version = Some(version);
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_complete_config() {
        let toml = r#"
bundle_name = "ChimeraX-Example"
package_name = "chimerax.example"
chimerax_path = "/Applications/ChimeraX.app/Contents/bin/ChimeraX"
default_script = "scripts/test.cxc"
user_install = true
"#;
        let config = Config::from_toml(toml).unwrap();
        assert_eq!(config.bundle_name, Some("ChimeraX-Example".to_string()));
        assert_eq!(config.package_name, Some("chimerax.example".to_string()));
        assert_eq!(
            config.chimerax_path,
            Some(PathBuf::from(
                "/Applications/ChimeraX.app/Contents/bin/ChimeraX"
            ))
        );
        assert_eq!(
            config.default_script,
            Some(PathBuf::from("scripts/test.cxc"))
        );
        assert!(config.user_install);
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml = "";
        let config = Config::from_toml(toml).unwrap();
        assert_eq!(config.bundle_name, None);
        assert_eq!(config.package_name, None);
        assert_eq!(config.chimerax_path, None);
        assert_eq!(config.default_script, None);
        assert!(!config.user_install);
    }

    #[test]
    fn test_parse_partial_config() {
        let toml = r#"
bundle_name = "ChimeraX-MyTool"
"#;
        let config = Config::from_toml(toml).unwrap();
        assert_eq!(config.bundle_name, Some("ChimeraX-MyTool".to_string()));
        assert_eq!(config.package_name, None);
        assert!(!config.user_install);
    }

    #[test]
    fn test_load_from_directory() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);

        fs::write(
            &config_path,
            r#"
bundle_name = "ChimeraX-Test"
user_install = true
"#,
        )
        .unwrap();

        let config = Config::load(temp.path()).unwrap().unwrap();
        assert_eq!(config.bundle_name, Some("ChimeraX-Test".to_string()));
        assert!(config.user_install);
    }

    #[test]
    fn test_load_searches_parent_directories() {
        let temp = TempDir::new().unwrap();

        // Create config in root
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(&config_path, r#"bundle_name = "ChimeraX-Parent""#).unwrap();

        // Create a nested directory
        let nested = temp.path().join("src").join("commands");
        fs::create_dir_all(&nested).unwrap();

        // Load from nested directory should find parent config
        let config = Config::load(&nested).unwrap().unwrap();
        assert_eq!(config.bundle_name, Some("ChimeraX-Parent".to_string()));
    }

    #[test]
    fn test_load_returns_none_when_not_found() {
        let temp = TempDir::new().unwrap();

        // No config file created
        let result = Config::load(temp.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.bundle_name, None);
        assert_eq!(config.package_name, None);
        assert_eq!(config.chimerax_path, None);
        assert_eq!(config.default_script, None);
        assert!(!config.user_install);
    }

    #[test]
    fn test_parse_invalid_toml() {
        let toml = "this is not valid toml [[[";
        let result = Config::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_global_config_path() {
        // Global config path should be available on supported platforms
        let path = GlobalConfig::path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.ends_with("config.toml"));
    }

    #[test]
    fn test_global_config_default() {
        let config = GlobalConfig::default();
        assert_eq!(config.chimerax_path, None);
        assert_eq!(config.chimerax_version, None);
    }

    #[test]
    fn test_global_config_serialization() {
        let config = GlobalConfig {
            chimerax_path: Some(PathBuf::from("/usr/bin/chimerax")),
            chimerax_version: Some("1.9".to_string()),
        };

        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("chimerax_path"));
        assert!(toml_str.contains("chimerax_version"));
        assert!(toml_str.contains("1.9"));
    }
}
