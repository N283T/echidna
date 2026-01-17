//! CLI integration tests for echidna.

use assert_cmd::Command;
use predicates::prelude::*;

/// Get a command to run echidna.
#[allow(deprecated)]
fn echidna() -> Command {
    Command::cargo_bin("echidna").unwrap()
}

#[test]
fn test_help() {
    echidna()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("ChimeraX Bundle Development CLI"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("install"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("setup-ide"))
        .stdout(predicate::str::contains("clean"))
        .stdout(predicate::str::contains("validate"))
        .stdout(predicate::str::contains("info"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("watch"))
        .stdout(predicate::str::contains("version"))
        .stdout(predicate::str::contains("debug"));
}

#[test]
fn test_version() {
    echidna()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_init_help() {
    echidna()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Generate a new ChimeraX bundle project",
        ))
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--type"))
        .stdout(predicate::str::contains("--force"));
}

#[test]
fn test_build_help() {
    echidna()
        .args(["build", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Build the bundle wheel"))
        .stdout(predicate::str::contains("--clean"));
}

#[test]
fn test_install_help() {
    echidna()
        .args(["install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Install the bundle to ChimeraX"))
        .stdout(predicate::str::contains("--wheel"))
        .stdout(predicate::str::contains("--user"));
}

#[test]
fn test_run_help() {
    echidna()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Build, install, and launch ChimeraX",
        ))
        .stdout(predicate::str::contains("--script"))
        .stdout(predicate::str::contains("--no-build"))
        .stdout(predicate::str::contains("--nogui"));
}

#[test]
fn test_python_help() {
    echidna()
        .args(["python", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Show ChimeraX Python environment info",
        ))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn test_docs_help() {
    echidna()
        .args(["docs", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Open ChimeraX documentation"))
        .stdout(predicate::str::contains("--dev"))
        .stdout(predicate::str::contains("--api"))
        .stdout(predicate::str::contains("--search"));
}

#[test]
fn test_publish_help() {
    echidna()
        .args(["publish", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Publish bundle to ChimeraX Toolshed",
        ))
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn test_invalid_subcommand() {
    echidna()
        .arg("invalid-subcommand")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_no_subcommand() {
    echidna()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn test_global_verbose_flag() {
    // --verbose is a global flag
    echidna()
        .args(["--verbose", "init", "--help"])
        .assert()
        .success();
}

#[test]
fn test_global_chimerax_option() {
    // --chimerax is a global option
    echidna()
        .args(["--chimerax", "/path/to/chimerax", "init", "--help"])
        .assert()
        .success();
}

#[test]
fn test_completions_bash() {
    echidna()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_echidna()"));
}

#[test]
fn test_completions_zsh() {
    echidna()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef echidna"));
}

#[test]
fn test_completions_fish() {
    echidna()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -c echidna"));
}

#[test]
fn test_completions_powershell() {
    echidna()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Register-ArgumentCompleter"));
}

#[test]
fn test_setup_ide_help() {
    echidna()
        .args(["setup-ide", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Set up IDE/type checker environment",
        ))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--force"))
        .stdout(predicate::str::contains("--no-config"))
        .stdout(predicate::str::contains("--configs"));
}

#[test]
fn test_clean_help() {
    echidna()
        .args(["clean", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Clean build artifacts"))
        .stdout(predicate::str::contains("--all"))
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn test_validate_help() {
    echidna()
        .args(["validate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Validate bundle structure and configuration",
        ))
        .stdout(predicate::str::contains("--strict"));
}

#[test]
fn test_info_help() {
    echidna()
        .args(["info", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Show bundle information and status",
        ));
}

#[test]
fn test_test_help() {
    echidna()
        .args(["test", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Run tests using ChimeraX Python environment",
        ))
        .stdout(predicate::str::contains("--filter"))
        .stdout(predicate::str::contains("--no-build"))
        .stdout(predicate::str::contains("--no-install"))
        .stdout(predicate::str::contains("--coverage"));
}

#[test]
fn test_watch_help() {
    echidna()
        .args(["watch", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Watch for changes and auto-rebuild",
        ))
        .stdout(predicate::str::contains("--run"))
        .stdout(predicate::str::contains("--test"));
}

#[test]
fn test_version_help() {
    echidna()
        .args(["version", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Manage bundle version in pyproject.toml",
        ));
}

#[test]
fn test_debug_help() {
    echidna()
        .args(["debug", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Launch ChimeraX in debug mode"))
        .stdout(predicate::str::contains("--pdb"))
        .stdout(predicate::str::contains("--profile"))
        .stdout(predicate::str::contains("--no-build"))
        .stdout(predicate::str::contains("--no-install"));
}
