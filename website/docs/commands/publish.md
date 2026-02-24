---
sidebar_position: 17
---

# echi publish

Publish bundle to the [ChimeraX Toolshed](https://cxtoolshed.rbvi.ucsf.edu/).

## Usage

```bash
echi publish [OPTIONS] [PATH]
```

## Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `PATH` | `.` | Project directory or wheel file |

## Options

| Option | Description |
|--------|-------------|
| `--dry-run` | Validate without publishing |

## Examples

```bash
# Publish current project
echi publish

# Validate without publishing
echi publish --dry-run
```
