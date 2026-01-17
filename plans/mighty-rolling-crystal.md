# Plan: Migrate NEXTPLAN.md to .cursor/plans

## Overview

NEXTPLAN.md contains 10 prioritized features. Split into individual files for better organization.

## Directory Structure

```
.cursor/
└── plans/
    ├── README.md                         # Roadmap overview + notes
    ├── priority-1-project-templates.md   # echidna init --type
    ├── priority-2-docs-command.md        # echidna docs
    ├── priority-3-toolshed-publish.md    # echidna publish
    ├── priority-4-advanced-validation.md # echidna validate enhancements
    ├── priority-5-test-improvements.md   # echidna test enhancements
    ├── priority-6-watch-mode.md          # echidna watch
    ├── priority-7-cpp-extension.md       # C/C++ extension support
    ├── priority-8-workspace.md           # Multiple bundle support
    ├── priority-9-version-management.md  # echidna version
    └── priority-10-debug-mode.md         # echidna debug
```

## Steps

1. Create `.cursor/plans/` directory
2. Create `README.md` with roadmap overview, implementation order, and notes
3. Create 10 individual plan files (one per priority)
4. Delete `NEXTPLAN.md`

## File Contents

### README.md
- Summary of all features with priority levels
- Implementation order (v0.5.0 - v1.0.0)
- General notes (offline, backwards compatibility, platform parity)

### Individual Plan Files
Each file will contain:
- Feature description
- CLI usage examples
- Implementation details
- Reference documentation

## Verification

- Confirm all 10 features are captured in individual files
- Confirm README has complete overview
- Delete NEXTPLAN.md after migration
