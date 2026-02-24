---
sidebar_position: 10
---

# validate

Validate bundle structure and configuration.

## Usage

```bash
echi validate [OPTIONS] [PATH]
```

## Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `PATH` | `.` | Project directory |

## Options

| Option | Description |
|--------|-------------|
| `--strict` | Treat warnings as errors |

## Examples

```bash
# Validate current directory
echi validate

# Validate specific project
echi validate ./my-tool

# Strict mode
echi validate --strict
```

## What It Checks

- Valid `pyproject.toml` with required ChimeraX fields
- Correct `bundle_info` structure
- Required source files exist
