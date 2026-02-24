//! `echidna packages` command implementation.

use crate::chimerax::{ChimeraXExecutor, Verbosity};
use crate::error::{EchidnaError, Result};
use crate::packages::{
    filter_stdlib, ConflictCheckResult, PackageInfo, PackageInstaller, PackageResolver,
};
use std::fs;
use std::path::PathBuf;

/// Output format for package info.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

/// Arguments for the `packages list` command.
pub struct ListArgs {
    pub format: OutputFormat,
    pub include_stdlib: bool,
    pub chimerax: PathBuf,
    pub verbosity: Verbosity,
}

/// Arguments for the `packages check` command.
pub struct CheckArgs {
    pub package: Option<String>,
    pub requirements: Option<PathBuf>,
    pub format: OutputFormat,
    pub chimerax: PathBuf,
    pub verbosity: Verbosity,
}

/// Arguments for the `packages install` command.
pub struct InstallArgs {
    pub packages: Vec<String>,
    pub upgrade: bool,
    pub dry_run: bool,
    pub chimerax: PathBuf,
    pub verbosity: Verbosity,
}

/// Execute the `packages list` command.
pub fn list(args: ListArgs) -> Result<()> {
    let executor = ChimeraXExecutor::new(args.chimerax, args.verbosity);

    eprintln!("Querying ChimeraX Python environment...");
    let python_info = executor.get_python_info()?;
    let resolver = PackageResolver::new(python_info.executable.into(), args.verbosity);

    eprintln!("Using {} backend...", resolver.backend());
    let packages = resolver.list_packages()?;

    // Filter stdlib if requested
    let packages = if args.include_stdlib {
        packages
    } else {
        filter_stdlib(packages)
    };

    // Output
    match args.format {
        OutputFormat::Text => print_package_list(&packages, args.include_stdlib),
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&packages)?);
        }
    }

    Ok(())
}

