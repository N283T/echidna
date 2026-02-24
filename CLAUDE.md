# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

**echidna** is a Rust CLI tool for developing UCSF ChimeraX bundles (extensions). Named after Echidna, the mother of Chimera in Greek mythology.

### Features
- Project scaffolding (`init`)
- Build automation (`build`, `run`, `watch`)
- Testing (`test`)
- IDE integration (`venv`)
- Package management (`packages list/check/install`)
- Publishing (`publish`)

## Development Flow (IMPORTANT)

### Standard Workflow

```
1. Create feature branch
2. Make changes
3. Create PR
4. Check CI status (gh pr checks)
5. Run review (code-reviewer agent)
6. Fix issues if any
7. Repeat 4-6 until CI passes AND review approves
8. ASK USER before merging
```

### Key Rules

- **Always create PR** - Never push directly to master (except tiny doc fixes)
- **Always check CI** - Wait for all checks to pass
- **Always review** - Use code-reviewer agent for code changes
- **Always ask before merge** - User confirms the merge
- **Small fixes** - Fix directly, push, re-check CI
- **Large changes** - Create plan in `plans/` first, discuss approach, then implement

### Plans Directory

`plans/` contains implementation plans and improvement ideas:
- Create `.md` file before starting large features
- Document approach, affected files, considerations
- Reference in PR description
- Mark as completed when done

Example: `plans/uv-integration.md`, `plans/next-steps.md`

## Build Commands

```bash
cargo build                    # Build debug version
cargo build --release          # Build release version
cargo install --path .         # Install locally
```

## Testing

```bash
cargo test                     # Run all tests
cargo test test_name           # Run specific test
cargo test -- --nocapture      # Run with output visible
```

**Note**: Some integration tests require ChimeraX installed.

## Code Quality

Run before committing:
```bash
cargo fmt                      # Format code
cargo clippy -- -D warnings    # Lint (warnings as errors)
```

## Architecture

### Module Structure (`src/`)

```
src/
├── main.rs          # CLI entry point (clap), routes to commands
├── lib.rs           # Library root
├── config.rs        # echidna.toml parsing
├── error.rs         # EchidnaError enum (thiserror)
├── workspace.rs     # Multi-bundle workspace support
├── chimerax/        # ChimeraX detection and execution
│   ├── detect.rs    # Auto-detection (macOS/Linux/Windows)
│   ├── executor.rs  # Subprocess execution
│   └── validator.rs # Path validation
├── commands/        # Command implementations
│   ├── build.rs, init.rs, run.rs, test.rs, ...
│   └── packages.rs  # packages list/check/install
├── packages/        # Package management
│   ├── mod.rs       # Types, shared utilities
│   ├── resolver.rs  # List/check with uv/pip backend
│   └── installer.rs # Install with uv/pip backend
├── templates/       # Bundle template generation
│   └── bundle.rs    # BundleType enum, rendering
└── venv/            # IDE integration
    ├── builder.rs   # Venv creation
    └── configs.rs   # ty.toml, ruff.toml generation
```

### Template Files (`templates/`)

Bundle type templates: `command/`, `tool/`, `tool-html/`, `format/`, `fetch/`, `selector/`, `preset/`, `cpp/`, `common/`

## Key Patterns

### Command Structure
```rust
// In commands/foo.rs
pub struct FooArgs {
    pub field: Type,
}

pub fn execute(args: FooArgs) -> Result<()> {
    // Implementation
}

// In main.rs
Command::Foo { field } => foo::execute(foo::FooArgs { field })
```

### Error Handling
- All functions return `Result<T>` with `EchidnaError`
- Use `thiserror` derive macros
- Provide helpful error messages

### uv/pip Backend Pattern
```rust
// Detect uv availability
let backend = if which::which("uv").is_ok() {
    PackageBackend::Uv
} else {
    PackageBackend::Pip
};

// Use appropriate command
match backend {
    PackageBackend::Uv => /* uv pip ... --python <chimerax> */,
    PackageBackend::Pip => /* <chimerax-python> -m pip ... */,
}
```

### ChimeraX Path Resolution
Priority: CLI `--chimerax` > `CHIMERAX_PATH` env > `echidna.toml` > auto-detection

## Testing Strategy

- **Unit tests**: `#[cfg(test)]` modules in source files
- **Integration tests**: `tests/` directory with `assert_cmd` and `tempfile`
- Tests create temporary directories for isolation

## CI/CD

### Workflows
- **ci.yml**: PR/push → fmt, clippy, tests (Linux/macOS/Windows)
- **release.yml**: `v*` tag → build binaries for 4 targets

### CI Optimization
- Docs-only changes skip Rust checks (detected by path filters)

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` (move Unreleased to new version)
3. Create PR, wait for CI
4. Merge PR
5. Create and push tag: `git tag v0.x.0 && git push origin v0.x.0`
6. Release workflow builds and publishes binaries
