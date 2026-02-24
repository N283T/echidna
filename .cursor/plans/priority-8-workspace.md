# Priority 8: Multiple Bundle Support

## Overview

Add workspace management for projects with multiple related bundles.

## CLI Usage

```bash
echidna workspace init           # Create workspace.toml
echidna build --all              # Build all bundles
echidna test --all               # Test all bundles
```

## Implementation Details

### Features

- `workspace.toml` configuration file
- Build all bundles in workspace
- Test all bundles in workspace

### workspace.toml Structure

```toml
[workspace]
members = [
    "bundles/bundle-a",
    "bundles/bundle-b",
]
```

### Commands

| Command | Description |
|---------|-------------|
| `workspace init` | Create workspace.toml |
| `build --all` | Build all bundles in workspace |
| `test --all` | Test all bundles in workspace |
