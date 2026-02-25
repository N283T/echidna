//! ChimeraX executable detection.

use crate::error::Result;
use directories::BaseDirs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Returns platform-specific default ChimeraX installation paths.
fn default_paths() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let mut paths = vec![PathBuf::from(
            "/Applications/ChimeraX.app/Contents/MacOS/ChimeraX",
        )];

        // Scan for versioned app bundles (e.g., ChimeraX-1.11.1.app, ChimeraX-1.10.app)
        // Sorted by version descending so newer versions are tried first
        if let Ok(entries) = std::fs::read_dir("/Applications") {
            let mut versioned: Vec<(Vec<u32>, PathBuf)> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name();
                    let name = name.to_string_lossy();
                    // Match ChimeraX-1.11.1.app or ChimeraX-1.10-rc2025.05.21.app
                    let version_str = name
                        .strip_prefix("ChimeraX-")
                        .and_then(|s| s.strip_suffix(".app"))?;
                    // Extract version numbers (handle pre-release suffixes like "-rc2025.05.21")
                    let version_part = version_str.split('-').next()?;
                    let parts: Vec<u32> = version_part
                        .split('.')
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if parts.is_empty() {
                        return None;
                    }
                    Some((parts, e.path().join("Contents/MacOS/ChimeraX")))
                })
                .collect();
            // Sort by version numbers descending so newer versions are tried first
            versioned.sort_by(|a, b| b.0.cmp(&a.0));
            paths.extend(versioned.into_iter().map(|(_, path)| path));
        }

        // Also check user's home Applications directory
        if let Some(home) = BaseDirs::new().map(|d| d.home_dir().to_path_buf()) {
            let user_apps = home.join("Applications");
            paths.push(user_apps.join("ChimeraX.app/Contents/MacOS/ChimeraX"));

            // Scan for versioned apps in user's Applications
            if let Ok(entries) = std::fs::read_dir(&user_apps) {
                let mut versioned: Vec<(Vec<u32>, PathBuf)> = entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let name = e.file_name();
                        let name = name.to_string_lossy();
                        let version_str = name
                            .strip_prefix("ChimeraX-")
                            .and_then(|s| s.strip_suffix(".app"))?;
                        let version_part = version_str.split('-').next()?;
                        let parts: Vec<u32> = version_part
                            .split('.')
                            .filter_map(|s| s.parse().ok())
                            .collect();
                        if parts.is_empty() {
                            return None;
                        }
                        Some((parts, e.path().join("Contents/MacOS/ChimeraX")))
                    })
                    .collect();
                versioned.sort_by(|a, b| b.0.cmp(&a.0));
                paths.extend(versioned.into_iter().map(|(_, path)| path));
            }
        }

        paths
    }

    #[cfg(target_os = "windows")]
    {
        let mut paths = vec![
            PathBuf::from(r"C:\Program Files\ChimeraX\bin\ChimeraX-console.exe"),
            PathBuf::from(r"C:\Program Files\ChimeraX\bin\chimerax.exe"),
        ];

        // Older installers (pre-1.7) used versioned directories like "ChimeraX 1.3"
        if let Ok(entries) = std::fs::read_dir(r"C:\Program Files") {
            let mut versioned: Vec<(Vec<u32>, PathBuf)> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name();
                    let name = name.to_string_lossy();
                    let version_str = name.strip_prefix("ChimeraX ")?;
                    let parts: Vec<u32> = version_str
                        .split('.')
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if parts.is_empty() {
                        return None;
                    }
                    Some((parts, e.path().join(r"bin\ChimeraX-console.exe")))
                })
                .collect();
            // Sort by version numbers descending so newer versions are tried first
            versioned.sort_by(|a, b| b.0.cmp(&a.0));
            paths.extend(versioned.into_iter().map(|(_, path)| path));
        }

        paths
    }

    #[cfg(target_os = "linux")]
    {
        let mut paths = vec![
            // Symlink installed by DEB and RPM packages
            PathBuf::from("/usr/bin/chimerax"),
            PathBuf::from("/usr/local/bin/chimerax"),
            // DEB package (Ubuntu/Debian) binary
            PathBuf::from("/usr/lib/ucsf-chimerax/bin/ChimeraX"),
            // RPM package (RHEL/Rocky/CentOS) binary
            PathBuf::from("/usr/libexec/UCSF-ChimeraX/bin/ChimeraX"),
        ];

        if let Some(home) = BaseDirs::new().map(|d| d.home_dir().to_path_buf()) {
            paths.push(home.join(".local/bin/chimerax"));
        }

        paths
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        vec![]
    }
}

/// Check if a path is executable.
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    path.extension()
        .map(|ext| {
            let ext = ext.to_string_lossy().to_lowercase();
            ext == "exe" || ext == "bat" || ext == "cmd"
        })
        .unwrap_or(false)
}

#[cfg(not(any(unix, windows)))]
fn is_executable(path: &Path) -> bool {
    path.exists()
}

/// Attempt to find ChimeraX executable.
///
/// Detection order:
/// 1. CHIMERAX_PATH environment variable
/// 2. PATH search (via `which`)
/// 3. Platform-specific default paths
///
/// Note: Flatpak installations (`flatpak run edu.ucsf.rbvi.ChimeraX`) are not
/// detected because they require a different invocation method. Users should set
/// CHIMERAX_PATH or add an alias to their PATH.
pub fn find_chimerax() -> Option<PathBuf> {
    // 1. Check environment variable
    if let Ok(path) = std::env::var("CHIMERAX_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() && is_executable(&p) {
            return Some(p);
        }
    }

    // 2. Check PATH (using which)
    if let Ok(path) = which::which("chimerax") {
        return Some(path);
    }

    #[cfg(target_os = "macos")]
    if let Ok(path) = which::which("ChimeraX") {
        return Some(path);
    }

    // 3. Check default installation paths
    default_paths()
        .into_iter()
        .find(|path| path.exists() && is_executable(path))
}

/// Get the ChimeraX version string from an executable.
///
/// This runs ChimeraX with a Python snippet to extract the version,
/// which is more reliable than parsing --version output.
pub fn get_chimerax_version(chimerax: &Path) -> Result<String> {
    let output = Command::new(chimerax)
        .args([
            "-c",
            "import chimerax; print('VERSION:' + chimerax.__version__)",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Extract version from output
    for line in stdout.lines() {
        if let Some(version) = line.strip_prefix("VERSION:") {
            return Ok(version.trim().to_string());
        }
    }

    // No fallback - require explicit VERSION: prefix for reliability
    Err(crate::error::EchidnaError::ChimeraXCommandFailed(
        "Could not extract ChimeraX version from output".into(),
    ))
}

/// Check if a path is a symlink and return the link target.
///
/// Returns `Some(target)` if the path is a symlink, `None` otherwise.
pub fn get_symlink_target(path: &Path) -> Option<PathBuf> {
    if path.is_symlink() {
        std::fs::read_link(path).ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_paths_not_empty() {
        let paths = default_paths();
        // At least one default path should be defined for supported platforms
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_get_symlink_target_nonexistent() {
        let path = Path::new("/nonexistent/path/to/chimerax");
        assert!(get_symlink_target(path).is_none());
    }

    #[test]
    fn test_get_symlink_target_regular_file() {
        // Create a temp file (not a symlink)
        let temp = tempfile::NamedTempFile::new().unwrap();
        assert!(get_symlink_target(temp.path()).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn test_get_symlink_target_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        std::fs::write(&target, "").unwrap();
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert_eq!(get_symlink_target(&link), Some(target));
    }
}
