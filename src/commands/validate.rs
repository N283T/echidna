//! `echidna validate` command implementation.

use crate::error::{EchidnaError, Result};
use std::path::{Path, PathBuf};

/// Arguments for the validate command.
pub struct ValidateArgs {
    /// Project directory
    pub path: PathBuf,
}

/// Validation result with issues found.
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }

    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }
}

/// Execute the validate command.
pub fn execute(args: ValidateArgs) -> Result<()> {
    let project_dir = args.path.canonicalize().unwrap_or(args.path.clone());

    println!("Validating bundle in {}...", project_dir.display());
    println!();

    let result = validate_bundle(&project_dir)?;

    // Print warnings
    for warning in &result.warnings {
        println!("  ⚠ {}", warning);
    }

    // Print errors
    for error in &result.errors {
        println!("  ✗ {}", error);
    }

    println!();

    if result.is_valid() {
        println!("✓ Bundle is valid");
        if !result.warnings.is_empty() {
            println!(
                "  ({} warning{})",
                result.warnings.len(),
                if result.warnings.len() == 1 { "" } else { "s" }
            );
        }
        Ok(())
    } else {
        println!(
            "✗ Validation failed with {} error{}",
            result.errors.len(),
            if result.errors.len() == 1 { "" } else { "s" }
        );
        Err(EchidnaError::ConfigError("bundle validation failed".into()))
    }
}

/// Validate a bundle directory structure and configuration.
pub fn validate_bundle(project_dir: &Path) -> Result<ValidationResult> {
    let mut result = ValidationResult::default();

    // Check pyproject.toml exists
    let pyproject_path = project_dir.join("pyproject.toml");
    if !pyproject_path.exists() {
        result.add_error("pyproject.toml not found");
        return Ok(result);
    }

    // Parse pyproject.toml
    let content = std::fs::read_to_string(&pyproject_path)?;
    let pyproject: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            result.add_error(format!("Failed to parse pyproject.toml: {}", e));
            return Ok(result);
        }
    };

    // Validate [build-system]
    validate_build_system(&pyproject, &mut result);

    // Validate [project]
    validate_project_section(&pyproject, &mut result);

    // Validate [chimerax]
    validate_chimerax_section(&pyproject, &mut result);

    // Validate source directory structure
    validate_source_structure(project_dir, &mut result);

    Ok(result)
}

/// Validate [build-system] section.
fn validate_build_system(pyproject: &toml::Value, result: &mut ValidationResult) {
    let build_system = match pyproject.get("build-system") {
        Some(bs) => bs,
        None => {
            result.add_error("[build-system] section missing");
            return;
        }
    };

    // Check requires
    if let Some(requires) = build_system.get("requires") {
        if let Some(requires_array) = requires.as_array() {
            let has_bundle_builder = requires_array.iter().any(|r| {
                r.as_str()
                    .map(|s| s.contains("ChimeraX-BundleBuilder"))
                    .unwrap_or(false)
            });
            if !has_bundle_builder {
                result.add_error("[build-system].requires must include 'ChimeraX-BundleBuilder'");
            }
        } else {
            result.add_error("[build-system].requires must be an array");
        }
    } else {
        result.add_error("[build-system].requires is missing");
    }

    // Check build-backend
    if let Some(backend) = build_system.get("build-backend") {
        if let Some(backend_str) = backend.as_str() {
            if backend_str != "chimerax.bundle_builder.cx_pep517" {
                result.add_warning(format!(
                    "Unexpected build-backend: '{}' (expected 'chimerax.bundle_builder.cx_pep517')",
                    backend_str
                ));
            }
        }
    } else {
        result.add_error("[build-system].build-backend is missing");
    }
}

/// Validate [project] section.
fn validate_project_section(pyproject: &toml::Value, result: &mut ValidationResult) {
    let project = match pyproject.get("project") {
        Some(p) => p,
        None => {
            result.add_error("[project] section missing");
            return;
        }
    };

    // Check name
    if let Some(name) = project.get("name") {
        if let Some(name_str) = name.as_str() {
            if !name_str.starts_with("ChimeraX-") {
                result.add_warning(format!(
                    "Bundle name '{}' doesn't follow convention (should start with 'ChimeraX-')",
                    name_str
                ));
            }
        } else {
            result.add_error("[project].name must be a string");
        }
    } else {
        result.add_error("[project].name is missing");
    }

    // Check version
    if project.get("version").is_none() {
        result.add_error("[project].version is missing");
    }
}

