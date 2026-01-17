//! `echidna publish` command implementation.
//!
//! Validates and guides bundle submission to ChimeraX Toolshed.

use crate::commands::validate::{validate_bundle, ValidationResult};
use crate::error::{EchidnaError, Result};
use std::path::{Path, PathBuf};

/// ChimeraX Toolshed submission URL.
const TOOLSHED_SUBMIT_URL: &str = "https://cxtoolshed.rbvi.ucsf.edu/submit/";

/// Arguments for the publish command.
pub struct PublishArgs {
    /// Path to wheel file or project directory
    pub path: PathBuf,
    /// Dry run (validate without publishing)
    pub dry_run: bool,
}

/// Result of publish preparation.
#[derive(Debug)]
pub struct PublishPreparation {
    /// Validation result
    pub validation: ValidationResult,
    /// Path to wheel file (if found)
    pub wheel_path: Option<PathBuf>,
    /// Whether LICENSE file exists
    pub has_license: bool,
    /// Whether README exists
    pub has_readme: bool,
}

impl PublishPreparation {
    /// Check if ready to publish.
    pub fn is_ready(&self) -> bool {
        self.validation.is_valid() && self.has_license && self.wheel_path.is_some()
    }
}

/// Execute the publish command.
pub fn execute(args: PublishArgs) -> Result<()> {
    let project_dir = args.path.canonicalize().unwrap_or(args.path.clone());

    println!("Preparing bundle for Toolshed submission...");
    println!();

    let prep = prepare_for_publish(&project_dir)?;

    // Print validation results
    print_preparation_results(&prep);

    if args.dry_run {
        println!();
        if prep.is_ready() {
            println!("✓ Bundle is ready for submission (dry run)");
        } else {
            println!("✗ Bundle is not ready for submission");
            return Err(EchidnaError::ConfigError(
                "bundle not ready for submission".into(),
            ));
        }
        return Ok(());
    }

    // Check if ready to publish
    if !prep.is_ready() {
        println!();
        println!("✗ Bundle is not ready for submission. Fix the issues above first.");
        return Err(EchidnaError::ConfigError(
            "bundle not ready for submission".into(),
        ));
    }

    // Open Toolshed submission page
    println!();
    println!("Opening Toolshed submission page...");
    println!("  URL: {}", TOOLSHED_SUBMIT_URL);
    println!();
    println!("Upload the wheel file:");
    if let Some(ref wheel) = prep.wheel_path {
        println!("  {}", wheel.display());
    }

    open::that(TOOLSHED_SUBMIT_URL).map_err(|e| {
        EchidnaError::Io(std::io::Error::other(format!(
            "Failed to open browser: {}",
            e
        )))
    })?;

    Ok(())
}

/// Prepare bundle for publishing by running all checks.
pub fn prepare_for_publish(project_dir: &Path) -> Result<PublishPreparation> {
    // Run standard validation
    let validation = validate_bundle(project_dir)?;

    // Check for LICENSE file
    let has_license = check_license_file(project_dir);

    // Check for README
    let has_readme = check_readme_file(project_dir);

    // Find wheel file
    let wheel_path = find_wheel(project_dir);

    Ok(PublishPreparation {
        validation,
        wheel_path,
        has_license,
        has_readme,
    })
}

/// Print preparation results.
fn print_preparation_results(prep: &PublishPreparation) {
    // Validation errors
    for error in &prep.validation.errors {
        println!("  ✗ {}", error);
    }

    // Validation warnings
    for warning in &prep.validation.warnings {
        println!("  ⚠ {}", warning);
    }

    // License check
    if prep.has_license {
        println!("  ✓ LICENSE file found");
    } else {
        println!("  ✗ LICENSE file not found (required for Toolshed)");
    }

    // README check
    if prep.has_readme {
        println!("  ✓ README file found");
    } else {
        println!("  ⚠ README file not found (recommended)");
    }

    // Wheel check
    if let Some(ref wheel) = prep.wheel_path {
        println!("  ✓ Wheel found: {}", wheel.display());
    } else {
        println!("  ✗ No wheel found in dist/. Run 'echidna build' first.");
    }
}

/// Check if LICENSE file exists.
fn check_license_file(project_dir: &Path) -> bool {
    let license_names = ["LICENSE", "LICENSE.txt", "LICENSE.md", "LICENCE", "COPYING"];
    license_names
        .iter()
        .any(|name| project_dir.join(name).exists())
}

