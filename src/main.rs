//! Echidna CLI entry point.

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use echi::OutputFormat;
use echi::chimerax::ChimeraXValidator;
use echi::commands::{
    build, clean, debug, docs, info, init, install, packages, publish, python, run, setup_ide,
    testing, validate, version, watch, workspace,
};
use echi::config::{Config, GlobalConfig};
use echi::error::{EchidnaError, Result};
use echi::templates::BundleType;
use echi::workspace::Workspace;
use std::io;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "echi")]
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
        /// Project name or path (e.g., "my-tool" or "./projects/my-tool")
        /// Creates the directory if it doesn't exist.
        /// If omitted, initializes in the current directory.
        path: Option<PathBuf>,

        /// Override the project name (defaults to directory name)
        #[arg(short, long)]
        name: Option<String>,

        /// Bundle type (command, tool, tool-html, format, fetch, selector, preset, cpp)
        #[arg(short = 't', long = "type", value_enum, default_value = "command")]
        bundle_type: BundleType,

        /// Bundle name (e.g., "ChimeraX-MyTool")
        #[arg(long)]
        bundle_name: Option<String>,

        /// Python package name (e.g., "chimerax.mytool")
        #[arg(long)]
        package: Option<String>,

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

        /// Build all bundles in workspace
        #[arg(long)]
        all: bool,
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

    /// Manage ChimeraX Python packages
    #[command(subcommand)]
    Packages(PackagesCommand),

    /// Set up virtual environment with ChimeraX Python for IDE support
    Venv {
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

    /// Set up IDE/type checker environment (alias for 'venv')
    #[command(hide = true)]
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

        /// Increase pytest verbosity (adds -v flag to pytest)
        #[arg(long)]
        pytest_verbose: bool,

        /// Skip build step
        #[arg(long)]
        no_build: bool,

        /// Skip install step
        #[arg(long)]
        no_install: bool,

        /// Generate coverage report
        #[arg(long)]
        coverage: bool,

        /// Run smoke test (scripts/smoke.cxc) instead of pytest
        #[arg(long)]
        smoke: bool,

        /// Test all bundles in workspace
        #[arg(long)]
        all: bool,

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

    /// Launch ChimeraX in debug mode
    Debug {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Enable Python debugger (pdb) on exceptions
        #[arg(long)]
        pdb: bool,

        /// Enable profiling
        #[arg(long)]
        profile: bool,

        /// Skip build step
        #[arg(long)]
        no_build: bool,

        /// Skip install step
        #[arg(long)]
        no_install: bool,
    },

    /// Manage bundle workspaces (multiple bundles)
    #[command(subcommand)]
    Workspace(WorkspaceCommand),
}

/// Workspace subcommands.
#[derive(Subcommand)]
enum WorkspaceCommand {
    /// Initialize a workspace in the current directory
    Init {
        /// Directory to initialize as workspace
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Force overwrite existing workspace.toml
        #[arg(short, long)]
        force: bool,
    },

