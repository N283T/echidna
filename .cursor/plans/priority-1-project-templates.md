# Priority 1: Enhanced Project Templates

## Overview

Add bundle type templates to `echidna init` command.

**Target Version**: v0.5.0

## CLI Usage

```bash
echidna init my-tool --type command    # (current default)
echidna init my-tool --type tool       # GUI tool (Qt-based)
echidna init my-tool --type tool-html  # GUI tool (HTML-based)
echidna init my-tool --type format     # File format reader/writer
echidna init my-tool --type fetch      # Network database fetcher
echidna init my-tool --type selector   # Chemical subgroup selector
echidna init my-tool --type preset     # Visualization presets
```

## Implementation Details

Each template should generate:
- Appropriate boilerplate in `src/`
- Proper `pyproject.toml` sections for the bundle type

### Bundle Types

| Type | Description |
|------|-------------|
| `command` | CLI command (current default) |
| `tool` | Qt-based GUI tool |
| `tool-html` | HTML-based GUI tool |
| `format` | File format reader/writer |
| `fetch` | Network database fetcher |
| `selector` | Chemical subgroup selector |
| `preset` | Visualization presets |

## Reference

- `tutorials/tutorial_*.html` in ChimeraX documentation
