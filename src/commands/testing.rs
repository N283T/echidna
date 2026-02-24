//! `echidna test` command implementation.
//!
//! Note: This module is named `testing` because `test` is a reserved keyword in Rust.

use crate::chimerax::{ChimeraXExecutor, Verbosity};
use crate::commands::{build, install};
use crate::error::{EchidnaError, Result};
use crate::ui;
use colored::Colorize;
use std::path::PathBuf;

/// Arguments for the test command.
pub struct TestArgs {
    /// Project directory
    pub path: PathBuf,
    /// Test filter expression (-k)
    pub filter: Option<String>,
    /// Increase pytest verbosity
    pub pytest_verbose: bool,
    /// Skip build step
    pub no_build: bool,
    /// Skip install step
    pub no_install: bool,
    /// Generate coverage report
    pub coverage: bool,
    /// Run smoke test (scripts/smoke.cxc) instead of pytest
    pub smoke: bool,
    /// Additional pytest arguments
    pub pytest_args: Vec<String>,
    /// Path to ChimeraX executable
    pub chimerax: PathBuf,
    /// Verbosity level
    pub verbosity: Verbosity,
}

/// Execute the test command.
pub fn execute(args: TestArgs) -> Result<()> {
    let project_dir = args.path.canonicalize().unwrap_or(args.path.clone());

    // Smoke test mode
    if args.smoke {
        return execute_smoke_test(&args, &project_dir);
    }

    // Check if tests directory exists
    let tests_dir = project_dir.join("tests");
    if !tests_dir.exists() {
        return Err(EchidnaError::ConfigError(
            "tests/ directory not found. Create a tests/ directory with test files.".into(),
        ));
    }

    // Build if not skipped
    if !args.no_build {
        println!("{}", "=== Building ===".cyan().bold());
        build::execute_quiet(build::BuildArgs {
            path: project_dir.clone(),
            clean: false,
            chimerax: args.chimerax.clone(),
            verbosity: args.verbosity,
        })?;
        println!();
    }

    // Install if not skipped
    if !args.no_install {
        println!("{}", "=== Installing ===".cyan().bold());
        install::execute_quiet(install::InstallArgs {
            path: project_dir.clone(),
            wheel: None,
            user: false,
            chimerax: args.chimerax.clone(),
            verbosity: args.verbosity,
        })?;
        println!();
    }

    println!("{}", "=== Running Tests ===".cyan().bold());
    if args.coverage {
        println!("  (coverage enabled)");
    }

    let executor = ChimeraXExecutor::new(args.chimerax, args.verbosity);

    // Build pytest arguments
    let mut pytest_args = vec![format!("\"{}\"", tests_dir.display())];

    if args.pytest_verbose {
        pytest_args.push("-v".to_string());
    }

    // Add coverage arguments
    if args.coverage {
        // Get the package name from pyproject.toml for coverage source
        let package_name = get_package_name(&project_dir);
        if let Some(pkg) = package_name {
            pytest_args.push(format!("\"--cov={}\"", pkg));
        } else {
            pytest_args.push("\"--cov=src\"".to_string());
        }
        pytest_args.push("\"--cov-report=term-missing\"".to_string());
        pytest_args.push("\"--cov-report=html:htmlcov\"".to_string());
    }

    if let Some(filter) = &args.filter {
        // Validate filter to prevent injection
        if !is_valid_pytest_filter(filter) {
            return Err(EchidnaError::InvalidName(format!(
                "Invalid test filter: {}",
                filter
            )));
        }
        pytest_args.push(format!("-k \"{}\"", filter));
    }

    // Add any additional pytest args with validation
    for arg in &args.pytest_args {
        if !is_valid_pytest_arg(arg) {
            return Err(EchidnaError::InvalidName(format!(
                "Invalid pytest argument: {}",
                arg
            )));
        }
        pytest_args.push(format!("\"{}\"", arg.replace('"', "\\\"")));
    }

    let pytest_args_str = pytest_args.join(", ");

    // Run pytest via ChimeraX Python
    let coverage_check = if args.coverage {
        r#"
# Check for pytest-cov
try:
    import pytest_cov
except ImportError:
    print("ERROR: pytest-cov is not installed in ChimeraX Python environment")
    print("Install it with: ChimeraX -m pip install pytest-cov")
    sys.exit(1)
"#
    } else {
        ""
    };

    let python_code = format!(
        r#"
import sys
import os

# Change to project directory
os.chdir("{project_dir}")

# Try to import pytest
try:
    import pytest
except ImportError:
    print("ERROR: pytest is not installed in ChimeraX Python environment")
    print("Install it with: ChimeraX -m pip install pytest")
    sys.exit(1)
{coverage_check}
# Run pytest
exit_code = pytest.main([{pytest_args}])
sys.exit(exit_code)
"#,
        project_dir = project_dir.display(),
        coverage_check = coverage_check,
        pytest_args = pytest_args_str
    );

    // Run the command and capture output
    let output = run_pytest_command(&executor, &python_code)?;

    // Parse and display results
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Print test output
    println!("{}", stdout);
    if !stderr.is_empty() {
        eprintln!("{}", stderr);
    }

    // Check exit code
    if output.status.success() {
        println!();
        ui::success("All tests passed!");
        if args.coverage {
            println!();
            ui::hint("Coverage report generated:");
            ui::hint(&format!(
                "  HTML: {}/htmlcov/index.html",
                project_dir.display()
            ));
        }
        Ok(())
    } else {
        let exit_code = output.status.code().unwrap_or(-1);
        Err(EchidnaError::TestFailed(exit_code))
    }
}

