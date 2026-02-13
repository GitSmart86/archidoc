# archidoc

[![CI](https://github.com/archidoc/archidoc/actions/workflows/ci.yml/badge.svg)](https://github.com/archidoc/archidoc/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/archidoc-cli.svg)](https://crates.io/crates/archidoc-cli)
[![npm](https://img.shields.io/npm/v/archidoc-ts.svg)](https://www.npmjs.com/package/archidoc-ts)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Your architecture diagrams are always wrong because nobody updates them. archidoc fixes this — it extracts C4 architecture documentation directly from source code annotations, so your diagrams stay in sync with your code. If they drift, `archidoc --check` fails your CI build.

## What It Does

Developers annotate module entry files (`mod.rs`, `index.ts`, `__init__.py`) with structured comments containing C4 markers, GoF pattern labels, and file-level responsibility tables. archidoc compiles these annotations into a single **ARCHITECTURE.md** containing:

- **Inline Mermaid C4 diagrams** (container and component levels)
- **Component index table** (linking directly to source files)
- **Relationship map** (dependency arrows with labels and protocols)
- **JSON IR** (portable intermediate representation for cross-language pipelines)

Optional sidecar outputs: **PlantUML** (`--plantuml`) and **draw.io CSV** (`--drawio`).

It also detects **architecture drift** (docs out of sync with code), validates **file tables** (ghost/orphan detection), and reports **architecture health** (pattern confidence, file maturity).

## Install

```bash
# From crates.io
cargo install archidoc-cli

# Or from source
cargo install --path core/archidoc-cli

# Or build locally
cargo build --release
# Binary at target/release/archidoc
```

```bash
# TypeScript adapter (npm)
npm install archidoc-ts
```

## Usage

```bash
# Generate ARCHITECTURE.md from source annotations
archidoc .

# Custom output path
archidoc . -o docs/ARCHITECTURE.md

# Also generate PlantUML and/or draw.io sidecar files
archidoc . --plantuml --drawio

# Check for documentation drift (CI gate — exits non-zero on drift)
archidoc --check .

# Print architecture health report
archidoc --health .

# Export JSON IR for cross-language pipelines
archidoc --emit-ir .

# Generate ARCHITECTURE.md from JSON IR (any language adapter)
archidoc --from-json-file ir.json .

# Merge IR from multiple adapters (polyglot projects)
archidoc --merge-ir --from-json-file rust.json --from-json-file ts.json .

# Validate IR against schema
archidoc --from-json-file ir.json --validate-ir

# Generate annotation template for an unannotated directory
archidoc suggest src/api/

# Write the template directly into a module entry file
archidoc suggest src/api/ >> src/api/mod.rs
```

## Annotation Convention

Container-level (`mod.rs`):

```rust
//! @c4 container
//!
//! # Bus Module
//!
//! Central messaging and caching backbone.
//!
//! @c4 uses agents_internal "Processed market data" "crossbeam channel"
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `lanes.rs` | Observer | Event routing channels | active |
//! | `store.rs` | Repository | Lock-free cache | stable |
```

TypeScript (`index.ts`):

```typescript
/**
 * @c4 container
 *
 * Central messaging and caching backbone.
 *
 * @c4 uses agents_internal "Processed market data" "WebSocket"
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `core.ts` | Facade | Entry point | stable |
 */
```

### C4 Markers

- `@c4 container` — marks a C4 container
- `@c4 component` — marks a C4 component
- `@c4 uses target "label" "protocol"` — declares a dependency

### File Table

Each row declares a file in the module with its GoF pattern, purpose, and health status:

| Column | Values |
|--------|--------|
| **Pattern** | Any GoF pattern name, or `--` for none |
| **Health** | `planned` (not yet implemented), `active` (in progress), `stable` (complete) |

### Pattern Confidence

Pattern labels have two tiers:
- **planned** — developer's stated intent
- **verified** — structurally confirmed by heuristic analysis (Observer, Strategy, Facade)

## Getting Started

To adopt archidoc on an existing project:

1. **Pick your top-level modules** — identify the 3-5 directories that represent your system's major containers (e.g. `api/`, `core/`, `database/`)

2. **Add a C4 marker** to each module's entry file (`mod.rs`, `index.ts`, or `__init__.py`):
   ```rust
   //! @c4 container
   //!
   //! # Api
   //!
   //! REST API gateway — handles authentication and request routing.
   ```

3. **Run archidoc** to generate your first diagrams:
   ```bash
   archidoc .
   ```

4. **Add relationships** between containers:
   ```rust
   //! @c4 uses database "Persists user data" "sqlx"
   ```

5. **Add file tables** to document each module's internal structure:
   ```rust
   //! | File | Pattern | Purpose | Health |
   //! |------|---------|---------|--------|
   //! | `routes.rs` | -- | HTTP route handlers | active |
   //! | `middleware.rs` | Strategy | Auth and rate limiting | stable |
   ```

6. **Gate your CI** to prevent architecture drift:
   ```bash
   archidoc --check .
   ```

Start with containers only. Add components and file tables as the architecture stabilizes.

## Project Structure

```
Cargo.toml              Workspace root
core/
  archidoc-types/       Shared types (ModuleDoc, C4Level, FileEntry, Relationship, etc.)
  archidoc-engine/      Language-agnostic generator (ARCHITECTURE.md, mermaid, plantuml, draw.io, IR, drift, health)
  archidoc-cli/         CLI binary: archidoc
  spec/                 JSON IR schema
  tests/                BDD test infrastructure (DSL, protocol drivers, fakes)
adapters/
  archidoc-rust/        Rust adapter (//! doc comments -> ModuleDoc)
  archidoc-ts/          TypeScript adapter (@c4 JSDoc -> JSON IR)
docs/                   Guides (annotation spec, getting started, LLM context)
examples/               Example annotated projects
```

## Architecture

archidoc follows a three-layer architecture:

1. **Types** (`archidoc-types`) — shared domain model: `ModuleDoc`, `C4Level`, `FileEntry`, `Relationship`, `PatternStatus`, `HealthStatus`
2. **Adapters** (`archidoc-rust`, `archidoc-ts`) — language-specific parsers that extract annotations and emit `ModuleDoc` arrays
3. **Engine** (`archidoc-engine`) — language-agnostic generators that consume `ModuleDoc` and produce ARCHITECTURE.md, diagrams, IR, drift reports, and health summaries

The CLI orchestrates: adapter parses source -> engine generates ARCHITECTURE.md.

### JSON IR

The intermediate representation (`ModuleDoc[]` as JSON) is the contract between adapters and the engine. Any language adapter that emits conforming JSON can use the full engine pipeline. See `core/spec/archidoc-ir-schema.json` for the schema.

## Writing a Language Adapter

To add support for a new language:

1. Scaffold with `archidoc init-adapter --lang python`
2. Implement a parser that extracts annotations from your language's comment format
3. Implement a walker that traverses source directories and collects `ModuleDoc` entries
4. Emit `ModuleDoc[]` JSON to stdout — the engine handles the rest

See the `archidoc-rust` and `archidoc-ts` adapters for reference implementations.

## Tests

```bash
# Run all Rust tests (131 tests)
cargo test

# Run TypeScript adapter tests (54 tests)
cd adapters/archidoc-ts && npm test
```

185 tests total across both platforms.

The test suite uses Dave Farley-style BDD: declarative test cases specify WHAT (behavior), protocol drivers translate to HOW (implementation). When the implementation changes, update drivers — not tests.

## License

MIT
