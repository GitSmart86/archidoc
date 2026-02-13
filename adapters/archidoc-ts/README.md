# archidoc-ts

TypeScript language adapter for [archidoc](https://github.com/archidoc/archidoc). Parses `@c4` JSDoc annotations from `index.ts` files and emits JSON IR (`ModuleDoc[]`) to stdout.

## Install

```bash
npm install -g archidoc-ts
```

## Usage

```bash
# Emit JSON IR from a TypeScript project
archidoc-ts ./src > ir.json

# Generate ARCHITECTURE.md using the core engine
archidoc --from-json-file ir.json .

# Or pipe directly (Unix)
archidoc-ts ./src | archidoc --from-json .
```

## Annotation Format

Annotate each module's `index.ts` with JSDoc containing `@c4` markers:

```typescript
/**
 * @c4 container
 *
 * Dashboard UI — real-time data visualization.
 *
 * @c4 uses api "Fetches data" "REST/HTTP"
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `charts.ts` | Observer | Chart rendering | active |
 * | `state.ts` | Facade | State management | stable |
 */
```

## Features

- Parses `@c4 container` and `@c4 component` markers
- Extracts `@c4 uses target "label" "protocol"` relationships
- Parses file tables with GoF pattern labels and health status
- Auto-discovers import/export relationships between modules
- Emits JSON conforming to the archidoc IR schema

## Polyglot Projects

Combine with the Rust adapter for mixed-language codebases:

```bash
archidoc --emit-ir ./backend/src > rust.json
archidoc-ts ./frontend/src > ts.json
archidoc --merge-ir --from-json-file rust.json --from-json-file ts.json .
```

## Development

```bash
npm install
npm test        # Run tests (54 tests)
npm run build   # Compile TypeScript
```
