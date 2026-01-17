//! `echidna workspace` command implementation.

use crate::error::{EchidnaError, Result};
use crate::workspace::{Workspace, WORKSPACE_FILE};
use std::path::PathBuf;

/// Arguments for the workspace init command.
pub struct WorkspaceInitArgs {
    /// Directory to initialize as workspace.
    pub path: PathBuf,
    /// Force overwrite existing workspace.toml.
    pub force: bool,
}

/// Execute the workspace init command.
pub fn init(args: WorkspaceInitArgs) -> Result<()> {
    let workspace_dir = args.path.canonicalize().unwrap_or(args.path.clone());
    let workspace_file = workspace_dir.join(WORKSPACE_FILE);

    // Check if workspace.toml already exists
    if workspace_file.exists() && !args.force {
        return Err(EchidnaError::ConfigError(format!(
            "Workspace already exists at '{}'. Use --force to overwrite.",
            workspace_file.display()
        )));
    }

    // Discover bundles in the directory
    let members = Workspace::discover_members(&workspace_dir)?;

    if members.is_empty() {
        println!(
            "No bundle directories found in '{}'.",
            workspace_dir.display()
        );
        println!("Creating empty workspace. Add members manually to workspace.toml.");
    } else {
        println!(
            "Found {} bundle{}:",
            members.len(),
            if members.len() == 1 { "" } else { "s" }
        );
        for member in &members {
            println!("  - {}", member);
        }
    }

    // Create workspace
    let workspace = Workspace::new(members);
    workspace.save(&workspace_file)?;

    println!();
    println!("Created {}", workspace_file.display());
    println!();
    println!("You can now use:");
    println!("  echidna build --all    Build all bundles");
    println!("  echidna test --all     Test all bundles");

    Ok(())
}

/// Arguments for the workspace list command.
pub struct WorkspaceListArgs {
    /// Directory to search for workspace.
    pub path: PathBuf,
}

/// Execute the workspace list command.
pub fn list(args: WorkspaceListArgs) -> Result<()> {
    let path = args.path.canonicalize().unwrap_or(args.path.clone());

    match Workspace::load_from_path(&path)? {
        Some((root, workspace)) => {
            println!("Workspace: {}", root.display());
            println!();
            println!("Members ({}):", workspace.workspace.members.len());
            for member in &workspace.workspace.members {
                let member_path = root.join(member);
                let status = if member_path.join("pyproject.toml").exists() {
                    ""
                } else {
                    " (not found)"
                };
                println!("  - {}{}", member, status);
            }
        }
        None => {
            println!("No workspace found.");
            println!();
            println!("To create a workspace, run:");
            println!("  echidna workspace init");
        }
    }

    Ok(())
}
