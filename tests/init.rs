//! Integration tests for the `echidna init` command.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Get a command to run echidna.
#[allow(deprecated)]
fn echidna() -> Command {
    Command::cargo_bin("echidna").unwrap()
}

#[test]
fn test_init_creates_bundle_structure() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("my-tool");

    echidna()
        .args(["init", "--name", "my-tool", project_dir.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created ChimeraX bundle project"))
        .stdout(predicate::str::contains("ChimeraX-MyTool"));

    // Verify directory structure
    assert!(project_dir.join("pyproject.toml").exists());
    assert!(project_dir.join("src/__init__.py").exists());
    assert!(project_dir.join("src/cmd.py").exists());
    assert!(project_dir.join("scripts/smoke.cxc").exists());
    assert!(project_dir.join("README.md").exists());
}

#[test]
fn test_init_with_name_option() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("project");

    echidna()
        .args([
            "init",
            "--name",
            "custom-name",
            project_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ChimeraX-CustomName"));

    // Verify the generated content uses the custom name
    let pyproject = fs::read_to_string(project_dir.join("pyproject.toml")).unwrap();
    assert!(pyproject.contains("ChimeraX-CustomName"));
    assert!(pyproject.contains("chimerax.customname"));
}

#[test]
fn test_init_uses_directory_name_as_default() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("example-bundle");

    echidna()
        .args(["init", project_dir.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ChimeraX-ExampleBundle"));
}

#[test]
fn test_init_rejects_existing_directory_with_content() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("existing");
    fs::create_dir_all(&project_dir).unwrap();

    // Create a file to make the directory non-empty
    fs::write(project_dir.join("existing-file.txt"), "content").unwrap();

    echidna()
        .args(["init", "--name", "test", project_dir.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Directory already exists"));
}

#[test]
fn test_init_force_overwrites_existing() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("existing");
    fs::create_dir_all(&project_dir).unwrap();

    // Create a file to make the directory non-empty
    fs::write(project_dir.join("existing-file.txt"), "content").unwrap();

    echidna()
        .args([
            "init",
            "--name",
            "test",
            "--force",
            project_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created ChimeraX bundle project"));

    // Original file should still exist
    assert!(project_dir.join("existing-file.txt").exists());
    // New files should be created
    assert!(project_dir.join("pyproject.toml").exists());
}

#[test]
fn test_init_generates_valid_toml() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("toml-test");

    echidna()
        .args(["init", "--name", "toml-test", project_dir.to_str().unwrap()])
        .assert()
        .success();

    let pyproject_content = fs::read_to_string(project_dir.join("pyproject.toml")).unwrap();

    // Parse the generated TOML to verify it's valid
    let parsed: toml::Value = toml::from_str(&pyproject_content).unwrap();

    // Check required sections exist
    assert!(parsed.get("build-system").is_some());
    assert!(parsed.get("project").is_some());
    assert!(parsed.get("chimerax").is_some());
}

#[test]
fn test_init_with_bundle_name_override() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("custom");

    echidna()
        .args([
            "init",
            "--name",
            "base",
            "--bundle-name",
            "ChimeraX-CustomBundle",
            project_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ChimeraX-CustomBundle"));

    let pyproject = fs::read_to_string(project_dir.join("pyproject.toml")).unwrap();
    assert!(pyproject.contains("ChimeraX-CustomBundle"));
}

#[test]
fn test_init_with_package_override() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("pkg");

    echidna()
        .args([
            "init",
            "--name",
            "base",
            "--package",
            "chimerax.custom_pkg",
            project_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    let pyproject = fs::read_to_string(project_dir.join("pyproject.toml")).unwrap();
    assert!(pyproject.contains("chimerax.custom_pkg"));
}

#[test]
fn test_init_in_current_directory() {
    let temp = TempDir::new().unwrap();

    echidna()
        .current_dir(temp.path())
        .args(["init", "--name", "current-dir-test", "."])
        .assert()
        .success();

    assert!(temp.path().join("pyproject.toml").exists());
}

#[test]
fn test_init_invalid_name() {
    let temp = TempDir::new().unwrap();

    echidna()
        .args([
            "init",
            "--name",
            "invalid.name", // dots not allowed
            temp.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid name"));
}

#[test]
fn test_init_empty_directory_succeeds() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("empty");
    fs::create_dir_all(&project_dir).unwrap();

    // Empty directory should succeed without --force
    echidna()
        .args(["init", "--name", "test", project_dir.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn test_init_shows_next_steps() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("steps");

    echidna()
        .args(["init", "--name", "test", project_dir.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Next steps:"))
        .stdout(predicate::str::contains("echidna build"))
        .stdout(predicate::str::contains("echidna install"))
        .stdout(predicate::str::contains("echidna run"));
}
