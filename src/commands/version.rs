//! `echidna version` command implementation.

use crate::error::{EchidnaError, Result};
use std::fs;
use std::path::PathBuf;
use toml::Value;

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

    // Read and parse pyproject.toml
    let content = fs::read_to_string(&pyproject_path)?;
    let toml_value: Value = content.parse().map_err(|e: toml::de::Error| {
        EchidnaError::ConfigError(format!("Invalid TOML in pyproject.toml: {}", e))
    })?;

    // Extract version from [project] or [tool.poetry] section
    let current_version = extract_version(&toml_value)
        .ok_or_else(|| EchidnaError::ConfigError("Cannot find version in pyproject.toml".into()))?;

    match args.action {
        VersionAction::Show => {
            println!("{}", current_version);
            Ok(())
        }
        VersionAction::BumpPatch
        | VersionAction::BumpMinor
        | VersionAction::BumpMajor
        | VersionAction::Set(_) => {
            let new_version = compute_new_version(&args.action, &current_version)?;
            update_version_in_file(&pyproject_path, &content, &current_version, &new_version)?;
            println!("{} -> {}", current_version, new_version);
            Ok(())
        }
    }
}

/// Compute new version based on action.
fn compute_new_version(action: &VersionAction, current_version: &str) -> Result<String> {
    match action {
        VersionAction::Show => unreachable!(),
        VersionAction::BumpPatch => {
            let semver = parse_semver(current_version)?;
            Ok(semver.bump_patch().to_string())
        }
        VersionAction::BumpMinor => {
            let semver = parse_semver(current_version)?;
            Ok(semver.bump_minor().to_string())
        }
        VersionAction::BumpMajor => {
            let semver = parse_semver(current_version)?;
            Ok(semver.bump_major().to_string())
        }
        VersionAction::Set(version) => {
            // Validate the new version format
            if SemVer::parse(version).is_none() {
                return Err(EchidnaError::ConfigError(format!(
                    "Invalid version format '{}'. Expected X.Y.Z",
                    version
                )));
            }
            Ok(version.clone())
        }
    }
}

/// Parse version string to SemVer with error.
fn parse_semver(version: &str) -> Result<SemVer> {
    SemVer::parse(version).ok_or_else(|| {
        EchidnaError::ConfigError(format!(
            "Cannot parse version '{}' as semantic version (X.Y.Z)",
            version
        ))
    })
}

/// Extract version from parsed TOML value.
fn extract_version(toml: &Value) -> Option<String> {
    // Try [project] section first (PEP 621)
    if let Some(version) = toml
        .get("project")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
    {
        return Some(version.to_string());
    }

    // Try [tool.poetry] section
    if let Some(version) = toml
        .get("tool")
        .and_then(|t| t.get("poetry"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
    {
        return Some(version.to_string());
    }

    None
}

/// Update version in pyproject.toml file.
///
/// Uses regex-based replacement to preserve formatting and comments.
fn update_version_in_file(
    path: &PathBuf,
    content: &str,
    old_version: &str,
    new_version: &str,
) -> Result<()> {
    // Find and replace version in [project] section first, then [tool.poetry]
    let new_content = if let Some(updated) =
        replace_version_in_section(content, "[project]", old_version, new_version)
    {
        updated
    } else if let Some(updated) =
        replace_version_in_section(content, "[tool.poetry]", old_version, new_version)
    {
        updated
    } else {
        return Err(EchidnaError::ConfigError(
            "Cannot find version field to update in pyproject.toml".into(),
        ));
    };

    // Write atomically by writing to temp file first
    let temp_path = path.with_extension("toml.tmp");
    fs::write(&temp_path, &new_content)?;

    // Clean up temp file on rename failure
    if let Err(e) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path); // Best effort cleanup
        return Err(e.into());
    }

    Ok(())
}

/// Replace version in a specific TOML section.
fn replace_version_in_section(
    content: &str,
    section_header: &str,
    old_version: &str,
    new_version: &str,
) -> Option<String> {
    // Find section start
    let section_start = content.find(section_header)?;

    // Find section end (next section or EOF)
    let section_content = &content[section_start + section_header.len()..];
    let section_end = section_content
        .find("\n[")
        .map(|i| section_start + section_header.len() + i)
        .unwrap_or(content.len());

    let section_text = &content[section_start..section_end];

    // Find version line within section
    // Match patterns: version = "X.Y.Z", version = 'X.Y.Z', version="X.Y.Z", etc.
    for line_start in section_text
        .match_indices('\n')
        .map(|(i, _)| section_start + i + 1)
        .chain(std::iter::once(section_start))
    {
        let line_end = content[line_start..]
            .find('\n')
            .map(|i| line_start + i)
            .unwrap_or(content.len());

        let line = &content[line_start..line_end];
        let trimmed = line.trim();

        // Check if this is a version line
        if trimmed.starts_with("version") {
            // Check if it contains our old version
            if trimmed.contains(old_version) {
                // Replace old version with new version in this line
                let new_line = line.replace(old_version, new_version);
                let mut result = content[..line_start].to_string();
                result.push_str(&new_line);
                result.push_str(&content[line_end..]);
                return Some(result);
            }
        }
    }

    None
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
        let content = "[project]\nname = \"mypackage\"\nversion = \"1.0.0\"\n";
        let toml: Value = toml::from_str(content).unwrap();
        assert_eq!(extract_version(&toml), Some("1.0.0".to_string()));
    }

    #[test]
    fn test_extract_version_poetry_section() {
        let content = "[tool.poetry]\nname = \"mypackage\"\nversion = \"2.0.0\"\n";
        let toml: Value = toml::from_str(content).unwrap();
        assert_eq!(extract_version(&toml), Some("2.0.0".to_string()));
    }

    #[test]
    fn test_extract_version_not_found() {
        let content = "[project]\nname = \"mypackage\"\n";
        let toml: Value = toml::from_str(content).unwrap();
        assert_eq!(extract_version(&toml), None);
    }

    #[test]
    fn test_replace_version_in_section() {
        let content = r#"[project]
name = "mypackage"
version = "1.0.0"
description = "A package"
"#;
        let result = replace_version_in_section(content, "[project]", "1.0.0", "1.0.1").unwrap();
        assert!(result.contains("version = \"1.0.1\""));
        assert!(!result.contains("1.0.0"));
    }

    #[test]
    fn test_replace_version_no_spaces() {
        let content = r#"[project]
name = "mypackage"
version="1.0.0"
"#;
        let result = replace_version_in_section(content, "[project]", "1.0.0", "1.0.1").unwrap();
        assert!(result.contains("version=\"1.0.1\""));
    }

    #[test]
    fn test_replace_version_preserves_other_sections() {
        let content = r#"[tool.some-tool]
version = "0.0.0"

[project]
name = "mypackage"
version = "1.0.0"
"#;
        let result = replace_version_in_section(content, "[project]", "1.0.0", "1.0.1").unwrap();
        // Ensure tool section is not modified
        assert!(result.contains("version = \"0.0.0\""));
        assert!(result.contains("version = \"1.0.1\""));
    }
}
