---
sidebar_position: 15
---

# workspace

Manage multi-bundle workspaces. A workspace allows you to develop multiple bundles in a single repository.

## Subcommands

### `echi workspace init`

Initialize a workspace in the current directory.

```bash
echi workspace init              # Initialize in current directory
echi workspace init ./my-ws      # Initialize in specific directory
echi workspace init --force      # Overwrite existing workspace.toml
```

| Option | Description |
|--------|-------------|
| `-f, --force` | Force overwrite existing `workspace.toml` |

### `echi workspace list`

List workspace members.

```bash
echi workspace list              # List members in current workspace
echi workspace list ./my-ws      # List members in specific workspace
```

## Workspace Commands

Several commands support the `--all` flag to operate on all workspace members:

- `echi build --all` - Build all bundles
- `echi test --all` - Test all bundles
