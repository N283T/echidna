---
sidebar_position: 3
---

# install

Install the bundle to ChimeraX.

## Usage

```bash
echi install [OPTIONS] [PATH]
```

## Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `PATH` | `.` | Project directory or wheel file |

## Options

| Option | Description |
|--------|-------------|
| `-w, --wheel <FILE>` | Specific wheel file to install |
| `--user` | Install as user bundle |

## Examples

```bash
# Install from current directory
echi install

# Install as user bundle
echi install --user

# Install a specific wheel file
echi install --wheel dist/MyBundle-0.1.0-py3-none-any.whl
```
