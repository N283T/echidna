---
sidebar_position: 1
---

# init

Generate a new ChimeraX bundle project.

## Usage

```bash
echi init [OPTIONS] [PATH]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `PATH` | Project name or path. Creates the directory if it doesn't exist. If omitted, initializes in the current directory. |

## Options

| Option | Description |
|--------|-------------|
| `-n, --name <NAME>` | Override the project name (defaults to directory name) |
| `-t, --type <TYPE>` | Bundle type (default: `command`). See [Bundle Types](../guides/bundle-types.md) |
| `--bundle-name <NAME>` | Bundle name (e.g., `ChimeraX-MyTool`) |
| `--package <NAME>` | Python package name (e.g., `chimerax.mytool`) |
| `-f, --force` | Overwrite existing files |

## Examples

```bash
# Create in new directory
echi init my-tool

# Create in current directory with custom name
echi init --name my-tool .

# Create a Qt GUI tool
echi init --type tool my-gui-tool

# Override bundle/package names
echi init --bundle-name ChimeraX-MyTool --package chimerax.mytool .
```

## Name Derivation

When you run `echi init my-tool`, the following names are derived automatically:

| Field | Value |
|-------|-------|
| Bundle name | `ChimeraX-MyTool` |
| Package name | `chimerax.mytool` |
| Command name | `my_tool` |
| Tool name | `My Tool` (for GUI tools) |

You can override these with `--bundle-name` and `--package`.
