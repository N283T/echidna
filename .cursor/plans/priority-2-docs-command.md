# Priority 2: Documentation Support

## Overview

Add `echidna docs` command for documentation scaffolding.

**Target Version**: v0.8.0

## CLI Usage

```bash
echidna docs init           # Create docs/ structure
echidna docs serve          # Local preview server
echidna docs build          # Build HTML docs
```

## Implementation Details

ChimeraX expects docs in specific locations:
- `src/docs/user/commands/COMMANDNAME.html`
- `src/docs/user/tools/TOOLNAME.html`

### Subcommands

| Command | Description |
|---------|-------------|
| `docs init` | Create documentation directory structure |
| `docs serve` | Start local preview server |
| `docs build` | Build HTML documentation |

## Reference

- `writing_bundles.html#bundle-documentation` in ChimeraX documentation
