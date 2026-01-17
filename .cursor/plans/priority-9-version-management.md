# Priority 9: Version Management

## Overview

Add `echidna version` command for consistent version bumping.

**Target Version**: v0.9.0

## CLI Usage

```bash
echidna version                  # Show current version
echidna version patch            # Bump patch version
echidna version minor            # Bump minor version
echidna version major            # Bump major version
echidna version 1.2.3            # Set specific version
```

## Implementation Details

### Subcommands

| Command | Description |
|---------|-------------|
| (none) | Show current version |
| `patch` | Bump patch version (0.0.X) |
| `minor` | Bump minor version (0.X.0) |
| `major` | Bump major version (X.0.0) |
| `X.Y.Z` | Set specific version |

### Version Location

Update version in `pyproject.toml` under:
- `[project]` section `version` field
