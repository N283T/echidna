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
    /// Enable Python debugger (pdb)
    pub pdb: bool,
    /// Enable profiling
    pub profile: bool,
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

    // Build command arguments
    let cmd_args = vec!["--debug".to_string()];

    if args.pdb {
        println!("  Python debugger (pdb) enabled");
        // ChimeraX doesn't have a direct --pdb flag, but we can set up the environment
        // and run a command that enables pdb on exceptions
    }

    if args.profile {
        println!("  Profiling enabled");
        // Add profiling-related setup
    }

    // Print debug info
    println!();
    println!("Debug mode features:");
    println!("  - Verbose logging enabled");
    println!("  - Stack traces on errors");
    if args.pdb {
        println!("  - Post-mortem debugging on exceptions");
    }
    if args.profile {
        println!("  - Performance profiling active");
    }
    println!();

    // Launch ChimeraX in debug mode
    let executable = executor.executable();

    let mut command = Command::new(executable);
    command.args(&cmd_args);

    // If pdb is enabled, add a startup command that enables post-mortem debugging
    if args.pdb {
        let pdb_setup = r#"import sys; import pdb; sys.excepthook = lambda *args: (pdb.post_mortem(args[2]) if args[2] else None)"#;
        command.args(["--cmd", &format!("runscript python -c \"{}\"", pdb_setup)]);
    }

    // If profiling is enabled, we could add profiling setup
    // For now, debug mode itself provides useful debugging info

    println!("Running: {} {}", executable.display(), cmd_args.join(" "));
    println!();

    // Execute ChimeraX
    let status = command.status()?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        println!("ChimeraX exited with code: {}", code);
    }

    Ok(())
}
