//! Package management for ChimeraX Python environment.
//!
//! This module provides functionality to list packages, check for conflicts,
//! and install new packages to the ChimeraX Python environment.

mod installer;
mod resolver;

pub use installer::{InstallResult, PackageInstaller};
pub use resolver::PackageResolver;

use serde::{Deserialize, Serialize};

/// Package information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

/// Conflict check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictCheckResult {
    /// The package specification that was checked.
    pub package: String,
    /// Whether the package can be installed without breaking conflicts.
    pub installable: bool,
    /// List of conflicts that prevent installation.
    pub conflicts: Vec<ConflictInfo>,
    /// Packages that would be installed.
    pub would_install: Vec<PackageInfo>,
    /// Packages that would be upgraded or downgraded.
    pub would_change: Vec<PackageChange>,
}

/// Conflict detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// The package causing or affected by the conflict.
    pub package: String,
    /// Human-readable description of the conflict.
    pub reason: String,
}

/// Package version change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageChange {
    pub name: String,
    pub from_version: String,
    pub to_version: String,
}

/// Backend used for package operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageBackend {
    /// Use uv (faster, preferred)
    Uv,
    /// Use pip (fallback)
    Pip,
}

impl std::fmt::Display for PackageBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageBackend::Uv => write!(f, "uv"),
            PackageBackend::Pip => write!(f, "pip"),
        }
    }
}

use crate::error::{EchidnaError, Result};

/// Validate a package specification to prevent command injection.
///
/// Rejects package names containing shell metacharacters that could
/// be interpreted specially by the shell or pip/uv.
///
/// Note: `<` and `>` are allowed as they're valid in version specifiers (>=, <=, etc.)
pub(crate) fn validate_package_spec(package: &str) -> Result<()> {
    const DANGEROUS_CHARS: &[char] = &[';', '|', '&', '`', '$', '\n', '\r', '\\', '"', '\''];

    for ch in DANGEROUS_CHARS {
        if package.contains(*ch) {
            return Err(EchidnaError::PackageError(format!(
                "Invalid character '{}' in package specification: {}",
                ch, package
            )));
        }
    }

    if package.trim().is_empty() {
        return Err(EchidnaError::PackageError(
            "Empty package specification".to_string(),
        ));
    }

    Ok(())
}

/// Filter out stdlib and common built-in packages from a package list.
pub fn filter_stdlib(packages: Vec<PackageInfo>) -> Vec<PackageInfo> {
    packages
        .into_iter()
        .filter(|p| !is_stdlib_package(&p.name))
        .collect()
}

/// Check if a package name is a stdlib or built-in package.
fn is_stdlib_package(name: &str) -> bool {
    // Normalize package name (lowercase, replace - with _)
    let normalized = name.to_lowercase().replace('-', "_");

    // Common built-in/bundled packages that come with Python or are
    // typically part of the base installation
    const BUILTIN_PACKAGES: &[&str] = &[
        "pip",
        "setuptools",
        "wheel",
        "pkg_resources",
        "_distutils_hack",
    ];

    BUILTIN_PACKAGES.contains(&normalized.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_stdlib() {
        let packages = vec![
            PackageInfo {
                name: "pip".to_string(),
                version: "24.0".to_string(),
            },
            PackageInfo {
                name: "numpy".to_string(),
                version: "1.26.0".to_string(),
            },
            PackageInfo {
                name: "setuptools".to_string(),
                version: "69.0".to_string(),
            },
            PackageInfo {
                name: "requests".to_string(),
                version: "2.31.0".to_string(),
            },
        ];

        let filtered = filter_stdlib(packages);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|p| p.name == "numpy"));
        assert!(filtered.iter().any(|p| p.name == "requests"));
    }

    #[test]
    fn test_is_stdlib_package() {
        assert!(is_stdlib_package("pip"));
        assert!(is_stdlib_package("setuptools"));
        assert!(is_stdlib_package("wheel"));
        assert!(!is_stdlib_package("numpy"));
        assert!(!is_stdlib_package("requests"));
    }

    #[test]
    fn test_package_backend_display() {
        assert_eq!(format!("{}", PackageBackend::Uv), "uv");
        assert_eq!(format!("{}", PackageBackend::Pip), "pip");
    }

    #[test]
    fn test_validate_package_spec_valid() {
        assert!(validate_package_spec("numpy").is_ok());
        assert!(validate_package_spec("numpy>=1.20").is_ok());
        assert!(validate_package_spec("requests[security]>=2.28").is_ok());
        assert!(validate_package_spec("my-package==1.0.0").is_ok());
    }

    #[test]
    fn test_validate_package_spec_rejects_dangerous_chars() {
        assert!(validate_package_spec("numpy; rm -rf /").is_err());
        assert!(validate_package_spec("numpy | cat /etc/passwd").is_err());
        assert!(validate_package_spec("numpy & whoami").is_err());
        assert!(validate_package_spec("numpy`whoami`").is_err());
        assert!(validate_package_spec("numpy$HOME").is_err());
        assert!(validate_package_spec("numpy\nwhoami").is_err());
    }

    #[test]
    fn test_validate_package_spec_rejects_empty() {
        assert!(validate_package_spec("").is_err());
        assert!(validate_package_spec("   ").is_err());
    }
}
