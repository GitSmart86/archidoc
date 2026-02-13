# DocAuto — Development Guidelines

## What This Project Is

A self-documenting architecture toolkit that unifies **Dave Farley BDD**, **Simon Brown C4**, and **outside-in development**. It ships as three independent layers:

- **Layer 1** — Annotation convention (zero-dependency comment format)
- **Layer 2** — CLI tools (`archidoc-engine` + language adapters)
- **Layer 3** — Claude Code skills (AI workflow automation)

## Architecture

```
Cargo.toml                          ← Workspace root
core/
  archidoc-types/               ← Shared types (ModuleDoc, enums)
  archidoc-engine/              ← Language-agnostic generator engine. Reads JSON IR, produces docs.
  archidoc-cli/                 ← CLI facade. Orchestrates adapter + engine. Binary: `archidoc`.
  tests/                            ← BDD test infrastructure (DSL, drivers, fakes)
adapters/
  archidoc-rust/                ← Rust adapter. Reads //! comments, emits JSON IR.
  archidoc-ts/                  ← TS adapter. Reads /** @c4 */ JSDoc, emits JSON IR.
```

The JSON IR (`ModuleDoc[]`) is the contract between adapters and core. See deliverables.md §2.4 for schema.

## How to Prepare for Implementation

When asked to "prepare to implement" or "start implementation":

1. Review the implementation checklist below
2. Identify which phase to start and confirm with the user

## Development Cycle

Each implementation phase follows this cycle:

```
1. Write acceptance test (what does "done" look like?)
2. Design the interface (types, traits, function signatures)
3. Implement the minimum code to pass the test
4. Run archidoc --check (if applicable) to validate
5. Commit with a message referencing the deliverable being addressed
```

## Code Style

- Rust for all core and adapter code
- TypeScript (Node/Bun) for the TS adapter only
- No backwards compatibility — prefer breaking changes over legacy shims
- Delete old code rather than commenting it out
- Fail fast, explicit error handling, no silent failures
- Prefer simple readable code — avoid over-engineering
- Write tests first. Offer test designs before writing production code.

## Implementation Checklist

### Phase A: Formalize Convention (Deliverable 1.1)

- [x] A1: Write `c4-annotation-spec.md` — the one-page annotation specification
- [x] A2: Define `<<container>>`, `<<component>>`, `<<uses: target, "label", "protocol">>` marker syntax
- [x] A3: Define file table format (`| File | Pattern | Purpose | Health |`)
- [x] A4: Define two-tier pattern labels (`planned` / `verified`)
- [x] A5: Define health indicators (`planned` → `active` → `stable`)
- [x] A6: Create `mod.rs` template for Rust modules
- [x] A7: Create `index.ts` template for TypeScript modules
- [x] A8: Create `__init__.py` template for Python modules (stretch)

### Phase B: Harden Compiler — Validation (Deliverables 2.1, 2.2)

- [x] B1: Implement `--check` mode — generate to temp dir, diff against existing, exit non-zero on drift
- [x] B2: Implement file existence validation — parse file tables, cross-check against directory contents
- [x] B3: Implement orphan detection — flag `.rs` files not listed in any file table
- [x] B4: Implement ghost detection — flag file table entries pointing to deleted files
- [x] B5: Implement fitness function wrapper — `#[test] fn architecture_docs_are_current()`
- [x] B6: Implement `--health` mode — aggregate planned/active/stable counts project-wide
- [x] B7: Wire `--check` into pre-commit hook example

### Phase C: Harden Compiler — Relationships (Deliverables 2.1, 2.2)

- [x] C1: Parse `<<uses: target, "label", "protocol">>` markers from `//!` comments
- [x] C2: Generate `Rel()` arrows in Mermaid C4 diagrams from `<<uses>>` markers
- [x] C3: Remove all hardcoded relationship strings from `mermaid.rs`
- [x] C4: Remove all hardcoded relationship strings from `drawio.rs`
- [ ] C5: Integrate `cargo-modules` for import graph extraction (optional validation)
- [ ] C6: Integrate `cargo-modules orphans` for undocumented file detection

### Phase D: JSON IR + Cross-Language Split (Deliverables 2.1, 2.2, 2.4)

