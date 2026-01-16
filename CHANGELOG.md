# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/nagaet/echidna/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/nagaet/echidna/releases/tag/v0.1.0
