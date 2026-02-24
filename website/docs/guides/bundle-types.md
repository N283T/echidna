---
sidebar_position: 1
---

# Bundle Types

echidna supports 8 bundle types, each providing a different template for ChimeraX extension development.

## Available Types

| Type | Description | Use Case |
|------|-------------|----------|
| `command` | Command-only bundle (default) | Add new commands to ChimeraX |
| `tool` | Qt-based GUI tool | Create a dockable panel with Qt widgets |
| `tool-html` | HTML-based GUI tool | Create a dockable panel with HTML/JS UI |
| `format` | File format reader/writer | Add support for new file formats |
| `fetch` | Network database fetcher | Fetch data from online databases |
| `selector` | Chemical subgroup selector | Define custom atom/residue selectors |
| `preset` | Visualization presets | Create reusable visualization styles |
| `cpp` | C/C++ extension bundle | Bundle with native C++ code |

## Selecting a Type

Use the `--type` (or `-t`) flag with `echi init`:

```bash
echi init --type command my-tool       # Default
echi init --type tool my-gui-tool
echi init --type tool-html my-html-tool
echi init --type format my-format
echi init --type fetch my-fetcher
echi init --type selector my-selector
echi init --type preset my-preset
echi init --type cpp my-native-ext
```

## Type Details

### command (default)

Registers a new command in ChimeraX. This is the most common bundle type.

**Generated files:**
- `src/__init__.py` - Bundle initialization, command registration
- `src/cmd.py` - Command implementation

**Example use cases:** Adding data analysis commands, automation scripts, custom workflows.

### tool

Creates a dockable GUI panel using Qt widgets. Use this for interactive tools that need native desktop UI elements.

**Generated files:**
- `src/__init__.py` - Bundle initialization, tool registration
- `src/tool.py` - Tool panel implementation with Qt

**Example use cases:** Property inspectors, control panels, interactive editors.

### tool-html

Creates a dockable GUI panel using an embedded HTML/JavaScript interface. Useful when you prefer web technologies for the UI.

**Generated files:**
- `src/__init__.py` - Bundle initialization, tool registration
- `src/tool.py` - Tool panel with HTML UI

**Example use cases:** Data dashboards, form-based tools, visualizations using web libraries.

### format

Adds support for reading (and optionally writing) a new file format.

**Generated files:**
- `src/__init__.py` - Bundle initialization, format registration
- `src/open.py` - File reader implementation

**Example use cases:** Custom molecular file formats, structure databases, experimental data formats.

### fetch

Registers a fetcher that downloads data from a network database.

**Generated files:**
- `src/__init__.py` - Bundle initialization, fetch registration
- `src/fetch.py` - Fetcher implementation

**Example use cases:** Database clients (PDB, UniProt, custom databases).

### selector

Defines a custom selector for picking atoms, residues, or other model components.

**Generated files:**
- `src/__init__.py` - Bundle initialization, selector registration
- `src/selector.py` - Selector implementation

**Example use cases:** Chemical subgroup selectors, property-based selection.

### preset

Creates reusable visualization presets that users can apply to models.

**Generated files:**
- `src/__init__.py` - Bundle initialization, preset registration
- `src/preset.py` - Preset implementation

**Example use cases:** Publication-quality styles, analysis-specific coloring schemes.

### cpp

A bundle that includes C/C++ code compiled as a native extension. Uses a different source layout with the package nested under `src/chimerax/`.

**Generated files:**
- `src/chimerax/<package>/__init__.py` - Bundle initialization
- `src/chimerax/<package>/cmd.py` - Python command wrapper
- `src/chimerax/<package>/_extension.cpp` - C++ extension code

**Example use cases:** Performance-critical computations, wrapping existing C/C++ libraries.

:::note
C++ bundles are marked as `pure = false` in `pyproject.toml` and require a C++ compiler to build.
:::
