//! CLI integration tests for echidna.

use assert_cmd::Command;
use predicates::prelude::*;

/// Get a command to run echidna.
#[allow(deprecated)]
fn echi() -> Command {
    Command::cargo_bin("echi").unwrap()
}

#[test]
fn test_help() {
    echi()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("ChimeraX Bundle Development CLI"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("install"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("packages"))
        .stdout(predicate::str::contains("venv"))
        .stdout(predicate::str::contains("clean"))
        .stdout(predicate::str::contains("validate"))
        .stdout(predicate::str::contains("info"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("watch"))
        .stdout(predicate::str::contains("version"))
        .stdout(predicate::str::contains("debug"))
        .stdout(predicate::str::contains("workspace"));
}

#[test]
fn test_version() {
    echi()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_init_help() {
    echi()
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
    echi()
        .args(["build", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Build the bundle wheel"))
        .stdout(predicate::str::contains("--clean"));
}

#[test]
fn test_install_help() {
    echi()
        .args(["install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Install the bundle to ChimeraX"))
        .stdout(predicate::str::contains("--user"));
}

#[test]
fn test_run_help() {
    echi()
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
    echi()
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
    echi()
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
    echi()
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
    echi()
        .arg("invalid-subcommand")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_no_subcommand() {
    echi()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn test_global_verbose_flag() {
    // --verbose is a global flag
    echi()
        .args(["--verbose", "init", "--help"])
        .assert()
        .success();
}

#[test]
fn test_global_chimerax_option() {
    // --chimerax is a global option
    echi()
        .args(["--chimerax", "/path/to/chimerax", "init", "--help"])
        .assert()
        .success();
}

#[test]
fn test_completions_bash() {
    echi()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_echi()"));
}

#[test]
fn test_completions_zsh() {
    echi()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef echi"));
}

#[test]
fn test_completions_fish() {
    echi()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -c echi"));
}

#[test]
fn test_completions_powershell() {
    echi()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Register-ArgumentCompleter"));
}

#[test]
fn test_venv_help() {
    echi()
        .args(["venv", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Set up virtual environment with ChimeraX Python",
        ))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--force"))
        .stdout(predicate::str::contains("--no-config"))
        .stdout(predicate::str::contains("--configs"));
}

#[test]
fn test_setup_ide_alias() {
    // setup-ide should still work as hidden alias
    echi()
        .args(["setup-ide", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alias for 'venv'"));
}

#[test]
fn test_clean_help() {
    echi()
        .args(["clean", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Clean build artifacts"))
        .stdout(predicate::str::contains("--all"))
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn test_validate_help() {
    echi()
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
    echi()
        .args(["info", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Show bundle information and status",
        ));
}

#[test]
fn test_test_help() {
    echi()
        .args(["test", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Run tests using ChimeraX Python environment",
        ))
        .stdout(predicate::str::contains("--filter"))
        .stdout(predicate::str::contains("--no-build"))
        .stdout(predicate::str::contains("--no-install"))
        .stdout(predicate::str::contains("--coverage"))
        .stdout(predicate::str::contains("--smoke"));
}

#[test]
fn test_watch_help() {
    echi()
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
    echi()
        .args(["version", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Manage bundle version in pyproject.toml",
        ));
}

#[test]
fn test_debug_help() {
    echi()
        .args(["debug", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Launch ChimeraX in debug mode"))
        .stdout(predicate::str::contains("--no-build"))
        .stdout(predicate::str::contains("--no-install"));
}

#[test]
fn test_workspace_help() {
    echi()
        .args(["workspace", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage bundle workspaces"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("list"));
}

#[test]
fn test_workspace_init_help() {
    echi()
        .args(["workspace", "init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize a workspace"))
        .stdout(predicate::str::contains("--force"));
}

#[test]
fn test_workspace_list_help() {
    echi()
        .args(["workspace", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List workspace members"));
}

#[test]
fn test_build_all_flag() {
    echi()
        .args(["build", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--all"));
}

#[test]
fn test_test_all_flag() {
    echi()
        .args(["test", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--all"));
}
