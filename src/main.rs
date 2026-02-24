//! Echidna CLI entry point.

mod cli;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Command, PackagesCommand, WorkspaceCommand};
use echi::chimerax::ChimeraXValidator;
use echi::commands::{
    build, clean, debug, docs, info, init, install, packages, publish, python, run, setup_ide,
    testing, validate, version, watch, workspace,
};
use echi::config::{Config, GlobalConfig};
use echi::error::{EchidnaError, Result};
use std::io;
use std::path::{Path, PathBuf};

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
                build_all(&path, clean, &chimerax_path, verbosity)
            } else {
                build::execute(build::BuildArgs {
                    path,
                    clean,
                    chimerax: chimerax_path()?,
                    verbosity,
                })
            }
        }

        Command::Install { path, user } => install::execute(install::InstallArgs {
            path,
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
                test_all(
                    &path,
                    filter,
                    pytest_verbose,
                    no_build,
                    no_install,
                    coverage,
                    smoke,
                    pytest_args,
                    &chimerax_path,
                    verbosity,
                )
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
            no_build,
            no_install,
        } => debug::execute(debug::DebugArgs {
            path,
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

/// Build all bundles in a workspace.
fn build_all(
    path: &Path,
    clean: bool,
    chimerax_path: &dyn Fn() -> Result<PathBuf>,
    verbosity: u8,
) -> Result<()> {
    use echi::workspace::Workspace;

    let path = path.canonicalize().unwrap_or(path.to_path_buf());
    match Workspace::load_from_path(&path)? {
        Some((root, ws)) => {
            let members = ws.member_paths(&root);
            if members.is_empty() {
                return Err(EchidnaError::ConfigError("Workspace has no members".into()));
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
}

/// Test all bundles in a workspace.
#[allow(clippy::too_many_arguments)]
fn test_all(
    path: &Path,
    filter: Option<String>,
    pytest_verbose: bool,
    no_build: bool,
    no_install: bool,
    coverage: bool,
    smoke: bool,
    pytest_args: Vec<String>,
    chimerax_path: &dyn Fn() -> Result<PathBuf>,
    verbosity: u8,
) -> Result<()> {
    use echi::workspace::Workspace;

    let path = path.canonicalize().unwrap_or(path.to_path_buf());
    match Workspace::load_from_path(&path)? {
        Some((root, ws)) => {
            let members = ws.member_paths(&root);
            if members.is_empty() {
                return Err(EchidnaError::ConfigError("Workspace has no members".into()));
            }
            println!("Testing {} bundles in workspace...\n", members.len());
            let chimerax = chimerax_path()?;
            let mut failed: Vec<String> = Vec::new();
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
                if let Err(e) = result {
                    eprintln!("Tests failed for {}: {}", member.display(), e);
                    failed.push(member.display().to_string());
                }
                println!();
            }
            if failed.is_empty() {
                println!("All {} bundles passed tests.", members.len());
                Ok(())
            } else {
                Err(EchidnaError::TestFailed(failed.len() as i32))
            }
        }
        None => Err(EchidnaError::ConfigError(
            "No workspace found. Use 'echi workspace init' to create one.".into(),
        )),
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
