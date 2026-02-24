# Improve: ChimeraX Detection UX

## Desired Improvements

1. 検出前に「Searching for ChimeraX...」メッセージ表示
2. Auto-detect時にバージョンも表示
3. シンボリックリンクの場合はその旨とリンク先を表示

## Current Behavior

検出結果がいきなり表示される。検出中であることがユーザーに伝わらない。

## Files to Modify

| File | Change |
|------|--------|
| `src/chimerax/validator.rs` | 検出メッセージ追加、シンボリックリンク表示 |
| `src/chimerax/detect.rs` | シンボリックリンク解決ヘルパー追加 |

## Implementation

### 1. Add Searching Message

**`src/chimerax/validator.rs`** の `validate_and_prompt()`:

```rust
pub fn validate_and_prompt(
    global_config: &mut GlobalConfig,
    project_config: &Option<Config>,
) -> Result<Option<PathBuf>> {
    // 検出が必要かチェック
    let needs_detection = project_config.as_ref()
        .and_then(|c| c.chimerax_path.as_ref())
        .is_none()
        && global_config.chimerax_path.is_none();

    if needs_detection {
        eprintln!("\x1b[90mSearching for ChimeraX...\x1b[0m");
    }

    let result = Self::validate(global_config, project_config);
    // ... rest of the function
}
```

### 2. Add Symlink Detection

**`src/chimerax/detect.rs`** に追加:

```rust
/// Check if a path is a symlink and return info about it.
pub fn get_symlink_info(path: &Path) -> Option<PathBuf> {
    if path.is_symlink() {
        std::fs::read_link(path).ok()
    } else {
        None
    }
}
```

### 3. Update Display Logic

**`src/chimerax/validator.rs`** のパス表示箇所:

```rust
// パスを表示した後
if let Some(target) = detect::get_symlink_info(&path) {
    eprintln!("  \x1b[90m-> {}\x1b[0m", target.display());
}
```

## Output Format

### Before (Current)
```
✓ Auto-detected ChimeraX:
  /usr/local/bin/chimerax (v1.9)
```

### After (Proposed)
```
Searching for ChimeraX...
✓ Found ChimeraX 1.9
  /usr/local/bin/chimerax
  -> /opt/UCSF/ChimeraX-1.9/bin/chimerax
```

## Verification

```bash
# Test with symlink
ln -s /Applications/ChimeraX.app/Contents/MacOS/ChimeraX /tmp/chimerax-test
CHIMERAX_PATH=/tmp/chimerax-test echidna build
# Should show symlink target

# Test with no prior config
rm ~/.config/echidna/config.toml
echidna build
# Should show "Searching for ChimeraX..." before result

# Test with cached config (second run)
echidna build
# Should NOT show searching message
```

## Dependencies

No new dependencies. Using standard library:
- `std::path::Path::is_symlink()` (stable since Rust 1.58)
- `std::fs::read_link()` (stable)
