//! CLI argument definitions.

use clap::{Parser, Subcommand};
use clap_complete::Shell;
use echi::OutputFormat;
use echi::templates::BundleType;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "echi")]
#[command(about = "ChimeraX Bundle Development CLI")]
#[command(version)]
#[command(author)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Path to ChimeraX executable (overrides auto-detection)
    #[arg(long, global = true, env = "CHIMERAX_PATH")]
    pub chimerax: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Command {
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
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,

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
pub enum WorkspaceCommand {
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
pub enum PackagesCommand {
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
