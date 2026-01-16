# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-01-17

### Added

- `echidna test` - Run tests using ChimeraX Python environment
  - Runs pytest within ChimeraX's Python interpreter
  - `-k` option for test filtering
  - `--verbose` flag for detailed output
  - `--no-build` and `--no-install` flags to skip build/install steps
  - Pass additional pytest arguments after `--`

## [0.3.0] - 2026-01-17

### Added

- `echidna validate` - Validate bundle structure and configuration
  - Checks pyproject.toml for required sections ([build-system], [project], [chimerax])
  - Validates presence of ChimeraX-BundleBuilder in build requirements
  - Validates source directory structure (src/__init__.py)
  - Reports errors and warnings for configuration issues
- `echidna info` - Show bundle information and status
  - Displays bundle name, package name, version, and description
  - Shows build status (latest wheel, build time)
  - Shows ChimeraX installation status if ChimeraX is available

## [0.2.0] - 2026-01-17

### Added

- `echidna setup-ide` - Set up IDE/type checker environment with venv pointing to ChimeraX Python
  - Creates `.venv` directory with symlinks to ChimeraX's Python
  - Generates `ty.toml` and ruff configuration by default
  - `--configs` flag to select which configs to generate (ty, ruff)
  - `--force` flag to overwrite existing venv
  - `--no-config` flag to skip configuration file generation
- `echidna clean` - Clean build artifacts
  - Removes `build/`, `dist/`, `*.egg-info/`, and `__pycache__/` directories
  - `--all` flag to also remove `.venv`
  - `--dry-run` flag to preview what would be deleted

## [0.1.0] - 2025-01-16

### Added

- `echidna init` - Generate a new ChimeraX bundle project with proper structure
- `echidna build` - Build the bundle wheel using ChimeraX's bundle builder
- `echidna install` - Install the bundle to ChimeraX
- `echidna run` - Build, install, and launch ChimeraX in one command
- `echidna python` - Show ChimeraX Python environment information
- Configuration file support (`echidna.toml`)
- Auto-detection of ChimeraX installation on macOS, Linux, and Windows
- Template generation for pyproject.toml, __init__.py, cmd.py, smoke.cxc, and README.md

[Unreleased]: https://github.com/nagaet/echidna/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/nagaet/echidna/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/nagaet/echidna/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/nagaet/echidna/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/nagaet/echidna/releases/tag/v0.1.0
