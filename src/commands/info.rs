//! `echidna info` command implementation.

use crate::chimerax::{ChimeraXExecutor, Verbosity};
use crate::error::{EchidnaError, Result};
use std::path::{Path, PathBuf};

/// Arguments for the info command.
pub struct InfoArgs {
    /// Project directory
    pub path: PathBuf,
    /// Path to ChimeraX executable (optional for basic info)
    pub chimerax: Option<PathBuf>,
    /// Verbosity level
    pub verbosity: Verbosity,
}

/// Bundle information extracted from pyproject.toml.
#[derive(Debug)]
pub struct BundleInfo {
    pub bundle_name: String,
    pub package_name: String,
    pub version: String,
    pub description: Option<String>,
    pub categories: Vec<String>,
}

/// Execute the info command.
pub fn execute(args: InfoArgs) -> Result<()> {
    let project_dir = args.path.canonicalize().unwrap_or(args.path.clone());

    // Check pyproject.toml exists
    let pyproject_path = project_dir.join("pyproject.toml");
    if !pyproject_path.exists() {
        return Err(EchidnaError::NotBundleDirectory(project_dir));
    }

    // Parse bundle info
    let info = parse_bundle_info(&pyproject_path)?;

    // Print bundle information
    println!("Bundle Information");
    println!("==================");
    println!();
    println!("Bundle name:    {}", info.bundle_name);
    println!("Package name:   {}", info.package_name);
    println!("Version:        {}", info.version);
    if let Some(ref desc) = info.description {
        println!("Description:    {}", desc);
    }
    if !info.categories.is_empty() {
        println!("Categories:     {}", info.categories.join(", "));
    }

    // Check build status
    println!();
    println!("Build Status");
    println!("------------");
    let dist_dir = project_dir.join("dist");
    if dist_dir.exists() {
        if let Ok(wheel) = crate::commands::build::find_newest_wheel(&dist_dir) {
            let wheel_name = wheel.file_name().unwrap_or_default().to_string_lossy();
            println!("Latest wheel:   {}", wheel_name);

            if let Ok(metadata) = wheel.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        let secs = elapsed.as_secs();
                        let time_str = if secs < 60 {
                            format!("{} seconds ago", secs)
                        } else if secs < 3600 {
                            format!("{} minutes ago", secs / 60)
                        } else if secs < 86400 {
                            format!("{} hours ago", secs / 3600)
                        } else {
                            format!("{} days ago", secs / 86400)
                        };
                        println!("Built:          {}", time_str);
                    }
                }
            }
        } else {
            println!("Latest wheel:   (none)");
        }
    } else {
        println!("Latest wheel:   (not built)");
    }

    // Check ChimeraX installation status if ChimeraX is available
    if let Some(chimerax_path) = args.chimerax {
        println!();
        println!("ChimeraX Status");
        println!("---------------");

        let executor = ChimeraXExecutor::new(chimerax_path, args.verbosity);

        // Get ChimeraX version
        match executor.get_python_info() {
            Ok(python_info) => {
                if let Some(cx_version) = python_info.chimerax_version {
                    println!("ChimeraX:       {}", cx_version);
                }

                // Check if bundle is installed
                let installed = check_bundle_installed(&executor, &info.package_name);
                match installed {
                    Ok(true) => println!("Installed:      Yes"),
                    Ok(false) => println!("Installed:      No"),
                    Err(_) => println!("Installed:      (unable to check)"),
                }
            }
            Err(_) => {
                println!("ChimeraX:       (unable to query)");
            }
        }
    }

    Ok(())
}

/// Parse bundle information from pyproject.toml.
fn parse_bundle_info(pyproject_path: &Path) -> Result<BundleInfo> {
    let content = std::fs::read_to_string(pyproject_path)?;
    let pyproject: toml::Value = toml::from_str(&content)?;

    // Get [project] section
    let project = pyproject
        .get("project")
        .ok_or_else(|| EchidnaError::ConfigError("[project] section missing".into()))?;

    let bundle_name = project
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| EchidnaError::ConfigError("[project].name missing".into()))?
        .to_string();

    let version = project
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| EchidnaError::ConfigError("[project].version missing".into()))?
        .to_string();

    let description = project
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Get [chimerax] section
    let chimerax = pyproject
        .get("chimerax")
        .ok_or_else(|| EchidnaError::ConfigError("[chimerax] section missing".into()))?;

    let package_name = chimerax
        .get("package")
        .and_then(|v| v.as_str())
        .ok_or_else(|| EchidnaError::ConfigError("[chimerax].package missing".into()))?
        .to_string();

    let categories = chimerax
        .get("categories")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(BundleInfo {
        bundle_name,
        package_name,
        version,
        description,
        categories,
    })
}

