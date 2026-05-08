# archidoc — Project Context for AI Assistants

## What This Is

archidoc is a C4 architecture documentation compiler. It extracts `@c4` annotations from source code comments, compiles them into ARCHITECTURE.md with Mermaid diagrams, and fails CI if docs drift from code.

## Commands

archidoc has **two subcommands** plus validation flags and a shorthand:

### `archidoc .` — Compile architecture docs (shorthand)

Parses `@c4` annotations from source → generates `ARCHITECTURE.md` + `ARCHITECTURE.ai.md`.

```bash
archidoc .                          # generate from annotations
archidoc . -o docs/ARCHITECTURE.md  # custom output
archidoc . --no-ai                  # skip AI context file
archidoc . --plantuml --drawio      # sidecar diagrams
```

### `archidoc init <handler> [dir]` — Generate files from context

Reads a directory and generates files using a named handler.

```bash
archidoc init --list                          # list handlers
archidoc init _index.md ./src/                # _index.md stubs for all dirs missing one
archidoc init _index.md ./src/ --dry-run      # list missing dirs
archidoc init c4-annotation ./src/api/        # @c4 annotation template (stdout)
archidoc init root-annotation .               # root-level lib.rs/index.ts template (stdout)
archidoc init tree .                          # dir tree → _context/ARCHITECTURE.ai.tree.md
archidoc init tree . --files --human --both   # all tree variants
archidoc init tree . --depth 3               # limit depth
```

### `archidoc scaffold <name> [--target dir] [--var k=v]` — Folder templates

Copies a named template tree from `.archidoc/templates/scaffold-templates/<name>/` with `{{variable}}` substitution.

```bash
archidoc scaffold --list                      # list templates (built-in + disk)
archidoc scaffold custom-scaffolds            # bootstrap .archidoc/scaffold-templates/
archidoc scaffold custom-inits                # bootstrap .archidoc/init-overrides/ with defaults
archidoc scaffold custom-trees                # bootstrap .archidoc/config.tree.json
archidoc scaffold <name> --inspect            # show vars + description
archidoc scaffold <name> --dry-run --var ...  # preview
archidoc scaffold <name> --target ./dir --var key=value --var key2=value2
archidoc scaffold <name> --force --var ...    # overwrite existing
```

### Validation flags (CI gates)

```bash
archidoc --check .      # exit 1 if ARCHITECTURE.md is stale
archidoc --validate .   # detect ghost/orphan files in tables
archidoc --health .     # report maturity + pattern confidence
archidoc --emit-ir .    # export JSON IR
```

All validation commands accept `--json` for machine-readable output.

## Crate Architecture

```
core/
  archidoc-types/       Shared domain types (ModuleDoc, C4Level, FileEntry, Relationship)
  archidoc-engine/      Generator engine + init handlers + folder scaffold
  archidoc-cli/         CLI binary (clap 4 derive)
  spec/                 JSON IR schema
  tests/                BDD test infrastructure
adapters/
  archidoc-rust/        Rust adapter (//! doc comments → ModuleDoc)
  archidoc-ts/          TypeScript adapter (@c4 JSDoc → JSON IR via npx)
```

## Key Engine Modules

| Module | Purpose |
|--------|---------|
| `architecture.rs` | ARCHITECTURE.md generator |
| `ai_context.rs` | ARCHITECTURE.ai.md (token-optimized, ~75% fewer tokens) |
| `mermaid.rs` | Mermaid C4 diagrams |
| `check.rs` | Drift detection (byte-compare generated vs committed) |
| `health.rs` | Health report (planned/active/stable counts) |
| `validate.rs` | Ghost/orphan file detection |
| `scaffold.rs` | _index.md stub generation |
| `tree.rs` | Directory tree generation |
| `init_cmd/` | Init handler registry (dispatches `archidoc init <handler>`) |
| `folder_scaffold/` | Folder template engine (discovers, substitutes, executes) |
| `custom.rs` | Template loading + `{{token}}` substitution |

## Annotation Convention

```rust
//! @c4 container
//! # Module Name
//! One-line description.
//! @c4 uses other_module "data flow description" "protocol"
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `file.rs` | Strategy | What it does | active |
```

- `@c4 container` or `@c4 component` — C4 level marker
- `@c4 uses target "label" "protocol"` — dependency declaration
- File table — one row per source file, GoF pattern + health status
- Health: `planned` → `active` → `stable`
- Patterns: any GoF name or `--`. Confidence: `planned` or `verified`

## Build & Test

```bash
cargo build                     # build all
cargo test                      # run all tests (87 tests)
cargo test -p archidoc-engine   # engine tests only (50 tests)
cargo install --path core/archidoc-cli  # install binary
```

## Template System

### Init overrides (`.archidoc/init-overrides/`)

Override the default output of built-in init handlers. Only files matching known handler output names are recognized: `_index.md`, `mod.rs`, `index.ts`, `architecture-table.md`. Unknown files are ignored.

### Scaffold templates (`.archidoc/scaffold-templates/`)

User-authored folder templates. Each subdirectory is a template — the folder name becomes the template name. Requires a `.archidoc-template.toml` manifest. Walk-up discovery finds templates at any ancestor `.archidoc/` directory.

```toml
[template]
name = "my-template"
description = "What it creates"
version = "0.1.0"

[[variables]]
name = "project_name"
description = "Name of the project"
required = true

[[post_hooks]]
command = "npm install"
description = "Post-scaffold action"
```
