---
sidebar_position: 14
---

# version

Manage bundle version in `pyproject.toml`.

## Usage

```bash
echi version [PATH] [ACTION]
```

## Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `PATH` | `.` | Project directory |
| `ACTION` | `show` | Version action: `show`, `patch`, `minor`, `major`, or a specific version like `1.2.3` |

## Examples

```bash
# Show current version
echi version

# Bump patch version (0.1.0 -> 0.1.1)
echi version patch

# Bump minor version (0.1.0 -> 0.2.0)
echi version minor

# Bump major version (0.1.0 -> 1.0.0)
echi version major

# Set specific version
echi version 2.0.0
```
