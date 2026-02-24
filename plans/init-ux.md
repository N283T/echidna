# Improve: `echidna init` UX (uv風に)

## Current vs New Behavior

| Scenario | Current | New |
|----------|---------|-----|
| 新規プロジェクト作成 | `mkdir hello-world && cd hello-world && echidna init --name hello-world` | `echidna init hello-world` |
| カレントディレクトリ初期化 | `echidna init --name my-project .` | `echidna init` or `echidna init .` |
| 名前のオーバーライド | N/A | `echidna init some-dir --name custom-name` |

## Files to Modify

| File | Change |
|------|--------|
| `src/main.rs` | CLI引数定義の変更 (lines 36-62) |
| `src/commands/init.rs` | ロジック変更 + `resolve_path_and_name()` 追加 |
| `tests/init.rs` | テスト更新 |

## Implementation

### 1. `src/main.rs` - CLI Definition

**Before:**
```rust
Init {
    #[arg(short, long)]
    name: Option<String>,
    // ...
    #[arg(default_value = ".")]
    path: PathBuf,
    // ...
}
```

**After:**
```rust
Init {
    /// Project name or path (e.g., "my-tool" or "./projects/my-tool")
    path: Option<PathBuf>,

    /// Override the project name (defaults to directory name)
    #[arg(short, long)]
    name: Option<String>,
    // ... rest unchanged
}
```

### 2. `src/commands/init.rs` - New Logic

```rust
/// Resolve the target directory and project name from arguments.
///
/// Behavior:
/// - `echidna init` or `echidna init .` -> current dir, name from dir
/// - `echidna init hello-world` -> ./hello-world/, name = "hello-world"
/// - `echidna init ./path/to/project` -> that path, name from last component
/// - `echidna init hello-world --name custom` -> ./hello-world/, name = "custom"
fn resolve_path_and_name(
    path_arg: Option<&PathBuf>,
    name_arg: Option<&String>,
) -> Result<(PathBuf, String)> {
    let target_dir = match path_arg {
        None => std::env::current_dir()?,
        Some(p) if p.as_os_str() == "." => std::env::current_dir()?,
        Some(p) => {
            // パスが存在せず、単純な名前の場合は ./name/ として扱う
            if !p.exists() && !p.to_string_lossy().contains(std::path::MAIN_SEPARATOR) {
                std::env::current_dir()?.join(p)
            } else {
                p.clone()
            }
        }
    };

    let name = match name_arg {
        Some(n) => n.clone(),
        None => {
            target_dir
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    EchidnaError::InvalidName(
                        "Could not determine project name. Use --name to specify.".into()
                    )
                })?
        }
    };

    Ok((target_dir, name))
}
```

### 3. `tests/init.rs` - Update Tests

**Existing tests to modify:**
- `test_init_creates_bundle_structure`
- `test_init_with_name_option`
- `test_init_uses_directory_name_as_default`

**New tests to add:**
- `test_init_with_path_creates_subdirectory`
- `test_init_without_args_uses_current_dir`
- `test_init_with_dot_uses_current_dir`

## Edge Cases

| Case | Behavior |
|------|----------|
| `echidna init .` | カレントディレクトリで初期化 |
| `echidna init /abs/path` | 絶対パスをそのまま使用 |
| `echidna init foo/bar` | `./foo/bar/` を作成、name = "bar" |
| ディレクトリが既存 & 非空 | `--force` なしでエラー |
| ディレクトリが既存 & 空 | 初期化を許可 |
| 名前が数字で始まる | エラー (Python package requirement) |
| ルート `/` で初期化 | `--name` 必須 |

## Verification

```bash
# Build
cargo build

# Test 1: 新規プロジェクト
cd /tmp && rm -rf test-echidna && mkdir test-echidna && cd test-echidna
echidna init hello-world
ls hello-world/  # pyproject.toml, src/, etc.

# Test 2: カレントディレクトリ
cd /tmp && rm -rf my-bundle && mkdir my-bundle && cd my-bundle
echidna init
ls  # pyproject.toml, src/, etc.

# Test 3: 名前オーバーライド
cd /tmp && rm -rf name-test && mkdir name-test && cd name-test
echidna init my-project --name custom-name
grep "ChimeraX-CustomName" my-project/pyproject.toml

# Test 4: ネストしたパス
cd /tmp && rm -rf nested && mkdir nested && cd nested
echidna init foo/bar/baz
ls foo/bar/baz/

# Lint & Test
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Breaking Changes

This is a **breaking change** for:
- `echidna init --name foo ./path` (positional arg order changes)

Common patterns work better:
- `echidna init my-project` is more intuitive than `echidna init --name my-project ./my-project`
