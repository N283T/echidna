//! `echidna run` command implementation.

use crate::chimerax::{ChimeraXExecutor, Verbosity};
use crate::commands::{build, install};
use crate::error::Result;
use crate::ui;
use colored::Colorize;
use std::path::{Path, PathBuf};

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
        println!("{}", "=== Building ===".cyan().bold());
        build::execute_quiet(build::BuildArgs {
            path: project_dir.clone(),
            clean: false,
            chimerax: args.chimerax.clone(),
            verbosity: args.verbosity,
        })?;
        println!();
    }

    // Install if not skipped
    if !args.no_install {
        println!("{}", "=== Installing ===".cyan().bold());
        install::execute_quiet(install::InstallArgs {
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
        println!("{}", "=== Running (nogui) ===".cyan().bold());
        if let Some(script) = script {
            ui::hint(&format!("Script: {}", script.display()));
            executor.run_script(&script)?;
        } else {
            ui::hint("No script specified, running ChimeraX in nogui mode");
            executor.run_command("exit")?;
        }
    } else {
        // Launch GUI
        println!("{}", "=== Launching ChimeraX ===".cyan().bold());
        if let Some(ref s) = script {
            ui::hint(&format!("Script: {}", s.display()));
        }
        executor.launch(script.as_deref())?;
        ui::success("ChimeraX launched.");

        // Show command hint if available
        if let Some(cmd_name) = get_command_name(&project_dir) {
            println!();
            ui::hint(&format!(
                "Try running `{}` in ChimeraX command line",
                cmd_name
            ));
        }
    }

    Ok(())
}

/// Get the primary command name from pyproject.toml.
fn get_command_name(project_dir: &Path) -> Option<String> {
    let pyproject_path = project_dir.join("pyproject.toml");
    let content = std::fs::read_to_string(pyproject_path).ok()?;
    let pyproject: toml::Value = toml::from_str(&content).ok()?;

    // Try [chimerax.command.<name>] first
    if let Some(chimerax) = pyproject.get("chimerax")
        && let Some(command) = chimerax.get("command")
        && let Some(table) = command.as_table()
    {
        // Return first command name found
        return table.keys().next().map(|s| s.to_string());
    }

    None
}
