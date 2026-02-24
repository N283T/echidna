# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Add Docusaurus documentation site at `website/` (deployed to GitHub Pages)
- Add GitHub Actions workflow for automatic docs deployment on merge to main
- Add `homepage` field to `Cargo.toml`

### Changed
- Slim down README.md to overview + links (full docs moved to website)
- Narrow CI paths filter to avoid docs changes triggering Rust checks

### Fixed
- Fix CONTRIBUTING.md Rust version requirement (1.70+ -> 1.85+ for edition 2024)
- Fix README.md and config.rs ChimeraX path (Contents/bin -> Contents/MacOS)
- Fix install.sh branch reference (master -> main)
- Fix Linux detection paths: add DEB/RPM binary paths, remove incorrect /opt path
- Add Windows detection for older versioned install directories (e.g., "ChimeraX 1.3")
- Show helpful message when ChimeraX auto-detection fails (including Flatpak hint)

### Changed
- Unify OutputFormat enum into shared type in lib.rs (was triplicated)
- Replace raw ANSI escape codes in validator.rs with colored crate
- Replace dirs crate with directories crate (redundant dependency)
- Split main.rs into cli.rs (CLI definitions) and main.rs (routing logic)
- Use data-driven template generation in bundle.rs (reduce repetition)
- Use type-specific smoke test templates (generic for non-command bundles)

### Removed
- Remove unused check_packages() method from PackageResolver
- Remove unused SaveTarget::Project variant
- Remove incomplete --pdb and --profile flags from debug command
- Clean up 13 stale branches (local and remote)

[Unreleased]: https://github.com/N283T/echidna/commits/main
