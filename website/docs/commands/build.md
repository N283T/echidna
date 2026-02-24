---
sidebar_position: 2
---

# build

Build the bundle wheel using ChimeraX's bundle builder.

## Usage

```bash
echi build [OPTIONS] [PATH]
```

## Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `PATH` | `.` | Project directory |

## Options

| Option | Description |
|--------|-------------|
| `--clean` | Clean build directory before building |
| `--all` | Build all bundles in workspace |

## Examples

```bash
# Build current directory
echi build

# Clean build first
echi build --clean

# Build a specific project
echi build ./my-tool

# Build all workspace members
echi build --all
```
