//! `echidna watch` command implementation.

use crate::chimerax::{ChimeraXExecutor, Verbosity};
use crate::commands::{build, install, run, testing};
use crate::error::Result;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
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

/// Directories to ignore.
const IGNORE_PATTERNS: &[&str] = &[
    "dist",
    "build",
    ".venv",
    "__pycache__",
    ".egg-info",
    ".git",
    "htmlcov",
];

/// Minimum time between rebuilds (debounce).
const DEBOUNCE_DURATION: Duration = Duration::from_millis(500);

/// Execute the watch command.
pub fn execute(args: WatchArgs) -> Result<()> {
    let project_dir = args.path.canonicalize().unwrap_or(args.path.clone());

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
                    last_build = now;

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
fn is_relevant_change(event: &notify::Event, project_dir: &PathBuf) -> bool {
    use notify::EventKind;

    // Only care about modifications and creations
    match event.kind {
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {}
        _ => return false,
    }

    // Check paths
    for path in &event.paths {
        // Skip ignored directories
        let path_str = path.to_string_lossy();
        let should_ignore = IGNORE_PATTERNS
            .iter()
            .any(|pattern| path_str.contains(pattern));

        if should_ignore {
            continue;
        }

        // Check if path is within watched directories
        if let Ok(relative) = path.strip_prefix(project_dir) {
            let relative_str = relative.to_string_lossy();

            // Check if it's a Python file or pyproject.toml
            if relative_str.ends_with(".py")
                || relative_str.ends_with(".toml")
                || relative_str.ends_with(".cxc")
            {
                return true;
            }
        }
    }

    false
}

/// Perform the build action.
fn do_build(args: &WatchArgs, project_dir: &Path) -> Result<()> {
    let executor = ChimeraXExecutor::new(args.chimerax.clone(), args.verbosity);

    // Build
    build::execute(build::BuildArgs {
        path: project_dir.to_path_buf(),
        clean: false,
        chimerax: executor.executable().to_path_buf(),
        verbosity: args.verbosity,
    })?;

    // Install
    install::execute(install::InstallArgs {
        path: project_dir.to_path_buf(),
        wheel: None,
        user: false,
        chimerax: executor.executable().to_path_buf(),
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
            chimerax: executor.executable().to_path_buf(),
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
            chimerax: executor.executable().to_path_buf(),
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
