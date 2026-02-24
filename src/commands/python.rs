//! `echidna python` command implementation.

use crate::chimerax::{ChimeraXExecutor, Verbosity};
use crate::error::Result;
use std::path::PathBuf;

/// Output format for python info.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

/// Arguments for the python command.
pub struct PythonArgs {
    pub format: OutputFormat,
    pub chimerax: PathBuf,
    pub verbosity: Verbosity,
}

/// Execute the python command.
pub fn execute(args: PythonArgs) -> Result<()> {
    let executor = ChimeraXExecutor::new(args.chimerax, args.verbosity);

    println!("Querying ChimeraX Python environment...");
    let info = executor.get_python_info()?;

    match args.format {
        OutputFormat::Text => {
            println!();
            println!("ChimeraX Python Environment");
            println!("============================");
            println!();
            println!("Executable: {}", info.executable);
            println!(
                "Version:    {}",
                info.version.lines().next().unwrap_or(&info.version)
            );
            println!("Prefix:     {}", info.prefix);

            if let Some(ref cx_version) = info.chimerax_version {
                println!("ChimeraX:   {}", cx_version);
            }

            if !info.site_packages.is_empty() {
                println!();
                println!("Site packages:");
                for sp in &info.site_packages {
                    println!("  {}", sp);
                }
            }

            println!();
            println!("Use this information to configure type checkers (ty, ruff):");
            println!("  pythonPath = \"{}\"", info.executable);
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&info)?;
            println!("{}", json);
        }
    }

    Ok(())
}