/// Check if README file exists.
fn check_readme_file(project_dir: &Path) -> bool {
    let readme_names = ["README.md", "README.txt", "README.rst", "README"];
    readme_names
        .iter()
        .any(|name| project_dir.join(name).exists())
}

/// Find the most recent wheel file in dist/.
fn find_wheel(project_dir: &Path) -> Option<PathBuf> {
    let dist_dir = project_dir.join("dist");
    if !dist_dir.exists() {
        return None;
    }

    let mut wheels: Vec<_> = std::fs::read_dir(&dist_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "whl"))
        .collect();

    // Sort by modification time (most recent first)
    wheels.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    wheels.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_valid_bundle(dir: &Path) {
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
    fn test_check_license_file() {
        let temp = TempDir::new().unwrap();

        // No license
        assert!(!check_license_file(temp.path()));

        // LICENSE
        fs::write(temp.path().join("LICENSE"), "MIT").unwrap();
        assert!(check_license_file(temp.path()));
    }

    #[test]
    fn test_check_license_file_variants() {
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("LICENSE.txt"), "MIT").unwrap();
        assert!(check_license_file(temp.path()));

        let temp2 = TempDir::new().unwrap();
        fs::write(temp2.path().join("COPYING"), "GPL").unwrap();
        assert!(check_license_file(temp2.path()));
    }

    #[test]
    fn test_check_readme_file() {
        let temp = TempDir::new().unwrap();

        // No readme
        assert!(!check_readme_file(temp.path()));

        // README.md
        fs::write(temp.path().join("README.md"), "# Test").unwrap();
        assert!(check_readme_file(temp.path()));
    }

    #[test]
    fn test_find_wheel_no_dist() {
        let temp = TempDir::new().unwrap();
        assert!(find_wheel(temp.path()).is_none());
    }

    #[test]
    fn test_find_wheel_empty_dist() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("dist")).unwrap();
        assert!(find_wheel(temp.path()).is_none());
    }

    #[test]
    fn test_find_wheel_with_wheel() {
        let temp = TempDir::new().unwrap();
        let dist = temp.path().join("dist");
        fs::create_dir(&dist).unwrap();
        fs::write(dist.join("test-0.1.0-py3-none-any.whl"), "").unwrap();

        let wheel = find_wheel(temp.path());
        assert!(wheel.is_some());
        assert!(wheel.unwrap().to_string_lossy().contains(".whl"));
    }

    #[test]
    fn test_prepare_for_publish_missing_license() {
        let temp = TempDir::new().unwrap();
        create_valid_bundle(temp.path());
        // No license file

        let prep = prepare_for_publish(temp.path()).unwrap();
        assert!(!prep.has_license);
        assert!(!prep.is_ready()); // Missing license
    }

    #[test]
    fn test_prepare_for_publish_missing_wheel() {
        let temp = TempDir::new().unwrap();
        create_valid_bundle(temp.path());
        fs::write(temp.path().join("LICENSE"), "MIT").unwrap();
        // No wheel

        let prep = prepare_for_publish(temp.path()).unwrap();
        assert!(prep.has_license);
        assert!(prep.wheel_path.is_none());
        assert!(!prep.is_ready()); // Missing wheel
    }

    #[test]
    fn test_find_wheel_selects_most_recent() {
        let temp = TempDir::new().unwrap();
        let dist = temp.path().join("dist");
        fs::create_dir(&dist).unwrap();

        // Create older wheel
        fs::write(dist.join("test-0.1.0-py3-none-any.whl"), "old").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Create newer wheel
        fs::write(dist.join("test-0.2.0-py3-none-any.whl"), "new").unwrap();

        let wheel = find_wheel(temp.path());
        assert!(wheel.is_some());
        assert!(wheel.unwrap().to_string_lossy().contains("0.2.0"));
    }

    #[test]
    fn test_prepare_for_publish_ready() {
        let temp = TempDir::new().unwrap();
        create_valid_bundle(temp.path());
        fs::write(temp.path().join("LICENSE"), "MIT").unwrap();
        fs::write(temp.path().join("README.md"), "# Test").unwrap();

        // Create wheel
        let dist = temp.path().join("dist");
        fs::create_dir(&dist).unwrap();
        fs::write(dist.join("ChimeraX_Test-0.1.0-py3-none-any.whl"), "").unwrap();

        let prep = prepare_for_publish(temp.path()).unwrap();
        assert!(prep.validation.is_valid());
        assert!(prep.has_license);
        assert!(prep.has_readme);
        assert!(prep.wheel_path.is_some());
        assert!(prep.is_ready());
    }
}
