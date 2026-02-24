---
sidebar_position: 18
---

# completions

Generate shell completions for `echi`.

## Usage

```bash
echi completions <SHELL>
```

## Supported Shells

- `bash`
- `zsh`
- `fish`
- `powershell`

## Setup

### Bash

```bash
echi completions bash > ~/.local/share/bash-completion/completions/echi
```

### Zsh

```bash
echi completions zsh > ~/.zfunc/_echi
```

Make sure `~/.zfunc` is in your `fpath`. Add to `~/.zshrc` if needed:

```bash
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
```

### Fish

```bash
echi completions fish > ~/.config/fish/completions/echi.fish
```

### PowerShell

```powershell
echi completions powershell > echi.ps1
. ./echi.ps1
```
