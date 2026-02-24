# echidna (echi)

[![CI](https://github.com/N283T/echidna/actions/workflows/ci.yml/badge.svg)](https://github.com/N283T/echidna/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

ChimeraX Bundle Development CLI - A command-line tool for developing [UCSF ChimeraX](https://www.cgl.ucsf.edu/chimerax/) bundles.

> **Why "echidna"?** In Greek mythology, [Echidna](https://en.wikipedia.org/wiki/Echidna_(mythology)) is the mother of the Chimera. Just as Echidna gave birth to the Chimera, this tool helps you create ChimeraX bundles. The CLI command is `echi` for brevity.

## Features

- **Project scaffolding** - Generate a new bundle project with proper structure (8 bundle types)
- **Build automation** - Build wheel packages using ChimeraX's bundle builder
- **Quick iteration** - Build, install, and launch ChimeraX in one command
- **IDE integration** - Set up virtual environment for type checkers and IDEs
- **Testing support** - Run pytest tests using ChimeraX's Python environment
- **Package management** - Install, check, and list packages in ChimeraX's Python
- **Cross-platform** - Works on macOS, Linux, and Windows

## Quick Start

```bash
# Install
curl -sSfL https://raw.githubusercontent.com/N283T/echidna/main/install.sh | sh

# Create a new bundle project
echi init my-tool
cd my-tool

# Build, install, and launch ChimeraX
echi run
```

## Commands

| Command | Description |
|---------|-------------|
| `echi init` | Generate a new ChimeraX bundle project |
| `echi build` | Build the bundle wheel |
| `echi install` | Install the bundle to ChimeraX |
| `echi run` | Build, install, and launch ChimeraX |
| `echi test` | Run tests using ChimeraX Python |
| `echi watch` | Watch for changes and auto-rebuild |
| `echi debug` | Launch ChimeraX in debug mode |
| `echi venv` | Set up IDE/type checker environment |
| `echi clean` | Clean build artifacts |
| `echi validate` | Validate bundle structure |
| `echi info` | Show bundle information |
| `echi python` | Show ChimeraX Python info |
| `echi packages` | Manage ChimeraX Python packages |
| `echi version` | Manage bundle version |
| `echi workspace` | Manage multi-bundle workspaces |
| `echi docs` | Open ChimeraX documentation |
| `echi publish` | Publish bundle to ChimeraX Toolshed |
| `echi completions` | Generate shell completions |

Run `echi <command> --help` for details on each command.

## Documentation

Full documentation is available at **[n283t.github.io/echidna](https://n283t.github.io/echidna/)**.

## Requirements

- [UCSF ChimeraX](https://www.cgl.ucsf.edu/chimerax/) installed (auto-detected on macOS, Linux, and Windows)

## License

MIT License - see [LICENSE](LICENSE) for details.
