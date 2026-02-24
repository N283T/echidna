//! `echidna build` command implementation.

use crate::chimerax::{ChimeraXExecutor, Verbosity};
use crate::error::{EchidnaError, Result};
use crate::ui;
use std::path::{Path, PathBuf};

/// Arguments for the build command.
pub struct BuildArgs {
    pub path: PathBuf,
    pub clean: bool,
    pub chimerax: PathBuf,
    pub verbosity: Verbosity,
}

/// Execute the build command with full output.
pub fn execute(args: BuildArgs) -> Result<()> {
    let wheel = execute_core(&args)?;

    ui::success("Build successful!");
    ui::hint(&format!("Wheel: {}", wheel.display()));
    ui::next_steps(&[
        ("echidna install", "Install the bundle to ChimeraX"),
        ("echidna run", "Build, install, and launch ChimeraX"),
    ]);

    Ok(())
}

/// Execute the build command without next steps (for use in run command).
pub fn execute_quiet(args: BuildArgs) -> Result<()> {
    let wheel = execute_core(&args)?;

    ui::success("Build successful!");
    ui::hint(&format!("Wheel: {}", wheel.display()));

    Ok(())
}

/// Core build logic, returns the wheel path.
fn execute_core(args: &BuildArgs) -> Result<PathBuf> {
    let project_dir = args.path.canonicalize().unwrap_or(args.path.clone());

    // Verify this is a bundle directory
    let pyproject = project_dir.join("pyproject.toml");
    if !pyproject.exists() {
        return Err(EchidnaError::NotBundleDirectory(project_dir));
    }

    // Clean if requested
    if args.clean {
        let build_dir = project_dir.join("build");
        let dist_dir = project_dir.join("dist");
        if build_dir.exists() {
            std::fs::remove_dir_all(&build_dir)?;
            println!("Removed build/");
        }
        if dist_dir.exists() {
            std::fs::remove_dir_all(&dist_dir)?;
            println!("Removed dist/");
        }
    }

    println!("Building bundle in {}...", project_dir.display());

    // Execute devel build
    let executor = ChimeraXExecutor::new(args.chimerax.clone(), args.verbosity);
    executor.devel_build(&project_dir)?;

    // Find the generated wheel
    let dist_dir = project_dir.join("dist");
    find_newest_wheel(&dist_dir)
}

/// Find the newest wheel file in a directory.
pub fn find_newest_wheel(dist_dir: &Path) -> Result<PathBuf> {
    if !dist_dir.exists() {
        return Err(EchidnaError::NoWheelFound);
    }

    let mut wheels: Vec<_> = std::fs::read_dir(dist_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "whl").unwrap_or(false))
        .collect();

    if wheels.is_empty() {
        return Err(EchidnaError::NoWheelFound);
    }

    // Sort by modification time (newest first)
    wheels.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    // Use into_iter().next() instead of remove(0) for safety
    wheels.into_iter().next().ok_or(EchidnaError::NoWheelFound)
}
