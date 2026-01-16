//! ChimeraX executable detection.

use std::path::{Path, PathBuf};

/// Returns platform-specific default ChimeraX installation paths.
fn default_paths() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let mut paths = vec![PathBuf::from(
            "/Applications/ChimeraX.app/Contents/MacOS/ChimeraX",
        )];

        if let Some(home) = dirs::home_dir() {
            paths.push(home.join("Applications/ChimeraX.app/Contents/MacOS/ChimeraX"));
        }

        paths
    }

    #[cfg(target_os = "windows")]
    {
        vec![
            PathBuf::from(r"C:\Program Files\ChimeraX\bin\ChimeraX-console.exe"),
            PathBuf::from(r"C:\Program Files\ChimeraX\bin\chimerax.exe"),
        ]
    }

    #[cfg(target_os = "linux")]
    {
        let mut paths = vec![
            PathBuf::from("/usr/bin/chimerax"),
            PathBuf::from("/usr/local/bin/chimerax"),
            PathBuf::from("/opt/UCSF/ChimeraX/bin/chimerax"),
        ];

        if let Some(home) = dirs::home_dir() {
            paths.push(home.join("ChimeraX/bin/chimerax"));
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
}
