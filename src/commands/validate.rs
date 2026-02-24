//! `echidna validate` command implementation.

use crate::error::{EchidnaError, Result};
use std::path::{Path, PathBuf};

/// Arguments for the validate command.
pub struct ValidateArgs {
    /// Project directory
    pub path: PathBuf,
    /// Treat warnings as errors
    pub strict: bool,
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

    /// Check if valid considering strict mode.
    pub fn is_valid_strict(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
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
    if args.strict {
        println!("  (strict mode: warnings are errors)");
    }
    println!();

    let result = validate_bundle(&project_dir)?;

    // Print warnings
    for warning in &result.warnings {
        if args.strict {
            println!("  ✗ {}", warning);
        } else {
            println!("  ⚠ {}", warning);
        }
    }

    // Print errors
    for error in &result.errors {
        println!("  ✗ {}", error);
    }

    println!();

    let is_valid = if args.strict {
        result.is_valid_strict()
    } else {
        result.is_valid()
    };

    if is_valid {
        println!("✓ Bundle is valid");
        if !args.strict && !result.warnings.is_empty() {
            println!(
                "  ({} warning{})",
                result.warnings.len(),
                if result.warnings.len() == 1 { "" } else { "s" }
            );
        }
        Ok(())
    } else {
        let error_count = if args.strict {
            result.errors.len() + result.warnings.len()
        } else {
            result.errors.len()
        };
        println!(
            "✗ Validation failed with {} error{}",
            error_count,
            if error_count == 1 { "" } else { "s" }
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
    validate_source_structure(project_dir, &pyproject, &mut result);

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

    // Check description (recommended)
    if project.get("description").is_none() {
        result.add_warning("[project].description is not set (recommended for Toolshed)");
    }

    // Check classifiers for Python version
    if let Some(classifiers) = project.get("classifiers") {
        if let Some(classifiers_array) = classifiers.as_array() {
            let has_python_classifier = classifiers_array.iter().any(|c| {
                c.as_str()
                    .map(|s| s.starts_with("Programming Language :: Python"))
                    .unwrap_or(false)
            });
            if !has_python_classifier {
                result
                    .add_warning("[project].classifiers should include Python version classifier");
            }
        }
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

    // Check min-session-version (recommended)
    if chimerax.get("min-session-version").is_none() {
        result.add_warning("[chimerax].min-session-version is not set (recommended)");
    }

    // Check min-chimerax-version (recommended)
    if chimerax.get("min-chimerax-version").is_none() {
        result.add_warning("[chimerax].min-chimerax-version is not set (recommended)");
    }
}

/// Validate source directory structure.
fn validate_source_structure(
    project_dir: &Path,
    pyproject: &toml::Value,
    result: &mut ValidationResult,
) {
    let src_dir = project_dir.join("src");

    if !src_dir.exists() {
        result.add_error("src/ directory not found");
        return;
    }

    // Check __init__.py exists
    let init_py = src_dir.join("__init__.py");
    if !init_py.exists() {
        result.add_error("src/__init__.py not found");
        return;
    }

    // Check for bundle_api or get_class in __init__.py
    if let Ok(init_content) = std::fs::read_to_string(&init_py) {
        let has_bundle_api = init_content.contains("bundle_api")
            || init_content.contains("get_class")
            || init_content.contains("BundleAPI");

        if !has_bundle_api {
            result.add_warning(
                "src/__init__.py should define bundle_api or get_class() for bundle registration",
            );
        }
    }

    // Check for declared commands/tools matching files
    if let Some(chimerax) = pyproject.get("chimerax") {
        // Check commands
        if let Some(commands) = chimerax.get("commands") {
            if commands.as_table().is_some() || commands.as_array().is_some() {
                let cmd_py = src_dir.join("cmd.py");
                if !cmd_py.exists() {
                    result
                        .add_warning("Commands declared but src/cmd.py not found (common pattern)");
                }
            }
        }

        // Check tools
        if let Some(tools) = chimerax.get("tools") {
            if tools.as_table().is_some() || tools.as_array().is_some() {
                let tool_py = src_dir.join("tool.py");
                if !tool_py.exists() {
                    result.add_warning("Tools declared but src/tool.py not found (common pattern)");
                }
            }
        }
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
description = "Test bundle"

[chimerax]
package = "chimerax.test"
categories = ["General"]
min-session-version = "1"
min-chimerax-version = "1.0"
"#;
        fs::write(dir.join("pyproject.toml"), pyproject).unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/__init__.py"), "bundle_api = None").unwrap();
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
        fs::write(temp.path().join("src/__init__.py"), "bundle_api = None").unwrap();

        let result = validate_bundle(temp.path()).unwrap();
        assert!(result.is_valid()); // Warnings don't fail validation
        assert!(!result.warnings.is_empty());
        assert!(result.warnings.iter().any(|w| w.contains("ChimeraX-")));
    }

    #[test]
    fn test_strict_mode() {
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
"#;
        fs::write(temp.path().join("pyproject.toml"), pyproject).unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/__init__.py"), "bundle_api = None").unwrap();

        let result = validate_bundle(temp.path()).unwrap();
        assert!(result.is_valid()); // Normal mode: valid
        assert!(!result.is_valid_strict()); // Strict mode: invalid (has warnings)
    }

    #[test]
    fn test_validate_missing_description_warning() {
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
categories = ["General"]
"#;
        fs::write(temp.path().join("pyproject.toml"), pyproject).unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/__init__.py"), "bundle_api = None").unwrap();

        let result = validate_bundle(temp.path()).unwrap();
        assert!(result.is_valid());
        assert!(result.warnings.iter().any(|w| w.contains("description")));
    }

    #[test]
    fn test_validate_missing_bundle_api_warning() {
        let temp = TempDir::new().unwrap();
        let pyproject = r#"
[build-system]
requires = ["ChimeraX-BundleBuilder"]
build-backend = "chimerax.bundle_builder.cx_pep517"

[project]
name = "ChimeraX-Test"
version = "0.1.0"
description = "Test"

[chimerax]
package = "chimerax.test"
categories = ["General"]
min-session-version = "1"
min-chimerax-version = "1.0"
"#;
        fs::write(temp.path().join("pyproject.toml"), pyproject).unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/__init__.py"), "# empty").unwrap();

        let result = validate_bundle(temp.path()).unwrap();
        assert!(result.is_valid());
        assert!(result.warnings.iter().any(|w| w.contains("bundle_api")));
    }

    #[test]
    fn test_validate_commands_without_cmd_py() {
        let temp = TempDir::new().unwrap();
        let pyproject = r#"
[build-system]
requires = ["ChimeraX-BundleBuilder"]
build-backend = "chimerax.bundle_builder.cx_pep517"

[project]
name = "ChimeraX-Test"
version = "0.1.0"
description = "Test"

[chimerax]
package = "chimerax.test"
categories = ["General"]
min-session-version = "1"
min-chimerax-version = "1.0"

[chimerax.commands.mycommand]
"#;
        fs::write(temp.path().join("pyproject.toml"), pyproject).unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/__init__.py"), "bundle_api = None").unwrap();

        let result = validate_bundle(temp.path()).unwrap();
        assert!(result.is_valid());
        assert!(result.warnings.iter().any(|w| w.contains("cmd.py")));
    }
}
