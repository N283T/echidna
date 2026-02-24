# Improve: CLI Output and Messages

## Desired Improvements

1. 成功メッセージに色（緑など）
2. コマンド成功後に次のステップを提案
3. `echidna run` 後にChimeraXコマンドのヒント
4. `smoke.cxc` テンプレートにサンプルコマンド

## Current State

成功メッセージは plain text で出力。`init` のみ next steps を表示。

## Recommended Crate: `colored`

```toml
[dependencies]
colored = "3"
```

Simple API: `"text".green()`, `"text".bold()`

## Files to Modify

| File | Change |
|------|--------|
| `Cargo.toml` | `colored` 追加 |
| `src/ui.rs` (new) | UI utilities |
| `src/lib.rs` | `pub mod ui;` 追加 |
| `src/commands/build.rs` | 色 + next steps |
| `src/commands/install.rs` | 色 + next steps |
| `src/commands/run.rs` | 色 + command hint |
| `src/commands/testing.rs` | 色 |
| `templates/common/smoke_cxc.tmpl` | サンプルコマンド |

## Implementation

### 1. New `src/ui.rs`

```rust
use colored::Colorize;

pub fn success(message: &str) {
    println!("{}", message.green().bold());
}

pub fn error(message: &str) {
    eprintln!("{}", format!("error: {}", message).red().bold());
}

pub fn hint(message: &str) {
    println!("{}", message.dimmed());
}

pub fn next_steps(steps: &[(&str, &str)]) {
    println!();
    println!("{}", "Next steps:".cyan().bold());
    for (cmd, desc) in steps {
        println!("  {}  {}", cmd.cyan(), format!("# {}", desc).dimmed());
    }
}
```

### 2. Update `build.rs`

```rust
use crate::ui;

pub fn execute(args: BuildArgs) -> Result<()> {
    // ... existing code ...

    ui::success("Build successful!");
    println!("Wheel: {}", wheel.display());

    ui::next_steps(&[
        ("echidna install", "Install the bundle to ChimeraX"),
        ("echidna run", "Build, install, and launch ChimeraX"),
    ]);

    Ok(())
}
```

### 3. Update `install.rs`

```rust
use crate::ui;

pub fn execute(args: InstallArgs) -> Result<()> {
    // ... existing code ...

    ui::success("Installation successful!");
    println!("The bundle is now available in ChimeraX.");

    ui::next_steps(&[
        ("echidna run", "Launch ChimeraX with the bundle"),
    ]);

    Ok(())
}
```

### 4. Update `run.rs`

```rust
use crate::ui;

pub fn execute(args: RunArgs) -> Result<()> {
    // ... existing code ...

    ui::success("ChimeraX launched.");

    // Bundle command hint
    if let Some(command_name) = get_bundle_command_name(&project_dir) {
        println!();
        ui::hint(&format!(
            "Try running '{}' in the ChimeraX command line",
            command_name
        ));
    }

    Ok(())
}

fn get_bundle_command_name(project_dir: &Path) -> Option<String> {
    let pyproject_path = project_dir.join("pyproject.toml");
    let content = std::fs::read_to_string(pyproject_path).ok()?;
    let pyproject: toml::Value = toml::from_str(&content).ok()?;

    let chimerax = pyproject.get("chimerax")?;

    if let Some(commands) = chimerax.get("command") {
        if let Some(table) = commands.as_table() {
            return table.keys().next().map(|s| s.to_string());
        }
    }

    None
}
```

### 5. Update `smoke_cxc.tmpl`

```
# Smoke test for {{bundle_name}}
# Run with: ChimeraX --script scripts/smoke.cxc
# Or use: echidna run

# Test the bundle's command
{{command_name}} "Hello from {{bundle_name}}!"

# Optional: Load a sample structure
# open 1a0a

# Display session info
info session
```

## Verification

```bash
# Build
cargo build

# Test colors
echidna build
echidna install
echidna run

# Test NO_COLOR support
NO_COLOR=1 echidna build

# Test new template
cd /tmp && echidna init test-bundle
cat test-bundle/scripts/smoke.cxc
```
