//! Bundle template generation.

use crate::error::{EchidnaError, Result};
use std::path::Path;

/// Bundle type for template generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BundleType {
    /// Command-only bundle (default)
    #[default]
    Command,
    /// Qt-based GUI tool
    Tool,
    /// HTML-based GUI tool
    ToolHtml,
    /// File format reader/writer
    Format,
    /// Network database fetcher
    Fetch,
    /// Chemical subgroup selector
    Selector,
    /// Visualization presets
    Preset,
    /// C/C++ extension bundle
    Cpp,
}

impl BundleType {
    /// Parse bundle type from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "command" => Some(Self::Command),
            "tool" => Some(Self::Tool),
            "tool-html" | "toolhtml" => Some(Self::ToolHtml),
            "format" => Some(Self::Format),
            "fetch" => Some(Self::Fetch),
            "selector" => Some(Self::Selector),
            "preset" => Some(Self::Preset),
            "cpp" | "c++" => Some(Self::Cpp),
            _ => None,
        }
    }

    /// Get display name for the bundle type.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Tool => "tool (Qt)",
            Self::ToolHtml => "tool (HTML)",
            Self::Format => "format",
            Self::Fetch => "fetch",
            Self::Selector => "selector",
            Self::Preset => "preset",
            Self::Cpp => "C++ extension",
        }
    }
}

// Embedded template files - Command (default)
const PYPROJECT_TOML_TEMPLATE: &str = include_str!("../../templates/command/pyproject.toml.tmpl");
const INIT_PY_TEMPLATE: &str = include_str!("../../templates/command/init_py.tmpl");
const CMD_PY_TEMPLATE: &str = include_str!("../../templates/command/cmd_py.tmpl");
const SMOKE_CXC_TEMPLATE: &str = include_str!("../../templates/common/smoke_cxc.tmpl");
const README_MD_TEMPLATE: &str = include_str!("../../templates/common/readme_md.tmpl");

// Tool (Qt) templates
const TOOL_PYPROJECT_TEMPLATE: &str = include_str!("../../templates/tool/pyproject.toml.tmpl");
const TOOL_INIT_TEMPLATE: &str = include_str!("../../templates/tool/init_py.tmpl");
const TOOL_PY_TEMPLATE: &str = include_str!("../../templates/tool/tool_py.tmpl");

// Tool (HTML) templates
const TOOL_HTML_PYPROJECT_TEMPLATE: &str =
    include_str!("../../templates/tool-html/pyproject.toml.tmpl");
const TOOL_HTML_INIT_TEMPLATE: &str = include_str!("../../templates/tool-html/init_py.tmpl");
const TOOL_HTML_PY_TEMPLATE: &str = include_str!("../../templates/tool-html/tool_py.tmpl");

// Format templates
const FORMAT_PYPROJECT_TEMPLATE: &str = include_str!("../../templates/format/pyproject.toml.tmpl");
const FORMAT_INIT_TEMPLATE: &str = include_str!("../../templates/format/init_py.tmpl");
const FORMAT_OPEN_TEMPLATE: &str = include_str!("../../templates/format/open_py.tmpl");

// Fetch templates
const FETCH_PYPROJECT_TEMPLATE: &str = include_str!("../../templates/fetch/pyproject.toml.tmpl");
const FETCH_INIT_TEMPLATE: &str = include_str!("../../templates/fetch/init_py.tmpl");
const FETCH_PY_TEMPLATE: &str = include_str!("../../templates/fetch/fetch_py.tmpl");

// Selector templates
const SELECTOR_PYPROJECT_TEMPLATE: &str =
    include_str!("../../templates/selector/pyproject.toml.tmpl");
const SELECTOR_INIT_TEMPLATE: &str = include_str!("../../templates/selector/init_py.tmpl");
const SELECTOR_PY_TEMPLATE: &str = include_str!("../../templates/selector/selector_py.tmpl");

// Preset templates
const PRESET_PYPROJECT_TEMPLATE: &str = include_str!("../../templates/preset/pyproject.toml.tmpl");
const PRESET_INIT_TEMPLATE: &str = include_str!("../../templates/preset/init_py.tmpl");
const PRESET_PY_TEMPLATE: &str = include_str!("../../templates/preset/preset_py.tmpl");

