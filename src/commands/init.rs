//! `echidna init` command implementation.

use crate::error::{EchidnaError, Result};
use crate::templates::{BundleTemplate, BundleType};
use crate::ui;
use std::path::{Path, PathBuf};

/// Arguments for the init command.
pub struct InitArgs {
    pub path: Option<PathBuf>,
    pub name: Option<String>,
    pub bundle_type: BundleType,
    pub bundle_name: Option<String>,
    pub package: Option<String>,
    pub force: bool,
}

/// Resolve the target directory and project name from arguments.
///
/// Behavior:
/// - `echidna init` or `echidna init .` -> current dir, name from dir
/// - `echidna init hello-world` -> ./hello-world/, name = "hello-world"
/// - `echidna init ./path/to/project` -> that path, name from last component
/// - `echidna init hello-world --name custom` -> ./hello-world/, name = "custom"
fn resolve_path_and_name(
    path_arg: Option<&PathBuf>,
    name_arg: Option<&String>,
) -> Result<(PathBuf, String)> {
    let target_dir = match path_arg {
        None => std::env::current_dir()?,
        Some(p) if p.as_os_str() == "." => std::env::current_dir()?,
        Some(p) => {
            // If path doesn't exist and looks like a simple name (no path separators),
            // treat it as a project name and create ./name/
            let path_str = p.to_string_lossy();
            if !p.exists()
                && !path_str.contains(std::path::MAIN_SEPARATOR)
                && !path_str.contains('/')
            {
                std::env::current_dir()?.join(p)
            } else {
                // Resolve relative paths
                if p.is_relative() {
                    std::env::current_dir()?.join(p)
                } else {
                    p.clone()
                }
            }
        }
    };

    let name = match name_arg {
        Some(n) => n.clone(),
        None => target_dir
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                EchidnaError::InvalidName(
                    "Could not determine project name from path. Use --name to specify.".into(),
                )
            })?,
    };

    Ok((target_dir, name))
}

/// Execute the init command.
pub fn execute(args: InitArgs) -> Result<()> {
    let bundle_type = args.bundle_type;

    // Resolve target directory and project name
    let (target_dir, name) = resolve_path_and_name(args.path.as_ref(), args.name.as_ref())?;

    // Create template with specified type
    let mut template = BundleTemplate::with_type(&name, bundle_type)?;

    // Override with explicit values if provided
    if let Some(bundle_name) = args.bundle_name {
        template.bundle_name = bundle_name;
    }
    if let Some(package) = args.package {
        template.package_name = package.clone();
        // Extract package_dir from package name (last segment)
        template.package_dir = package
            .split('.')
            .next_back()
            .unwrap_or(&package)
            .to_string();
    }

    // Check if target directory exists and has content
    if target_dir.exists() {
        let has_content = target_dir.read_dir()?.next().is_some();
        if has_content && !args.force {
            return Err(EchidnaError::DirectoryExists(target_dir.clone()));
        }
    } else {
        std::fs::create_dir_all(&target_dir)?;
    }

    // Generate files
    let created_files = template.generate(&target_dir)?;

    // Print summary
    ui::success(&format!(
        "Created ChimeraX bundle project: {} (type: {})",
        template.bundle_name,
        template.bundle_type.display_name()
    ));
    println!();
    println!("Generated files:");
    for file in &created_files {
        // Make path relative for cleaner output
        let display_path = Path::new(file)
            .strip_prefix(&target_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file.clone());
        ui::hint(&format!("  {}", display_path));
    }

    // Show cd hint if not current directory
    let current_dir = std::env::current_dir().ok();
    let needs_cd = current_dir
        .as_ref()
        .map(|c| c != &target_dir)
        .unwrap_or(true);

    if needs_cd {
        ui::next_steps(&[
            (
                &format!("cd {}", target_dir.display()),
                "Navigate to project",
            ),
            ("echidna build", "Build the wheel"),
            ("echidna install", "Install to ChimeraX"),
            ("echidna run", "Build, install, and launch ChimeraX"),
        ]);
    } else {
        ui::next_steps(&[
            ("echidna build", "Build the wheel"),
            ("echidna install", "Install to ChimeraX"),
            ("echidna run", "Build, install, and launch ChimeraX"),
        ]);
    }

    Ok(())
}
