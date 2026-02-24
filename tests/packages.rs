//! Integration tests for the packages command.

use assert_cmd::Command;
use predicates::prelude::*;

/// Get a command to run echidna.
#[allow(deprecated)]
fn echidna() -> Command {
    Command::cargo_bin("echidna").unwrap()
}

#[test]
fn test_packages_help() {
    echidna()
        .args(["packages", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage ChimeraX Python packages"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("check"))
        .stdout(predicate::str::contains("install"));
}

#[test]
fn test_packages_list_help() {
    echidna()
        .args(["packages", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List installed packages"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--include-stdlib"));
}

#[test]
fn test_packages_check_help() {
    echidna()
        .args(["packages", "check", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Check for conflicts"))
        .stdout(predicate::str::contains("--requirements"))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn test_packages_check_requires_package_or_requirements() {
    // Running check without any package or requirements file should fail
    // (assuming ChimeraX is available; if not, this will fail differently)
    // This test is skipped if ChimeraX is not available
    if std::env::var("CHIMERAX_PATH").is_err() && which::which("ChimeraX").is_err() {
        return; // Skip test if ChimeraX not available
    }

    echidna()
        .args(["packages", "check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("package name or --requirements"));
}

#[test]
fn test_packages_install_help() {
    echidna()
        .args(["packages", "install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Install packages"))
        .stdout(predicate::str::contains("--upgrade"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("PACKAGES"));
}

#[test]
fn test_packages_install_requires_packages() {
    // Running install without packages should fail
    echidna()
        .args(["packages", "install"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PACKAGES"));
}

#[test]
fn test_packages_without_subcommand() {
    // Running packages without a subcommand should show help or error
    echidna()
        .args(["packages"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("packages"));
}
