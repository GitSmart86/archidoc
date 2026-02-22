# archidoc

Architecture documentation compiler. Generates C4 diagrams (Mermaid, PlantUML, draw.io) from source code annotations.

This npm package downloads the native `archidoc` binary and includes the TypeScript adapter (`archidoc-ts`). One install gives you full polyglot support for Rust + TypeScript projects.

## Install

```bash
npm install -D archidoc
```

## Usage

```bash
# Generate ARCHITECTURE.md from source annotations
npx archidoc .

# Check for documentation drift (CI gate)
npx archidoc --check .

# Architecture health report
npx archidoc --health .

# Validate file tables
npx archidoc --validate .
```

For polyglot projects (Rust + TypeScript), `archidoc` auto-detects both languages, runs the appropriate adapters, and merges the results. No manual steps needed.

## How It Works

1. On `npm install`, the postinstall script downloads the prebuilt native binary for your platform from [GitHub Releases](https://github.com/GitSmart86/archidoc/releases)
2. The `archidoc-ts` TypeScript adapter is included as a dependency
3. When you run `archidoc .`, it auto-detects TypeScript sources (via `package.json`) and runs `archidoc-ts` internally
4. Results from all language adapters are merged into unified architecture documentation

## Supported Platforms

| OS | Architecture |
|----|-------------|
| Linux | x64 |
| macOS | x64, arm64 |
| Windows | x64 |

## Alternative Install Methods

```bash
# From crates.io (requires Rust toolchain)
cargo install archidoc-cli

# From source
git clone https://github.com/GitSmart86/archidoc
cd archidoc && cargo install --path core/archidoc-cli
```

## Documentation

See the [archidoc repository](https://github.com/GitSmart86/archidoc) for the full annotation convention, CLI reference, and language adapter details.

## License

MIT