/// Validate that a package name is safe to use in Python code.
/// Package names should only contain alphanumeric characters, underscores, and dots.
fn is_valid_package_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // First character must be a letter or underscore
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }

    // Remaining characters must be alphanumeric, underscore, or dot
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

/// Check if a bundle is installed in ChimeraX.
fn check_bundle_installed(executor: &ChimeraXExecutor, package_name: &str) -> Result<bool> {
    // Validate package name to prevent code injection
    if !is_valid_package_name(package_name) {
        return Err(EchidnaError::ConfigError(format!(
            "Invalid package name: {}",
            package_name
        )));
    }

    // Use importlib.util.find_spec which is safer than direct import
    let python_code = format!(
        r#"
import importlib.util
spec = importlib.util.find_spec("{}")
print("INSTALLED:YES" if spec else "INSTALLED:NO")
"#,
        package_name
    );

    let escaped = python_code.replace('\n', "\\n").replace('"', "\\\"");
    let cmd = format!("runscript python -c \"exec(\\\"{}\\\")\"; exit", escaped);

    let output = executor.run_command(&cmd)?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    Ok(stdout.contains("INSTALLED:YES"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_bundle_info() {
        let temp = TempDir::new().unwrap();
        let pyproject = r#"
[project]
name = "ChimeraX-Test"
version = "1.2.3"
description = "A test bundle"

[chimerax]
package = "chimerax.test"
categories = ["General", "Analysis"]
"#;
        fs::write(temp.path().join("pyproject.toml"), pyproject).unwrap();

        let info = parse_bundle_info(&temp.path().join("pyproject.toml")).unwrap();

        assert_eq!(info.bundle_name, "ChimeraX-Test");
        assert_eq!(info.package_name, "chimerax.test");
        assert_eq!(info.version, "1.2.3");
        assert_eq!(info.description, Some("A test bundle".to_string()));
        assert_eq!(info.categories, vec!["General", "Analysis"]);
    }

    #[test]
    fn test_parse_bundle_info_minimal() {
        let temp = TempDir::new().unwrap();
        let pyproject = r#"
[project]
name = "ChimeraX-Test"
version = "0.1.0"

[chimerax]
package = "chimerax.test"
"#;
        fs::write(temp.path().join("pyproject.toml"), pyproject).unwrap();

        let info = parse_bundle_info(&temp.path().join("pyproject.toml")).unwrap();

        assert_eq!(info.bundle_name, "ChimeraX-Test");
        assert_eq!(info.version, "0.1.0");
        assert!(info.description.is_none());
        assert!(info.categories.is_empty());
    }

    #[test]
    fn test_parse_bundle_info_missing_project() {
        let temp = TempDir::new().unwrap();
        let pyproject = r#"
[chimerax]
package = "chimerax.test"
"#;
        fs::write(temp.path().join("pyproject.toml"), pyproject).unwrap();

        let result = parse_bundle_info(&temp.path().join("pyproject.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_package_names() {
        // Valid package names
        assert!(is_valid_package_name("chimerax.test"));
        assert!(is_valid_package_name("chimerax.my_tool"));
        assert!(is_valid_package_name("chimerax.mytool123"));
        assert!(is_valid_package_name("_private.module"));
        assert!(is_valid_package_name("a"));
    }

    #[test]
    fn test_invalid_package_names() {
        // Empty string
        assert!(!is_valid_package_name(""));

        // Starts with number
        assert!(!is_valid_package_name("123abc"));

        // Contains invalid characters
        assert!(!is_valid_package_name("os; rm -rf /"));
        assert!(!is_valid_package_name("chimerax.test; import os"));
        assert!(!is_valid_package_name("chimerax.test\nimport os"));
        assert!(!is_valid_package_name("package-name"));
        assert!(!is_valid_package_name("package name"));
        assert!(!is_valid_package_name("package(name)"));
    }
}
