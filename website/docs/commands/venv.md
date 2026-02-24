---
sidebar_position: 8
---

# venv

Set up a virtual environment with ChimeraX Python for IDE support. This enables type checking and autocompletion for `chimerax` imports in your editor.

## Usage

```bash
echi venv [OPTIONS] [PATH]
```

## Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `PATH` | `.` | Project directory |

## Options

| Option | Description |
|--------|-------------|
| `-o, --output <DIR>` | Output directory for venv (default: `.venv`) |
| `-f, --force` | Force overwrite existing venv |
| `--no-config` | Skip generating type checker config files |
| `--configs <LIST>` | Config files to generate, comma-separated (e.g., `ty,ruff`) |

## Examples

```bash
# Create .venv in current directory
echi venv

# Specify output directory
echi venv --output .venv

# Overwrite existing venv
echi venv --force

# Skip config file generation
echi venv --no-config

# Generate specific config files only
echi venv --configs ty,ruff
```

## What It Does

1. Creates a virtual environment that references ChimeraX's Python interpreter
2. Generates config files for type checkers (`ty.toml`, `ruff.toml`) so they can resolve `chimerax.*` imports

This allows IDEs like VS Code, PyCharm, and editors using LSP to provide autocompletion and type checking for ChimeraX APIs.
