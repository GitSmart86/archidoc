# archidoc for LLMs

A structured reference for AI coding assistants to annotate any project with archidoc-compatible C4 architecture documentation.

## ARCHITECTURE.ai.md — Token-Optimized Context

archidoc generates `ARCHITECTURE.ai.md` alongside `ARCHITECTURE.md` by default. This file is designed for LLM consumption:

- **~75% fewer tokens** than the human-readable version
- Same architectural information — module tree, GoF patterns, descriptions, relationships
- No Mermaid diagrams, ASCII art, markdown tables, or repeated descriptions
- Each module appears exactly once in an indented tree format

**Format example:**
```
# Architecture (AI Context)

# My Trading App

Day trading backend integrating IBKR and Alpaca brokers.

## Data Flow

1. Frontend -> Router -> Engine -> Brokers
2. Brokers -> Bus -> Internal Agents -> Frontend

agents_external/ Adapter — Broker adapters for trading APIs.
  alpaca/ Observer — Alpaca REST+WS adapter.
  ibkr/ Observer — IBKR TWS/Gateway adapter.
bus/ Mediator — Central messaging backbone.
  calc/ Strategy — Candle aggregation and indicators.
  store/ Repository — Lock-free market data cache.
engine/ Mediator — Core orchestration layer.
```

**Usage in CLAUDE.md or similar AI context files:**
```markdown
## Architecture Reference
See `docs/ARCHITECTURE.ai.md` for the full module tree with GoF patterns.
```

To suppress generation, use `archidoc --no-ai`.

## What archidoc Expects

archidoc extracts structured documentation from **module entry files** — one per directory. Each entry file contains a documentation block with C4 markers, relationships, and a file table. archidoc compiles these into Mermaid C4 diagrams, markdown docs, and portable JSON IR.

## Annotation Syntax Reference

### Module Entry Files

| Language | Entry File | Comment Style |
|----------|-----------|---------------|
| Rust | `mod.rs` or `lib.rs` | `//!` inner doc comments |
| TypeScript | `index.ts` | `/** ... */` JSDoc block |
| Python | `__init__.py` | `"""..."""` module docstring |

### C4 Level Markers

Mark each module as a C4 container or component.

**Rust**:
```rust
//! @c4 container
//! @c4 component
```

**TypeScript**:
```typescript
/** @c4 container */
/** @c4 component */
```

**When to use which**:
- `container` — top-level subsystem directory (e.g., `src/api/`, `src/database/`)
- `component` — sub-module within a container (e.g., `src/api/auth/`, `src/api/routes/`)

### Relationship Markers

Declare runtime dependencies between modules.

**Rust**:
```rust
//! @c4 uses target_module "description of data flow" "protocol or technology"
```

**TypeScript**:
```typescript
/** @c4 uses target_module "description of data flow" "protocol or technology" */
```

- `target_module`: dot-notation module path of the dependency
- Description: what data flows between the modules
- Protocol: how the communication happens (e.g., `"HTTP"`, `"gRPC"`, `"channel"`, `"sqlx"`)

### File Table

Document each file in the module directory.

```
| File | Pattern | Purpose | Health |
|------|---------|---------|--------|
| `filename.rs` | PatternName | One-line description | status |
```

**Valid Pattern names**: Mediator, Observer, Strategy, Facade, Adapter, Repository, Singleton, Factory, Active Object, Memento, Command, Chain of Responsibility, Registry, Composite, Interpreter, Flyweight, Publisher, Builder, Decorator. Use `--` if no pattern applies.

**Valid Health statuses**: `planned` (not yet implemented), `active` (in development), `stable` (complete and tested).

**Valid Pattern statuses**: `planned` (default), `verified` (structurally confirmed by heuristics). Use `(verified)` suffix: `Strategy (verified)`.

### Module Path Convention

Module paths use dot-notation derived from directory hierarchy relative to `src/`:

```
src/api/mod.rs           -> api
src/api/auth/mod.rs      -> api.auth
src/bus/calc/mod.rs      -> bus.calc
lib.rs                   -> _lib
```

### Parent Container Derivation

The first segment of a dot-notation path is the parent container:
- `api.auth` -> parent is `api`
- `bus.calc.indicators` -> parent is `bus`
- `api` -> no parent (top-level container)

