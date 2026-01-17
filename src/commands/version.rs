//! `echidna version` command implementation.

use crate::error::{EchidnaError, Result};
use std::fs;
use std::path::PathBuf;

/// Arguments for the version command.
pub struct VersionArgs {
    /// Project directory
    pub path: PathBuf,
    /// Version action (show, bump, or set)
    pub action: VersionAction,
}

/// Version action to perform.
#[derive(Debug, Clone)]
pub enum VersionAction {
    /// Show current version
    Show,
    /// Bump patch version (0.0.X)
    BumpPatch,
    /// Bump minor version (0.X.0)
    BumpMinor,
    /// Bump major version (X.0.0)
    BumpMajor,
    /// Set specific version
    Set(String),
}

/// Parsed semantic version.
#[derive(Debug, Clone, PartialEq)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    /// Parse a version string.
    pub fn parse(version: &str) -> Option<Self> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;

        Some(Self {
            major,
            minor,
            patch,
        })
    }

    /// Bump patch version.
    pub fn bump_patch(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
        }
    }

    /// Bump minor version.
    pub fn bump_minor(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
        }
    }

    /// Bump major version.
    pub fn bump_major(&self) -> Self {
        Self {
            major: self.major + 1,
            minor: 0,
            patch: 0,
        }
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Execute the version command.
pub fn execute(args: VersionArgs) -> Result<()> {
    let project_dir = args.path.canonicalize().map_err(|e| {
        EchidnaError::ConfigError(format!(
            "Cannot access project directory '{}': {}",
            args.path.display(),
            e
        ))
    })?;

    let pyproject_path = project_dir.join("pyproject.toml");
    if !pyproject_path.exists() {
        return Err(EchidnaError::NotBundleDirectory(project_dir));
    }

    // Read current version
    let content = fs::read_to_string(&pyproject_path)?;
    let current_version = extract_version(&content)
        .ok_or_else(|| EchidnaError::ConfigError("Cannot find version in pyproject.toml".into()))?;

    match args.action {
        VersionAction::Show => {
            println!("{}", current_version);
            Ok(())
        }
        VersionAction::BumpPatch => {
            let semver = SemVer::parse(&current_version).ok_or_else(|| {
                EchidnaError::ConfigError(format!(
                    "Cannot parse version '{}' as semantic version",
                    current_version
                ))
            })?;
            let new_version = semver.bump_patch();
            update_version(
                &pyproject_path,
                &content,
                &current_version,
                &new_version.to_string(),
            )?;
            println!("{} -> {}", current_version, new_version);
            Ok(())
        }
        VersionAction::BumpMinor => {
            let semver = SemVer::parse(&current_version).ok_or_else(|| {
                EchidnaError::ConfigError(format!(
                    "Cannot parse version '{}' as semantic version",
                    current_version
                ))
            })?;
            let new_version = semver.bump_minor();
            update_version(
                &pyproject_path,
                &content,
                &current_version,
                &new_version.to_string(),
            )?;
            println!("{} -> {}", current_version, new_version);
            Ok(())
        }
        VersionAction::BumpMajor => {
            let semver = SemVer::parse(&current_version).ok_or_else(|| {
                EchidnaError::ConfigError(format!(
                    "Cannot parse version '{}' as semantic version",
                    current_version
                ))
            })?;
            let new_version = semver.bump_major();
            update_version(
                &pyproject_path,
                &content,
                &current_version,
                &new_version.to_string(),
            )?;
            println!("{} -> {}", current_version, new_version);
            Ok(())
        }
        VersionAction::Set(new_version) => {
            // Validate the new version format
            if SemVer::parse(&new_version).is_none() {
                return Err(EchidnaError::ConfigError(format!(
                    "Invalid version format '{}'. Expected X.Y.Z",
                    new_version
                )));
            }
            update_version(&pyproject_path, &content, &current_version, &new_version)?;
            println!("{} -> {}", current_version, new_version);
            Ok(())
        }
    }
}

/// Extract version from pyproject.toml content.
fn extract_version(content: &str) -> Option<String> {
    // Try [project] section first (PEP 621)
    if let Some(version) = extract_version_from_section(content, "[project]") {
        return Some(version);
    }

    // Try [tool.poetry] section
    if let Some(version) = extract_version_from_section(content, "[tool.poetry]") {
        return Some(version);
    }

    None
}

/// Extract version from a specific TOML section.
fn extract_version_from_section(content: &str, section: &str) -> Option<String> {
    let section_start = content.find(section)?;
    let section_content = &content[section_start..];

    // Find the next section or end of file
    let section_end = section_content[1..]
        .find("\n[")
        .map(|i| i + 1)
        .unwrap_or(section_content.len());

    let section_text = &section_content[..section_end];

    // Find version line
    for line in section_text.lines() {
        let line = line.trim();
        if line.starts_with("version") {
            // Parse version = "X.Y.Z"
            if let Some(eq_pos) = line.find('=') {
                let value = line[eq_pos + 1..].trim();
                // Remove quotes
                let version = value.trim_matches('"').trim_matches('\'');
                return Some(version.to_string());
            }
        }
    }

    None
}

/// Update version in pyproject.toml.
fn update_version(
    path: &PathBuf,
    content: &str,
    old_version: &str,
    new_version: &str,
) -> Result<()> {
    // Simple string replacement - replace first occurrence of the version string
    // This preserves formatting and comments
    let old_pattern = format!("version = \"{}\"", old_version);
    let new_pattern = format!("version = \"{}\"", new_version);

    let new_content = if content.contains(&old_pattern) {
        content.replacen(&old_pattern, &new_pattern, 1)
    } else {
        // Try with single quotes
        let old_pattern = format!("version = '{}'", old_version);
        let new_pattern = format!("version = '{}'", new_version);
        if content.contains(&old_pattern) {
            content.replacen(&old_pattern, &new_pattern, 1)
        } else {
            return Err(EchidnaError::ConfigError(
                "Cannot find version field to update".into(),
            ));
        }
    };

    fs::write(path, new_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_semver_parse_invalid() {
        assert!(SemVer::parse("1.2").is_none());
        assert!(SemVer::parse("1.2.3.4").is_none());
        assert!(SemVer::parse("abc").is_none());
    }

    #[test]
    fn test_semver_bump_patch() {
        let v = SemVer::parse("1.2.3").unwrap();
        let bumped = v.bump_patch();
        assert_eq!(bumped.to_string(), "1.2.4");
    }

    #[test]
    fn test_semver_bump_minor() {
        let v = SemVer::parse("1.2.3").unwrap();
        let bumped = v.bump_minor();
        assert_eq!(bumped.to_string(), "1.3.0");
    }

    #[test]
    fn test_semver_bump_major() {
        let v = SemVer::parse("1.2.3").unwrap();
        let bumped = v.bump_major();
        assert_eq!(bumped.to_string(), "2.0.0");
    }

    #[test]
    fn test_extract_version_project_section() {
        let content = r#"
[project]
name = "mypackage"
version = "1.0.0"
description = "A package"
"#;
        assert_eq!(extract_version(content), Some("1.0.0".to_string()));
    }

    #[test]
    fn test_extract_version_poetry_section() {
        let content = r#"
[tool.poetry]
name = "mypackage"
version = "2.0.0"
"#;
        assert_eq!(extract_version(content), Some("2.0.0".to_string()));
    }

    #[test]
    fn test_extract_version_not_found() {
        let content = r#"
[project]
name = "mypackage"
"#;
        assert_eq!(extract_version(content), None);
    }
}
