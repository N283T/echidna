//! `echidna setup-ide` command implementation.

use crate::chimerax::{ChimeraXExecutor, Verbosity};
use crate::error::Result;
use crate::venv::{ConfigGenerator, ConfigType, VenvBuilder};
use std::collections::HashSet;
use std::path::PathBuf;

/// Arguments for the setup-ide command.
pub struct SetupIdeArgs {
    /// Project directory
    pub path: PathBuf,
    /// Output directory for venv (default: .venv)
    pub output: PathBuf,
    /// Force overwrite existing venv
    pub force: bool,
    /// Skip config file generation
    pub no_config: bool,
    /// Config types to generate (empty = defaults)
    pub configs: Vec<String>,
    /// Path to ChimeraX executable
    pub chimerax: PathBuf,
    /// Verbosity level
    pub verbosity: Verbosity,
}

/// Execute the setup-ide command.
pub fn execute(args: SetupIdeArgs) -> Result<()> {
    let project_root = args.path.canonicalize().unwrap_or(args.path.clone());

    // Determine venv output path
    let venv_path = if args.output.is_absolute() {
        args.output.clone()
    } else {
        project_root.join(&args.output)
    };

    println!("Setting up IDE environment...");
    println!();

    // Get Python info from ChimeraX
    let executor = ChimeraXExecutor::new(args.chimerax, args.verbosity);
    println!("Querying ChimeraX Python environment...");
    let python_info = executor.get_python_info()?;

    // Create venv
    println!("Creating venv at {}...", venv_path.display());
    let builder = VenvBuilder::new(venv_path.clone(), python_info.clone()).force(args.force);
    builder.build()?;
    println!("  Created pyvenv.cfg");
    println!("  Created Python symlinks");

    // Generate config files
    if !args.no_config {
        let config_types = if args.configs.is_empty() {
            ConfigType::defaults()
        } else {
            let mut set = HashSet::new();
            for name in &args.configs {
                if let Some(ct) = ConfigType::parse(name) {
                    set.insert(ct);
                } else {
                    eprintln!(
                        "Warning: Unknown config type '{}', skipping. Valid types: ty, ruff, pyright, vscode",
                        name
                    );
                }
            }
            set
        };

        if !config_types.is_empty() {
            println!();
            println!("Generating type checker configurations...");

            let generator = ConfigGenerator::new(&python_info, &venv_path, &project_root);
            let generated = generator.generate(&config_types)?;

            for file in &generated {
                println!("  Created {}", file);
            }
        }
    }

    // Print summary
    println!();
    println!("IDE setup complete!");
    println!();
    println!("Python executable: {}", python_info.executable);
    println!(
        "Python version:    {}",
        python_info
            .version
            .lines()
            .next()
            .unwrap_or(&python_info.version)
    );
    if let Some(ref cx_version) = python_info.chimerax_version {
        println!("ChimeraX version:  {}", cx_version);
    }
    println!("Venv path:         {}", venv_path.display());

    // Usage hints
    println!();
    println!("Next steps:");
    println!(
        "  1. Select '{}' as your Python interpreter in your IDE",
        venv_path.display()
    );
    println!("  2. Your IDE should now recognize 'chimerax' imports");
    println!();
    println!("Type checking:");
    println!("  ty check         # Using ty");
    println!("  ruff check .     # Using ruff");

    Ok(())
}
