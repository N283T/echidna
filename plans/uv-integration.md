# uv 内部統合計画

echidna内部でuvを活用し、パッケージ管理を高速化する。

## 実装状況

### 完了済み

- **`packages list`** - uv対応済み (`PackageResolver` in `src/packages/resolver.rs`)
- **`packages check`** - uv対応済み (同上)
- **`packages install`** - uv対応済み (`PackageInstaller` in `src/packages/installer.rs`, PR #42)
- **`venv`** - uv対応済み (`VenvBuilder` in `src/venv/builder.rs`, PR #41)

すべてuvがPATHにあれば自動で使用、なければフォールバック。

---

## 実装済み: `packages install`

### 概要

ChimeraXのPython環境にパッケージをインストールするコマンド。

```bash
echidna packages install pytest
echidna packages install numpy pandas  # 複数も可
echidna packages install "requests>=2.28"  # バージョン指定も可
```

### 内部動作

1. uvがあれば:
   ```bash
   uv pip install --python <chimerax-python> <packages>
   ```

2. uvがなければ:
   ```bash
   <chimerax-python> -m pip install <packages>
   ```

### 実装箇所

1. **`src/packages/mod.rs`**
   - `PackagesCommand` enumに `Install` variant追加

2. **`src/packages/installer.rs`** (新規)
   - `PackageInstaller` struct
   - uv/pip切り替えロジック（`PackageResolver`と同様のパターン）

3. **`src/commands/packages.rs`**
   - `install` サブコマンドのハンドラ追加

4. **`src/main.rs`**
   - CLIパース部分（`PackagesCommand::Install`）

### CLI設計

```
echidna packages install [OPTIONS] <PACKAGES>...

Arguments:
  <PACKAGES>...  Packages to install (e.g., pytest, numpy>=1.20)

Options:
  --upgrade, -U    Upgrade packages if already installed
  --dry-run        Show what would be installed without installing
```

### 実装手順

1. `PackagesCommand::Install` を追加
2. `PackageInstaller` struct を作成（`resolver.rs`の`PackageResolver`を参考）
3. uv用とpip用のinstallメソッド実装
4. テスト追加
5. PR作成

### 技術メモ

uvは既存のPythonインタープリタを `--python` フラグで指定可能：
```bash
uv pip install --python /path/to/chimerax/python package
```

参考: https://docs.astral.sh/uv/pip/environments/