## Step-by-Step Instructions

Given a codebase to annotate:

### 1. Find entry files

Scan for `mod.rs`, `lib.rs`, `index.ts`, or `__init__.py` files. Each represents a potential architectural element.

### 2. Determine C4 level

- If the directory is directly under `src/`, it's usually a `container`
- If it's nested inside another module, it's usually a `component`
- `lib.rs` at the crate root is special — use module path `_lib`

### 3. Write the description

The description is the first non-empty, non-marker line. It should be a single sentence explaining what this module does. Use active voice.

Good: `"REST API gateway — handles authentication and request routing."`
Bad: `"This module contains the API code."`

### 4. List relationships

For each `use`/`import` that crosses module boundaries, add a `@c4 uses` relationship marker. Focus on runtime dependencies, not dev/test imports.

### 5. Build the file table

List every `.rs` or `.ts` file in the directory (excluding `mod.rs`, `lib.rs`, `main.rs`, `index.ts`). For each file:
- Identify the GoF pattern if one applies, otherwise use `--`
- Write a one-line purpose description
- Set health: `stable` if complete, `active` if under development, `planned` if not yet implemented

### 6. Validate

```bash
archidoc --validate .    # Check for ghost/orphan files
archidoc --check .       # Check for documentation drift
archidoc --health .      # View architecture health summary
```

## Scaffolding Commands

archidoc provides two scaffolding commands to bootstrap annotations:

### `archidoc init` — Root-level template

Generates a project-level `lib.rs` / `index.ts` doc comment with sections for purpose, C4 context diagram, data flow, concurrency patterns, deployment, and external dependencies. Each section has TODO placeholders.

```bash
archidoc init              # auto-detects language from Cargo.toml / package.json
archidoc init --lang rust  # explicit Rust
archidoc init --lang ts    # explicit TypeScript
```

The C4 Context mermaid diagram in the template renders in `ARCHITECTURE.md` but is automatically stripped from `ARCHITECTURE.ai.md` (code blocks are excluded from the AI format).

### `archidoc suggest <dir>` — Module-level template

Generates a `@c4 container` or `@c4 component` annotation (auto-detected from directory depth) with a file table listing all source files in the directory.

```bash
archidoc suggest src/api/                    # prints to stdout
archidoc suggest src/api/ >> src/api/mod.rs  # append to entry file
```

## Template

### Rust Container Template

```rust
//! @c4 container
//!
//! # ModuleName
//!
//! One-line description of this module's responsibility.
//!
//! @c4 uses other_module "What data flows" "protocol"
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `file1.rs` | -- | What this file does | active |
//! | `file2.rs` | Strategy | What this file does | stable |
```

### TypeScript Container Template

```typescript
/**
 * @c4 container
 *
 * One-line description of this module's responsibility.
 *
 * @c4 uses other_module "What data flows" "protocol"
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `file1.ts` | -- | What this file does | active |
 * | `file2.ts` | Facade | What this file does | stable |
 */
```

## Common Pitfalls

1. **Wrong C4 level**: Don't make everything a container. Nested modules should be components.
2. **Missing relationships**: If module A calls module B at runtime, add a `@c4 uses` marker.
3. **Wrong health status**: `stable` means "done and tested", not "exists". New files start as `active` or `planned`.
4. **Pattern names are case-sensitive**: `Strategy` not `strategy`. `Observer` not `observer`.
5. **Forgetting the separator row**: The file table needs `|------|---------|---------|--------|` after the header.
6. **Ghost entries**: Don't list files that don't exist yet unless their health is `planned`.
7. **Missing protocol**: Every relationship needs a protocol field, even if it's just `"internal"` or `"function call"`.

## JSON IR Schema

The compiled output is a `ModuleDoc[]` JSON array. See `core/spec/archidoc-ir-schema.json` for the formal schema. Key constraints:
- `c4_level` must be `"container"`, `"component"`, or `"unknown"`
- `pattern_status` must be `"planned"` or `"verified"`
- `health` in file entries must be `"planned"`, `"active"`, or `"stable"`
- All fields are required (use `null` for `parent_container` on top-level elements)
