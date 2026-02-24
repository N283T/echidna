---
sidebar_position: 13
---

# packages

Manage packages in ChimeraX's Python environment. Uses [uv](https://github.com/astral-sh/uv) when available for faster operations, otherwise falls back to pip.

## Subcommands

### `echi packages list`

List installed packages in ChimeraX's Python environment.

```bash
echi packages list                    # List all packages
echi packages list --format json      # JSON output
echi packages list --include-stdlib   # Include stdlib packages (pip, setuptools, wheel)
```

| Option | Description |
|--------|-------------|
| `-f, --format <FORMAT>` | Output format: `text` (default) or `json` |
| `--include-stdlib` | Include stdlib packages (pip, setuptools, wheel) |

### `echi packages check`

Check for conflicts before installing a package.

```bash
echi packages check numpy                  # Check single package
echi packages check -r requirements.txt    # Check from file
echi packages check numpy --format json    # JSON output
```

| Option | Description |
|--------|-------------|
| `-r, --requirements <FILE>` | Read packages from requirements file |
| `-f, --format <FORMAT>` | Output format: `text` (default) or `json` |

### `echi packages install`

Install packages into ChimeraX's Python environment.

```bash
echi packages install pytest               # Install single package
echi packages install numpy pandas         # Install multiple packages
echi packages install "requests>=2.28"     # With version specifier
echi packages install -U numpy             # Upgrade if already installed
echi packages install --dry-run pytest     # Preview installation
```

| Option | Description |
|--------|-------------|
| `-U, --upgrade` | Upgrade packages if already installed |
| `--dry-run` | Show what would be installed without installing |
