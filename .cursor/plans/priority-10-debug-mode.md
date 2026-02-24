# Priority 10: Debug Mode

## Overview

Add `echidna debug` command for launching ChimeraX in debug mode.

## CLI Usage

```bash
echidna debug                    # ChimeraX --debug mode
echidna debug --pdb              # Enable Python debugger
echidna debug --profile          # Enable profiling
```

## Implementation Details

### Options

| Flag | Description |
|------|-------------|
| (none) | Launch ChimeraX in debug mode |
| `--pdb` | Enable Python debugger |
| `--profile` | Enable profiling |

### Features

- Launch ChimeraX with `--debug` flag
- Optional Python debugger integration
- Optional profiling support

## Reference

- `writing_bundles.html#building-and-testing-bundles` in ChimeraX documentation
