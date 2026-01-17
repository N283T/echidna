//! `echidna watch` command implementation.

use crate::chimerax::Verbosity;
use crate::commands::{build, install, run, testing};
use crate::error::{EchidnaError, Result};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

/// Arguments for the watch command.
pub struct WatchArgs {
    /// Project directory
    pub path: PathBuf,
    /// Also launch ChimeraX after build
    pub run: bool,
    /// Run tests on changes
    pub test: bool,
    /// Path to ChimeraX executable
    pub chimerax: PathBuf,
    /// Verbosity level
    pub verbosity: Verbosity,
}

/// Directories and patterns to watch.
const WATCH_PATTERNS: &[&str] = &["src", "tests", "pyproject.toml"];

/// Directory names to ignore (matched by path component).
const IGNORE_DIRS: &[&str] = &["dist", "build", ".venv", "__pycache__", ".git", "htmlcov"];

/// File extensions to watch.
const WATCH_EXTENSIONS: &[&str] = &["py", "pyi", "toml", "cxc"];

/// Minimum time between rebuilds (debounce).
const DEBOUNCE_DURATION: Duration = Duration::from_millis(500);

/// Execute the watch command.
pub fn execute(args: WatchArgs) -> Result<()> {
    // Canonicalize and validate project directory
    let project_dir = args.path.canonicalize().map_err(|e| {
        EchidnaError::ConfigError(format!(
            "Cannot access project directory '{}': {}",
            args.path.display(),
            e
        ))
    })?;

    // Validate it's a bundle directory
    let pyproject = project_dir.join("pyproject.toml");
    if !pyproject.exists() {
        return Err(EchidnaError::NotBundleDirectory(project_dir));
    }

    println!("Watching for changes in: {}", project_dir.display());
    println!("Press Ctrl+C to stop");
    println!();

    if args.run {
        println!("Mode: build + install + run ChimeraX");
    } else if args.test {
        println!("Mode: build + install + test");
    } else {
        println!("Mode: build + install");
    }
    println!();

    // Initial build
    println!("=== Initial Build ===");
    if let Err(e) = do_build(&args, &project_dir) {
        eprintln!("Initial build failed: {}", e);
    }

    // Set up file watcher
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Watch relevant directories
    for pattern in WATCH_PATTERNS {
        let watch_path = project_dir.join(pattern);
        if watch_path.exists() {
            let mode = if watch_path.is_dir() {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            if let Err(e) = watcher.watch(&watch_path, mode) {
                eprintln!("Warning: Failed to watch {}: {}", watch_path.display(), e);
            } else {
                println!("Watching: {}", watch_path.display());
            }
        }
    }

    println!();
    println!("Waiting for changes...");

    let mut last_build = Instant::now();

    loop {
        match rx.recv() {
            Ok(event) => {
                if let Ok(event) = event {
                    // Check if this is a relevant change
                    if !is_relevant_change(&event, &project_dir) {
                        continue;
                    }

                    // Debounce: ignore events too close together
                    let now = Instant::now();
                    if now.duration_since(last_build) < DEBOUNCE_DURATION {
                        continue;
                    }

                    println!();
                    println!("=== Change Detected ===");
                    for path in &event.paths {
                        if let Ok(relative) = path.strip_prefix(&project_dir) {
                            println!("  Changed: {}", relative.display());
                        }
                    }

                    // Rebuild
                    if let Err(e) = do_build(&args, &project_dir) {
                        eprintln!("Build failed: {}", e);
                    }

                    // Update debounce timer AFTER build completes
                    last_build = Instant::now();

                    println!();
                    println!("Waiting for changes...");
                }
            }
            Err(e) => {
                eprintln!("Watch error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Check if the event is a relevant change.
fn is_relevant_change(event: &notify::Event, project_dir: &Path) -> bool {
    use notify::EventKind;

    // Only care about modifications and creations
    match event.kind {
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {}
        _ => return false,
    }

    // Check paths
    for path in &event.paths {
        // Skip ignored directories (using component-based matching)
        if should_ignore_path(path) {
            continue;
        }

        // Check if path is within watched directories
        if path.strip_prefix(project_dir).is_ok() {
            // Check file extension
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if WATCH_EXTENSIONS.contains(&ext) {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if a path should be ignored based on directory components.
fn should_ignore_path(path: &Path) -> bool {
    path.components().any(|component| {
        if let std::path::Component::Normal(name) = component {
            // Check exact directory name match
            if IGNORE_DIRS.iter().any(|dir| name == OsStr::new(dir)) {
                return true;
            }
            // Also ignore .egg-info directories (suffix match)
            if let Some(name_str) = name.to_str() {
                if name_str.ends_with(".egg-info") {
                    return true;
                }
            }
        }
        false
    })
}

/// Perform the build action.
fn do_build(args: &WatchArgs, project_dir: &Path) -> Result<()> {
    // Build
    build::execute(build::BuildArgs {
        path: project_dir.to_path_buf(),
        clean: false,
        chimerax: args.chimerax.clone(),
        verbosity: args.verbosity,
    })?;

    // Install
    install::execute(install::InstallArgs {
        path: project_dir.to_path_buf(),
        wheel: None,
        user: false,
        chimerax: args.chimerax.clone(),
        verbosity: args.verbosity,
    })?;

    if args.run {
        // Run ChimeraX
        println!();
        println!("=== Launching ChimeraX ===");
        run::execute(run::RunArgs {
            path: project_dir.to_path_buf(),
            script: None,
            no_build: true,   // Already built
            no_install: true, // Already installed
            nogui: false,
            chimerax: args.chimerax.clone(),
            verbosity: args.verbosity,
        })?;
    } else if args.test {
        // Run tests
        println!();
        println!("=== Running Tests ===");
        let test_result = testing::execute(testing::TestArgs {
            path: project_dir.to_path_buf(),
            filter: None,
            verbose: false,
            no_build: true,   // Already built
            no_install: true, // Already installed
            coverage: false,
            pytest_args: vec![],
            chimerax: args.chimerax.clone(),
            verbosity: args.verbosity,
        });

        // Don't fail the watch loop on test failures
        if let Err(e) = test_result {
            eprintln!("Tests failed: {}", e);
        }
    }

    println!();
    println!("Build complete!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_ignore_path_dist() {
        assert!(should_ignore_path(Path::new("project/dist/wheel.whl")));
        assert!(should_ignore_path(Path::new("/abs/path/dist/file.py")));
    }

    #[test]
    fn test_should_ignore_path_pycache() {
        assert!(should_ignore_path(Path::new("src/__pycache__/module.pyc")));
    }

    #[test]
    fn test_should_ignore_path_egg_info() {
        assert!(should_ignore_path(Path::new(
            "src/mypackage.egg-info/PKG-INFO"
        )));
    }

    #[test]
    fn test_should_not_ignore_normal_paths() {
        assert!(!should_ignore_path(Path::new("src/module.py")));
        assert!(!should_ignore_path(Path::new("tests/test_module.py")));
        assert!(!should_ignore_path(Path::new("pyproject.toml")));
    }

    #[test]
    fn test_should_not_ignore_substring_matches() {
        // "dist" as substring should NOT be ignored
        assert!(!should_ignore_path(Path::new("src/redistribution.py")));
        // "build" as substring should NOT be ignored
        assert!(!should_ignore_path(Path::new("src/rebuild_utils.py")));
    }
}
