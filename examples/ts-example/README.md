# TypeScript Example

A minimal TypeScript project annotated with archidoc `@c4` JSDoc markers.

## Structure

```
src/
  dashboard/index.ts   @c4 container — Dashboard UI
  websocket/index.ts   @c4 container — WebSocket client
```

Each `index.ts` contains `@c4` JSDoc annotations, file tables, and relationship declarations.

## Generate JSON IR

The TypeScript adapter emits JSON IR, which the core engine consumes:

```bash
# Install the TS adapter
npm install -g archidoc-ts

# Emit IR from this example
archidoc-ts src > ir.json

# Generate ARCHITECTURE.md from the IR
archidoc --from-json-file ir.json .
```

## Polyglot: Combine with Rust

```bash
# Emit IR from both adapters
archidoc --emit-ir ../rust-example/src > rust.json
archidoc-ts src > ts.json

# Merge into a unified ARCHITECTURE.md
archidoc --merge-ir --from-json-file rust.json --from-json-file ts.json .
```
