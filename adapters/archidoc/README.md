# archidoc

Architecture documentation compiler. Extracts C4 structure from source annotations, produces documentation and LLM context, validates architecture conformance.

This npm package downloads the native `archidoc` binary and includes the TypeScript adapter (`archidoc-ts`). One install gives you full polyglot support for Rust + TypeScript projects.

## Install

```bash
npm install -D archidoc
```

## Usage

```bash
# Scan source → JSON IR
npx archidoc compile ir .

# Render documentation (from IR or directory)
npx archidoc render md _context/current.json
npx archidoc render context ./src/
npx archidoc render ai _context/current.json

# Validate architecture conformance
npx archidoc validate _context/architecture.json _context/current.json

# Scaffold a project template
npx archidoc scaffold --list
```

For polyglot projects (Rust + TypeScript), `archidoc` auto-detects both languages, runs the appropriate adapters, and merges the results.

## How It Works

1. On `npm install`, the postinstall script downloads the prebuilt native binary for your platform
2. The `archidoc-ts` TypeScript adapter is included as a dependency
3. When you run `archidoc compile ir .`, it auto-detects TypeScript sources and runs `archidoc-ts` internally
4. Results from all language adapters are merged into a unified ArchitectureIR v2.0

## Supported Platforms

| OS | Architecture |
|----|-------------|
| Linux | x64 |
| macOS | x64, arm64 |
| Windows | x64 |

## Alternative Install

```bash
# From crates.io (requires Rust)
cargo install archidoc-cli

# From source
git clone https://github.com/GitSmart86/archidoc
cd archidoc && cargo install --path core/archidoc-cli
```

## License

MIT
