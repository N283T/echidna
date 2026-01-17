# echidna Roadmap

Future features and improvements based on ChimeraX development documentation.

## Feature Overview

| Priority | Feature | Target | Description |
|----------|---------|--------|-------------|
| 1 | [Project Templates](priority-1-project-templates.md) | v0.5.0 | `echidna init --type` with bundle type templates |
| 2 | [Documentation Support](priority-2-docs-command.md) | v0.8.0 | `echidna docs` command for documentation |
| 3 | [Toolshed Integration](priority-3-toolshed-publish.md) | v1.0.0 | `echidna publish` for Toolshed submission |
| 4 | [Advanced Validation](priority-4-advanced-validation.md) | v0.7.0 | Enhanced `echidna validate` checks |
| 5 | [Test Improvements](priority-5-test-improvements.md) | - | `echidna test` enhancements |
| 6 | [Watch Mode](priority-6-watch-mode.md) | v0.6.0 | `echidna watch` for auto-rebuild |
| 7 | [C/C++ Extension Support](priority-7-cpp-extension.md) | - | Build C++ extensions |
| 8 | [Workspace Management](priority-8-workspace.md) | - | Multiple bundle support |
| 9 | [Version Management](priority-9-version-management.md) | v0.9.0 | `echidna version` command |
| 10 | [Debug Mode](priority-10-debug-mode.md) | - | `echidna debug` command |

## Implementation Order

1. **v0.5.0**: `echidna init --type` (tool templates)
2. **v0.6.0**: `echidna watch` (developer experience)
3. **v0.7.0**: Enhanced validation
4. **v0.8.0**: `echidna docs`
5. **v0.9.0**: `echidna version`
6. **v1.0.0**: `echidna publish` + polish

## Notes

- All features should work offline when possible
- Maintain backwards compatibility with existing projects
- Consider Windows/Linux parity (currently macOS-focused)
- Test with real ChimeraX bundle projects
