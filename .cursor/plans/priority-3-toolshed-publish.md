# Priority 3: Toolshed Integration

## Overview

Add `echidna publish` command for ChimeraX Toolshed submission.

**Target Version**: v1.0.0

## CLI Usage

```bash
echidna publish --dry-run   # Validate bundle for submission
echidna publish             # Guide through submission process
```

## Implementation Details

### Features

- Validate all required metadata
- Check for license file
- Verify wheel builds correctly
- Open browser to toolshed submission page

### Validation Checks

1. Required metadata present in `pyproject.toml`
2. LICENSE file exists
3. Wheel builds successfully
4. Bundle passes all validation checks

## Reference

- `tutorials/introduction.html#distributing-bundles` in ChimeraX documentation
