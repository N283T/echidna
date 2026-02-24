//! `echidna install` command implementation.

use crate::chimerax::{ChimeraXExecutor, Verbosity};
use crate::commands::build::find_newest_wheel;
use crate::error::{EchidnaError, Result};
use crate::ui;
use std::path::PathBuf;

/// Arguments for the install command.
pub struct InstallArgs {
    pub path: PathBuf,
    pub wheel: Option<PathBuf>,
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
        ("echidna run", "Launch ChimeraX with the bundle"),
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
    let executor = ChimeraXExecutor::new(args.chimerax.clone(), args.verbosity);

    // Determine the wheel to install
    let wheel = match &args.wheel {
        Some(w) => {
            if !w.exists() {
                return Err(EchidnaError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Wheel not found: {}", w.display()),
                )));
            }
            w.clone()
        }
        None => {
            let project_dir = args.path.canonicalize().unwrap_or(args.path.clone());
            let dist_dir = project_dir.join("dist");
            find_newest_wheel(&dist_dir)?
        }
    };

    println!("Installing {}...", wheel.display());
    if args.user {
        println!("Installing as user bundle");
    }

    // Use toolshed install
    executor.toolshed_install(&wheel, args.user)?;

    Ok(())
}
