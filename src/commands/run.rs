//! `echidna run` command implementation.

use crate::chimerax::{ChimeraXExecutor, Verbosity};
use crate::commands::{build, install};
use crate::error::Result;
use std::path::PathBuf;

/// Arguments for the run command.
pub struct RunArgs {
    pub path: PathBuf,
    pub script: Option<PathBuf>,
    pub no_build: bool,
    pub no_install: bool,
    pub nogui: bool,
    pub chimerax: PathBuf,
    pub verbosity: Verbosity,
}

/// Execute the run command.
pub fn execute(args: RunArgs) -> Result<()> {
    let project_dir = args.path.canonicalize().unwrap_or(args.path.clone());

    // Build if not skipped
    if !args.no_build {
        println!("=== Building ===");
        build::execute(build::BuildArgs {
            path: project_dir.clone(),
            clean: false,
            chimerax: args.chimerax.clone(),
            verbosity: args.verbosity,
        })?;
        println!();
    }

    // Install if not skipped
    if !args.no_install {
        println!("=== Installing ===");
        install::execute(install::InstallArgs {
            path: project_dir.clone(),
            wheel: None,
            user: false,
            chimerax: args.chimerax.clone(),
            verbosity: args.verbosity,
        })?;
        println!();
    }

    // Determine script to run
    let script = args.script.or_else(|| {
        // Try default script location
        let default_script = project_dir.join("scripts/smoke.cxc");
        if default_script.exists() {
            Some(default_script)
        } else {
            None
        }
    });

    let executor = ChimeraXExecutor::new(args.chimerax, args.verbosity);

    if args.nogui {
        // Run in nogui mode
        println!("=== Running (nogui) ===");
        if let Some(script) = script {
            println!("Script: {}", script.display());
            executor.run_script(&script)?;
        } else {
            println!("No script specified, running ChimeraX in nogui mode");
            executor.run_command("exit")?;
        }
    } else {
        // Launch GUI
        println!("=== Launching ChimeraX ===");
        if let Some(ref s) = script {
            println!("Script: {}", s.display());
        }
        executor.launch(script.as_deref())?;
        println!("ChimeraX launched.");
    }

    Ok(())
}
