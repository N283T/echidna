//! `echidna init` command implementation.

use crate::error::{EchidnaError, Result};
use crate::templates::BundleTemplate;
use std::path::Path;

/// Arguments for the init command.
pub struct InitArgs {
    pub name: Option<String>,
    pub bundle_name: Option<String>,
    pub package: Option<String>,
    pub path: std::path::PathBuf,
    pub force: bool,
}

/// Execute the init command.
pub fn execute(args: InitArgs) -> Result<()> {
    let target_dir = &args.path;

    // Determine the project name
    let name = match args.name {
        Some(n) => n,
        None => {
            // Use directory name as project name
            target_dir
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    EchidnaError::InvalidName("Could not determine project name from path".into())
                })?
        }
    };

    // Create template
    let mut template = BundleTemplate::new(&name)?;

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
            return Err(EchidnaError::DirectoryExists(target_dir.to_path_buf()));
        }
    } else {
        std::fs::create_dir_all(target_dir)?;
    }

    // Generate files
    let created_files = template.generate(target_dir)?;

    // Print summary
    println!("Created ChimeraX bundle project: {}", template.bundle_name);
    println!();
    println!("Generated files:");
    for file in &created_files {
        // Make path relative for cleaner output
        let display_path = Path::new(file)
            .strip_prefix(target_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file.clone());
        println!("  {}", display_path);
    }
    println!();
    println!("Next steps:");
    println!("  cd {}", target_dir.display());
    println!("  echidna build      # Build the wheel");
    println!("  echidna install    # Install to ChimeraX");
    println!("  echidna run        # Build, install, and launch ChimeraX");

    Ok(())
}
