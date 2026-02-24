# Fix: `echidna setup-ide` Error

## Problem
`echidna setup-ide` 実行時にエラー:
```
ChimeraX command failed: exit code: Some(70)
chimerax.core.errors.UserError: Missing or invalid "scriptFile" argument: File 'python' does not exist
```

## Root Cause
コードが `runscript python -c "..."` という構文を使用しているが、ChimeraXの`runscript`コマンドは`.py`ファイルへのパスを期待している。`python -c`スタイルのインラインコードは使えない。

## Solution
ChimeraXは `-c` フラグをコマンドライン最初の引数として受け付け、Pythonコードを直接実行できる:
```
ChimeraX -c "python_code"
```
これは`--nogui`と`--silent`を暗黙的に含む。

## Affected Files

| File | Issue |
|------|-------|
| `src/chimerax/executor.rs` | `get_python_info()` method (line 188) |
| `src/chimerax/detect.rs` | `get_chimerax_version()` function (line 117) |
| `src/commands/testing.rs` | test execution (line 161) |
| `src/commands/info.rs` | `check_bundle_installed()` function (line 216) |
| `src/commands/debug.rs` | pdb setup (line 95) |

## Implementation

### Approach: Use ChimeraX's `-c` flag directly

Add a new method to `ChimeraXExecutor`:

```rust
/// Run Python code directly using ChimeraX's -c flag.
/// This implies --nogui and --silent.
pub fn run_python_code(&self, code: &str) -> Result<std::process::Output> {
    let output = std::process::Command::new(&self.chimerax_path)
        .arg("-c")
        .arg(code)
        .output()
        .map_err(|e| EchidnaError::ChimeraXExecution(e.to_string()))?;

    Ok(output)
}
```

### Update `get_python_info()` in `executor.rs`

**Before:**
```rust
pub fn get_python_info(&self) -> Result<PythonInfo> {
    let cmd = format!(
        "runscript python -c \"exec(\\\"{}\\\")\"; exit",
        PYTHON_INFO_SCRIPT.replace('\n', "\\n").replace('"', "\\\"")
    );
    // ...
}
```

**After:**
```rust
pub fn get_python_info(&self) -> Result<PythonInfo> {
    let output = self.run_python_code(PYTHON_INFO_SCRIPT)?;
    // Parse output...
}
```

### Update `get_chimerax_version()` in `detect.rs`

Use the same `-c` flag approach.

## Verification

```bash
cargo build
echidna setup-ide --help
echidna setup-ide  # Should complete without error
ls .venv/          # Should exist
cat ty.toml        # Should have correct config
```

## Alternative Approach (if -c doesn't work)

Write Python code to a temporary file and execute via `--script`:

```rust
pub fn run_python_code(&self, code: &str) -> Result<std::process::Output> {
    let temp_file = tempfile::NamedTempFile::with_suffix(".py")?;
    std::fs::write(temp_file.path(), code)?;

    let output = std::process::Command::new(&self.chimerax_path)
        .arg("--nogui")
        .arg("--silent")
        .arg("--script")
        .arg(temp_file.path())
        .output()?;

    Ok(output)
}
```

Note: `tempfile` is currently dev-only dependency; would need to move to main dependencies.
