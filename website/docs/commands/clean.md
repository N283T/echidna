---
sidebar_position: 9
---

# echi clean

Clean build artifacts from the project.

## Usage

```bash
echi clean [OPTIONS] [PATH]
```

## Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `PATH` | `.` | Project directory |

## Options

| Option | Description |
|--------|-------------|
| `--all` | Also remove `.venv/` directory |
| `--dry-run` | Show what would be deleted without actually deleting |

## Examples

```bash
# Remove build/, dist/, *.egg-info/
echi clean

# Also remove .venv/
echi clean --all

# Preview what would be deleted
echi clean --dry-run
```
