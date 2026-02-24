# Echidna 改善メモ

実際に使ってみて気づいた改善点をメモしていく。

---

## インストール/配布
- [x] install.sh スクリプト追加 (PR #35)

## CLI/UX
- [x] `echidna init <name>` で `uv init` のようなUIに (PR #31)
- [x] Auto-detected ChimeraX 時にバージョンも表示する (PR #32)
- [x] シンボリックリンクの場合はその旨とリンク先パスを表示する (PR #32)
- [x] 「ChimeraXを探しています...」等の説明を表示 (PR #32)
- [x] `Build successful!` 等の成功メッセージに色をつける (PR #33)
- [x] コマンド成功後に次のステップを提案する (PR #33)
- [x] テンプレートの `smoke.cxc` に実際のサンプルコマンドを入れておく (PR #33)
- [x] `echidna run --nogui` は既にある（ハイフンなし）
- [x] `echidna run` と `echidna test` の役割を明確化 (PR #34)
  - `test` → pytest (default)
  - `test --smoke` → smoke.cxc実行
- [x] `echidna run` 後に「Try running `hello_world` in ChimeraX command line」ヒント表示 (PR #34)
- [x] `setup-ide` → `venv` にリネーム (PR #34)

## ドキュメント

## バグ
- [x] `echidna test` でpanic (PR #30)
- [x] `echidna setup-ide` でエラー (PR #30)

## 新機能アイデア

