---
sidebar_position: 4
---

# run

Build, install, and launch ChimeraX in one command. This is the primary development workflow command.

## Usage

```bash
echi run [OPTIONS] [PATH]
```

## Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `PATH` | `.` | Project directory |

## Options

| Option | Description |
|--------|-------------|
| `-s, --script <FILE>` | Script to execute after launch (`.cxc` file) |
| `--no-build` | Skip build step |
| `--no-install` | Skip install step |
| `--nogui` | Run in nogui mode |

## Examples

```bash
# Full cycle: build, install, launch
echi run

# Skip build (install and launch only)
echi run --no-build

# Run a script after launch
echi run --script test.cxc

# Run without GUI
echi run --nogui
```

## Default Script

If you have `default_script` set in [echidna.toml](../configuration/echidna-toml.md), it will be executed automatically on launch unless you specify a different script with `--script`.
