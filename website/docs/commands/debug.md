---
sidebar_position: 7
---

# debug

Launch ChimeraX in debug mode.

## Usage

```bash
echi debug [OPTIONS] [PATH]
```

## Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `PATH` | `.` | Project directory |

## Options

| Option | Description |
|--------|-------------|
| `--no-build` | Skip build step |
| `--no-install` | Skip install step |

## Examples

```bash
# Launch in debug mode
echi debug

# Skip build
echi debug --no-build
```
