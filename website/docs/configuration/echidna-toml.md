---
sidebar_position: 1
---

# echidna.toml

Create an `echidna.toml` file in your project root to configure echidna.

## Example

```toml
# Bundle name (e.g., "ChimeraX-MyTool")
bundle_name = "ChimeraX-MyTool"

# Python package name (e.g., "chimerax.mytool")
package_name = "chimerax.mytool"

# Path to ChimeraX executable (optional, auto-detected)
chimerax_path = "/Applications/ChimeraX.app/Contents/MacOS/ChimeraX"

# Default script to run on `echi run`
default_script = "scripts/test.cxc"

# Install as user bundle by default
user_install = true
```

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bundle_name` | string | No | Bundle name (e.g., `ChimeraX-MyTool`) |
| `package_name` | string | No | Python package name (e.g., `chimerax.mytool`) |
| `chimerax_path` | string | No | Path to ChimeraX executable. Overrides auto-detection. |
| `default_script` | string | No | Default `.cxc` script to run with `echi run` |
| `user_install` | boolean | No | Install as user bundle by default (default: `false`) |

All fields are optional. If not specified, echidna uses sensible defaults or auto-detects values.

## Config File Resolution

echidna searches for `echidna.toml` starting from the current directory and walking up parent directories. This means you can place the config file at the root of a monorepo and it will be found from any subdirectory.

## Global Config

echidna also supports a global configuration file at the platform-specific config directory:

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/echidna/config.toml` |
| Linux | `~/.config/echidna/config.toml` |
| Windows | `C:\Users\<User>\AppData\Roaming\echidna\config.toml` |

The global config stores:
- `chimerax_path` - Cached ChimeraX executable path
- `chimerax_version` - Cached ChimeraX version string

Project-level `echidna.toml` settings take priority over global config.
