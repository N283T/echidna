---
sidebar_position: 100
---

# Contributing

Thank you for your interest in contributing to echidna!

## Development Setup

### Prerequisites

- Rust 1.85+ (install via [rustup](https://rustup.rs/), edition 2024)
- [UCSF ChimeraX](https://www.cgl.ucsf.edu/chimerax/) (optional, for integration testing)

### Getting Started

```bash
git clone https://github.com/N283T/echidna.git
cd echidna
cargo build
cargo test
```

## Code Style

### Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy -- -D warnings
```

### Commit Messages

Use clear, descriptive commit messages in imperative mood:

- `Add shell completion support`
- `Fix config file parsing for Windows paths`
- `Update clap to version 4.5`

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Writing Tests

- Unit tests go in the same file as the code (`#[cfg(test)]` module)
- Integration tests go in the `tests/` directory
- Use descriptive test names that explain what is being tested

## Pull Request Process

1. Fork the repository and create a feature branch
2. Make your changes with appropriate tests
3. Ensure all tests pass: `cargo test`
4. Ensure code is formatted: `cargo fmt --check`
5. Ensure clippy passes: `cargo clippy -- -D warnings`
6. Update `CHANGELOG.md` if applicable (under `[Unreleased]`)
7. Submit a pull request with a clear description

## Reporting Issues

When reporting issues, please include:

- echi version (`echi --version`)
- Operating system and version
- ChimeraX version (if relevant)
- Steps to reproduce the issue
- Expected vs actual behavior

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
