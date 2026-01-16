//! `echidna test` command implementation.
//!
//! Note: This module is named `testing` because `test` is a reserved keyword in Rust.

use crate::chimerax::{ChimeraXExecutor, Verbosity};
use crate::commands::{build, install};
use crate::error::{EchidnaError, Result};
use std::path::PathBuf;

/// Arguments for the test command.
pub struct TestArgs {
    /// Project directory
    pub path: PathBuf,
    /// Test filter expression (-k)
    pub filter: Option<String>,
    /// Increase pytest verbosity
    pub verbose: bool,
    /// Skip build step
    pub no_build: bool,
    /// Skip install step
    pub no_install: bool,
    /// Additional pytest arguments
    pub pytest_args: Vec<String>,
    /// Path to ChimeraX executable
    pub chimerax: PathBuf,
    /// Verbosity level
    pub verbosity: Verbosity,
}

/// Test result summary.
#[derive(Debug, Default)]
pub struct TestResult {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub errors: u32,
}

impl TestResult {
    pub fn is_success(&self) -> bool {
        self.failed == 0 && self.errors == 0
    }

    pub fn total(&self) -> u32 {
        self.passed + self.failed + self.skipped + self.errors
    }
}

/// Execute the test command.
pub fn execute(args: TestArgs) -> Result<()> {
    let project_dir = args.path.canonicalize().unwrap_or(args.path.clone());

    // Check if tests directory exists
    let tests_dir = project_dir.join("tests");
    if !tests_dir.exists() {
        return Err(EchidnaError::ConfigError(
            "tests/ directory not found. Create a tests/ directory with test files.".into(),
        ));
    }

    // Build if not skipped
    if !args.no_build {
        println!("=== Building ===");
        build::execute(build::BuildArgs {
            path: project_dir.clone(),
            clean: false,
            chimerax: args.chimerax.clone(),
            verbosity: args.verbosity,
        })?;
        println!();
    }

    // Install if not skipped
    if !args.no_install {
        println!("=== Installing ===");
        install::execute(install::InstallArgs {
            path: project_dir.clone(),
            wheel: None,
            user: false,
            chimerax: args.chimerax.clone(),
            verbosity: args.verbosity,
        })?;
        println!();
    }

    println!("=== Running Tests ===");

    let executor = ChimeraXExecutor::new(args.chimerax, args.verbosity);

    // Build pytest arguments
    let mut pytest_args = vec![format!("\"{}\"", tests_dir.display())];

    if args.verbose {
        pytest_args.push("-v".to_string());
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

    // Add any additional pytest args (already validated by clap)
    for arg in &args.pytest_args {
        // Basic validation for additional args
        if arg.contains('\n') || arg.contains('\r') {
            return Err(EchidnaError::InvalidName(
                "pytest arguments cannot contain newlines".into(),
            ));
        }
        pytest_args.push(format!("\"{}\"", arg.replace('"', "\\\"")));
    }

    let pytest_args_str = pytest_args.join(", ");

    // Run pytest via ChimeraX Python
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

# Run pytest
exit_code = pytest.main([{pytest_args}])
sys.exit(exit_code)
"#,
        project_dir = project_dir.display(),
        pytest_args = pytest_args_str
    );

    let escaped = python_code.replace('\n', "\\n").replace('"', "\\\"");
    let cmd = format!("runscript python -c \"exec(\\\"{}\\\")\"", escaped);

    // Run the command and capture output
    let output = run_pytest_command(&executor, &cmd)?;

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
        println!("All tests passed!");
        Ok(())
    } else {
        let exit_code = output.status.code().unwrap_or(-1);
        Err(EchidnaError::TestFailed(exit_code))
    }
}

/// Validate pytest filter expression to prevent injection.
fn is_valid_pytest_filter(filter: &str) -> bool {
    if filter.is_empty() {
        return false;
    }

    // Allow alphanumeric, underscore, spaces, and common pytest operators
    // Reject quotes, semicolons, newlines, and other dangerous characters
    const DANGEROUS_CHARS: &[char] = &['"', '\'', ';', '\n', '\r', '`', '$', '\\'];

    for ch in DANGEROUS_CHARS {
        if filter.contains(*ch) {
            return false;
        }
    }

    true
}

/// Run pytest command and return output (without checking exit code).
fn run_pytest_command(executor: &ChimeraXExecutor, cmd: &str) -> Result<std::process::Output> {
    // We need to run the command without the automatic exit code check
    // because pytest returns non-zero on test failures
    use std::process::{Command, Stdio};

    let executable = executor.executable();

    let output = Command::new(executable)
        .args(["--nogui", "--exit", "--cmd", cmd])
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
    fn test_result_is_success() {
        let mut result = TestResult::default();
        assert!(result.is_success());

        result.passed = 10;
        assert!(result.is_success());

        result.failed = 1;
        assert!(!result.is_success());

        result.failed = 0;
        result.errors = 1;
        assert!(!result.is_success());
    }

    #[test]
    fn test_result_total() {
        let result = TestResult {
            passed: 5,
            failed: 2,
            skipped: 1,
            errors: 0,
        };
        assert_eq!(result.total(), 8);
    }
}
