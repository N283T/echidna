//! Package resolver with uv/pip backends.

use crate::chimerax::Verbosity;
use crate::error::{EchidnaError, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::{
    validate_package_spec, ConflictCheckResult, ConflictInfo, PackageBackend, PackageChange,
    PackageInfo,
};

/// Package resolver that uses uv (if available) or pip as fallback.
pub struct PackageResolver {
    python_executable: PathBuf,
    backend: PackageBackend,
    verbosity: Verbosity,
}

impl PackageResolver {
    /// Create a new package resolver.
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

    /// List all installed packages.
    pub fn list_packages(&self) -> Result<Vec<PackageInfo>> {
        match self.backend {
            PackageBackend::Uv => self.list_uv(),
            PackageBackend::Pip => self.list_pip(),
        }
    }

    /// Check if a package can be installed without conflicts.
    pub fn check_package(&self, package: &str) -> Result<ConflictCheckResult> {
        // Validate package specification to prevent command injection
        validate_package_spec(package)?;

        match self.backend {
            PackageBackend::Uv => self.check_uv(package),
            PackageBackend::Pip => self.check_pip(package),
        }
    }

    /// Check multiple packages at once.
    pub fn check_packages(&self, packages: &[String]) -> Result<Vec<ConflictCheckResult>> {
        packages.iter().map(|p| self.check_package(p)).collect()
    }

    // === uv implementation ===

    fn list_uv(&self) -> Result<Vec<PackageInfo>> {
        self.log_command(&format!(
            "uv pip list --python {} --format json",
            self.python_executable.display()
        ));

        let output = Command::new("uv")
            .args([
                "pip",
                "list",
                "--python",
                &self.python_executable.to_string_lossy(),
                "--format",
                "json",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(EchidnaError::PackageError(format!(
                "uv pip list failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.log_output(&stdout);

        // uv returns JSON array of {name, version} objects
        let packages: Vec<UvPackage> = serde_json::from_str(&stdout).map_err(|e| {
            EchidnaError::PackageError(format!(
                "Failed to parse uv package list: {}. Output: {}",
                e,
                truncate_for_error(&stdout)
            ))
        })?;
        Ok(packages
            .into_iter()
            .map(|p| PackageInfo {
                name: p.name,
                version: p.version,
            })
            .collect())
    }

    fn check_uv(&self, package: &str) -> Result<ConflictCheckResult> {
        self.log_command(&format!(
            "uv pip install --python {} --dry-run {}",
            self.python_executable.display(),
            package
        ));

        let output = Command::new("uv")
            .args([
                "pip",
                "install",
                "--python",
                &self.python_executable.to_string_lossy(),
                "--dry-run",
                package,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        self.log_output(&format!("stdout: {}\nstderr: {}", stdout, stderr));

        self.parse_uv_check_output(package, output.status.success(), &stdout, &stderr)
    }

    fn parse_uv_check_output(
        &self,
        package: &str,
        success: bool,
        stdout: &str,
        stderr: &str,
    ) -> Result<ConflictCheckResult> {
        let mut result = ConflictCheckResult {
            package: package.to_string(),
            installable: success,
            conflicts: Vec::new(),
            would_install: Vec::new(),
            would_change: Vec::new(),
        };

        if success {
            // Parse stdout for packages that would be installed/upgraded
            // uv dry-run output format: "Would install package==version"
            for line in stdout.lines().chain(stderr.lines()) {
                let line = line.trim();
                if line.starts_with("Would install") || line.starts_with("Resolved") {
                    // Skip header lines
                    continue;
                }
                // Lines like: " + package==version" or " - package==version → package==version"
                if let Some(pkg) = parse_uv_package_line(line) {
                    match pkg {
                        UvDryRunLine::Install(name, version) => {
                            result.would_install.push(PackageInfo { name, version });
                        }
                        UvDryRunLine::Change(name, from, to) => {
                            result.would_change.push(PackageChange {
                                name,
                                from_version: from,
                                to_version: to,
                            });
                        }
                    }
                }
            }
        } else {
            // Parse stderr for conflict information
            // uv outputs errors like: "error: Because package requires X, but Y is installed..."
            for line in stderr.lines() {
                let line = line.trim();
                if line.starts_with("error:")
                    || line.contains("conflict")
                    || line.contains("requires")
                {
                    result.conflicts.push(ConflictInfo {
                        package: package.to_string(),
                        reason: line.to_string(),
                    });
                }
            }
            if result.conflicts.is_empty() && !stderr.is_empty() {
                // Fallback: include the entire stderr as a conflict reason
                result.conflicts.push(ConflictInfo {
                    package: package.to_string(),
                    reason: stderr.trim().to_string(),
                });
            }
        }

        Ok(result)
    }

    // === pip implementation ===

    fn list_pip(&self) -> Result<Vec<PackageInfo>> {
        self.log_command(&format!(
            "{} -m pip list --format json",
            self.python_executable.display()
        ));

        let output = Command::new(&self.python_executable)
            .args(["-m", "pip", "list", "--format", "json"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(EchidnaError::PackageError(format!(
                "pip list failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.log_output(&stdout);

        // pip returns JSON array of {name, version} objects
        let packages: Vec<PipPackage> = serde_json::from_str(&stdout).map_err(|e| {
            EchidnaError::PackageError(format!(
                "Failed to parse pip package list: {}. Output: {}",
                e,
                truncate_for_error(&stdout)
            ))
        })?;
        Ok(packages
            .into_iter()
            .map(|p| PackageInfo {
                name: p.name,
                version: p.version,
            })
            .collect())
    }

    fn check_pip(&self, package: &str) -> Result<ConflictCheckResult> {
        self.log_command(&format!(
            "{} -m pip install --dry-run {}",
            self.python_executable.display(),
            package
        ));

        let output = Command::new(&self.python_executable)
            .args(["-m", "pip", "install", "--dry-run", package])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        self.log_output(&format!("stdout: {}\nstderr: {}", stdout, stderr));

        self.parse_pip_check_output(package, output.status.success(), &stdout, &stderr)
    }

    fn parse_pip_check_output(
        &self,
        package: &str,
        success: bool,
        stdout: &str,
        stderr: &str,
    ) -> Result<ConflictCheckResult> {
        let mut result = ConflictCheckResult {
            package: package.to_string(),
            installable: success,
            conflicts: Vec::new(),
            would_install: Vec::new(),
            would_change: Vec::new(),
        };

        if success {
            // Parse stdout and stderr for packages that would be installed
            // pip dry-run output format varies by version - check both streams
            // Format: "Would install pkg1-1.0.0 pkg2-2.0.0 ..."
            for line in stdout.lines().chain(stderr.lines()) {
                let line = line.trim();
                if line.starts_with("Would install") {
                    let packages_str = line.strip_prefix("Would install").unwrap_or("").trim();
                    for pkg_str in packages_str.split_whitespace() {
                        if let Some((name, version)) = parse_pip_package_version(pkg_str) {
                            result.would_install.push(PackageInfo { name, version });
                        }
                    }
                }
            }
        } else {
            // Parse stderr for conflict information
            for line in stderr.lines() {
                let line = line.trim();
                if line.contains("ERROR:")
                    || line.contains("conflict")
                    || line.contains("incompatible")
                {
                    result.conflicts.push(ConflictInfo {
                        package: package.to_string(),
                        reason: line.to_string(),
                    });
                }
            }
            if result.conflicts.is_empty() && !stderr.is_empty() {
                result.conflicts.push(ConflictInfo {
                    package: package.to_string(),
                    reason: stderr.trim().to_string(),
                });
            }
        }

        Ok(result)
    }

    fn log_command(&self, cmd: &str) {
        if self.verbosity >= 1 {
            eprintln!("[echidna] Executing: {}", cmd);
        }
    }

    fn log_output(&self, output: &str) {
        if self.verbosity >= 2 {
            eprintln!("[echidna] Output:\n{}", output);
        }
    }
}

// === Helper types for JSON parsing ===

#[derive(serde::Deserialize)]
struct UvPackage {
    name: String,
    version: String,
}

#[derive(serde::Deserialize)]
struct PipPackage {
    name: String,
    version: String,
}

// === Output parsing helpers ===

enum UvDryRunLine {
    Install(String, String),        // name, version
    Change(String, String, String), // name, from_version, to_version
}

fn parse_uv_package_line(line: &str) -> Option<UvDryRunLine> {
    let line = line.trim();

    // Handle lines like: " + package==version"
    if let Some(rest) = line.strip_prefix('+') {
        let rest = rest.trim();
        if let Some((name, version)) = parse_name_version(rest) {
            return Some(UvDryRunLine::Install(name, version));
        }
    }

    // Handle lines like: " ~ package==old → package==new"
    if let Some(rest) = line.strip_prefix('~') {
        let rest = rest.trim();
        if let Some((left, right)) = rest.split_once('→') {
            let left = left.trim();
            let right = right.trim();
            if let (Some((name, from)), Some((_, to))) =
                (parse_name_version(left), parse_name_version(right))
            {
                return Some(UvDryRunLine::Change(name, from, to));
            }
        }
    }

    None
}

fn parse_name_version(s: &str) -> Option<(String, String)> {
    // Parse "package==version" or "package-version"
    if let Some((name, version)) = s.split_once("==") {
        return Some((name.to_string(), version.to_string()));
    }
    None
}

fn parse_pip_package_version(s: &str) -> Option<(String, String)> {
    // pip uses format "package-version" where version starts with a digit
    // Find the last hyphen that's followed by a digit
    let s = s.trim();
    for (i, _) in s.rmatch_indices('-') {
        if let Some(rest) = s.get(i + 1..) {
            if rest
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                let name = &s[..i];
                let version = rest;
                return Some((name.to_string(), version.to_string()));
            }
        }
    }
    None
}

/// Truncate a string for error messages.
fn truncate_for_error(s: &str) -> String {
    const MAX_LEN: usize = 200;
    if s.len() <= MAX_LEN {
        s.to_string()
    } else {
        format!("{}...", &s[..MAX_LEN])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uv_package_line_install() {
        let line = " + numpy==1.26.0";
        let result = parse_uv_package_line(line);
        assert!(
            matches!(result, Some(UvDryRunLine::Install(n, v)) if n == "numpy" && v == "1.26.0")
        );
    }

    #[test]
    fn test_parse_uv_package_line_change() {
        let line = " ~ numpy==1.24.0 → numpy==1.26.0";
        let result = parse_uv_package_line(line);
        assert!(
            matches!(result, Some(UvDryRunLine::Change(n, f, t)) if n == "numpy" && f == "1.24.0" && t == "1.26.0")
        );
    }

    #[test]
    fn test_parse_pip_package_version() {
        assert_eq!(
            parse_pip_package_version("numpy-1.26.0"),
            Some(("numpy".to_string(), "1.26.0".to_string()))
        );
        assert_eq!(
            parse_pip_package_version("typing-extensions-4.8.0"),
            Some(("typing-extensions".to_string(), "4.8.0".to_string()))
        );
    }

    #[test]
    fn test_parse_name_version() {
        assert_eq!(
            parse_name_version("numpy==1.26.0"),
            Some(("numpy".to_string(), "1.26.0".to_string()))
        );
    }

    #[test]
    fn test_truncate_for_error_short() {
        assert_eq!(truncate_for_error("short"), "short");
    }

    #[test]
    fn test_truncate_for_error_long() {
        let long_str = "a".repeat(300);
        let result = truncate_for_error(&long_str);
        assert!(result.ends_with("..."));
        assert!(result.len() < 210); // 200 + "..."
    }
}
