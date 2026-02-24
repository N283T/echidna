# インストールスクリプトの追加

## 概要
`curl | sh` 形式のインストールスクリプトを追加し、一般ユーザーが簡単にインストールできるようにする。

## 要件
- OS/アーキテクチャ自動検出 (Linux, macOS x86_64/ARM64, Windows)
- GitHub Releasesから適切なバイナリをダウンロード
- `/usr/local/bin` または `~/.local/bin` に配置
- PATH設定が必要な場合は案内表示

## 使用方法
```bash
curl -sSfL https://raw.githubusercontent.com/N283T/echidna/master/install.sh | sh
```

## 参考
- rustup のインストーラ
- starship のインストーラ
