# Annotation Specification

Authoritative reference for the archidoc annotation convention.

## Overview

archidoc annotations are structured comments placed in module entry files. They declare C4 architecture levels, inter-module relationships, and file-level design metadata. The annotations serve as the single source of truth for architecture documentation.

## C4 Level Markers

Each annotated module must declare its C4 architecture level.

### Rust Syntax

Use `@c4` markers:

```rust
//! @c4 container
//!
//! # Bus
//!
//! Central messaging backbone for cross-module communication.
```

The `@c4` syntax avoids rustdoc HTML tag warnings and provides consistent syntax across all language adapters.

### TypeScript Syntax

Use `@c4` JSDoc tags:

```typescript
/** @c4 container */
/** @c4 component */
```

### Values

| Value | Meaning |
|-------|---------|
| `container` | A deployable unit or top-level subsystem |
| `component` | A sub-module within a container |
| `unknown` | Default when no marker is present |

## Relationship Markers

Declare runtime dependencies between modules.

### Rust Syntax

```
@c4 uses target "label" "protocol"
```

- `target`: dot-notation module path of the dependency
- `label`: description of the data flow (quoted string)
- `protocol`: communication mechanism (quoted string)

Example:

```rust
//! @c4 uses database "Persists user data" "sqlx"
//! @c4 uses events "Domain events" "crossbeam channel"
```

### TypeScript Syntax

```
@c4 uses target "label" "protocol"
```

Example:

```typescript
/** @c4 uses api "Fetches data" "REST/HTTP" */
```

### Notes

- All three fields (target, label, protocol) are required
- A module may declare zero or more relationships
- Relationships are directional: the declaring module depends on the target

## File Table Format

Each module may include a markdown table documenting its constituent files.

### Format

```
| File | Pattern | Purpose | Health |
|------|---------|---------|--------|
| `filename.ext` | PatternName | Description | status |
```

### Column Definitions

| Column | Type | Description |
|--------|------|-------------|
| File | string | Filename in backticks (e.g., `` `routes.rs` ``) |
| Pattern | string | GoF pattern name or `--` |
| Purpose | string | One-line responsibility description |
| Health | enum | Implementation maturity |

### Recognized GoF Pattern Names

Mediator, Observer, Strategy, Facade, Adapter, Repository, Singleton, Factory, Active Object, Memento, Command, Chain of Responsibility, Registry, Composite, Interpreter, Flyweight, Publisher, Builder, Decorator.

Use `--` when no GoF pattern applies.

### Pattern Status

| Value | Meaning |
|-------|---------|
| `planned` | Developer's stated intent (default) |
| `verified` | Structurally confirmed by heuristic analysis |

To mark a pattern as verified, append `(verified)` to the pattern name: `Strategy (verified)`.

Automatic verification is supported for: Observer, Strategy, Facade, Builder, Factory, Adapter, Decorator, Singleton, Command.

### Health Status

| Value | Meaning |
|-------|---------|
| `planned` | Not yet implemented |
| `active` | Under active development |
| `stable` | Complete and tested |

## Module Entry Files

archidoc scans specific files per language convention:

| Language | Entry File | Comment Prefix |
|----------|-----------|----------------|
| Rust | `mod.rs`, `lib.rs`, `*.rs` (with C4 markers) | `//!` |
| TypeScript | `index.ts` | `/** ... */` |
| Python | `__init__.py` | `"""..."""` |

Only the first documentation block in each entry file is parsed.

### Rust Flat Module Support

Modern Rust allows declaring modules as standalone `.rs` files instead of `mod.rs`:

- **Traditional**: `src/foo/mod.rs` declares module `foo`
- **Flat**: `src/foo.rs` declares module `foo`
- **Nested flat**: `src/foo/bar.rs` declares module `foo.bar`

The Rust adapter recognizes flat modules if they contain C4 markers (`@c4 container` or `@c4 component`). Files without C4 markers are skipped (unless they are `mod.rs` or `lib.rs`).

If both `src/foo/mod.rs` and `src/foo.rs` exist, `mod.rs` takes priority.

## Module Path Derivation

Module paths use dot-notation derived from the directory hierarchy relative to the project root:

| File Path | Module Path |
|-----------|-------------|
| `src/api/mod.rs` | `api` |
| `src/api/auth/mod.rs` | `api.auth` |
| `src/bus/calc/mod.rs` | `bus.calc` |
| `src/router.rs` | `router` (flat module at root) |
| `src/bus/events.rs` | `bus.events` (nested flat module) |
| `lib.rs` | `_lib` |
| `src/dashboard/index.ts` | `dashboard` |
| `src/dashboard/charts/index.ts` | `dashboard.charts` |

### Parent Container

The first segment of a module path is the parent container:

| Module Path | Parent Container |
|-------------|-----------------|
| `api` | *(none)* |
| `api.auth` | `api` |
| `bus.calc.indicators` | `bus` |

## Description Extraction

The description is the first non-empty line that is:
- Not a heading (`#`)
- Not a C4 marker (`@c4`)
- Not a table row (`|`)
- Not a GoF label (`GoF:`)

This line appears in generated diagrams and documentation indexes.

## Validation Rules

### Ghost Detection

A file table entry pointing to a file that does not exist on disk. Detected by `archidoc --validate`.

### Orphan Detection

A `.rs` or `.ts` file on disk that is not listed in any file table. Structural files (`mod.rs`, `lib.rs`, `main.rs`) are excluded from orphan detection.

### Drift Detection

Generated documentation that does not match the current source annotations. Detected by `archidoc --check`, which exits non-zero on drift.
