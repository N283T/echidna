# Priority 4: Advanced Validation

## Overview

Enhance `echidna validate` with comprehensive checks.

**Target Version**: v0.7.0

## CLI Usage

```bash
echidna validate --strict   # Fail on warnings
echidna validate --fix      # Auto-fix common issues
```

## Implementation Details

### Validation Checks

- [ ] Command/tool declarations match actual code
- [ ] Required `bundle_api` object exists in `__init__.py`
- [ ] Dependencies are valid ChimeraX bundles
- [ ] Python classifiers are correct
- [ ] Session version compatibility
- [ ] Help files exist for declared commands/tools

### Options

| Flag | Description |
|------|-------------|
| `--strict` | Treat warnings as errors |
| `--fix` | Auto-fix common issues |

## Reference

- `tutorials/pyproject.html` in ChimeraX documentation
