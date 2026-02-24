# echidna (echi)

[![CI](https://github.com/N283T/echidna/actions/workflows/ci.yml/badge.svg)](https://github.com/N283T/echidna/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

ChimeraX Bundle Development CLI - A command-line tool for developing [UCSF ChimeraX](https://www.cgl.ucsf.edu/chimerax/) bundles.

> **Why "echidna"?** In Greek mythology, [Echidna](https://en.wikipedia.org/wiki/Echidna_(mythology)) is the mother of the Chimera. Just as Echidna gave birth to the Chimera, this tool helps you create ChimeraX bundles. The CLI command is `echi` for brevity.

## Features

- **Project scaffolding** - Generate a new bundle project with proper structure
- **Build automation** - Build wheel packages using ChimeraX's bundle builder
- **Quick iteration** - Build, install, and launch ChimeraX in one command
- **IDE integration** - Set up virtual environment for type checkers and IDEs
- **Testing support** - Run pytest tests using ChimeraX's Python environment
- **Validation** - Validate bundle structure and configuration
- **Cross-platform** - Works on macOS, Linux, and Windows

## Installation

### Quick Install (macOS / Linux)

```bash
curl -sSfL https://raw.githubusercontent.com/N283T/echidna/main/install.sh | sh
```

### From GitHub Releases

Download the latest binary from the [Releases](https://github.com/N283T/echidna/releases) page.

#### macOS / Linux

```bash
# Download and extract (replace TARGET with your platform)
# Targets: x86_64-unknown-linux-gnu, x86_64-apple-darwin, aarch64-apple-darwin
curl -LO https://github.com/N283T/echidna/releases/latest/download/echi-TARGET.tar.gz
tar -xzf echi-TARGET.tar.gz
sudo mv echi /usr/local/bin/
```

#### Windows

Download `echi-x86_64-pc-windows-msvc.zip` from Releases and add to PATH.

### From Source

```bash
git clone https://github.com/N283T/echidna.git
cd echidna
cargo install --path .
```

## Quick Start

```bash
# Create a new bundle project
echi init my-tool
cd my-tool

# Build, install, and launch ChimeraX
echi run
```

## Commands

### `echi init [PATH]`

Generate a new ChimeraX bundle project.

```bash
# Create in new directory
echi init my-tool

# Create in current directory with custom name
echi init --name my-tool .

# Override bundle/package names
echi init --bundle-name ChimeraX-MyTool --package chimerax.mytool .
```

### `echi build [PATH]`

Build the bundle wheel using ChimeraX's bundle builder.

```bash
echi build           # Build current directory
echi build --clean   # Clean build directory first
```

### `echi install [PATH]`

Install the bundle to ChimeraX.

```bash
echi install         # Install from current directory
echi install --user  # Install as user bundle
echi install --wheel dist/MyBundle-0.1.0-py3-none-any.whl
```

### `echi run [PATH]`

Build, install, and launch ChimeraX in one command.

```bash
echi run                     # Full cycle
echi run --no-build          # Skip build step
echi run --script test.cxc   # Run script after launch
echi run --nogui             # Run in nogui mode
```

### `echi python`

Show ChimeraX Python environment information.

```bash
echi python              # Text output
echi python --format json
```

### `echi venv [PATH]`

Set up IDE and type checker environment by creating a virtual environment that references ChimeraX's Python.

```bash
echi venv                   # Create .venv in current directory
echi venv --output .venv    # Specify output directory
echi venv --force           # Overwrite existing venv
echi venv --no-config       # Skip generating config files
echi venv --configs ty,ruff # Generate specific config files
```

This creates a `.venv` directory that IDEs and type checkers (ty, ruff, pyright) can use to resolve `chimerax` imports.

### `echi clean [PATH]`

Clean build artifacts from the project.

```bash
echi clean              # Remove build/, dist/, *.egg-info/
echi clean --all        # Also remove .venv/
echi clean --dry-run    # Show what would be deleted
```

### `echi validate [PATH]`

Validate bundle structure and configuration.

```bash
echi validate           # Validate current directory
echi validate ./my-tool # Validate specific project
```

Checks for:
- Valid `pyproject.toml` with required fields
- Correct bundle_info structure
- Required source files exist

### `echi info [PATH]`

Show bundle information and status.

```bash
echi info               # Show info for current directory
```

Displays:
- Bundle name and version
- Package name and description
- ChimeraX installation status
- Build artifacts status

### `echi test [PATH]`

Run tests using ChimeraX's Python environment with pytest.

```bash
echi test                       # Run all tests in tests/
echi test -k test_foo           # Filter tests by expression
echi test --verbose             # Increase pytest verbosity
echi test --no-build            # Skip build step
echi test --no-install          # Skip install step
echi test --smoke               # Run smoke test (scripts/smoke.cxc)
echi test -- --cov=src          # Pass additional pytest args
```

**Note:** Requires pytest installed in ChimeraX's Python environment:
```bash
echi packages install pytest
```

### `echi packages`

Manage packages in ChimeraX's Python environment.

```bash
# List installed packages
echi packages list                    # List all installed packages
echi packages list --format json      # JSON output

# Check for conflicts before installing
echi packages check numpy             # Check if numpy can be installed
echi packages check -r requirements.txt  # Check packages from file

# Install packages
echi packages install pytest          # Install single package
echi packages install numpy pandas    # Install multiple packages
echi packages install "requests>=2.28"  # With version specifier
echi packages install -U numpy        # Upgrade if already installed
echi packages install --dry-run pytest  # Preview what would be installed
```

Uses [uv](https://github.com/astral-sh/uv) if available for faster operations, otherwise falls back to pip.

### Other Commands

- `echi watch` - Watch for file changes and auto-rebuild
- `echi debug` - Launch ChimeraX in debug mode (with pdb/profiling)
- `echi version` - Manage bundle version in pyproject.toml
- `echi workspace` - Manage multi-bundle workspaces
- `echi docs` - Open ChimeraX documentation
- `echi publish` - Publish bundle to ChimeraX Toolshed

Run `echi <command> --help` for details on each command.

## Configuration

Create `echidna.toml` in your project root:

```toml
# Bundle name (e.g., "ChimeraX-MyTool")
bundle_name = "ChimeraX-MyTool"

# Python package name (e.g., "chimerax.mytool")
package_name = "chimerax.mytool"

# Path to ChimeraX executable (optional, auto-detected)
chimerax_path = "/Applications/ChimeraX.app/Contents/bin/ChimeraX"

# Default script to run on `echi run`
default_script = "scripts/test.cxc"

# Install as user bundle by default
user_install = true
```

## Project Structure

Generated bundle structure:

```
my-tool/
├── pyproject.toml      # Bundle metadata and build config
├── src/
│   ├── __init__.py     # Bundle initialization
│   └── cmd.py          # Command implementation
├── tests/              # Test files (for echi test)
│   └── test_*.py       # pytest test modules
├── scripts/
│   └── smoke.cxc       # Test script
└── README.md
```

## Requirements

- [UCSF ChimeraX](https://www.cgl.ucsf.edu/chimerax/) installed
- ChimeraX is auto-detected on:
  - macOS: `/Applications/ChimeraX*.app`
  - Linux: `/usr/bin/chimerax`, `~/.local/bin/chimerax`
  - Windows: `C:\Program Files\ChimeraX*`

## Shell Completions

Generate shell completions:

```bash
# Bash
echi completions bash > ~/.local/share/bash-completion/completions/echi

# Zsh
echi completions zsh > ~/.zfunc/_echi

# Fish
echi completions fish > ~/.config/fish/completions/echi.fish

# PowerShell
echi completions powershell > echi.ps1
```

## License

MIT License - see [LICENSE](LICENSE) for details.
