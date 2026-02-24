---
sidebar_position: 5
---

# test

Run tests using ChimeraX's Python environment with pytest.

## Usage

```bash
echi test [OPTIONS] [PATH] [-- <PYTEST_ARGS>...]
```

## Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `PATH` | `.` | Project directory |
| `PYTEST_ARGS` | | Additional arguments passed to pytest (after `--`) |

## Options

| Option | Description |
|--------|-------------|
| `-k, --filter <EXPR>` | Only run tests matching the given expression |
| `--pytest-verbose` | Increase pytest verbosity (adds `-v` flag) |
| `--no-build` | Skip build step |
| `--no-install` | Skip install step |
| `--coverage` | Generate coverage report |
| `--smoke` | Run smoke test (`scripts/smoke.cxc`) instead of pytest |
| `--all` | Test all bundles in workspace |

## Examples

```bash
# Run all tests in tests/
echi test

# Filter tests by expression
echi test -k test_foo

# Increase pytest verbosity
echi test --pytest-verbose

# Skip build and install steps
echi test --no-build --no-install

# Run smoke test
echi test --smoke

# Pass additional pytest args
echi test -- --cov=src
```

## Prerequisites

pytest must be installed in ChimeraX's Python environment:

```bash
echi packages install pytest
```
