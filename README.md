# echidna

[![CI](https://github.com/N283T/echidna/actions/workflows/ci.yml/badge.svg)](https://github.com/N283T/echidna/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

ChimeraX Bundle Development CLI - A command-line tool for developing [UCSF ChimeraX](https://www.cgl.ucsf.edu/chimerax/) bundles.

## Features

- **Project scaffolding** - Generate a new bundle project with proper structure
- **Build automation** - Build wheel packages using ChimeraX's bundle builder
- **Quick iteration** - Build, install, and launch ChimeraX in one command
- **Cross-platform** - Works on macOS, Linux, and Windows

## Installation

### From GitHub Releases

Download the latest binary from the [Releases](https://github.com/N283T/echidna/releases) page.

#### macOS / Linux

```bash
# Download and extract (replace VERSION and TARGET)
curl -LO https://github.com/N283T/echidna/releases/latest/download/echidna-TARGET.tar.gz
tar -xzf echidna-TARGET.tar.gz
sudo mv echidna /usr/local/bin/
```

#### Windows

Download `echidna-x86_64-pc-windows-msvc.zip` from Releases and add to PATH.

### From Source

```bash
git clone https://github.com/N283T/echidna.git
cd echidna
cargo install --path .
```

## Quick Start

```bash
# Create a new bundle project
echidna init my-tool
cd my-tool

# Build, install, and launch ChimeraX
echidna run
```

## Commands

### `echidna init [PATH]`

Generate a new ChimeraX bundle project.

```bash
# Create in new directory
echidna init my-tool

# Create in current directory with custom name
echidna init --name my-tool .

# Override bundle/package names
echidna init --bundle-name ChimeraX-MyTool --package chimerax.mytool .
```

### `echidna build [PATH]`

Build the bundle wheel using ChimeraX's bundle builder.

```bash
echidna build           # Build current directory
echidna build --clean   # Clean build directory first
```

### `echidna install [PATH]`

Install the bundle to ChimeraX.

```bash
echidna install         # Install from current directory
echidna install --user  # Install as user bundle
echidna install --wheel dist/MyBundle-0.1.0-py3-none-any.whl
```

### `echidna run [PATH]`

Build, install, and launch ChimeraX in one command.

```bash
echidna run                     # Full cycle
echidna run --no-build          # Skip build step
echidna run --script test.cxc   # Run script after launch
echidna run --nogui             # Run in nogui mode
```

### `echidna python`

Show ChimeraX Python environment information.

```bash
echidna python              # Text output
echidna python --format json
```

## Configuration

Create `echidna.toml` in your project root:

```toml
# Bundle name (e.g., "ChimeraX-MyTool")
bundle_name = "ChimeraX-MyTool"

# Python package name (e.g., "chimerax.mytool")
package_name = "chimerax.mytool"

# Path to ChimeraX executable (optional, auto-detected)
chimerax_path = "/Applications/ChimeraX.app/Contents/bin/ChimeraX"

# Default script to run on `echidna run`
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
echidna completions bash > ~/.local/share/bash-completion/completions/echidna

# Zsh
echidna completions zsh > ~/.zfunc/_echidna

# Fish
echidna completions fish > ~/.config/fish/completions/echidna.fish

# PowerShell
echidna completions powershell > echidna.ps1
```

## License

MIT License - see [LICENSE](LICENSE) for details.
