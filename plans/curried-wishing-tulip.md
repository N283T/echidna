# echidna packages コマンド

## 概要
ChimeraX Python環境のパッケージ一覧表示と、新規パッケージ追加時の競合チェック機能を追加。

## 要件
1. **パッケージ一覧**: ChimeraX Python環境のパッケージを表示（標準ライブラリ除外）
2. **競合チェック**: 新規パッケージ追加時の依存関係競合を検出
3. **uv対応**: 利用可能ならuvを使用、なければpipにフォールバック

## CLI設計

```bash
echidna packages                        # パッケージ一覧（listのエイリアス）
echidna packages list                   # 明示的な一覧表示
echidna packages list --json            # JSON形式で出力
echidna packages list --include-stdlib  # 標準ライブラリも含める

echidna packages check numpy            # numpyを追加した場合の競合確認
echidna packages check "requests>=2.28" # バージョン指定付き
echidna packages check -r requirements.txt  # ファイルから確認
```

## アーキテクチャ

### バックエンド優先順位
```
uv（PATH上にある場合）
  ↓
pip（ChimeraX Python経由）
```

### Python実行パス取得
既存の `PythonInfo.executable` を使用（`get_python_info()` で取得済み）

## ファイル構成

```
src/
├── commands/
│   ├── mod.rs              # pub mod packages; 追加
│   └── packages.rs         # 新規: コマンド実装
├── packages/               # 新規: パッケージ管理モジュール
│   ├── mod.rs              # データ構造とエクスポート
│   └── resolver.rs         # uv/pip バックエンド実装
├── main.rs                 # PackagesCommand 追加
└── error.rs                # PackageError 追加
```

## 実装詳細

### 1. データ構造 (`src/packages/mod.rs`)

```rust
mod resolver;

pub use resolver::PackageResolver;

/// パッケージ情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

/// 競合チェック結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictCheckResult {
    pub package: String,
    pub installable: bool,
    pub conflicts: Vec<ConflictInfo>,
    pub would_install: Vec<PackageInfo>,
    pub would_upgrade: Vec<PackageChange>,
}

/// 競合詳細
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub package: String,
    pub reason: String,
}

/// バージョン変更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageChange {
    pub name: String,
    pub from_version: String,
    pub to_version: String,
}

/// 使用するバックエンド
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageBackend {
    Uv,
    Pip,
}
```

### 2. リゾルバー (`src/packages/resolver.rs`)

```rust
pub struct PackageResolver {
    python_executable: PathBuf,
    backend: PackageBackend,
    verbosity: u8,
}

impl PackageResolver {
    pub fn new(python_executable: PathBuf, verbosity: u8) -> Self {
        let backend = if which::which("uv").is_ok() {
            PackageBackend::Uv
        } else {
            PackageBackend::Pip
        };
        Self { python_executable, backend, verbosity }
    }

    pub fn backend(&self) -> PackageBackend { self.backend }

    pub fn list_packages(&self) -> Result<Vec<PackageInfo>> {
        match self.backend {
            PackageBackend::Uv => self.list_uv(),
            PackageBackend::Pip => self.list_pip(),
        }
    }

    pub fn check_package(&self, package: &str) -> Result<ConflictCheckResult> {
        match self.backend {
            PackageBackend::Uv => self.check_uv(package),
            PackageBackend::Pip => self.check_pip(package),
        }
    }

    // uv実装
    fn list_uv(&self) -> Result<Vec<PackageInfo>> {
        // uv pip list --python <path> --format json
    }

    fn check_uv(&self, package: &str) -> Result<ConflictCheckResult> {
        // uv pip install --python <path> --dry-run <package>
    }

    // pip実装（フォールバック）
    fn list_pip(&self) -> Result<Vec<PackageInfo>> {
        // <python> -m pip list --format json
    }

    fn check_pip(&self, package: &str) -> Result<ConflictCheckResult> {
        // <python> -m pip install --dry-run <package>
    }
}
```

### 3. コマンド実装 (`src/commands/packages.rs`)

