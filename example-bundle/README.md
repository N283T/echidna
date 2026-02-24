# ChimeraX-HelloWorld

ChimeraX HelloWorld bundle

## Installation

### From source (development)

```bash
# Using echidna
echidna install

# Or directly with ChimeraX
ChimeraX --nogui --exit --cmd 'devel install .'
```

### From wheel

```bash
ChimeraX --nogui --exit --cmd 'toolshed install dist/ChimeraX-HelloWorld-0.1.0-py3-none-any.whl'
```

## Development

```bash
# Build wheel
echidna build

# Install to ChimeraX
echidna install

# Build, install, and launch ChimeraX
echidna run

# Run with a test script
echidna run --script scripts/smoke.cxc
```

## License

MIT