/// Execute smoke test (scripts/smoke.cxc).
fn execute_smoke_test(args: &TestArgs, project_dir: &std::path::Path) -> Result<()> {
    // Check if smoke script exists
    let smoke_script = project_dir.join("scripts/smoke.cxc");
    if !smoke_script.exists() {
        return Err(EchidnaError::ConfigError(
            "scripts/smoke.cxc not found. Create a smoke test script first.".into(),
        ));
    }

    // Build if not skipped
    if !args.no_build {
        println!("{}", "=== Building ===".cyan().bold());
        build::execute_quiet(build::BuildArgs {
            path: project_dir.to_path_buf(),
            clean: false,
            chimerax: args.chimerax.clone(),
            verbosity: args.verbosity,
        })?;
        println!();
    }

    // Install if not skipped
    if !args.no_install {
        println!("{}", "=== Installing ===".cyan().bold());
        install::execute_quiet(install::InstallArgs {
            path: project_dir.to_path_buf(),
            wheel: None,
            user: false,
            chimerax: args.chimerax.clone(),
            verbosity: args.verbosity,
        })?;
        println!();
    }

    println!("{}", "=== Running Smoke Test ===".cyan().bold());
    ui::hint(&format!("Script: {}", smoke_script.display()));

    let executor = ChimeraXExecutor::new(args.chimerax.clone(), args.verbosity);

    // Run smoke.cxc via ChimeraX in nogui mode
    let output = executor.run_script(&smoke_script)?;

    // Display output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.is_empty() {
        println!("{}", stdout);
    }
    if !stderr.is_empty() {
        eprintln!("{}", stderr);
    }

    // Check exit code
    if output.status.success() {
        println!();
        ui::success("Smoke test passed!");
        Ok(())
    } else {
        let exit_code = output.status.code().unwrap_or(-1);
        Err(EchidnaError::TestFailed(exit_code))
    }
}

/// Get the package name from pyproject.toml for coverage.
fn get_package_name(project_dir: &std::path::Path) -> Option<String> {
    let pyproject_path = project_dir.join("pyproject.toml");
    let content = std::fs::read_to_string(pyproject_path).ok()?;
    let pyproject: toml::Value = toml::from_str(&content).ok()?;

    pyproject
        .get("chimerax")?
        .get("package")?
        .as_str()
        .map(|s| s.to_string())
}

/// Characters that are dangerous in shell/Python contexts.
const DANGEROUS_CHARS: &[char] = &['"', '\'', ';', '\n', '\r', '`', '$', '\\'];

/// Validate pytest filter expression to prevent injection.
fn is_valid_pytest_filter(filter: &str) -> bool {
    if filter.is_empty() {
        return false;
    }

    // Allow alphanumeric, underscore, spaces, and common pytest operators
    // Reject quotes, semicolons, newlines, and other dangerous characters
    for ch in DANGEROUS_CHARS {
        if filter.contains(*ch) {
            return false;
        }
    }

    true
}

/// Validate additional pytest arguments to prevent injection.
fn is_valid_pytest_arg(arg: &str) -> bool {
    // Reject dangerous characters that could enable injection
    for ch in DANGEROUS_CHARS {
        if arg.contains(*ch) {
            return false;
        }
    }

    true
}

/// Run pytest command and return output (without checking exit code).
fn run_pytest_command(executor: &ChimeraXExecutor, code: &str) -> Result<std::process::Output> {
    // We need to run the command without the automatic exit code check
    // because pytest returns non-zero on test failures
    // Use ChimeraX's -c flag to run Python code directly
    use std::process::{Command, Stdio};

    let executable = executor.executable();

    let output = Command::new(executable)
        .args(["-c", code])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_pytest_filter() {
        assert!(is_valid_pytest_filter("test_foo"));
        assert!(is_valid_pytest_filter("test_foo or test_bar"));
        assert!(is_valid_pytest_filter("test_foo and not test_slow"));
        assert!(is_valid_pytest_filter("TestClass"));
        assert!(is_valid_pytest_filter("test_[param1]"));
    }

    #[test]
    fn test_invalid_pytest_filter() {
        assert!(!is_valid_pytest_filter(""));
        assert!(!is_valid_pytest_filter("test; rm -rf /"));
        assert!(!is_valid_pytest_filter("test\nimport os"));
        assert!(!is_valid_pytest_filter("test\"injection"));
        assert!(!is_valid_pytest_filter("test'injection"));
        assert!(!is_valid_pytest_filter("test`cmd`"));
        assert!(!is_valid_pytest_filter("$HOME"));
    }

    #[test]
    fn test_valid_pytest_arg() {
        assert!(is_valid_pytest_arg("--cov=src"));
        assert!(is_valid_pytest_arg("-v"));
        assert!(is_valid_pytest_arg("--maxfail=2"));
        assert!(is_valid_pytest_arg("-x"));
    }

    #[test]
    fn test_invalid_pytest_arg() {
        assert!(!is_valid_pytest_arg("--cov; rm -rf /"));
        assert!(!is_valid_pytest_arg("--cov\nimport os"));
        assert!(!is_valid_pytest_arg("--cov\"injection"));
        assert!(!is_valid_pytest_arg("$HOME"));
        assert!(!is_valid_pytest_arg("`cmd`"));
    }

    #[test]
    fn test_get_package_name() {
        use std::fs;
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let pyproject = r#"
[chimerax]
package = "chimerax.mytest"
"#;
        fs::write(temp.path().join("pyproject.toml"), pyproject).unwrap();

        let pkg = get_package_name(temp.path());
        assert_eq!(pkg, Some("chimerax.mytest".to_string()));
    }

    #[test]
    fn test_get_package_name_missing() {
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let pkg = get_package_name(temp.path());
        assert_eq!(pkg, None);
    }
}
