# Priority 7: C/C++ Extension Support

## Overview

Support building C/C++ extensions via `[chimerax.extension.*]` in pyproject.toml.

## CLI Usage

```bash
echidna build                    # Detect and compile C/C++
echidna init --type cpp          # Template with C++ extension
```

## Implementation Details

### Features

- Detect `[chimerax.extension.*]` sections in pyproject.toml
- Compile C/C++ extensions during build
- Provide C++ extension project template

### Template

The `--type cpp` template should include:
- C++ source files in appropriate location
- Proper `[chimerax.extension.*]` configuration
- Build instructions

## Reference

- `tutorials/pyproject.html#source-extensions` in ChimeraX documentation
