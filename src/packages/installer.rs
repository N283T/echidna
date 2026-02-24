//! Package installer with uv/pip backends.

use crate::chimerax::Verbosity;
use crate::error::{EchidnaError, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::{validate_package_spec, PackageBackend};

/// Result of an install operation.
#[derive(Debug)]
pub struct InstallResult {
    /// Packages that were installed.
    pub installed: Vec<String>,
    /// Whether the installation succeeded.
    pub success: bool,
    /// Output from the install command (for verbose mode).
    pub output: String,
}

/// Package installer that uses uv (if available) or pip as fallback.
pub struct PackageInstaller {
    python_executable: PathBuf,
    backend: PackageBackend,
    verbosity: Verbosity,
}

impl PackageInstaller {
    /// Create a new package installer.
    ///
    /// Automatically detects whether uv is available on PATH.
    pub fn new(python_executable: PathBuf, verbosity: Verbosity) -> Self {
        let backend = if which::which("uv").is_ok() {
            PackageBackend::Uv
        } else {
            PackageBackend::Pip
        };
        Self {
            python_executable,
            backend,
            verbosity,
        }
    }

    /// Get the backend being used.
    pub fn backend(&self) -> PackageBackend {
        self.backend
    }

    /// Install packages.
    ///
    /// # Arguments
    /// * `packages` - Package specifications to install (e.g., "numpy", "requests>=2.28")
    /// * `upgrade` - Whether to upgrade packages if already installed
    /// * `dry_run` - Only show what would be installed without actually installing
    pub fn install(
        &self,
        packages: &[String],
        upgrade: bool,
        dry_run: bool,
    ) -> Result<InstallResult> {
        // Validate all package specifications
        for pkg in packages {
            validate_package_spec(pkg)?;
        }

        match self.backend {
            PackageBackend::Uv => self.install_uv(packages, upgrade, dry_run),
            PackageBackend::Pip => self.install_pip(packages, upgrade, dry_run),
        }
    }

    // === uv implementation ===

    fn install_uv(
        &self,
        packages: &[String],
        upgrade: bool,
        dry_run: bool,
    ) -> Result<InstallResult> {
        let mut args = vec![
            "pip".to_string(),
            "install".to_string(),
            "--python".to_string(),
            self.python_executable.to_string_lossy().to_string(),
        ];

        if upgrade {
            args.push("--upgrade".to_string());
        }

        if dry_run {
            args.push("--dry-run".to_string());
        }

        // Add packages
        args.extend(packages.iter().cloned());

        self.log_command(&format!("uv {}", args.join(" ")));

        let output = Command::new("uv")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}\n{}", stdout, stderr);

        self.log_output(&combined);

        if !output.status.success() {
            return Err(EchidnaError::PackageError(format!(
                "uv pip install failed:\n{}",
                stderr.trim()
            )));
        }

        Ok(InstallResult {
            installed: packages.to_vec(),
            success: true,
            output: combined,
        })
    }

    // === pip implementation ===

    fn install_pip(
        &self,
        packages: &[String],
        upgrade: bool,
        dry_run: bool,
    ) -> Result<InstallResult> {
        let mut args = vec!["-m".to_string(), "pip".to_string(), "install".to_string()];

        if upgrade {
            args.push("--upgrade".to_string());
        }

        if dry_run {
            args.push("--dry-run".to_string());
        }

        // Add packages
        args.extend(packages.iter().cloned());

        self.log_command(&format!(
            "{} {}",
            self.python_executable.display(),
            args.join(" ")
        ));

        let output = Command::new(&self.python_executable)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}\n{}", stdout, stderr);

        self.log_output(&combined);

        if !output.status.success() {
            return Err(EchidnaError::PackageError(format!(
                "pip install failed:\n{}",
                stderr.trim()
            )));
        }

        Ok(InstallResult {
            installed: packages.to_vec(),
            success: true,
            output: combined,
        })
    }

    fn log_command(&self, cmd: &str) {
        if self.verbosity >= 1 {
            eprintln!("[echi] Executing: {}", cmd);
        }
    }

    fn log_output(&self, output: &str) {
        if self.verbosity >= 2 {
            eprintln!("[echi] Output:\n{}", output);
        }
    }
}
