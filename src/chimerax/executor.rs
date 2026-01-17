//! ChimeraX command execution.

use crate::error::{EchidnaError, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// Validate that a path is safe for use in ChimeraX commands.
/// Rejects paths containing characters that could be interpreted specially.
fn validate_path_for_command(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    // Characters that could cause issues in ChimeraX command strings
    // Note: Backslash is excluded on Windows since it's the native path separator
    #[cfg(unix)]
    const DANGEROUS_CHARS: &[char] = &['"', '\'', ';', '\n', '\r', '`', '$', '\\'];

    #[cfg(windows)]
    const DANGEROUS_CHARS: &[char] = &['"', '\'', ';', '\n', '\r', '`', '$'];

    for ch in DANGEROUS_CHARS {
        if path_str.contains(*ch) {
            return Err(EchidnaError::InvalidName(format!(
                "path contains invalid character '{}': {}",
                ch, path_str
            )));
        }
    }

    Ok(())
}

/// Verbosity levels for output.
/// - 0: quiet (errors only)
/// - 1: normal (-v, show commands)
/// - 2: verbose (-vv, show commands + output)
/// - 3+: debug (-vvv, show everything)
pub type Verbosity = u8;

/// Wrapper for executing ChimeraX commands.
pub struct ChimeraXExecutor {
    executable: PathBuf,
    verbosity: Verbosity,
}

impl ChimeraXExecutor {
    /// Create a new executor with the given ChimeraX executable path.
    pub fn new(executable: PathBuf, verbosity: Verbosity) -> Self {
        Self {
            executable,
            verbosity,
        }
    }

    /// Get the path to the ChimeraX executable.
    pub fn executable(&self) -> &PathBuf {
        &self.executable
    }

    /// Execute a ChimeraX command in nogui mode.
    pub fn run_command(&self, cmd: &str) -> Result<Output> {
        self.log_execution(&format!("ChimeraX --nogui --exit --cmd '{}'", cmd));

        let output = Command::new(&self.executable)
            .args(["--nogui", "--exit", "--cmd", cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        self.log_output(&output);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(EchidnaError::ChimeraXCommandFailed(format!(
                "exit code: {:?}\nstdout: {}\nstderr: {}",
                output.status.code(),
                stdout,
                stderr
            )));
        }

        Ok(output)
    }

    /// Execute a ChimeraX script in nogui mode.
    pub fn run_script(&self, script: &Path) -> Result<Output> {
        let script_str = script.to_string_lossy();
        self.log_execution(&format!(
            "ChimeraX --nogui --exit --script '{}'",
            script_str
        ));

        let output = Command::new(&self.executable)
            .args(["--nogui", "--exit", "--script", &script_str])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        self.log_output(&output);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(EchidnaError::ChimeraXCommandFailed(format!(
                "exit code: {:?}\nstdout: {}\nstderr: {}",
                output.status.code(),
                stdout,
                stderr
            )));
        }

        Ok(output)
    }

    /// Launch ChimeraX with GUI (optionally with a script).
    pub fn launch(&self, script: Option<&Path>) -> Result<()> {
        let mut cmd = Command::new(&self.executable);

        if let Some(script) = script {
            cmd.args(["--script", &script.to_string_lossy()]);
        }

        self.log_execution(&format!(
            "ChimeraX{}",
            script
                .map(|s| format!(" --script '{}'", s.display()))
                .unwrap_or_default()
        ));

        cmd.spawn()?;
        Ok(())
    }

    /// Execute `devel build` command.
    pub fn devel_build(&self, path: &Path) -> Result<Output> {
        validate_path_for_command(path)?;
        let cmd = format!("devel build \"{}\" exit true", path.display());
        self.run_command(&cmd)
    }

    /// Execute `devel install` command.
    pub fn devel_install(&self, path: &Path, user: bool) -> Result<Output> {
        validate_path_for_command(path)?;
        let user_flag = if user { "user true" } else { "user false" };
        let cmd = format!(
            "devel install \"{}\" {} exit true",
            path.display(),
            user_flag
        );
        self.run_command(&cmd)
    }

    /// Execute `toolshed install` command.
    pub fn toolshed_install(&self, wheel: &Path, user: bool) -> Result<Output> {
        validate_path_for_command(wheel)?;
        let user_flag = if user { " user true" } else { "" };
        let cmd = format!("toolshed install \"{}\"{}", wheel.display(), user_flag);
        self.run_command(&cmd)
    }

    /// Get Python environment information from ChimeraX.
    pub fn get_python_info(&self) -> Result<PythonInfo> {
        let python_code = r#"
import sys
import json
info = {
    "executable": sys.executable,
    "version": sys.version,
    "prefix": sys.prefix,
    "path": sys.path,
}
try:
    import chimerax
    info["chimerax_version"] = getattr(chimerax, "__version__", "unknown")
except Exception:
    info["chimerax_version"] = None
try:
    import site
    info["site_packages"] = site.getsitepackages()
except Exception:
    info["site_packages"] = []
print("ECHIDNA_JSON_START")
print(json.dumps(info))
print("ECHIDNA_JSON_END")
"#;
        // Escape for shell
        let escaped = python_code.replace('\n', "\\n").replace('"', "\\\"");
        let cmd = format!("runscript python -c \"exec(\\\"{}\\\")\"; exit", escaped);

        let output = self.run_command(&cmd)?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract JSON from output
        let start_marker = "ECHIDNA_JSON_START";
        let end_marker = "ECHIDNA_JSON_END";

        let start = stdout
            .find(start_marker)
            .ok_or_else(|| EchidnaError::ChimeraXCommandFailed("JSON output not found".into()))?;
        let end = stdout
            .find(end_marker)
            .ok_or_else(|| EchidnaError::ChimeraXCommandFailed("JSON output not found".into()))?;

        let json_str = stdout[start + start_marker.len()..end].trim();
        let info: PythonInfo = serde_json::from_str(json_str)?;

        Ok(info)
    }

    fn log_execution(&self, msg: &str) {
        // Level 1+: show commands being executed
        if self.verbosity >= 1 {
            eprintln!("[echidna] Executing: {}", msg);
        }
    }

    fn log_output(&self, output: &Output) {
        // Level 2+: show command output
        if self.verbosity >= 2 {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stdout.is_empty() {
                eprintln!("[echidna] stdout:\n{}", stdout);
            }
            if !stderr.is_empty() {
                eprintln!("[echidna] stderr:\n{}", stderr);
            }
        }
    }
}

/// Python environment information from ChimeraX.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PythonInfo {
    pub executable: String,
    pub version: String,
    pub prefix: String,
    pub path: Vec<String>,
    pub chimerax_version: Option<String>,
    #[serde(default)]
    pub site_packages: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_normal() {
        let path = Path::new("/Users/test/my-project");
        assert!(validate_path_for_command(path).is_ok());
    }

    #[test]
    fn test_validate_path_with_spaces() {
        let path = Path::new("/Users/test/my project");
        assert!(validate_path_for_command(path).is_ok());
    }

    #[test]
    fn test_validate_path_rejects_quotes() {
        let path = Path::new("/Users/test/my\"project");
        assert!(validate_path_for_command(path).is_err());

        let path = Path::new("/Users/test/my'project");
        assert!(validate_path_for_command(path).is_err());
    }

    #[test]
    fn test_validate_path_rejects_semicolon() {
        let path = Path::new("/Users/test/my;project");
        assert!(validate_path_for_command(path).is_err());
    }

    #[test]
    fn test_validate_path_rejects_newlines() {
        let path = Path::new("/Users/test/my\nproject");
        assert!(validate_path_for_command(path).is_err());
    }

    #[test]
    fn test_validate_path_rejects_backtick() {
        let path = Path::new("/Users/test/my`project");
        assert!(validate_path_for_command(path).is_err());
    }

    #[test]
    fn test_validate_path_rejects_dollar() {
        let path = Path::new("/Users/test/$HOME/project");
        assert!(validate_path_for_command(path).is_err());
    }

    #[test]
    fn test_executor_new() {
        let executor = ChimeraXExecutor::new(PathBuf::from("/usr/bin/chimerax"), 0);
        assert_eq!(executor.verbosity, 0);
    }

    #[test]
    fn test_verbosity_levels() {
        // Level 0: quiet
        let executor = ChimeraXExecutor::new(PathBuf::from("/test"), 0);
        assert_eq!(executor.verbosity, 0);

        // Level 1: normal
        let executor = ChimeraXExecutor::new(PathBuf::from("/test"), 1);
        assert_eq!(executor.verbosity, 1);

        // Level 2: verbose
        let executor = ChimeraXExecutor::new(PathBuf::from("/test"), 2);
        assert_eq!(executor.verbosity, 2);
    }
}
