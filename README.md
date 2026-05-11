# archidoc

Architecture documentation compiler — deterministic JSON metadata for directories.

Archidoc converts any file directory into a structured JSON representation (ArchitectureIR) that LLMs and tools can consume instantly — no searching the filesystem each time. It also creates directories deterministically from JSON templates, so your architecture plans become executable.

**What it does:**
1. **UNDERSTAND** — Scan any directory → JSON snapshot of structure + strategy + health
2. **CREATE** — Instantiate directory structures from JSON scaffold templates
3. **ANNOTATE** — Add strategy metadata (@c4 level, purpose, patterns) to directories
4. **VALIDATE** — Catch architectural drift — diff design intent vs implementation

## Install

```bash
cargo install --path core/archidoc-cli
```

## Quick Start

```bash
archidoc compile ir .                  # scan → _context/current.json
archidoc render ai-structure .         # instant LLM context (one step)
archidoc render human-strategy _context/current.json
```

## Commands

### compile — Extract JSON from source (Phase 1)

```bash
archidoc compile ir .                        # scan → _context/current.json
archidoc compile ir . --design               # → _context/architecture.json (all health=planned)
archidoc compile scaffold ./templates/firm/  # folder template → ScaffoldIR JSON
```

Two output types:
- `current.json` — implementation state (what exists on disk right now)
- `architecture.json` — target truth (the declared architecture, all health starts at `planned`)

### render — Produce documentation from IR (Phase 2)

Accepts a `.json` IR file or a directory (auto-compiled on the fly).

```bash
archidoc render ai-files ./src/                     # every file per directory
archidoc render ai-structure .                      # compressed tree + strategy + health
archidoc render ai-strategy _context/current.json   # module narrative + relationships
archidoc render human-strategy _context/current.json  # ARCHITECTURE.md
archidoc render plantuml _context/current.json      # C4 diagrams
archidoc render drawio _context/current.json        # draw.io CSV
```

| Format | Output file | Purpose |
|--------|------------|----------|
| `ai-files` | ai-files.md | Every file listed explicitly per dir — for LLM grep/search |
| `ai-structure` | ai-structure.md | Compressed tree + strategy + health — for LLM cold-start |
| `ai-strategy` | ai-strategy.md | Module narrative + relationships — for architecture decisions |
| `human-strategy` | ARCHITECTURE.md | Full human-readable doc with Mermaid C4 diagrams |
| `human-structure` | human-structure.md | Human structure overview (reserved) |
| `plantuml` | diagrams/c4.puml | PlantUML C4 diagrams |
| `drawio` | diagrams/c4.drawio.csv | draw.io CSV import |

### validate — Check architecture conformance

```bash
archidoc validate _context/architecture.json _context/current.json
archidoc validate _context/architecture.json _context/current.json --strict
archidoc validate _context/architecture.json _context/current.json --log
```

Findings:
- **UNIMPLEMENTED** — in architecture but not current (planned work not done)
- **UNDOCUMENTED** — in current but not architecture (undocumented growth)
- **DIVERGED** — same path, conflicting attributes (C4 level, pattern)
- **REGRESSION** — health moved backward (stable → active)

### scaffold — Instantiate a template

```bash
archidoc scaffold --list
archidoc scaffold client --var client_name=Acme
archidoc scaffold project --dry-run --var engagement_name=Test
```

Convert existing folder templates to JSON:
```bash
archidoc compile scaffold .archidoc/scaffold-templates/firm/ --output-dir .archidoc
```

### annotate — Add @c4 entry points

```bash
archidoc annotate rs ./src/auth/        # create mod.rs with @c4 block
archidoc annotate md . --recursive      # create _index.md in all subdirs
```

## For Greenfield Projects

```bash
archidoc compile ir . --design           # declare target architecture
archidoc scaffold project --var ...      # create structure from template
archidoc validate arch.json curr.json    # ensure implementation tracks design
```

## For Brownfield Projects

```bash
archidoc compile ir .                    # snapshot existing structure → JSON
archidoc render ai-structure .           # instant LLM-ready project summary
archidoc annotate md . --recursive       # add strategy metadata to all dirs
```

## Annotation Format

```rust
//! @c4 container
//!
//! Central messaging backbone.
//!
//! @c4 uses database "Persists data" "sqlx"
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `lanes.rs` | Observer | Event routing | active |
//! | `store.rs` | Repository | Lock-free cache | stable |
```

## JSON IR (v2.0)

A nested directory tree where each node carries structure, strategy, and health:

```json
{
  "version": "2.0",
  "scan_root": "/path/to/project",
  "root": {
    "name": ".",
    "path": ".",
    "c4_level": "container",
    "description": "REST API gateway",
    "dirs": [
      {
        "name": "api",
        "path": "api",
        "c4_level": "component",
        "description": "HTTP route handlers",
        "files": [
          { "name": "mod.rs", "purpose": "Module entry", "health": "stable" }
        ]
      }
    ]
  }
}
```

## CI Integration

```yaml
- run: archidoc compile ir .
- run: archidoc validate _context/architecture.json _context/current.json --strict
- run: archidoc render human-strategy _context/current.json --validate
```

## Tests

```bash
cargo test
```

## License

MIT
