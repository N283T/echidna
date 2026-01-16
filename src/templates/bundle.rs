//! Bundle template generation.

use crate::error::{EchidnaError, Result};
use std::path::Path;

// Embedded template files
const PYPROJECT_TOML_TEMPLATE: &str = include_str!("../../templates/pyproject.toml.tmpl");
const INIT_PY_TEMPLATE: &str = include_str!("../../templates/init_py.tmpl");
const CMD_PY_TEMPLATE: &str = include_str!("../../templates/cmd_py.tmpl");
const SMOKE_CXC_TEMPLATE: &str = include_str!("../../templates/smoke_cxc.tmpl");
const README_MD_TEMPLATE: &str = include_str!("../../templates/readme_md.tmpl");

/// Bundle template with computed names.
#[derive(Debug, Clone)]
pub struct BundleTemplate {
    /// Bundle name (e.g., "ChimeraX-MyTool")
    pub bundle_name: String,
    /// Python package name (e.g., "chimerax.mytool")
    pub package_name: String,
    /// Package directory name (e.g., "mytool")
    pub package_dir: String,
    /// Command name (e.g., "mytool")
    pub command_name: String,
    /// Version string
    pub version: String,
    /// Description
    pub description: String,
}

impl BundleTemplate {
    /// Create a new bundle template from a project name.
    ///
    /// The name is normalized and used to derive all other names:
    /// - Input: "my-tool" or "MyTool"
    /// - bundle_name: "ChimeraX-MyTool"
    /// - package_name: "chimerax.my_tool"
    /// - package_dir: "my_tool"
    /// - command_name: "my_tool"
    pub fn new(name: &str) -> Result<Self> {
        let name = name.trim();
        if name.is_empty() {
            return Err(EchidnaError::InvalidName("name cannot be empty".into()));
        }

        // Validate: must start with a letter (Python package requirement)
        if !name
            .chars()
            .next()
            .map(|c| c.is_alphabetic())
            .unwrap_or(false)
        {
            return Err(EchidnaError::InvalidName(format!(
                "name '{}' must start with a letter",
                name
            )));
        }

        // Validate: alphanumeric, hyphens, underscores only
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(EchidnaError::InvalidName(format!(
                "name '{}' contains invalid characters (use alphanumeric, hyphens, or underscores)",
                name
            )));
        }

        // ChimeraX derives package name from bundle name by removing separators
        // e.g., "ChimeraX-HelloWorld" -> "chimerax.helloworld"
        let package_dir = name.to_lowercase().replace(['-', '_', ' '], "");
        let command_name = name.to_lowercase().replace(['-', ' '], "_");
        let capitalized = capitalize_words(name);

        Ok(Self {
            bundle_name: format!("ChimeraX-{}", capitalized),
            package_name: format!("chimerax.{}", package_dir),
            package_dir,
            command_name,
            version: "0.1.0".to_string(),
            description: format!("ChimeraX {} bundle", capitalized),
        })
    }

    /// Generate the bundle files in the target directory.
    pub fn generate(&self, target_dir: &Path) -> Result<Vec<String>> {
        // ChimeraX bundle builder copies src/ contents into chimerax/<package>/
        // So we put files directly in src/, not in a subdirectory
        let src_dir = target_dir.join("src");
        let scripts_dir = target_dir.join("scripts");

        std::fs::create_dir_all(&src_dir)?;
        std::fs::create_dir_all(&scripts_dir)?;

        let mut created_files = Vec::new();

        // Generate each file
        let files = [
            (target_dir.join("pyproject.toml"), PYPROJECT_TOML_TEMPLATE),
            (src_dir.join("__init__.py"), INIT_PY_TEMPLATE),
            (src_dir.join("cmd.py"), CMD_PY_TEMPLATE),
            (scripts_dir.join("smoke.cxc"), SMOKE_CXC_TEMPLATE),
            (target_dir.join("README.md"), README_MD_TEMPLATE),
        ];

        for (path, template) in files {
            let content = self.render_template(template);
            std::fs::write(&path, content)?;
            created_files.push(path.to_string_lossy().to_string());
        }

        Ok(created_files)
    }

    /// Render a template with variable substitution.
    fn render_template(&self, template: &str) -> String {
        // Create PascalCase version for class names
        let pascal_case = to_pascal_case(&self.command_name);

        template
            .replace("{{bundle_name}}", &self.bundle_name)
            .replace("{{package_name}}", &self.package_name)
            .replace("{{package_dir}}", &self.package_dir)
            .replace("{{command_name}}", &self.command_name)
            .replace("{{command_name_pascal}}", &pascal_case)
            .replace("{{version}}", &self.version)
            .replace("{{description}}", &self.description)
    }
}

