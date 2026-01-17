//! Echidna CLI entry point.

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use echidna::chimerax::find_chimerax;
use echidna::commands::{
    build, clean, docs, info, init, install, publish, python, run, setup_ide, testing, validate,
    version, watch,
};
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

        /// Bundle type (command, tool, tool-html, format, fetch, selector, preset)
        #[arg(short = 't', long = "type", default_value = "command")]
        bundle_type: String,

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

        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
    },

    /// Show bundle information and status
    Info {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Run tests using ChimeraX Python environment
    Test {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Only run tests matching the given expression
        #[arg(short = 'k', long)]
        filter: Option<String>,

        /// Increase pytest verbosity
        #[arg(long)]
        verbose: bool,

        /// Skip build step
        #[arg(long)]
        no_build: bool,

        /// Skip install step
        #[arg(long)]
        no_install: bool,

        /// Generate coverage report
        #[arg(long)]
        coverage: bool,

        /// Additional arguments passed to pytest
        #[arg(last = true)]
        pytest_args: Vec<String>,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Open ChimeraX documentation
    Docs {
        /// Open developer documentation
        #[arg(long)]
        dev: bool,

        /// Open API reference
        #[arg(long)]
        api: bool,

        /// Search query
        #[arg(short, long)]
        search: Option<String>,
    },

    /// Publish bundle to ChimeraX Toolshed
    Publish {
        /// Project directory or wheel file
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Validate without publishing
        #[arg(long)]
        dry_run: bool,
    },

    /// Watch for changes and auto-rebuild
    Watch {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Also launch ChimeraX after build
        #[arg(long, conflicts_with = "test")]
        run: bool,

        /// Run tests on changes
        #[arg(long, conflicts_with = "run")]
        test: bool,
    },

    /// Manage bundle version in pyproject.toml
    Version {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Version action: show (default), patch, minor, major, or X.Y.Z
        #[arg(default_value = "show")]
        action: String,
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
            bundle_type,
            bundle_name,
            package,
            path,
            force,
        } => init::execute(init::InitArgs {
            name,
            bundle_type,
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

        Command::Validate { path, strict } => {
            validate::execute(validate::ValidateArgs { path, strict })
        }

        Command::Info { path } => info::execute(info::InfoArgs {
            path,
            chimerax: chimerax_path().ok(),
            verbosity,
        }),

        Command::Test {
            path,
            filter,
            verbose,
            no_build,
            no_install,
            coverage,
            pytest_args,
        } => testing::execute(testing::TestArgs {
            path,
            filter,
            verbose,
            no_build,
            no_install,
            coverage,
            pytest_args,
            chimerax: chimerax_path()?,
            verbosity,
        }),

        Command::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "echidna", &mut io::stdout());
            Ok(())
        }

        Command::Docs { dev, api, search } => docs::execute(docs::DocsArgs {
            dev,
            api,
            query: search,
        }),

        Command::Publish { path, dry_run } => {
            publish::execute(publish::PublishArgs { path, dry_run })
        }

        Command::Watch { path, run, test } => watch::execute(watch::WatchArgs {
            path,
            run,
            test,
            chimerax: chimerax_path()?,
            verbosity,
        }),

        Command::Version { path, action } => {
            let version_action = parse_version_action(&action)?;
            version::execute(version::VersionArgs {
                path,
                action: version_action,
            })
        }
    }
}

/// Parse version action string into VersionAction enum.
fn parse_version_action(action: &str) -> Result<version::VersionAction> {
    match action {
        "show" => Ok(version::VersionAction::Show),
        "patch" => Ok(version::VersionAction::BumpPatch),
        "minor" => Ok(version::VersionAction::BumpMinor),
        "major" => Ok(version::VersionAction::BumpMajor),
        _ => {
            // Check if it's a valid version string (X.Y.Z)
            if action.split('.').count() == 3
                && action.split('.').all(|part| part.parse::<u32>().is_ok())
            {
                Ok(version::VersionAction::Set(action.to_string()))
            } else {
                Err(EchidnaError::ConfigError(format!(
                    "Invalid version action '{}'. Use: show, patch, minor, major, or X.Y.Z",
                    action
                )))
            }
        }
    }
}