/// Execute the `packages check` command.
pub fn check(args: CheckArgs) -> Result<()> {
    // Validate arguments
    if args.package.is_none() && args.requirements.is_none() {
        return Err(EchidnaError::PackageError(
            "Either a package name or --requirements file must be specified".to_string(),
        ));
    }

    // Collect packages to check first (before moving args)
    let packages = collect_packages_to_check(&args)?;

    // Ensure we have at least one package to check
    if packages.is_empty() {
        return Err(EchidnaError::PackageError(
            "No packages to check (requirements file may contain only comments)".to_string(),
        ));
    }

    let executor = ChimeraXExecutor::new(args.chimerax, args.verbosity);

    eprintln!("Querying ChimeraX Python environment...");
    let python_info = executor.get_python_info()?;
    let resolver = PackageResolver::new(python_info.executable.into(), args.verbosity);

    eprintln!("Using {} backend...", resolver.backend());

    // Check each package
    let mut results = Vec::new();
    let mut has_conflicts = false;

    for package in &packages {
        eprintln!("Checking '{}'...", package);
        let result = resolver.check_package(package)?;
        if !result.installable {
            has_conflicts = true;
        }
        results.push(result);
    }

    // Output results
    match args.format {
        OutputFormat::Text => {
            for result in &results {
                print_check_result(result);
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
    }

    if has_conflicts {
        Err(EchidnaError::PackageConflict)
    } else {
        Ok(())
    }
}

/// Execute the `packages install` command.
pub fn install(args: InstallArgs) -> Result<()> {
    // Validate that we have packages to install
    if args.packages.is_empty() {
        return Err(EchidnaError::PackageError(
            "No packages specified to install".to_string(),
        ));
    }

    let executor = ChimeraXExecutor::new(args.chimerax, args.verbosity);

    eprintln!("Querying ChimeraX Python environment...");
    let python_info = executor.get_python_info()?;
    let installer = PackageInstaller::new(python_info.executable.into(), args.verbosity);

    eprintln!("Using {} backend...", installer.backend());

    let action = if args.dry_run {
        "Would install"
    } else {
        "Installing"
    };
    let packages_str = args.packages.join(", ");
    eprintln!("{}: {}", action, packages_str);

    let result = installer.install(&args.packages, args.upgrade, args.dry_run)?;

    if args.dry_run {
        println!();
        println!("Dry run - no packages were installed.");
        println!("Output from {}:", installer.backend());
        println!("{}", result.output.trim());
    } else {
        println!();
        println!(
            "Successfully installed {} package(s): {}",
            result.installed.len(),
            result.installed.join(", ")
        );
    }

    Ok(())
}

/// Collect packages to check from args.
fn collect_packages_to_check(args: &CheckArgs) -> Result<Vec<String>> {
    let mut packages = Vec::new();

    // Add package from command line if specified
    if let Some(ref pkg) = args.package {
        packages.push(pkg.clone());
    }

    // Add packages from requirements file if specified
    if let Some(ref req_file) = args.requirements {
        let contents = fs::read_to_string(req_file).map_err(|e| {
            EchidnaError::PackageError(format!(
                "Failed to read requirements file '{}': {}",
                req_file.display(),
                e
            ))
        })?;

        for line in contents.lines() {
            let line = line.trim();
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Skip options like -e, --editable, etc.
            if line.starts_with('-') {
                continue;
            }
            packages.push(line.to_string());
        }
    }

    Ok(packages)
}

/// Print package list in text format.
fn print_package_list(packages: &[PackageInfo], include_stdlib: bool) {
    println!();
    println!("ChimeraX Python packages:");
    println!();

    // Calculate column widths
    let max_name_len = packages
        .iter()
        .map(|p| p.name.len())
        .max()
        .unwrap_or(7)
        .max(7);

    // Print header
    println!("{:<width$}  Version", "Package", width = max_name_len);
    println!("{}", "─".repeat(max_name_len + 2 + 20));

    // Print packages
    for pkg in packages {
        println!(
            "{:<width$}  {}",
            pkg.name,
            pkg.version,
            width = max_name_len
        );
    }

    println!();
    let stdlib_note = if include_stdlib {
        ""
    } else {
        " (excluding stdlib)"
    };
    println!("Found {} packages{}", packages.len(), stdlib_note);
}

/// Print conflict check result in text format.
fn print_check_result(result: &ConflictCheckResult) {
    println!();

    if result.installable {
        println!(
            "\u{2713} Package '{}' can be installed without conflicts",
            result.package
        );

        if !result.would_install.is_empty() {
            println!();
            println!("Would install:");
            for pkg in &result.would_install {
                println!("  {} {}", pkg.name, pkg.version);
            }
        }

        if !result.would_change.is_empty() {
            println!();
            println!("Would change:");
            for change in &result.would_change {
                println!(
                    "  {}: {} → {}",
                    change.name, change.from_version, change.to_version
                );
            }
        }
    } else {
        println!("\u{2717} Conflicts detected for '{}'!", result.package);

        if !result.conflicts.is_empty() {
            println!();
            println!("Conflicts:");
            for conflict in &result.conflicts {
                println!("  - {}", conflict.reason);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_collect_packages_from_package_arg() {
        let args = CheckArgs {
            package: Some("numpy>=1.20".to_string()),
            requirements: None,
            format: OutputFormat::Text,
            chimerax: PathBuf::new(),
            verbosity: 0,
        };

        let packages = collect_packages_to_check(&args).unwrap();
        assert_eq!(packages, vec!["numpy>=1.20"]);
    }

    #[test]
    fn test_collect_packages_from_requirements_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "numpy>=1.20").unwrap();
        writeln!(file, "requests").unwrap();

        let args = CheckArgs {
            package: None,
            requirements: Some(file.path().to_path_buf()),
            format: OutputFormat::Text,
            chimerax: PathBuf::new(),
            verbosity: 0,
        };

        let packages = collect_packages_to_check(&args).unwrap();
        assert_eq!(packages, vec!["numpy>=1.20", "requests"]);
    }

    #[test]
    fn test_collect_packages_skips_comments() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# comment").unwrap();
        writeln!(file, "numpy>=1.20").unwrap();
        writeln!(file, "  # another comment").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "requests").unwrap();

        let args = CheckArgs {
            package: None,
            requirements: Some(file.path().to_path_buf()),
            format: OutputFormat::Text,
            chimerax: PathBuf::new(),
            verbosity: 0,
        };

        let packages = collect_packages_to_check(&args).unwrap();
        assert_eq!(packages, vec!["numpy>=1.20", "requests"]);
    }

    #[test]
    fn test_collect_packages_skips_options() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "-e ./local").unwrap();
        writeln!(file, "--extra-index-url https://example.com").unwrap();
        writeln!(file, "numpy>=1.20").unwrap();

        let args = CheckArgs {
            package: None,
            requirements: Some(file.path().to_path_buf()),
            format: OutputFormat::Text,
            chimerax: PathBuf::new(),
            verbosity: 0,
        };

        let packages = collect_packages_to_check(&args).unwrap();
        assert_eq!(packages, vec!["numpy>=1.20"]);
    }

    #[test]
    fn test_collect_packages_combines_args_and_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "scipy").unwrap();

        let args = CheckArgs {
            package: Some("numpy".to_string()),
            requirements: Some(file.path().to_path_buf()),
            format: OutputFormat::Text,
            chimerax: PathBuf::new(),
            verbosity: 0,
        };

        let packages = collect_packages_to_check(&args).unwrap();
        assert_eq!(packages, vec!["numpy", "scipy"]);
    }
}
