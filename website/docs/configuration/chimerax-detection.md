---
sidebar_position: 2
---

# ChimeraX Detection

echidna auto-detects ChimeraX on all supported platforms. You can also specify the path manually.

## Resolution Priority

ChimeraX path is resolved in this order:

1. **CLI flag**: `--chimerax /path/to/ChimeraX`
2. **Environment variable**: `CHIMERAX_PATH=/path/to/ChimeraX`
3. **Project config**: `chimerax_path` in `echidna.toml`
4. **Auto-detection**: Platform-specific default paths

## Auto-Detection Paths

### macOS

| Path | Source |
|------|--------|
| `/Applications/ChimeraX.app/Contents/MacOS/ChimeraX` | Standard install |
| `~/Applications/ChimeraX.app/Contents/MacOS/ChimeraX` | User install |

Also searches PATH for `chimerax` and `ChimeraX`.

### Linux

| Path | Source |
|------|--------|
| `/usr/bin/chimerax` | DEB/RPM symlink |
| `/usr/local/bin/chimerax` | Manual install |
| `/usr/lib/ucsf-chimerax/bin/ChimeraX` | DEB package (Ubuntu/Debian) |
| `/usr/libexec/UCSF-ChimeraX/bin/ChimeraX` | RPM package (RHEL/Rocky/CentOS) |
| `~/.local/bin/chimerax` | User install |

### Windows

| Path | Source |
|------|--------|
| `C:\Program Files\ChimeraX\bin\ChimeraX-console.exe` | Standard install |
| `C:\Program Files\ChimeraX\bin\chimerax.exe` | Alternative name |
| `C:\Program Files\ChimeraX 1.x\bin\ChimeraX-console.exe` | Legacy versioned install (pre-1.7) |

For legacy installs, echidna scans `C:\Program Files` for directories matching `ChimeraX <version>` and tries the newest version first.

## Manual Configuration

### Using the CLI Flag

```bash
echi run --chimerax /custom/path/to/ChimeraX
```

This flag is available on all commands (it's a global option).

### Using an Environment Variable

```bash
export CHIMERAX_PATH=/custom/path/to/ChimeraX
echi run
```

### Using echidna.toml

```toml
chimerax_path = "/custom/path/to/ChimeraX"
```

## Flatpak Installations

Flatpak installations are not auto-detected because they require `flatpak run` instead of direct execution. To use a Flatpak ChimeraX:

1. Create a wrapper script:
   ```bash
   #!/bin/bash
   flatpak run edu.ucsf.rbvi.ChimeraX "$@"
   ```
2. Save it as `chimerax` somewhere on your PATH (e.g., `~/.local/bin/chimerax`)
3. Make it executable: `chmod +x ~/.local/bin/chimerax`
