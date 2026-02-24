---
sidebar_position: 2
---

# Quick Start

## Create Your First Bundle

```bash
# Create a new bundle project
echi init my-tool
cd my-tool
```

This generates a complete ChimeraX bundle project with the following structure:

```
my-tool/
├── pyproject.toml      # Bundle metadata and build config
├── src/
│   ├── __init__.py     # Bundle initialization
│   └── cmd.py          # Command implementation
├── scripts/
│   └── smoke.cxc       # Test script
└── README.md
```

## Build and Run

```bash
# Build, install, and launch ChimeraX in one command
echi run
```

This single command:
1. Builds the bundle wheel using ChimeraX's bundle builder
2. Installs it into ChimeraX
3. Launches ChimeraX with the bundle loaded

## Iterate on Your Code

Use watch mode to automatically rebuild when you save changes:

```bash
echi watch --run
```

## Set Up IDE Support

Create a virtual environment that enables type checking and autocompletion for `chimerax` imports:

```bash
echi venv
```

This works with type checkers like [ty](https://github.com/astral-sh/ty), [ruff](https://github.com/astral-sh/ruff), and [pyright](https://github.com/microsoft/pyright).

## Run Tests

```bash
# Install pytest first
echi packages install pytest

# Run your tests
echi test
```

## Next Steps

- Learn about different [bundle types](../guides/bundle-types.md) (command, tool, format, etc.)
- Explore [all commands](../commands/init.md)
- Configure your project with [echidna.toml](../configuration/echidna-toml.md)
