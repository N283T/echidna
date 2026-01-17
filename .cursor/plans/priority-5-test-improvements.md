# Priority 5: Test Improvements

## Overview

Enhance `echidna test` with ChimeraX-specific features.

## CLI Usage

```bash
echidna test --coverage          # Generate coverage report
echidna test --wheel             # Run wheel-mode tests
echidna test --session           # Tests requiring ChimeraX session
```

## Implementation Details

### Options

| Flag | Description |
|------|-------------|
| `--coverage` | Generate coverage report |
| `--wheel` | Run wheel-mode tests |
| `--session` | Run tests requiring ChimeraX session |

### Pytest Configuration

Generate pytest configuration with ChimeraX fixtures:
- `test_production_session` fixture
- `@pytest.mark.wheel` marker support

## Reference

- `testing/main.html` in ChimeraX documentation
