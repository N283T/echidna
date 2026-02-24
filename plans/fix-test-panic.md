# Fix: `echidna test` Panic

## Problem
`echidna test` でpanicが発生:
```
thread 'main' panicked at clap_builder: Mismatch between definition and access of `verbose`.
Could not downcast to TypeId...
```

## Root Cause
clapの引数名衝突:

1. **Global `verbose` flag** (`main.rs` line 27-28):
   ```rust
   #[arg(short, long, action = clap::ArgAction::Count, global = true)]
   verbose: u8,
   ```
   - 型: `u8` (count action)
   - スコープ: `global = true`

2. **Test command's `verbose` flag** (`main.rs` line 195-196):
   ```rust
   #[arg(long)]
   verbose: bool,
   ```
   - 型: `bool`
   - スコープ: Test subcommand local

グローバルの`verbose`が`global = true`のため全サブコマンドに伝播。Testコマンドが異なる型で同名のフラグを定義しているため、実行時に型の不一致が発生。

## Solution
Testコマンドの`verbose`フラグを別名にリネーム。

## Files to Modify

| File | Change |
|------|--------|
| `src/main.rs` | Test variant の `verbose: bool` を `pytest_verbose: bool` にリネーム |
| `src/commands/testing.rs` | `TestArgs` の `verbose` を `pytest_verbose` にリネーム |

## Implementation

### 1. `src/main.rs` (line 195-196)

**Before:**
```rust
#[arg(long)]
verbose: bool,
```

**After:**
```rust
/// Increase pytest verbosity (-v flag)
#[arg(long = "verbose")]
pytest_verbose: bool,
```

### 2. `src/commands/testing.rs` (line 17)

**Before:**
```rust
pub struct TestArgs {
    // ...
    pub verbose: bool,
    // ...
}
```

**After:**
```rust
pub struct TestArgs {
    // ...
    pub pytest_verbose: bool,
    // ...
}
```

Also update usage in execute function.

### 3. `src/main.rs` - Command dispatch

Update the dispatch to pass the renamed field.

## Verification

```bash
cargo build
cargo test
echidna test --help
echidna test  # Should not panic
```