```rust
pub struct ListArgs {
    pub format: OutputFormat,
    pub include_stdlib: bool,
    pub chimerax: PathBuf,
    pub verbosity: u8,
}

pub struct CheckArgs {
    pub package: Option<String>,
    pub requirements: Option<PathBuf>,
    pub format: OutputFormat,
    pub chimerax: PathBuf,
    pub verbosity: u8,
}

pub fn list(args: ListArgs) -> Result<()> {
    let executor = ChimeraXExecutor::new(args.chimerax, args.verbosity);
    let python_info = executor.get_python_info()?;
    let resolver = PackageResolver::new(python_info.executable.into(), args.verbosity);

    eprintln!("Using {:?} backend...", resolver.backend());
    let packages = resolver.list_packages()?;

    // 標準ライブラリをフィルタ
    let packages = if args.include_stdlib {
        packages
    } else {
        filter_stdlib(packages)
    };

    // 出力
    match args.format {
        OutputFormat::Text => print_text(&packages),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&packages)?),
    }
    Ok(())
}

pub fn check(args: CheckArgs) -> Result<()> {
    // 引数検証
    // リゾルバー作成
    // 競合チェック実行
    // 結果出力
}
```

### 4. main.rs 変更

```rust
#[derive(Subcommand)]
enum PackagesCommand {
    /// List packages in ChimeraX Python environment
    List {
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
        #[arg(long)]
        include_stdlib: bool,
    },
    /// Check for conflicts when adding a package
    Check {
        package: Option<String>,
        #[arg(short, long)]
        requirements: Option<PathBuf>,
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },
}

// Command enumに追加
#[command(subcommand)]
Packages(PackagesCommand),
```

### 5. エラー追加 (`src/error.rs`)

```rust
#[error("Package operation failed: {0}")]
PackageError(String),

#[error("Package conflict detected")]
PackageConflict,
```

## 標準ライブラリフィルタ

Python 3.9+ の標準ライブラリリストをハードコード:
- `pip`, `setuptools`, `wheel` などビルトインツール
- `typing-extensions` など一般的な互換パッケージはフィルタしない

## 出力例

### `echidna packages list`
```
ChimeraX Python packages (using uv):

Package          Version
──────────────────────────
numpy            1.24.3
scipy            1.11.0
Pillow           10.0.0
...

Found 42 packages (excluding stdlib)
```

### `echidna packages check numpy`
```
Checking package 'numpy' (using uv)...

✓ Package can be installed without conflicts

Would install:
  numpy 1.26.0
```

### 競合がある場合
```
Checking package 'numpy==1.20.0' (using uv)...

✗ Conflicts detected!

Conflicts:
  - scipy 1.11.0 requires numpy>=1.21.0, but numpy==1.20.0 requested

Would downgrade:
  numpy: 1.24.3 → 1.20.0
```

## 検証方法

テスト用ChimeraX: `/Applications/ChimeraX-1.10-rc2025.05.21.app`

1. **パッケージ一覧**
   ```bash
   echidna packages list --chimerax /Applications/ChimeraX-1.10-rc2025.05.21.app/Contents/MacOS/ChimeraX
   echidna packages list --json
   ```

2. **競合チェック（競合なし）**
   ```bash
   echidna packages check requests
   ```

3. **競合チェック（競合あり）**
   ```bash
   echidna packages check "numpy==1.20.0"  # 古いバージョン指定
   ```

4. **requirements.txt チェック**
   ```bash
   echo "requests>=2.28" > test-reqs.txt
   echidna packages check -r test-reqs.txt
   ```

5. **uvなしでのフォールバック確認**
   ```bash
   PATH=/usr/bin echidna packages list  # uvを除外したPATH
   ```

## 実装順序

1. **Phase 1**: 基本構造
   - `src/packages/mod.rs` - データ構造
   - `src/error.rs` - エラー追加

2. **Phase 2**: リゾルバー実装
   - `src/packages/resolver.rs` - pip実装（基本）
   - uv実装追加

3. **Phase 3**: CLI統合
   - `src/commands/packages.rs` - listコマンド
   - `src/main.rs` - ルーティング追加

4. **Phase 4**: 競合チェック
   - checkコマンド実装
   - requirements.txt対応

5. **Phase 5**: テスト
   - ユニットテスト
   - 統合テスト（tests/packages.rs）

## 変更ファイル一覧

| ファイル | 変更内容 |
|---------|---------|
| `src/packages/mod.rs` | 新規: データ構造 |
| `src/packages/resolver.rs` | 新規: uv/pipバックエンド |
| `src/commands/packages.rs` | 新規: コマンド実装 |
| `src/commands/mod.rs` | packages モジュール追加 |
| `src/main.rs` | PackagesCommand追加 |
| `src/error.rs` | PackageError追加 |
| `src/lib.rs` | packagesモジュール公開 |
| `tests/packages.rs` | 新規: 統合テスト |
