---
sidebar_position: 6
---

# echi watch

Watch for file changes and auto-rebuild.

## Usage

```bash
echi watch [OPTIONS] [PATH]
```

## Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `PATH` | `.` | Project directory |

## Options

| Option | Description |
|--------|-------------|
| `--run` | Also launch ChimeraX after build |
| `--test` | Run tests on changes (mutually exclusive with `--run`) |

## Examples

```bash
# Watch and rebuild on changes
echi watch

# Watch, rebuild, and launch ChimeraX
echi watch --run

# Watch and run tests on changes
echi watch --test
```
