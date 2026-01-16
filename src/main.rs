//! Echidna CLI entry point.

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use echidna::chimerax::find_chimerax;
use echidna::commands::{build, clean, info, init, install, python, run, setup_ide, validate};
use echidna::config::Config;
use echidna::error::{EchidnaError, Result};
use std::io;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "echidna")]
#[command(about = "ChimeraX Bundle Development CLI")]
#[command(version)]
#[command(author)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Path to ChimeraX executable (overrides auto-detection)
    #[arg(long, global = true, env = "CHIMERAX_PATH")]
    chimerax: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a new ChimeraX bundle project
    Init {
        /// Project name (e.g., "my-tool")
        #[arg(short, long)]
        name: Option<String>,

        /// Bundle name (e.g., "ChimeraX-MyTool")
        #[arg(long)]
        bundle_name: Option<String>,

        /// Python package name (e.g., "chimerax.mytool")
        #[arg(long)]
        package: Option<String>,

        /// Directory to create project in (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Overwrite existing files
        #[arg(short, long)]
        force: bool,
    },

    /// Build the bundle wheel
    Build {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Clean build directory before building
        #[arg(long)]
        clean: bool,
    },

    /// Install the bundle to ChimeraX
    Install {
        /// Project directory or wheel file
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Specific wheel file to install
        #[arg(short, long)]
        wheel: Option<PathBuf>,

        /// Install as user bundle
        #[arg(long)]
        user: bool,
    },

    /// Build, install, and launch ChimeraX
    Run {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Script to execute after launch (.cxc file)
        #[arg(short, long)]
        script: Option<PathBuf>,

        /// Skip build step
        #[arg(long)]
        no_build: bool,

        /// Skip install step
        #[arg(long)]
        no_install: bool,

        /// Run in nogui mode
        #[arg(long)]
        nogui: bool,
    },

    /// Show ChimeraX Python environment info
    Python {
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },

    /// Set up IDE/type checker environment
    SetupIde {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output directory for venv
        #[arg(short, long, default_value = ".venv")]
        output: PathBuf,

        /// Force overwrite existing venv
        #[arg(short, long)]
        force: bool,

        /// Skip generating type checker config files
        #[arg(long)]
        no_config: bool,

        /// Config files to generate (comma-separated: ty,ruff)
        #[arg(long, value_delimiter = ',')]
        configs: Vec<String>,
    },

    /// Clean build artifacts
    Clean {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Also remove .venv directory
        #[arg(long)]
        all: bool,

        /// Show what would be deleted without actually deleting
        #[arg(long)]
        dry_run: bool,
    },

    /// Validate bundle structure and configuration
    Validate {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Show bundle information and status
    Info {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

impl From<OutputFormat> for python::OutputFormat {
    fn from(f: OutputFormat) -> Self {
        match f {
            OutputFormat::Text => python::OutputFormat::Text,
            OutputFormat::Json => python::OutputFormat::Json,
        }
    }
}

fn main() {
    if let Err(e) = run_cli() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    let verbosity = cli.verbose;

    // Load optional config
    let config = Config::load_from_cwd()?.unwrap_or_default();

    // Determine ChimeraX path (CLI > config > auto-detect)
    let chimerax_path = || -> Result<PathBuf> {
        let path = if let Some(ref path) = cli.chimerax {
            path.clone()
        } else if let Some(ref path) = config.chimerax_path {
            path.clone()
        } else {
            return find_chimerax().ok_or(EchidnaError::ChimeraXNotFound);
        };

        // Validate the specified path exists
        if !path.exists() {
            return Err(EchidnaError::ChimeraXCommandFailed(format!(
                "ChimeraX not found at specified path: {}",
                path.display()
            )));
        }

        Ok(path)
    };

    match cli.command {
        Command::Init {
            name,
            bundle_name,
            package,
            path,
            force,
        } => init::execute(init::InitArgs {
            name,
            bundle_name,
            package,
            path,
            force,
        }),

        Command::Build { path, clean } => build::execute(build::BuildArgs {
            path,
            clean,
            chimerax: chimerax_path()?,
            verbosity,
        }),

        Command::Install { path, wheel, user } => install::execute(install::InstallArgs {
            path,
            wheel,
            user: user || config.user_install,
            chimerax: chimerax_path()?,
            verbosity,
        }),

        Command::Run {
            path,
            script,
            no_build,
            no_install,
            nogui,
        } => run::execute(run::RunArgs {
            path,
            script: script.or(config.default_script),
            no_build,
            no_install,
            nogui,
            chimerax: chimerax_path()?,
            verbosity,
        }),

        Command::Python { format } => python::execute(python::PythonArgs {
            format: format.into(),
            chimerax: chimerax_path()?,
            verbosity,
        }),

        Command::SetupIde {
            path,
            output,
            force,
            no_config,
            configs,
        } => setup_ide::execute(setup_ide::SetupIdeArgs {
            path,
            output,
            force,
            no_config,
            configs,
            chimerax: chimerax_path()?,
            verbosity,
        }),

        Command::Clean { path, all, dry_run } => {
            clean::execute(clean::CleanArgs { path, all, dry_run })
        }

        Command::Validate { path } => validate::execute(validate::ValidateArgs { path }),

        Command::Info { path } => info::execute(info::InfoArgs {
            path,
            chimerax: chimerax_path().ok(),
            verbosity,
        }),

        Command::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "echidna", &mut io::stdout());
            Ok(())
        }
    }
}
