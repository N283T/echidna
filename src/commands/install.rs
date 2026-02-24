//! `echidna install` command implementation.

use crate::chimerax::{ChimeraXExecutor, Verbosity};
use crate::error::{EchidnaError, Result};
use crate::ui;
use std::path::PathBuf;

/// Arguments for the install command.
pub struct InstallArgs {
    pub path: PathBuf,
    pub user: bool,
    pub chimerax: PathBuf,
    pub verbosity: Verbosity,
}

/// Execute the install command with full output.
pub fn execute(args: InstallArgs) -> Result<()> {
    execute_core(&args)?;

    ui::success("Installation successful!");
    ui::hint("The bundle is now available in ChimeraX.");
    ui::next_steps(&[
        ("echi run", "Launch ChimeraX with the bundle"),
        ("chimerax", "Open ChimeraX manually"),
    ]);

    Ok(())
}

/// Execute the install command without next steps (for use in run command).
pub fn execute_quiet(args: InstallArgs) -> Result<()> {
    execute_core(&args)?;

    ui::success("Installation successful!");

    Ok(())
}

/// Core install logic.
fn execute_core(args: &InstallArgs) -> Result<()> {
    let project_dir = args.path.canonicalize().unwrap_or(args.path.clone());

    // Verify this is a bundle directory
    let pyproject = project_dir.join("pyproject.toml");
    if !pyproject.exists() {
        return Err(EchidnaError::NotBundleDirectory(project_dir));
    }

    let executor = ChimeraXExecutor::new(args.chimerax.clone(), args.verbosity);

    println!("Installing bundle from {}...", project_dir.display());
    if args.user {
        println!("Installing as user bundle");
    }

    // Use devel install (handles permissions correctly on macOS /Applications)
    // devel install internally calls toolshed install with proper permission handling
    executor.devel_install(&project_dir, args.user)?;

    Ok(())
}
