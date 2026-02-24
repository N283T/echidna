---
sidebar_position: 1
---

# Installation

## Quick Install (macOS / Linux)

```bash
curl -sSfL https://raw.githubusercontent.com/N283T/echidna/main/install.sh | sh
```

## From GitHub Releases

Download the latest binary from the [Releases](https://github.com/N283T/echidna/releases) page.

### macOS / Linux

```bash
# Download and extract (replace TARGET with your platform)
# Targets: x86_64-unknown-linux-gnu, x86_64-apple-darwin, aarch64-apple-darwin
curl -LO https://github.com/N283T/echidna/releases/latest/download/echi-TARGET.tar.gz
tar -xzf echi-TARGET.tar.gz
sudo mv echi /usr/local/bin/
```

### Windows

Download `echi-x86_64-pc-windows-msvc.zip` from [Releases](https://github.com/N283T/echidna/releases) and add to your PATH.

## From Source

Requires [Rust](https://www.rust-lang.org/tools/install) 1.85+ (edition 2024).

```bash
git clone https://github.com/N283T/echidna.git
cd echidna
cargo install --path .
```

## Verify Installation

```bash
echi --version
```