    /// List workspace members
    List {
        /// Directory to search for workspace
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

/// Packages subcommands.
#[derive(Subcommand)]
enum PackagesCommand {
    /// List installed packages in ChimeraX Python environment
    List {
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,

        /// Include stdlib packages (pip, setuptools, wheel)
        #[arg(long)]
        include_stdlib: bool,
    },

    /// Check for conflicts when adding a package
    Check {
        /// Package specification (e.g., "numpy", "requests>=2.28")
        package: Option<String>,

        /// Read packages from requirements file
        #[arg(short, long)]
        requirements: Option<PathBuf>,

        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },

    /// Install packages into ChimeraX Python environment
    Install {
        /// Packages to install (e.g., pytest, numpy>=1.20)
        #[arg(required = true)]
        packages: Vec<String>,

        /// Upgrade packages if already installed
        #[arg(short = 'U', long)]
        upgrade: bool,

        /// Show what would be installed without installing
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    if let Err(e) = run_cli() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run_cli() -> Result<()> {
    use std::cell::RefCell;

    let cli = Cli::parse();
    let verbosity = cli.verbose;

    // Load optional project config
    let project_config = Config::load_from_cwd()?;
    let config = project_config.clone().unwrap_or_default();

    // Load global config (wrapped in RefCell for interior mutability)
    let global_config = RefCell::new(GlobalConfig::load()?.unwrap_or_default());

    // Cache for validated ChimeraX path (to avoid multiple prompts)
    let validated_chimerax: RefCell<Option<PathBuf>> = RefCell::new(None);

    // Get ChimeraX path with interactive validation (cached after first call)
    // Priority: CLI > env var > project config > global config (validated) > auto-detect
    let chimerax_path = || -> Result<PathBuf> {
        // 1. CLI flag (already parsed from env var by clap)
        if let Some(ref path) = cli.chimerax {
            if !path.exists() {
                return Err(EchidnaError::ChimeraXCommandFailed(format!(
                    "ChimeraX not found at specified path: {}",
                    path.display()
                )));
            }
            return Ok(path.clone());
        }

        // 2. Project config (no interactive prompt for explicitly configured paths)
        if let Some(ref path) = config.chimerax_path {
            if !path.exists() {
                return Err(EchidnaError::ChimeraXCommandFailed(format!(
                    "ChimeraX not found at configured path: {}",
                    path.display()
                )));
            }
            return Ok(path.clone());
        }

        // 3. Use cached result if available
        {
            let cache = validated_chimerax.borrow();
            if let Some(ref path) = *cache {
                return Ok(path.clone());
            }
        }

        // 4. Interactive validation (may prompt user)
        let result = ChimeraXValidator::validate_and_prompt(
            &mut global_config.borrow_mut(),
            &project_config,
        )?
        .ok_or(EchidnaError::ChimeraXNotFound)?;

        // Cache the successful result
        *validated_chimerax.borrow_mut() = Some(result.clone());

        Ok(result)
    };

    match cli.command {
        Command::Init {
            path,
            name,
            bundle_type,
            bundle_name,
            package,
            force,
        } => init::execute(init::InitArgs {
            path,
            name,
            bundle_type,
            bundle_name,
            package,
            force,
        }),

        Command::Build { path, clean, all } => {
            if all {
                // Build all bundles in workspace
                let path = path.canonicalize().unwrap_or(path.clone());
                match Workspace::load_from_path(&path)? {
                    Some((root, ws)) => {
                        let members = ws.member_paths(&root);
                        if members.is_empty() {
                            return Err(EchidnaError::ConfigError(
                                "Workspace has no members".into(),
                            ));
                        }
                        println!("Building {} bundles in workspace...\n", members.len());
                        let chimerax = chimerax_path()?;
                        for member in members {
                            println!("=== {} ===", member.display());
                            build::execute(build::BuildArgs {
                                path: member,
                                clean,
                                chimerax: chimerax.clone(),
                                verbosity,
                            })?;
                            println!();
                        }
                        Ok(())
                    }
                    None => Err(EchidnaError::ConfigError(
                        "No workspace found. Use 'echi workspace init' to create one.".into(),
                    )),
                }
            } else {
                build::execute(build::BuildArgs {
                    path,
                    clean,
                    chimerax: chimerax_path()?,
                    verbosity,
                })
            }
        }

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
            format,
            chimerax: chimerax_path()?,
            verbosity,
        }),

        Command::Packages(cmd) => match cmd {
            PackagesCommand::List {
                format,
                include_stdlib,
            } => packages::list(packages::ListArgs {
                format,
                include_stdlib,
                chimerax: chimerax_path()?,
                verbosity,
            }),
            PackagesCommand::Check {
                package,
                requirements,
                format,
            } => packages::check(packages::CheckArgs {
                package,
                requirements,
                format,
                chimerax: chimerax_path()?,
                verbosity,
            }),
            PackagesCommand::Install {
                packages,
                upgrade,
                dry_run,
            } => packages::install(packages::InstallArgs {
                packages,
                upgrade,
                dry_run,
                chimerax: chimerax_path()?,
                verbosity,
            }),
        },

        Command::Venv {
            path,
            output,
            force,
            no_config,
            configs,
        }
        | Command::SetupIde {
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
            pytest_verbose,
            no_build,
            no_install,
            coverage,
            smoke,
            all,
            pytest_args,
        } => {
            if all {
                // Test all bundles in workspace
                let path = path.canonicalize().unwrap_or(path.clone());
                match Workspace::load_from_path(&path)? {
                    Some((root, ws)) => {
                        let members = ws.member_paths(&root);
                        if members.is_empty() {
                            return Err(EchidnaError::ConfigError(
                                "Workspace has no members".into(),
                            ));
                        }
                        println!("Testing {} bundles in workspace...\n", members.len());
                        let chimerax = chimerax_path()?;
                        let mut all_passed = true;
                        for member in &members {
                            println!("=== {} ===", member.display());
                            let result = testing::execute(testing::TestArgs {
                                path: member.clone(),
                                filter: filter.clone(),
                                pytest_verbose,
                                no_build,
                                no_install,
                                coverage,
                                smoke,
                                pytest_args: pytest_args.clone(),
                                chimerax: chimerax.clone(),
                                verbosity,
                            });
                            if result.is_err() {
                                all_passed = false;
                                eprintln!("Tests failed for {}", member.display());
                            }
                            println!();
                        }
                        if all_passed {
                            println!("All {} bundles passed tests.", members.len());
                            Ok(())
                        } else {
                            Err(EchidnaError::TestFailed(1))
                        }
                    }
                    None => Err(EchidnaError::ConfigError(
                        "No workspace found. Use 'echi workspace init' to create one.".into(),
                    )),
                }
            } else {
                testing::execute(testing::TestArgs {
                    path,
                    filter,
                    pytest_verbose,
                    no_build,
                    no_install,
                    coverage,
                    smoke,
                    pytest_args,
                    chimerax: chimerax_path()?,
                    verbosity,
                })
            }
        }

        Command::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "echi", &mut io::stdout());
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

        Command::Debug {
            path,
            pdb,
            profile,
            no_build,
            no_install,
        } => debug::execute(debug::DebugArgs {
            path,
            pdb,
            profile,
            no_build,
            no_install,
            chimerax: chimerax_path()?,
            verbosity,
        }),

        Command::Workspace(cmd) => match cmd {
            WorkspaceCommand::Init { path, force } => {
                workspace::init(workspace::WorkspaceInitArgs { path, force })
            }
            WorkspaceCommand::List { path } => {
                workspace::list(workspace::WorkspaceListArgs { path })
            }
        },
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
