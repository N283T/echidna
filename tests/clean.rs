//! Integration tests for `echidna clean` command.

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
fn test_clean_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    echidna()
        .args(["clean", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Nothing to clean"));
}

#[test]
fn test_clean_removes_build_directory() {
    let temp_dir = TempDir::new().unwrap();
    let build_dir = temp_dir.path().join("build");
    fs::create_dir(&build_dir).unwrap();
    fs::write(build_dir.join("test.txt"), "test").unwrap();

    echidna()
        .args(["clean", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleting:"))
        .stdout(predicate::str::contains("build"));

    assert!(!build_dir.exists());
}

#[test]
fn test_clean_removes_dist_directory() {
    let temp_dir = TempDir::new().unwrap();
    let dist_dir = temp_dir.path().join("dist");
    fs::create_dir(&dist_dir).unwrap();
    fs::write(dist_dir.join("test.whl"), "test").unwrap();

    echidna()
        .args(["clean", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleting:"))
        .stdout(predicate::str::contains("dist"));

    assert!(!dist_dir.exists());
}

#[test]
fn test_clean_removes_egg_info() {
    let temp_dir = TempDir::new().unwrap();
    let egg_info = temp_dir.path().join("MyBundle.egg-info");
    fs::create_dir(&egg_info).unwrap();

    echidna()
        .args(["clean", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleting:"));

    assert!(!egg_info.exists());
}

#[test]
fn test_clean_dry_run_shows_but_does_not_delete() {
    let temp_dir = TempDir::new().unwrap();
    let build_dir = temp_dir.path().join("build");
    fs::create_dir(&build_dir).unwrap();
    fs::write(build_dir.join("test.txt"), "test").unwrap();

    echidna()
        .args(["clean", "--dry-run", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"))
        .stdout(predicate::str::contains("Would delete:"))
        .stdout(predicate::str::contains("Would delete 1 item"));

    // Directory should still exist
    assert!(build_dir.exists());
}

#[test]
fn test_clean_preserves_venv_by_default() {
    let temp_dir = TempDir::new().unwrap();
    let venv_dir = temp_dir.path().join(".venv");
    fs::create_dir(&venv_dir).unwrap();
    fs::write(venv_dir.join("pyvenv.cfg"), "test").unwrap();

    echidna()
        .args(["clean", temp_dir.path().to_str().unwrap()])
        .assert()
        .success();

    // .venv should still exist
    assert!(venv_dir.exists());
}

#[test]
fn test_clean_all_removes_venv() {
    let temp_dir = TempDir::new().unwrap();
    let venv_dir = temp_dir.path().join(".venv");
    fs::create_dir(&venv_dir).unwrap();
    fs::write(venv_dir.join("pyvenv.cfg"), "test").unwrap();

    echidna()
        .args(["clean", "--all", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleting:"))
        .stdout(predicate::str::contains(".venv"));

    // .venv should be deleted
    assert!(!venv_dir.exists());
}

#[test]
fn test_clean_removes_pycache_recursively() {
    let temp_dir = TempDir::new().unwrap();

    // Create nested __pycache__ directories
    let pycache1 = temp_dir.path().join("__pycache__");
    fs::create_dir(&pycache1).unwrap();

    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    let pycache2 = src_dir.join("__pycache__");
    fs::create_dir(&pycache2).unwrap();

    echidna()
        .args(["clean", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleaned 2 item"));

    assert!(!pycache1.exists());
    assert!(!pycache2.exists());
    assert!(src_dir.exists()); // src directory should remain
}

#[test]
fn test_clean_multiple_artifacts() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple artifacts
    fs::create_dir(temp_dir.path().join("build")).unwrap();
    fs::create_dir(temp_dir.path().join("dist")).unwrap();
    fs::create_dir(temp_dir.path().join("MyBundle.egg-info")).unwrap();

    echidna()
        .args(["clean", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleaned 3 item"));

    assert!(!temp_dir.path().join("build").exists());
    assert!(!temp_dir.path().join("dist").exists());
    assert!(!temp_dir.path().join("MyBundle.egg-info").exists());
}
