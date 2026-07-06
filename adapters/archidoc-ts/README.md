# archidoc-ts

TypeScript language adapter for [archidoc](https://github.com/GitSmart86/archidoc). Scans `index.ts` files for `@c4` JSDoc annotations and emits ArchitectureIR v2.0 JSON.

## Install

```bash
npm install -g archidoc-ts
```

## Usage

```bash
# Scan a TypeScript project → ArchitectureIR v2.0 JSON
archidoc-ts ./src > _context/archidoc/current.json

# Render documentation from the IR (requires archidoc Rust CLI)
archidoc render md _context/archidoc/current.json
archidoc render context _context/archidoc/current.json
archidoc render ai _context/archidoc/current.json

# Validate architecture conformance
archidoc validate _context/archidoc/architecture.json _context/archidoc/current.json
```

## Output Format

ArchitectureIR v2.0 — a nested directory tree where every node carries structure, strategy, and health:

```json
{
  "version": "2.0",
  "scan_root": "/path/to/project",
  "root": {
    "name": ".",
    "path": ".",
    "dirs": [
      {
        "name": "dashboard",
        "path": "dashboard",
        "c4_level": "container",
        "description": "Real-time trading dashboard",
        "pattern": "Mediator",
        "files": [
          { "name": "core.ts", "purpose": "Entry point", "health": "stable" }
        ]
      }
    ]
  }
}
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

- Emits ArchitectureIR v2.0 (nested tree with strategy + health)
- Parses `@c4 container` and `@c4 component` markers
- Extracts `@c4 uses target "label" "protocol"` relationships
- Parses file tables with GoF pattern labels and health status
- Auto-discovers import/export relationships between modules
- Compact JSON output (skips empty/undefined fields)

## Development

```bash
npm install
npm test        # 60 tests
npm run build   # Compile TypeScript
```