/// Validate [chimerax] section.
fn validate_chimerax_section(pyproject: &toml::Value, result: &mut ValidationResult) {
    let chimerax = match pyproject.get("chimerax") {
        Some(c) => c,
        None => {
            result.add_error("[chimerax] section missing");
            return;
        }
    };

    // Check package
    if let Some(package) = chimerax.get("package") {
        if let Some(package_str) = package.as_str() {
            if !package_str.starts_with("chimerax.") {
                result.add_warning(format!(
                    "Package '{}' doesn't follow convention (should start with 'chimerax.')",
                    package_str
                ));
            }
        } else {
            result.add_error("[chimerax].package must be a string");
        }
    } else {
        result.add_error("[chimerax].package is missing");
    }

    // Check categories (optional but recommended)
    if chimerax.get("categories").is_none() {
        result.add_warning("[chimerax].categories is not set");
    }
}

/// Validate source directory structure.
fn validate_source_structure(project_dir: &Path, result: &mut ValidationResult) {
    let src_dir = project_dir.join("src");

    if !src_dir.exists() {
        result.add_error("src/ directory not found");
        return;
    }

    // Check __init__.py exists
    let init_py = src_dir.join("__init__.py");
    if !init_py.exists() {
        result.add_error("src/__init__.py not found");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_valid_bundle(dir: &std::path::Path) {
        let pyproject = r#"
[build-system]
requires = ["ChimeraX-BundleBuilder"]
build-backend = "chimerax.bundle_builder.cx_pep517"

[project]
name = "ChimeraX-Test"
version = "0.1.0"

[chimerax]
package = "chimerax.test"
categories = ["General"]
"#;
        fs::write(dir.join("pyproject.toml"), pyproject).unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/__init__.py"), "").unwrap();
    }

    #[test]
    fn test_validate_valid_bundle() {
        let temp = TempDir::new().unwrap();
        create_valid_bundle(temp.path());

        let result = validate_bundle(temp.path()).unwrap();
        assert!(result.is_valid());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_missing_pyproject() {
        let temp = TempDir::new().unwrap();

        let result = validate_bundle(temp.path()).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("pyproject.toml")));
    }

    #[test]
    fn test_validate_missing_src() {
        let temp = TempDir::new().unwrap();
        let pyproject = r#"
[build-system]
requires = ["ChimeraX-BundleBuilder"]
build-backend = "chimerax.bundle_builder.cx_pep517"

[project]
name = "ChimeraX-Test"
version = "0.1.0"

[chimerax]
package = "chimerax.test"
"#;
        fs::write(temp.path().join("pyproject.toml"), pyproject).unwrap();

        let result = validate_bundle(temp.path()).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("src/")));
    }

    #[test]
    fn test_validate_missing_init_py() {
        let temp = TempDir::new().unwrap();
        let pyproject = r#"
[build-system]
requires = ["ChimeraX-BundleBuilder"]
build-backend = "chimerax.bundle_builder.cx_pep517"

[project]
name = "ChimeraX-Test"
version = "0.1.0"

[chimerax]
package = "chimerax.test"
"#;
        fs::write(temp.path().join("pyproject.toml"), pyproject).unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();

        let result = validate_bundle(temp.path()).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("__init__.py")));
    }

    #[test]
    fn test_validate_warning_non_standard_name() {
        let temp = TempDir::new().unwrap();
        let pyproject = r#"
[build-system]
requires = ["ChimeraX-BundleBuilder"]
build-backend = "chimerax.bundle_builder.cx_pep517"

[project]
name = "MyBundle"
version = "0.1.0"

[chimerax]
package = "chimerax.test"
categories = ["General"]
"#;
        fs::write(temp.path().join("pyproject.toml"), pyproject).unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/__init__.py"), "").unwrap();

        let result = validate_bundle(temp.path()).unwrap();
        assert!(result.is_valid()); // Warnings don't fail validation
        assert!(!result.warnings.is_empty());
        assert!(result.warnings.iter().any(|w| w.contains("ChimeraX-")));
    }
}
