//! `echidna clean` command implementation.

use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Arguments for the clean command.
pub struct CleanArgs {
    /// Project directory
    pub path: PathBuf,
    /// Also remove .venv directory
    pub all: bool,
    /// Only show what would be deleted
    pub dry_run: bool,
}

/// Directories to clean (always).
const CLEAN_DIRS: &[&str] = &["build", "dist"];

/// Glob patterns for additional cleanup.
const CLEAN_PATTERNS: &[&str] = &["*.egg-info"];

/// Execute the clean command.
pub fn execute(args: CleanArgs) -> Result<()> {
    let project_root = args.path.canonicalize().unwrap_or(args.path.clone());

    if args.dry_run {
        println!("Dry run: showing what would be deleted...");
    } else {
        println!("Cleaning build artifacts...");
    }
    println!();

    let mut deleted_count = 0;

    // Clean standard directories
    for dir_name in CLEAN_DIRS {
        let dir_path = project_root.join(dir_name);
        if dir_path.exists() {
            deleted_count += clean_path(&dir_path, args.dry_run)?;
        }
    }

    // Clean .egg-info directories
    for pattern in CLEAN_PATTERNS {
        deleted_count += clean_glob_pattern(&project_root, pattern, args.dry_run)?;
    }

    // Clean __pycache__ directories recursively
    deleted_count += clean_pycache(&project_root, args.dry_run)?;

    // Clean .venv if --all is specified
    if args.all {
        let venv_path = project_root.join(".venv");
        if venv_path.exists() {
            deleted_count += clean_path(&venv_path, args.dry_run)?;
        }
    }

    println!();
    if args.dry_run {
        if deleted_count == 0 {
            println!("Nothing to clean.");
        } else {
            println!(
                "Would delete {} item(s). Run without --dry-run to actually delete.",
                deleted_count
            );
        }
    } else if deleted_count == 0 {
        println!("Nothing to clean.");
    } else {
        println!("Cleaned {} item(s).", deleted_count);
    }

    Ok(())
}

/// Clean a single path (file or directory).
fn clean_path(path: &Path, dry_run: bool) -> Result<usize> {
    if dry_run {
        println!("  Would delete: {}", path.display());
    } else {
        println!("  Deleting: {}", path.display());
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(1)
}

/// Clean directories matching a glob pattern in the project root.
fn clean_glob_pattern(project_root: &Path, pattern: &str, dry_run: bool) -> Result<usize> {
    let mut count = 0;

    // Simple glob matching for *.egg-info pattern
    if pattern == "*.egg-info" {
        if let Ok(entries) = fs::read_dir(project_root) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if name.to_string_lossy().ends_with(".egg-info") {
                    count += clean_path(&entry.path(), dry_run)?;
                }
            }
        }
    }

    Ok(count)
}

/// Recursively clean __pycache__ directories.
fn clean_pycache(dir: &Path, dry_run: bool) -> Result<usize> {
    let mut count = 0;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                let name = entry.file_name();
                if name == "__pycache__" {
                    count += clean_path(&path, dry_run)?;
                } else if name != ".venv" && name != ".git" && name != "node_modules" {
                    // Recurse into subdirectories (skip .venv, .git, node_modules)
                    count += clean_pycache(&path, dry_run)?;
                }
            }
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_clean_empty_project() {
        let temp_dir = TempDir::new().unwrap();

        let result = execute(CleanArgs {
            path: temp_dir.path().to_path_buf(),
            all: false,
            dry_run: true,
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_clean_with_build_dir() {
        let temp_dir = TempDir::new().unwrap();
        let build_dir = temp_dir.path().join("build");
        fs::create_dir(&build_dir).unwrap();
        fs::write(build_dir.join("test.txt"), "test").unwrap();

        // Dry run should not delete
        let result = execute(CleanArgs {
            path: temp_dir.path().to_path_buf(),
            all: false,
            dry_run: true,
        });
        assert!(result.is_ok());
        assert!(build_dir.exists());

        // Actual clean should delete
        let result = execute(CleanArgs {
            path: temp_dir.path().to_path_buf(),
            all: false,
            dry_run: false,
        });
        assert!(result.is_ok());
        assert!(!build_dir.exists());
    }

    #[test]
    fn test_clean_preserves_venv_by_default() {
        let temp_dir = TempDir::new().unwrap();
        let venv_dir = temp_dir.path().join(".venv");
        fs::create_dir(&venv_dir).unwrap();

        let result = execute(CleanArgs {
            path: temp_dir.path().to_path_buf(),
            all: false,
            dry_run: false,
        });
        assert!(result.is_ok());
        assert!(venv_dir.exists());
    }

    #[test]
    fn test_clean_all_removes_venv() {
        let temp_dir = TempDir::new().unwrap();
        let venv_dir = temp_dir.path().join(".venv");
        fs::create_dir(&venv_dir).unwrap();

        let result = execute(CleanArgs {
            path: temp_dir.path().to_path_buf(),
            all: true,
            dry_run: false,
        });
        assert!(result.is_ok());
        assert!(!venv_dir.exists());
    }
}
