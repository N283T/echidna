# ChimeraX MCP Server Plan

ChimeraX用の包括的なMCPサーバーを構築する計画。

## 既存実装の調査結果

### 1. chatmol/molecule-mcp
- **通信方式**: XML-RPC (port 42184)
- **ツール**: `open_chimerax`, `run_chimerax_command` の2つのみ
- **URL**: https://github.com/chatmol/molecule-mcp

### 2. GDAmitha/chimerax-alphafold-mcp
- **通信方式**: REST API (port 63269, `remotecontrol rest start`)
- **ツール**: 4つ (parse_and_execute, alphafold_predict, alphafold_fetch, search_protein_pdbs)
- **URL**: https://github.com/GDAmitha/chimerax-alphafold-mcp

### 3. jessicalh/chimerax-mcp
- LobeHubに掲載 (v1.0.0, Oct 2025)
- 詳細不明 (GitHubリポジトリにアクセスできず)

## 既存実装の課題

1. **限定的なツール**: 基本的なコマンド実行のみ
2. **状態管理なし**: 開いているモデル、セッション状態の追跡なし
3. **スクリーンショット**: 明示的なツールなし (コマンドで可能だが)
4. **echidna連携なし**: バンドル開発ワークフローとの統合なし

## 目標機能

### コア機能
1. **ChimeraX検出・起動**
   - echidnaの検出ロジックを活用
   - REST APIを有効化して起動
   - 接続状態の確認

2. **コマンド実行**
   - 任意のChimeraXコマンドを実行
   - 結果・エラーの取得
   - コマンド履歴の追跡

3. **状態管理**
   - 開いているモデル一覧
   - 現在のビュー設定
   - セッション情報

4. **スクリーンショット**
   - 現在のビューをキャプチャ
   - 解像度・フォーマット指定
   - base64エンコードで返却

### echidna連携
5. **バンドル開発サポート**
   - `echidna run` でビルド・インストール・起動
   - バンドルコマンドのテスト
   - ログの取得

6. **スモークテスト**
   - `scripts/smoke.cxc` の実行
   - 結果の確認

## 技術設計

### 通信方式
REST API (`remotecontrol rest start`) を採用
- シンプルなHTTP GET
- レスポンスがJSON形式で扱いやすい
- XML-RPCより現代的

### エンドポイント
```
http://127.0.0.1:{port}/run?command={encoded_command}
```

### MCPツール一覧 (案)

| ツール名 | 説明 |
|----------|------|
| `chimerax_start` | ChimeraXを起動 (REST有効化) |
| `chimerax_status` | 接続状態・バージョン確認 |
| `chimerax_run` | 任意のコマンドを実行 |
| `chimerax_models` | 開いているモデル一覧 |
| `chimerax_screenshot` | スクリーンショット取得 |
| `chimerax_session_save` | セッション保存 |
| `chimerax_session_open` | セッション読込 |
| `bundle_install` | バンドルをビルド・インストール |
| `bundle_test` | バンドルコマンドをテスト |

### 実装言語
**Python** (FastMCP使用)
- ChimeraX自体がPython
- 既存MCPサーバーもPython
- echidnaとの連携はsubprocessで

### ディレクトリ構成 (案)
```
chimerax-mcp/
├── pyproject.toml
├── src/
│   └── chimerax_mcp/
│       ├── __init__.py
│       ├── server.py      # FastMCPサーバー
│       ├── chimerax.py    # ChimeraX通信
│       ├── tools.py       # MCPツール定義
│       └── echidna.py     # echidna連携
├── tests/
└── README.md
```

## 実装ステップ

### Phase 1: 基本機能
1. プロジェクト作成 (uv init)
2. ChimeraX通信クラス (REST API)
3. 基本ツール: start, status, run
4. テスト

### Phase 2: 拡張機能
5. models, screenshot ツール
6. session_save, session_open ツール
7. エラーハンドリング強化

### Phase 3: echidna連携
8. bundle_install ツール
9. bundle_test ツール
10. ドキュメント

## 参考資料

- [ChimeraX Remote Control](https://www.cgl.ucsf.edu/chimerax/docs/user/tools/remotecontrol.html)
- [FastMCP Documentation](https://github.com/jlowin/fastmcp)
- [MCP Specification](https://spec.modelcontextprotocol.io/)
