//! Venv builder for IDE support.

use crate::chimerax::PythonInfo;
use crate::error::{EchidnaError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Builder for creating IDE-compatible venv directories.
pub struct VenvBuilder {
    output_dir: PathBuf,
    python_info: PythonInfo,
    force: bool,
}

impl VenvBuilder {
    /// Create a new venv builder.
    pub fn new(output_dir: PathBuf, python_info: PythonInfo) -> Self {
        Self {
            output_dir,
            python_info,
            force: false,
        }
    }

    /// Set whether to force overwrite existing venv.
    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Build the venv directory structure.
    pub fn build(&self) -> Result<()> {
        // Check if venv already exists
        if self.output_dir.exists() {
            if self.force {
                fs::remove_dir_all(&self.output_dir)?;
            } else {
                return Err(EchidnaError::VenvExists(self.output_dir.clone()));
            }
        }

        // Create venv directory
        fs::create_dir_all(&self.output_dir)?;

        // Create pyvenv.cfg
        self.create_pyvenv_cfg()?;

        // Create bin/ or Scripts/ directory with symlinks
        #[cfg(unix)]
        self.create_unix_links()?;

        #[cfg(windows)]
        self.create_windows_links()?;

        Ok(())
    }

    /// Create the pyvenv.cfg file.
    fn create_pyvenv_cfg(&self) -> Result<()> {
        // Extract Python version (e.g., "3.11.6" from "3.11.6 (main, ...)")
        let version = self
            .python_info
            .version
            .split_whitespace()
            .next()
            .unwrap_or("3.11.0");

        // The "home" directory is the directory containing the Python executable
        let python_path = Path::new(&self.python_info.executable);
        let home = python_path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| self.python_info.prefix.clone());

        let cfg_content = format!(
            "home = {home}\n\
             include-system-site-packages = true\n\
             version = {version}\n\
             executable = {executable}\n",
            home = home,
            version = version,
            executable = self.python_info.executable,
        );

        let cfg_path = self.output_dir.join("pyvenv.cfg");
        fs::write(&cfg_path, cfg_content)?;

        Ok(())
    }

    /// Create bin/ directory with symlinks (Unix).
    #[cfg(unix)]
    fn create_unix_links(&self) -> Result<()> {
        use std::os::unix::fs::symlink;

        let bin_dir = self.output_dir.join("bin");
        fs::create_dir_all(&bin_dir)?;

        let python_path = Path::new(&self.python_info.executable);

        // Create python symlink
        let python_link = bin_dir.join("python");
        symlink(python_path, &python_link)?;

        // Create python3 symlink pointing to python
        let python3_link = bin_dir.join("python3");
        symlink(&python_link, &python3_link)?;

        Ok(())
    }

    /// Create Scripts/ directory with links (Windows).
    /// Tries symlink first (works with admin or Developer Mode), then hard link,
    /// then falls back to copying (for cross-volume scenarios).
    #[cfg(windows)]
    fn create_windows_links(&self) -> Result<()> {
        use std::os::windows::fs::symlink_file;

        let scripts_dir = self.output_dir.join("Scripts");
        fs::create_dir_all(&scripts_dir)?;

        let python_path = Path::new(&self.python_info.executable);
        let python_link = scripts_dir.join("python.exe");

        // Try symlink first (works with admin privileges or Developer Mode)
        if symlink_file(python_path, &python_link).is_ok() {
            return Ok(());
        }

        // Fall back to hard link (only works on same volume)
        if fs::hard_link(python_path, &python_link).is_ok() {
            return Ok(());
        }

        // Final fallback: copy the file (works cross-volume)
        fs::copy(python_path, &python_link)?;
        Ok(())
    }

    /// Get the output directory.
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn mock_python_info() -> PythonInfo {
        PythonInfo {
            executable: "/usr/bin/python3".to_string(),
            version: "3.11.6 (main, Oct  2 2023, 00:00:00)".to_string(),
            prefix: "/usr".to_string(),
            path: vec![],
            chimerax_version: Some("1.7".to_string()),
            site_packages: vec![],
        }
    }

    #[test]
    fn test_venv_builder_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let venv_path = temp_dir.path().join(".venv");

        let builder = VenvBuilder::new(venv_path.clone(), mock_python_info());

        // Don't actually build since we don't have a real Python, just test the struct
        assert_eq!(builder.output_dir(), venv_path);
    }

    #[test]
    fn test_venv_builder_force_flag() {
        let temp_dir = TempDir::new().unwrap();
        let venv_path = temp_dir.path().join(".venv");

        let builder = VenvBuilder::new(venv_path.clone(), mock_python_info()).force(true);

        // The force flag should be set
        assert!(builder.force);
    }
}
