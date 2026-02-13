# Rust Example

A minimal Rust project annotated with archidoc C4 markers.

## Structure

```
src/
  api/mod.rs        @c4 container — REST API gateway
  database/mod.rs   @c4 container — Persistence layer
  events/mod.rs     @c4 container — Event bus
```

Each `mod.rs` contains C4 annotations, file tables, and relationship declarations.

## Generate ARCHITECTURE.md

```bash
# From the archidoc repo root:
archidoc examples/rust-example/src -o examples/rust-example/ARCHITECTURE.md

# Or from this directory:
archidoc src
```

## Check for Drift

```bash
archidoc --check src
```

## View Health Report

```bash
archidoc --health src
```