/// Capitalize words in a name (e.g., "my-tool" -> "MyTool").
fn capitalize_words(name: &str) -> String {
    name.split(['-', '_'])
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Convert a snake_case name to PascalCase.
fn to_pascal_case(name: &str) -> String {
    name.split('_')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_name_derivation() {
        let template = BundleTemplate::new("my-tool").unwrap();
        assert_eq!(template.bundle_name, "ChimeraX-MyTool");
        assert_eq!(template.package_name, "chimerax.mytool");
        assert_eq!(template.package_dir, "mytool");
        assert_eq!(template.command_name, "my_tool");
    }

    #[test]
    fn test_underscore_name() {
        let template = BundleTemplate::new("my_tool").unwrap();
        assert_eq!(template.bundle_name, "ChimeraX-MyTool");
        assert_eq!(template.package_name, "chimerax.mytool");
        assert_eq!(template.package_dir, "mytool");
        assert_eq!(template.command_name, "my_tool");
    }

    #[test]
    fn test_name_with_numbers() {
        let template = BundleTemplate::new("tool2").unwrap();
        assert_eq!(template.bundle_name, "ChimeraX-Tool2");
        assert_eq!(template.package_name, "chimerax.tool2");
        assert_eq!(template.package_dir, "tool2");
        assert_eq!(template.command_name, "tool2");
    }

    #[test]
    fn test_single_character_name() {
        let template = BundleTemplate::new("x").unwrap();
        assert_eq!(template.bundle_name, "ChimeraX-X");
        assert_eq!(template.package_name, "chimerax.x");
        assert_eq!(template.package_dir, "x");
        assert_eq!(template.command_name, "x");
    }

    #[test]
    fn test_capitalize_words() {
        assert_eq!(capitalize_words("my-tool"), "MyTool");
        assert_eq!(capitalize_words("my_tool"), "MyTool");
        assert_eq!(capitalize_words("mytool"), "Mytool");
        assert_eq!(capitalize_words("MyTool"), "MyTool"); // preserves existing caps
    }

    #[test]
    fn test_pascal_case() {
        assert_eq!(to_pascal_case("my_tool"), "MyTool");
        assert_eq!(to_pascal_case("example"), "Example");
        assert_eq!(to_pascal_case("multi_word_name"), "MultiWordName");
    }

    #[test]
    fn test_invalid_name() {
        assert!(BundleTemplate::new("").is_err());
        assert!(BundleTemplate::new("my tool").is_err()); // space not allowed
        assert!(BundleTemplate::new("my.tool").is_err()); // dot not allowed
    }

    #[test]
    fn test_whitespace_only_name() {
        assert!(BundleTemplate::new("   ").is_err());
    }

    #[test]
    fn test_name_with_special_chars() {
        assert!(BundleTemplate::new("my@tool").is_err());
        assert!(BundleTemplate::new("my#tool").is_err());
        assert!(BundleTemplate::new("my/tool").is_err());
    }

    #[test]
    fn test_name_starting_with_number() {
        assert!(BundleTemplate::new("123tool").is_err());
        assert!(BundleTemplate::new("1st-tool").is_err());
        // But numbers after first char are OK
        assert!(BundleTemplate::new("tool123").is_ok());
        assert!(BundleTemplate::new("my2tool").is_ok());
    }

    #[test]
    fn test_name_starting_with_hyphen_or_underscore() {
        assert!(BundleTemplate::new("-tool").is_err());
        assert!(BundleTemplate::new("_tool").is_err());
    }

    #[test]
    fn test_render_template_substitution() {
        let template = BundleTemplate::new("example").unwrap();
        let input = "Name: {{bundle_name}}, Package: {{package_name}}, Dir: {{package_dir}}, Cmd: {{command_name}}";
        let output = template.render_template(input);
        assert_eq!(
            output,
            "Name: ChimeraX-Example, Package: chimerax.example, Dir: example, Cmd: example"
        );
    }

    #[test]
    fn test_render_template_pascal_case() {
        let template = BundleTemplate::new("my-tool").unwrap();
        let input = "class {{command_name_pascal}}Command:";
        let output = template.render_template(input);
        assert_eq!(output, "class MyToolCommand:");
    }

    #[test]
    fn test_render_template_version_and_description() {
        let template = BundleTemplate::new("test").unwrap();
        let input = "version = \"{{version}}\"\ndescription = \"{{description}}\"";
        let output = template.render_template(input);
        assert!(output.contains("version = \"0.1.0\""));
        assert!(output.contains("ChimeraX Test bundle"));
    }

    #[test]
    fn test_generate_creates_files() {
        let temp = TempDir::new().unwrap();
        let template = BundleTemplate::new("test-bundle").unwrap();

        let created = template.generate(temp.path()).unwrap();

        // Check expected files are created
        assert!(temp.path().join("pyproject.toml").exists());
        assert!(temp.path().join("src/__init__.py").exists());
        assert!(temp.path().join("src/cmd.py").exists());
        assert!(temp.path().join("scripts/smoke.cxc").exists());
        assert!(temp.path().join("README.md").exists());

        // Check created files list
        assert_eq!(created.len(), 5);
    }

    #[test]
    fn test_generate_pyproject_content() {
        let temp = TempDir::new().unwrap();
        let template = BundleTemplate::new("my-tool").unwrap();

        template.generate(temp.path()).unwrap();

        let content = std::fs::read_to_string(temp.path().join("pyproject.toml")).unwrap();
        assert!(content.contains("name = \"ChimeraX-MyTool\""));
        assert!(content.contains("package = \"chimerax.mytool\""));
        assert!(content.contains("[chimerax.command.my_tool]"));
    }

    #[test]
    fn test_trimmed_name() {
        let template = BundleTemplate::new("  my-tool  ").unwrap();
        assert_eq!(template.bundle_name, "ChimeraX-MyTool");
    }
}