// C++ extension templates
const CPP_PYPROJECT_TEMPLATE: &str = include_str!("../../templates/cpp/pyproject.toml.tmpl");
const CPP_INIT_TEMPLATE: &str = include_str!("../../templates/cpp/init_py.tmpl");
const CPP_CMD_TEMPLATE: &str = include_str!("../../templates/cpp/cmd_py.tmpl");
const CPP_EXTENSION_TEMPLATE: &str = include_str!("../../templates/cpp/extension_cpp.tmpl");

/// Bundle template with computed names.
#[derive(Debug, Clone)]
pub struct BundleTemplate {
    /// Bundle type
    pub bundle_type: BundleType,
    /// Bundle name (e.g., "ChimeraX-MyTool")
    pub bundle_name: String,
    /// Python package name (e.g., "chimerax.mytool")
    pub package_name: String,
    /// Package directory name (e.g., "mytool")
    pub package_dir: String,
    /// Command/tool name (e.g., "mytool")
    pub command_name: String,
    /// Tool display name (e.g., "My Tool") for GUI tools
    pub tool_name: String,
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
    /// - tool_name: "My Tool" (for GUI tools)
    pub fn new(name: &str) -> Result<Self> {
        Self::with_type(name, BundleType::default())
    }

    /// Create a new bundle template with a specific bundle type.
    pub fn with_type(name: &str, bundle_type: BundleType) -> Result<Self> {
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
        let tool_name = capitalize_words_with_spaces(name);

        Ok(Self {
            bundle_type,
            bundle_name: format!("ChimeraX-{}", capitalized),
            package_name: format!("chimerax.{}", package_dir),
            package_dir,
            command_name,
            tool_name,
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

        // Common files (README and smoke test)
        let common_files = [
            (scripts_dir.join("smoke.cxc"), SMOKE_CXC_TEMPLATE),
            (target_dir.join("README.md"), README_MD_TEMPLATE),
        ];

        for (path, template) in common_files {
            let content = self.render_template(template);
            std::fs::write(&path, content)?;
            created_files.push(path.to_string_lossy().to_string());
        }

        // Type-specific files
        match self.bundle_type {
            BundleType::Command => {
                self.write_file(
                    &target_dir.join("pyproject.toml"),
                    PYPROJECT_TOML_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &src_dir.join("__init__.py"),
                    INIT_PY_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(&src_dir.join("cmd.py"), CMD_PY_TEMPLATE, &mut created_files)?;
            }
            BundleType::Tool => {
                self.write_file(
                    &target_dir.join("pyproject.toml"),
                    TOOL_PYPROJECT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &src_dir.join("__init__.py"),
                    TOOL_INIT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &src_dir.join("tool.py"),
                    TOOL_PY_TEMPLATE,
                    &mut created_files,
                )?;
            }
            BundleType::ToolHtml => {
                self.write_file(
                    &target_dir.join("pyproject.toml"),
                    TOOL_HTML_PYPROJECT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &src_dir.join("__init__.py"),
                    TOOL_HTML_INIT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &src_dir.join("tool.py"),
                    TOOL_HTML_PY_TEMPLATE,
                    &mut created_files,
                )?;
            }
            BundleType::Format => {
                self.write_file(
                    &target_dir.join("pyproject.toml"),
                    FORMAT_PYPROJECT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &src_dir.join("__init__.py"),
                    FORMAT_INIT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &src_dir.join("open.py"),
                    FORMAT_OPEN_TEMPLATE,
                    &mut created_files,
                )?;
            }
            BundleType::Fetch => {
                self.write_file(
                    &target_dir.join("pyproject.toml"),
                    FETCH_PYPROJECT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &src_dir.join("__init__.py"),
                    FETCH_INIT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &src_dir.join("fetch.py"),
                    FETCH_PY_TEMPLATE,
                    &mut created_files,
                )?;
            }
            BundleType::Selector => {
                self.write_file(
                    &target_dir.join("pyproject.toml"),
                    SELECTOR_PYPROJECT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &src_dir.join("__init__.py"),
                    SELECTOR_INIT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &src_dir.join("selector.py"),
                    SELECTOR_PY_TEMPLATE,
                    &mut created_files,
                )?;
            }
            BundleType::Preset => {
                self.write_file(
                    &target_dir.join("pyproject.toml"),
                    PRESET_PYPROJECT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &src_dir.join("__init__.py"),
                    PRESET_INIT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &src_dir.join("preset.py"),
                    PRESET_PY_TEMPLATE,
                    &mut created_files,
                )?;
            }
            BundleType::Cpp => {
                // For C++ bundles, we need to put source in src/chimerax/<package>/
                // because that's where pyproject.toml expects it
                let cpp_src_dir = src_dir.join("chimerax").join(&self.package_dir);
                std::fs::create_dir_all(&cpp_src_dir)?;

                self.write_file(
                    &target_dir.join("pyproject.toml"),
                    CPP_PYPROJECT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &cpp_src_dir.join("__init__.py"),
                    CPP_INIT_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &cpp_src_dir.join("cmd.py"),
                    CPP_CMD_TEMPLATE,
                    &mut created_files,
                )?;
                self.write_file(
                    &cpp_src_dir.join("_extension.cpp"),
                    CPP_EXTENSION_TEMPLATE,
                    &mut created_files,
                )?;
            }
        }

        Ok(created_files)
    }

    /// Write a template file and track it.
    fn write_file(
        &self,
        path: &Path,
        template: &str,
        created_files: &mut Vec<String>,
    ) -> Result<()> {
        let content = self.render_template(template);
        std::fs::write(path, content)?;
        created_files.push(path.to_string_lossy().to_string());
        Ok(())
    }

    /// Render a template with variable substitution.
    fn render_template(&self, template: &str) -> String {
        // Create PascalCase version for class names
        let pascal_case = to_pascal_case(&self.command_name);
        // Create PascalCase from the capitalized bundle name (MyTool from ChimeraX-MyTool)
        let pascal_name = capitalize_words(&self.command_name.replace('_', "-"));

        template
            .replace("{{bundle_name}}", &self.bundle_name)
            .replace("{{package_name}}", &self.package_name)
            .replace("{{package_dir}}", &self.package_dir)
            .replace("{{command_name}}", &self.command_name)
            .replace("{{command_name_pascal}}", &pascal_case)
            .replace("{{pascal_name}}", &pascal_name)
            .replace("{{tool_name}}", &self.tool_name)
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

/// Capitalize words with spaces (e.g., "my-tool" -> "My Tool").
fn capitalize_words_with_spaces(name: &str) -> String {
    name.split(['-', '_'])
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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
        assert_eq!(template.tool_name, "My Tool");
        assert_eq!(template.bundle_type, BundleType::Command);
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

    #[test]
    fn test_bundle_type_from_str() {
        assert_eq!(BundleType::parse("command"), Some(BundleType::Command));
        assert_eq!(BundleType::parse("tool"), Some(BundleType::Tool));
        assert_eq!(BundleType::parse("tool-html"), Some(BundleType::ToolHtml));
        assert_eq!(BundleType::parse("toolhtml"), Some(BundleType::ToolHtml));
        assert_eq!(BundleType::parse("format"), Some(BundleType::Format));
        assert_eq!(BundleType::parse("fetch"), Some(BundleType::Fetch));
        assert_eq!(BundleType::parse("selector"), Some(BundleType::Selector));
        assert_eq!(BundleType::parse("preset"), Some(BundleType::Preset));
        assert_eq!(BundleType::parse("cpp"), Some(BundleType::Cpp));
        assert_eq!(BundleType::parse("c++"), Some(BundleType::Cpp));
        assert_eq!(BundleType::parse("COMMAND"), Some(BundleType::Command));
        assert_eq!(BundleType::parse("invalid"), None);
    }

    #[test]
    fn test_bundle_type_display_name() {
        assert_eq!(BundleType::Command.display_name(), "command");
        assert_eq!(BundleType::Tool.display_name(), "tool (Qt)");
        assert_eq!(BundleType::ToolHtml.display_name(), "tool (HTML)");
        assert_eq!(BundleType::Format.display_name(), "format");
        assert_eq!(BundleType::Fetch.display_name(), "fetch");
        assert_eq!(BundleType::Selector.display_name(), "selector");
        assert_eq!(BundleType::Preset.display_name(), "preset");
        assert_eq!(BundleType::Cpp.display_name(), "C++ extension");
    }

    #[test]
    fn test_with_type() {
        let template = BundleTemplate::with_type("my-tool", BundleType::Tool).unwrap();
        assert_eq!(template.bundle_type, BundleType::Tool);
        assert_eq!(template.bundle_name, "ChimeraX-MyTool");
    }

    #[test]
    fn test_capitalize_words_with_spaces() {
        assert_eq!(capitalize_words_with_spaces("my-tool"), "My Tool");
        assert_eq!(capitalize_words_with_spaces("my_tool"), "My Tool");
        assert_eq!(capitalize_words_with_spaces("mytool"), "Mytool");
        assert_eq!(
            capitalize_words_with_spaces("multi-word-name"),
            "Multi Word Name"
        );
    }

    #[test]
    fn test_generate_tool_creates_files() {
        let temp = TempDir::new().unwrap();
        let template = BundleTemplate::with_type("test-tool", BundleType::Tool).unwrap();

        let created = template.generate(temp.path()).unwrap();

        assert!(temp.path().join("pyproject.toml").exists());
        assert!(temp.path().join("src/__init__.py").exists());
        assert!(temp.path().join("src/tool.py").exists());
        assert!(temp.path().join("scripts/smoke.cxc").exists());
        assert!(temp.path().join("README.md").exists());
        assert_eq!(created.len(), 5);
    }

    #[test]
    fn test_generate_format_creates_files() {
        let temp = TempDir::new().unwrap();
        let template = BundleTemplate::with_type("test-format", BundleType::Format).unwrap();

        let created = template.generate(temp.path()).unwrap();

        assert!(temp.path().join("pyproject.toml").exists());
        assert!(temp.path().join("src/__init__.py").exists());
        assert!(temp.path().join("src/open.py").exists());
        assert_eq!(created.len(), 5);
    }

    #[test]
    fn test_generate_selector_creates_files() {
        let temp = TempDir::new().unwrap();
        let template = BundleTemplate::with_type("test-sel", BundleType::Selector).unwrap();

        let created = template.generate(temp.path()).unwrap();

        assert!(temp.path().join("pyproject.toml").exists());
        assert!(temp.path().join("src/__init__.py").exists());
        assert!(temp.path().join("src/selector.py").exists());
        assert_eq!(created.len(), 5);
    }

    #[test]
    fn test_render_tool_name() {
        let template = BundleTemplate::with_type("my-tool", BundleType::Tool).unwrap();
        let input = "Tool: {{tool_name}}";
        let output = template.render_template(input);
        assert_eq!(output, "Tool: My Tool");
    }

    #[test]
    fn test_generate_cpp_creates_files() {
        let temp = TempDir::new().unwrap();
        let template = BundleTemplate::with_type("test-cpp", BundleType::Cpp).unwrap();

        let created = template.generate(temp.path()).unwrap();

        // C++ templates go in src/chimerax/<package>/
        assert!(temp.path().join("pyproject.toml").exists());
        assert!(temp
            .path()
            .join("src/chimerax/testcpp/__init__.py")
            .exists());
        assert!(temp.path().join("src/chimerax/testcpp/cmd.py").exists());
        assert!(temp
            .path()
            .join("src/chimerax/testcpp/_extension.cpp")
            .exists());
        assert!(temp.path().join("scripts/smoke.cxc").exists());
        assert!(temp.path().join("README.md").exists());
        assert_eq!(created.len(), 6);
    }

    #[test]
    fn test_cpp_pyproject_contains_extension() {
        let temp = TempDir::new().unwrap();
        let template = BundleTemplate::with_type("my-ext", BundleType::Cpp).unwrap();

        template.generate(temp.path()).unwrap();

        let content = std::fs::read_to_string(temp.path().join("pyproject.toml")).unwrap();
        assert!(content.contains("[chimerax.extension._myext]"));
        assert!(content.contains("pure = false"));
        assert!(content.contains("language = \"c++\""));
    }

    #[test]
    fn test_render_pascal_name() {
        let template = BundleTemplate::new("my-tool").unwrap();
        let input = "class _{{pascal_name}}API:";
        let output = template.render_template(input);
        assert_eq!(output, "class _MyToolAPI:");
    }
}
