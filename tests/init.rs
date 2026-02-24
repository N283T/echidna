//! Integration tests for the `echidna init` command.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Get a command to run echidna.
#[allow(deprecated)]
fn echi() -> Command {
    Command::cargo_bin("echi").unwrap()
}

#[test]
fn test_init_creates_bundle_structure() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("my-tool");

    echi()
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

    echi()
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

    echi()
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

    echi()
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

    echi()
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

    echi()
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

    echi()
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

    echi()
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

    echi()
        .current_dir(temp.path())
        .args(["init", "--name", "current-dir-test", "."])
        .assert()
        .success();

    assert!(temp.path().join("pyproject.toml").exists());
}

#[test]
fn test_init_invalid_name() {
    let temp = TempDir::new().unwrap();

    echi()
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
    echi()
        .args(["init", "--name", "test", project_dir.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn test_init_shows_next_steps() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("steps");

    echi()
        .args(["init", "--name", "test", project_dir.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Next steps:"))
        .stdout(predicate::str::contains("echi build"))
        .stdout(predicate::str::contains("echi install"))
        .stdout(predicate::str::contains("echi run"));
}

#[test]
fn test_init_with_type_tool() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("my-tool");

    echi()
        .args([
            "init",
            "--type",
            "tool",
            "--name",
            "my-tool",
            project_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("type: tool (Qt)"));

    // Verify tool-specific files
    assert!(project_dir.join("pyproject.toml").exists());
    assert!(project_dir.join("src/__init__.py").exists());
    assert!(project_dir.join("src/tool.py").exists());

    // Verify pyproject.toml contains tool configuration
    let pyproject = fs::read_to_string(project_dir.join("pyproject.toml")).unwrap();
    assert!(pyproject.contains("[chimerax.tool."));
}

#[test]
fn test_init_with_type_format() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("my-format");

    echi()
        .args([
            "init",
            "--type",
            "format",
            "--name",
            "my-format",
            project_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("type: format"));

    // Verify format-specific files
    assert!(project_dir.join("src/open.py").exists());

    // Verify pyproject.toml contains format configuration
    let pyproject = fs::read_to_string(project_dir.join("pyproject.toml")).unwrap();
    assert!(pyproject.contains("[chimerax.data-format."));
}

#[test]
fn test_init_with_type_cpp() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("my-ext");

    echi()
        .args([
            "init",
            "--type",
            "cpp",
            "--name",
            "my-ext",
            project_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("type: C++ extension"));

    // Verify C++ extension-specific files
    // For C++ bundles, files are in src/chimerax/<package>/
    assert!(project_dir.join("pyproject.toml").exists());
    assert!(project_dir.join("src/chimerax/myext/__init__.py").exists());
    assert!(project_dir.join("src/chimerax/myext/cmd.py").exists());
    assert!(
        project_dir
            .join("src/chimerax/myext/_extension.cpp")
            .exists()
    );

    // Verify pyproject.toml contains extension configuration
    let pyproject = fs::read_to_string(project_dir.join("pyproject.toml")).unwrap();
    assert!(pyproject.contains("[chimerax.extension._myext]"));
    assert!(pyproject.contains("pure = false"));
}

#[test]
fn test_init_with_invalid_type() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("invalid");

    echi()
        .args([
            "init",
            "--type",
            "invalid-type",
            "--name",
            "test",
            project_dir.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_init_default_type_is_command() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("default");

    echi()
        .args(["init", "--name", "default", project_dir.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("type: command"));

    // Verify command-specific files
    assert!(project_dir.join("src/cmd.py").exists());
}

#[test]
fn test_init_with_project_name_creates_subdirectory() {
    let temp = TempDir::new().unwrap();

    // `echidna init hello-world` should create ./hello-world/
    echi()
        .current_dir(temp.path())
        .args(["init", "hello-world"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ChimeraX-HelloWorld"));

    // Verify subdirectory was created with files
    assert!(temp.path().join("hello-world/pyproject.toml").exists());
    assert!(temp.path().join("hello-world/src/__init__.py").exists());
}

#[test]
fn test_init_without_args_uses_current_dir() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("my-project");
    fs::create_dir_all(&project_dir).unwrap();

    // `echidna init` in a directory should use that directory
    echi()
        .current_dir(&project_dir)
        .args(["init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ChimeraX-MyProject"));

    // Files should be in the current directory
    assert!(project_dir.join("pyproject.toml").exists());
}

#[test]
fn test_init_with_dot_uses_current_dir() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("dot-test");
    fs::create_dir_all(&project_dir).unwrap();

    // `echidna init .` should use current directory
    echi()
        .current_dir(&project_dir)
        .args(["init", "."])
        .assert()
        .success()
        .stdout(predicate::str::contains("ChimeraX-DotTest"));

    assert!(project_dir.join("pyproject.toml").exists());
}

#[test]
fn test_init_with_name_override() {
    let temp = TempDir::new().unwrap();

    // `echidna init foo --name custom` should create ./foo/ with name "custom"
    echi()
        .current_dir(temp.path())
        .args(["init", "foo", "--name", "custom-name"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ChimeraX-CustomName"));

    // Directory is "foo" but project name is "custom-name"
    assert!(temp.path().join("foo/pyproject.toml").exists());
    let pyproject = fs::read_to_string(temp.path().join("foo/pyproject.toml")).unwrap();
    assert!(pyproject.contains("ChimeraX-CustomName"));
}
