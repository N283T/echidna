---
sidebar_position: 3
---

# Requirements

## UCSF ChimeraX

[UCSF ChimeraX](https://www.cgl.ucsf.edu/chimerax/) must be installed on your system. echidna auto-detects ChimeraX on all supported platforms.

### Supported Platforms

| Platform | Auto-detected Paths |
|----------|-------------------|
| macOS | `/Applications/ChimeraX.app`, `~/Applications/ChimeraX.app` |
| Linux (DEB) | `/usr/bin/chimerax`, `/usr/lib/ucsf-chimerax/bin/ChimeraX` |
| Linux (RPM) | `/usr/bin/chimerax`, `/usr/libexec/UCSF-ChimeraX/bin/ChimeraX` |
| Linux (local) | `/usr/local/bin/chimerax`, `~/.local/bin/chimerax` |
| Windows | `C:\Program Files\ChimeraX\bin\ChimeraX-console.exe` |
| Windows (legacy) | `C:\Program Files\ChimeraX 1.x\bin\ChimeraX-console.exe` |

If ChimeraX is installed in a non-standard location, see [ChimeraX Detection](../configuration/chimerax-detection.md) for how to configure the path manually.

### Flatpak

Flatpak installations (`flatpak run edu.ucsf.rbvi.ChimeraX`) are **not auto-detected** because they require a different invocation method. To use a Flatpak installation:

1. Create a wrapper script that calls `flatpak run edu.ucsf.rbvi.ChimeraX "$@"`
2. Place it on your PATH as `chimerax`
3. Or set the `CHIMERAX_PATH` environment variable to point to the wrapper

## Optional Tools

### uv (Recommended)

echidna uses [uv](https://github.com/astral-sh/uv) for faster package operations when available. If uv is not installed, echidna falls back to pip automatically.

Install uv:

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```
