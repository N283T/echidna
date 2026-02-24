# Priority 6: Watch Mode

## Overview

Add `echidna watch` command for auto-rebuild on file changes.

**Target Version**: v0.6.0

## CLI Usage

```bash
echidna watch                    # Watch and rebuild
echidna watch --run              # Also launch ChimeraX
echidna watch --test             # Run tests on change
```

## Implementation Details

### Options

| Flag | Description |
|------|-------------|
| (none) | Watch and rebuild on changes |
| `--run` | Also launch ChimeraX |
| `--test` | Run tests on changes |

### Technical Implementation

Use file watcher (notify crate) to detect changes and trigger rebuilds.