- [x] D1: Define `ModuleDoc[]` JSON IR schema (`archidoc-ir-schema.json`)
- [x] D2: Split into two crates: `archidoc-rust` (adapter) + `archidoc-engine` (generator)
- [x] D3: Adapter outputs JSON IR to stdout
- [x] D4: Core reads JSON IR from stdin (`--from-json` flag)
- [x] D5: Core reads JSON IR from file (`--from-json path/to/ir.json`)
- [x] D6: Bundled binary mode — adapter pipes directly to core (single `archidoc` command)
- [x] D7: Validate IR against schema before processing (`--validate-ir`)
- [x] D8: Write "How to Write a Language Adapter" guide
- [x] D9: (backfill) Negative validation tests — reject malformed IR (missing fields, invalid enum values like `"system"` for c4_level)
- [x] D10: (backfill) File-based IR consumption test — test `--from-json-file` (D5) through the DSL
- [x] D11: (backfill) Double roundtrip stability test — IR → core → IR produces identical IR (idempotency proof)

### Phase E: Project Template (Deliverable 1.2)

- [x] E1: Create template repo directory structure (docs/, tests/, src/, .github/, .claude/)
- [x] E2: Bundle annotation spec (1.1) into template
- [x] E3: Create GitHub Actions CI template with `archidoc --check` gate
- [x] E4: Create pre-commit hook configuration
- [x] E5: Create CLAUDE.md template for new projects using this methodology
- [x] E6: Create template README with 6-phase workflow documentation
- [x] E7: Include example annotated module files per language

### Phase F: TypeScript Adapter (Deliverable 2.3)

- [x] F1: Implement `index.ts` file discovery (walk directory tree)
- [x] F2: Parse `/** @c4 container */` and `/** @c4 component */` JSDoc blocks
- [x] F3: Parse `@c4 uses` JSDoc tags for relationship extraction
- [x] F4: Extract GoF pattern names from JSDoc content
- [x] F5: Build module paths in dot notation from directory hierarchy
- [ ] F6: Parse `import`/`export` statements for relationship extraction (deferred — `@c4 uses` is the primary mechanism)
- [x] F7: Emit JSON IR to stdout (same schema as Rust adapter)
- [x] F8: Package as npm module (`archidoc-ts`)

### Phase G: Claude Code Skills (Deliverables 3.2, 3.3, 3.4)

- [x] G1: Create S1 Architecture Compiler skill (`/self-doc-scaffold`, `/self-doc-compile`, `/self-doc-check`, `/self-doc-health`)
- [x] G2: Create S2 Architecture Sketcher skill (`/self-doc-sketch-context`, `/self-doc-sketch-containers`, `/self-doc-sketch-components`)
- [x] G3: Create Orchestrator skill (`/bdd-start`, `/bdd-status`)
- [x] G4: Orchestrator reads project artifacts to determine current phase
- [x] G5: Orchestrator routes to correct persona/skill based on phase

### Phase H: Pattern Validation (Deliverable 2.2 advanced)

- [x] H1: Implement `syn`-based structural heuristic for Observer pattern
- [x] H2: Implement `syn`-based structural heuristic for Strategy pattern
- [x] H3: Implement `syn`-based structural heuristic for Facade pattern
- [x] H4: Fitness function: `all_strategy_modules_define_a_trait()`
- [x] H5: Fitness function: `all_facade_modules_reexport_submodules()`
- [x] H6: Fitness function: `all_observer_modules_have_channels_or_callbacks()`
- [x] H7: Auto-promote pattern labels from `planned` → `verified` when heuristics pass

## Phase Dependencies

```
A ──→ B ──→ C ──→ D ──→ E
                  D ──→ F
                  E ──→ G
                  B ──→ H
```

- **A** (convention) is prerequisite for everything
- **B** (validation) and **C** (relationships) can run in parallel after A
- **D** (JSON IR split) depends on B+C being stable
- **E** (template) depends on D (packages the split tools)
- **F** (TS adapter) depends on D (needs the JSON IR schema)
- **G** (skills) depends on E (needs the template structure)
- **H** (pattern validation) depends on B (extends validation infrastructure)


## Commit Style

- Reference the deliverable and checklist item: `"Phase B: implement --check mode (B1, Deliverable 2.1)"`
- Keep commits small — one checklist item per commit when practical
