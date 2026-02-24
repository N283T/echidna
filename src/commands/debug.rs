//! `echidna debug` command implementation.

use crate::chimerax::{ChimeraXExecutor, Verbosity};
use crate::commands::{build, install};
use crate::error::Result;
use std::path::PathBuf;
use std::process::Command;

/// Arguments for the debug command.
pub struct DebugArgs {
    /// Project directory
    pub path: PathBuf,
    /// Skip build step
    pub no_build: bool,
    /// Skip install step
    pub no_install: bool,
    /// Path to ChimeraX executable
    pub chimerax: PathBuf,
    /// Verbosity level
    pub verbosity: Verbosity,
}

/// Execute the debug command.
pub fn execute(args: DebugArgs) -> Result<()> {
    let project_dir = args.path.canonicalize().unwrap_or(args.path.clone());
    let executor = ChimeraXExecutor::new(args.chimerax.clone(), args.verbosity);

    // Build if not skipped
    if !args.no_build {
        println!("=== Building ===");
        build::execute(build::BuildArgs {
            path: project_dir.clone(),
            clean: false,
            chimerax: executor.executable().to_path_buf(),
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
            chimerax: executor.executable().to_path_buf(),
            verbosity: args.verbosity,
        })?;
        println!();
    }

    println!("=== Launching ChimeraX in Debug Mode ===");
    println!();
    println!("Debug mode features:");
    println!("  - Verbose logging enabled");
    println!("  - Stack traces on errors");
    println!();

    let executable = executor.executable();
    let cmd_args = ["--debug"];

    println!("Running: {} {}", executable.display(), cmd_args.join(" "));
    println!();

    let status = Command::new(executable).args(cmd_args).status()?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        println!("ChimeraX exited with code: {}", code);
    }

    Ok(())
}
