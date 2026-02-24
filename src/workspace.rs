//! Workspace support for managing multiple bundles.

use crate::error::{EchidnaError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Workspace configuration filename.
pub const WORKSPACE_FILE: &str = "workspace.toml";

/// Workspace configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Workspace settings.
    pub workspace: WorkspaceSettings,
}

/// Workspace settings section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    /// List of member bundle paths (relative to workspace root).
    pub members: Vec<String>,
}

impl Workspace {
    /// Create a new workspace with the given members.
    pub fn new(members: Vec<String>) -> Self {
        Self {
            workspace: WorkspaceSettings { members },
        }
    }

    /// Load workspace configuration from a file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| {
            EchidnaError::ConfigError(format!(
                "Failed to read workspace file '{}': {}",
                path.display(),
                e
            ))
        })?;

        toml::from_str(&content).map_err(|e| {
            EchidnaError::ConfigError(format!(
                "Invalid workspace.toml '{}': {}",
                path.display(),
                e
            ))
        })
    }

    /// Find workspace root by searching upward from the given path.
    pub fn find_root(start: &Path) -> Option<PathBuf> {
        let mut current = start.to_path_buf();
        loop {
            let workspace_file = current.join(WORKSPACE_FILE);
            if workspace_file.exists() {
                return Some(current);
            }
            if !current.pop() {
                return None;
            }
        }
    }

    /// Load workspace from the current or parent directories.
    pub fn load_from_path(path: &Path) -> Result<Option<(PathBuf, Self)>> {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if let Some(root) = Self::find_root(&path) {
            let workspace_file = root.join(WORKSPACE_FILE);
            let workspace = Self::load(&workspace_file)?;
            Ok(Some((root, workspace)))
        } else {
            Ok(None)
        }
    }

    /// Save workspace configuration to a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            EchidnaError::ConfigError(format!("Failed to serialize workspace: {}", e))
        })?;

        fs::write(path, content)?;
        Ok(())
    }

    /// Get all member paths resolved relative to the workspace root.
    pub fn member_paths(&self, workspace_root: &Path) -> Vec<PathBuf> {
        self.workspace
            .members
            .iter()
            .map(|m| workspace_root.join(m))
            .collect()
    }

    /// Discover bundles in a directory (directories containing pyproject.toml).
    pub fn discover_members(root: &Path) -> Result<Vec<String>> {
        let mut members = Vec::new();

        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let pyproject = path.join("pyproject.toml");
                if pyproject.exists() {
                    if let Some(name) = path.file_name() {
                        members.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }

        members.sort();
        Ok(members)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workspace_new() {
        let ws = Workspace::new(vec!["bundle-a".to_string(), "bundle-b".to_string()]);
        assert_eq!(ws.workspace.members.len(), 2);
        assert_eq!(ws.workspace.members[0], "bundle-a");
    }

    #[test]
    fn test_workspace_save_and_load() {
        let temp = TempDir::new().unwrap();
        let ws_file = temp.path().join(WORKSPACE_FILE);

        let ws = Workspace::new(vec!["bundles/a".to_string(), "bundles/b".to_string()]);
        ws.save(&ws_file).unwrap();

        let loaded = Workspace::load(&ws_file).unwrap();
        assert_eq!(loaded.workspace.members.len(), 2);
        assert_eq!(loaded.workspace.members[0], "bundles/a");
    }

    #[test]
    fn test_find_root() {
        let temp = TempDir::new().unwrap();
        let ws_file = temp.path().join(WORKSPACE_FILE);

        // Create workspace file
        let ws = Workspace::new(vec!["bundle".to_string()]);
        ws.save(&ws_file).unwrap();

        // Create a subdirectory
        let subdir = temp.path().join("bundle").join("src");
        fs::create_dir_all(&subdir).unwrap();

        // Find root from subdirectory
        let root = Workspace::find_root(&subdir);
        assert!(root.is_some());
        assert_eq!(root.unwrap(), temp.path());
    }

    #[test]
    fn test_find_root_not_found() {
        let temp = TempDir::new().unwrap();
        let root = Workspace::find_root(temp.path());
        assert!(root.is_none());
    }

    #[test]
    fn test_member_paths() {
        let ws = Workspace::new(vec!["bundles/a".to_string(), "bundles/b".to_string()]);
        let root = PathBuf::from("/workspace");
        let paths = ws.member_paths(&root);

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], PathBuf::from("/workspace/bundles/a"));
        assert_eq!(paths[1], PathBuf::from("/workspace/bundles/b"));
    }

    #[test]
    fn test_discover_members() {
        let temp = TempDir::new().unwrap();

        // Create two bundle directories
        let bundle_a = temp.path().join("bundle-a");
        let bundle_b = temp.path().join("bundle-b");
        let not_bundle = temp.path().join("not-a-bundle");

        fs::create_dir_all(&bundle_a).unwrap();
        fs::create_dir_all(&bundle_b).unwrap();
        fs::create_dir_all(&not_bundle).unwrap();

        // Only bundle directories have pyproject.toml
        fs::write(bundle_a.join("pyproject.toml"), "[project]\nname = \"a\"").unwrap();
        fs::write(bundle_b.join("pyproject.toml"), "[project]\nname = \"b\"").unwrap();

        let members = Workspace::discover_members(temp.path()).unwrap();
        assert_eq!(members.len(), 2);
        assert!(members.contains(&"bundle-a".to_string()));
        assert!(members.contains(&"bundle-b".to_string()));
    }
}
